use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use serde_json::{Value as JsonValue, Map as JsonMap};

use compiler::callable_struct;
use compiler::interner::Interner;
use compiler::native::{Callable, Globals, set_global_error};
use compiler::value::{Value, value_as_string};

fn load_kv_store(path_str: &str) -> JsonMap<String, JsonValue> {
    if !Path::new(path_str).exists() {
        return JsonMap::new();
    }
    match fs::read_to_string(path_str) {
        Ok(content) => match serde_json::from_str::<JsonValue>(&content) {
            Ok(JsonValue::Object(map)) => map,
            _ => JsonMap::new(),
        },
        Err(_) => JsonMap::new(),
    }
}

fn save_kv_store(path_str: &str, map: &JsonMap<String, JsonValue>) -> bool {
    let json_val = JsonValue::Object(map.clone());
    match serde_json::to_string_pretty(&json_val) {
        Ok(json_str) => fs::write(path_str, json_str).is_ok(),
        Err(_) => false,
    }
}

fn json_to_lox(json: &JsonValue, interner: &mut Interner) -> Value {
    match json {
        JsonValue::Null => Value::Nil,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        JsonValue::String(s) => Value::Str(interner.intern(s)),
        JsonValue::Array(arr) => {
            let elements: Vec<Value> = arr.iter().map(|item| json_to_lox(item, interner)).collect();
            Value::Array(Rc::new(RefCell::new(elements)))
        }
        JsonValue::Object(map) => {
            let mut lox_map = rustc_hash::FxHashMap::default();
            for (k, v) in map {
                let key_val = Value::Str(interner.intern(k));
                let val_val = json_to_lox(v, interner);
                lox_map.insert(key_val, val_val);
            }
            Value::Map(Rc::new(RefCell::new(lox_map)))
        }
    }
}

fn lox_to_json(val: &Value, interner: &Interner) -> JsonValue {
    match val {
        Value::Nil => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Number(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                JsonValue::Number(num)
            } else {
                JsonValue::Null
            }
        }
        Value::Str(s) | Value::Identifier(s) => JsonValue::String(interner.lookup(s).to_string()),
        Value::Buffer(buf) => {
            let bytes = buf.borrow();
            let arr: Vec<JsonValue> = bytes.iter().map(|&b| JsonValue::Number(b.into())).collect();
            JsonValue::Array(arr)
        }
        Value::Array(arr) => {
            let borrow = arr.borrow();
            let items: Vec<JsonValue> = borrow.iter().map(|item| lox_to_json(item, interner)).collect();
            JsonValue::Array(items)
        }
        Value::Map(map) => {
            let borrow = map.borrow();
            let mut obj = serde_json::Map::new();
            for (k, v) in borrow.iter() {
                let k_str = match k {
                    Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
                    _ => value_as_string(k, interner),
                };
                obj.insert(k_str, lox_to_json(v, interner));
            }
            JsonValue::Object(obj)
        }
        _ => JsonValue::String(value_as_string(val, interner)),
    }
}

callable_struct!(Set, "set", 3, "set(file_path, key, value)
Persists a key-value entry to the specified KV store file path.
Arguments:
  file_path: String path of KV store file.
  key: String or value representing key name.
  value: Any Lox value to persist.
Returns: Bool (true on success).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let file_path = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string file_path for kv.set"); return Value::Nil; }
    };

    let key_str = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => value_as_string(&args[1], interner),
    };

    let json_val = lox_to_json(&args[2], interner);

    let mut store = load_kv_store(&file_path);
    store.insert(key_str, json_val);

    Value::Bool(save_kv_store(&file_path, &store))
});

callable_struct!(Get, "get", 2, "get(file_path, key, [default])
Reads a value by key from the specified KV store file path.
Arguments:
  file_path: String path of KV store file.
  key: String or value key name.
  default: (Optional) Fallback value if key is not found.
Returns: Stored Lox value or default/Nil.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let file_path = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string file_path for kv.get"); return Value::Nil; }
    };

    let key_str = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => value_as_string(&args[1], interner),
    };

    let default_val = if args.len() > 2 { args[2].clone() } else { Value::Nil };

    let store = load_kv_store(&file_path);
    match store.get(&key_str) {
        Some(j_val) => json_to_lox(j_val, interner),
        None => default_val,
    }
});

callable_struct!(Has, "has", 2, "has(file_path, key)
Checks if a key exists in the specified KV store file.
Arguments:
  file_path: String path of KV store file.
  key: String key name.
Returns: Bool.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let file_path = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string file_path for kv.has"); return Value::Nil; }
    };

    let key_str = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => value_as_string(&args[1], interner),
    };

    let store = load_kv_store(&file_path);
    Value::Bool(store.contains_key(&key_str))
});

callable_struct!(Delete, "delete", 2, "delete(file_path, key)
Removes a key from the specified KV store file.
Arguments:
  file_path: String path of KV store file.
  key: String key name.
Returns: Bool (true if removed).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let file_path = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string file_path for kv.delete"); return Value::Nil; }
    };

    let key_str = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => value_as_string(&args[1], interner),
    };

    let mut store = load_kv_store(&file_path);
    let removed = store.remove(&key_str).is_some();
    if removed {
        save_kv_store(&file_path, &store);
    }
    Value::Bool(removed)
});

callable_struct!(Keys, "keys", 1, "keys(file_path)
Returns an Array of all key strings stored in the specified KV file.
Arguments:
  file_path: String path of KV store file.
Returns: Array of key strings.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let file_path = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string file_path for kv.keys"); return Value::Nil; }
    };

    let store = load_kv_store(&file_path);
    let keys_vec: Vec<Value> = store.keys().map(|k| Value::Str(interner.intern(k))).collect();
    Value::Array(Rc::new(RefCell::new(keys_vec)))
});

callable_struct!(All, "all", 1, "all(file_path)
Returns a Map containing all key-value entries stored in the specified KV file.
Arguments:
  file_path: String path of KV store file.
Returns: Map of all key-value entries.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let file_path = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string file_path for kv.all"); return Value::Nil; }
    };

    let store = load_kv_store(&file_path);
    let mut lox_map = rustc_hash::FxHashMap::default();
    for (k, v) in store.iter() {
        let k_val = Value::Str(interner.intern(k));
        let v_val = json_to_lox(v, interner);
        lox_map.insert(k_val, v_val);
    }
    Value::Map(Rc::new(RefCell::new(lox_map)))
});

callable_struct!(Clear, "clear", 1, "clear(file_path)
Deletes all entries from the specified KV store file.
Arguments:
  file_path: String path of KV store file.
Returns: Bool (true on success).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let file_path = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string file_path for kv.clear"); return Value::Nil; }
    };

    let empty = JsonMap::new();
    Value::Bool(save_kv_store(&file_path, &empty))
});

pub fn register(interner: &mut Interner, globals: &mut Globals, alias: &str) {
    let funcs: &[(&str, Rc<dyn Callable>)] = &[
        ("set", Rc::new(Set)),
        ("get", Rc::new(Get)),
        ("has", Rc::new(Has)),
        ("delete", Rc::new(Delete)),
        ("keys", Rc::new(Keys)),
        ("all", Rc::new(All)),
        ("clear", Rc::new(Clear)),
    ];

    for (name, callable) in funcs {
        let full_name = format!("{}.{}", alias, name);
        let id = interner.intern(&full_name);
        globals.insert(id, Value::NativeFunction(callable.clone()));
    }
}
