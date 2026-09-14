#!/usr/bin/env bash
# Revisión local de scripts y workflows, usando los mismos analizadores que CI.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
STRICT=0
while (($#)); do
    case "$1" in
        --strict) STRICT=1 ;;
        -h|--help)
            printf 'Uso: %s [--strict]\n' "$0"
            printf 'Ejecuta ShellCheck y actionlint; --strict falla si falta cualquiera.\n'
            exit 0
            ;;
        *) printf 'Opción desconocida: %s\n' "$1" >&2; exit 2 ;;
    esac
    shift
done

mapfile -d '' -t shell_scripts < <(
    find "$ROOT_DIR" -type f -name '*.sh' \
        -not -path "$ROOT_DIR/.git/*" \
        -not -path "$ROOT_DIR/rust/target/*" \
        -not -path "$ROOT_DIR/dist/*" \
        -not -path "$ROOT_DIR/release/*" -print0
)

missing=()
status=0
if command -v shellcheck >/dev/null 2>&1; then
    printf '[REVIEW] ShellCheck: %s scripts\n' "${#shell_scripts[@]}"
    if ((${#shell_scripts[@]})) && ! shellcheck --severity=error "${shell_scripts[@]}"; then
        status=1
    fi
else
    printf '[REVIEW][SKIP] ShellCheck no está instalado.\n'
    missing+=(shellcheck)
fi

if command -v actionlint >/dev/null 2>&1; then
    printf '[REVIEW] actionlint: workflows de GitHub Actions\n'
    if ! (cd -- "$ROOT_DIR" && actionlint -color); then
        status=1
    fi
else
    printf '[REVIEW][SKIP] actionlint no está instalado.\n'
    missing+=(actionlint)
fi

if ((${#missing[@]})); then
    if ((STRICT)); then
        printf '[REVIEW][FAIL] Revisión estricta incompleta; faltan: %s\n' "${missing[*]}" >&2
        status=1
    else
        printf '[REVIEW][INFO] No se consideran superados los analizadores omitidos: %s\n' "${missing[*]}"
    fi
fi

if ((status)); then
    printf '[REVIEW][FAIL] La revisión estática requiere atención.\n' >&2
    exit 1
fi
if ((${#missing[@]} == 0)); then
    printf '[REVIEW][PASS] ShellCheck y actionlint superados.\n'
fi
