use std::rc::Rc;
use std::{cell::RefCell, future::Future};

use crate::{
    common::Opcode,
    dbgln,
    debug::disassemble_instruction,
    fun::Fun,
    interner::{Interner, StrId},
    native::*,
    value::{
        print_value,
        Value::{self, *},
    },
};
use anyhow::*;
use rustc_hash::FxHashMap;

#[allow(unused_imports)]
use crate::{xprint, xprintln};

#[derive(Debug, Default, Clone, Copy)]
struct CallFrame {
    pub fun_idx: usize,
    pub ip: usize,
    pub start_len: usize,   // Length of the stack before this frame
    pub slot_offset: usize, // Offset of this call-frame from the base of the stack
}

pub const ERR_STRING: &str = "errString";

pub struct Vm<'src, F, Fut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = String>,
{
    frames: Vec<CallFrame>,
    functions: Vec<Fun>,
    stack: Vec<Value>,
    interner: &'src mut Interner,
    globals: FxHashMap<StrId, Value>,
    global_error_id: StrId, // StrId of global error variable
    read_async: F,
}

macro_rules! binop {
    ($vm: ident, $typ: tt, $op: tt) => {
        {
            let b = $vm.pop_unchecked();
            let a = $vm.pop_unchecked();
            match (a, b) {
                (Number(a), Number(b)) => {
                    let result = a $op b;
                    $vm.stack.push($typ(result));
                },
                (first, second)=> { $vm.runtime_error(&format!("Operands must be numbers, but got {first} and {second}")); }
            }

        }
    };
}

macro_rules! frame {
    ($inst: expr) => {
        unsafe { $inst.frames.last().unwrap_unchecked() }
    };
}

macro_rules! frame_mut {
    ($inst: expr) => {
        unsafe { $inst.frames.last_mut().unwrap_unchecked() }
    };
}

macro_rules! register_native {
    ($vm: ident, $name: ident) => {
        let name = $vm.interner.intern(stringify!($name));
        dbgln!("Registering native function {}", stringify!($name));
        $vm.globals.insert(name, Value::NativeFunction(Rc::new($name)));
    };
}

fn get_array(arr: &Value, index: &Value) -> anyhow::Result<Value, Error> {
    match (arr, index) {
        (Value::Array(array), Value::Number(index)) => {
            let index = *index as usize;
            if index < array.borrow().len() {
                Ok(array.borrow()[index].clone())
            } else {
                bail!("Index out of bounds: {index}")
            }
        }
        (arr, index) => {
            bail!(format!("Tried to index value of type {arr} with index {index}"));
        }
    }
}

fn set_array(arr: &mut Value, index: &Value, new_value: Value) -> anyhow::Result<(), Error> {
    match (arr, index) {
        (Value::Array(array), Value::Number(index)) => {
            let index = *index as usize;
            if index < array.borrow().len() {
                array.borrow_mut()[index] = new_value;
                Ok(())
            } else {
                bail!("Index out of bounds: {index}")
            }
        }
        (arr, index) => {
            bail!(format!("Tried to index value of type {arr} with index {index}"));
        }
    }
}

impl<'src, F, Fut> Vm<'src, F, Fut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = String>,
{
    pub fn new(interner: &'src mut Interner, functions: Vec<Fun>, read_async: F) -> Vm<F, Fut> {
        let global_error_id = interner.intern(ERR_STRING);

        let mut frames: Vec<CallFrame> = Vec::with_capacity(10240);
        frames.push(CallFrame {
            fun_idx: functions.len() - 1,
            ip: 0,
            start_len: 0,
            slot_offset: 0,
        });

        Vm {
            frames,
            functions,
            stack: Vec::with_capacity(10240),
            interner,
            globals: Default::default(),
            global_error_id,
            read_async,
        }
    }

    fn code(&self, offset: usize) -> u8 {
        self.functions[frame!(self).fun_idx].chunk.code[offset]
    }

    fn constant(&self, index: usize) -> &Value {
        #[allow(unused_unsafe)]
        unsafe {
            self.functions[frame!(self).fun_idx].chunk.constants.get_unchecked(index)
        }
    }

    fn read_byte(&mut self) -> u8 {
        let value = unsafe { self.code(self.frames.last().unwrap_unchecked().ip) };
        frame_mut!(self).ip += 1;
        value
    }

    fn read_constant(&mut self) -> &Value {
        let index: usize = self.read_byte() as usize;
        return self.constant(index);
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
            Number(n) => (*n - 0.0).abs() < f64::EPSILON,
            Array(arr) => arr.borrow().is_empty(),
            _ => false,
        }
    }

    fn read_u16(&mut self) -> u16 {
        frame_mut!(self).ip += 2;

        let high_byte = self.code(frame!(self).ip - 2) as u16;
        let low_byte = self.code(frame!(self).ip - 1) as u16;
        (high_byte << 8) | low_byte
    }

    #[cfg(not(feature = "tracing"))]
    fn stack_trace(&self) {}

    fn runtime_error(&self, msg: &str) -> ! {
        xprintln!("Runtime error: {msg}");
        xprintln!("Traceback (most recent call first):");
        let mut idx = (self.frames.len() - 1) as isize;

        while idx >= 0 {
            let frame = &self.frames[idx as usize];
            let fun: &Fun = &self.functions[frame.fun_idx];
            let fun_name = match fun.name {
                Some(name) => self.interner.lookup(&name),
                None => "<script>",
            };
            xprintln!("[line {:3}] in {}", fun.chunk.lines[&frame.ip], fun_name);
            idx -= 1;
        }

        panic!("Exiting due to runtime error");
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().context("Nothing in stack to pop")
    }

    fn pop_unchecked(&mut self) -> Value {
        unsafe { self.stack.pop().unwrap_unchecked() }
    }

    fn read_string_or_id(&mut self) -> StrId {
        let value = self.read_constant();
        match value {
            Value::Str(id) => *id,
            Value::Identifier(id) => *id,
            other => panic!("Found {other} instead"),
        }
    }

    fn reset_err_string(&mut self) {
        self.globals.insert(self.global_error_id, Value::Nil);
    }

    async fn call_value(&mut self, arg_count: u8) -> bool {
        let callee = self.peek(arg_count as usize);
        match callee {
            Function(idx) => {
                let fun = &self.functions[*idx];

                if arg_count as usize != fun.arity {
                    self.runtime_error(&format!("Expected {} arguments but got {} instead", fun.arity, arg_count));
                }

                let new_frame_offset = self.stack.len() - arg_count as usize;
                let orig_len = self.stack.len() - 1 - fun.arity;
                let frame: CallFrame = CallFrame {
                    fun_idx: *idx,
                    ip: 0,
                    start_len: orig_len,
                    slot_offset: new_frame_offset, // -1 here in book ?
                };
                self.frames.push(frame);
                true
            }
            NativeFunction(fun) => {
                if arg_count as usize != fun.arity() {
                    self.runtime_error(&format!("Expected {} arguments but got {} instead", fun.arity(), arg_count));
                }

                let function = fun.clone();

                // Speacial read input functions - take the prompt, and convert it the the user response
                if function.name() == "ReadString" || function.name() == "ReadNumber" || function.name() == "ReadBool" {
                    let len = self.stack.len();
                    let first_arg = &mut self.stack[len - arg_count as usize];
                    match first_arg {
                        Value::Str(id) => {
                            let prompt = self.interner.lookup(id);
                            let input = (self.read_async)(prompt.to_string()).await;
                            *first_arg = Value::Str(self.interner.intern(&input));
                        }
                        _ => *first_arg = Value::Nil,
                    }
                };

                let args = &self.stack[self.stack.len() - arg_count as usize..];
                self.globals.insert(self.global_error_id, Value::Nil); // Reset error string

                let result = function.call(self.interner, &mut self.globals, args);

                dbgln!("Truncating to length {}", self.stack.len() - 1 - arg_count as usize);
                self.stack.truncate(self.stack.len() - 1 - arg_count as usize);
                self.stack.push(result);

                true
            }
            other => {
                self.runtime_error(&format!("Can only call functions, got {other}"));
            }
        }
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            self.stack_trace();
            disassemble_instruction(&self.functions[frame!(self).fun_idx].chunk, frame!(self).ip, self.interner);
            let instruction = unsafe { Opcode::try_from(self.read_byte()).unwrap_unchecked() };
            match instruction {
                Opcode::Print => {
                    print_value(&self.pop_unchecked(), self.interner);
                    xprintln!("");
                }
                Opcode::JumpIfFalse => {
                    let offset: u16 = self.read_u16();
                    if self.is_falsey(self.peek(0)) {
                        frame_mut!(self).ip += offset as usize;
                    }
                }
                Opcode::Loop => {
                    let offset = self.read_u16();
                    frame_mut!(self).ip -= offset as usize;
                }
                Opcode::Jump => {
                    let offset: u16 = self.read_u16();
                    frame_mut!(self).ip += offset as usize;
                }
                Opcode::Call => {
                    let arg_count = self.read_byte();
                    if !self.call_value(arg_count).await {
                        self.runtime_error("Could not call value");
                    }
                }
                Opcode::Return => {
                    let value = self.pop().expect("Nothing to return");
                    let orig_len = frame!(self).start_len;
                    self.frames.pop();

                    if self.frames.is_empty() {
                        return Ok(());
                    }

                    self.stack_trace();
                    dbgln!("Truncating to length {}", orig_len,);
                    self.stack.truncate(orig_len);
                    self.stack.push(value);
                }
                Opcode::Constant => {
                    let constant = self.read_constant().clone();
                    self.stack.push(constant);
                }
                Opcode::Negate => {
                    let value = self.pop_unchecked();
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
                    self.pop_unchecked();
                }
                Opcode::GetLocal => {
                    let array_index = self.pop_unchecked();
                    let slot = self.read_byte() as usize;
                    let value = &self.stack[frame!(self).slot_offset + slot];

                    if array_index == Value::Nil {
                        self.stack.push(value.clone());
                    } else {
                        self.stack.push(get_array(value, &array_index).unwrap_or_else(|err| {
                            self.runtime_error(&format!("Error getting array: {err}"));
                        }));
                    }
                }
                Opcode::GetGlobal => {
                    let name = self.read_string_or_id();
                    let array_index = self.pop_unchecked();

                    if let Some(value) = self.globals.get(&name) {
                        if array_index == Value::Nil {
                            self.stack.push(value.clone());
                        } else {
                            self.stack.push(get_array(value, &array_index).unwrap_or_else(|err| {
                                self.runtime_error(&format!("Error getting array: {err}"));
                            }));
                        }
                    } else {
                        self.runtime_error(&format!("Undefined variable {}", self.interner.lookup(&name)));
                    }
                }
                Opcode::SetLocal => {
                    let slot: usize = self.read_byte() as usize;
                    let new_value = self.pop_unchecked();
                    let array_index = self.pop_unchecked();
                    self.stack.push(new_value.clone());
                    let value_to_be_modified = &mut self.stack[frame!(self).slot_offset + slot];

                    if array_index == Value::Nil {
                        *value_to_be_modified = new_value;
                    } else {
                        set_array(value_to_be_modified, &array_index, new_value).unwrap_or_else(|err| {
                            self.runtime_error(&format!("Error setting array: {err}"));
                        });
                    }
                }
                Opcode::SetGlobal => {
                    let name = self.read_string_or_id();

                    if !self.globals.contains_key(&name) {
                        self.runtime_error(&format!("Undefined variable {}", self.interner.lookup(&name)));
                    } else {
                        let new_value = self.pop_unchecked();
                        let array_index = self.pop_unchecked();
                        self.stack.push(new_value.clone());
                        let value_to_be_modified = self.globals.get_mut(&name).unwrap();

                        if array_index == Value::Nil {
                            *value_to_be_modified = new_value;
                        } else {
                            set_array(value_to_be_modified, &array_index, new_value).unwrap_or_else(|err| {
                                self.runtime_error(&format!("Error setting array: {err}"));
                            });
                        }
                    }
                }
                Opcode::DefineGlobal => {
                    let name = self.read_string_or_id();
                    let value = self.pop_unchecked();
                    self.globals.insert(name, value);
                }
                Opcode::DeclareArray => {
                    let size_val = self.pop_unchecked();
                    match size_val {
                        Number(len) => {
                            self.stack.push(Value::Array(Rc::new(RefCell::new(vec![Nil; len as usize]))));
                        }
                        other => {
                            self.runtime_error(&format!("Expected number, got {other}"));
                        }
                    }
                }
                Opcode::Equal => {
                    let a = self.pop_unchecked();
                    let b = self.pop_unchecked();
                    self.stack.push(Bool(a == b))
                }
                Opcode::Nil => self.stack.push(Nil),
                Opcode::Add => {
                    let b = self.pop_unchecked();
                    let a = self.pop_unchecked();
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
                        (Number(b), Str(a)) => {
                            let mut new_string = String::from(self.interner.lookup(&a));
                            new_string.push_str(&b.to_string());
                            let id = self.interner.intern(&new_string);
                            self.stack.push(Str(id));
                        }
                        (left, right) => {
                            self.runtime_error(&format!("Operands must be numbers but got {left} {right}"));
                        }
                    }
                }
                Opcode::Subtract => binop!(self, Number, -),
                Opcode::Multiply => binop!(self, Number, *),
                Opcode::Modulo => binop!(self, Number, %),
                Opcode::Divide => binop!(self, Number, /),
                Opcode::Not => {
                    let val = self.pop_unchecked();
                    self.stack.push(Bool(self.is_falsey(&val)))
                }
                Opcode::Greater => binop!(self, Bool, >),
                Opcode::Less => binop!(self, Bool, <),
            }
        }
    }

    pub async fn interpret(functions: Vec<Fun>, interner: &'src mut Interner, read_async: F) -> Result<()> {
        dbgln!("== Interpreter VM ==");
        let mut vm = Vm::new(interner, functions, read_async);

        vm.reset_err_string();

        register_native!(vm, Clock);
        register_native!(vm, Sleep);
        register_native!(vm, TypeOf);
        register_native!(vm, Print);
        register_native!(vm, ReadNumber);
        register_native!(vm, ReadString);
        register_native!(vm, ReadBool);
        register_native!(vm, ToString);
        register_native!(vm, ToNumber);
        register_native!(vm, StringAt);
        register_native!(vm, StrLen);
        register_native!(vm, ArrLen);
        register_native!(vm, Ceil);
        register_native!(vm, Floor);
        register_native!(vm, Sort);
        register_native!(vm, IndexOf);
        register_native!(vm, Rand);
        dbgln!("Interpreting  code");
        vm.run().await
    }

    fn peek(&self, distance: usize) -> &Value {
        return self
            .stack
            .get(self.stack.len() - 1 - distance)
            .unwrap_or_else(|| panic!("Failed to peek {distance} deep"));
    }
}
