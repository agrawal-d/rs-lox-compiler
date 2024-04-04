use crate::{
    appendOutput,
    chunk::{Chunk, Opcode},
};

pub struct Vm {
    chunk: Chunk,
    ip: usize,
}

impl Vm {
    pub fn interpret(chunk: Chunk) {
        let mut vm = Vm { chunk, ip: 0 };
        vm.run()
    }

    pub fn run(&mut self) {
        loop {
            let instruction = &self.chunk.code[self.ip];
            self.ip += 1;

            match instruction {
                Opcode::Return => {
                    return;
                }
                Opcode::Constant(index) => {
                    let constant = self.chunk.constants[*index];
                    appendOutput(format!("{:?}", constant));
                }
            }
        }
    }
}
