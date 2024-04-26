use crate::scanner::Token;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Eq, TryFromPrimitive, PartialEq, PartialOrd, IntoPrimitive, strum_macros::Display)]
pub enum Opcode {
    Return,
    Constant,
    Not,
    Print,
    Negate,
    Add,
    Subtract,
    Multiply,
    Modulo,
    Divide,
    Nil,
    True,
    False,
    Greater,
    Pop,
    Equal,
    Less,
    DefineGlobal,
    DeclareArray,
    SetGlobal,
    GetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Loop,
}

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

pub fn identifiers_equal(a: &Token, b: &Token) -> bool {
    a.source == b.source
}
