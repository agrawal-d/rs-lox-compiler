use std::cell::RefCell;
use std::rc::Rc;

use crate::interner::Interner;
use crate::{interner::StrId, xprint};
use strum_macros::Display;

#[derive(Debug, Display, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Str(StrId),
    Identifier(StrId),
    Array(Rc<RefCell<ValueArray>>),
    Nil,
}

pub type ValueArray = Vec<Value>;

pub fn print_value(value: &Value, interner: &Interner) {
    match value {
        Value::Number(num) => xprint!("{num}"),
        Value::Bool(b) => xprint!("{b}"),
        Value::Nil => xprint!("Nil"),
        Value::Str(s) => {
            xprint!("{}", interner.lookup(s));
        }
        Value::Identifier(id) => {
            xprint!("Identifier: {}", interner.lookup(id))
        }
        Value::Array(arr) => {
            xprint!("<Array[{}]>", arr.borrow().len());
        }
    }
}

use Value::*;

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Number(a), Number(b)) => (a - b) < f64::EPSILON,
            (Bool(a), Bool(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (Nil, Nil) => true,
            (Array(a), Array(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
