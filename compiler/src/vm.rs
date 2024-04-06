use crate::{chunk::Chunk, common::Opcode, value::Value, value::Value::*, xprintln};
use anyhow::*;

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

macro_rules! binop {
    ($vm: ident, $typ: tt, $op: tt) => {
        {
            let b = $vm.stack.pop().context("Stack underflow")?;
            let a = $vm.stack.pop().context("Stack underflow")?;
            match (a, b) {
                (Number(a), Number(b)) => {
                    let result = a $op b;
                    $vm.stack.push($typ(result));
                },
                _ => { $vm.runtime_error("Operands must be numbers"); }
            }

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

    fn read_constant(&mut self) -> Option<&Value> {
        let index: usize = self.read_byte() as usize;
        return self.chunk.constants.get(index);
    }

    #[cfg(feature = "tracing")]
    fn stack_trace(&self) {
        use crate::xprint;

        if !self.stack.is_empty() {
            xprint!("Stack values: ");
        }
        xprint!("[ ");
        for value in &self.stack {
            self.chunk.print_value(*value);
            xprint!(" ");
        }
        xprint!("]");

        if !self.stack.is_empty() {
            xprintln!("");
        }
    }

    #[cfg(not(feature = "tracing"))]
    fn stack_trace(&self) {}

    fn runtime_error(&mut self, msg: &str) {
        xprintln!("Runtime error: {msg}");
        let line = self.chunk.lines[&self.ip];
        xprintln!("[line {line}] in script");
    }

    pub fn interpret(chunk: Chunk) -> Result<()> {
        let mut vm: Vm = Vm::new(chunk);
        xprintln!("Interpreting chunk of {} bytes of code", vm.chunk.code.len());
        loop {
            vm.chunk.disassemble_instruction(vm.ip);
            vm.stack_trace();
            let instruction = Opcode::try_from(vm.read_byte()).context("Byte to opcode failed")?;
            match instruction {
                Opcode::Return => {
                    let value = vm.stack.pop().context("Nothing in VM stack when returning")?;
                    xprintln!("Returned value: {}", value);
                    return Ok(());
                }
                Opcode::Constant => {
                    let constant = *vm.read_constant().context("Could not interpret constant opcode")?;
                    vm.stack.push(constant);
                }
                Opcode::Negate => {
                    let value = vm.stack.pop().context("Nothing in stack to negate")?;
                    match value {
                        Number(num) => vm.stack.push(Value::Number(-num)),
                        _ => {
                            vm.runtime_error("Operand must be a number");
                        }
                    }
                }
                Opcode::False => vm.stack.push(Bool(false)),
                Opcode::True => vm.stack.push(Bool(true)),
                Opcode::Nil => vm.stack.push(Nil),
                Opcode::Add => binop!(vm, Number, +),
                Opcode::Subtract => binop!(vm, Number, -),
                Opcode::Multiply => binop!(vm, Number, *),
                Opcode::Divide => binop!(vm, Number, /),
            }
        }
    }
}