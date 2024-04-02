use log::*;
use std::panic;
use wasm_bindgen::prelude::*;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(|p| {
        let s = p.to_string();
        error!("{}", s);
    }));

    console_log::init_with_level(Level::Debug).unwrap();
    info!("Rust code initialized");
    Ok(())
}

#[wasm_bindgen]
pub fn run_code(code: &str) {
    info!("run_code called in rust with code '{code}'");
}
