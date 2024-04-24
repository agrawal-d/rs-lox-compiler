use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    chunk::Chunk,
    common::Opcode,
    dbgln,
    debug::disassemble_instruction,
    interner::{Interner, StrId},
    value::{
        print_value,
        Value::{self, *},
    },
};
use anyhow::*;
use rustc_hash::FxHashMap;

#[allow(unused_imports)]
use crate::{xprint, xprintln};

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

    fn is_falsey(&self, value: &Value) -> bool {
        match value {
            Nil => true,
            Bool(b) => !b,
            _ => false,
        }
    }

    fn read_u16(&mut self) -> u16 {
        self.ip += 2;
        
        (self.chunk.code[self.ip - 2] as u16) << 8 | self.chunk.code[self.ip - 1] as u16
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
                Opcode::JumpIfFalse => {
                    let offset: u16 = self.read_u16();
                    if self.is_falsey(self.peek(0)) {
                        self.ip += offset as usize;
                    }
                }
                Opcode::Loop => {
                    let offset = self.read_u16();
                    self.ip -= offset as usize;
                }
                Opcode::Jump => {
                    let offset: u16 = self.read_u16();
                    self.ip += offset as usize;
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
                    let array_index = self.pop()?;
                    let slot = self.read_byte() as usize;
                    let value = self.get_value(self.stack[slot].clone(), &array_index);
                    self.stack.push(value)
                }
                Opcode::GetGlobal => {
                    let name = self.read_string_or_id();
                    let array_index = self.pop()?;

                    if let Some(value) = self.globals.get(&name) {
                        let value = self.get_value(value.clone(), &array_index);
                        self.stack.push(value);
                    } else {
                        self.runtime_error(&format!("Undefined variable {}", self.interner.lookup(&name)));
                    }
                }
                Opcode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    let new_value = self.pop()?;
                    let array_index = self.pop()?;
                    self.stack.push(new_value.clone());
                    let mut value_to_be_modified = self.stack[slot].clone();
                    self.set_value(&mut value_to_be_modified, &array_index, new_value);
                    self.stack[slot] = value_to_be_modified;
                }
                Opcode::SetGlobal => {
                    let name = self.read_string_or_id();

                    if !self.globals.contains_key(&name) {
                        self.runtime_error(&format!("Undefined variable {}", self.interner.lookup(&name)));
                    } else {
                        let new_value = self.pop()?;
                        let array_index = self.pop()?;
                        self.stack.push(new_value.clone());
                        let mut value_to_be_modified = self.globals.get(&name).unwrap().clone();
                        self.set_value(&mut value_to_be_modified, &array_index, new_value);
                        self.globals.insert(name, value_to_be_modified);
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
                            self.stack
                                .push(Value::Array(Rc::new(RefCell::new(vec![Number(199.99); len as usize]))));
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
                    self.stack.push(Bool(self.is_falsey(&val)))
                }
                Opcode::Greater => binop!(self, Bool, >),
                Opcode::Less => binop!(self, Bool, <),
            }
        }
    }

    // If array index is valid, return the value at that index
    // Otherwise, return the value itself
    fn get_value(&mut self, value: Value, array_index: &Value) -> Value {
        match (value, array_index) {
            (Value::Array(array), Value::Number(index)) => {
                let index = *index as usize;
                if index < array.borrow().len() {
                    array.borrow()[index].clone()
                } else {
                    self.runtime_error(&format!("Index out of bounds: {index}"));
                    Nil
                }
            }
            (value, Value::Nil) => value,
            (value, index) => {
                self.runtime_error(&format!("Tried to index value of type {value} with index {index}"));
                value
            }
        }
    }

    // If array index is valid, change the value at that index
    // Otherwise, change the value itself
    fn set_value(&mut self, value: &mut Value, array_index: &Value, new_value: Value) {
        match (value, array_index) {
            (Value::Array(array), Value::Number(index)) => {
                let index = *index as usize;
                if index < array.borrow().len() {
                    array.borrow_mut()[index] = new_value;
                } else {
                    self.runtime_error(&format!("Index out of bounds: {index}"));
                }
            }
            (value, Value::Nil) => *value = new_value,
            (value, index) => {
                self.runtime_error(&format!("Tried to index value of type {value} with index {index}"));
            }
        }
    }

    pub fn interpret(chunk: Chunk, interner: &'src mut Interner) -> Result<()> {
        dbgln!("== Interpreter VM ==");
        let mut vm: Vm = Vm::new(chunk, interner);
        dbgln!("Interpreting chunk of {} bytes of code", vm.chunk.code.len());
        vm.run()
    }

    fn peek(&self, distance: usize) -> &Value {
        return self
            .stack
            .get(self.stack.len() - 1 - distance)
            .unwrap_or_else(|| panic!("Failed to peek {distance} deep"));
    }
}
