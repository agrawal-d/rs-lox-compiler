use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use crate::interner::Interner;
use crate::value::{Value, LoxMap};
use crate::native::{Callable, Globals, set_global_error};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LoxValueType {
    Nil,
    Bool,
    Number,
    String,
    Array,
    Buffer,
    Map,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LoxFfiValueUnion {
    pub boolean: bool,
    pub number: f64,
    pub string: *const c_char,
    pub array: *mut LoxFfiArray,
    pub buffer: *mut LoxFfiBuffer,
    pub map: *mut LoxFfiMap,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LoxFfiValue {
    pub typ: LoxValueType,
    pub as_val: LoxFfiValueUnion,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LoxFfiArray {
    pub elements: *mut LoxFfiValue,
    pub length: i32,
    pub capacity: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LoxFfiBuffer {
    pub bytes: *mut u8,
    pub size: i32,
    pub capacity: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LoxFfiMapEntry {
    pub key: LoxFfiValue,
    pub value: LoxFfiValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LoxFfiMap {
    pub entries: *mut LoxFfiMapEntry,
    pub length: i32,
    pub capacity: i32,
}

pub type LoxNativeFn = extern "C" fn(arg_count: i32, args: *const LoxFfiValue) -> LoxFfiValue;

#[repr(C)]
pub struct LoxFfiApi {
    pub define_function: extern "C" fn(name: *const c_char, arity: i32, func: LoxNativeFn),
    pub define_global: extern "C" fn(name: *const c_char, value: LoxFfiValue),
    pub make_nil: extern "C" fn() -> LoxFfiValue,
    pub make_bool: extern "C" fn(b: bool) -> LoxFfiValue,
    pub make_number: extern "C" fn(d: f64) -> LoxFfiValue,
    pub make_string: extern "C" fn(s: *const c_char) -> LoxFfiValue,
    pub make_array: extern "C" fn(length: i32, elements: *const LoxFfiValue) -> LoxFfiValue,
    pub make_buffer: extern "C" fn(size: i32, bytes: *const u8) -> LoxFfiValue,
    pub set_error: extern "C" fn(message: *const c_char),
    pub define_function_with_help: extern "C" fn(name: *const c_char, arity: i32, func: LoxNativeFn, help: *const c_char),
    pub make_map: extern "C" fn(length: i32, entries: *const LoxFfiMapEntry) -> LoxFfiValue,
}

trait FfiBorrow {}
impl<'a, T: ?Sized> FfiBorrow for std::cell::Ref<'a, T> {}
impl<'a, T: ?Sized> FfiBorrow for std::cell::RefMut<'a, T> {}

struct FfiCallContext<'a> {
    c_strings: Vec<CString>,
    arrays: Vec<Box<LoxFfiArray>>,
    array_elements: Vec<Vec<LoxFfiValue>>,
    buffers: Vec<Box<LoxFfiBuffer>>,
    maps: Vec<Box<LoxFfiMap>>,
    map_entries: Vec<Vec<LoxFfiMapEntry>>,
    borrows: Vec<Box<dyn FfiBorrow + 'a>>,
}

fn value_to_ffi<'a>(
    value: &'a Value,
    interner: &Interner,
    ctx: &mut FfiCallContext<'a>,
) -> LoxFfiValue {
    match value {
        Value::Nil => LoxFfiValue {
            typ: LoxValueType::Nil,
            as_val: LoxFfiValueUnion { number: 0.0 },
        },
        Value::Bool(b) => LoxFfiValue {
            typ: LoxValueType::Bool,
            as_val: LoxFfiValueUnion { boolean: *b },
        },
        Value::Number(n) => LoxFfiValue {
            typ: LoxValueType::Number,
            as_val: LoxFfiValueUnion { number: *n },
        },
        Value::Str(id) | Value::Identifier(id) => {
            let s = interner.lookup(id);
            let c_str = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
            let ptr = c_str.as_ptr();
            ctx.c_strings.push(c_str);
            LoxFfiValue {
                typ: LoxValueType::String,
                as_val: LoxFfiValueUnion { string: ptr },
            }
        }
        Value::Array(arr) => {
            let arr_borrow = arr.borrow();
            let slice_ptr = arr_borrow.as_slice() as *const [Value];
            let length = arr_borrow.len() as i32;

            ctx.borrows.push(Box::new(arr_borrow));

            let slice = unsafe { &*slice_ptr };
            let mut ffi_elements = Vec::with_capacity(slice.len());
            for item in slice.iter() {
                ffi_elements.push(value_to_ffi(item, interner, ctx));
            }
            let ptr = ffi_elements.as_mut_ptr();
            let capacity = ffi_elements.capacity() as i32;
            ctx.array_elements.push(ffi_elements);

            let mut ffi_arr = Box::new(LoxFfiArray {
                elements: ptr,
                length,
                capacity,
            });
            let arr_ptr = &mut *ffi_arr as *mut LoxFfiArray;
            ctx.arrays.push(ffi_arr);

            LoxFfiValue {
                typ: LoxValueType::Array,
                as_val: LoxFfiValueUnion { array: arr_ptr },
            }
        }
        Value::Buffer(buf) => {
            let borrow = buf.borrow_mut();
            let ptr = borrow.as_ptr() as *mut u8;
            let size = borrow.len() as i32;

            ctx.borrows.push(Box::new(borrow));

            let mut ffi_buf = Box::new(LoxFfiBuffer {
                bytes: ptr,
                size,
                capacity: size, // We are borrowing, so size == capacity
            });
            let buf_ptr = &mut *ffi_buf as *mut LoxFfiBuffer;
            ctx.buffers.push(ffi_buf);

            LoxFfiValue {
                typ: LoxValueType::Buffer,
                as_val: LoxFfiValueUnion { buffer: buf_ptr },
            }
        }
        Value::Map(m) => {
            let m_borrow = m.borrow();
            let map_ptr = &*m_borrow as *const LoxMap;
            let length = m_borrow.len() as i32;

            ctx.borrows.push(Box::new(m_borrow));

            let map_ref = unsafe { &*map_ptr };
            let mut ffi_entries = Vec::with_capacity(length as usize);
            for (k, v) in map_ref.iter() {
                ffi_entries.push(LoxFfiMapEntry {
                    key: value_to_ffi(k, interner, ctx),
                    value: value_to_ffi(v, interner, ctx),
                });
            }
            let ptr = ffi_entries.as_mut_ptr();
            let capacity = ffi_entries.capacity() as i32;
            ctx.map_entries.push(ffi_entries);

            let mut ffi_map = Box::new(LoxFfiMap {
                entries: ptr,
                length,
                capacity,
            });
            let map_ptr = &mut *ffi_map as *mut LoxFfiMap;
            ctx.maps.push(ffi_map);

            LoxFfiValue {
                typ: LoxValueType::Map,
                as_val: LoxFfiValueUnion { map: map_ptr },
            }
        }
        _ => LoxFfiValue {
            typ: LoxValueType::Nil,
            as_val: LoxFfiValueUnion { number: 0.0 },
        },
    }
}

fn ffi_to_value(ffi: &LoxFfiValue, interner: &mut Interner) -> Value {
    match ffi.typ {
        LoxValueType::Nil => Value::Nil,
        LoxValueType::Bool => Value::Bool(unsafe { ffi.as_val.boolean }),
        LoxValueType::Number => Value::Number(unsafe { ffi.as_val.number }),
        LoxValueType::String => {
            let ptr = unsafe { ffi.as_val.string };
            if ptr.is_null() {
                Value::Str(interner.intern(""))
            } else {
                let c_str = unsafe { CStr::from_ptr(ptr) };
                let s = c_str.to_str().unwrap_or("");
                Value::Str(interner.intern(s))
            }
        }
        LoxValueType::Array => {
            let ptr = unsafe { ffi.as_val.array };
            if ptr.is_null() {
                Value::Array(std::rc::Rc::new(std::cell::RefCell::new(Vec::new())))
            } else {
                let ffi_arr = unsafe { Box::from_raw(ptr) };
                let mut elements = Vec::with_capacity(ffi_arr.length as usize);
                if !ffi_arr.elements.is_null() && ffi_arr.length > 0 {
                    let vec = unsafe { Vec::from_raw_parts(ffi_arr.elements, ffi_arr.length as usize, ffi_arr.capacity as usize) };
                    for item in &vec {
                        elements.push(ffi_to_value(item, interner));
                    }
                }
                Value::Array(std::rc::Rc::new(std::cell::RefCell::new(elements)))
            }
        }
        LoxValueType::Buffer => {
            let ptr = unsafe { ffi.as_val.buffer };
            if ptr.is_null() {
                Value::Buffer(std::rc::Rc::new(std::cell::RefCell::new(Vec::new())))
            } else {
                let ffi_buf = unsafe { Box::from_raw(ptr) };
                let bytes = if ffi_buf.bytes.is_null() || ffi_buf.size <= 0 {
                    Vec::new()
                } else {
                    unsafe {
                        Vec::from_raw_parts(ffi_buf.bytes, ffi_buf.size as usize, ffi_buf.capacity as usize)
                    }
                };
                Value::Buffer(std::rc::Rc::new(std::cell::RefCell::new(bytes)))
            }
        }
        LoxValueType::Map => {
            let ptr = unsafe { ffi.as_val.map };
            if ptr.is_null() {
                Value::Map(std::rc::Rc::new(std::cell::RefCell::new(rustc_hash::FxHashMap::default())))
            } else {
                let ffi_map = unsafe { Box::from_raw(ptr) };
                let mut map = rustc_hash::FxHashMap::default();
                if !ffi_map.entries.is_null() && ffi_map.length > 0 {
                    let vec = unsafe { Vec::from_raw_parts(ffi_map.entries, ffi_map.length as usize, ffi_map.capacity as usize) };
                    for entry in &vec {
                        let k = ffi_to_value(&entry.key, interner);
                        let v = ffi_to_value(&entry.value, interner);
                        map.insert(k, v);
                    }
                }
                Value::Map(std::rc::Rc::new(std::cell::RefCell::new(map)))
            }
        }
    }
}

#[derive(Debug)]
pub struct FfiCallable {
    pub name: String,
    pub arity: usize,
    pub func: LoxNativeFn,
    pub help: Option<String>,
}

impl Callable for FfiCallable {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interner: &mut Interner, globals: &mut Globals, args: &[Value]) -> Value {
        let mut ctx = FfiCallContext {
            c_strings: Vec::new(),
            arrays: Vec::new(),
            array_elements: Vec::new(),
            buffers: Vec::new(),
            maps: Vec::new(),
            map_entries: Vec::new(),
            borrows: Vec::new(),
        };

        let mut ffi_args = Vec::with_capacity(args.len());
        for arg in args {
            ffi_args.push(value_to_ffi(arg, interner, &mut ctx));
        }

        CURRENT_ERROR.with(|err| {
            *err.borrow_mut() = None;
        });

        let result = (self.func)(args.len() as i32, ffi_args.as_ptr());

        let err_msg = CURRENT_ERROR.with(|err| err.borrow_mut().take());
        if let Some(msg) = err_msg {
            set_global_error(interner, globals, &msg);
            Value::Nil
        } else {
            ffi_to_value(&result, interner)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn help(&self) -> Option<String> {
        self.help.clone()
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct FfiRegistry {
    functions: Vec<(String, usize, LoxNativeFn, Option<String>)>,
    globals: Vec<(String, LoxFfiValue)>,
    error: Option<String>,
}

thread_local! {
    pub static CURRENT_ERROR: RefCell<Option<String>> = RefCell::new(None);
    #[cfg(not(target_arch = "wasm32"))]
    static CURRENT_REGISTRY: RefCell<Option<FfiRegistry>> = RefCell::new(None);
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_define_function(name: *const c_char, arity: i32, func: LoxNativeFn) {
    let name_str = unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("").to_string();
    CURRENT_REGISTRY.with(|reg| {
        if let Some(ref mut r) = *reg.borrow_mut() {
            r.functions.push((name_str, arity as usize, func, None));
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_define_function_with_help(name: *const c_char, arity: i32, func: LoxNativeFn, help: *const c_char) {
    let name_str = unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("").to_string();
    let help_str = if help.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(help) }.to_str().unwrap_or("").to_string())
    };
    CURRENT_REGISTRY.with(|reg| {
        if let Some(ref mut r) = *reg.borrow_mut() {
            r.functions.push((name_str, arity as usize, func, help_str));
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_define_global(name: *const c_char, value: LoxFfiValue) {
    let name_str = unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("").to_string();
    CURRENT_REGISTRY.with(|reg| {
        if let Some(ref mut r) = *reg.borrow_mut() {
            r.globals.push((name_str, value));
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_make_nil() -> LoxFfiValue {
    LoxFfiValue {
        typ: LoxValueType::Nil,
        as_val: LoxFfiValueUnion { number: 0.0 },
    }
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_make_bool(b: bool) -> LoxFfiValue {
    LoxFfiValue {
        typ: LoxValueType::Bool,
        as_val: LoxFfiValueUnion { boolean: b },
    }
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_make_number(d: f64) -> LoxFfiValue {
    LoxFfiValue {
        typ: LoxValueType::Number,
        as_val: LoxFfiValueUnion { number: d },
    }
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_make_string(s: *const c_char) -> LoxFfiValue {
    LoxFfiValue {
        typ: LoxValueType::String,
        as_val: LoxFfiValueUnion { string: s },
    }
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_make_array(length: i32, elements: *const LoxFfiValue) -> LoxFfiValue {
    let slice = unsafe { std::slice::from_raw_parts(elements, length as usize) };
    let mut vec = Vec::with_capacity(length as usize);
    vec.extend_from_slice(slice);
    let ptr = vec.as_mut_ptr();
    let capacity = vec.capacity() as i32;
    std::mem::forget(vec);

    let arr = Box::new(LoxFfiArray {
        elements: ptr,
        length,
        capacity,
    });

    LoxFfiValue {
        typ: LoxValueType::Array,
        as_val: LoxFfiValueUnion { array: Box::into_raw(arr) },
    }
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_make_buffer(size: i32, bytes: *const u8) -> LoxFfiValue {
    let mut vec = vec![0u8; size as usize];
    if !bytes.is_null() && size > 0 {
        unsafe {
            std::ptr::copy_nonoverlapping(bytes, vec.as_mut_ptr(), size as usize);
        }
    }
    let ptr = vec.as_mut_ptr();
    let capacity = vec.capacity() as i32;
    std::mem::forget(vec);

    let ffi_buf = Box::new(LoxFfiBuffer {
        bytes: ptr,
        size,
        capacity,
    });
    LoxFfiValue {
        typ: LoxValueType::Buffer,
        as_val: LoxFfiValueUnion { buffer: Box::into_raw(ffi_buf) },
    }
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_make_map(length: i32, entries: *const LoxFfiMapEntry) -> LoxFfiValue {
    let mut vec = Vec::with_capacity(length as usize);
    if !entries.is_null() && length > 0 {
        let slice = unsafe { std::slice::from_raw_parts(entries, length as usize) };
        vec.extend_from_slice(slice);
    }
    let ptr = vec.as_mut_ptr();
    let capacity = vec.capacity() as i32;
    std::mem::forget(vec);

    let map = Box::new(LoxFfiMap {
        entries: ptr,
        length,
        capacity,
    });

    LoxFfiValue {
        typ: LoxValueType::Map,
        as_val: LoxFfiValueUnion { map: Box::into_raw(map) },
    }
}

#[cfg(not(target_arch = "wasm32"))]
extern "C" fn api_set_error(message: *const c_char) {
    let msg_str = unsafe { CStr::from_ptr(message) }.to_str().unwrap_or("").to_string();
    CURRENT_REGISTRY.with(|reg| {
        if let Some(ref mut r) = *reg.borrow_mut() {
            r.error = Some(msg_str.clone());
        }
    });
    CURRENT_ERROR.with(|err| {
        *err.borrow_mut() = Some(msg_str);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn load_native_module(
    _path: &str,
    _alias: &str,
    _interner: &mut Interner,
    _globals: &mut Globals,
) -> Result<Box<dyn std::any::Any>, String> {
    Err("Dynamic modules are not supported in WASM environment.".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_native_module(
    path: &str,
    alias: &str,
    interner: &mut Interner,
    globals: &mut Globals,
) -> Result<libloading::Library, String> {
    let lib = unsafe {
        match libloading::Library::new(path) {
            Ok(l) => l,
            Err(e) => return Err(format!("Failed to load dynamic library '{}': {}", path, e)),
        }
    };

    let init_fn: libloading::Symbol<unsafe extern "C" fn(*const LoxFfiApi)> = unsafe {
        match lib.get(b"lox_module_init") {
            Ok(sym) => sym,
            Err(e) => return Err(format!("Symbol 'lox_module_init' not found in library '{}': {}", path, e)),
        }
    };

    let registry = FfiRegistry {
        functions: Vec::new(),
        globals: Vec::new(),
        error: None,
    };

    CURRENT_REGISTRY.with(|reg| {
        *reg.borrow_mut() = Some(registry);
    });

    let api = Box::into_raw(Box::new(LoxFfiApi {
        define_function: api_define_function,
        define_global: api_define_global,
        make_nil: api_make_nil,
        make_bool: api_make_bool,
        make_number: api_make_number,
        make_string: api_make_string,
        make_array: api_make_array,
        make_buffer: api_make_buffer,
        set_error: api_set_error,
        define_function_with_help: api_define_function_with_help,
        make_map: api_make_map,
    }));

    unsafe {
        init_fn(api);
    }

    let registry = CURRENT_REGISTRY.with(|reg| reg.borrow_mut().take()).unwrap();

    if let Some(err) = registry.error {
        return Err(format!("Module initialization error: {}", err));
    }

    // Register functions
    for (name, arity, func, help) in registry.functions {
        let prefixed_name = format!("{}.{}", alias, name);
        let name_id = interner.intern(&prefixed_name);
        let callable = std::rc::Rc::new(FfiCallable {
            name: prefixed_name,
            arity,
            func,
            help,
        });
        globals.insert(name_id, Value::NativeFunction(callable));
    }

    // Register globals
    for (name, ffi_val) in registry.globals {
        let prefixed_name = format!("{}.{}", alias, name);
        let name_id = interner.intern(&prefixed_name);
        let val = ffi_to_value(&ffi_val, interner);
        globals.insert(name_id, val);
    }

    // Register alias itself as a placeholder to allow help(alias)
    let alias_id = interner.intern(alias);
    globals.insert(alias_id, Value::Str(interner.intern(&format!("module:{}", alias))));

    Ok(lib)
}
