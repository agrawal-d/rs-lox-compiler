use std::cell::RefCell;
use std::rc::Rc;

use crate::interner::Interner;
use crate::native::Callable;
use crate::{interner::StrId, xprint};
use strum_macros::Display;

#[derive(Debug, Clone)]
pub struct ClassData {
    pub name: StrId,
    pub methods: RefCell<rustc_hash::FxHashMap<StrId, usize>>,
}

#[derive(Debug, Clone)]
pub struct InstanceData {
    pub class: Rc<ClassData>,
    pub fields: RefCell<rustc_hash::FxHashMap<StrId, Value>>,
}

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
    Class(Rc<ClassData>),
    Instance(Rc<RefCell<InstanceData>>),
    BoundMethod {
        instance: Rc<RefCell<InstanceData>>,
        method_idx: usize,
    },
}

pub type ValueArray = Vec<Value>;

pub fn print_value(value: &Value, interner: &Interner) {
    xprint!("{}", value_as_string(value, interner));
}

pub fn value_as_string(value: &Value, interner: &Interner) -> String {
    match value {
        Value::Number(num) => format!("{num}"),
        Value::Bool(b) => format!("{b}"),
        Value::Nil => "Nil".to_string(),
        Value::Str(s) => interner.lookup(s).to_string(),
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
        Value::Class(class) => {
            format!("<Class {}>", interner.lookup(&class.name))
        }
        Value::Instance(instance) => {
            format!("<Instance of {}>", interner.lookup(&instance.borrow().class.name))
        }
        Value::BoundMethod { instance, method_idx } => {
            format!(
                "<Bound Method {} of {}>",
                method_idx,
                interner.lookup(&instance.borrow().class.name)
            )
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
            (Class(a), Class(b)) => Rc::ptr_eq(a, b),
            (Instance(a), Instance(b)) => Rc::ptr_eq(a, b),
            (
                BoundMethod {
                    instance: a_inst,
                    method_idx: a_idx,
                },
                BoundMethod {
                    instance: b_inst,
                    method_idx: b_idx,
                },
            ) => Rc::ptr_eq(a_inst, b_inst) && a_idx == b_idx,
            _ => false,
        }
    }
}
