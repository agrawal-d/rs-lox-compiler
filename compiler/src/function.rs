use crate::{chunk::Chunk, interner::StrId};

#[derive(Debug)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: Option<StrId>,
}

impl Function {
    pub fn new() -> Function {
        Function {
            arity: 0,
            chunk: Chunk::default(),
            name: None,
        }
    }
}
