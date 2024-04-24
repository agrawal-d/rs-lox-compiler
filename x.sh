#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR/wasm"
cargo watch -c -w src -s 'wasm-pack build --release --target web --no-typescript --no-pack' -s 'cp -r pkg/* ../docs/' 