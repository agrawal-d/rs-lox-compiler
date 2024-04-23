use std::collections::HashMap;

use crate::{
    common::*,
    debug::disassemble_instruction,
    interner::Interner,
    value::{Value, ValueArray},
    xprintln,
};

#[derive(Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: HashMap<usize, usize>,
    pub constants: ValueArray,
}

impl Chunk {
    pub fn write_opcode(&mut self, opcode: Opcode, line: usize) {
        self.write_byte(opcode as u8, line);
    }

    pub fn write_byte(&mut self, data: u8, line: usize) {
        self.lines.insert(self.code.len(), line);
        self.code.push(data);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    // Unused
    // pub(crate) fn write_constant(&mut self, constant_index: usize, line: usize) {
    //     self.write_opcode(Opcode::Constant, line);
    //     self.code.push(constant_index.try_into().unwrap());
    // }
}

// Disassemble related methods
impl Chunk {
    pub fn disassemble(&self, name: &str, interner: &Interner) {
        xprintln!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = disassemble_instruction(self, offset, interner);
        }

        xprintln!("====");
    }
}
