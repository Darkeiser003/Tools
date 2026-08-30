#!/usr/bin/env bash
# Lanzador compatible de LTools. El backend es siempre Rust.
# Los módulos Bash históricos viven en legacy/bash y no forman parte del
# funcionamiento normal ni del paquete distribuible.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

if [[ "${1:-}" == "--bash" ]]; then
    printf 'La implementación Bash es legacy y ya no se usa como backend. Usa el binario Rust.\n' >&2
    exit 2
fi

if [[ "${1:-}" == "--module-help" ]]; then
    printf 'LTools usa un único backend Rust; consulta la ayuda completa con --help.\n'
    exit 0
fi

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

ARGS=()
for arg in "$@"; do
    [[ "$arg" == "--rust" ]] || ARGS+=("$arg")
done
if [[ "${#ARGS[@]}" -eq 0 ]]; then
    ARGS=(menu)
fi
exec "$BIN" "${ARGS[@]}"
