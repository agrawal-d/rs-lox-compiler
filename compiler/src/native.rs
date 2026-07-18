#![allow(unused_variables)]

use crate::{
    interner::{Interner, StrId},
    value::{print_value, value_as_string, Value},
    vm::ERR_STRING,
    xclear, xprintln,
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
    ($struct_name:ident, $lox_name:expr, $arity:expr, $interner:ident: &mut Interner, $globals:ident: &mut Globals, $args:ident: &[Value], $body:block) => {
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
                $lox_name
            }
        }
    };
}

callable_struct!(Clock, "clock", 0, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    Value::Number(epoch.as_millis() as f64)
});

callable_struct!(Sleep, "sleep", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    // The VM intercepts the "sleep" call and awaits the async sleep hook before
    // this Callable::call is ever reached. This body is therefore unreachable in
    // normal execution, but we keep it valid as a fallback.
    if let Some(Value::Number(_)) = args.first() {
        Value::Nil
    } else {
        set_global_error(interner, globals, "Expected number as argument to sleep");
        Value::Nil
    }
});

callable_struct!(Print, "print", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value],{
    print_value(&args[0], interner);
    xprintln!("");
    Value::Nil
});

// Arg is what the user gave
callable_struct!(ReadString, "input", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(TypeOf, "typeof", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    Value::Str(interner.intern(&format!("{}", args[0])))
});

callable_struct!(StrCast, "str", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    Value::Str(interner.intern(&value_as_string(&args[0], interner)))
});

callable_struct!(IntCast, "int", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.trunc()),
        Value::Bool(b) => Value::Number(*b as i32 as f64),
        Value::Str(s) => {
            let str = interner.lookup(s);
            match str.parse::<f64>() {
                Ok(n) => Value::Number(n.trunc()),
                Err(err) => {
                    set_global_error(interner, globals, &format!("Failed to parse int: {}", err));
                    Value::Nil
                }
            }
        }
        _ => {
            set_global_error(interner, globals, "Expected number, bool, or string as argument to int");
            Value::Nil
        }
    }
});

callable_struct!(FloatCast, "float", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(*n),
        Value::Bool(b) => Value::Number(*b as i8 as f64),
        Value::Str(s) => {
            let str = interner.lookup(s);
            match str.parse::<f64>() {
                Ok(n) => Value::Number(n),
                Err(err) => {
                    set_global_error(interner, globals, &format!("Failed to parse float: {}", err));
                    Value::Nil
                }
            }
        }
        _ => {
            set_global_error(interner, globals, "Expected number, bool, or string as argument to float");
            Value::Nil
        }
    }
});

callable_struct!(BoolCast, "bool", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Nil => Value::Bool(false),
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => Value::Bool((*n - 0.0).abs() >= f64::EPSILON),
        Value::Str(s) => Value::Bool(!interner.lookup(s).is_empty()),
        Value::Array(arr) => Value::Bool(!arr.borrow().is_empty()),
        _ => Value::Bool(true),
    }
});

callable_struct!(StringAt, "stringat", 2, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(Len, "len", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Str(s) => {
            let str = interner.lookup(s);
            Value::Number(str.len() as f64)
        }
        Value::Array(arr) => Value::Number(arr.borrow().len() as f64),
        _ => {
            set_global_error(interner, globals, "Expected string or array as argument to len");
            Value::Nil
        }
    }
});

callable_struct!(Ceil, "ceil", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.ceil()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to ceil");
            Value::Nil
        }
    }
});

callable_struct!(Floor, "floor", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.floor()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to floor");
            Value::Nil
        }
    }
});

callable_struct!(Abs, "abs", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.abs()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to abs");
            Value::Nil
        }
    }
});

callable_struct!(Sort, "sort", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(IndexOf, "indexof", 2, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(Rand, "rand", 0, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    let num: u32 = rand::random();
    Value::Number(num as f64)
});

callable_struct!(Sin, "sin", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.sin()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to sin");
            Value::Nil
        }
    }
});

callable_struct!(Cos, "cos", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.cos()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to cos");
            Value::Nil
        }
    }
});

callable_struct!(Sqrt, "sqrt", 1, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.sqrt()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to sqrt");
            Value::Nil
        }
    }
});

callable_struct!(Pow, "pow", 2, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match (&args[0], &args[1]) {
        (Value::Number(base), Value::Number(exp)) => Value::Number(base.powf(*exp)),
        _ => {
            set_global_error(interner, globals, "Expected two numbers as arguments to pow");
            Value::Nil
        }
    }
});

callable_struct!(Pi, "pi", 0, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    Value::Number(std::f64::consts::PI)
});

// Printf: format string with variable number of arguments
// Usage: printf("Hello {0} you are {1} years old", name, age)
// or: printf("Hello {name} you are {age} years old") - requires variables in scope
#[derive(Debug, Default)]
pub struct Printf;

impl Callable for Printf {
    fn arity(&self) -> usize {
        1 // Minimum arity is 1 (format string)
    }

    fn call(&self, interner: &mut Interner, globals: &mut Globals, args: &[Value]) -> Value {
        if args.is_empty() {
            return Value::Nil;
        }

        let format_str = match &args[0] {
            Value::Str(id) => interner.lookup(id).to_string(),
            _ => {
                set_global_error(interner, globals, "First argument to printf must be a format string");
                return Value::Nil;
            }
        };

        let mut result = String::new();
        let mut chars = format_str.chars().peekable();
        let mut arg_index = 1;
        let mut in_brace = false;
        let mut brace_content = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    if in_brace {
                        // Double brace {{ inside expression - treat as literal {
                        brace_content.push(ch);
                    } else {
                        // Check for double opening brace
                        if chars.peek() == Some(&'{') {
                            // This is {{ - output a literal {
                            result.push('{');
                            chars.next(); // consume the second {
                        } else {
                            // Single { - start of expression
                            in_brace = true;
                            brace_content.clear();
                        }
                    }
                }
                '}' => {
                    if in_brace {
                        // Check for double closing brace
                        if brace_content.is_empty() && chars.peek() == Some(&'}') {
                            // This would be }}, but only if brace_content is empty
                            result.push('}');
                            chars.next(); // consume second }
                            in_brace = false;
                        } else {
                            // End of expression
                            let trimmed = brace_content.trim();

                            // Try as numeric index first
                            if let Ok(idx) = trimmed.parse::<usize>() {
                                let arg_pos = idx + 1; // +1 because first arg is format string
                                if arg_pos < args.len() {
                                    result.push_str(&value_as_string(&args[arg_pos], interner));
                                } else {
                                    result.push_str(&format!("{{index {}: out of bounds}}", idx));
                                }
                            } else if !trimmed.is_empty() {
                                // Try as variable name
                                let var_id = interner.intern(trimmed);
                                if let Some(var_value) = globals.get(&var_id) {
                                    result.push_str(&value_as_string(var_value, interner));
                                } else if arg_index < args.len() {
                                    // Fallback to positional argument
                                    result.push_str(&value_as_string(&args[arg_index], interner));
                                    arg_index += 1;
                                } else {
                                    result.push_str(&format!("{{undefined: {}}}", trimmed));
                                }
                            }
                            in_brace = false;
                            brace_content.clear();
                        }
                    } else {
                        // Closing brace without opening
                        if chars.peek() == Some(&'}') {
                            // This is }} - output a literal }
                            result.push('}');
                            chars.next(); // consume the second }
                        } else {
                            // Single } treated as literal
                            result.push(ch);
                        }
                    }
                }
                '\\' => {
                    // Handle escape sequences
                    if in_brace {
                        brace_content.push(ch);
                    } else {
                        if let Some(next_ch) = chars.next() {
                            match next_ch {
                                'n' => result.push('\n'),
                                't' => result.push('\t'),
                                'r' => result.push('\r'),
                                '\\' => result.push('\\'),
                                '"' => result.push('"'),
                                _ => {
                                    result.push('\\');
                                    result.push(next_ch);
                                }
                            }
                        }
                    }
                }
                _ => {
                    if in_brace {
                        brace_content.push(ch);
                    } else {
                        result.push(ch);
                    }
                }
            }
        }

        // Handle unclosed brace
        if in_brace {
            result.push('{');
            result.push_str(&brace_content);
        }

        xprintln!("{}", result);
        Value::Nil
    }

    fn name(&self) -> &str {
        "printf"
    }
}

callable_struct!(Clear, "clear", 0, interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    xclear!();
    Value::Nil
});
