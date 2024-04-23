use std::rc::Rc;

use crate::xprint;
use crate::{
    chunk::Chunk,
    common::Opcode,
    debug::disassemble_instruction,
    interner::{Interner, StrId},
    value::{
        print_value,
        Value::{self, *},
    },
    xprintln,
};
use anyhow::*;
use rustc_hash::FxHashMap;

pub struct Vm<'src> {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    interner: &'src mut Interner,
    globals: FxHashMap<StrId, Value>,
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
            globals: Default::default(),
        }
    }

    fn read_byte(&mut self) -> u8 {
        let value = self.chunk.code[self.ip];
        self.ip += 1;
        value
    }

    fn read_constant(&mut self) -> Option<Value> {
        let index: usize = self.read_byte() as usize;
        return self.chunk.constants.get(index).cloned();
    }

    #[cfg(feature = "tracing")]
    fn stack_trace(&self) {
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
            _ => false,
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

    fn read_string_or_id(&mut self) -> StrId {
        let value = self.read_constant().expect("Could not read constant");
        match value {
            Value::Str(id) => id,
            Value::Identifier(id) => id,
            other => panic!("Found {other} instead"),
        }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            self.stack_trace();
            disassemble_instruction(&self.chunk, self.ip, self.interner);
            let instruction = Opcode::try_from(self.read_byte()).context("Byte to opcode failed")?;
            match instruction {
                Opcode::Print => {
                    print_value(&self.pop()?, self.interner);
                    xprintln!("");
                }
                Opcode::Return => {
                    return Ok(());
                }
                Opcode::Constant => {
                    let constant = self.read_constant().context("Could not interpret constant opcode")?;
                    self.stack.push(constant);
                }
                Opcode::Negate => {
                    let value = self.pop()?;
                    match value {
                        Number(num) => self.stack.push(Value::Number(-num)),
                        _ => {
                            self.runtime_error("Operand must be a number");
                        }
                    }
                }
                Opcode::True => self.stack.push(Bool(true)),
                Opcode::False => self.stack.push(Bool(false)),
                Opcode::Pop => {
                    self.pop()?;
                }
                Opcode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    self.stack.push(self.stack[slot].clone())
                }
                Opcode::GetGlobal => {
                    let name = self.read_string_or_id();

                    if let Some(value) = self.globals.get(&name) {
                        self.stack.push(value.clone());
                    } else {
                        self.runtime_error(&format!("Undefined variable {}", self.interner.lookup(&name)));
                    }
                }
                Opcode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    self.stack[slot] = self.peek(0).clone();
                }
                Opcode::SetGlobal => {
                    let name = self.read_string_or_id();
                    if !self.globals.contains_key(&name) {
                        self.runtime_error(&format!("Undefined variable {}", self.interner.lookup(&name)));
                    } else {
                        self.globals.insert(name, self.peek(0).clone());
                    }
                }
                Opcode::DefineGlobal => {
                    let name = self.read_string_or_id();
                    self.globals.insert(name, self.peek(0).clone());
                    self.pop().unwrap();
                }
                Opcode::DeclareArray => {
                    let size_val = self.pop()?;
                    match size_val {
                        Number(len) => {
                            self.stack.push(Value::Array(Rc::new(vec![Nil; len as usize])));
                        }
                        other => {
                            self.runtime_error(&format!("Expected number, got {other}"));
                        }
                    }
                }
                Opcode::Equal => {
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.stack.push(Bool(a == b))
                }
                Opcode::Nil => self.stack.push(Nil),
                Opcode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (b, a) {
                        (Number(a), Number(b)) => {
                            self.stack.push(Number(a + b));
                        }
                        (Str(b), Str(a)) => {
                            let mut new_string = String::from(self.interner.lookup(&a));
                            new_string.push_str(self.interner.lookup(&b));
                            let id = self.interner.intern(&new_string);
                            self.stack.push(Str(id));
                        }
                        _ => {
                            self.runtime_error("Operands must be numbers");
                            self.stack.push(Nil);
                        }
                    }
                }
                Opcode::Subtract => binop!(self, Number, -),
                Opcode::Multiply => binop!(self, Number, *),
                Opcode::Divide => binop!(self, Number, /),
                Opcode::Not => {
                    let val = self.pop()?;
                    self.stack.push(Bool(self.is_falsey(val)))
                }
                Opcode::Greater => binop!(self, Bool, >),
                Opcode::Less => binop!(self, Bool, <),
            }
        }
    }

    pub fn interpret(chunk: Chunk, interner: &'src mut Interner) -> Result<()> {
        let mut vm: Vm = Vm::new(chunk, interner);
        xprintln!("Interpreting chunk of {} bytes of code", vm.chunk.code.len());
        vm.run()
    }

    fn peek(&self, distance: usize) -> &Value {
        return self
            .stack
            .get(self.stack.len() - 1 - distance)
            .unwrap_or_else(|| panic!("Failed to peek {distance} deep"));
    }
}
