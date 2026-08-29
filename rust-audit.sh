#!/usr/bin/env bash

# Stable launcher for the incremental Rust auditor.
set -uo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd -P)"
for binary in "$SCRIPT_DIR/rust/target/release/ltools" "$SCRIPT_DIR/rust/target/debug/ltools"; do
    if [[ -x "$binary" ]]; then
        exec "$binary" "$@"
    fi
done
printf 'El núcleo Rust todavía no está compilado. Ejecuta:\n  cargo build --release --manifest-path %q\n' "$SCRIPT_DIR/rust/Cargo.toml" >&2
exit 2
