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
        end();
    }));

    Ok(())
}

#[no_mangle]
#[wasm_bindgen(module = "/src/snippet.js")]
extern "C" {
    pub fn print(output: String);
    pub fn println(output: String);
    pub fn end();
    pub async fn sleep(ms: u32);
    pub async fn readAsync(text: String) -> JsValue;
}

async fn read_async(text: String) -> String {
    readAsync(text).await.as_string().unwrap_or_default()
}

#[wasm_bindgen]
pub async fn run(code: &str) {
    if !COMPILER_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        COMPILER_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
        init(print, println);
    }

    run_code(code, read_async).await;
}
