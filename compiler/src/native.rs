#![allow(unused_variables)]

use crate::{
    interner::{Interner, StrId},
    value::{print_value, value_as_string, Value},
    vm::ERR_STRING,
    xprintln,
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

pub fn set_global_error(interner: &mut Interner, globals: &mut Globals, message: &str) {
    globals.insert(interner.intern(ERR_STRING), Value::Str(interner.intern(message)));
}

macro_rules! callable_struct {
    ($struct_name:ident, $arity:expr, $interner:ident: &mut Interner, $globals:ident: &mut Globals, $args:ident: &[Value], $body:block) => {
        #[derive(Debug, Default)]
        pub struct $struct_name;

        impl Callable for $struct_name {
            fn arity(&self) -> usize {
                $arity
            }

            fn call(&self, $interner: &mut Interner, $globals: &mut Globals, $args: &[Value]) -> Value {
                $body
            }

            fn name(&self) -> &str {
                stringify!($struct_name)
            }
        }
    };
}

callable_struct!(Clock, 0, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    Value::Number(epoch.as_millis() as f64)
});

callable_struct!(Sleep, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    if let Some(Value::Number(n)) = args.first() {
        std::thread::sleep(std::time::Duration::from_millis(*n as u64));
        Value::Nil
    } else {
        set_global_error(interner, globals, "Expected number as argument to sleep");
        Value::Nil
    }
});

callable_struct!(Print, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value],{
    print_value(&args[0], interner);
    xprintln!("");
    Value::Nil
});

// Arg is what the user gave
callable_struct!(ReadString, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Str(s) => {
            args[0].clone()
        }
        _ => {
            set_global_error(interner, globals, "Expected string as argument to read");
            Value::Nil
        }
    }
});

// Arg is what the user gave
callable_struct!(ReadNumber, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Str(user_input) => {
            let input = interner.lookup(user_input);
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
            Value::Nil
        }
    }
});

callable_struct!(ReadBool, 0, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(TypeOf, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    Value::Str(interner.intern(&format!("{}", args[0])))
});

callable_struct!(ToString, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    Value::Str(interner.intern(&value_as_string(&args[0], interner)))
});

callable_struct!(ToNumber, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(StringAt, 2, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match (&args[0], &args[1]) {
        (Value::Str(s), Value::Number(n)) => {
            let str = interner.lookup(s);
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

callable_struct!(StrLen, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Str(s) => {
            let str = interner.lookup(s);
            Value::Number(str.len() as f64)
        }
        _ => {
            set_global_error(interner, globals, "Expected string as argument to strlen");
            Value::Nil
        }
    }
});

callable_struct!(ArrLen, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Array(arr) => Value::Number(arr.borrow().len() as f64),
        _ => {
            set_global_error(interner, globals, "Expected array as argument to arrlen");
            Value::Nil
        }
    }
});

callable_struct!(Ceil, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.ceil()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to ceil");
            Value::Nil
        }
    }
});

callable_struct!(Floor, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.floor()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to floor");
            Value::Nil
        }
    }
});

callable_struct!(Abs, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.abs()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to abs");
            Value::Nil
        }
    }
});

callable_struct!(Sort, 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Array(arr) => {
            let mut arr = arr.borrow_mut();
            arr.sort_by(|a, b| match (a, b) {
                (Value::Number(a), Value::Number(b)) => a.partial_cmp(b).unwrap(),
                _ => {
                    set_global_error(interner, globals, "Expected array of numbers");
                    std::cmp::Ordering::Equal
                }
            });
            Value::Nil
        }
        _ => {
            set_global_error(interner, globals, "Expected array as argument to sort");
            Value::Nil
        }
    }
});

callable_struct!(IndexOf, 2, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match (&args[0], &args[1]) {
        (Value::Array(arr), value) => {
            let arr = arr.borrow();
            for (i, v) in arr.iter().enumerate() {
                if v == value {
                    return Value::Number(i as f64);
                }
            }
            Value::Number(arr.len() as f64)
        }
        _ => {
            set_global_error(interner, globals, "Expected array and value as arguments to find");
            Value::Nil
        }
    }
});

callable_struct!(Rand, 0, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    let num: u32 = rand::random();
    Value::Number(num as f64)
});
