# Rust Lox Compiler

A programming language interpreter that runs in the browser (using WASM) or locally (binary executable).

![Screenshot](./screenshot.png)

Rust implementation of [craftinginterpreters.com](https://craftinginterpreters.com/).

<center><a href="https://agrawal-d.com/rs-lox-compiler/">>>>>> Try it online <<<<<</a></center>

## Build & Run from source

1. Clone the repository and setup the Rust toolchain. Then, `cd` into the repository root.
1. Run `cargo install cargo-watch`.
1. Install wasm pack from https://rustwasm.github.io/wasm-pack.
1. Run `x.sh` (requires `bash`).
1. Open docs/index.html using a web server ( like `python3 -m http.server` ).