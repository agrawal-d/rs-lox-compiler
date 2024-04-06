use strum_macros::Display;

#[derive(Debug, Display, Clone, Copy)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
}
pub type ValueArray = Vec<Value>;
