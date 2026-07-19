use std::rc::Rc;
use std::cell::RefCell;
use reqwest::blocking::Client;

use compiler::callable_struct;
use compiler::interner::Interner;
use compiler::native::{Callable, Globals, set_global_error};
use compiler::value::Value;

callable_struct!(Get, "get", 1, "get(url)
Performs a synchronous HTTP GET request and returns response body as text.
Arguments:
  url: String representing URL.
Returns: String response body, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let url_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s),
        _ => { set_global_error(interner, globals, "Expected string URL for fetch.get"); return Value::Nil; }
    };

    let client = Client::new();
    match client.get(url_str).send() {
        Ok(resp) => match resp.text() {
            Ok(body) => Value::Str(interner.intern(&body)),
            Err(e) => {
                set_global_error(interner, globals, &format!("Failed to read HTTP response text: {}", e));
                Value::Nil
            }
        },
        Err(e) => {
            set_global_error(interner, globals, &format!("HTTP GET request to '{}' failed: {}", url_str, e));
            Value::Nil
        }
    }
});

callable_struct!(GetBuf, "get_buf", 1, "get_buf(url)
Performs a synchronous HTTP GET request and returns response body as a zero-copy Buffer.
Arguments:
  url: String representing URL.
Returns: Buffer containing binary response bytes, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let url_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s),
        _ => { set_global_error(interner, globals, "Expected string URL for fetch.get_buf"); return Value::Nil; }
    };

    let client = Client::new();
    match client.get(url_str).send() {
        Ok(resp) => match resp.bytes() {
            Ok(bytes) => Value::Buffer(Rc::new(RefCell::new(bytes.to_vec()))),
            Err(e) => {
                set_global_error(interner, globals, &format!("Failed to read HTTP response bytes: {}", e));
                Value::Nil
            }
        },
        Err(e) => {
            set_global_error(interner, globals, &format!("HTTP GET request to '{}' failed: {}", url_str, e));
            Value::Nil
        }
    }
});

callable_struct!(Post, "post", 2, "post(url, body)
Performs a synchronous HTTP POST request with a String or Buffer body.
Arguments:
  url: String representing URL.
  body: String or Buffer for request body.
Returns: String response body, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let url_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string URL for fetch.post"); return Value::Nil; }
    };

    let client = Client::new();
    let req = client.post(&url_str);

    let req = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => req.body(interner.lookup(s).to_string()),
        Value::Buffer(buf) => req.body(buf.borrow().clone()),
        _ => { set_global_error(interner, globals, "Expected string or buffer body for fetch.post"); return Value::Nil; }
    };

    match req.send() {
        Ok(resp) => match resp.text() {
            Ok(body) => Value::Str(interner.intern(&body)),
            Err(e) => {
                set_global_error(interner, globals, &format!("Failed to read HTTP POST response: {}", e));
                Value::Nil
            }
        },
        Err(e) => {
            set_global_error(interner, globals, &format!("HTTP POST request to '{}' failed: {}", url_str, e));
            Value::Nil
        }
    }
});

callable_struct!(Request, "request", 4, "request(url, method, headers, body)
Performs a custom HTTP request with custom method, headers (Map or Array), and body payload.
Arguments:
  url: String representing URL.
  method: String method (e.g. \"GET\", \"POST\", \"PUT\", \"DELETE\", \"PATCH\").
  headers: Map or Array of key-value pairs (e.g. {\"Content-Type\": \"application/json\"}).
  body: (Optional) String or Buffer body payload.
Returns: String response body, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let url_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string URL for fetch.request"); return Value::Nil; }
    };

    let method_str = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_uppercase(),
        _ => { set_global_error(interner, globals, "Expected string method for fetch.request"); return Value::Nil; }
    };

    let method = match reqwest::Method::from_bytes(method_str.as_bytes()) {
        Ok(m) => m,
        Err(_) => { set_global_error(interner, globals, &format!("Invalid HTTP method: {}", method_str)); return Value::Nil; }
    };

    let client = Client::new();
    let mut req = client.request(method, &url_str);

    // Process headers
    match &args[2] {
        Value::Map(map) => {
            let borrow = map.borrow();
            for (k, v) in borrow.iter() {
                let k_str = match k {
                    Value::Str(id) | Value::Identifier(id) => interner.lookup(id),
                    _ => continue,
                };
                let v_str = match v {
                    Value::Str(id) | Value::Identifier(id) => interner.lookup(id).to_string(),
                    _ => compiler::value::value_as_string(v, interner),
                };
                req = req.header(k_str, v_str);
            }
        }
        Value::Array(arr) => {
            let borrow = arr.borrow();
            for item in borrow.iter() {
                if let Value::Array(pair) = item {
                    let p_borrow = pair.borrow();
                    if p_borrow.len() == 2 {
                        let k_str = match &p_borrow[0] {
                            Value::Str(id) | Value::Identifier(id) => interner.lookup(id),
                            _ => continue,
                        };
                        let v_str = match &p_borrow[1] {
                            Value::Str(id) | Value::Identifier(id) => interner.lookup(id).to_string(),
                            _ => compiler::value::value_as_string(&p_borrow[1], interner),
                        };
                        req = req.header(k_str, v_str);
                    }
                }
            }
        }
        Value::Nil => {}
        _ => {}
    }

    // Process body
    req = match &args[3] {
        Value::Str(s) | Value::Identifier(s) => req.body(interner.lookup(s).to_string()),
        Value::Buffer(buf) => req.body(buf.borrow().clone()),
        Value::Nil => req,
        _ => req,
    };

    match req.send() {
        Ok(resp) => match resp.text() {
            Ok(body) => Value::Str(interner.intern(&body)),
            Err(e) => {
                set_global_error(interner, globals, &format!("Failed to read HTTP request response: {}", e));
                Value::Nil
            }
        },
        Err(e) => {
            set_global_error(interner, globals, &format!("HTTP request to '{}' failed: {}", url_str, e));
            Value::Nil
        }
    }
});

pub fn register(interner: &mut Interner, globals: &mut Globals, alias: &str) {
    let funcs: &[(&str, Rc<dyn Callable>)] = &[
        ("get", Rc::new(Get)),
        ("get_buf", Rc::new(GetBuf)),
        ("post", Rc::new(Post)),
        ("request", Rc::new(Request)),
    ];

    for (name, callable) in funcs {
        let full_name = format!("{}.{}", alias, name);
        let id = interner.intern(&full_name);
        globals.insert(id, Value::NativeFunction(callable.clone()));
    }
}
