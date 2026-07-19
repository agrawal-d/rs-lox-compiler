#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, CStr, CString};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

static mut G_API: *const LoxFfiApi = std::ptr::null();

thread_local! {
    static STRINGS_RETAINER: std::cell::RefCell<Vec<CString>> = std::cell::RefCell::new(Vec::new());
}

unsafe fn lox_make_nil() -> LoxFfiValue {
    unsafe { ((*G_API).make_nil)() }
}

unsafe fn lox_make_string(s: &str) -> LoxFfiValue {
    let c_str = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
    let ptr = c_str.as_ptr();
    STRINGS_RETAINER.with(|retainer| {
        retainer.borrow_mut().push(c_str);
    });
    unsafe { ((*G_API).make_string)(ptr) }
}

unsafe fn lox_set_error(message: &str) {
    let c_str = CString::new(message).unwrap_or_else(|_| CString::new("").unwrap());
    unsafe { ((*G_API).set_error)(c_str.as_ptr()); }
}

unsafe fn c_to_str<'a>(ptr: *const c_char) -> &'a str {
    if ptr.is_null() {
        return "";
    }
    unsafe { CStr::from_ptr(ptr).to_str().unwrap_or("") }
}

fn clear_retainer() {
    STRINGS_RETAINER.with(|retainer| {
        retainer.borrow_mut().clear();
    });
}

unsafe fn json_to_ffi(val: &serde_json::Value) -> LoxFfiValue {
    match val {
        serde_json::Value::Null => lox_make_nil(),
        serde_json::Value::Bool(b) => {
            ((*G_API).make_bool)(*b)
        }
        serde_json::Value::Number(n) => {
            let num = n.as_f64().unwrap_or(0.0);
            ((*G_API).make_number)(num)
        }
        serde_json::Value::String(s) => {
            lox_make_string(s)
        }
        serde_json::Value::Array(arr) => {
            let mut ffi_elements = Vec::with_capacity(arr.len());
            for item in arr {
                ffi_elements.push(json_to_ffi(item));
            }
            ((*G_API).make_array)(ffi_elements.len() as i32, ffi_elements.as_ptr())
        }
        serde_json::Value::Object(map) => {
            let mut ffi_entries = Vec::with_capacity(map.len());
            for (k, v) in map {
                let key_ffi = lox_make_string(k);
                let val_ffi = json_to_ffi(v);
                ffi_entries.push(LoxFfiMapEntry {
                    key: key_ffi,
                    value: val_ffi,
                });
            }
            ((*G_API).make_map)(ffi_entries.len() as i32, ffi_entries.as_ptr())
        }
    }
}

unsafe fn ffi_to_json(val: &LoxFfiValue) -> serde_json::Value {
    match val.type_ {
        LoxValueType::VAL_NIL => serde_json::Value::Null,
        LoxValueType::VAL_BOOL => serde_json::Value::Bool(val.as_.boolean),
        LoxValueType::VAL_NUMBER => {
            if let Some(n) = serde_json::Number::from_f64(val.as_.number) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        }
        LoxValueType::VAL_STRING => {
            let s = c_to_str(val.as_.string);
            serde_json::Value::String(s.to_string())
        }
        LoxValueType::VAL_BUFFER => {
            let buf_ptr = val.as_.buffer;
            if buf_ptr.is_null() {
                serde_json::Value::Null
            } else {
                let buf = &*buf_ptr;
                let bytes_slice = std::slice::from_raw_parts(buf.bytes, buf.size as usize);
                let arr: Vec<serde_json::Value> = bytes_slice.iter()
                    .map(|&b| serde_json::Value::Number(serde_json::Number::from(b)))
                    .collect();
                serde_json::Value::Array(arr)
            }
        }
        LoxValueType::VAL_ARRAY => {
            let arr_ptr = val.as_.array;
            if arr_ptr.is_null() {
                serde_json::Value::Null
            } else {
                let arr = &*arr_ptr;
                let len = arr.length as usize;
                let mut items = Vec::with_capacity(len);
                for i in 0..len {
                    let item = &*arr.elements.add(i);
                    items.push(ffi_to_json(item));
                }
                serde_json::Value::Array(items)
            }
        }
        LoxValueType::VAL_MAP => {
            let map_ptr = val.as_.map;
            if map_ptr.is_null() {
                serde_json::Value::Object(serde_json::Map::new())
            } else {
                let ffi_map = &*map_ptr;
                let len = ffi_map.length as usize;
                let mut map = serde_json::Map::new();
                for i in 0..len {
                    let entry = &*ffi_map.entries.add(i);
                    let k_str = if entry.key.type_ == LoxValueType::VAL_STRING {
                        c_to_str(entry.key.as_.string).to_string()
                    } else {
                        format!("{:?}", entry.key.type_)
                    };
                    map.insert(k_str, ffi_to_json(&entry.value));
                }
                serde_json::Value::Object(map)
            }
        }
    }
}

extern "C" fn json_parse(arg_count: std::ffi::c_int, args: *const LoxFfiValue) -> LoxFfiValue {
    clear_retainer();

    if arg_count < 1 {
        unsafe { lox_set_error("json.parse requires 1 argument"); }
        return unsafe { lox_make_nil() };
    }
    let str_val = unsafe { &*args.offset(0) };
    if str_val.type_ != LoxValueType::VAL_STRING {
        unsafe { lox_set_error("Argument to json.parse must be a string"); }
        return unsafe { lox_make_nil() };
    }
    let s = unsafe { c_to_str(str_val.as_.string) };

    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(val) => unsafe { json_to_ffi(&val) },
        Err(e) => {
            let err_msg = format!("JSON parsing failed: {}", e);
            unsafe { lox_set_error(&err_msg); }
            unsafe { lox_make_nil() }
        }
    }
}

extern "C" fn json_stringify(arg_count: std::ffi::c_int, args: *const LoxFfiValue) -> LoxFfiValue {
    clear_retainer();

    if arg_count < 1 {
        unsafe { lox_set_error("json.stringify requires at least 1 argument"); }
        return unsafe { lox_make_nil() };
    }
    let val = unsafe { &*args.offset(0) };
    let json_val = unsafe { ffi_to_json(val) };

    match serde_json::to_string(&json_val) {
        Ok(s) => unsafe { lox_make_string(&s) },
        Err(e) => {
            let err_msg = format!("JSON stringification failed: {}", e);
            unsafe { lox_set_error(&err_msg); }
            unsafe { lox_make_nil() }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lox_module_init(api: *const LoxFfiApi) {
    unsafe {
        G_API = api;
        let define_fn = (*api).define_function_with_help;

        let parse_name = CString::new("parse").unwrap();
        let parse_help = CString::new("parse(json_str)\nParses a JSON string into Lox values.\nArguments:\n  json_str: String containing JSON.\nReturns: Lox value (Nil, Bool, Number, String, Array, or Object pair array).\nError Cases: Sets error if JSON is invalid.").unwrap();
        define_fn(parse_name.as_ptr(), 1, json_parse, parse_help.as_ptr());

        let stringify_name = CString::new("stringify").unwrap();
        let stringify_help = CString::new("stringify(value)\nConverts a Lox value to its JSON string representation.\nArguments:\n  value: Lox value (supporting Nil, Bool, Number, String, Array, or Object pair array).\nReturns: String containing JSON representation.\nError Cases: Sets error if stringification fails.").unwrap();
        define_fn(stringify_name.as_ptr(), 1, json_stringify, stringify_help.as_ptr());
    }
}
