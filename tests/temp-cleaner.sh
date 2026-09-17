#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
source "$ROOT_DIR/scripts/lib/temp-clean.sh"
owned_dir="$(mktemp -d /tmp/ltools-native-help.XXXXXX)"
suffix="${owned_dir##*.}"
link_path="/tmp/ltools-menu-e2e.$suffix"
report_path="/tmp/ltools-registry-dry-run-$((900000000 + $$)).reg"
active_dir="/tmp/ltools-cleaner-active-$$-$(date +%s%N)"
license_dir="$(mktemp -d "/tmp/ltools-third-party-licenses-$$-XXXXXX")"
link_created=0
report_created=0
active_created=0
cleanup_fixture() {
    ((link_created == 0)) || rm -f -- "$link_path"
    ((report_created == 0)) || rm -f -- "$report_path"
    ((active_created == 0)) || rm -rf -- "$active_dir"
    rm -rf -- "$owned_dir"
    rm -rf -- "$license_dir"
}
for fixture in "$link_path" "$report_path" "$active_dir"; do
    if [[ -e "$fixture" || -L "$fixture" ]]; then
        rmdir -- "$owned_dir"
        echo "No se pisa un temporal preexistente de prueba: $fixture" >&2
        exit 1
    fi
done
trap cleanup_fixture EXIT
touch "$report_path"
report_created=1
ln -s "$owned_dir" "$link_path"
link_created=1
mkdir "$active_dir"
active_created=1

ltools_temp_name_is_owned ltools-native-help.Abc123 || { echo 'No reconoció un directorio de prueba permitido.' >&2; exit 1; }
ltools_temp_name_is_owned ltools-smoke.Abc123 || { echo 'No reconoció el temporal de smoke LTools.' >&2; exit 1; }
ltools_temp_name_is_owned ltools-e2e.Abc123 || { echo 'No reconoció el temporal de E2E LTools.' >&2; exit 1; }
legacy_prefix='cachy'
legacy_prefix+='os-smoke'
! ltools_temp_name_is_owned "$legacy_prefix.Abc123" || { echo 'Aceptó un nombre de la identidad antigua.' >&2; exit 1; }
ltools_temp_name_is_owned ltools-storage-map-123-456-2 || { echo 'No reconoció el formato temporal del mapa.' >&2; exit 1; }
ltools_temp_name_is_owned "ltools-third-party-licenses-$$-Abc123" || { echo 'No reconoció el temporal del bundle legal.' >&2; exit 1; }
ltools_temp_name_is_owned "ltools-registry-dry-run-$((900000000 + $$)).reg" || { echo 'No reconoció el informe temporal Wine.' >&2; exit 1; }
ltools_temp_name_is_owned ltools-cleaner-preserved-123-456 || { echo 'No reconoció el temporal de prueba del limpiador.' >&2; exit 1; }
[[ "$(ltools_temp_embedded_pid "ltools-cleaner-preserved-$$-456")" == "$$" ]] || {
    echo 'No reconoció el PID de un temporal activo del limpiador.' >&2
    exit 1
}
[[ "$(ltools_temp_embedded_pid "${license_dir##*/}")" == "$$" ]] || {
    echo 'No reconoció el PID del bundle legal en curso.' >&2
    exit 1
}
! ltools_temp_name_is_owned ltools-not-an-ltools-output.Abc123 || { echo 'Aceptó un nombre ajeno.' >&2; exit 1; }
! ltools_temp_name_is_owned ltools-windows-wine.Abc123-extra || { echo 'Aceptó un formato malformado.' >&2; exit 1; }

ltools_temp_candidate_check "$owned_dir" /tmp || {
    echo 'Rechazó un directorio temporal propio y normal.' >&2
    exit 1
}
ltools_temp_candidate_check "$report_path" /tmp || {
    echo 'Rechazó un archivo temporal propio y normal.' >&2
    exit 1
}
if ltools_temp_candidate_check "$active_dir" /tmp; then
    echo 'Aceptó un temporal cuyo PID sigue activo.' >&2
    exit 1
else
    status=$?
    [[ "$status" -eq 3 ]] || {
        echo "La comprobación del temporal activo devolvió un estado inesperado: $status" >&2
        exit 1
    }
fi
if ltools_temp_candidate_check "$license_dir" /tmp; then
    echo 'Aceptó un bundle de licencias cuyo proceso sigue activo.' >&2
    exit 1
else
    status=$?
    [[ "$status" -eq 3 ]] || {
        echo "La comprobación del bundle legal activo devolvió un estado inesperado: $status" >&2
        exit 1
    }
fi
if ltools_temp_candidate_check "$link_path" /tmp; then
    echo 'Aceptó un enlace simbólico.' >&2
    exit 1
fi
if ltools_temp_candidate_check "$owned_dir" /var/tmp; then
    echo 'Aceptó una entrada fuera de la raíz temporal declarada.' >&2
    exit 1
fi

preview="$(bash "$ROOT_DIR/scripts/build.sh" clean --only build-tmp --dry-run)"
grep -Fq -- "$owned_dir" <<<"$preview" || {
    echo 'La simulación del builder no incluyó el temporal propio permitido.' >&2
    exit 1
}
grep -Fq -- "$report_path" <<<"$preview" || {
    echo 'La simulación del builder no incluyó el informe temporal permitido.' >&2
    exit 1
}
grep -Fq -- "$link_path" <<<"$preview" || {
    echo 'La simulación no informó del enlace simbólico protegido.' >&2
    exit 1
}
grep -Fq -- "$active_dir (parece estar en uso)" <<<"$preview" || {
    echo 'La simulación no protegió el temporal del proceso de prueba todavía activo.' >&2
    exit 1
}
grep -Fq -- "$license_dir (parece estar en uso)" <<<"$preview" || {
    echo 'La simulación no protegió el bundle de licencias en curso.' >&2
    exit 1
}
grep -Fq 'Simulación terminada; no se ha modificado ningún archivo.' <<<"$preview" || {
    echo 'El modo predeterminado del limpiador no confirmó que fuera una simulación.' >&2
    exit 1
}

legacy_preview="$(bash "$ROOT_DIR/scripts/build.sh" clean --only tmp --dry-run)"
grep -Fq -- "$owned_dir" <<<"$legacy_preview" || {
    echo 'El alias compatible --only tmp dejó de reconocer temporales propios.' >&2
    exit 1
}
if bash "$ROOT_DIR/scripts/build.sh" clean --only build-tmp --plans-only --dry-run >/dev/null 2>&1; then
    echo 'Aceptó combinar la limpieza de temporales con la de planes.' >&2
    exit 1
fi
grep -Fq 'formatos allowlist de build/E2E de LTools' <<<"$preview" || {
    echo 'La simulación no identifica claramente que solo contempla temporales de build/E2E LTools.' >&2
    exit 1
}

printf 'OK: allowlist de temporales build/E2E, alias compatible, propiedad, rutas directas y enlaces simbólicos.\n'
