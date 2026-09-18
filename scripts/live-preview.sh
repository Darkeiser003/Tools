#!/usr/bin/env bash
# Preview GUI vigilado en un target debug aislado; nunca empaqueta ni toca release/.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
MANIFEST="$ROOT_DIR/rust/Cargo.toml"
TARGET_DIR="$ROOT_DIR/rust/target/live-preview"
PREVIEW_BIN="$TARGET_DIR/debug/ltools"
PREVIEW_PID=''
INTERVAL="${LTOOLS_PREVIEW_INTERVAL:-1}"

usage() {
    printf 'Uso: bash scripts/live-preview.sh [--help]\n'
    printf 'Compila en rust/target/live-preview, abre la GUI y la reinicia al cambiar fuentes. Ctrl+C detiene el vigilante.\n'
}

if [[ "${1:-}" == -h || "${1:-}" == --help ]]; then usage; exit 0; fi
if (($#)); then usage >&2; exit 2; fi
if ! [[ "$INTERVAL" =~ ^([1-9][0-9]*|0\.[0-9]+)$ ]]; then
    printf 'LTOOLS_PREVIEW_INTERVAL debe ser un número positivo de segundos.\n' >&2
    exit 2
fi
if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
    printf 'El preview GUI necesita una sesión gráfica (DISPLAY o WAYLAND_DISPLAY).\n' >&2
    exit 2
fi

source_fingerprint() {
    local path
    {
        find "$ROOT_DIR/rust/src" "$ROOT_DIR/rust/crates" "$ROOT_DIR/rust/.cargo" \
            -type f -print0 2>/dev/null || true
        for path in "$ROOT_DIR/rust/Cargo.toml" "$ROOT_DIR/rust/Cargo.lock" "$ROOT_DIR/rust/build.rs" \
            "$ROOT_DIR/rust/rust-toolchain" "$ROOT_DIR/rust/rust-toolchain.toml"; do
            [[ ! -f "$path" ]] || printf '%s\0' "$path"
        done
    } | sort -zu | while IFS= read -r -d '' path; do sha256sum -- "$path"; done | sha256sum | awk '{print $1}'
}

stop_preview() {
    if [[ -n "$PREVIEW_PID" ]] && kill -0 "$PREVIEW_PID" 2>/dev/null; then
        kill -TERM "$PREVIEW_PID" 2>/dev/null || true
        wait "$PREVIEW_PID" 2>/dev/null || true
    fi
    PREVIEW_PID=''
}
finish() { stop_preview; }
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

start_preview() {
    # El preview debe demostrar la interfaz que se va a revisar. No debe
    # ocultar un fallo GTK entrando silenciosamente en el menú de consola.
    # En X11 desactivamos portales para que funcione igual bajo Xvfb y en una
    # sesión local; en Wayland se conserva el backend elegido por el usuario.
    if [[ -n "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
        export GDK_BACKEND="${GDK_BACKEND:-x11}"
    fi
    LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0 GTK_USE_PORTAL=0 \
        "$PREVIEW_BIN" &
    PREVIEW_PID=$!
}

printf 'Preview GUI vigilado (target aislado: %s). Ctrl+C para terminar.\n' "$TARGET_DIR"
last_fingerprint=''
while true; do
    before="$(source_fingerprint)"
    if [[ "$before" != "$last_fingerprint" || ! -x "$PREVIEW_BIN" ]]; then
        stop_preview
        printf '\nCambios detectados; compilando solo el perfil debug de preview...\n'
        if ! CARGO_TARGET_DIR="$TARGET_DIR" cargo build --manifest-path "$MANIFEST"; then
            printf 'La compilación falló. Se reintentará cuando cambie algún archivo vigilado.\n' >&2
            last_fingerprint="$(source_fingerprint)"
            sleep "$INTERVAL"
            continue
        fi
        after="$(source_fingerprint)"
        if [[ "$before" != "$after" ]]; then
            printf 'Las fuentes cambiaron durante la compilación; se repetirá con el estado nuevo.\n'
            last_fingerprint=''
            continue
        fi
        last_fingerprint="$after"
        printf 'Abriendo preview GUI.\n'
        start_preview
    fi

    sleep "$INTERVAL"
    current="$(source_fingerprint)"
    if [[ "$current" != "$last_fingerprint" ]]; then
        printf '\nCambio detectado; cerrando solo el preview y recompilando.\n'
        stop_preview
        last_fingerprint=''
    elif [[ -n "$PREVIEW_PID" ]] && ! kill -0 "$PREVIEW_PID" 2>/dev/null; then
        wait "$PREVIEW_PID" 2>/dev/null || true
        PREVIEW_PID=''
        printf 'La ventana se cerró; el vigilante continúa esperando cambios.\n'
    fi
done
