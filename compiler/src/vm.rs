use std::rc::Rc;
use std::{cell::RefCell, future::Future};

use crate::{
    common::Opcode,
    dbgln,
    fun::Fun,
    interner::{Interner, StrId},
    native::*,
    value::{
        print_value, ClassData, InstanceData,
        Value::{self, *},
    },
};

#[cfg(feature = "tracing")]
use crate::debug::disassemble_instruction;
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
    pub arg_count: usize,
}

pub const ERR_STRING: &str = "errString";

thread_local! {
    pub static RUNNING_FUNCTIONS: std::cell::RefCell<Option<*const Vec<crate::fun::Fun>>> = std::cell::RefCell::new(None);
}

pub struct Vm<'src, F, Fut, SF, SFut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = String>,
    SF: Fn(u64) -> SFut,
    SFut: Future<Output = ()>,
{
    frames: Vec<CallFrame>,
    pub functions: Vec<Fun>,
    stack: Vec<Value>,
    pub interner: &'src mut Interner,
    globals: FxHashMap<StrId, Value>,
    global_error_id: StrId, // StrId of global error variable
    read_async: F,
    sleep_async: SF,
    #[cfg(not(target_arch = "wasm32"))]
    loaded_libs: Vec<libloading::Library>,
    #[cfg(target_arch = "wasm32")]
    loaded_libs: Vec<Box<dyn std::any::Any>>,
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
        let func = Rc::new($name);
        let name_str = func.name();
        let name = $vm.interner.intern(name_str);
        dbgln!("Registering native function {}", name_str);
        $vm.globals.insert(name, Value::NativeFunction(func));
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
        (Value::Buffer(buf), Value::Number(index)) => {
            let index = *index as usize;
            let bytes = buf.borrow();
            if index < bytes.len() {
                Ok(Value::Number(bytes[index] as f64))
            } else {
                bail!("Index out of bounds: {index}")
            }
        }
        (Value::Map(map), key) => {
            if let Some(val) = map.borrow().get(key) {
                Ok(val.clone())
            } else {
                Ok(Value::Nil)
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
        (Value::Buffer(buf), Value::Number(index)) => {
            let index = *index as usize;
            let mut bytes = buf.borrow_mut();
            if index < bytes.len() {
                match new_value {
                    Value::Number(n) => {
                        bytes[index] = n as u8;
                        Ok(())
                    }
                    _ => bail!("Buffer element must be a byte number"),
                }
            } else {
                bail!("Index out of bounds: {index}")
            }
        }
        (Value::Map(map), key) => {
            map.borrow_mut().insert(key.clone(), new_value);
            Ok(())
        }
        (arr, index) => {
            bail!(format!("Tried to index value of type {arr} with index {index}"));
        }
    }
}

impl<'src, F, Fut, SF, SFut> Vm<'src, F, Fut, SF, SFut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = String>,
    SF: Fn(u64) -> SFut,
    SFut: Future<Output = ()>,
{
    pub fn new(interner: &'src mut Interner, functions: Vec<Fun>, read_async: F, sleep_async: SF) -> Vm<'src, F, Fut, SF, SFut> {
        let global_error_id = interner.intern(ERR_STRING);

        let mut frames: Vec<CallFrame> = Vec::with_capacity(10240);
        frames.push(CallFrame {
            fun_idx: functions.len() - 1,
            ip: 0,
            start_len: 0,
            slot_offset: 0,
            arg_count: 0,
        });

        let vm = Vm {
            frames,
            functions,
            stack: Vec::with_capacity(1024),
            interner,
            globals: FxHashMap::default(),
            global_error_id,
            read_async,
            sleep_async,
            loaded_libs: Vec::new(),
        };

        vm
    }

    pub fn new_repl(interner: &'src mut Interner, read_async: F, sleep_async: SF) -> Vm<'src, F, Fut, SF, SFut> {
        let global_error_id = interner.intern(ERR_STRING);
        let mut vm = Vm {
            frames: Vec::with_capacity(10240),
            functions: Vec::new(),
            stack: Vec::with_capacity(1024),
            interner,
            globals: FxHashMap::default(),
            global_error_id,
            read_async,
            sleep_async,
            loaded_libs: Vec::new(),
        };

        register_native!(vm, Clock);
        register_native!(vm, Sleep);
        register_native!(vm, TypeOf);
        register_native!(vm, Print);
        register_native!(vm, Printf);
        register_native!(vm, ReadString);
        register_native!(vm, StrCast);
        register_native!(vm, BufCast);
        register_native!(vm, ChrCast);
        register_native!(vm, HelpCast);
        register_native!(vm, IntCast);
        register_native!(vm, FloatCast);
        register_native!(vm, BoolCast);
        register_native!(vm, StringAt);
        register_native!(vm, Len);
        register_native!(vm, Ceil);
        register_native!(vm, Floor);
        register_native!(vm, Abs);
        register_native!(vm, Sort);
        register_native!(vm, IndexOf);
        register_native!(vm, Rand);
        register_native!(vm, Clear);
        register_native!(vm, Sin);
        register_native!(vm, Cos);
        register_native!(vm, Sqrt);
        register_native!(vm, Pow);
        register_native!(vm, Pi);
        register_native!(vm, MapConstructor);
        register_native!(vm, Keys);
        register_native!(vm, Values);
        register_native!(vm, Has);

        vm
    }

    fn load_native_imports(&mut self, fun_idx: usize) {
        let imports = self.functions[fun_idx].native_imports.clone();
        for (path, alias) in imports {
            match crate::ffi::load_native_module(&path, &alias, self.interner, &mut self.globals) {
                std::result::Result::Ok(lib) => {
                    self.loaded_libs.push(lib);
                }
                std::result::Result::Err(e) => {
                    self.runtime_error(&format!("Failed to load native module '{}': {}", path, e));
                }
            }
        }
    }

    pub async fn run_repl_chunk(&mut self, fun: Fun) -> Result<()> {
        self.stack.clear();
        self.frames.clear();

        let fun_idx = self.functions.len();
        self.functions.push(fun);

        self.load_native_imports(fun_idx);

        self.frames.push(CallFrame {
            fun_idx,
            ip: 0,
            start_len: 0,
            slot_offset: 0,
            arg_count: 0,
        });

        self.reset_err_string();
        self.run().await
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
            Buffer(buf) => buf.borrow().is_empty(),
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
        let callee = self.peek(arg_count as usize).clone();
        match &callee {
            Function(idx) => {
                let fun = &self.functions[*idx];

                let arg_count_usize = arg_count as usize;
                if arg_count_usize < fun.min_arity || arg_count_usize > fun.arity {
                    self.runtime_error(&format!(
                        "Expected between {} and {} arguments but got {} instead",
                        fun.min_arity, fun.arity, arg_count
                    ));
                }

                // If fewer than fun.arity arguments were passed, push Nil placeholders for the remaining parameters
                for _ in arg_count_usize..fun.arity {
                    self.stack.push(Value::Nil);
                }

                let new_frame_offset = self.stack.len() - fun.arity;
                let orig_len = self.stack.len() - 1 - fun.arity;
                let frame: CallFrame = CallFrame {
                    fun_idx: *idx,
                    ip: 0,
                    start_len: orig_len,
                    slot_offset: new_frame_offset,
                    arg_count: arg_count_usize,
                };
                self.frames.push(frame);
                true
            }
            Class(class) => {
                let instance = Rc::new(RefCell::new(InstanceData {
                    class: Rc::clone(class),
                    fields: RefCell::new(rustc_hash::FxHashMap::default()),
                }));

                let callee_slot = self.stack.len() - 1 - arg_count as usize;
                self.stack[callee_slot] = Value::Instance(Rc::clone(&instance));

                let constructor_id = class.name;
                if let Some(method_idx) = class.methods.borrow().get(&constructor_id).copied() {
                    let fun = &self.functions[method_idx];
                    let arg_count_usize = arg_count as usize;
                    if arg_count_usize < fun.min_arity || arg_count_usize > fun.arity {
                        self.runtime_error(&format!(
                            "Expected between {} and {} arguments but got {} instead",
                            fun.min_arity, fun.arity, arg_count
                        ));
                    }
                    for _ in arg_count_usize..fun.arity {
                        self.stack.push(Value::Nil);
                    }
                    let new_frame_offset = self.stack.len() - fun.arity;
                    let orig_len = self.stack.len() - 1 - fun.arity;
                    let frame: CallFrame = CallFrame {
                        fun_idx: method_idx,
                        ip: 0,
                        start_len: orig_len,
                        slot_offset: new_frame_offset,
                        arg_count: arg_count_usize,
                    };
                    self.frames.push(frame);
                } else {
                    if arg_count > 0 {
                        self.runtime_error("Constructor expected 0 arguments but got some");
                    }
                    self.stack.truncate(callee_slot + 1);
                }
                true
            }
            BoundMethod { instance, method_idx } => {
                let callee_slot = self.stack.len() - 1 - arg_count as usize;
                self.stack[callee_slot] = Value::Instance(Rc::clone(instance));

                let fun = &self.functions[*method_idx];
                let arg_count_usize = arg_count as usize;
                if arg_count_usize < fun.min_arity || arg_count_usize > fun.arity {
                    self.runtime_error(&format!(
                        "Expected between {} and {} arguments but got {} instead",
                        fun.min_arity, fun.arity, arg_count
                    ));
                }
                for _ in arg_count_usize..fun.arity {
                    self.stack.push(Value::Nil);
                }
                let new_frame_offset = self.stack.len() - fun.arity;
                let orig_len = self.stack.len() - 1 - fun.arity;
                let frame: CallFrame = CallFrame {
                    fun_idx: *method_idx,
                    ip: 0,
                    start_len: orig_len,
                    slot_offset: new_frame_offset,
                    arg_count: arg_count_usize,
                };
                self.frames.push(frame);
                true
            }
            NativeFunction(fun) => {
                let mut arg_count_usize = arg_count as usize;
                let name = fun.name();

                let is_input = name == "input";
                let is_printf = name == "printf";
                let valid_arity = if is_input {
                    arg_count_usize <= 1
                } else if is_printf {
                    arg_count_usize >= 1 // printf needs at least format string
                } else {
                    arg_count_usize == fun.arity()
                };

                if !valid_arity {
                    self.runtime_error(&format!("Expected {} arguments but got {} instead", fun.arity(), arg_count));
                }

                if is_input && arg_count_usize == 0 {
                    let default_prompt = Value::Str(self.interner.intern(""));
                    self.stack.push(default_prompt);
                    arg_count_usize = 1;
                }

                let function = fun.clone();

                // Special read input functions - take the prompt, and convert it to the user response
                if function.name() == "input" {
                    let len = self.stack.len();
                    let first_arg = &mut self.stack[len - arg_count_usize];
                    match first_arg {
                        Value::Str(id) => {
                            let prompt = self.interner.lookup(id);
                            let input = (self.read_async)(prompt.to_string()).await;
                            *first_arg = Value::Str(self.interner.intern(&input));
                        }
                        _ => *first_arg = Value::Nil,
                    }
                };

                // Special sleep function - await the async sleep hook directly
                if function.name() == "sleep" {
                    let ms = match self.stack.last() {
                        Some(Value::Number(n)) => *n as u64,
                        _ => 0,
                    };
                    (self.sleep_async)(ms).await;
                    self.stack.truncate(self.stack.len() - 1 - arg_count_usize);
                    self.stack.push(Value::Nil);
                    return true;
                }

                let args = &self.stack[self.stack.len() - arg_count_usize..];
                self.globals.insert(self.global_error_id, Value::Nil); // Reset error string

                let result = function.call(self.interner, &mut self.globals, args);

                self.stack.truncate(self.stack.len() - 1 - arg_count_usize);
                self.stack.push(result);

                true
            }
            other => {
                self.runtime_error(&format!("Can only call functions, got {other}"));
            }
        }
    }

    async fn run(&mut self) -> Result<()> {
        struct RunningFunctionsGuard;
        impl Drop for RunningFunctionsGuard {
            fn drop(&mut self) {
                RUNNING_FUNCTIONS.with(|funcs| {
                    *funcs.borrow_mut() = None;
                });
            }
        }
        RUNNING_FUNCTIONS.with(|funcs| {
            *funcs.borrow_mut() = Some(&self.functions as *const Vec<crate::fun::Fun>);
        });
        let _guard = RunningFunctionsGuard;

        loop {
            #[cfg(feature = "tracing")]
            {
                self.stack_trace();
                disassemble_instruction(&self.functions[frame!(self).fun_idx].chunk, frame!(self).ip, self.interner);
            }
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
                Opcode::DefaultArg => {
                    let arg_index = self.read_byte() as usize;
                    let offset = self.read_u16() as usize;
                    if frame!(self).arg_count >= arg_index {
                        frame_mut!(self).ip += offset;
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
                Opcode::Dup => {
                    let val = self.peek(0).clone();
                    self.stack.push(val);
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
                Opcode::ArrayLiteral => {
                    let count = self.read_byte() as usize;
                    let start = self.stack.len() - count;
                    let elements: Vec<Value> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Array(Rc::new(RefCell::new(elements))));
                }
                Opcode::Equal => {
                    let a = self.pop_unchecked();
                    let b = self.pop_unchecked();
                    self.stack.push(Bool(a == b))
                }
                Opcode::Nil => self.stack.push(Nil),
                Opcode::Class => {
                    let name = self.read_string_or_id();
                    let class = Rc::new(ClassData {
                        name,
                        methods: RefCell::new(rustc_hash::FxHashMap::default()),
                    });
                    self.stack.push(Value::Class(class));
                }
                Opcode::Method => {
                    let name = self.read_string_or_id();
                    let method_val = self.pop_unchecked();
                    if let Value::Function(idx) = method_val {
                        if let Some(Value::Class(class)) = self.stack.last() {
                            class.methods.borrow_mut().insert(name, idx);
                        }
                    }
                }
                Opcode::GetProperty => {
                    let name = self.read_string_or_id();
                    let object = self.pop_unchecked();
                    match object {
                        Value::Instance(instance) => {
                            if let Some(value) = instance.borrow().fields.borrow().get(&name).cloned() {
                                self.stack.push(value);
                            } else {
                                let method_idx = instance.borrow().class.methods.borrow().get(&name).copied();
                                if let Some(idx) = method_idx {
                                    self.stack.push(Value::BoundMethod {
                                        instance: Rc::clone(&instance),
                                        method_idx: idx,
                                    });
                                } else {
                                    self.stack.push(Value::Nil);
                                }
                            }
                        }
                        Value::Map(map) => {
                            let key = Value::Str(name);
                            if let Some(value) = map.borrow().get(&key).cloned() {
                                self.stack.push(value);
                            } else {
                                self.stack.push(Value::Nil);
                            }
                        }
                        _ => {
                            self.runtime_error("Only instances and maps have properties.");
                        }
                    }
                }
                Opcode::SetProperty => {
                    let name = self.read_string_or_id();
                    let value = self.pop_unchecked();
                    let object = self.pop_unchecked();
                    match object {
                        Value::Instance(instance) => {
                            instance.borrow().fields.borrow_mut().insert(name, value.clone());
                            self.stack.push(value);
                        }
                        Value::Map(map) => {
                            let key = Value::Str(name);
                            map.borrow_mut().insert(key, value.clone());
                            self.stack.push(value);
                        }
                        _ => {
                            self.runtime_error("Only instances and maps have properties.");
                        }
                    }
                }
                Opcode::GetReceiver => {
                    let receiver = self.stack[frame!(self).slot_offset - 1].clone();
                    self.stack.push(receiver);
                }
                Opcode::GetIndex => {
                    let index = self.pop_unchecked();
                    let arr = self.pop_unchecked();
                    self.stack.push(get_array(&arr, &index).unwrap_or_else(|err| {
                        self.runtime_error(&format!("Error getting array: {err}"));
                    }));
                }
                Opcode::SetIndex => {
                    let new_value = self.pop_unchecked();
                    let index = self.pop_unchecked();
                    let mut arr = self.pop_unchecked();
                    set_array(&mut arr, &index, new_value.clone()).unwrap_or_else(|err| {
                        self.runtime_error(&format!("Error setting array: {err}"));
                    });
                    self.stack.push(new_value);
                }
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

    pub async fn interpret(functions: Vec<Fun>, interner: &'src mut Interner, read_async: F, sleep_async: SF) -> Result<()> {
        dbgln!("== Interpreter VM ==");
        let mut vm = Vm::new(interner, functions, read_async, sleep_async);

        vm.reset_err_string();

        register_native!(vm, Clock);
        register_native!(vm, Sleep);
        register_native!(vm, TypeOf);
        register_native!(vm, Print);
        register_native!(vm, Printf);
        register_native!(vm, ReadString);
        register_native!(vm, StrCast);
        register_native!(vm, BufCast);
        register_native!(vm, ChrCast);
        register_native!(vm, HelpCast);
        register_native!(vm, IntCast);
        register_native!(vm, FloatCast);
        register_native!(vm, BoolCast);
        register_native!(vm, StringAt);
        register_native!(vm, Len);
        register_native!(vm, Ceil);
        register_native!(vm, Floor);
        register_native!(vm, Abs);
        register_native!(vm, Sort);
        register_native!(vm, IndexOf);
        register_native!(vm, Rand);
        register_native!(vm, Clear);
        register_native!(vm, Sin);
        register_native!(vm, Cos);
        register_native!(vm, Sqrt);
        register_native!(vm, Pow);
        register_native!(vm, Pi);
        register_native!(vm, MapConstructor);
        register_native!(vm, Keys);
        register_native!(vm, Values);
        register_native!(vm, Has);

        vm.load_native_imports(vm.functions.len() - 1);

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
