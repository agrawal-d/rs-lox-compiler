#![allow(unused_variables)]

use crate::{
    interner::{Interner, StrId},
    value::{print_value, value_as_string, Value},
    vm::ERR_STRING,
    xprintln, IMPORTS,
};
use rustc_hash::FxHashMap;
use std::fmt::Debug;
use web_time::SystemTime;

type Globals = FxHashMap<StrId, Value>;

pub trait Callable: Debug {
    fn arity(&self) -> usize;
    fn call(&self, interner: &mut Interner, globals: &mut Globals, args: &[Value]) -> Value;
    fn name(&self) -> &str;
}

macro_rules! callable_struct {
    ($name:ident, $arity:expr, $body:expr) => {
        #[derive(Debug, Default)]
        pub struct $name;

        impl Callable for $name {
            fn arity(&self) -> usize {
                $arity
            }

            fn call(&self, interner: &mut Interner, globals: &mut Globals, args: &[Value]) -> Value {
                $body(interner, globals, &args)
            }

            fn name(&self) -> &str {
                stringify!($name)
            }
        }
    };
}

fn set_global_error(interner: &mut Interner, globals: &mut Globals, message: &str) {
    globals.insert(interner.intern(ERR_STRING), Value::Str(interner.intern(message)));
}

callable_struct!(Clock, 0, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    return Value::Number(epoch.as_millis() as f64);
});

callable_struct!(Sleep, 1, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    if let Some(Value::Number(n)) = args.get(0) {
        std::thread::sleep(std::time::Duration::from_millis(*n as u64));
        return Value::Nil;
    } else {
        set_global_error(interner, globals, "Expected number as argument to sleep");
        return Value::Nil;
    }
});

callable_struct!(Print, 1, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    print_value(&args[0], interner);
    xprintln!("");
    Value::Nil
});

callable_struct!(ReadString, 1, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    match &args[0] {
        Value::Str(s) => {
            let prompt_text = interner.lookup(&s);
            let input = (IMPORTS.get().expect("Compiler not initialized").read_fn)(prompt_text.to_string());
            return Value::Str(interner.intern(&input));
        }
        _ => {
            set_global_error(interner, globals, "Expected string as argument to read");
            return Value::Nil;
        }
    }
});

callable_struct!(ReadNumber, 1, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    match &args[0] {
        Value::Str(s) => {
            let prompt_text = interner.lookup(&s);
            let input = (IMPORTS.get().expect("Compiler not initialized").read_fn)(prompt_text.to_string());
            match input.parse::<f64>() {
                Ok(n) => Value::Number(n),
                Err(err) => {
                    set_global_error(interner, globals, &format!("Failed to parse number: {}", err));
                    Value::Nil
                }
            }
        }
        _ => {
            set_global_error(interner, globals, "Expected string as argument to read");
            return Value::Nil;
        }
    }
});

callable_struct!(ReadBool, 0, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let len = input.len();
    input.truncate(len - 1);
    match input.as_str() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            set_global_error(interner, globals, "Expected 'true' or 'false'");
            Value::Nil
        }
    }
});

callable_struct!(GetType, 1, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    return Value::Str(interner.intern(&format!("{}", args[0])));
});

callable_struct!(ToString, 1, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    return Value::Str(interner.intern(&value_as_string(&args[0], interner)));
});

callable_struct!(ToNumber, 1, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    match args[0] {
        Value::Number(n) => Value::Number(n),
        Value::Bool(b) => Value::Number(b as i8 as f64),
        Value::Str(s) => {
            let str = interner.lookup(&s);
            match str.parse::<f64>() {
                Ok(n) => Value::Number(n),
                Err(err) => {
                    set_global_error(interner, globals, &format!("Failed to parse number: {}", err));
                    Value::Nil
                }
            }
        }
        _ => {
            set_global_error(interner, globals, "Expected number, bool, or string as argument to tonumber");
            Value::Nil
        }
    }
});

callable_struct!(StringAt, 2, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    match (&args[0], &args[1]) {
        (Value::Str(s), Value::Number(n)) => {
            let str = interner.lookup(&s);
            let index = *n as usize;
            if index < str.len() {
                let c = str.chars().nth(index).unwrap();
                Value::Str(interner.intern(&c.to_string()))
            } else {
                set_global_error(interner, globals, "Index out of bounds");
                Value::Nil
            }
        }
        _ => {
            set_global_error(interner, globals, "Expected string and number as arguments to stringat");
            Value::Nil
        }
    }
});

callable_struct!(StrLen, 1, |interner: &mut Interner, globals: &mut Globals, args: &[Value]| {
    match &args[0] {
        Value::Str(s) => {
            let str = interner.lookup(&s);
            Value::Number(str.len() as f64)
        }
        _ => {
            let error = "Expected string as argument to strlen";
            let strid = interner.intern(error);
            xprintln!("{}", error);
            Value::Str(strid)
        }
    }
});
