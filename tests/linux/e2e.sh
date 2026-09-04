#!/usr/bin/env bash
# E2E aislado: migra y revierte un prefijo sintético sin tocar datos del usuario.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
APPIMAGE_PATH=""
KEEP_TEMP=0

die() { printf 'E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        --appimage) (($# >= 2)) || die '--appimage necesita una ruta'; APPIMAGE_PATH="$2"; shift ;;
        --keep-temp) KEEP_TEMP=1 ;;
        -h|--help) printf 'Uso: %s [--binary RUTA] [--appimage RUTA] [--keep-temp]\n' "$0"; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ -x "$BIN" ]] || die "no existe el binario ejecutable: $BIN"
command -v rsync >/dev/null 2>&1 || { printf 'E2E SKIP: rsync no está disponible.\n'; exit 0; }
if ! command -v gio >/dev/null 2>&1 && ! command -v trash-put >/dev/null 2>&1; then
    printf 'E2E SKIP: no hay gio ni trash-put para probar rollback.\n'
    exit 0
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cachyos-e2e.XXXXXX")"
if [[ "$KEEP_TEMP" -eq 0 ]]; then
    trap 'rm -rf -- "$TMP_DIR"' EXIT
else
    printf 'Temporales conservados en: %s\n' "$TMP_DIR"
fi

export HOME="$TMP_DIR/home"
export XDG_DATA_HOME="$TMP_DIR/home/.local/share"
export XDG_CONFIG_HOME="$TMP_DIR/home/.config"
export XDG_STATE_HOME="$TMP_DIR/home/.local/state"
mkdir -p "$HOME" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_STATE_HOME"

SOURCE="$TMP_DIR/source-prefix"
DEST="$TMP_DIR/destination-prefix"
PLAN="$TMP_DIR/migrate-plan.tsv"
mkdir -p "$SOURCE/drive_c/bin" "$SOURCE/dosdevices"
printf 'system-registry\n' > "$SOURCE/system.reg"
printf 'user-registry\n' > "$SOURCE/user.reg"
printf 'synthetic-game\n' > "$SOURCE/drive_c/bin/game.exe"
ln -s "$SOURCE/drive_c" "$SOURCE/dosdevices/c:" 2>/dev/null || true

run_tool() {
    if [[ -n "$APPIMAGE_PATH" ]]; then
        APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" "$@"
    else
        "$BIN" "$@"
    fi
}

printf 'E2E: migrando prefijo sintético...\n'
printf 'y\n' | run_tool --plan "$PLAN" prefix migrate \
    --source "$SOURCE" --dest "$DEST" --force
[[ -f "$DEST/system.reg" ]] || die 'no se copió system.reg'
[[ -f "$DEST/drive_c/bin/game.exe" ]] || die 'no se copió el contenido de drive_c'
cmp -s "$SOURCE/system.reg" "$DEST/system.reg" || die 'system.reg difiere tras la migración'
cmp -s "$SOURCE/drive_c/bin/game.exe" "$DEST/drive_c/bin/game.exe" || die 'ejecutable difiere tras la migración'
[[ -f "$PLAN" ]] || die 'no se creó el plan de operación'
ok 'migración verificada con comparación de contenido'

printf 'E2E: comprobando bloqueos de seguridad...\n'
NONEMPTY="$TMP_DIR/nonempty-destination"
mkdir -p "$NONEMPTY"
printf 'must-stay\n' > "$NONEMPTY/marker.txt"
set +e
printf 'y\n' | run_tool prefix migrate --source "$SOURCE" --dest "$NONEMPTY" >"$TMP_DIR/nonempty.out" 2>&1
NONEMPTY_STATUS=$?
set -e
[[ "$NONEMPTY_STATUS" -ne 0 ]] || die 'se permitió usar un destino no vacío'
[[ -f "$NONEMPTY/marker.txt" ]] || die 'el bloqueo del destino no vacío alteró datos'

set +e
run_tool prefix migrate --source "$SOURCE" --dest "$SOURCE/inside" >"$TMP_DIR/inside.out" 2>&1
INSIDE_STATUS=$?
set -e
[[ "$INSIDE_STATUS" -ne 0 ]] || die 'se permitió un destino dentro del origen'

set +e
run_tool prefix migrate --source "$SOURCE" --dest "$TMP_DIR/traversal" --include '../outside' >"$TMP_DIR/traversal.out" 2>&1
TRAVERSAL_STATUS=$?
set -e
[[ "$TRAVERSAL_STATUS" -ne 0 ]] || die 'se permitió una selección fuera del primer nivel'

DRY_DEST="$TMP_DIR/dry-run-destination"
printf 'y\n' | run_tool --dry-run prefix migrate --source "$SOURCE" --dest "$DRY_DEST" >"$TMP_DIR/dry-run.out"
[[ ! -e "$DRY_DEST" ]] || die 'dry-run creó el destino'
ok 'bloqueos de seguridad y dry-run sin mutaciones'

printf 'E2E: probando rollback...\n'
printf 'E2E: comprobando rollback en dry-run...\n'
run_tool --dry-run rollback --plan "$PLAN" >"$TMP_DIR/rollback-dry-run.out"
grep -Fq 'Rollback simulado' "$TMP_DIR/rollback-dry-run.out" || die 'rollback --dry-run no informó simulación'
[[ -e "$DEST" ]] || die 'rollback --dry-run modificó el destino'
[[ -d "$SOURCE" ]] || die 'rollback --dry-run alteró el origen'
ok 'rollback dry-run sin mutaciones'

printf 'E2E: rechazando planes con formato inválido...\n'
BAD_PLAN="$TMP_DIR/bad-plan.tsv"
printf 'esto no es un plan de LTools\n' > "$BAD_PLAN"
set +e
run_tool --dry-run rollback --plan "$BAD_PLAN" >"$TMP_DIR/bad-plan.out" 2>&1
BAD_PLAN_STATUS=$?
set -e
[[ "$BAD_PLAN_STATUS" -ne 0 ]] || die 'rollback aceptó un plan con formato inválido'
grep -Fq 'formato de plan no reconocido' "$TMP_DIR/bad-plan.out" || die 'rollback no explicó el formato inválido'
ok 'rollback rechaza planes manipulados o incompletos'

printf 'y\n' | run_tool rollback --plan "$PLAN" >/dev/null
[[ ! -e "$DEST" ]] || die 'rollback no retiró el destino creado'
[[ -d "$SOURCE" ]] || die 'rollback alteró el origen'
ok 'rollback retiró solo el destino y conservó el origen'

printf 'E2E completado correctamente.\n'
