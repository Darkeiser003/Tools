#!/usr/bin/env bash

# Devuelve la primera clave pública de entorno que no esté vacía tras quitar
# espacios. Mantiene la precedencia LTOOLS_* sobre el alias LTERMINAL_*.
ltools_effective_update_public_key() {
    local candidate
    for candidate in "${LTOOLS_UPDATE_PUBLIC_KEY:-}" "${LTERMINAL_UPDATE_PUBLIC_KEY:-}"; do
        candidate="$(printf '%s' "$candidate" | tr -d '[:space:]')"
        if [[ -n "$candidate" ]]; then
            printf '%s' "$candidate"
            return 0
        fi
    done
    return 1
}
