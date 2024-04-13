use crate::interner::Interner;
use crate::{interner::StrId, xprint};
use strum_macros::Display;

#[derive(Debug, Display, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Str(StrId),
    Nil,
}
pub type ValueArray = Vec<Value>;

#[cfg(feature = "tracing")]
pub fn print_value(value: &Value, interner: &Interner) {
    match value {
        Value::Number(num) => xprint!("{num}"),
        Value::Bool(b) => xprint!("{b}"),
        Value::Nil => xprint!("Nil"),
        Value::Str(s) => {
            xprint!("{}", interner.lookup(s));
        }
    }
}

#[cfg(not(feature = "tracing"))]
pub fn print_value(value: Value) {
    xprint!("Value {value}");
}

use Value::*;

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Number(a), Number(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (Nil, Nil) => true,
            _ => false,
        }
    }
}
