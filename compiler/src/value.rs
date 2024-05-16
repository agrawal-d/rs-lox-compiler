use std::cell::RefCell;
use std::rc::Rc;

use crate::interner::Interner;
use crate::native::Callable;
use crate::{interner::StrId, xprint};
use strum_macros::Display;

#[derive(Debug, Display, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Str(StrId),
    Identifier(StrId),
    Array(Rc<RefCell<ValueArray>>),
    Function(usize),
    NativeFunction(Rc<dyn Callable>),
    Nil,
}

pub type ValueArray = Vec<Value>;

pub fn print_value(value: &Value, interner: &Interner) {
    xprint!("{}", value_as_string(value, interner));
}

pub fn value_as_string(value: &Value, interner: &Interner) -> String {
    match value {
        Value::Number(num) => format!("{num}"),
        Value::Bool(b) => format!("{b}"),
        Value::Nil => format!("Nil"),
        Value::Str(s) => {
            format!("{}", interner.lookup(s))
        }
        Value::Identifier(id) => {
            format!("Identifier: {}", interner.lookup(id))
        }
        Value::Array(arr) => {
            let mut s = format!("Array<{} elements [", arr.borrow().len());
            for (i, v) in arr.borrow().iter().enumerate() {
                if i != 0 {
                    s.push_str(", ");
                }

                if i >= 10 {
                    s.push_str("...");
                    break;
                }

                s.push_str(&value_as_string(v, interner));
            }
            s.push_str("]>");
            s
        }
        Value::Function(idx) => {
            format!("<Function {idx}>")
        }
        Value::NativeFunction(fun) => {
            format!("<Native Function {}>", fun.as_ref().name())
        }
    }
}

use Value::*;

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
            (Bool(a), Bool(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (Nil, Nil) => true,
            (Array(a), Array(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
