#!/usr/bin/env bash
# E2E de menús y funciones: usa un HOME sintético y no modifica el equipo.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
KEEP_TEMP=0

die() { printf 'MENU E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        --keep-temp) KEEP_TEMP=1 ;;
        -h|--help) printf 'Uso: %s [--binary RUTA] [--keep-temp]\n' "$0"; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ -x "$BIN" ]] || die "no existe el binario ejecutable: $BIN"
[[ -x "$ROOT_DIR/ltools.sh" ]] || die "no existe ltools.sh ejecutable"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ltools-menu-e2e.XXXXXX")"
if (( KEEP_TEMP )); then
    printf 'Temporales conservados en: %s\n' "$TMP_DIR"
else
    trap 'rm -rf -- "$TMP_DIR"' EXIT
fi

export HOME="$TMP_DIR/home"
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_STATE_HOME="$HOME/.local/state"
export LTOOLS_NO_MOUNTS=1
export LTOOLS_NO_AUTO_TERMINAL=1
mkdir -p "$HOME" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_STATE_HOME"

PREFIX="$HOME/Games/ExamplePrefix"
FIXTURE="$TMP_DIR/fixture"
AUDIT_OUT="$TMP_DIR/audit"
GAMES_OUT="$TMP_DIR/games"
PKG_OUT="$TMP_DIR/packages"
mkdir -p "$PREFIX/drive_c/Program Files/Example" "$PREFIX/dosdevices" \
    "$HOME/.config/heroic" \
    "$FIXTURE" "$HOME/.local/share/lutris/games" \
    "$HOME/.local/share/Steam/steamapps/common/ExampleGame" \
    "$HOME/.local/share/Steam/steamapps/compatdata/123456/pfx"
printf 'system-registry\n' > "$PREFIX/system.reg"
printf 'user-registry\n' > "$PREFIX/user.reg"
printf 'example\n' > "$PREFIX/drive_c/Program Files/Example/example.exe"
printf 'archive\n' > "$FIXTURE/example.AppImage"
printf 'package\n' > "$FIXTURE/example.deb"
printf 'same-content\n' > "$FIXTURE/duplicate-a.bin"
cp -a -- "$FIXTURE/duplicate-a.bin" "$FIXTURE/duplicate-b.bin"
printf 'game_path: %s\n' "$HOME/Games" > "$HOME/.local/share/lutris/system.yml"
printf 'name: Example\nprefix: %s\nexe: %s\n' "$PREFIX" "$PREFIX/drive_c/Program Files/Example/example.exe" \
    > "$HOME/.local/share/lutris/games/example.yml"
cat > "$HOME/.config/heroic/config.json" <<EOF
{"defaultSettings":{"defaultWinePrefix":"$HOME/Games","winePrefix":"$PREFIX","wineVersion":{"bin":"/usr/bin/wine"}}}
EOF
mkdir -p "$HOME/.local/share/umu/compatibilitytools/UMU-Latest"
printf '"manifest" "synthetic"\n' > "$HOME/.local/share/umu/compatibilitytools/UMU-Latest/toolmanifest.vdf"
cat > "$HOME/.local/share/Steam/steamapps/libraryfolders.vdf" <<EOF
"libraryfolders"
{
  "0" { "path" "$HOME/.local/share/Steam" }
}
EOF
cat > "$HOME/.local/share/Steam/steamapps/appmanifest_123456.acf" <<'EOF'
"AppState"
{
  "appid" "123456"
  "name" "Example Game"
  "installdir" "ExampleGame"
}
EOF

run_bash() {
    env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_DATA_HOME="$XDG_DATA_HOME" \
        XDG_STATE_HOME="$XDG_STATE_HOME" LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 \
        "$ROOT_DIR/ltools.sh" "$@"
}

run_menu() {
    local name input expected output status
    name="$1"
    input="$2"
    expected="$3"
    output="$TMP_DIR/$name.out"
    shift 3
    set +e
    printf '%b' "$input" | timeout 60 env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" \
        XDG_DATA_HOME="$XDG_DATA_HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
        LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 "$ROOT_DIR/ltools.sh" "$@" \
        >"$output" 2>&1
    status=$?
    set -e
    [[ "$status" -eq 0 ]] || { sed -n '1,120p' "$output" >&2; die "$name terminó con código $status"; }
    grep -Fq -- "$expected" "$output" || { sed -n '1,120p' "$output" >&2; die "$name no mostró: $expected"; }
    ok "$name"
}

assert_file() { [[ -f "$1" ]] || die "falta el fichero esperado: $1"; }

printf 'E2E: ejecutando entradas y submenús de la fachada Bash...\n'
run_menu main-doctor $'8\nq\n' 'Dependencias del sistema'
run_menu main-defaults $'5\n\nq\n' 'Rutas y defaults detectados'
run_menu main-prefix-list $'4\n4\nq\nq\n' 'ExamplePrefix'
run_menu main-prefix-inspect "4\n5\nm\n$PREFIX\nq\nq\n" 'Ejecutables:'
run_menu main-system $'7\nq\nq\n' 'Administración del sistema'
run_menu main-clean $'3\n0\nq\n' 'disk-clean: menú principal'
run_menu main-help $'h\nq\n' 'Uso:'

printf 'E2E: ejecutando los módulos reales con fixtures aislados...\n'
run_bash audit --no-mounts --root "$FIXTURE" --out "$AUDIT_OUT" \
    --duplicates --min-size-mb 0 >/dev/null
assert_file "$AUDIT_OUT/appimages.tsv"
assert_file "$AUDIT_OUT/package-artifacts.tsv"
assert_file "$AUDIT_OUT/duplicates.tsv"
grep -Fq 'example.AppImage' "$AUDIT_OUT/appimages.tsv" || die 'audit no detectó AppImage'
grep -Fq 'example.deb' "$AUDIT_OUT/package-artifacts.tsv" || die 'audit no detectó .deb'
ok 'auditoría general, AppImage, .deb y duplicados'

run_bash games --no-mounts --root "$HOME" --out "$GAMES_OUT" >/dev/null
assert_file "$GAMES_OUT/wine-prefixes.tsv"
assert_file "$GAMES_OUT/configuration-validation.tsv"
assert_file "$GAMES_OUT/configuration-databases.tsv"
grep -Fq 'ExamplePrefix' "$GAMES_OUT/wine-prefixes.tsv" || die 'games no detectó el prefijo'
grep -Fq 'Heroic' "$GAMES_OUT/configuration-validation.tsv" || die 'games no validó Heroic'
grep -Fq 'Lutris' "$GAMES_OUT/configuration-validation.tsv" || die 'games no validó Lutris'
ok 'auditoría de juegos, prefijo, Heroic, Lutris, UMU y Steam'

run_bash packages --no-mounts --no-home --root "$FIXTURE" --out "$PKG_OUT" >/dev/null
assert_file "$PKG_OUT/package-managers.tsv"
assert_file "$PKG_OUT/package-artifacts.tsv"
ok 'inventario de gestores y formatos de paquetes'

CLEAN_PATH="$HOME/cache-candidate"
mkdir -p "$CLEAN_PATH"
printf 'keep-me\n' > "$CLEAN_PATH/file.txt"
CLEAN_PLAN="$TMP_DIR/clean-plan.tsv"
run_bash clean --dry-run --path "$CLEAN_PATH" --plan "$CLEAN_PLAN" >/dev/null
[[ -d "$CLEAN_PATH" ]] || die 'clean --dry-run modificó una ruta'
grep -Fq $'trash-move\t' "$CLEAN_PLAN" || die 'clean --dry-run no registró el plan'
ok 'limpieza protegida en dry-run sin mutar datos'

SYSTEM_PLAN="$TMP_DIR/system-plan.tsv"
run_bash system --dry-run --plan "$SYSTEM_PLAN" status >/dev/null
assert_file "$SYSTEM_PLAN"
ok 'consulta real de systemd con plan'

DEFAULTS_OUTPUT="$TMP_DIR/defaults.out"
run_bash defaults >"$DEFAULTS_OUTPUT"
grep -Fq 'Wine:' "$DEFAULTS_OUTPUT" || die 'defaults no informó Wine'
grep -Fq 'Proton:' "$DEFAULTS_OUTPUT" || die 'defaults no informó Proton'
grep -Fq 'Heroic' "$DEFAULTS_OUTPUT" || die 'defaults no informó Heroic'
ok 'rutas efectivas de Wine, winetricks, Proton, Steam y Heroic'

printf 'E2E: comprobando también menú Rust sin ejecutar acciones destructivas...\n'
RUST_MENU_OUTPUT="$TMP_DIR/rust-menu.out"
printf 'q\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_MENU_OUTPUT" 2>&1
grep -Fq 'LTools' "$RUST_MENU_OUTPUT" || die 'el menú Rust no arrancó'
grep -Fq 'Elige una opción' "$RUST_MENU_OUTPUT" || die 'el menú Rust no mostró el prompt'
ok 'menú Rust responde y sale limpiamente'

printf 'E2E de menús y funciones completado correctamente.\n'
