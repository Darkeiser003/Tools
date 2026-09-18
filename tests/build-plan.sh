#!/usr/bin/env bash
# Prueba del modo de planificación del builder. No compila ni toca el checkout.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/ltools-build-plan.XXXXXX")"
cleanup() { rm -rf -- "$temporary_root"; }
trap cleanup EXIT

output_path="$temporary_root/dist"
release_path="$temporary_root/release"
plan="$(bash "$ROOT_DIR/scripts/build.sh" --plan --component frontend --no-log \
    --output "$output_path" --release-dir "$release_path")"
grep -Fq 'plan de build (sin ejecución)' <<<"$plan" || { printf '%s\n' 'BUILD PLAN ERROR: falta la cabecera.' >&2; exit 1; }
grep -Fq 'no existe un frontend separado' <<<"$plan" || { printf '%s\n' 'BUILD PLAN ERROR: frontend no está explicado como alias Rust.' >&2; exit 1; }
grep -Fq 'No se ha compilado, empaquetado, firmado ni borrado nada.' <<<"$plan" || {
    printf '%s\n' 'BUILD PLAN ERROR: el plan no declara que es de solo lectura.' >&2
    exit 1
}
[[ ! -e "$output_path" && ! -e "$release_path" ]] || {
    printf '%s\n' 'BUILD PLAN ERROR: el plan creó una salida.' >&2
    exit 1
}

if command -v pwsh >/dev/null 2>&1; then
    windows_plan="$(pwsh -NoProfile -NonInteractive -File "$ROOT_DIR/scripts/build.ps1" \
        -Plan -Component frontend -NoLog -Output "$output_path" -ReleaseOutput "$release_path")"
    grep -Fq 'plan de build (sin ejecución)' <<<"$windows_plan" || {
        printf '%s\n' 'BUILD PLAN ERROR: falta la cabecera Windows.' >&2
        exit 1
    }
    grep -Fq 'no existe un frontend separado' <<<"$windows_plan" || {
        printf '%s\n' 'BUILD PLAN ERROR: frontend Windows no está explicado como alias Rust.' >&2
        exit 1
    }
    [[ ! -e "$output_path" && ! -e "$release_path" ]] || {
        printf '%s\n' 'BUILD PLAN ERROR: el plan Windows creó una salida.' >&2
        exit 1
    }
else
    printf '%s\n' 'SKIP: pwsh no está instalado; solo se valida el plan Linux.'
fi

printf '%s\n' 'Plan de build Linux/Windows y ausencia de efectos laterales validados.'
