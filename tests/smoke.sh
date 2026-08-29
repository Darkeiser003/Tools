#!/usr/bin/env bash
# Smoke tests seguros: no recorren los discos reales ni modifican la cuenta.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
APPIMAGE_PATH=""
RUNNER_PATH=""
LOG_PATH=""
KEEP_TEMP=0

die() { printf 'SMOKE ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }
skip() { printf '  SKIP  %s\n' "$1"; }
usage() { printf 'Uso: %s [--binary RUTA] [--appimage RUTA] [--runner RUTA] [--log RUTA] [--keep-temp]\n' "$0"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        --appimage) (($# >= 2)) || die '--appimage necesita una ruta'; APPIMAGE_PATH="$2"; shift ;;
        --runner) (($# >= 2)) || die '--runner necesita una ruta'; RUNNER_PATH="$2"; shift ;;
        --log) (($# >= 2)) || die '--log necesita una ruta'; LOG_PATH="$2"; shift ;;
        --keep-temp) KEEP_TEMP=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ -x "$BIN" ]] || die "no existe el binario ejecutable: $BIN"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cachyos-smoke.XXXXXX")"
if [[ "$KEEP_TEMP" -eq 0 ]]; then
    trap 'rm -rf -- "$TMP_DIR"' EXIT
else
    printf 'Temporales conservados en: %s\n' "$TMP_DIR"
fi

printf 'Smoke tests de LTools\n'
while IFS= read -r -d '' file; do
    bash -n "$file"
done < <(find "$ROOT_DIR" -maxdepth 3 -type f -name '*.sh' -print0)
ok 'sintaxis de todos los scripts Bash'

"$BIN" --version >/dev/null
ok 'backend Rust responde a --version'
"$BIN" --help >/dev/null
ok 'backend Rust responde a --help'
EN_HELP="$(LTOOLS_LANG=en "$BIN" --help)"
grep -Fq 'Usage: ltools' <<<"$EN_HELP" || die 'el idioma inglés no se aplicó al backend Rust'
DE_HELP="$("$BIN" --lang de --help)"
grep -Fq 'Verwendung:' <<<"$DE_HELP" || die 'la opción --lang no se aplicó al backend Rust'
MENU_OUTPUT="$(printf 'q\n' | LTOOLS_LANG=fr "$ROOT_DIR/ltools.sh")"
grep -Fq 'Choisissez une option' <<<"$MENU_OUTPUT" || die 'el catálogo francés no se aplicó al menú Bash'
ok 'idiomas en, de y fr en backend y fachada Bash'
DOCTOR_OUTPUT="$(HOME="$TMP_DIR/home" XDG_STATE_HOME="$TMP_DIR/state" "$BIN" doctor)"
grep -Fq 'LTools host diagnostics' <<<"$DOCTOR_OUTPUT" || die 'doctor no funciona como operación de solo lectura'
ok 'doctor Rust sin crear planes ni modificar el estado'

mkdir -p "$TMP_DIR/root/demo-prefix/drive_c"
printf 'synthetic-prefix\n' > "$TMP_DIR/root/demo-prefix/system.reg"
printf 'demo\n' > "$TMP_DIR/root/demo-prefix/drive_c/demo.exe"
PLAN="$TMP_DIR/list-plan.tsv"
LIST_OUTPUT="$("$BIN" --dry-run --plan "$PLAN" prefix list --root "$TMP_DIR/root")"
[[ -s "$PLAN" ]] || die 'no se creó el plan del listado'
grep -Fq 'demo-prefix' <<<"$LIST_OUTPUT" || die 'el listado no detectó el prefijo sintético'
ok 'listado aislado de un prefijo sintético'

if [[ -n "$APPIMAGE_PATH" ]]; then
    [[ -x "$APPIMAGE_PATH" ]] || die "AppImage no ejecutable: $APPIMAGE_PATH"
    DIRECT_LOG="${LOG_PATH:-$TMP_DIR/appimage-direct.log}"
    mkdir -p "$(dirname -- "$DIRECT_LOG")"
    : > "$DIRECT_LOG"
    {
        printf 'LTools: prueba directa del AppImage\n'
        printf 'Fecha: %s\n' "$(date --iso-8601=seconds)"
        printf 'AppImage: %s\n' "$APPIMAGE_PATH"
        printf 'Permisos: %s\n\n' "$(stat -c '%A %a %U:%G' "$APPIMAGE_PATH" 2>/dev/null || printf 'desconocidos')"
        printf '$ %q --doctor\n' "$APPIMAGE_PATH"
    } >> "$DIRECT_LOG"
    set +e
    timeout 30 "$APPIMAGE_PATH" --doctor >> "$DIRECT_LOG" 2>&1
    DIRECT_STATUS=$?
    set -e
    if [[ "$DIRECT_STATUS" -ne 0 ]]; then
        printf 'Salida de la ejecución directa:\n' >&2
        sed -n '1,160p' "$DIRECT_LOG" >&2
        die "el AppImage no se pudo abrir directamente (código $DIRECT_STATUS); log: $DIRECT_LOG"
    fi
    ok "ejecución directa del AppImage; log: $DIRECT_LOG"
    NOARGS_OUTPUT="$(printf 'q\n' | timeout 30 env LTOOLS_NO_AUTO_TERMINAL=1 "$APPIMAGE_PATH" 2>&1)"
    grep -Fq 'Elige una opción' <<<"$NOARGS_OUTPUT" || die 'el menú interactivo no se mostró al iniciar sin argumentos'
    ok 'menú interactivo del AppImage'
    APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --version >/dev/null
    ok 'AppImage responde usando extracción temporal'
    APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --doctor >/dev/null
    ok 'diagnóstico del AppImage'
    if [[ -c /dev/fuse ]] &&
        { command -v fusermount3 >/dev/null 2>&1 || command -v fusermount >/dev/null 2>&1; }; then
        "$APPIMAGE_PATH" --version >/dev/null
        ok 'AppImage responde con montaje FUSE normal'
    else
        skip 'montaje FUSE normal: no disponible; se validó el fallback por extracción'
    fi
    set +e
    FUSE_OUTPUT="$(APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --fuse-check 2>&1)"
    FUSE_STATUS=$?
    set -e
    [[ "$FUSE_STATUS" -eq 0 || "$FUSE_STATUS" -eq 1 ]] || die '--fuse-check terminó con un código inesperado'
    grep -Fq 'FUSE' <<<"$FUSE_OUTPUT" || die '--fuse-check no mostró diagnóstico FUSE'
    ok 'diagnóstico FUSE'
    if [[ -n "$RUNNER_PATH" ]]; then
        [[ -x "$RUNNER_PATH" ]] || die "lanzador no ejecutable: $RUNNER_PATH"
        LTOOLS_FORCE_EXTRACT=1 "$RUNNER_PATH" --version >/dev/null
        ok 'lanzador externo con fallback forzado sin FUSE'
    fi
else
    skip 'AppImage: no se proporcionó --appimage'
fi

printf 'Smoke tests completados correctamente.\n'
