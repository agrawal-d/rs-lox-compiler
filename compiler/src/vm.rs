use crate::{
    chunk::Chunk,
    common::Opcode,
    debug::disassemble_instruction,
    interner::Interner,
    value::{
        print_value,
        Value::{self, *},
    },
    xprintln,
};
use anyhow::*;

pub struct Vm<'src> {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    interner: &'src mut Interner,
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

impl<'src> Vm<'src> {
    pub fn new(chunk: Chunk, interner: &'src mut Interner) -> Vm {
        Vm {
            chunk,
            ip: 0,
            stack: Default::default(),
            interner,
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
        use crate::{value::print_value, xprint};

        xprint!("Stack values: ");
        xprint!("[ ");
        for value in &self.stack {
            print_value(value, self.interner);
            xprint!(", ");
        }
        xprint!("]");

        xprintln!("");
    }

    fn is_falsey(&self, value: Value) -> bool {
        match value {
            Nil => true,
            Bool(b) => !b,
            Number(n) => n == 0.0,
            Str(s) => self.interner.lookup(&s).is_empty(),
        }
    }

    #[cfg(not(feature = "tracing"))]
    fn stack_trace(&self) {}

    fn runtime_error(&mut self, msg: &str) {
        xprintln!("Runtime error: {msg}");
        let line = self.chunk.lines[&self.ip];
        xprintln!("[line {line}] in script");
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().context("Nothing in stack to pop")
    }

    pub fn interpret(chunk: Chunk, interner: &'src mut Interner) -> Result<()> {
        let mut vm: Vm = Vm::new(chunk, interner);
        xprintln!("Interpreting chunk of {} bytes of code", vm.chunk.code.len());
        loop {
            disassemble_instruction(&vm.chunk, vm.ip, vm.interner);
            vm.stack_trace();
            let instruction = Opcode::try_from(vm.read_byte()).context("Byte to opcode failed")?;
            match instruction {
                Opcode::Print => {
                    print_value(&vm.pop()?, &vm.interner);
                    xprintln!("");
                }
                Opcode::Return => {
                    return Ok(());
                }
                Opcode::Constant => {
                    let constant = vm.read_constant().context("Could not interpret constant opcode")?.clone();
                    vm.stack.push(constant);
                }
                Opcode::Negate => {
                    let value = vm.pop()?;
                    match value {
                        Number(num) => vm.stack.push(Value::Number(-num)),
                        _ => {
                            vm.runtime_error("Operand must be a number");
                        }
                    }
                }
                Opcode::True => vm.stack.push(Bool(true)),
                Opcode::False => vm.stack.push(Bool(false)),
                Opcode::Pop => {
                    vm.pop()?;
                }
                Opcode::Equal => {
                    let a = vm.pop()?;
                    let b = vm.pop()?;
                    vm.stack.push(Bool(a == b))
                }
                Opcode::Nil => vm.stack.push(Nil),
                Opcode::Add => {
                    let b = vm.pop()?;
                    let a = vm.pop()?;
                    match (b, a) {
                        (Number(a), Number(b)) => {
                            vm.stack.push(Number(a + b));
                        }
                        (Str(b), Str(a)) => {
                            let mut new_string = String::from(vm.interner.lookup(&a));
                            new_string.push_str(vm.interner.lookup(&b));
                            let id = vm.interner.intern(&new_string);
                            vm.stack.push(Str(id));
                        }
                        _ => {
                            vm.runtime_error("Operands must be numbers");
                            vm.stack.push(Nil);
                        }
                    }
                }
                Opcode::Subtract => binop!(vm, Number, -),
                Opcode::Multiply => binop!(vm, Number, *),
                Opcode::Divide => binop!(vm, Number, /),
                Opcode::Not => {
                    let val = vm.pop()?;
                    vm.stack.push(Bool(vm.is_falsey(val)))
                }
                Opcode::Greater => binop!(vm, Bool, >),
                Opcode::Less => binop!(vm, Bool, <),
            }
        }
    }
}
