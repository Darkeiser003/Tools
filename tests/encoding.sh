#!/usr/bin/env bash
# Comprueba las codificaciones que requiere cada familia de archivos.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
failures=0

fail() {
    printf 'ENCODING ERROR: %s\n' "$1" >&2
    failures=$((failures + 1))
}

hex_prefix() {
    od -An -tx1 -N3 -- "$1" | tr -d '[:space:]'
}

utf8_check() {
    local file="$1"
    iconv -f UTF-8 -t UTF-8 -- "$file" >/dev/null 2>&1 ||
        fail "UTF-8 invalido: ${file#$ROOT_DIR/}"
}

no_bom_check() {
    local file="$1"
    [[ "$(hex_prefix "$file")" != efbbbf ]] ||
        fail "BOM no permitido: ${file#$ROOT_DIR/}"
}

utf8_bom_check() {
    local file="$1"
    [[ "$(hex_prefix "$file")" == efbbbf ]] ||
        fail "PowerShell requiere UTF-8 BOM: ${file#$ROOT_DIR/}"
}

cmd_ansi_check() {
    local file="$1"
    no_bom_check "$file"
    # CMD debe mantenerse en ASCII/ANSI seguro: los bytes ASCII son validos
    # tanto en las paginas de codigo Windows habituales como en UTF-8.
    local non_ascii
    non_ascii="$(LC_ALL=C tr -d '\000-\177\r\n\t' < "$file" | od -An -tx1 | tr -d '[:space:]')"
    [[ -z "$non_ascii" ]] || fail "CMD contiene bytes no ANSI-seguros: ${file#$ROOT_DIR/}"
}

while IFS= read -r -d '' file; do
    case "$file" in
        *.ps1) utf8_bom_check "$file"; utf8_check "$file" ;;
        *.cmd) cmd_ansi_check "$file"; utf8_check "$file" ;;
        *) no_bom_check "$file"; utf8_check "$file" ;;
    esac
done < <(
    rg --files --hidden -0 \
        -g '!.git/**' -g '!dist/**' -g '!rust/target/**' \
        "$ROOT_DIR"
)

if (( failures )); then
    exit 1
fi
printf 'Codificaciones correctas: UTF-8 sin BOM para fuentes/JSON/Bash, UTF-8 BOM para PowerShell y ASCII/ANSI seguro para CMD.\n'
