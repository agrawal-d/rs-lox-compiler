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

pub type Globals = FxHashMap<StrId, Value>;

pub trait Callable: Debug {
    fn arity(&self) -> usize;
    fn call(&self, interner: &mut Interner, globals: &mut Globals, args: &[Value]) -> Value;
    fn name(&self) -> &str;
    fn help(&self) -> Option<String> {
        None
    }
}

pub fn set_global_error(interner: &mut Interner, globals: &mut Globals, message: &str) {
    globals.insert(interner.intern(ERR_STRING), Value::Str(interner.intern(message)));
}

macro_rules! callable_struct {
    ($struct_name:ident, $lox_name:expr, $arity:expr, $help:expr, $interner:ident: &mut Interner, $globals:ident: &mut Globals, $args:ident: &[Value], $body:block) => {
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

            fn help(&self) -> Option<String> {
                Some($help.to_string())
            }
        }
    };
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

callable_struct!(Clock, "clock", 0, "clock()
Returns the current system time in milliseconds since the UNIX epoch.
Arguments: None.
Returns: Number representing epoch milliseconds.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    Value::Number(epoch.as_millis() as f64)
});

callable_struct!(Sleep, "sleep", 1, "sleep(ms)
Suspends execution of the current script for the specified duration in milliseconds.
Arguments:
  ms: Number of milliseconds to sleep.
Returns: Nil.
Error Cases: Sets error if argument is not a number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(Print, "print", 1, "print(val)
Prints the string representation of the value to stdout.
Arguments:
  val: Any value to print.
Returns: Nil.",
interner: &mut Interner, globals: &mut Globals, args: &[Value],{
    print_value(&args[0], interner);
    xprintln!("");
    Value::Nil
});

// Arg is what the user gave
callable_struct!(ReadString, "input", 1, "input(prompt)
Prints prompt and reads a line of input from stdin.
Arguments:
  prompt: String to display before input.
Returns: String containing the read line.
Error Cases: Sets error if argument is not a string.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(TypeOf, "typeof", 1, "typeof(value)
Returns the type of the given value as a string.
Arguments:
  value: Any value to inspect.
Returns: String (e.g. \"Number\", \"String\", \"Array\", \"Buffer\").",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    Value::Str(interner.intern(&format!("{}", args[0])))
});

callable_struct!(StrCast, "str", 1, "str(value)
Converts the given value to a string. If the value is a Buffer, decodes it as a UTF-8 string.
Arguments:
  value: Any value to convert.
Returns: String value.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Buffer(buf) => {
            let bytes = buf.borrow();
            let s = String::from_utf8_lossy(&bytes).into_owned();
            Value::Str(interner.intern(&s))
        }
        _ => Value::Str(interner.intern(&value_as_string(&args[0], interner)))
    }
});

callable_struct!(BufCast, "buf", 1, "buf(arg)
Creates an FFI-compatible Buffer (array of bytes).
Arguments:
  arg: Can be:
       - Number: Allocates a new buffer of specified size filled with 0s.
       - String: Encodes the string into its raw UTF-8 bytes.
       - Array: Converts a numeric array into a buffer of bytes.
       - Buffer: Clones the buffer.
Returns: Buffer object.
Error Cases: Sets error if argument is not supported.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    use std::rc::Rc;
    use std::cell::RefCell;
    match &args[0] {
        Value::Number(n) => {
            let size = *n as usize;
            Value::Buffer(Rc::new(RefCell::new(vec![0; size])))
        }
        Value::Str(s) | Value::Identifier(s) => {
            let str_val = interner.lookup(s);
            Value::Buffer(Rc::new(RefCell::new(str_val.as_bytes().to_vec())))
        }
        Value::Array(arr) => {
            let arr_borrow = arr.borrow();
            let mut bytes = Vec::with_capacity(arr_borrow.len());
            for item in arr_borrow.iter() {
                match item {
                    Value::Number(n) => bytes.push(*n as u8),
                    _ => bytes.push(0),
                }
            }
            Value::Buffer(Rc::new(RefCell::new(bytes)))
        }
        Value::Buffer(buf) => {
            let bytes = buf.borrow().clone();
            Value::Buffer(Rc::new(RefCell::new(bytes)))
        }
        _ => {
            set_global_error(interner, globals, "Expected number, string, array, or buffer as argument to buf");
            Value::Nil
        }
    }
});

callable_struct!(ChrCast, "chr", 1, "chr(arg)
If arg is a string, gets the ASCII/byte value (0-255) of the first character.
If arg is a number, converts the byte value to its equivalent ASCII character string.
Arguments:
  arg: String or Number (0-255).
Returns: Number, String, or Nil.
Error Cases: Sets error if argument is not a string/number or if number is out of bounds [0, 255].",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Str(id) | Value::Identifier(id) => {
            let s = interner.lookup(id);
            if s.is_empty() {
                Value::Nil
            } else {
                let first_byte = s.as_bytes()[0];
                Value::Number(first_byte as f64)
            }
        }
        Value::Number(n) => {
            let byte_val = *n as i64;
            if byte_val < 0 || byte_val > 255 {
                set_global_error(interner, globals, "Argument to chr must be a byte value between 0 and 255");
                Value::Nil
            } else {
                let ch = byte_val as u8 as char;
                Value::Str(interner.intern(&ch.to_string()))
            }
        }
        _ => {
            set_global_error(interner, globals, "Expected string or number as argument to chr");
            Value::Nil
        }
    }
});

callable_struct!(IntCast, "int", 1, "int(val)
Truncates or parses the value to an integer.
Arguments:
  val: Number, Bool, or String to parse.
Returns: Number representing the integer.
Error Cases: Sets error if parsing fails or argument type is invalid.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(FloatCast, "float", 1, "float(val)
Parses the string value as a floating point number.
Arguments:
  val: String to parse.
Returns: Number representing the float value.
Error Cases: Sets error if parsing fails.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(BoolCast, "bool", 1, "bool(val)
Converts the value to a boolean.
Arguments:
  val: Any value to cast.
Returns: Bool (false if Nil or false, true otherwise).",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Nil => Value::Bool(false),
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => Value::Bool((*n - 0.0).abs() >= f64::EPSILON),
        Value::Str(s) => Value::Bool(!interner.lookup(s).is_empty()),
        Value::Array(arr) => Value::Bool(!arr.borrow().is_empty()),
        _ => Value::Bool(true),
    }
});

callable_struct!(StringAt, "stringat", 2, "stringat(str, index)
Returns the single-character string at the specified 0-based index.
Arguments:
  str: String to index.
  index: Number representing the 0-based position.
Returns: Single-character String, or Nil if index is out of bounds.
Error Cases: Sets error if arguments are invalid.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match (&args[0], &args[1]) {
        (Value::Str(s), Value::Number(n)) => {
            let str = interner.lookup(s);
            let index = *n as usize;
            if let Some(c) = str.chars().nth(index) {
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

callable_struct!(Len, "len", 1, "len(val)
Returns the length/size of the given value.
Arguments:
  val: String, Array, or Buffer to inspect.
Returns: Number representing the length.
Error Cases: Sets error if argument type is not supported.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Str(s) => {
            let str = interner.lookup(s);
            Value::Number(str.chars().count() as f64)
        }
        Value::Array(arr) => Value::Number(arr.borrow().len() as f64),
        Value::Buffer(buf) => Value::Number(buf.borrow().len() as f64),
        _ => {
            set_global_error(interner, globals, "Expected string, array, or buffer as argument to len");
            Value::Nil
        }
    }
});

callable_struct!(Ceil, "ceil", 1, "ceil(x)
Returns the smallest integer greater than or equal to x.
Arguments:
  x: Number to round up.
Returns: Number representing the ceiling value.
Error Cases: Sets error if argument is not a number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.ceil()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to ceil");
            Value::Nil
        }
    }
});

callable_struct!(Floor, "floor", 1, "floor(x)
Returns the largest integer less than or equal to x.
Arguments:
  x: Number to round down.
Returns: Number representing the floor value.
Error Cases: Sets error if argument is not a number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.floor()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to floor");
            Value::Nil
        }
    }
});

callable_struct!(Abs, "abs", 1, "abs(x)
Returns the absolute value of x.
Arguments:
  x: Number to inspect.
Returns: Number representing the absolute value.
Error Cases: Sets error if argument is not a number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.abs()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to abs");
            Value::Nil
        }
    }
});

callable_struct!(Sort, "sort", 1, "sort(arr)
Sorts the array of numbers in-place in ascending order.
Arguments:
  arr: Array of numbers to sort.
Returns: Nil.
Error Cases: Sets error if argument is not an array.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(IndexOf, "indexof", 2, "indexof(arr, val)
Returns the first index of the value in the array, or the array length if not found.
Arguments:
  arr: Array to search.
  val: Value to search for.
Returns: Number representing the index, or the array length.
Error Cases: Sets error if first argument is not an array.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

callable_struct!(Rand, "rand", 0, "rand()
Returns a pseudo-random floating point number in [0.0, 1.0).
Arguments: None.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    let num: u32 = rand::random();
    Value::Number(num as f64)
});

callable_struct!(Sin, "sin", 1, "sin(x)
Calculates the sine of the angle in radians.
Arguments:
  x: Number representing the angle in radians.
Returns: Number representing the sine value.
Error Cases: Sets error if argument is not a number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.sin()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to sin");
            Value::Nil
        }
    }
});

callable_struct!(Cos, "cos", 1, "cos(x)
Calculates the cosine of the angle in radians.
Arguments:
  x: Number representing the angle in radians.
Returns: Number representing the cosine value.
Error Cases: Sets error if argument is not a number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.cos()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to cos");
            Value::Nil
        }
    }
});

callable_struct!(Sqrt, "sqrt", 1, "sqrt(x)
Calculates the square root of x.
Arguments:
  x: Non-negative Number.
Returns: Number representing the square root.
Error Cases: Sets error if argument is not a number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match &args[0] {
        Value::Number(n) => Value::Number(n.sqrt()),
        _ => {
            set_global_error(interner, globals, "Expected number as argument to sqrt");
            Value::Nil
        }
    }
});

callable_struct!(Pow, "pow", 2, "pow(base, exp)
Calculates base raised to the exponent power.
Arguments:
  base: Number representing the base.
  exp: Number representing the exponent.
Returns: Number.
Error Cases: Sets error if arguments are not numbers.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    match (&args[0], &args[1]) {
        (Value::Number(base), Value::Number(exp)) => Value::Number(base.powf(*exp)),
        _ => {
            set_global_error(interner, globals, "Expected two numbers as arguments to pow");
            Value::Nil
        }
    }
});

callable_struct!(Pi, "pi", 0, "pi()
Returns the mathematical constant pi.
Arguments: None.
Returns: Number representing pi (3.14159...).",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
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

    fn help(&self) -> Option<String> {
        Some("printf(format, ...)
Prints formatted text to stdout. Uses Python-style {} indexing and C-style escape characters.
Arguments:
  format: String containing format slots.
  ...: Positional arguments corresponding to format slots.
Returns: Nil.
Error Cases: Sets error if first argument is not a string.".to_string())
    }
}

callable_struct!(Clear, "clear", 0, "clear()
Clears the terminal screen.
Arguments: None.
Returns: Nil.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    xclear!();
    Value::Nil
});

callable_struct!(HelpCast, "help", 1, "help(fn_or_name)
Prints documentation, signature, and usage instructions for the given function.
Arguments:
  fn_or_name: Function object or a string name of a function (e.g. \"clock\" or \"math.sin\").
Returns: Nil.",
interner: &mut Interner, globals: &mut Globals, args: &[Value] ,{
    let mut resolved_callable: Option<(String, usize, Option<String>)> = None;

    match &args[0] {
        Value::NativeFunction(c) => {
            resolved_callable = Some((c.name().to_string(), c.arity(), c.help()));
        }
        Value::Function(idx) => {
            crate::vm::RUNNING_FUNCTIONS.with(|funcs| {
                if let Some(ptr) = *funcs.borrow() {
                    let functions = unsafe { &*ptr };
                    if *idx < functions.len() {
                        let f = &functions[*idx];
                        let name = f.name.map(|id| interner.lookup(&id).to_string()).unwrap_or_else(|| "anonymous".to_string());
                        resolved_callable = Some((name, f.arity, f.help.clone()));
                    }
                }
            });
        }
        Value::BoundMethod { instance: _, method_idx } => {
            crate::vm::RUNNING_FUNCTIONS.with(|funcs| {
                if let Some(ptr) = *funcs.borrow() {
                    let functions = unsafe { &*ptr };
                    if *method_idx < functions.len() {
                        let f = &functions[*method_idx];
                        let name = f.name.map(|id| interner.lookup(&id).to_string()).unwrap_or_else(|| "anonymous".to_string());
                        resolved_callable = Some((name, f.arity, f.help.clone()));
                    }
                }
            });
        }
        Value::Str(id) | Value::Identifier(id) => {
            let name_str = interner.lookup(id).to_string();
            if let Some(val) = globals.get(id) {
                match val {
                    Value::NativeFunction(c) => {
                        resolved_callable = Some((c.name().to_string(), c.arity(), c.help()));
                    }
                    Value::Function(idx) => {
                        crate::vm::RUNNING_FUNCTIONS.with(|funcs| {
                            if let Some(ptr) = *funcs.borrow() {
                                let functions = unsafe { &*ptr };
                                if *idx < functions.len() {
                                    let f = &functions[*idx];
                                    resolved_callable = Some((name_str.clone(), f.arity, f.help.clone()));
                                }
                            }
                        });
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    if let Some((name, arity, help_opt)) = resolved_callable {
        xprintln!("Help for function {}:", name);
        xprintln!("--------------------------------------------------");
        xprintln!("Arity: {}", arity);
        if let Some(help_str) = help_opt {
            xprintln!("{}", help_str);
        } else {
            xprintln!("No documentation available.");
        }
        xprintln!("--------------------------------------------------");
    } else {
        xprintln!("No documentation or function found for the given argument.");
    }
    Value::Nil
});
