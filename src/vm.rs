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
    pub fn new(chunk: Chunk) -> Vm {
        Vm {
            chunk,
            ip: 0,
            stack: Default::default(),
        }
    }

    fn read_byte(&mut self) -> u8 {
        let value = self.chunk.code[self.ip];
        self.ip += 1;
        value
    }

    fn read_constant(&mut self) -> Option<&f64> {
        let index: usize = self.read_byte() as usize;
        return self.chunk.constants.get(index);
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

    pub fn interpret(chunk: Chunk) -> Result<()> {
        let mut vm: Vm = Vm::new(chunk);
        jsprintln!("Interpreting chunk of {} bytes of code", vm.chunk.code.len());
        loop {
            vm.chunk.disassemble_instruction(vm.ip);
            vm.stack_trace();
            let instruction = Opcode::try_from(vm.read_byte()).context("Byte to opcode failed")?;
            match instruction {
                Opcode::Return => {
                    let value = vm.stack.pop().context("Nothing in VM stack when returning")?;
                    jsprintln!("Returned value: {}", value);
                    return Ok(());
                }
                Opcode::Constant => {
                    let constant = *vm.read_constant().context("Could not interpret constant opcode")?;
                    vm.stack.push(constant);
                }
                Opcode::Negate => {
                    let value = vm.stack.pop().context("Nothing in stack to negate")?;
                    vm.stack.push(-value);
                }
                Opcode::Add => binop!(vm, +),
                Opcode::Subtract => binop!(vm, -),
                Opcode::Multiply => binop!(vm, *),
                Opcode::Divide => binop!(vm, /),
            }
        }
    }
}
