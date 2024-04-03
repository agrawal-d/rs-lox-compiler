#[derive(Debug)]
pub enum Opcode {
    Return,
}

#[derive(Debug)]
pub struct Chunk {
    code: Vec<Opcode>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk { code: Vec::new() }
    }

    pub fn write(&mut self, opcode: Opcode) {
        self.code.push(opcode);
    }
}
