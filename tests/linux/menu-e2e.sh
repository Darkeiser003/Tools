#!/usr/bin/env bash
# E2E de menús y funciones: usa un HOME sintético y no modifica el equipo.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/rust/Cargo.toml" | head -n1)"
APPIMAGE_PATH=""
KEEP_TEMP=0
REQUIRE_GUI=0

die() { printf 'MENU E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        --appimage) (($# >= 2)) || die '--appimage necesita una ruta'; APPIMAGE_PATH="$2"; shift ;;
        --keep-temp) KEEP_TEMP=1 ;;
        --require-gui) REQUIRE_GUI=1 ;;
        -h|--help) printf 'Uso: %s [--binary RUTA] [--appimage RUTA] [--require-gui] [--keep-temp]\n' "$0"; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

if (( REQUIRE_GUI )); then
    command -v timeout >/dev/null 2>&1 || die '--require-gui exige timeout'
    command -v xvfb-run >/dev/null 2>&1 || die '--require-gui exige xvfb-run'
    command -v xdpyinfo >/dev/null 2>&1 || die '--require-gui exige xdpyinfo (x11-utils)'
    timeout 10 xvfb-run -a xdpyinfo >/dev/null 2>&1 || die '--require-gui no pudo iniciar un display Xvfb'
fi

[[ -x "$BIN" ]] || die "no existe el binario ejecutable: $BIN"
[[ -x "$ROOT_DIR/ltools.sh" ]] || die "no existe ltools.sh ejecutable"
BIN="$(realpath -- "$BIN")"
if [[ -n "$APPIMAGE_PATH" ]]; then
    [[ -x "$APPIMAGE_PATH" ]] || die "AppImage no ejecutable: $APPIMAGE_PATH"
    APPIMAGE_PATH="$(realpath -- "$APPIMAGE_PATH")"
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ltools-menu-e2e.XXXXXX")"
if (( KEEP_TEMP )); then
    printf 'Temporales conservados en: %s\n' "$TMP_DIR"
else
    trap 'rm -rf -- "$TMP_DIR"' EXIT
fi

export TMPDIR="$TMP_DIR"
export HOME="$TMP_DIR/home"
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_STATE_HOME="$HOME/.local/state"
export LTOOLS_LANG=es
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

run_rust() {
    if [[ -n "$APPIMAGE_PATH" ]]; then
        env APPIMAGE_EXTRACT_AND_RUN=1 HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" \
            XDG_DATA_HOME="$XDG_DATA_HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
            LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 "$APPIMAGE_PATH" --rust "$@"
    else
        env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_DATA_HOME="$XDG_DATA_HOME" \
            XDG_STATE_HOME="$XDG_STATE_HOME" LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 \
            "$BIN" "$@"
    fi
}

run_menu() {
    local name input expected output status
    local -a command=()
    name="$1"
    input="$2"
    expected="$3"
    output="$TMP_DIR/$name.out"
    if [[ -n "$APPIMAGE_PATH" ]]; then
        command=(env APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" menu)
    else
        command=("$BIN" menu)
    fi
    shift 3
    set +e
    (
        cd "$TMP_DIR"
        printf '%b' "$input" | timeout 60 env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" \
            XDG_DATA_HOME="$XDG_DATA_HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
            LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 "${command[@]}" "$@"
    ) >"$output" 2>&1
    status=$?
    set -e
    [[ "$status" -eq 0 ]] || { sed -n '1,120p' "$output" >&2; die "$name terminó con código $status"; }
    grep -Fq -- "$expected" "$output" || { sed -n '1,120p' "$output" >&2; die "$name no mostró: $expected"; }
    ok "$name"
}

run_menu_expected_status() {
    local name input expected expected_status output status
    local -a command=()
    name="$1"
    input="$2"
    expected="$3"
    expected_status="$4"
    output="$TMP_DIR/$name.out"
    if [[ -n "$APPIMAGE_PATH" ]]; then
        command=(env APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" menu)
    else
        command=("$BIN" menu)
    fi
    set +e
    (
        cd "$TMP_DIR"
        printf '%b' "$input" | timeout 60 env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" \
            XDG_DATA_HOME="$XDG_DATA_HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
            LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 "${command[@]}"
    ) >"$output" 2>&1
    status=$?
    set -e
    [[ "$status" -eq "$expected_status" ]] || { sed -n '1,120p' "$output" >&2; die "$name terminó con código $status; se esperaba $expected_status"; }
    grep -Fq -- "$expected" "$output" || { sed -n '1,120p' "$output" >&2; die "$name no mostró: $expected"; }
    ok "$name"
}

assert_file() { [[ -f "$1" ]] || die "falta el fichero esperado: $1"; }

printf 'E2E: recorriendo el índice completo de guías CLI y GUI...\n'
GUIDE_TOPICS=(
    audit packages software installable git gh aliases automation automation-register clean
    storage storage-partitions storage-filesystems storage-volumes system
    services accounts native network connectivity boot registry diagnostics
    wine defaults settings containers containers-lifecycle containers-images
    containers-volumes containers-compose kubernetes ssh adb utilities actions
    privileges
)
for guide_topic in "${GUIDE_TOPICS[@]}"; do
    for guide_mode in cli gui; do
        guide_output="$TMP_DIR/guide-${guide_mode}-${guide_topic}.out"
        if [[ -n "$APPIMAGE_PATH" ]]; then
            if timeout 60 env APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" guide "$guide_mode" "$guide_topic" >"$guide_output" 2>&1; then
                guide_status=0
            else
                guide_status=$?
            fi
        else
            if timeout 60 "$BIN" guide "$guide_mode" "$guide_topic" >"$guide_output" 2>&1; then
                guide_status=0
            else
                guide_status=$?
            fi
        fi
        if [[ "$guide_status" -ne 0 ]]; then
            sed -n '1,100p' "$guide_output" >&2
            die "la guía $guide_mode/$guide_topic terminó con código $guide_status"
        fi
        grep -Eq 'GUÍA (CLI|GRÁFICA)' "$guide_output" ||
            die "la guía $guide_mode/$guide_topic no tiene cabecera"
        if [[ "$guide_mode" == gui ]]; then
            # Las guías de familias sin pantalla propia (por ejemplo
            # privilegios o acciones) son referencias transversales; las que
            # sí se abren desde un menú deben contener su índice contextual.
            if ! grep -Eq 'Esta categoría no tiene un catálogo contextual registrado' "$guide_output"; then
                grep -Fq 'Volver' "$guide_output" ||
                    die "la guía GUI/$guide_topic no documenta la navegación de vuelta"
                grep -Eq 'Menú completo|PANTALLA PRINCIPAL|MENÚ COMPLETO' "$guide_output" ||
                    die "la guía GUI/$guide_topic no contiene su índice de menú"
            fi
        else
            grep -Eq 'Opciones de consulta:|GUÍA CLI' "$guide_output" ||
                die "la guía CLI/$guide_topic no contiene sus opciones nativas"
        fi
    done
done
ALL_GUIDES_OUTPUT="$TMP_DIR/guide-all.out"
if [[ -n "$APPIMAGE_PATH" ]]; then
    if timeout 60 env APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" guide gui all >"$ALL_GUIDES_OUTPUT" 2>&1; then
        guide_status=0
    else
        guide_status=$?
    fi
else
    if timeout 60 "$BIN" guide gui all >"$ALL_GUIDES_OUTPUT" 2>&1; then
        guide_status=0
    else
        guide_status=$?
    fi
fi
if [[ "$guide_status" -ne 0 ]]; then
    sed -n '1,100p' "$ALL_GUIDES_OUTPUT" >&2
    die "el índice de guías GUI terminó con código $guide_status"
fi
for index_marker in 'Panel principal:' 'Cada categoría abre un menú propio' 'Las guías contextuales' \
    'ÍNDICE DETALLADO DE MENÚS Y SUBMENÚS' 'Vaciar caché DNS' 'Entradas EFI / NVRAM' \
    'Gestionar servicio' 'Analizadores de código y CI' 'Crear tabla GPT' \
    'Restaurar cabecera LUKS' 'Aplicar manifiesto'; do
    grep -Fq "$index_marker" "$ALL_GUIDES_OUTPUT" || die "el índice GUI no contiene: $index_marker"
done
ok 'todas las guías CLI/GUI incluyen índices detallados, opciones y navegación documentada'

CLI_ALL_OUTPUT="$TMP_DIR/guide-cli-all.out"
if [[ -n "$APPIMAGE_PATH" ]]; then
    if timeout 60 env APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" guide cli all >"$CLI_ALL_OUTPUT" 2>&1; then
        guide_status=0
    else
        guide_status=$?
    fi
else
    if timeout 60 "$BIN" guide cli all >"$CLI_ALL_OUTPUT" 2>&1; then
        guide_status=0
    else
        guide_status=$?
    fi
fi
if [[ "$guide_status" -ne 0 ]]; then
    sed -n '1,100p' "$CLI_ALL_OUTPUT" >&2
    die "el índice detallado de guías CLI terminó con código $guide_status"
fi
for cli_marker in 'ÍNDICE DETALLADO DE FAMILIAS CLI' 'GUÍA CLI: GIT Y GITHUB' \
    'GUÍA CLI: ALMACENAMIENTO' 'GUÍA CLI: SERVICIOS' '--include-personal' \
    '--scope system|user|both'; do
    grep -Fq -- "$cli_marker" "$CLI_ALL_OUTPUT" || die "el índice CLI no contiene: $cli_marker"
done
ok 'guide cli all incluye opciones, argumentos y gestión de cada familia'

printf 'E2E: comprobando la fachada, que siempre usa el backend Rust...\n'
run_menu main-facade-quit $'q\n' 'LTools'

printf 'E2E: ejecutando funciones Rust con fixtures aislados...\n'
run_menu main-audit-default "1\n1\n\n\n\n\n" 'Auditoría general'
grep -Fq 'Informe:' "$TMP_DIR/main-audit-default.out" || die 'la auditoría por defecto no generó informe'
ok 'auditoría con todos los valores predeterminados'

run_menu main-audit "1\n1\nn\ny\n$FIXTURE\n$AUDIT_OUT\nq\n" 'Auditoría general'
assert_file "$AUDIT_OUT/appimages.tsv"
assert_file "$AUDIT_OUT/installers-and-archives.tsv"
assert_file "$AUDIT_OUT/duplicates.tsv"
grep -Fq 'example.AppImage' "$AUDIT_OUT/appimages.tsv" || die 'audit no detectó AppImage'
grep -Fq 'example.deb' "$AUDIT_OUT/installers-and-archives.tsv" || die 'audit no detectó .deb'
ok 'auditoría Rust, AppImage, .deb y duplicados'

run_menu main-audit-full "1\n1\ny\nn\n$FIXTURE\n$TMP_DIR/audit-full\n" 'Auditoría general'
grep -Fq 'Informe:' "$TMP_DIR/main-audit-full.out" || die 'la auditoría completa no generó informe'
ok 'auditoría con escaneo completo y opciones modificadas'

run_menu main-games-default "1\n2\n\n\n\n" 'Auditoría de juegos, Wine y Proton'
DEFAULT_GAMES_REPORT="$(sed -n 's/^Informe: //p' "$TMP_DIR/main-games-default.out" | tail -1)"
DEFAULT_GAMES_VALIDATION="$(sed -n 's/^Validación de Heroic\/Lutris\/UMU\/Steam: //p' "$TMP_DIR/main-games-default.out" | tail -1)"
[[ -f "$DEFAULT_GAMES_REPORT/wine-prefixes.tsv" ]] || die 'la auditoría de juegos por defecto no generó su informe'
[[ -f "$DEFAULT_GAMES_VALIDATION" ]] || die 'la auditoría de juegos por defecto no generó validación'
[[ "$DEFAULT_GAMES_VALIDATION" == "$DEFAULT_GAMES_REPORT/configuration-validation.tsv" ]] || die 'el informe y la validación de juegos por defecto quedaron separados'
ok 'auditoría de juegos con todos los valores predeterminados'

run_menu main-games "1\n2\nn\n\n$GAMES_OUT\nq\n" 'Auditoría de juegos, Wine y Proton'
assert_file "$GAMES_OUT/wine-prefixes.tsv"
assert_file "$GAMES_OUT/configuration-validation.tsv"
assert_file "$GAMES_OUT/configuration-databases.tsv"
grep -Fq 'ExamplePrefix' "$GAMES_OUT/wine-prefixes.tsv" || die 'games no detectó el prefijo'
grep -Fq 'Heroic' "$GAMES_OUT/configuration-validation.tsv" || die 'games no validó Heroic'
grep -Fq 'Lutris' "$GAMES_OUT/configuration-validation.tsv" || die 'games no validó Lutris'
ok 'auditoría de juegos, prefijo, Heroic, Lutris, UMU y Steam'

run_menu main-games-full "1\n2\ny\n$FIXTURE\n$TMP_DIR/games-full\n" 'Auditoría de juegos, Wine y Proton'
grep -Fq 'Informe:' "$TMP_DIR/main-games-full.out" || die 'la auditoría completa de juegos no generó informe'
ok 'auditoría de juegos con escaneo completo y ruta modificada'

run_menu main-packages "1\n3\n$PKG_OUT\nq\n" 'Inventario de paquetes y almacenes'
assert_file "$PKG_OUT/inventory.tsv"
assert_file "$PKG_OUT/summary.txt"
grep -Fq $'kind\tscope\tmanager\tdata\tpath\tbytes' "$PKG_OUT/inventory.tsv" || die 'el inventario compacto no generó su cabecera'
grep -Fq 'Informe:' "$TMP_DIR/main-packages.out" || die 'el inventario no informó su ruta'
grep -Fq 'Elige una opción' "$TMP_DIR/main-packages.out" || die 'el menú no ofreció leer el informe generado'
ok 'inventario compacto de gestores, paquetes y artefactos'

AUTO_SCRIPT="$TMP_DIR/automation.sh"
printf '#!/usr/bin/env bash\nprintf automation-e2e-ok\n' > "$AUTO_SCRIPT"
chmod +x "$AUTO_SCRIPT"
run_bash automation add --name e2e-script --program "$AUTO_SCRIPT" --cwd "$TMP_DIR" --args '--example "two words"' >"$TMP_DIR/automation-add.out"
run_bash automation list >"$TMP_DIR/automation-list.out"
grep -Fq 'e2e-script' "$TMP_DIR/automation-list.out" || die 'automation add/list no registró el script'
run_bash automation run e2e-script >"$TMP_DIR/automation-run.out"
grep -Fq 'automation-e2e-ok' "$TMP_DIR/automation-run.out" || die 'automation run no ejecutó el script registrado'
run_bash automation remove e2e-script >"$TMP_DIR/automation-remove.out"
! grep -Fq 'e2e-script' <(run_bash automation list) || die 'automation remove no retiró el registro'
ok 'Importar scripts: registrar, listar, ejecutar y retirar en configuración aislada'

run_menu main-packages-empty "1\n3\n\n" 'Pulsa Enter para volver:'
ok 'opción 8 conserva la ventana cuando la ruta queda vacía'
run_menu main-storage "3\n1\n11\n\nq\n" 'Herramientas de almacenamiento Linux'
run_menu main-storage-guided "3\n1\n12\n\nq\nq\n" 'Selección segura de almacenamiento'
run_menu main-clean "2\n6\nq\n" 'Limpieza protegida'
run_menu main-registry "3\n6\n1\nq\n" 'Registros y configuración Linux'
run_menu main-tools "4\n1\nq\nq\n" 'Operaciones Git y GitHub'
run_menu main-software "4\n2\n1\nq\nq\nq\n" 'Stores de paquetes detectadas'
run_menu main-utilities "4\n7\nq\n\nq\nq\n" 'Utilidades del sistema Linux'

BOOT_MENU_OUT="$TMP_DIR/boot-menu.out"
printf '2\n\n3\n\nq\n' | timeout 60 env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" \
    XDG_DATA_HOME="$XDG_DATA_HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 "$BIN" boot menu >"$BOOT_MENU_OUT" 2>&1
grep -Fq 'Arranque, GRUB, EFI y systemd-boot' "$BOOT_MENU_OUT" || die 'el menú de arranque no se abrió'
grep -Fq 'Entradas EFI / NVRAM' "$BOOT_MENU_OUT" || die 'el menú de arranque no ofrece EFI como opción'
grep -Fq 'Entradas GRUB' "$BOOT_MENU_OUT" || die 'el menú de arranque no ofrece selector/listado GRUB'
ok 'menú de arranque con EFI, GRUB y systemd-boot'

NETWORK_MENU_OUT="$TMP_DIR/network-menu.out"
printf '1\n\nq\n' | timeout 60 env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" \
    XDG_DATA_HOME="$XDG_DATA_HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 "$BIN" native network menu >"$NETWORK_MENU_OUT" 2>&1
grep -Fq 'Gestión de red Linux' "$NETWORK_MENU_OUT" || die 'el menú de red no se abrió'
grep -Fq 'Activar interfaz' "$NETWORK_MENU_OUT" || die 'el menú de red no ofrece acciones de interfaz'
grep -Fq 'Conectar con NetworkManager' "$NETWORK_MENU_OUT" || die 'el menú de red no ofrece NetworkManager'
ok 'menú de red con interfaces, DNS y NetworkManager'
run_menu main-multi-session "1\n3\n$PKG_OUT\n\nq\n5\nq\nq\n" 'Operación terminada correctamente.'
[[ "$(grep -o '=== LTools' "$TMP_DIR/main-multi-session.out" | wc -l)" -ge 2 ]] || die 'el menú no volvió a mostrarse tras una acción'
grep -Fq $'\033[2J\033[H=== LTools' "$TMP_DIR/main-multi-session.out" || die 'el menú no limpió la pantalla al volver'
grep -Fq 'Informe de paquetes:' "$TMP_DIR/main-multi-session.out" || die 'la sesión múltiple no ejecutó la opción 8'
[[ -z "$(sed -n 's/^Plan: //p' "$TMP_DIR/main-multi-session.out")" ]] || die 'la sesión de consultas generó un plan innecesario'
ok 'sesión múltiple: volver con Enter, cambiar de opción y no crear planes innecesarios'

printf 'E2E: recorriendo todas las opciones principales del menú...\n'
run_menu main-clean-empty "2\n6\n" 'LTools'
run_menu main-prefix "1\n4\n" 'Prefijos detectados:'
run_menu main-defaults "6\n1\n" 'Defaults efectivos'
run_menu main-system "3\n2\nq\n" 'Servicios, procesos y journal'
run_menu main-accounts "3\n3\nq\n" 'Usuarios, grupos y sesiones Linux'
run_menu main-native "3\n4\nq\n" 'Red, hardware, energía y seguridad Linux'
run_menu main-native-tools "3\n4\nq\n\nq\n\nq\n" 'Red, hardware, energía y seguridad Linux'
run_menu main-native-containers "4\n5\nq\n\nq\n\nq\n" 'Docker / Podman'
run_menu main-native-kubernetes "4\n6\nq\n\nq\n\nq\n" 'Kubernetes'
run_menu main-storage-guide "3\n1\n17\n\nq\nq\n" 'Guía de particionado Linux'
run_menu main-services-doctor "2\n1\n" 'Diagnóstico'
run_menu main-services-native-diagnostics "2\n7\n" 'Diagnóstico nativo del sistema'
run_menu main-automation-category "5\nq\n" 'Automatización'
run_menu main-guided-actions "5\n2\nq\nq\n" 'Acciones guiadas'
run_menu main-settings-category "7\nq\n" 'Ajustes'
run_menu main-help "h\n" 'Comandos:'
run_menu main-quit "q\n" 'LTools'
if grep -Fq 'Opción no válida' "$TMP_DIR/main-quit.out"; then
    die 'q se trató como una opción inválida'
fi
run_menu main-invalid "x\nq\n" 'Opción no válida'
ok 'categorías principales, ayuda, salida y entrada inválida'

LANGUAGE_PROMPTS=(
    'es:Enter para volver' 'en:Enter to go back' 'de:Enter zum Zurückgehen'
    'fr:Entrée pour revenir' 'pt:Enter para voltar' 'it:Invio per tornare indietro'
    'pl:Enter, aby wrócić'
    'ar:Enter للعودة' 'hi:वापस जाने के लिए Enter' 'ja:戻るにはEnter'
    'ko:돌아가려면 Enter' 'ro:Enter pentru revenire' 'ru:Enter — назад'
    'uk:Enter для повернення' 'zh:按 Enter 返回'
)
for language_prompt in "${LANGUAGE_PROMPTS[@]}"; do
    language="${language_prompt%%:*}"
    marker="${language_prompt#*:}"
    language_output="$TMP_DIR/menu-language-${language}.out"
    printf '\n' | env LTOOLS_LANG="$language" HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
        LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$language_output" 2>&1
    grep -Fq -- "$marker" "$language_output" || die "el prompt del menú no se tradujo a $language"
    ! grep -Fq 'Opción no válida' "$language_output" || die "Enter se trató como inválido en $language"
done
ok 'prompts de navegación y Enter vacío en los 15 idiomas compatibles'

LANGUAGE_CATEGORIES=(
    'es:Auditar / Inventariar' 'en:Audit / Inventory'
    'de:Prüfen / Inventarisieren' 'fr:Auditer / Inventorier'
    'pt:Auditar / Inventariar' 'it:Audit / Inventario'
    'pl:Audyt / Inwentaryzacja'
    'ar:التدقيق / الجرد' 'hi:ऑडिट / इन्वेंटरी' 'ja:監査 / インベントリ'
    'ko:감사 / 인벤토리' 'ro:Audit / Inventar' 'ru:Проверка / Инвентаризация'
    'uk:Аудит / Інвентаризація' 'zh:审计 / 清单'
)
for language_category in "${LANGUAGE_CATEGORIES[@]}"; do
    language="${language_category%%:*}"
    marker="${language_category#*:}"
    language_output="$TMP_DIR/menu-category-${language}.out"
    printf '\n' | env LTOOLS_LANG="$language" HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
        LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$language_output" 2>&1
    grep -Fq -- "$marker" "$language_output" || die "la categoría principal no se tradujo a $language"
done
ok 'categorías principales traducidas en los 15 idiomas compatibles'

SETTINGS_GUIDE_LABELS=(
    'es:Guía de ajustes y visibilidad' 'en:Settings and visibility guide'
    'de:Hilfe zu Einstellungen und Sichtbarkeit'
    'fr:Guide des réglages et de la visibilité'
    'pt:Guia de definições e visibilidade' 'it:Guida a impostazioni e visibilità'
    'pl:Przewodnik po ustawieniach i widoczności'
    'ar:دليل الإعدادات والظهور' 'hi:सेटिंग और दृश्यता मार्गदर्शिका'
    'ja:設定と表示のガイド' 'ko:설정 및 표시 안내'
    'ro:Ghid pentru setări și vizibilitate'
    'ru:Руководство по настройкам и видимости'
    'uk:Посібник із налаштувань і видимості' 'zh:设置与可见性指南'
)
for settings_guide_label in "${SETTINGS_GUIDE_LABELS[@]}"; do
    language="${settings_guide_label%%:*}"
    marker="${settings_guide_label#*:}"
    guide_output="$TMP_DIR/settings-guide-${language}.out"
    env LTOOLS_LANG="$language" LTOOLS_NO_MOUNTS=1 "$BIN" guide gui settings >"$guide_output" 2>&1 ||
        die "la guía GUI/Ajustes no respondió en $language"
    grep -Fq -- "$marker" "$guide_output" ||
        die "la guía GUI/Ajustes no refleja sus controles traducidos en $language"
done
ok 'guía de Ajustes coincide con su control de ayuda en los 15 idiomas compatibles'

ELEVATION_SETTINGS=(
    'es:¿Elevar acciones modificadoras por defecto con sudo/UAC?:desactivada'
    'en:Elevate modifying actions by default with sudo/UAC?:disabled'
    'de:Ändernde Aktionen standardmäßig mit sudo/UAC erhöhen?:deaktiviert'
    'fr:Élever les actions modificatrices par défaut avec sudo/UAC ?:désactivée'
    'pt:Elevar ações modificadoras por predefinição com sudo/UAC?:desativada'
    'it:Elevare per impostazione predefinita le azioni modificative con sudo/UAC?:disattivata'
    'pl:Uruchamiać domyślnie działania modyfikujące z sudo/UAC?:wyłączona'
    'ar:رفع صلاحيات الإجراءات المعدِّلة افتراضيًا باستخدام sudo/UAC؟:معطّلة'
    'hi:संशोधन करने वाली कार्रवाइयों को sudo/UAC के साथ डिफ़ॉल्ट रूप से उन्नत करें?:निष्क्रिय'
    'ja:変更操作を sudo/UAC で既定の昇格実行にしますか？:無効'
    'ko:수정 작업을 기본적으로 sudo/UAC로 권한 상승하여 실행할까요?:비활성화됨'
    'ro:Rulezi implicit acțiunile modificatoare cu sudo/UAC?:dezactivată'
    'ru:Запускать изменяющие действия по умолчанию с sudo/UAC?:отключена'
    'uk:Типово запускати дії зі зміною системи через sudo/UAC?:вимкнено'
    'zh:是否默认使用 sudo/UAC 提升修改操作？:已禁用'
)
for elevation_setting in "${ELEVATION_SETTINGS[@]}"; do
    language="${elevation_setting%%:*}"
    remainder="${elevation_setting#*:}"
    prompt_marker="${remainder%:*}"
    disabled_marker="${remainder##*:}"
    elevation_output="$TMP_DIR/elevation-${language}.out"
    printf '7\n4\nn\n' | env LTOOLS_LANG="$language" HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
        LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$elevation_output" 2>&1
    grep -Fq -- "$prompt_marker" "$elevation_output" ||
        die "el prompt de elevación no se tradujo a $language"
    grep -Fq -- "$disabled_marker" "$elevation_output" ||
        die "el estado de elevación no se tradujo a $language"
done
ok 'menú CLI de elevación y estado guardado traducidos en los 15 idiomas compatibles'

CLEAN_PATH="$HOME/cache-candidate"
mkdir -p "$CLEAN_PATH"
printf 'keep-me\n' > "$CLEAN_PATH/file.txt"
CLEAN_PLAN="$TMP_DIR/clean-plan.tsv"
printf 'y\n' | run_bash clean --dry-run --path "$CLEAN_PATH" --plan "$CLEAN_PLAN" >/dev/null
[[ -d "$CLEAN_PATH" ]] || die 'clean --dry-run modificó una ruta'
grep -Fq $'trash-move\t' "$CLEAN_PLAN" || die 'clean --dry-run no registró el plan'
ok 'limpieza protegida en dry-run sin mutar datos'
CLEAN_FAKE_RG="$TMP_DIR/clean-fake-rg"
mkdir -p "$CLEAN_FAKE_RG"
cat >"$CLEAN_FAKE_RG/rg" <<'EOF'
#!/bin/sh
printf 'synthetic unreadable configuration tree\n' >&2
exit 2
EOF
chmod +x "$CLEAN_FAKE_RG/rg"
CLEAN_UNVERIFIED_PATH="$TMP_DIR/clean-reference-scan-failure"
printf 'must remain untouched\n' >"$CLEAN_UNVERIFIED_PATH"
if output="$(PATH="$CLEAN_FAKE_RG:$PATH" run_bash clean --dry-run \
    --path "$CLEAN_UNVERIFIED_PATH" --plan "$TMP_DIR/clean-failed-scan-plan.tsv" <<<y 2>&1)"; then
    die 'clean trató una búsqueda rg fallida como ausencia de referencias'
fi
grep -Fq 'no se pudo completar la comprobación de referencias' <<<"$output" ||
    die 'clean no explicó por qué bloqueó una búsqueda de referencias fallida'
[[ -f "$CLEAN_UNVERIFIED_PATH" ]] || die 'la ruta no verificada dejó de existir'
ok 'limpieza de rutas falla de forma segura si rg no puede completar la búsqueda'

MAP_COPY_DEST="$TMP_DIR/map-copy.bin"
MAP_MOVE_DEST="$TMP_DIR/map-move.bin"
MAP_DELETE_TARGET="$FIXTURE/duplicate-b.bin"
run_bash storage manage copy --source "$FIXTURE/duplicate-a.bin" --destination "$MAP_COPY_DEST" --dry-run >"$TMP_DIR/map-copy.out"
grep -Fq 'Simulación: se ejecutaría copy' "$TMP_DIR/map-copy.out" || die 'el mapa no expuso la acción de copiar en dry-run'
run_bash storage manage move --source "$FIXTURE/duplicate-a.bin" --destination "$MAP_MOVE_DEST" --dry-run >"$TMP_DIR/map-move.out"
grep -Fq 'Simulación: se ejecutaría move' "$TMP_DIR/map-move.out" || die 'el mapa no expuso la acción de mover en dry-run'
run_bash storage manage delete --path "$MAP_DELETE_TARGET" --dry-run >"$TMP_DIR/map-delete.out"
[[ -s "$TMP_DIR/map-delete.out" ]] || die 'el mapa no expuso la acción de papelera en dry-run'
[[ -f "$FIXTURE/duplicate-a.bin" && -f "$MAP_DELETE_TARGET" && ! -e "$MAP_COPY_DEST" && ! -e "$MAP_MOVE_DEST" ]] ||
    die 'las acciones del mapa en dry-run modificaron el fixture'
ok 'acciones del mapa (copiar, mover y papelera) verificadas sin mutar datos'

if [[ "$(id -u)" -eq 0 ]]; then
    printf '  SKIP  permisos efectivos del mapa: root puede eludir los bits de modo\n'
else
    MAP_READ_ONLY="$TMP_DIR/map-read-only.txt"
    printf 'read-only fixture\n' >"$MAP_READ_ONLY"
    chmod 0444 "$MAP_READ_ONLY"
    run_bash storage map --path "$MAP_READ_ONLY" --depth 0 --format json >"$TMP_DIR/map-read-only.json"
    grep -Fq '"writable":false' "$TMP_DIR/map-read-only.json" ||
        die 'el mapa marcó como escribible un archivo que el usuario actual no puede modificar'
    chmod 0644 "$MAP_READ_ONLY"
    run_bash storage map --path "$MAP_READ_ONLY" --depth 0 --format json >"$TMP_DIR/map-writable.json"
    grep -Fq '"writable":true' "$TMP_DIR/map-writable.json" ||
        die 'el mapa no reconoció un archivo escribible por el usuario actual'
    ok 'el mapa contrasta escritura efectiva del usuario, no solo bits del modo'
fi

MAP_COPY_SOURCE="$TMP_DIR/map-copy-source.txt"
MAP_COPY_REAL="$TMP_DIR/map-copy-real.txt"
MAP_MOVE_SOURCE="$TMP_DIR/map-move-source.txt"
MAP_MOVE_REAL="$TMP_DIR/map-move-real.txt"
MAP_DELETE_REAL="$TMP_DIR/map-delete-real.txt"
printf 'copy fixture\n' >"$MAP_COPY_SOURCE"
printf 'move fixture\n' >"$MAP_MOVE_SOURCE"
printf 'trash fixture\n' >"$MAP_DELETE_REAL"
MAP_FAKE_BIN="$TMP_DIR/fake-bin"
MAP_TRASH_DIR="$TMP_DIR/fake-trash"
mkdir -p "$MAP_FAKE_BIN" "$MAP_TRASH_DIR"
cat >"$MAP_FAKE_BIN/gio" <<'EOF'
#!/usr/bin/env bash
set -eu
[[ "$#" -eq 3 && "$1" == trash && "$2" == -- && -n "${LTOOLS_TEST_TRASH_DIR:-}" ]]
mv -- "$3" "$LTOOLS_TEST_TRASH_DIR/"
EOF
chmod +x "$MAP_FAKE_BIN/gio"
MAP_FAKE_PACKAGE_BIN="$TMP_DIR/fake-package-bin"
mkdir -p "$MAP_FAKE_PACKAGE_BIN"
cat >"$MAP_FAKE_PACKAGE_BIN/pacman" <<'EOF'
#!/usr/bin/env bash
set -eu
printf '%s\n' "$*" >>"$LTOOLS_TEST_PACMAN_LOG"
if [[ "$*" == '-Qdtq' ]]; then
    exit 1
fi
printf 'synthetic package database failure\n' >&2
exit 23
EOF
chmod +x "$MAP_FAKE_PACKAGE_BIN/pacman"
MAP_PACKAGE_REPORT="$TMP_DIR/package-query-report"
MAP_PACMAN_LOG="$TMP_DIR/pacman-queries.log"
if PATH="$MAP_FAKE_PACKAGE_BIN:$PATH" LTOOLS_TEST_PACMAN_LOG="$MAP_PACMAN_LOG" \
    run_rust packages --packages-only --full --out "$MAP_PACKAGE_REPORT" \
    >"$TMP_DIR/package-query.out" 2>&1; then
        die 'el inventario informó éxito pese a tener consultas de gestor fallidas'
    fi
[[ -s "$MAP_PACKAGE_REPORT/manager-errors.tsv" ]] ||
    die 'el inventario de paquetes no conservó errores de consulta'
grep -Fq 'synthetic package database failure' "$MAP_PACKAGE_REPORT/manager-errors.tsv" ||
    die 'el informe de paquetes no incluyó stderr del gestor'
if grep -Fq $'pacman\t-Qdtq\t' "$MAP_PACKAGE_REPORT/manager-errors.tsv"; then
    die 'el informe trató la ausencia normal de paquetes huérfanos como error'
fi
for query_args in '-Q' '-Qm' '-Qdtq' '-Qqe'; do
    [[ "$(grep -Fxc -- "$query_args" "$MAP_PACMAN_LOG")" -eq 1 ]] ||
        die "la consulta pacman '$query_args' no se ejecutó exactamente una vez"
done
[[ -s "$MAP_PACKAGE_REPORT/packages-pacman.tsv" ]] ||
    die 'el modo --full no conservó el TSV detallado de pacman'
[[ "$(awk -F '\t' '$1 == "pacman" { count++ } END { print count+0 }' \
    "$MAP_PACKAGE_REPORT/manager-errors.tsv")" -eq 3 ]] ||
    die 'el informe no registró exactamente las tres consultas pacman fallidas'
ok 'inventario Rust conserva stderr/código de error y reconoce pacman sin huérfanos'
run_bash storage manage copy --source "$MAP_COPY_SOURCE" --destination "$MAP_COPY_REAL" --yes >/dev/null
cmp -s "$MAP_COPY_SOURCE" "$MAP_COPY_REAL" || die 'storage manage copy no conservó el contenido'
MAP_SPECIAL_MODE_SOURCE="$TMP_DIR/map-copy-special-mode-source"
MAP_SPECIAL_MODE_COPY="$TMP_DIR/map-copy-special-mode-copy"
printf 'unprivileged executable fixture\n' >"$MAP_SPECIAL_MODE_SOURCE"
chmod 4755 "$MAP_SPECIAL_MODE_SOURCE"
run_rust storage manage copy --source "$MAP_SPECIAL_MODE_SOURCE" \
    --destination "$MAP_SPECIAL_MODE_COPY" --yes >/dev/null
[[ "$(stat -c '%a' "$MAP_SPECIAL_MODE_COPY")" == 755 ]] ||
    die 'storage manage copy propagó el bit SUID a un destino potencialmente propiedad de root'
ok 'copias Rust conservan rwx y eliminan bits especiales SUID/SGID/sticky'
MAP_EXISTING_DEST="$TMP_DIR/map-existing-destination.txt"
printf 'must stay unchanged\n' >"$MAP_EXISTING_DEST"
if run_bash storage manage copy --source "$MAP_COPY_SOURCE" --destination "$MAP_EXISTING_DEST" --yes >"$TMP_DIR/map-copy-collision.out" 2>&1; then
    die 'storage manage copy sobrescribió un destino existente'
fi
[[ "$(<"$MAP_EXISTING_DEST")" == 'must stay unchanged' ]] || die 'un destino existente cambió tras rechazar la copia'
MAP_SOURCE_DIR="$TMP_DIR/map-copy-tree"
MAP_SOURCE_ALIAS="$TMP_DIR/map-copy-tree-alias"
mkdir -p "$MAP_SOURCE_DIR"
printf 'tree fixture\n' >"$MAP_SOURCE_DIR/data.txt"
ln -s "$MAP_SOURCE_DIR" "$MAP_SOURCE_ALIAS"
if run_bash storage manage copy --source "$MAP_SOURCE_DIR" --destination "$MAP_SOURCE_ALIAS/nested-copy" --yes >"$TMP_DIR/map-copy-alias.out" 2>&1; then
    die 'storage manage copy permitió crear el destino dentro de la fuente a través de un enlace'
fi
[[ ! -e "$MAP_SOURCE_DIR/nested-copy" ]] || die 'la copia mediante enlace comenzó a mutar la fuente'
if run_rust storage manage copy --source "$MAP_SOURCE_ALIAS" --destination "$TMP_DIR/map-copy-link-target" --yes >"$TMP_DIR/map-copy-link.out" 2>&1; then
    die 'storage manage copy siguió un enlace simbólico de origen'
fi
[[ ! -e "$TMP_DIR/map-copy-link-target" ]] || die 'copiar un enlace creó inesperadamente una copia de su destino'
run_bash storage manage move --source "$MAP_MOVE_SOURCE" --destination "$MAP_MOVE_REAL" --yes >/dev/null
[[ ! -e "$MAP_MOVE_SOURCE" && -f "$MAP_MOVE_REAL" ]] || die 'storage manage move no trasladó el fixture'
MAP_MOVE_LINK="$TMP_DIR/map-move-link"
MAP_MOVED_LINK="$TMP_DIR/map-moved-link"
ln -s "$MAP_COPY_SOURCE" "$MAP_MOVE_LINK"
run_rust storage manage move --source "$MAP_MOVE_LINK" --destination "$MAP_MOVED_LINK" --yes >/dev/null
[[ ! -L "$MAP_MOVE_LINK" && -L "$MAP_MOVED_LINK" ]] || die 'mover en el mismo volumen no conservó el enlace simbólico'
cmp -s "$MAP_COPY_SOURCE" "$MAP_COPY_REAL" || die 'mover un enlace alteró su destino'
MAP_ARCHIVE_LINK="$TMP_DIR/map-archive-link"
ln -s "$MAP_SOURCE_DIR" "$MAP_ARCHIVE_LINK"
if run_rust storage manage tar --source "$MAP_ARCHIVE_LINK" --destination "$TMP_DIR/map-link.tar" --yes >"$TMP_DIR/map-archive-link.out" 2>&1; then
    die 'storage manage tar siguió un enlace simbólico de origen'
fi
[[ ! -e "$TMP_DIR/map-link.tar" ]] || die 'archivar un enlace creó un archivo inesperadamente'
MAP_ARCHIVE_TREE="$TMP_DIR/map-archive-tree"
MAP_ARCHIVE_TARGET="$TMP_DIR/map-archive-outside"
mkdir -p "$MAP_ARCHIVE_TREE" "$MAP_ARCHIVE_TARGET"
printf 'must not enter archive\n' >"$MAP_ARCHIVE_TARGET/secret.txt"
ln -s "$MAP_ARCHIVE_TARGET" "$MAP_ARCHIVE_TREE/redirect"
if run_rust storage manage zip --source "$MAP_ARCHIVE_TREE" \
    --destination "$TMP_DIR/map-nested-link.zip" --yes >"$TMP_DIR/map-archive-nested.out" 2>&1; then
    die 'storage manage zip atravesó un enlace simbólico anidado'
fi
[[ ! -e "$TMP_DIR/map-nested-link.zip" && -f "$MAP_ARCHIVE_TARGET/secret.txt" ]] ||
    die 'el archivo atravesó un enlace simbólico anidado o alteró su destino'
MAP_ARCHIVE_SAFE_SOURCE="$TMP_DIR/map-archive-safe-source"
mkdir -p "$MAP_ARCHIVE_SAFE_SOURCE"
printf 'archive fixture\n' >"$MAP_ARCHIVE_SAFE_SOURCE/inside.txt"
MAP_ARCHIVE_SAFE_TAR="$TMP_DIR/map-safe.tar"
run_rust storage manage tar --source "$MAP_ARCHIVE_SAFE_SOURCE" \
    --destination "$MAP_ARCHIVE_SAFE_TAR" --yes >"$TMP_DIR/map-archive-tar.out" 2>&1
tar -tf "$MAP_ARCHIVE_SAFE_TAR" | grep -Fq 'inside.txt' || die 'el TAR seguro no incluye el contenido esperado'
[[ -f "$MAP_ARCHIVE_SAFE_SOURCE/inside.txt" ]] || die 'crear un TAR modificó la fuente'
MAP_ARCHIVE_EXISTING="$TMP_DIR/map-existing.tar"
printf 'preserve existing archive\n' >"$MAP_ARCHIVE_EXISTING"
if run_rust storage manage tar --source "$MAP_ARCHIVE_SAFE_SOURCE" \
    --destination "$MAP_ARCHIVE_EXISTING" --yes >"$TMP_DIR/map-archive-existing.out" 2>&1; then
    die 'el archivador aceptó un destino preexistente'
fi
grep -Fxq 'preserve existing archive' "$MAP_ARCHIVE_EXISTING" || die 'el archivador sobrescribió un destino preexistente'
if command -v zip >/dev/null 2>&1; then
    MAP_ARCHIVE_SAFE_ZIP="$TMP_DIR/map-safe.zip"
    run_rust storage manage zip --source "$MAP_ARCHIVE_SAFE_SOURCE" \
        --destination "$MAP_ARCHIVE_SAFE_ZIP" --yes >"$TMP_DIR/map-archive-zip.out" 2>&1
    zip -T "$MAP_ARCHIVE_SAFE_ZIP" >/dev/null || die 'el ZIP seguro no es un archivo válido'
    ok 'TAR/ZIP se publican completos, conservan la fuente y no sustituyen destinos existentes'
else
    ok 'TAR validado; ZIP omitido porque no está instalado'
fi
if find "$TMP_DIR" -mindepth 1 -maxdepth 1 -type d -name '.ltools-stage-*' -print -quit | grep -q .; then
    die 'el gestor de archivos dejó directorios de staging tras completar las operaciones'
fi
MAP_ROLLBACK_SOURCE="$TMP_DIR/map-rollback-source.txt"
MAP_ROLLBACK_DEST="$TMP_DIR/map-rollback-destination.txt"
MAP_ROLLBACK_PLAN="$TMP_DIR/map-rollback-plan.tsv"
printf 'rollback fixture\n' >"$MAP_ROLLBACK_SOURCE"
run_bash storage manage move --source "$MAP_ROLLBACK_SOURCE" --destination "$MAP_ROLLBACK_DEST" --yes --plan "$MAP_ROLLBACK_PLAN" >/dev/null
grep -Fq $'path-move\t' "$MAP_ROLLBACK_PLAN" || die 'storage manage move no registró el origen para rollback'
printf 'y\n' | run_bash rollback --plan "$MAP_ROLLBACK_PLAN" >"$TMP_DIR/map-rollback.out"
[[ -f "$MAP_ROLLBACK_SOURCE" && ! -e "$MAP_ROLLBACK_DEST" ]] || die 'el rollback no restauró un movimiento del mapa'
MAP_ROLLBACK_DANGLING_SOURCE="$TMP_DIR/map-rollback-dangling-source"
MAP_ROLLBACK_DANGLING_DEST="$TMP_DIR/map-rollback-dangling-destination"
MAP_ROLLBACK_DANGLING_PLAN="$TMP_DIR/map-rollback-dangling-plan.tsv"
ln -s "$TMP_DIR/missing-rollback-target" "$MAP_ROLLBACK_DANGLING_SOURCE"
run_rust storage manage move --source "$MAP_ROLLBACK_DANGLING_SOURCE" \
    --destination "$MAP_ROLLBACK_DANGLING_DEST" --yes --plan "$MAP_ROLLBACK_DANGLING_PLAN" >/dev/null
[[ ! -L "$MAP_ROLLBACK_DANGLING_SOURCE" && -L "$MAP_ROLLBACK_DANGLING_DEST" ]] ||
    die 'mover un enlace colgante no trasladó el enlace'
printf 'y\n' | run_rust rollback --plan "$MAP_ROLLBACK_DANGLING_PLAN" \
    >"$TMP_DIR/map-rollback-dangling.out"
[[ -L "$MAP_ROLLBACK_DANGLING_SOURCE" && ! -L "$MAP_ROLLBACK_DANGLING_DEST" ]] ||
    die 'el rollback no restauró el movimiento del enlace colgante'
MAP_SYMLINK_TARGET="$TMP_DIR/map-trash-target.txt"
MAP_SYMLINK="$TMP_DIR/map-trash-link"
MAP_DANGLING="$TMP_DIR/map-trash-dangling-link"
printf 'preserve target\n' >"$MAP_SYMLINK_TARGET"
ln -s "$MAP_SYMLINK_TARGET" "$MAP_SYMLINK"
ln -s "$TMP_DIR/missing-target" "$MAP_DANGLING"
PATH="$MAP_FAKE_BIN:$PATH" LTOOLS_TEST_TRASH_DIR="$MAP_TRASH_DIR" \
    run_rust storage manage delete --path "$MAP_DELETE_REAL" --yes >/dev/null
[[ ! -e "$MAP_DELETE_REAL" && -f "$MAP_TRASH_DIR/map-delete-real.txt" ]] ||
    die 'storage manage delete no retiró el fixture a la papelera'
PATH="$MAP_FAKE_BIN:$PATH" LTOOLS_TEST_TRASH_DIR="$MAP_TRASH_DIR" \
    run_rust storage manage delete --path "$MAP_SYMLINK" --yes >/dev/null
[[ ! -L "$MAP_SYMLINK" && -L "$MAP_TRASH_DIR/map-trash-link" ]] ||
    die 'borrar a papelera siguió el destino de un enlace simbólico'
[[ "$(<"$MAP_SYMLINK_TARGET")" == 'preserve target' ]] ||
    die 'borrar a papelera alteró el destino del enlace'
PATH="$MAP_FAKE_BIN:$PATH" LTOOLS_TEST_TRASH_DIR="$MAP_TRASH_DIR" \
    run_rust storage manage delete --path "$MAP_DANGLING" --yes >/dev/null
[[ ! -L "$MAP_DANGLING" && -L "$MAP_TRASH_DIR/map-trash-dangling-link" ]] ||
    die 'borrar a papelera no gestionó un enlace simbólico colgante'
ok 'mapa: copia, movimiento, archivado, papelera y rollback respetan enlaces simbólicos'

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

PACKAGE_ONLY_OUT="$TMP_DIR/packages-only"
run_bash packages --packages-only --out "$PACKAGE_ONLY_OUT" >/dev/null
assert_file "$PACKAGE_ONLY_OUT/inventory.tsv"
ok 'inventario de paquetes en modo solo paquetes'
REPORT_VIEW_OUTPUT="$("$BIN" report view --path "$PACKAGE_ONLY_OUT/summary.txt")"
grep -Fq 'Inventario principal:' <<<"$REPORT_VIEW_OUTPUT" || die 'report view no leyó el resumen directamente'
REPORT_MENU_OUTPUT="$(printf 'c\nq\n' | "$BIN" report menu --path "$PACKAGE_ONLY_OUT")"
grep -Fq 'Informe:' <<<"$REPORT_MENU_OUTPUT" || die 'report menu no mostró los informes disponibles'
grep -Fq 'Inventario principal:' <<<"$REPORT_MENU_OUTPUT" || die 'report menu no leyó el resumen seleccionado'
ok 'lector integrado: resumen directo y menú de informe'

PREFIX_INSPECT_OUTPUT="$TMP_DIR/prefix-inspect.out"
run_bash prefix inspect "$PREFIX" >"$PREFIX_INSPECT_OUTPUT"
grep -Fq 'Ejecutables/instaladores: 1' "$PREFIX_INSPECT_OUTPUT" || die 'inspect no contó el ejecutable del prefijo'
ok 'inspección detallada de un prefijo'

CREATE_PLAN="$TMP_DIR/create-prefix-plan.tsv"
CREATE_DEST="$TMP_DIR/created-prefix"
printf 'y\n' | run_bash prefix create --dry-run --dest "$CREATE_DEST" --plan "$CREATE_PLAN" >"$TMP_DIR/create-prefix.out"
grep -Fq 'Simulación: no se ejecutaría wineboot' "$TMP_DIR/create-prefix.out" || die 'create --dry-run no informó simulación'
[[ ! -e "$CREATE_DEST" ]] || die 'create --dry-run modificó el destino'
ok 'creación de prefijo en dry-run'

MIGRATE_PLAN="$TMP_DIR/migrate-prefix-plan.tsv"
MIGRATE_DEST="$TMP_DIR/migrated-prefix"
printf 'y\n' | run_bash prefix migrate --dry-run --source "$PREFIX" --dest "$MIGRATE_DEST" --plan "$MIGRATE_PLAN" >"$TMP_DIR/migrate-prefix.out"
grep -Fq 'Simulación: no se copiaría' "$TMP_DIR/migrate-prefix.out" || die 'migrate --dry-run no informó simulación'
[[ ! -e "$MIGRATE_DEST" ]] || die 'migrate --dry-run modificó el destino'
ok 'migración de prefijo en dry-run'

SYSTEM_USER_PLAN="$TMP_DIR/system-user-plan.tsv"
run_bash system --dry-run --user services --plan "$SYSTEM_USER_PLAN" >"$TMP_DIR/system-user.out"
assert_file "$SYSTEM_USER_PLAN"
ok 'servicios de usuario con parámetros modificados'

SYSTEM_BOTH_OUTPUT="$TMP_DIR/system-both.out"
run_bash system services --scope both --filter all --category all --limit 8 >"$SYSTEM_BOTH_OUTPUT"
grep -Fq 'del sistema y del usuario' "$SYSTEM_BOTH_OUTPUT" || die 'services --scope both no combinó los ámbitos'
grep -Fq 'Columnas:' "$SYSTEM_BOTH_OUTPUT" || die 'services no mostró sus columnas completas'
ok 'servicios combinados con filtros, categorías y columnas propias'

SYSTEM_AUTO_OUTPUT="$TMP_DIR/system-automatic.out"
run_bash system services --scope system --filter automatic --limit 4 >"$SYSTEM_AUTO_OUTPUT"
grep -Fq 'Filtro: automatic' "$SYSTEM_AUTO_OUTPUT" || die 'services no aceptó el filtro automático'
grep -Fq 'origen' "$SYSTEM_AUTO_OUTPUT" || die 'services no mostró el origen de la unidad'
SYSTEM_MANUAL_OUTPUT="$TMP_DIR/system-manual.out"
run_bash system services --scope system --filter manual --limit 4 >"$SYSTEM_MANUAL_OUTPUT"
grep -Fq 'Filtro: manual' "$SYSTEM_MANUAL_OUTPUT" || die 'services no aceptó el filtro manual'
ok 'servicios automáticos/manuales con ámbito y origen'

SYSTEM_FAILED_OUTPUT="$TMP_DIR/system-failed.out"
run_bash system failed --journal >"$SYSTEM_FAILED_OUTPUT"
grep -Fq 'Servicios fallidos' "$SYSTEM_FAILED_OUTPUT" || die 'failed no mostró el resumen de fallos'
if ! grep -Fq 'No hay servicios fallidos' "$SYSTEM_FAILED_OUTPUT" &&
   ! grep -Eq '^  [^[:space:]]+ \|' "$SYSTEM_FAILED_OUTPUT"; then
    die 'failed no mostró ni la ausencia de fallos ni filas de servicios fallidos'
fi
ok 'servicios fallidos separados de not-found y estados normales'

SYSTEM_PROCESS_OUTPUT="$TMP_DIR/system-process.out"
run_bash system processes --sort cpu --limit 3 >"$SYSTEM_PROCESS_OUTPUT"
grep -Fq 'PID | PPID' "$SYSTEM_PROCESS_OUTPUT" || die 'processes no mostró columnas propias'
ok 'procesos filtrados por CPU y límite'

SYSTEM_JOURNAL_OUTPUT="$TMP_DIR/system-journal.out"
run_bash system journal --level error --hours 1 --limit 3 >"$SYSTEM_JOURNAL_OUTPUT"
grep -Fq 'Journal: nivel error' "$SYSTEM_JOURNAL_OUTPUT" || die 'journal no respetó el nivel solicitado'
ok 'journal filtrado por nivel, horas y límite'

SYSTEM_EXPORT_JSON="$TMP_DIR/system-report.json"
run_bash system export --scope both --format json --out "$SYSTEM_EXPORT_JSON" >/dev/null
assert_file "$SYSTEM_EXPORT_JSON"
if grep -Fq '"unit"' "$SYSTEM_EXPORT_JSON"; then
    grep -Fq '"scope"' "$SYSTEM_EXPORT_JSON" || die 'export json omitió el alcance del servicio'
else
    compact_json="$(tr -d '[:space:]' <"$SYSTEM_EXPORT_JSON")"
    [[ "$compact_json" == "[]" ]] || die 'export json no produjo ni filas ni un JSON vacío válido'
fi
ok 'informe de servicios exportado a JSON'

SYSTEM_DEP_OUTPUT="$TMP_DIR/system-dependencies.out"
run_bash system dependencies --unit ltools-test.service >"$SYSTEM_DEP_OUTPUT"
grep -Fq 'Dependencias de ltools-test.service' "$SYSTEM_DEP_OUTPUT" || die 'dependencies no mostró la unidad solicitada'
ok 'árbol de dependencias con unidad validada'

SYSTEM_ACTION_PLAN="$TMP_DIR/system-action-plan.tsv"
printf 'y\n' | run_bash system --dry-run service restart ltools-test.service --plan "$SYSTEM_ACTION_PLAN" >"$TMP_DIR/system-action.out"
grep -Fq 'Simulación: no se cambiaría el servicio' "$TMP_DIR/system-action.out" || die 'service --dry-run no protegió la acción'
assert_file "$SYSTEM_ACTION_PLAN"
ok 'gestión de servicio con confirmación y dry-run'

printf 'E2E: comprobando también menú Rust sin ejecutar acciones destructivas...\n'
RUST_MENU_OUTPUT="$TMP_DIR/rust-menu.out"
printf 'q\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_MENU_OUTPUT" 2>&1
grep -Fq 'LTools' "$RUST_MENU_OUTPUT" || die 'el menú Rust no arrancó'
grep -Fq 'Elige una opción' "$RUST_MENU_OUTPUT" || die 'el menú Rust no mostró el prompt'
ok 'menú Rust responde y sale limpiamente'

RUST_NOARGS_OUTPUT="$TMP_DIR/rust-noargs.out"
timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 LTOOLS_CLI=1 "$BIN" >"$RUST_NOARGS_OUTPUT" 2>&1
grep -Fq 'Comandos:' "$RUST_NOARGS_OUTPUT" || die 'el perfil CLI Rust no mostró la ayuda sin argumentos'
ok 'perfil CLI Rust muestra ayuda sin argumentos'

if command -v xvfb-run >/dev/null 2>&1 && command -v xdpyinfo >/dev/null 2>&1; then
    if timeout 10 xvfb-run -a xdpyinfo >/dev/null 2>&1; then
        timeout 30 xvfb-run -a env GDK_BACKEND=x11 HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
            LTOOLS_NO_MOUNTS=1 LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_REQUIRED=1 "$BIN" \
            >"$TMP_DIR/rust-gui-noargs.out" 2>&1 \
            || { sed -n '1,120p' "$TMP_DIR/rust-gui-noargs.out" >&2; die 'la GUI Rust no arrancó sin argumentos'; }
        ok 'ejecutable Rust normal abre y cierra la GUI sin argumentos'
    else
        (( REQUIRE_GUI == 0 )) || die '--require-gui no pudo iniciar Xvfb para el arranque GUI'
        printf '  SKIP  GUI Rust sin argumentos: Xvfb no puede crear un display aislado\n'
    fi
else
    (( REQUIRE_GUI == 0 )) || die '--require-gui exige xvfb-run y xdpyinfo para el arranque GUI'
    printf '  SKIP  GUI Rust sin argumentos: xvfb-run o xdpyinfo no está disponible\n'
fi

RUST_DEFAULTS_OUTPUT="$TMP_DIR/rust-defaults-menu.out"
printf '6\n1\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_DEFAULTS_OUTPUT" 2>&1
grep -Fq 'Defaults efectivos' "$RUST_DEFAULTS_OUTPUT" || die 'el menú Rust no ejecutó defaults'
ok 'menú Rust ejecuta la opción defaults'

RUST_DOCTOR_OUTPUT="$TMP_DIR/rust-doctor-menu.out"
printf '2\n1\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_DOCTOR_OUTPUT" 2>&1
grep -Fq 'LTools host diagnostics' "$RUST_DOCTOR_OUTPUT" || die 'el menú Rust no ejecutó doctor'
ok 'menú Rust ejecuta la opción doctor'

RUST_GAMES_OUTPUT="$TMP_DIR/rust-games-menu.out"
printf '1\n2\nn\n%s\n%s\n' "$FIXTURE" "$GAMES_OUT/rust-menu" | timeout 60 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_GAMES_OUTPUT" 2>&1
grep -Fq 'Validación de Heroic/Lutris/UMU/Steam' "$RUST_GAMES_OUTPUT" || die 'el menú Rust no ejecutó juegos'
assert_file "$GAMES_OUT/rust-menu/configuration-validation.tsv"
ok 'menú Rust ejecuta juegos, Wine, Proton y validación'

RUST_CLEAN_OUTPUT="$TMP_DIR/rust-clean-menu.out"
printf '2\n6\nq\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_CLEAN_OUTPUT" 2>&1
grep -Fq 'Limpieza protegida' "$RUST_CLEAN_OUTPUT" || die 'el menú Rust no abrió limpieza'
[[ "$(grep -o '=== LTools' "$RUST_CLEAN_OUTPUT" | wc -l)" -ge 2 ]] || die 'q no volvió directamente al menú principal desde limpieza'
ok 'menú Rust abre y cierra limpieza protegida'

RUST_CLEAN_PATH_OUTPUT="$TMP_DIR/rust-clean-path-menu.out"
printf '2\n6\n4\n%s\nn\nq\n' "$FIXTURE/duplicate-a.bin" | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_CLEAN_PATH_OUTPUT" 2>&1
grep -Fq 'Ruta:' "$RUST_CLEAN_PATH_OUTPUT" || die 'el submenú Rust clean no pidió una ruta'
[[ "$(grep -o '=== LTools' "$RUST_CLEAN_PATH_OUTPUT" | wc -l)" -ge 2 ]] || die 'clean no volvió al menú principal tras cancelar'
ok 'submenú Rust clean revisa y cancela una ruta'

RUST_CLEAN_ORPHANS_OUTPUT="$TMP_DIR/rust-clean-orphans-menu.out"
printf '2\n6\n1\nq\nq\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_CLEAN_ORPHANS_OUTPUT" 2>&1
grep -Fq 'paquetes huérfanos' "$RUST_CLEAN_ORPHANS_OUTPUT" || die 'el submenú Rust clean no ejecutó huérfanos'
ok 'submenú Rust clean ejecuta revisión de huérfanos sin confirmar borrado'

RUST_CLEAN_CACHE_OUTPUT="$TMP_DIR/rust-clean-cache-menu.out"
printf '2\n6\n2\nq\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_CLEAN_CACHE_OUTPUT" 2>&1
grep -Fq 'cachés' "$RUST_CLEAN_CACHE_OUTPUT" || die 'el submenú Rust clean no ejecutó cachés'
ok 'submenú Rust clean ejecuta revisión de cachés sin confirmar borrado'

RUST_CLEAN_FLATPAK_OUTPUT="$TMP_DIR/rust-clean-flatpak-menu.out"
printf '2\n6\n3\nn\nq\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$HOME/.local/state" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_CLEAN_FLATPAK_OUTPUT" 2>&1
grep -Fq 'Limpieza protegida' "$RUST_CLEAN_FLATPAK_OUTPUT" || die 'el submenú Rust clean no ejecutó Flatpak'
ok 'submenú Rust clean ofrece Flatpak sin confirmar borrado'

RUST_CLEAN_PACKAGE_OUTPUT="$TMP_DIR/rust-clean-package-menu.out"
printf '2\n6\n5\n%s\nn\nq\n' "$FIXTURE/example.deb" | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_CLEAN_PACKAGE_OUTPUT" 2>&1
grep -Fq 'Paquete:' "$RUST_CLEAN_PACKAGE_OUTPUT" || die 'el submenú Rust clean no pidió un paquete'
ok 'submenú Rust clean revisa y cancela un paquete'

RUST_PREFIX_OUTPUT="$TMP_DIR/rust-prefix-menu.out"
printf '1\n4\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_PREFIX_OUTPUT" 2>&1
grep -Fq 'ExamplePrefix' "$RUST_PREFIX_OUTPUT" || die 'el menú Rust no listó prefijos'
ok 'menú Rust lista prefijos Wine/Proton'

RUST_SYSTEM_OUTPUT="$TMP_DIR/rust-system-menu.out"
printf '3\n2\n1\nq\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_SYSTEM_OUTPUT" 2>&1
grep -Fq 'Servicios, procesos y journal' "$RUST_SYSTEM_OUTPUT" || die 'el menú Rust no abrió system'
grep -Fq 'Estado de systemd' "$RUST_SYSTEM_OUTPUT" || die 'el submenú Rust system no ejecutó status'
ok 'menú Rust ejecuta systemd desde su submenú'

for system_choice in 2 3 4 5; do
    RUST_SYSTEM_CHOICE_OUTPUT="$TMP_DIR/rust-system-${system_choice}-menu.out"
    printf '3\n2\n%s\nq\n' "$system_choice" | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
        LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_SYSTEM_CHOICE_OUTPUT" 2>&1
    grep -Fq 'Servicios, procesos y journal' "$RUST_SYSTEM_CHOICE_OUTPUT" || die "el submenú Rust system no mostró la opción $system_choice"
done
ok 'submenú Rust system recorre servicios de sistema/usuario, procesos y journal'

for system_choice in 7 8; do
    RUST_SYSTEM_BACK_OUTPUT="$TMP_DIR/rust-system-back-${system_choice}.out"
    printf '3\n2\n%s\nq\n' "$system_choice" | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
        LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_SYSTEM_BACK_OUTPUT" 2>&1
    grep -Fq 'Servicios, procesos y journal' "$RUST_SYSTEM_BACK_OUTPUT" || die "system no volvió desde la opción $system_choice"
done
ok 'menú system vuelve con Enter vacío en gestión y dependencias'

RUST_SYSTEM_INVALID_OUTPUT="$TMP_DIR/rust-system-invalid-menu.out"
printf '3\n2\nx\nq\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_SYSTEM_INVALID_OUTPUT" 2>&1
grep -Fq 'Opción no válida' "$RUST_SYSTEM_INVALID_OUTPUT" || die 'el submenú Rust system no gestionó una opción inválida'
ok 'submenú Rust system gestiona entrada inválida y salida'

RUST_PACKAGES_OUTPUT="$TMP_DIR/rust-packages-menu.out"
printf '1\n3\n%s\n' "$PKG_OUT/rust-menu" | timeout 60 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_PACKAGES_OUTPUT" 2>&1
grep -Fq 'Inventario de paquetes y almacenes' "$RUST_PACKAGES_OUTPUT" || die 'el menú Rust no abrió paquetes'
assert_file "$PKG_OUT/rust-menu/inventory.tsv"
ok 'menú Rust ejecuta inventario compacto de paquetes y almacenes'

PACKAGE_FULL_OUT="$TMP_DIR/packages-full"
run_bash packages --full --packages-only --out "$PACKAGE_FULL_OUT" >/dev/null
assert_file "$PACKAGE_FULL_OUT/inventory.tsv"
assert_file "$PACKAGE_FULL_OUT/package-managers.tsv"
assert_file "$PACKAGE_FULL_OUT/package-artifacts.tsv"
ok 'inventario detallado opcional conserva los TSV separados'

RUST_HELP_OUTPUT="$TMP_DIR/rust-help-menu.out"
printf 'h\n' | timeout 30 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_HELP_OUTPUT" 2>&1
grep -Fq 'Comandos:' "$RUST_HELP_OUTPUT" || die 'el menú Rust no ejecutó ayuda'
ok 'menú Rust ejecuta ayuda'

RUST_AUDIT_OUTPUT="$TMP_DIR/rust-audit-menu.out"
printf '1\n1\nn\ny\n%s\n%s\n' "$FIXTURE" "$AUDIT_OUT/rust-menu" | timeout 60 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
    LTOOLS_NO_MOUNTS=1 "$BIN" menu >"$RUST_AUDIT_OUTPUT" 2>&1
grep -Fq 'Fase 7/7' "$RUST_AUDIT_OUTPUT" || die 'el menú Rust no avanzó por la auditoría y duplicados'
assert_file "$AUDIT_OUT/rust-menu/summary.txt"
assert_file "$AUDIT_OUT/rust-menu/duplicates.tsv"
ok 'menú Rust ejecuta la opción auditoría y genera el informe'

if [[ -n "$APPIMAGE_PATH" ]]; then
    RUST_APPIMAGE_OUTPUT="$TMP_DIR/rust-appimage-menu.out"
    printf '1\n1\nn\nn\n%s\n%s\n' "$FIXTURE" "$AUDIT_OUT/rust-appimage-menu" | timeout 60 env HOME="$HOME" XDG_STATE_HOME="$XDG_STATE_HOME" \
        LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 APPIMAGE_EXTRACT_AND_RUN=1 \
        "$APPIMAGE_PATH" --rust menu >"$RUST_APPIMAGE_OUTPUT" 2>&1
    grep -Fq "Rust $VERSION" "$RUST_APPIMAGE_OUTPUT" || die 'la AppImage no arrancó el menú Rust'
    grep -Fq 'Fase 6/6' "$RUST_APPIMAGE_OUTPUT" || die 'la AppImage no avanzó por la auditoría Rust'
    assert_file "$AUDIT_OUT/rust-appimage-menu/summary.txt"
    ok 'AppImage ejecuta menú Rust, opción auditoría e informe'
fi

run_ctrl_c() {
    local name="$1" output="$TMP_DIR/$1-ctrl-c.out" fifo="$TMP_DIR/$1-ctrl-c.fifo" pid ready=0 status
    local -a command=()
    if [[ -n "$APPIMAGE_PATH" ]]; then
        command=(env APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" menu)
    else
        command=("$BIN" menu)
    fi
    mkfifo "$fifo"
    setsid env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_DATA_HOME="$XDG_DATA_HOME" \
        XDG_STATE_HOME="$XDG_STATE_HOME" LTOOLS_NO_MOUNTS=1 LTOOLS_NO_AUTO_TERMINAL=1 \
        "${command[@]}" <"$fifo" >"$output" 2>&1 &
    pid=$!
    exec 9>"$fifo"
    for _ in {1..100}; do
        if grep -Fq 'Elige una opción' "$output" 2>/dev/null; then
            ready=1
            break
        fi
        if ! kill -0 "$pid" 2>/dev/null; then
            break
        fi
        sleep 0.1
    done
    if [[ "$ready" -ne 1 ]]; then
        kill "$pid" 2>/dev/null || true
        exec 9>&-
        rm -f "$fifo"
        sed -n '1,120p' "$output" >&2
        die "$name no mostró el menú antes de Ctrl+C"
    fi
    kill -INT -- "-$pid" 2>/dev/null || kill -INT "$pid" 2>/dev/null || true
    exited=0
    for _ in {1..50}; do
        if ! kill -0 "$pid" 2>/dev/null; then
            exited=1
            break
        fi
        sleep 0.1
    done
    if [[ "$exited" -ne 1 ]]; then
        kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
        exec 9>&-
        rm -f "$fifo"
        sed -n '1,120p' "$output" >&2
        die "$name no terminó en 5 segundos tras Ctrl+C"
    fi
    set +e
    wait "$pid"
    status=$?
    set -e
    exec 9>&-
    rm -f "$fifo"
    [[ "$status" -eq 0 ]] || { sed -n '1,120p' "$output" >&2; die "$name terminó con código $status tras Ctrl+C"; }
    grep -Fq 'Interrupción recibida' "$output" || { sed -n '1,120p' "$output" >&2; die "$name no informó una salida limpia tras Ctrl+C"; }
    ! grep -Eiq 'panic|se cerró inesperadamente|aborted' "$output" || die "$name mostró un cierre tipo crash tras Ctrl+C"
    ok "$name sale limpiamente con Ctrl+C"
}

printf 'E2E: comprobando Ctrl+C sin panic ni cierre inesperado...\n'
run_ctrl_c ctrl-c-backend
if [[ -n "$APPIMAGE_PATH" ]]; then
    run_ctrl_c ctrl-c-appimage
fi

printf 'E2E de menús y funciones completado correctamente.\n'
