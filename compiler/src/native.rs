use crate::{
    interner::Interner,
    value::{print_value, Value},
    xprintln, IMPORTS,
};
use std::fmt::Debug;
use web_time::SystemTime;

pub trait Callable: Debug {
    fn arity(&self) -> usize;
    fn call(&self, interner: &mut Interner, args: &[Value]) -> Value;
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

            fn call(&self, interner: &mut Interner, args: &[Value]) -> Value {
                $body(interner, &args)
            }

            fn name(&self) -> &str {
                stringify!($name)
            }
        }
    };
}

callable_struct!(Clock, 0, |_interner: &mut Interner, _args: &[Value]| {
    let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    return Value::Number(epoch.as_millis() as f64);
});

callable_struct!(Sleep, 1, |interner: &mut Interner, args: &[Value]| {
    if let Some(Value::Number(n)) = args.get(0) {
        std::thread::sleep(std::time::Duration::from_millis(*n as u64));
        return Value::Nil;
    } else {
        let error = "Expected number as argument to sleep";
        let strid = interner.intern(error);
        xprintln!("{}", error);
        return Value::Str(strid);
    }
});

callable_struct!(Print, 1, |interner: &mut Interner, args: &[Value]| {
    print_value(&args[0], interner);
    xprintln!("");
    Value::Nil
});

callable_struct!(ReadString, 1, |interner: &mut Interner, args: &[Value]| {
    match &args[0] {
        Value::Str(s) => {
            let prompt_text = interner.lookup(&s);
            let input = (IMPORTS.get().expect("Compiler not initialized").read_fn)(prompt_text.to_string());
            return Value::Str(interner.intern(&input));
        }
        _ => {
            let error = "Expected string as argument to read";
            let strid = interner.intern(error);
            xprintln!("{}", error);
            return Value::Str(strid);
        }
    }
});

callable_struct!(ReadNumber, 0, |interner: &mut Interner, _args: &[Value]| {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let len = input.len();
    input.truncate(len - 1);
    match input.parse::<f64>() {
        Ok(n) => Value::Number(n),
        Err(_) => {
            let error = "Failed to parse number";
            let strid = interner.intern(error);
            xprintln!("{}", error);
            Value::Str(strid)
        }
    }
});

callable_struct!(ReadBool, 0, |interner: &mut Interner, _args: &[Value]| {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let len = input.len();
    input.truncate(len - 1);
    match input.as_str() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            let error = "Failed to parse boolean";
            let strid = interner.intern(error);
            xprintln!("{}", error);
            Value::Str(strid)
        }
    }
});

callable_struct!(GetType, 1, |interner: &mut Interner, args: &[Value]| {
    return Value::Str(interner.intern(&format!("{}", args[0])));
});

callable_struct!(ToString, 1, |interner: &mut Interner, args: &[Value]| {
    return Value::Str(interner.intern(&format!("{}", args[0])));
});

callable_struct!(ToNumber, 1, |interner: &mut Interner, args: &[Value]| {
    match args[0] {
        Value::Number(n) => Value::Number(n),
        Value::Bool(b) => Value::Number(b as i8 as f64),
        Value::Str(s) => {
            let str = interner.lookup(&s);
            match str.parse::<f64>() {
                Ok(n) => Value::Number(n),
                Err(_) => {
                    let error = "Failed to parse number";
                    let strid = interner.intern(error);
                    xprintln!("{}", error);
                    Value::Str(strid)
                }
            }
        }
        _ => {
            let error = "Failed to convert value to number";
            let strid = interner.intern(error);
            xprintln!("{}", error);
            Value::Str(strid)
        }
    }
});

callable_struct!(StringAt, 2, |interner: &mut Interner, args: &[Value]| {
    match (&args[0], &args[1]) {
        (Value::Str(s), Value::Number(n)) => {
            let str = interner.lookup(&s);
            let index = *n as usize;
            if index < str.len() {
                let c = str.chars().nth(index).unwrap();
                Value::Str(interner.intern(&c.to_string()))
            } else {
                let error = "Index out of bounds";
                let strid = interner.intern(error);
                xprintln!("{}", error);
                Value::Str(strid)
            }
        }
        _ => {
            let error = "Expected string and number as arguments to string_at";
            let strid = interner.intern(error);
            xprintln!("{}", error);
            Value::Str(strid)
        }
    }
});

callable_struct!(StrLen, 1, |interner: &mut Interner, args: &[Value]| {
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
