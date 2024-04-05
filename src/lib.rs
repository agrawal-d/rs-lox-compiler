pub mod chunk;
pub mod common;
pub mod compiler;
pub mod scanner;
pub mod value;
pub mod vm;

use log::*;
use std::{panic, rc::Rc};
use wasm_bindgen::prelude::*;

use crate::{chunk::Chunk, vm::Vm};

#[macro_export]
macro_rules! jsprint {
    ($($arg:tt)*) => {
        crate::print(format!($($arg)*));
    };
}

#[macro_export]
macro_rules! jsprintln {
    ($($arg:tt)*) => {
        crate::println(format!($($arg)*));
    };
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(|p| {
        let s = p.to_string();
        jsprintln!("{}", s);
    }));

    console_log::init_with_level(Level::Debug).unwrap();
    info!("Rust code initialized");
    Ok(())
}

#[wasm_bindgen(module = "/src/snippet.js")]
extern "C" {
    pub fn print(output: String);
    pub fn println(output: String);
    pub fn resetOutput();
}

#[wasm_bindgen]
pub fn run_code(code: &str) {
    resetOutput();
    let source: Rc<str> = Rc::from(code);
    let chunk = compiler::Compiler::compile(source).unwrap();
    Vm::interpret(chunk).unwrap();
}
