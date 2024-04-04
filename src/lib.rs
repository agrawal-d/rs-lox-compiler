pub mod chunk;
pub mod common;
pub mod scanner;
pub mod value;
pub mod vm;

use log::*;
use std::panic;
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
    jsprintln!("run_code called in rust with code '{code}'");
    let mut chunk = Chunk {
        code: Vec::new(),
        lines: std::collections::HashMap::new(),
        constants: value::ValueArray::new(),
    };

    let index = chunk.add_constant(1.2);
    chunk.write_constant(index, 0);
    chunk.write_opcode(common::Opcode::Return, 1);
    Vm::interpret(chunk).unwrap();
}
