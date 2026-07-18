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

extern "C" fn fetch_get(arg_count: std::ffi::c_int, args: *const LoxFfiValue) -> LoxFfiValue {
    clear_retainer();

    if arg_count < 1 {
        unsafe { lox_set_error("fetch.get requires at least 1 argument"); }
        return unsafe { lox_make_nil() };
    }
    let url_val = unsafe { &*args.offset(0) };
    if url_val.type_ != LoxValueType::VAL_STRING {
        unsafe { lox_set_error("Argument to fetch.get must be a string"); }
        return unsafe { lox_make_nil() };
    }
    let url = unsafe { c_to_str(url_val.as_.string) };

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new());

    match client.get(url).send() {
        Ok(resp) => {
            match resp.text() {
                Ok(text) => unsafe { lox_make_string(&text) },
                Err(e) => {
                    let err_msg = format!("Failed to read fetch body: {}", e);
                    unsafe { lox_set_error(&err_msg); }
                    unsafe { lox_make_nil() }
                }
            }
        }
        Err(e) => {
            let err_msg = format!("HTTP GET request failed: {}", e);
            unsafe { lox_set_error(&err_msg); }
            unsafe { lox_make_nil() }
        }
    }
}

extern "C" fn fetch_post(arg_count: std::ffi::c_int, args: *const LoxFfiValue) -> LoxFfiValue {
    clear_retainer();

    if arg_count < 2 {
        unsafe { lox_set_error("fetch.post requires url and body arguments"); }
        return unsafe { lox_make_nil() };
    }
    let url_val = unsafe { &*args.offset(0) };
    let body_val = unsafe { &*args.offset(1) };

    if url_val.type_ != LoxValueType::VAL_STRING || body_val.type_ != LoxValueType::VAL_STRING {
        unsafe { lox_set_error("Arguments to fetch.post must be strings"); }
        return unsafe { lox_make_nil() };
    }
    let url = unsafe { c_to_str(url_val.as_.string) };
    let body = unsafe { c_to_str(body_val.as_.string) };

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new());

    match client.post(url).body(body.to_string()).send() {
        Ok(resp) => {
            match resp.text() {
                Ok(text) => unsafe { lox_make_string(&text) },
                Err(e) => {
                    let err_msg = format!("Failed to read fetch body: {}", e);
                    unsafe { lox_set_error(&err_msg); }
                    unsafe { lox_make_nil() }
                }
            }
        }
        Err(e) => {
            let err_msg = format!("HTTP POST request failed: {}", e);
            unsafe { lox_set_error(&err_msg); }
            unsafe { lox_make_nil() }
        }
    }
}

extern "C" fn fetch_request(arg_count: std::ffi::c_int, args: *const LoxFfiValue) -> LoxFfiValue {
    clear_retainer();

    if arg_count < 4 {
        unsafe { lox_set_error("fetch.request requires url, method, headers, and body arguments"); }
        return unsafe { lox_make_nil() };
    }
    let url_val = unsafe { &*args.offset(0) };
    let method_val = unsafe { &*args.offset(1) };
    let headers_val = unsafe { &*args.offset(2) };
    let body_val = unsafe { &*args.offset(3) };

    if url_val.type_ != LoxValueType::VAL_STRING || method_val.type_ != LoxValueType::VAL_STRING {
        unsafe { lox_set_error("URL and Method must be strings"); }
        return unsafe { lox_make_nil() };
    }

    let url = unsafe { c_to_str(url_val.as_.string) };
    let method_str = unsafe { c_to_str(method_val.as_.string) };

    let method = match method_str.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => {
            unsafe { lox_set_error(&format!("Unsupported HTTP method: {}", method_str)); }
            return unsafe { lox_make_nil() };
        }
    };

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new());
    let mut req_builder = client.request(method, url);

    // Unpack headers array
    if headers_val.type_ == LoxValueType::VAL_ARRAY {
        let arr_ptr = unsafe { headers_val.as_.array };
        if !arr_ptr.is_null() {
            let arr = unsafe { &*arr_ptr };
            let len = arr.length as usize;
            let mut i = 0;
            while i + 1 < len {
                let key_val = unsafe { &*arr.elements.add(i) };
                let val_val = unsafe { &*arr.elements.add(i + 1) };
                if key_val.type_ == LoxValueType::VAL_STRING && val_val.type_ == LoxValueType::VAL_STRING {
                    let key = unsafe { c_to_str(key_val.as_.string) };
                    let val = unsafe { c_to_str(val_val.as_.string) };
                    req_builder = req_builder.header(key, val);
                }
                i += 2;
            }
        }
    }

    // Set body if present
    if body_val.type_ == LoxValueType::VAL_STRING {
        let body = unsafe { c_to_str(body_val.as_.string) };
        req_builder = req_builder.body(body.to_string());
    } else if body_val.type_ == LoxValueType::VAL_BUFFER {
        let buf_ptr = unsafe { body_val.as_.buffer };
        if !buf_ptr.is_null() {
            let buf = unsafe { &*buf_ptr };
            let bytes_slice = unsafe { std::slice::from_raw_parts(buf.bytes, buf.size as usize) };
            req_builder = req_builder.body(bytes_slice.to_vec());
        }
    }

    match req_builder.send() {
        Ok(resp) => {
            match resp.text() {
                Ok(text) => unsafe { lox_make_string(&text) },
                Err(e) => {
                    let err_msg = format!("Failed to read fetch body: {}", e);
                    unsafe { lox_set_error(&err_msg); }
                    unsafe { lox_make_nil() }
                }
            }
        }
        Err(e) => {
            let err_msg = format!("HTTP request failed: {}", e);
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

        let get_name = CString::new("get").unwrap();
        let get_help = CString::new("get(url)\nPerforms a synchronous HTTP GET request.\nArguments:\n  url: String representing URL.\nReturns: String response body, Nil on error.").unwrap();
        define_fn(get_name.as_ptr(), 1, fetch_get, get_help.as_ptr());

        let post_name = CString::new("post").unwrap();
        let post_help = CString::new("post(url, body)\nPerforms a synchronous HTTP POST request.\nArguments:\n  url: String representing URL.\n  body: String containing request body.\nReturns: String response body, Nil on error.").unwrap();
        define_fn(post_name.as_ptr(), 2, fetch_post, post_help.as_ptr());

        let req_name = CString::new("request").unwrap();
        let req_help = CString::new("request(url, method, headers, body)\nPerforms a synchronous HTTP request.\nArguments:\n  url: String representing URL.\n  method: String method (e.g. \"GET\", \"POST\", \"PUT\", \"DELETE\").\n  headers: Array of strings representing header key-value pairs (e.g. [\"Content-Type\", \"application/json\"]).\n  body: String or Buffer for request body.\nReturns: String response body, Nil on error.").unwrap();
        define_fn(req_name.as_ptr(), 4, fetch_request, req_help.as_ptr());
    }
}
