use crate::xprint;
use std::rc::Rc;
use strum_macros::Display;

#[derive(Debug, Display, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    XString(Rc<str>),
    Nil,
}
pub type ValueArray = Vec<Value>;

#[cfg(feature = "tracing")]
pub fn print_value(value: &Value) {
    match value {
        Value::Number(num) => xprint!("{num}"),
        Value::Bool(b) => xprint!("{b}"),
        Value::Nil => xprint!("Nil"),
        Value::XString(s) => {
            let data = s.as_ref();
            xprint!("{data}");
        }
    }
}

#[cfg(not(feature = "tracing"))]
pub fn print_value(&self, value: Value) {
    xprint!("Value {value}");
}

use Value::*;

pub fn values_equal(a: Value, b: Value) -> bool {
    match (a, b) {
        (Number(a), Number(b)) => a == b,
        (Bool(a), Bool(b)) => a == b,
        (XString(a), XString(b)) => a.as_ref() == b.as_ref(),
        (Nil, Nil) => true,
        _ => false,
    }
}
