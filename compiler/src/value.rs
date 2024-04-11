use std::rc::Rc;

use strum_macros::Display;

#[derive(Debug, Display, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
}
pub type ValueArray = Vec<Value>;
