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
    Divide,
    Nil,
    True,
    False,
    Greater,
    Pop,
    Equal,
    Less,
}

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
