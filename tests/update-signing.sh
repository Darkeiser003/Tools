#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
source "$ROOT_DIR/scripts/lib/signing.sh"

fail() { printf 'UPDATE SIGNING ERROR: %s\n' "$1" >&2; exit 1; }

LTOOLS_UPDATE_PUBLIC_KEY=$'  PRIMARY\nKEY  '
LTERMINAL_UPDATE_PUBLIC_KEY='FALLBACKKEY'
[[ "$(ltools_effective_update_public_key)" == 'PRIMARYKEY' ]] ||
    fail 'la clave primaria válida debe prevalecer y normalizar espacios'

LTOOLS_UPDATE_PUBLIC_KEY=$' \n\t '
[[ "$(ltools_effective_update_public_key)" == 'FALLBACKKEY' ]] ||
    fail 'la clave primaria en blanco debe permitir usar la variable heredada'

LTERMINAL_UPDATE_PUBLIC_KEY=$' \n\t '
if ltools_effective_update_public_key >/dev/null; then
    fail 'no debe devolver una clave cuando ambas variables están vacías'
fi

printf 'Precedencia y fallback de claves públicas verificados.\n'
