use compiler::callable_struct;
use compiler::interner::Interner;
use compiler::native::{Callable, Globals, set_global_error};
use compiler::value::Value;

callable_struct!(Sin, "sin", 1, "sin(x)
Calculates the sine of the angle in radians.
Arguments:
  x: Number in radians.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.sin()),
        _ => { set_global_error(interner, globals, "Expected number argument for sin"); Value::Nil }
    }
});

callable_struct!(Cos, "cos", 1, "cos(x)
Calculates the cosine of the angle in radians.
Arguments:
  x: Number in radians.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.cos()),
        _ => { set_global_error(interner, globals, "Expected number argument for cos"); Value::Nil }
    }
});

callable_struct!(Tan, "tan", 1, "tan(x)
Calculates the tangent of the angle in radians.
Arguments:
  x: Number in radians.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.tan()),
        _ => { set_global_error(interner, globals, "Expected number argument for tan"); Value::Nil }
    }
});

callable_struct!(Asin, "asin", 1, "asin(x)
Calculates the arcsine of x.
Arguments:
  x: Number between -1 and 1.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.asin()),
        _ => { set_global_error(interner, globals, "Expected number argument for asin"); Value::Nil }
    }
});

callable_struct!(Acos, "acos", 1, "acos(x)
Calculates the arccosine of x.
Arguments:
  x: Number between -1 and 1.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.acos()),
        _ => { set_global_error(interner, globals, "Expected number argument for acos"); Value::Nil }
    }
});

callable_struct!(Atan, "atan", 1, "atan(x)
Calculates the arctangent of x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.atan()),
        _ => { set_global_error(interner, globals, "Expected number argument for atan"); Value::Nil }
    }
});

callable_struct!(Atan2, "atan2", 2, "atan2(y, x)
Calculates the arctangent of y/x using sign to determine quadrant.
Arguments:
  y: Number.
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match (&args[0], &args[1]) {
        (Value::Number(y), Value::Number(x)) => Value::Number(y.atan2(*x)),
        _ => { set_global_error(interner, globals, "Expected two numbers for atan2"); Value::Nil }
    }
});

callable_struct!(Sinh, "sinh", 1, "sinh(x)
Calculates the hyperbolic sine of x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.sinh()),
        _ => { set_global_error(interner, globals, "Expected number argument for sinh"); Value::Nil }
    }
});

callable_struct!(Cosh, "cosh", 1, "cosh(x)
Calculates the hyperbolic cosine of x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.cosh()),
        _ => { set_global_error(interner, globals, "Expected number argument for cosh"); Value::Nil }
    }
});

callable_struct!(Tanh, "tanh", 1, "tanh(x)
Calculates the hyperbolic tangent of x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.tanh()),
        _ => { set_global_error(interner, globals, "Expected number argument for tanh"); Value::Nil }
    }
});

callable_struct!(Sqrt, "sqrt", 1, "sqrt(x)
Calculates the square root of x.
Arguments:
  x: Non-negative number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.sqrt()),
        _ => { set_global_error(interner, globals, "Expected number argument for sqrt"); Value::Nil }
    }
});

callable_struct!(Cbrt, "cbrt", 1, "cbrt(x)
Calculates the cube root of x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.cbrt()),
        _ => { set_global_error(interner, globals, "Expected number argument for cbrt"); Value::Nil }
    }
});

callable_struct!(Pow, "pow", 2, "pow(base, exp)
Calculates base raised to exp power.
Arguments:
  base: Number.
  exp: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match (&args[0], &args[1]) {
        (Value::Number(b), Value::Number(e)) => Value::Number(b.powf(*e)),
        _ => { set_global_error(interner, globals, "Expected two numbers for pow"); Value::Nil }
    }
});

callable_struct!(Exp, "exp", 1, "exp(x)
Calculates e^x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.exp()),
        _ => { set_global_error(interner, globals, "Expected number argument for exp"); Value::Nil }
    }
});

callable_struct!(Log, "log", 1, "log(x)
Calculates the natural logarithm (base e) of x.
Arguments:
  x: Positive number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.ln()),
        _ => { set_global_error(interner, globals, "Expected number argument for log"); Value::Nil }
    }
});

callable_struct!(Log10, "log10", 1, "log10(x)
Calculates base-10 logarithm of x.
Arguments:
  x: Positive number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.log10()),
        _ => { set_global_error(interner, globals, "Expected number argument for log10"); Value::Nil }
    }
});

callable_struct!(Log2, "log2", 1, "log2(x)
Calculates base-2 logarithm of x.
Arguments:
  x: Positive number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.log2()),
        _ => { set_global_error(interner, globals, "Expected number argument for log2"); Value::Nil }
    }
});

callable_struct!(Abs, "abs", 1, "abs(x)
Returns the absolute value of x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.abs()),
        _ => { set_global_error(interner, globals, "Expected number argument for abs"); Value::Nil }
    }
});

callable_struct!(Floor, "floor", 1, "floor(x)
Returns largest integer less than or equal to x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.floor()),
        _ => { set_global_error(interner, globals, "Expected number argument for floor"); Value::Nil }
    }
});

callable_struct!(Ceil, "ceil", 1, "ceil(x)
Returns smallest integer greater than or equal to x.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.ceil()),
        _ => { set_global_error(interner, globals, "Expected number argument for ceil"); Value::Nil }
    }
});

callable_struct!(Round, "round", 1, "round(x)
Rounds x to nearest integer.
Arguments:
  x: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(n) => Value::Number(n.round()),
        _ => { set_global_error(interner, globals, "Expected number argument for round"); Value::Nil }
    }
});

callable_struct!(Min, "min", 2, "min(a, b)
Returns the smaller of two numbers.
Arguments:
  a: Number.
  b: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Value::Number(a.min(*b)),
        _ => { set_global_error(interner, globals, "Expected two numbers for min"); Value::Nil }
    }
});

callable_struct!(Max, "max", 2, "max(a, b)
Returns the larger of two numbers.
Arguments:
  a: Number.
  b: Number.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Value::Number(a.max(*b)),
        _ => { set_global_error(interner, globals, "Expected two numbers for max"); Value::Nil }
    }
});

callable_struct!(Pi, "pi", 0, "pi()
Returns mathematical constant pi (3.14159...).
Arguments: None.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    Value::Number(std::f64::consts::PI)
});

callable_struct!(E, "e", 0, "e()
Returns Euler's number (2.71828...).
Arguments: None.
Returns: Number.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    Value::Number(std::f64::consts::E)
});

pub fn register(interner: &mut Interner, globals: &mut Globals, alias: &str) {
    let funcs: &[(&str, std::rc::Rc<dyn Callable>)] = &[
        ("sin", std::rc::Rc::new(Sin)),
        ("cos", std::rc::Rc::new(Cos)),
        ("tan", std::rc::Rc::new(Tan)),
        ("asin", std::rc::Rc::new(Asin)),
        ("acos", std::rc::Rc::new(Acos)),
        ("atan", std::rc::Rc::new(Atan)),
        ("atan2", std::rc::Rc::new(Atan2)),
        ("sinh", std::rc::Rc::new(Sinh)),
        ("cosh", std::rc::Rc::new(Cosh)),
        ("tanh", std::rc::Rc::new(Tanh)),
        ("sqrt", std::rc::Rc::new(Sqrt)),
        ("cbrt", std::rc::Rc::new(Cbrt)),
        ("pow", std::rc::Rc::new(Pow)),
        ("exp", std::rc::Rc::new(Exp)),
        ("log", std::rc::Rc::new(Log)),
        ("log10", std::rc::Rc::new(Log10)),
        ("log2", std::rc::Rc::new(Log2)),
        ("abs", std::rc::Rc::new(Abs)),
        ("floor", std::rc::Rc::new(Floor)),
        ("ceil", std::rc::Rc::new(Ceil)),
        ("round", std::rc::Rc::new(Round)),
        ("min", std::rc::Rc::new(Min)),
        ("max", std::rc::Rc::new(Max)),
        ("pi", std::rc::Rc::new(Pi)),
        ("e", std::rc::Rc::new(E)),
    ];

    for (name, callable) in funcs {
        let full_name = format!("{}.{}", alias, name);
        let id = interner.intern(&full_name);
        globals.insert(id, Value::NativeFunction(callable.clone()));
    }
}
