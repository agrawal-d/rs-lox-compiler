use compiler::{init, run_code};
use std::{panic, sync::atomic::AtomicBool};
use wasm_bindgen::prelude::*;

static COMPILER_INITIALIZED: AtomicBool = AtomicBool::new(false);

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(|p| {
        let s = p.to_string();
        println(s);
    }));

    Ok(())
}

#[no_mangle]
#[wasm_bindgen(module = "/src/snippet.js")]
extern "C" {
    pub fn print(output: String);
    pub fn println(output: String);
    pub fn read(text: String) -> String;
}

#[wasm_bindgen]
pub fn run(code: &str) {
    if !COMPILER_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        COMPILER_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
        init(print, println, read);
    }

    run_code(code);
}
