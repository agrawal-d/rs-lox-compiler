use crate::{chunk::Chunk, common::Opcode, print};

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
        loop {}
    }
}
