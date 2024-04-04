use log::*;
use std::collections::HashMap;

use crate::{
    common::*,
    jsprint, jsprintln,
    value::{Value, ValueArray},
};

#[derive(Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: HashMap<usize, usize>,
    pub constants: ValueArray,
}

impl Chunk {
    pub fn write_opcode(&mut self, opcode: Opcode, line: usize) {
        self.write(opcode as u8, line);
    }

    pub fn write(&mut self, data: u8, line: usize) {
        self.lines.insert(self.code.len(), line);
        self.code.push(data);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub(crate) fn write_constant(&mut self, constant_index: usize, line: usize) {
        self.write_opcode(Opcode::Constant, line);
        self.code.push(constant_index.try_into().unwrap());
    }
}

// Disassemble related methods
impl Chunk {
    pub fn disassemble(&self, name: &str) {
        jsprint!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }

        jsprintln!("====");
    }

    #[cfg(feature = "tracing")]
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        jsprint!("{offset:04} ");
        jsprint!("{:4} ", self.lines[&offset]);

        let instruction = Opcode::try_from(self.code[offset]);
        let Ok(instruction) = instruction else {
            jsprint!("Invalid opcode {:04}", self.code[offset],);
            return offset + 1;
        };

        let ret: usize = match instruction {
            Opcode::Return | Opcode::Negate => self.simple_instruction(instruction, offset),
            Opcode::Constant => self.constant_instruction(instruction, offset),
            Opcode::Add | Opcode::Subtract | Opcode::Multiply | Opcode::Divide => {
                self.simple_instruction(instruction, offset)
            }
        };

        jsprintln!();

        ret
    }

    #[cfg(not(feature = "tracing"))]
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        self.code.len()
    }

    fn simple_instruction(&self, instruction: Opcode, offset: usize) -> usize {
        jsprint!("{instruction}");

        offset + 1
    }

    fn constant_instruction(&self, instruction: Opcode, offset: usize) -> usize {
        let Ok(constant_idx): Result<usize, _> = self.code[offset + 1].try_into() else {
            jsprint!(
                "Failed to convert data {} at offset {} into constant index",
                self.code[offset + 1],
                offset + 1
            );
            return offset + 2;
        };
        jsprint!("{instruction} Idx {constant_idx} ");
        self.print_value(self.constants[constant_idx]);

        offset + 2
    }

    #[cfg(feature = "tracing")]
    pub fn print_value(&self, value: Value) {
        jsprint!("Value {value}");
    }

    #[cfg(not(feature = "tracing"))]
    pub fn print_value(&self, value: Value) {
        jsprint!("Value {value}");
    }

    #[cfg(feature = "tracing")]
    pub fn line() {
        jsprintln!();
    }

    #[cfg(not(feature = "tracing"))]
    pub fn line() {}
}
