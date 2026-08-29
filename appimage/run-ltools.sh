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

if [[ "${LTOOLS_FORCE_EXTRACT:-0}" != 1 ]] &&
    [[ -c /dev/fuse ]] &&
    { command -v fusermount3 >/dev/null 2>&1 || command -v fusermount >/dev/null 2>&1; }; then
    exec "$appimage" "$@"
fi

printf 'FUSE no está disponible en este sistema; se usará extracción temporal.\n' >&2
printf 'Para habilitar el montaje normal, revisa: %s --fuse-check\n' "$appimage" >&2
export APPIMAGE_EXTRACT_AND_RUN=1
exec "$appimage" "$@"
