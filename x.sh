#!/bin/bash
cargo watch -c -w src -s 'wasm-pack build --target web --no-typescript --dev --no-pack --features tracing,print_code' -s 'cp -r pkg/* docs/' 