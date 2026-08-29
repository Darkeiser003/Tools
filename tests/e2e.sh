#!/usr/bin/env bash
# E2E aislado: migra y revierte un prefijo sintético sin tocar datos del usuario.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
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

printf 'E2E: probando rollback...\n'
printf 'y\n' | run_tool rollback --plan "$PLAN" >/dev/null
[[ ! -e "$DEST" ]] || die 'rollback no retiró el destino creado'
[[ -d "$SOURCE" ]] || die 'rollback alteró el origen'
ok 'rollback retiró solo el destino y conservó el origen'

printf 'E2E completado correctamente.\n'
