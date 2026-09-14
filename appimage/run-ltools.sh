#!/usr/bin/env bash
# Lanzador tolerante para AppImage: usa FUSE cuando existe y, si no,
# activa la extracción temporal soportada por el runtime de AppImage.

set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
appimage="${LTOOLS_APPIMAGE:-}"

if [[ "${1:-}" == --appimage ]]; then
    [[ $# -ge 2 ]] || { printf 'Uso: %s [--appimage RUTA] [opciones...]\n' "$0" >&2; exit 2; }
    appimage="$2"
    shift 2
elif [[ "${1:-}" == *.AppImage && -f "${1:-}" ]]; then
    appimage="$1"
    shift
fi

if [[ -z "$appimage" ]]; then
    while IFS= read -r candidate; do
        appimage="$candidate"
        break
    done < <(find "$SCRIPT_DIR" -maxdepth 1 -type f -name 'ltools-*.AppImage' -print | sort -V -r)
fi

if [[ -z "$appimage" || ! -f "$appimage" ]]; then
    printf 'No se encontró el AppImage junto al lanzador.\n' >&2
    printf 'Usa --appimage /ruta/ltools.AppImage para indicarlo.\n' >&2
    exit 1
fi

if [[ "${APPIMAGE_EXTRACT_AND_RUN:-0}" == 1 ]]; then
    exec "$appimage" "$@"
fi

run_with_fuse_fallback() {
    local temp_dir tee_pid status
    if ! temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/ltools-fuse.XXXXXX")"; then
        printf 'No se pudo preparar la comprobación de FUSE; se usará extracción temporal.\n' >&2
        APPIMAGE_EXTRACT_AND_RUN=1 "$appimage" "$@"
        return $?
    fi
    if ! mkfifo "$temp_dir/stderr.fifo"; then
        rmdir -- "$temp_dir" 2>/dev/null || true
        printf 'No se pudo comprobar FUSE; se usará extracción temporal.\n' >&2
        APPIMAGE_EXTRACT_AND_RUN=1 "$appimage" "$@"
        return $?
    fi

    tee "$temp_dir/stderr.log" <"$temp_dir/stderr.fifo" >&2 &
    tee_pid=$!
    "$appimage" "$@" 2>"$temp_dir/stderr.fifo"
    status=$?
    wait "$tee_pid" 2>/dev/null || true

    if [[ "$status" -eq 0 ]]; then
        rm -f -- "$temp_dir/stderr.fifo" "$temp_dir/stderr.log"
        rmdir -- "$temp_dir" 2>/dev/null || true
        return 0
    fi
    if grep -Eiq 'Cannot mount AppImage|mount failed:|Operation not permitted|Cannot access /dev/fuse|fusermount.*failed' "$temp_dir/stderr.log"; then
        rm -f -- "$temp_dir/stderr.fifo" "$temp_dir/stderr.log"
        rmdir -- "$temp_dir" 2>/dev/null || true
        printf 'FUSE está anunciado, pero el montaje fue rechazado; se reintentará por extracción temporal.\n' >&2
        APPIMAGE_EXTRACT_AND_RUN=1 "$appimage" "$@"
        return $?
    fi

    rm -f -- "$temp_dir/stderr.fifo" "$temp_dir/stderr.log"
    rmdir -- "$temp_dir" 2>/dev/null || true
    return "$status"
}

if [[ "${LTOOLS_FORCE_EXTRACT:-0}" != 1 ]] &&
    [[ -c /dev/fuse ]] &&
    { command -v fusermount3 >/dev/null 2>&1 || command -v fusermount >/dev/null 2>&1; }; then
    run_with_fuse_fallback "$@"
    exit $?
fi

printf 'FUSE no está disponible en este sistema; se usará extracción temporal.\n' >&2
printf 'Para habilitar el montaje normal, revisa: %s --fuse-check\n' "$appimage" >&2
export APPIMAGE_EXTRACT_AND_RUN=1
exec "$appimage" "$@"
