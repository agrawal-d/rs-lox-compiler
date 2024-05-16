use crate::{chunk::Chunk, interner::StrId};

#[derive(Debug)]
pub struct Fun {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: Option<StrId>,
}

impl Default for Fun {
    fn default() -> Self {
        Self::new()
    }
}

impl Fun {
    pub fn new() -> Fun {
        Fun {
            arity: 0,
            chunk: Chunk::default(),
            name: None,
        }
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord)]
pub enum FunType {
    Function,
    Script,
}
