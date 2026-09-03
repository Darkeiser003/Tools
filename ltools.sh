#!/usr/bin/env bash
# Lanzador compatible de LTools. El backend es siempre Rust.
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

ARGS=()
for arg in "$@"; do
    [[ "$arg" == "--rust" ]] || ARGS+=("$arg")
done
# El lanzador normal conserva el perfil gráfico del backend. El lanzador
# separado ltools-cli.sh fuerza la ayuda/salida de consola. Esta fachada de
# checkout conserva además el menú si se invoca sin argumentos, como esperan
# los usos desde una terminal y las pruebas del repositorio; las releases
# gráficas (AppImage/EXE) entran directamente por su ventana nativa.
# Si un integrador ya ha seleccionado el perfil CLI mediante LTOOLS_CLI=1,
# también se conserva aquí sin convertir este lanzador en backend funcional.
if [[ "${LTOOLS_CLI:-0}" == 1 ]]; then
    export LTOOLS_NO_AUTO_TERMINAL=1
elif [[ "${#ARGS[@]}" -eq 0 ]]; then
    ARGS=(menu)
fi
exec "$BIN" "${ARGS[@]}"
