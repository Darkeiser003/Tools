#!/bin/bash -eu

cd "$SRC/ltools"
cargo fuzz build --fuzz-dir fuzz -O native-argv
cp fuzz/target/x86_64-unknown-linux-gnu/release/native-argv "$OUT/native-argv"
