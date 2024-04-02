#!/bin/bash
cargo watch -c -w src -s 'wasm-pack build --target web --no-typescript --dev --no-pack'