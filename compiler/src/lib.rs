pub mod chunk;
pub mod common;
pub mod compiler;
pub mod debug;
pub mod fun;
pub mod interner;
pub mod native;
pub mod ffi;
pub mod scanner;
pub mod value;
pub mod vm;
use std::{future::Future, sync::OnceLock};

use crate::vm::Vm;
use std::rc::Rc;

const INTERNER_DEFAULT_CAP: usize = 1024;

struct Imports {
    print_fn: fn(String) -> (),
    println_fn: fn(String) -> (),
    clear_fn: fn() -> (),
}

static WRITERS: OnceLock<Imports> = OnceLock::new();

#[macro_export]
macro_rules! xprint {
    ($($arg:tt)*) => {
        ($crate::WRITERS.get().expect("Compiler not initialized").print_fn)(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! xprintln {
    ($($arg:tt)*) => {
        ($crate::WRITERS.get().expect("Compiler not initialized").println_fn)(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! xclear {
    () => {
        ($crate::WRITERS.get().expect("Compiler not initialized").clear_fn)()
    };
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

pub fn init(print_fn: fn(String) -> (), println_fn: fn(String) -> (), clear_fn: fn() -> ()) {
    let res = WRITERS.set(Imports {
        print_fn,
        println_fn,
        clear_fn,
    });

    if res.is_err() {
        panic!("Compiler already initialized");
    }
}

pub async fn run_code<F, Fut, SF, SFut>(code: &str, read_async: F, sleep_async: SF)
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = String>,
    SF: Fn(u64) -> SFut,
    SFut: Future<Output = ()>,
{
    let source: Rc<str> = Rc::from(code);
    let mut interner = interner::Interner::with_capacity(INTERNER_DEFAULT_CAP);
    let mut functions: Vec<fun::Fun> = Vec::new();
    let (fun, had_error) = compiler::Compiler::compile(source, None, &mut interner, &mut functions, fun::FunType::Script).unwrap();
    if !had_error {
        functions.push(fun);
        Vm::interpret(functions, &mut interner, read_async, sleep_async).await.unwrap();
    }
}

pub async fn run_file<F, Fut, SF, SFut>(file_path: &str, read_async: F, sleep_async: SF)
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = String>,
    SF: Fn(u64) -> SFut,
    SFut: Future<Output = ()>,
{
    use std::fs;
    use std::path::Path;

    let path = Path::new(file_path);
    let current_dir = path.parent().map(|p| p.to_path_buf());
    let code = fs::read_to_string(path).expect("Failed to read file");

    let source: Rc<str> = Rc::from(code);
    let mut interner = interner::Interner::with_capacity(INTERNER_DEFAULT_CAP);
    let mut functions: Vec<fun::Fun> = Vec::new();
    let (fun, had_error) = compiler::Compiler::compile(source, current_dir, &mut interner, &mut functions, fun::FunType::Script).unwrap();
    if !had_error {
        functions.push(fun);
        Vm::interpret(functions, &mut interner, read_async, sleep_async).await.unwrap();
    }
}
