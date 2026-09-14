#!/usr/bin/env bash
# Extrae y ejecuta la distribución runtime Linux, no solo valida su hash.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
TARBALL=""
REQUIRE_GUI=0
KEEP_TEMP=0

die() { printf 'TARBALL E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }
usage() { printf 'Uso: %s --tarball RUTA [--require-gui] [--keep-temp]\n' "$0"; }

while (($#)); do
    case "$1" in
        --tarball) (($# >= 2)) || die '--tarball necesita una ruta'; TARBALL="$2"; shift ;;
        --require-gui) REQUIRE_GUI=1 ;;
        --keep-temp) KEEP_TEMP=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ -s "$TARBALL" ]] || die "no existe el tarball: $TARBALL"
command -v tar >/dev/null 2>&1 || die 'tar no está disponible'
command -v timeout >/dev/null 2>&1 || die 'timeout no está disponible'

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ltools-tarball-e2e.XXXXXX")"
if (( KEEP_TEMP )); then
    printf 'Temporales conservados en: %s\n' "$TMP_DIR"
else
    trap 'rm -rf -- "$TMP_DIR"' EXIT
fi

archive_root=""
ARCHIVE_LIST="$TMP_DIR/archive.list"
tar -tzf "$TARBALL" >"$ARCHIVE_LIST" || die 'el tarball no se puede listar o está dañado'
while IFS= read -r entry; do
    [[ "$entry" != /* ]] || die "ruta absoluta insegura dentro del tarball: $entry"
    case "/$entry/" in
        */../*) die "ruta con .. insegura dentro del tarball: $entry" ;;
    esac
    if [[ -z "$archive_root" ]]; then archive_root="${entry%%/*}"; fi
    [[ "$entry" == "$archive_root/"* ]] || die 'el tarball contiene más de una raíz'
done <"$ARCHIVE_LIST"
[[ -n "$archive_root" ]] || die 'el tarball no contiene entradas'

tar --no-same-owner -xzf "$TARBALL" -C "$TMP_DIR"
PACKAGE_DIR="$TMP_DIR/$archive_root"
BACKEND="$PACKAGE_DIR/rust/target/release/ltools"
[[ -x "$PACKAGE_DIR/ltools" ]] || die 'falta el lanzador distribuible ltools'
[[ -x "$PACKAGE_DIR/ltools-cli" ]] || die 'falta el lanzador distribuible ltools-cli'
[[ -x "$BACKEND" ]] || die 'falta el backend Rust release dentro del tarball'
[[ -s "$PACKAGE_DIR/ltools-capabilities.json" ]] || die 'falta el descriptor de capacidades'
[[ -s "$PACKAGE_DIR/ltools-terminal.json" ]] || die 'falta el descriptor de integración de terminal'

export HOME="$TMP_DIR/home"
export XDG_CONFIG_HOME="$TMP_DIR/config"
export XDG_DATA_HOME="$TMP_DIR/data"
export XDG_CACHE_HOME="$TMP_DIR/cache"
export XDG_STATE_HOME="$TMP_DIR/state"
export LTOOLS_LANG=es
mkdir -p -- "$HOME" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME" "$XDG_STATE_HOME"

version_output="$("$PACKAGE_DIR/ltools-cli" --version)" || die 'ltools-cli --version falló desde el tarball'
grep -Eq "[0-9]+\.[0-9]+\.[0-9]+" <<<"$version_output" || die 'la versión del tarball no tiene formato reconocible'
help_output="$("$PACKAGE_DIR/ltools-cli" --help)" || die 'ltools-cli --help falló desde el tarball'
grep -Fq 'Uso: ltools' <<<"$help_output" || die 'el perfil CLI del tarball no expuso su ayuda'
"$BACKEND" capabilities --format json >"$TMP_DIR/capabilities.json" ||
    die 'el backend extraído no respondió a la consulta de capacidades'
grep -Fq 'ltools-capabilities-v1' "$TMP_DIR/capabilities.json" ||
    die 'el backend extraído devolvió un contrato de capacidades inesperado'
ok 'tarball extraído: lanzadores, versión, ayuda CLI y backend funcionan'

if command -v jq >/dev/null 2>&1; then
    jq -e '.schema == "ltools-capabilities-v1"' "$PACKAGE_DIR/ltools-capabilities.json" >/dev/null ||
        die 'el descriptor incluido en el tarball es inválido'
    jq -e '.schema == "ltools-terminal-integration-v1"' "$PACKAGE_DIR/ltools-terminal.json" >/dev/null ||
        die 'el descriptor de terminal incluido en el tarball es inválido'
    ok 'descriptores JSON del tarball válidos'
fi

if command -v xvfb-run >/dev/null 2>&1; then
    GUI_LOG="$TMP_DIR/gui.log"
    if ! timeout 35 xvfb-run -a -s "-screen 0 1280x900x24" env GDK_BACKEND=x11 LTOOLS_GUI_REQUIRED=1 \
        LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_SMOKE_HOLD_MS=1200 LTOOLS_DISABLE_GUI=0 \
        LTOOLS_TERMINAL=auto "$BACKEND" >"$GUI_LOG" 2>&1; then
        cat "$GUI_LOG" >&2
        die 'el backend empaquetado no pudo abrir y cerrar la GUI bajo Xvfb'
    fi
    ok 'GUI del backend extraído abre y cierra desde un entorno gráfico virtual'
elif (( REQUIRE_GUI )); then
    die '--require-gui exige xvfb-run'
else
    printf '  SKIP  no hay xvfb-run; se validó CLI y ejecución del tarball\n'
fi

printf 'E2E del tarball completada correctamente: %s\n' "$TARBALL"
