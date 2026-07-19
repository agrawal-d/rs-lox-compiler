use std::rc::Rc;
use std::cell::RefCell;
use serde_json::Value as JsonValue;

use compiler::callable_struct;
use compiler::interner::Interner;
use compiler::native::{Callable, Globals, set_global_error};
use compiler::value::{Value, value_as_string};

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

callable_struct!(Parse, "parse", 1, "parse(json_str)
Parses a JSON string into Lox values, Arrays, and Map objects.
Arguments:
  json_str: String containing valid JSON.
Returns: Lox value (Nil, Bool, Number, String, Array, or Map).
Error Cases: Sets error if JSON is invalid.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let json_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s),
        _ => { set_global_error(interner, globals, "Expected string argument for json.parse"); return Value::Nil; }
    };

    match serde_json::from_str::<JsonValue>(json_str) {
        Ok(parsed) => json_to_lox(&parsed, interner),
        Err(e) => {
            set_global_error(interner, globals, &format!("JSON parse failed: {}", e));
            Value::Nil
        }
    }
});

callable_struct!(Stringify, "stringify", 1, "stringify(value)
Converts a Lox value, Array, or Map to its JSON string representation.
Arguments:
  value: Any Lox value to serialize.
Returns: String containing JSON, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let json_val = lox_to_json(&args[0], interner);
    match serde_json::to_string(&json_val) {
        Ok(json_str) => Value::Str(interner.intern(&json_str)),
        Err(e) => {
            set_global_error(interner, globals, &format!("JSON stringify failed: {}", e));
            Value::Nil
        }
    }
});

pub fn register(interner: &mut Interner, globals: &mut Globals, alias: &str) {
    let funcs: &[(&str, Rc<dyn Callable>)] = &[
        ("parse", Rc::new(Parse)),
        ("stringify", Rc::new(Stringify)),
    ];

    for (name, callable) in funcs {
        let full_name = format!("{}.{}", alias, name);
        let id = interner.intern(&full_name);
        globals.insert(id, Value::NativeFunction(callable.clone()));
    }
}
