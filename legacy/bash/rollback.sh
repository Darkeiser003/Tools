#!/usr/bin/env bash

# Universal entry point for reversible operations recorded by LTools.
set -uo pipefail
VERSION="0.3.0"
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd -P)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib/ltools-plan.sh"

usage() {
    cat <<EOF
Uso: $(basename "$0") --plan FICHERO

Revierte operaciones reversibles registradas por --plan o --dry-run.
Las desinstalaciones de paquetes, cambios de servicios y operaciones que no
tengan una copia de seguridad se muestran como no reversibles.
EOF
}

main() {
    local plan=""
    while (($#)); do
        case "$1" in
            --version) printf '%s %s\n' "$(basename "$0")" "$VERSION"; return 0 ;;
            --plan) [[ $# -ge 2 ]] || { printf 'Falta el fichero del plan.\n' >&2; exit 2; }; plan="$2"; shift 2 ;;
            -h|--help) usage; return 0 ;;
            *) printf 'Opción desconocida: %s\n' "$1" >&2; return 2 ;;
        esac
    done
    [[ -n "$plan" ]] || { usage; return 2; }
    rollback_plan "$plan"
}

main "$@"
