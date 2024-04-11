pub mod chunk;
pub mod common;
pub mod compiler;
pub mod scanner;
pub mod value;
pub mod vm;
use std::sync::OnceLock;

use crate::vm::Vm;
use std::rc::Rc;

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

pub fn init(print_fn: fn(String) -> (), println_fn: fn(String) -> ()) {
    let res = LOGGER.set(Logger { print_fn, println_fn });

    if res.is_err() {
        panic!("Compiler already initialized");
    }
}

pub fn run_code(code: &str) {
    let source: Rc<str> = Rc::from(code);
    let chunk = compiler::Compiler::compile(source).unwrap();
    Vm::interpret(chunk).unwrap();
}
