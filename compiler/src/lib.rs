pub mod chunk;
pub mod common;
pub mod compiler;
pub mod debug;
pub mod fun;
pub mod interner;
pub mod scanner;
pub mod value;
pub mod vm;
use std::sync::OnceLock;

use crate::vm::Vm;
use std::rc::Rc;

const INTERNER_DEFAULT_CAP: usize = 1024;

struct Logger {
    print_fn: fn(String) -> (),
    println_fn: fn(String) -> (),
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[macro_export]
macro_rules! xprint {
    ($($arg:tt)*) => {
        ($crate::LOGGER.get().expect("Compiler not initialized").print_fn)(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! xprintln {
    ($($arg:tt)*) => {
        ($crate::LOGGER.get().expect("Compiler not initialized").println_fn)(format!($($arg)*))
    }
}

#[macro_export]
#[cfg(not(feature = "tracing"))]
macro_rules! dbg {
    // No-op version
    ($($arg:tt)*) => {};
}

#[macro_export]
#[cfg(not(feature = "tracing"))]
macro_rules! dbgln {
    // No-op version
    ($($arg:tt)*) => {};
}

#[macro_export]
#[cfg(feature = "tracing")]
macro_rules! dbg {
    ($($arg:tt)*) => {
        $crate::xprint!($($arg)*)
    }
}

#[macro_export]
#[cfg(feature = "tracing")]
macro_rules! dbgln {
    ($($arg:tt)*) => {
        $crate::xprintln!($($arg)*)
    }
}

pub fn init(print_fn: fn(String) -> (), println_fn: fn(String) -> ()) {
    let res = LOGGER.set(Logger { print_fn, println_fn });

    if res.is_err() {
        panic!("Compiler already initialized");
    }
}

pub fn run_code(code: &str) {
    let source: Rc<str> = Rc::from(code);
    let mut interner = interner::Interner::with_capacity(INTERNER_DEFAULT_CAP);
    let mut functions: Vec<fun::Fun> = Vec::new();
    let fun = compiler::Compiler::compile(source, &mut interner, &mut functions, fun::FunType::Script).unwrap();
    Vm::interpret(fun.chunk, &mut interner).unwrap();
}
