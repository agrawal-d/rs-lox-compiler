use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Eq, TryFromPrimitive, PartialEq, PartialOrd, IntoPrimitive, strum_macros::Display)]
pub enum Opcode {
    Return,
    Constant,
    Not,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
}
