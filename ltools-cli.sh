#!/usr/bin/env bash
# Perfil CLI de LTools: no abre una terminal ni entra en el menú.
# Sin argumentos muestra la ayuda; con argumentos ejecuta el comando indicado.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
BIN="${LTOOLS_BINARY:-}"
if [[ -z "$BIN" ]]; then
    for candidate in \
        "$ROOT_DIR/rust/target/release/ltools" \
        "$ROOT_DIR/rust/target/debug/ltools"; do
        if [[ -x "$candidate" ]]; then
            BIN="$candidate"
            break
        fi
    done
fi

if [[ -z "$BIN" || ! -x "$BIN" ]]; then
    printf 'El backend Rust no está compilado. Ejecuta:\n  cargo build --release --manifest-path %q\n' \
        "$ROOT_DIR/rust/Cargo.toml" >&2
    exit 2
fi

export LTOOLS_CLI=1
export LTOOLS_NO_AUTO_TERMINAL=1
exec "$BIN" "$@"
