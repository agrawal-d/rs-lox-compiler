use crate::{chunk::Chunk, common::Opcode, jsprintln, value::Value};
use anyhow::*;

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

macro_rules! binop {
    ($vm: ident, $op: tt) => {
        {
            let b = $vm.stack.pop().context("Stack underflow")?;
            let a = $vm.stack.pop().context("Stack underflow")?;
            let result = a $op b;
            $vm.stack.push(result);
        }
    };
}

impl Vm {
    pub fn interpret(chunk: Chunk) -> Result<()> {
        let mut vm = Vm {
            chunk,
            ip: 0,
            stack: Vec::new(),
        };
        vm.run()?;
        Ok(())
    }

    fn read_byte(&mut self) -> u8 {
        let value = self.chunk.code[self.ip];
        self.ip += 1;
        value
    }

    fn read_constant(&mut self) -> Option<&f64> {
        let index: usize = self.read_byte() as usize;
        let code = &self.chunk.code;
        return self.chunk.constants.get(index);
    }

    fn print_value(&self, value: Value) {
        jsprintln!("{:?}", value);
    }

    #[cfg(feature = "tracing")]
    fn stack_trace(&self) {
        use crate::jsprint;

        if !self.stack.is_empty() {
            jsprint!("Stack values: ");
        }
        for value in &self.stack {
            jsprint!("[ ");
            self.chunk.print_value(*value);
            jsprint!("  ]");
        }

        if !self.stack.is_empty() {
            jsprintln!("");
        }
    }

    #[cfg(not(feature = "tracing"))]
    fn stack_trace(&self) {}

    pub fn run(&mut self) -> Result<()> {
        jsprintln!(
            "Interpreting chunk of {} bytes of code",
            self.chunk.code.len()
        );
        loop {
            self.chunk.disassemble_instruction(self.ip);
            self.stack_trace();
            let instruction =
                Opcode::try_from(self.read_byte()).context("Byte to opcode failed")?;
            match instruction {
                Opcode::Return => {
                    let value = self
                        .stack
                        .pop()
                        .context("Nothing in VM stack when returning")?;
                    return Ok(());
                }
                Opcode::Constant => {
                    let constant = *self
                        .read_constant()
                        .context("Could not interpret constant opcode")?;
                    self.stack.push(constant);
                }
                Opcode::Negate => {
                    let value = self.stack.pop().context("Nothing in stack to negate")?;
                    self.stack.push(-value);
                }
                Opcode::Add => binop!(self, +),
                Opcode::Subtract => binop!(self, -),
                Opcode::Multiply => binop!(self, *),
                Opcode::Divide => binop!(self, /),
            }
        }
    }
}
