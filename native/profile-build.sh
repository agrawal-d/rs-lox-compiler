#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $DIR
cargo +nightly build -Z build-std --target x86_64-unknown-linux-gnu --release
sudo $(which flamegraph) -o $DIR/flamegraph.svg -- $DIR/../target/x86_64-unknown-linux-gnu/release/native $DIR/code.cs