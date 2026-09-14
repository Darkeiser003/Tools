#!/usr/bin/env bash
# Smoke tests seguros: no recorren los discos reales ni modifican la cuenta.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/rust/Cargo.toml" | head -n1)"
APPIMAGE_PATH=""
RUNNER_PATH=""
LOG_PATH=""
KEEP_TEMP=0
REQUIRE_FUSE=0
REQUIRE_GUI=0
APPIMAGE_FUSE_WORKS=0

die() { printf 'SMOKE ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }
skip() { printf '  SKIP  %s\n' "$1"; }
usage() { printf 'Uso: %s [--binary RUTA] [--appimage RUTA] [--runner RUTA] [--log RUTA] [--require-fuse] [--require-gui] [--keep-temp]\n' "$0"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        --appimage) (($# >= 2)) || die '--appimage necesita una ruta'; APPIMAGE_PATH="$2"; shift ;;
        --runner) (($# >= 2)) || die '--runner necesita una ruta'; RUNNER_PATH="$2"; shift ;;
        --log) (($# >= 2)) || die '--log necesita una ruta'; LOG_PATH="$2"; shift ;;
        --require-fuse) REQUIRE_FUSE=1 ;;
        --require-gui) REQUIRE_GUI=1 ;;
        --keep-temp) KEEP_TEMP=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

if (( REQUIRE_GUI )); then
    for command_name in timeout xvfb-run xdotool import identify file; do
        command -v "$command_name" >/dev/null 2>&1 || die "--require-gui exige «$command_name»"
    done
    timeout 10 xvfb-run -a -s "-screen 0 1280x900x24" true >/dev/null 2>&1 ||
        die '--require-gui no pudo iniciar un display Xvfb'
fi

[[ -x "$BIN" ]] || die "no existe el binario ejecutable: $BIN"
[[ -f "$ROOT_DIR/ltools-cli.sh" ]] || die 'no existe el lanzador CLI Linux'
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cachyos-smoke.XXXXXX")"
cleanup_smoke_temp() {
    local status=$?
    if (( KEEP_TEMP != 0 || status != 0 )); then
        if (( status != 0 && KEEP_TEMP == 0 )); then
            printf 'Smoke fallido; temporales conservados para diagnóstico: %s\n' "$TMP_DIR" >&2
        fi
    else
        rm -rf -- "$TMP_DIR"
    fi
}
if [[ "$KEEP_TEMP" -ne 0 ]]; then
    printf 'Temporales conservados en: %s\n' "$TMP_DIR"
fi
trap cleanup_smoke_temp EXIT
# Toda la batería debe ser hermética: los dry-run de acciones mutables crean
# una frontera de transacción aunque no ejecuten el comando. Si se hereda el
# XDG_STATE_HOME del host, una política de solo lectura del entorno puede
# hacer fallar el smoke antes de probar LTools.
export HOME="$TMP_DIR/default-home"
export XDG_CONFIG_HOME="$TMP_DIR/default-config"
export XDG_DATA_HOME="$TMP_DIR/default-data"
export XDG_STATE_HOME="$TMP_DIR/default-state"
export LTOOLS_LANG=es
mkdir -p -- "$HOME" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_STATE_HOME"

printf 'Smoke tests de LTools\n'
while IFS= read -r -d '' file; do
    bash -n "$file"
done < <(find "$ROOT_DIR" -maxdepth 4 -type f -name '*.sh' -print0)
ok 'sintaxis de todos los scripts Bash'

"$BIN" --version >/dev/null
ok 'backend Rust responde a --version'
HELP_OUTPUT="$("$BIN" --help)"
grep -Fq 'doctor --install TOOL' <<<"$HELP_OUTPUT" || die 'la ayuda no documenta la instalación explícita'
ok 'backend Rust responde a --help'
GUIDE_GIT_OUTPUT="$("$BIN" guide git)" || die 'la guía de Git no responde'
grep -Fq 'GIT Y GITHUB' <<<"$GUIDE_GIT_OUTPUT" || die 'la guía de Git no documenta GitHub'
grep -Fq 'clone' <<<"$GUIDE_GIT_OUTPUT" || die 'la guía de Git no documenta clone'
grep -Fq 'push' <<<"$GUIDE_GIT_OUTPUT" || die 'la guía de Git no documenta push'
grep -Fq -- '--dry-run' <<<"$GUIDE_GIT_OUTPUT" || die 'la guía de Git no documenta dry-run'
GUIDE_GUI_STORAGE_OUTPUT="$("$BIN" guide gui storage)"
grep -Fq 'GUÍA GRÁFICA' <<<"$GUIDE_GUI_STORAGE_OUTPUT" || die 'la guía gráfica no se identifica como GUI'
grep -Fq 'Proceso complejo' <<<"$GUIDE_GUI_STORAGE_OUTPUT" || die 'la guía gráfica no explica un proceso complejo'
! grep -Fq 'parted' <<<"$GUIDE_GUI_STORAGE_OUTPUT" || die 'la guía gráfica expuso comandos internos'
for internal_option in wipefs mdadm --depth pkexec; do
    ! grep -Fq -- "$internal_option" <<<"$GUIDE_GUI_STORAGE_OUTPUT" ||
        die "la guía gráfica expuso la opción interna $internal_option"
done
GUIDE_ALL_OUTPUT="$("$BIN" guide all)" || die 'el índice de guías no responde'
for guide_topic in network boot services storage wine containers kubernetes actions; do
    grep -Fq "$guide_topic" <<<"$GUIDE_ALL_OUTPUT" || die "el índice de guías no incluye $guide_topic"
done
GUIDE_NETWORK_OUTPUT="$("$BIN" guide network)"
grep -Fq 'set-interface' <<<"$GUIDE_NETWORK_OUTPUT" || die 'la guía CLI de red no explica la opción de interfaz'
grep -Fq -- '--interface' <<<"$GUIDE_NETWORK_OUTPUT" || die 'la guía CLI de red no explica el argumento de interfaz'
GUIDE_BOOT_OUTPUT="$("$BIN" guide boot)"
grep -Fq 'efi-entries' <<<"$GUIDE_BOOT_OUTPUT" || die 'la guía CLI de arranque no explica EFI'
grep -Fq 'set-next' <<<"$GUIDE_BOOT_OUTPUT" || die 'la guía CLI de arranque no explica GRUB'
GUIDE_SERVICES_OUTPUT="$("$BIN" guide services)"
grep -Fq -- '--scope system|user|both' <<<"$GUIDE_SERVICES_OUTPUT" || die 'la guía de servicios no explica sus scopes'
grep -Fq 'enable' <<<"$GUIDE_SERVICES_OUTPUT" || die 'la guía de servicios no explica activar servicios'
ok 'guías CLI por familia y Git/GitHub avanzado'
CLI_HELP_OUTPUT="$(LTOOLS_CLI=1 "$BIN")"
grep -Fq 'Uso: ltools' <<<"$CLI_HELP_OUTPUT" || die 'el perfil CLI sin argumentos no mostró la ayuda'
ok 'perfil CLI sin argumentos muestra ayuda sin abrir menú'
CLI_WRAPPER_OUTPUT="$("$ROOT_DIR/ltools-cli.sh")"
grep -Fq 'Uso: ltools' <<<"$CLI_WRAPPER_OUTPUT" || die 'ltools-cli.sh sin argumentos no mostró la ayuda'
ok 'lanzador CLI Linux conserva el modo sin argumentos'

# El gestor de alias debe ser utilizable sin tocar la configuración real del
# usuario y debe conservar los argumentos como valores separados.
ALIAS_HOME="$TMP_DIR/alias-config"
ALIAS_BIN="$TMP_DIR/alias-bin"
ALIAS_ENSURE_OUTPUT="$(LTOOLS_ALIAS_HOME="$ALIAS_HOME" LTOOLS_ALIAS_BIN="$ALIAS_BIN" \
    LTOOLS_ALIAS_EXECUTABLE="$BIN" "$BIN" aliases ensure)"
grep -Fq 'Alias predeterminados listos:' <<<"$ALIAS_ENSURE_OUTPUT" || die 'aliases ensure no creó el registro'
grep -Fq 'Lanzador creado:' <<<"$ALIAS_ENSURE_OUTPUT" || die 'aliases ensure no creó el lanzador'
[[ -x "$ALIAS_BIN/ltools" ]] || die 'el lanzador Linux gestionado no es ejecutable'
ALIAS_LIST_OUTPUT="$(LTOOLS_ALIAS_HOME="$ALIAS_HOME" "$BIN" aliases list)"
grep -Fq 'tnet -> native network' <<<"$ALIAS_LIST_OUTPUT" || die 'el registro no contiene el alias nativo de red'
LTOOLS_ALIAS_HOME="$ALIAS_HOME" "$BIN" aliases add tguide guide gui storage >/dev/null
ALIAS_GUIDE_OUTPUT="$(LTOOLS_ALIAS_HOME="$ALIAS_HOME" "$BIN" tguide)"
grep -Fq 'GUÍA GRÁFICA: ALMACENAMIENTO' <<<"$ALIAS_GUIDE_OUTPUT" || die 'el alias personalizado no conservó sus argumentos'
LTOOLS_ALIAS_HOME="$ALIAS_HOME" "$BIN" aliases disable tguide >/dev/null
set +e
DISABLED_ALIAS_STATUS=0
LTOOLS_ALIAS_HOME="$ALIAS_HOME" "$BIN" tguide >"$TMP_DIR/disabled-alias.log" 2>&1 || DISABLED_ALIAS_STATUS=$?
set -e
(( DISABLED_ALIAS_STATUS != 0 )) || die 'un alias desactivado se ejecutó inesperadamente'
grep -Fq 'comando desconocido: tguide' "$TMP_DIR/disabled-alias.log" || die 'un alias desactivado no bloqueó la expansión'
ok 'gestor de alias Linux: creación, expansión, desactivación y lanzador'

MAP_OUTPUT="$(XDG_STATE_HOME="$TMP_DIR/map-state" "$BIN" storage map --path "$TMP_DIR" --depth 1 --max-children 3)"
grep -Fq 'MAPA DE DISCOS' <<<"$MAP_OUTPUT" || die 'el mapa de almacenamiento no mostró su cabecera'
grep -Fq 'Contenido = tamaño accesible acumulado' <<<"$MAP_OUTPUT" || die 'el mapa no documentó el tamaño acumulado'
grep -Fq 'total=' <<<"$MAP_OUTPUT" || die 'el mapa no mostró capacidad total, ocupado y libre'
MAP_JSON="$(XDG_STATE_HOME="$TMP_DIR/map-state" "$BIN" storage map --path "$TMP_DIR" --depth 0 --format json)"
grep -Fq 'ltools-storage-map-v1' <<<"$MAP_JSON" || die 'el mapa JSON no declaró su esquema'
grep -Fq 'filesystem_total' <<<"$MAP_JSON" || die 'el mapa JSON no mostró capacidad total del sistema de archivos'
grep -Fq 'filesystem_free' <<<"$MAP_JSON" || die 'el mapa JSON no mostró espacio libre total'
grep -Fq 'filesystem_available' <<<"$MAP_JSON" || die 'el mapa JSON no mostró espacio disponible para la cuenta'
EXPLAIN_OUTPUT="$(XDG_STATE_HOME="$TMP_DIR/map-state" "$BIN" storage explain --path "$TMP_DIR")"
grep -Fq 'Permisos máximos del proceso:' <<<"$EXPLAIN_OUTPUT" || die 'storage explain no mostró el contexto de permisos'
ok 'mapa Linux: árbol, tamaños acumulados, JSON y explicación de permisos'

MENU_OUTPUT="$(printf 'q\n' | HOME="$TMP_DIR/menu-home" XDG_STATE_HOME="$TMP_DIR/menu-state" "$BIN" menu 2>&1)"
for marker in 'Auditar / Inventariar' 'Dependencias' 'Herramientas nativas' 'Herramientas instalables' 'Automatización' 'Rutas predeterminadas'; do
    grep -Fq -- "$marker" <<<"$MENU_OUTPUT" || die "el menú principal no muestra la categoría: $marker"
done
! grep -Fq -- 'WinSlim' <<<"$MENU_OUTPUT" || die 'la build Linux mostró la categoría exclusiva de WinSlim'
ok 'menú principal Linux con categorías generales y sin WinSlim'
if command -v xvfb-run >/dev/null 2>&1; then
    if timeout 10 xvfb-run -a -s "-screen 0 1280x900x24" true >/dev/null 2>&1; then
        set +e
        GUI_OUTPUT="$(timeout 30 xvfb-run -a -s "-screen 0 1280x900x24" env GDK_BACKEND=x11 \
            LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0 \
            LTOOLS_TERMINAL=auto \
            HOME="$TMP_DIR/gui-home" XDG_STATE_HOME="$TMP_DIR/gui-state" \
            "$BIN" 2>&1)"
        GUI_STATUS=$?
        set -e
        if [[ "$GUI_STATUS" -ne 0 ]]; then
            printf 'Salida de la GUI Rust (código %s):\n%s\n' "$GUI_STATUS" "$GUI_OUTPUT" >&2
            die 'la GUI Rust no pudo abrir y cerrar una ventana de prueba'
        fi
        ok 'GUI Rust Linux abre y cierra una ventana aislada'

        # Auditoría estructural y capturas: el E2E visita páginas reales de la
        # GUI y conserva evidencia visual de sus títulos, opciones y campos.
        # El marcador lo escribe la propia construcción GTK, por lo que aquí
        # también se detectan botones que no llevan comando, argumento o
        # navegación asociada. Las capturas se dejan en dist/captures para
        # poder revisarlas después del build.
        GUI_CAPTURE_DIR="${LTOOLS_GUI_CAPTURE_DIR:-$ROOT_DIR/dist/captures}"
        GUI_AUDIT_MARKER="$TMP_DIR/gui-audit.marker"
        mkdir -p -- "$GUI_CAPTURE_DIR"
        rm -f -- "$GUI_AUDIT_MARKER"

        capture_gui_page() {
            local page capture marker capture_log window_log scroll_capture
            page="$1"
            capture="$2"
            marker="${3:-}"
            scroll_capture="${4:-}"
            capture_log="$TMP_DIR/gui-capture-${page}.log"
            window_log="$TMP_DIR/gui-window-${page}.log"
            timeout 30 xvfb-run -a -s "-screen 0 1280x900x24" bash -c '
                set -Eeuo pipefail
                export GDK_BACKEND=x11
                unset WAYLAND_DISPLAY WAYLAND_SOCKET
                export GTK_USE_PORTAL=0
                binary="$1"
                page="$2"
                capture="$3"
                marker="$4"
                window_log="$7"
                scroll_capture="$8"
                export LTOOLS_NO_MOUNTS=1
                export LTOOLS_GUI_SMOKE=1
                export LTOOLS_GUI_REQUIRED=1
                export LTOOLS_DISABLE_GUI=0
                if [[ -n "$scroll_capture" ]]; then
                    export LTOOLS_GUI_SMOKE_HOLD_MS=6000
                else
                    export LTOOLS_GUI_SMOKE_HOLD_MS=1800
                fi
                export LTOOLS_GUI_SMOKE_NAV_PAGE="$page"
                export HOME="$5"
                export XDG_STATE_HOME="$6"
                if [[ -n "$marker" ]]; then
                    export LTOOLS_GUI_AUDIT_MARKER="$marker"
                else
                    unset LTOOLS_GUI_AUDIT_MARKER
                fi
                "$binary" >"$window_log" 2>&1 &
                pid=$!
                window_id=""
                for _ in {1..24}; do
                    window_id="$(xdotool search --onlyvisible --name 'LTools' 2>/dev/null | head -n1 || true)"
                    if [[ -n "$window_id" ]]; then
                        break
                    fi
                    sleep 0.15
                done
                [[ -n "$window_id" ]]
                # Espera a que GTK haya pintado los controles, no solo a que
                # exista la ventana. Así se evita guardar un root negro por
                # capturar durante el primer ciclo del compositor.
                sleep 0.6
                import -window "$window_id" "$capture" >/dev/null 2>&1 ||
                    import -window root "$capture" >/dev/null 2>&1
                if [[ -n "$scroll_capture" ]]; then
                    # Captura también las opciones que quedan fuera del primer
                    # viewport y confirma que los menús largos se desplazan.
                    xdotool mousemove --window "$window_id" 700 560 \
                        click --repeat 50 --delay 80 5
                    sleep 0.3
                    import -window "$window_id" "$scroll_capture" >/dev/null 2>&1 ||
                        import -window root "$scroll_capture" >/dev/null 2>&1
                fi
                wait "$pid"
                [[ -s "$capture" ]]
            ' _ "$BIN" "$page" "$capture" "$marker" \
                "$TMP_DIR/gui-page-${page}-home" "$TMP_DIR/gui-page-${page}-state" \
                "$window_log" "$scroll_capture" \
                >"$capture_log" 2>&1
        }

        capture_gui_page 8 "$GUI_CAPTURE_DIR/linux-git-github.png" "$GUI_AUDIT_MARKER" \
            "$GUI_CAPTURE_DIR/linux-git-github-bottom.png" || {
            cat "$TMP_DIR/gui-capture-8.log" >&2 || true
            die 'la captura/auditoría de la página Git y GitHub falló'
        }
        capture_gui_page 7 "$GUI_CAPTURE_DIR/linux-settings.png" "" \
            "$GUI_CAPTURE_DIR/linux-settings-bottom.png" || die 'la captura de Ajustes falló'
        capture_gui_page 6 "$GUI_CAPTURE_DIR/linux-software-packages.png" || die 'la captura del menú de paquetes falló'
        capture_gui_page 10 "$GUI_CAPTURE_DIR/linux-storage-menu.png" "" \
            "$GUI_CAPTURE_DIR/linux-storage-menu-bottom.png" || die 'la captura del menú de almacenamiento falló'
        capture_gui_page 21 "$GUI_CAPTURE_DIR/linux-storage-partitions.png" "" \
            "$GUI_CAPTURE_DIR/linux-storage-partitions-bottom.png" || die 'la captura de particionado falló'
        capture_gui_page 22 "$GUI_CAPTURE_DIR/linux-storage-filesystems.png" "" \
            "$GUI_CAPTURE_DIR/linux-storage-filesystems-bottom.png" || die 'la captura de sistemas de archivos falló'
        capture_gui_page 23 "$GUI_CAPTURE_DIR/linux-storage-volumes.png" "" \
            "$GUI_CAPTURE_DIR/linux-storage-volumes-bottom.png" || die 'la captura de volúmenes falló'
        capture_gui_page 27 "$GUI_CAPTURE_DIR/linux-network.png" "" \
            "$GUI_CAPTURE_DIR/linux-network-bottom.png" || die 'la captura de red falló'
        capture_gui_page 28 "$GUI_CAPTURE_DIR/linux-boot-efi.png" "" \
            "$GUI_CAPTURE_DIR/linux-boot-efi-bottom.png" || die 'la captura de arranque EFI falló'
        capture_gui_page 29 "$GUI_CAPTURE_DIR/linux-services.png" "" \
            "$GUI_CAPTURE_DIR/linux-services-bottom.png" || die 'la captura de servicios falló'
        capture_gui_page 30 "$GUI_CAPTURE_DIR/linux-wine-proton.png" || die 'la captura de Wine/Proton falló'

        # Abre el diálogo real de gh, rellena campos representativos y lo
        # cancela automáticamente: se auditan título, argumentos y captura
        # sin ejecutar ninguna petición contra GitHub.
        GH_DIALOG_CAPTURE="$GUI_CAPTURE_DIR/linux-git-gh-native-dialog.png"
        timeout 12 xvfb-run -a -s "-screen 0 1280x900x24" bash -c '
            set -Eeuo pipefail
            export GDK_BACKEND=x11 GTK_USE_PORTAL=0 LTOOLS_NO_MOUNTS=1
            export LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0
            export LTOOLS_GUI_SMOKE_HOLD_MS=4000 LTOOLS_GUI_SMOKE_NAV_PAGE=8
            export LTOOLS_GUI_GH_DIALOG_SMOKE=1 LTOOLS_LANG=es
            export HOME="$4" XDG_STATE_HOME="$5" LTOOLS_GUI_AUDIT_MARKER="$3"
            "$1" >"$6" 2>&1 &
            pid=$!
            trap "kill \"$pid\" 2>/dev/null || true" EXIT
            dialog_id=""
            for _ in {1..30}; do
                dialog_id="$(xdotool search --onlyvisible --name "Comando nativo de GitHub CLI" 2>/dev/null | tail -n1 || true)"
                [[ -n "$dialog_id" ]] && break
                sleep 0.1
            done
            [[ -n "$dialog_id" ]]
            sleep 0.2
            import -window "$dialog_id" "$2" >/dev/null 2>&1
            wait "$pid"
            trap - EXIT
        ' _ "$BIN" "$GH_DIALOG_CAPTURE" "$GUI_AUDIT_MARKER" \
            "$TMP_DIR/gui-gh-dialog-home" "$TMP_DIR/gui-gh-dialog-state" \
            "$TMP_DIR/gui-gh-dialog.log" || {
            cat "$TMP_DIR/gui-gh-dialog.log" >&2 || true
            die 'no se pudo abrir/capturar de forma segura el diálogo nativo de gh'
        }
        grep -Fq $'DIALOG\tComando nativo de GitHub CLI (gh)\t--command!' "$GUI_AUDIT_MARKER" ||
            die 'el diálogo gh no expone el campo de comando obligatorio en el título esperado'
        grep -Fq $'DIALOG\tComando nativo de GitHub CLI (gh)\t--command!=Comando gh' "$GUI_AUDIT_MARKER" ||
            die 'la ayuda del campo de comando gh no coincide con la GUI'
        grep -Fq $'DIALOG_INITIAL_VALUES\tgh-native\tissue\tlist --state open' "$GUI_AUDIT_MARKER" ||
            die 'el E2E no rellenó la muestra de comando y argumentos del diálogo gh'
        grep -Fq 'DIALOG_SHOWN	gh-native' "$GUI_AUDIT_MARKER" || die 'no se registró la apertura del diálogo gh'
        grep -Fq 'DIALOG_CANCELLED	gh-native' "$GUI_AUDIT_MARKER" || die 'el E2E no canceló el diálogo gh sin ejecutarlo'
        ok 'diálogo nativo de gh: título, campos, captura y cancelación sin efectos remotos'

        TREE_CAPTURE="$GUI_CAPTURE_DIR/linux-storage-map-dialog.png"
        TREE_MARKER="$TMP_DIR/gui-tree.marker"
        TREE_FIXTURE="$TMP_DIR/gui-tree-fixture"
        mkdir -p "$TREE_FIXTURE/Carpeta de prueba"
        mkdir -p "$TMP_DIR/gui-tree-home" "$TMP_DIR/gui-tree-state"
        printf 'contenido visible\n' >"$TREE_FIXTURE/Carpeta de prueba/archivo.txt"
        timeout 45 xvfb-run -a -s "-screen 0 1280x900x24" bash -c '
            set -Eeuo pipefail
            export GDK_BACKEND=x11 GTK_USE_PORTAL=0
            unset WAYLAND_DISPLAY WAYLAND_SOCKET
            export LTOOLS_NO_MOUNTS=1 LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0
            export LTOOLS_GUI_TREE_SMOKE=1 LTOOLS_GUI_TREE_PATH="$1" LTOOLS_GUI_TREE_MARKER="$2"
            export LTOOLS_LANG=es HOME="$3" XDG_STATE_HOME="$4"
            "$5" >"$6" 2>&1 &
            pid=$!
            window_id=""
            for _ in {1..100}; do
                window_id="$(xdotool search --onlyvisible --name "Mapa interactivo" 2>/dev/null | head -n1 || true)"
                [[ -n "$window_id" ]] && break
                sleep 0.1
            done
            if [[ -z "$window_id" ]]; then
                printf "No apareció el mapa interactivo. Ventanas visibles:\n" >&2
                xdotool search --onlyvisible --name . 2>/dev/null | while read -r visible_window; do
                    printf "%s %s\n" "$visible_window" \
                        "$(xdotool getwindowname "$visible_window" 2>/dev/null || true)" >&2
                done
                cat "$6" >&2 || true
                exit 1
            fi
            sleep 0.2
            # Verifica el árbol con menos ancho que el predeterminado para
            # detectar regresiones de columnas que oculten el espacio libre.
            xdotool windowsize --sync "$window_id" 800 640
            sleep 0.3
            geometry="$(xdotool getwindowgeometry --shell "$window_id")"
            window_top="$(sed -n "s/^Y=//p" <<<"$geometry")"
            window_height="$(sed -n "s/^HEIGHT=//p" <<<"$geometry")"
            [[ "$window_top" =~ ^-?[0-9]+$ && "$window_height" =~ ^[0-9]+$ ]]
            ((window_top >= 0 && window_top + window_height <= 900)) || {
                printf "El diálogo del mapa se sale de la pantalla Xvfb: Y=%s HEIGHT=%s\n" \
                    "$window_top" "$window_height" >&2
                exit 1
            }
            printf "MAP_WINDOW_FITS_SCREEN y=%s height=%s screen=900\n" \
                "$window_top" "$window_height"
            import -window "$window_id" "$7" >/dev/null 2>&1 ||
                import -window root "$7" >/dev/null 2>&1
            wait "$pid"
            [[ -s "$2" && -s "$7" ]]
        ' _ "$TREE_FIXTURE" "$TREE_MARKER" "$TMP_DIR/gui-tree-home" \
            "$TMP_DIR/gui-tree-state" "$BIN" "$TMP_DIR/gui-tree.log" "$TREE_CAPTURE" \
            >"$TMP_DIR/gui-tree-capture.log" 2>&1 || {
            cat "$TMP_DIR/gui-tree-capture.log" "$TMP_DIR/gui-tree.log" >&2 || true
            die 'la E2E no pudo abrir/capturar/cerrar el mapa interactivo real'
        }
        grep -Fq 'tree-opened' "$TREE_MARKER" || die 'la E2E no confirmó que se abriera el mapa interactivo'
        bash "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" \
            --binary "$BIN" --tmp "$TMP_DIR" --captures "$GUI_CAPTURE_DIR" \
            >"$TMP_DIR/storage-map-gui-actions.log" 2>&1 || {
            cat "$TMP_DIR/storage-map-gui-actions.log" >&2 || true
            die 'falló la E2E de acciones reales del mapa GUI'
        }
        cat "$TMP_DIR/storage-map-gui-actions.log"

        GUI_CAPTURES=(
            "$GUI_CAPTURE_DIR/linux-git-github.png"
            "$GUI_CAPTURE_DIR/linux-git-github-bottom.png"
            "$GUI_CAPTURE_DIR/linux-settings.png"
            "$GUI_CAPTURE_DIR/linux-settings-bottom.png"
            "$GUI_CAPTURE_DIR/linux-storage-menu.png"
            "$GUI_CAPTURE_DIR/linux-storage-menu-bottom.png"
            "$GUI_CAPTURE_DIR/linux-storage-partitions.png"
            "$GUI_CAPTURE_DIR/linux-storage-filesystems.png"
            "$GUI_CAPTURE_DIR/linux-storage-volumes.png"
            "$GUI_CAPTURE_DIR/linux-network.png"
            "$GUI_CAPTURE_DIR/linux-network-bottom.png"
            "$GUI_CAPTURE_DIR/linux-boot-efi.png"
            "$GUI_CAPTURE_DIR/linux-boot-efi-bottom.png"
            "$GUI_CAPTURE_DIR/linux-services.png"
            "$GUI_CAPTURE_DIR/linux-services-bottom.png"
            "$GUI_CAPTURE_DIR/linux-wine-proton.png"
            "$GH_DIALOG_CAPTURE"
            "$TREE_CAPTURE"
            "$GUI_CAPTURE_DIR/linux-storage-map-confirm-no-es.png"
            "$GUI_CAPTURE_DIR/linux-storage-map-confirm-yes-es.png"
        )
        for capture in "${GUI_CAPTURES[@]}"; do
            [[ -s "$capture" ]] || die "captura GUI vacía: $capture"
            if command -v identify >/dev/null 2>&1; then
                dimensions="$(identify -format '%w %h' "$capture" 2>/dev/null || true)"
                read -r width height <<<"$dimensions"
                [[ "${width:-0}" -ge 800 && "${height:-0}" -ge 600 ]] ||
                    die "captura GUI recortada o con dimensiones insuficientes: $capture (${dimensions:-desconocidas})"
                colors="$(identify -format '%k' "$capture" 2>/dev/null || echo 0)"
                [[ "$colors" -ge 20 ]] || die "captura GUI sin contenido visual suficiente: $capture ($colors colores)"
            else
                file_type="$(file -b --mime-type "$capture" 2>/dev/null || true)"
                [[ "$file_type" == image/png ]] || die "captura GUI no es PNG: $capture ($file_type)"
            fi
        done
        ok 'capturas GUI verificadas con dimensiones visibles y no solo existencia'

        [[ -s "$GUI_AUDIT_MARKER" ]] || die 'la GUI no produjo el marcador estructural'
        grep -Fq 'GUI_AUDIT_BEGIN' "$GUI_AUDIT_MARKER" || die 'la auditoría GUI no comenzó'
        grep -Fq 'GUI_AUDIT_END' "$GUI_AUDIT_MARKER" || die 'la auditoría GUI no terminó'
        for page_title in \
            $'PAGE\t8\tGit / GitHub' \
            $'PAGE\t10\tAlmacenamiento y particiones' \
            $'PAGE\t6\tSoftware, paquetes y almacenes' \
            $'PAGE\t21\tParticionado y tablas' \
            $'PAGE\t22\tSistemas de archivos' \
            $'PAGE\t23\tCifrado y volúmenes' \
            $'PAGE\t27\tRed, rutas, DNS y puertos escuchando' \
            $'PAGE\t28\tArranque, EFI y cargador del sistema' \
            $'PAGE\t29\tServicios del sistema' \
            $'PAGE\t30\tGestión de prefijos Wine y Proton'; do
            grep -Fq "$page_title" "$GUI_AUDIT_MARKER" ||
                die "la auditoría GUI no registró el título: ${page_title#*$'\t'}"
        done
        for expected_button in \
            $'BUTTON\tGuía completa de Git y GitHub (gh)\tCLI\tcommand=guide\targs=git' \
            $'BUTTON\tComando nativo de gh…\tGIT\toperation=gh-native' \
            $'BUTTON\tDiagnosticar repositorio\tGIT\toperation=diagnose' \
            $'BUTTON\tReconstruir índice .git…\tGIT\toperation=repair' \
            $'BUTTON\tRecuperar .git desde remoto…\tGIT\toperation=repair-remote' \
            $'BUTTON\tCrear tabla GPT\tSTORAGE\toperation=mklabel-gpt\tfields=--device!' \
            $'BUTTON\tMapa desplegable de discos y rutas\tCLI\tcommand=storage\targs=map --depth 4 --interactive-tree' \
            $'BUTTON\tExplicar ruta y permisos\tSTORAGE\toperation=manage-permissions\tfields=--path!' \
            $'BUTTON\tBorrar a la papelera\tSTORAGE\toperation=manage-delete\tfields=--path!' \
            $'BUTTON\tCopiar archivo o carpeta\tSTORAGE\toperation=manage-copy\tfields=--source!,--destination!' \
            $'BUTTON\tCrear archivo ZIP\tSTORAGE\toperation=manage-zip\tfields=--source!,--destination!' \
            $'BUTTON\tCrear / formatear sistema de archivos\tSTORAGE\toperation=mkfs' \
            $'BUTTON\tGestionar volúmenes LVM\tSTORAGE\toperation=lvm' \
            $'BUTTON\tGestionar conjuntos RAID\tSTORAGE\toperation=raid' \
            $'BUTTON\tActivar / desactivar interfaz\tNETWORK\taction=set-interface' \
            $'BUTTON\tProgramar siguiente entrada GRUB\tBOOT' \
            $'BUTTON\tAutomáticos y estáticos (system)\tCLI\tcommand=system\targs=services --scope system --filter automatic --limit 100' \
            $'BUTTON\tConceder permisos de administrador\tACCOUNT\taction=admin-add' \
            $'BUTTON\tVer grupo y miembros administradores\tACCOUNT\taction=admin-groups' \
            $'BUTTON\tCrear prefijo\tWINE'; do
            grep -Fq "$expected_button" "$GUI_AUDIT_MARKER" ||
                die "la auditoría GUI no registró la opción: ${expected_button#*$'\t'}"
        done
        for field in git_repo_placeholder git_url_placeholder git_destination_placeholder git_remote_message_placeholder git_notes_placeholder git_limit_placeholder; do
            grep -Fq $'FIELD\tgit\t'"$field"$'\t' "$GUI_AUDIT_MARKER" ||
                die "la auditoría GUI no registró el argumento/campo Git: $field"
        done
        duplicate_pages="$(awk -F '\t' '
            $1 == "GUI_AUDIT_BEGIN" { session++; next }
            $1 == "PAGE" {
                key = session SUBSEP $3
                if (key in page_id && page_id[key] != $2) print $3
                else page_id[key] = $2
            }
        ' "$GUI_AUDIT_MARKER" | sort -u)"
        [[ -z "$duplicate_pages" ]] || die "la GUI tiene títulos de página duplicados: $duplicate_pages"
        duplicate_categories="$(awk -F '\t' '
            $1 == "GUI_AUDIT_BEGIN" { session++; next }
            $1 == "BUTTON" && $3 == "CATEGORY" {
                key = session SUBSEP $2
                if (seen[key]++) print $2
            }
        ' "$GUI_AUDIT_MARKER" | sort -u)"
        [[ -z "$duplicate_categories" ]] || die "la GUI tiene categorías duplicadas: $duplicate_categories"

        # Contrato guía→menú: cada botón que abre una guía debe apuntar al
        # índice del mismo menú y ese índice debe contener todas las etiquetas
        # de botones que GTK construyó en esa página. Se normalizan saltos de
        # línea porque las guías largas ajustan visualmente algunos nombres.
        GUIDE_AUDIT_DIR="$TMP_DIR/guide-audit"
        mkdir -p -- "$GUIDE_AUDIT_DIR"
        while IFS= read -r guide_button; do
            guide_label="$(cut -f2 <<<"$guide_button")"
            guide_topic="$(sed -n 's/.*args=\([^[:space:]]*\).*/\1/p' <<<"$guide_button")"
            guide_page="$(sed -n 's/.*\tpage=\([0-9][0-9]*\)$/\1/p' <<<"$guide_button")"
            [[ -n "$guide_topic" && -n "$guide_page" ]] ||
                die "botón de guía sin tema o página: $guide_label"
            guide_file="$GUIDE_AUDIT_DIR/${guide_page}-${guide_topic}.out"
            "$BIN" guide gui "$guide_topic" >"$guide_file" 2>&1 ||
                die "no se pudo abrir la guía del botón: $guide_label"
            guide_flat="$(tr '\n' ' ' <"$guide_file" | sed -E 's/[[:space:]]+/ /g')"
            while IFS= read -r menu_label; do
                [[ -n "$menu_label" ]] || continue
                grep -Fq -- "$menu_label" <<<"$guide_flat" ||
                    die "la guía '$guide_topic' no enumera '$menu_label' del menú página $guide_page"
            done < <(awk -F '\t' -v page="$guide_page" '
                $1 == "BUTTON" {
                    found = 0
                    for (i = 4; i <= NF; i++) if ($i == "page=" page) found = 1
                    if (found && $2 != "Guía completa de Git y GitHub (gh)") print $2
                }
            ' "$GUI_AUDIT_MARKER")
        done < <(awk -F '\t' '$1 == "BUTTON" && $4 == "command=guide" { print }' "$GUI_AUDIT_MARKER")
        ok 'E2E compara cada índice de guía con todos los botones del menú real'
        ok 'E2E GUI abre menús, ayudas, campos y argumentos; auditoría sin duplicados'
        ok "capturas GUI con pantalla Xvfb 1280x900 y primera/última página de menús desplazados: $GUI_CAPTURE_DIR (paquetes, ajustes, almacenamiento, Git, red, EFI, servicios y Wine/Proton)"
else
    (( REQUIRE_GUI == 0 )) || die '--require-gui no pudo iniciar un display Xvfb para la GUI Rust'
    skip 'GUI Rust Linux: xvfb-run está instalado, pero Xvfb no puede crear un display aislado'
fi
else
    (( REQUIRE_GUI == 0 )) || die '--require-gui exige xvfb-run para la GUI Rust'
    skip 'GUI Rust Linux: xvfb-run no está disponible'
fi
CAPABILITIES_JSON="$("$BIN" capabilities --format json)"
grep -Fq '"schema": "ltools-capabilities-v1"' <<<"$CAPABILITIES_JSON" ||
    die 'el contrato JSON de capacidades no se pudo generar'
grep -Fq 'lterminal-startup-v1' <<<"$CAPABILITIES_JSON" ||
    die 'el contrato JSON no declara integración de terminal'
LEGACY_CAPABILITIES_JSON="$("$BIN" --ltools-capabilities --format json)"
grep -Fq '"schema": "ltools-capabilities-v1"' <<<"$LEGACY_CAPABILITIES_JSON" ||
    die 'el alias de capacidades usado por la integración no funciona'
ok 'alias de capacidades para AppRun y terminales anfitrionas'
if command -v jq >/dev/null 2>&1; then
    jq -e '(.host_tools | length >= 10) and any(.host_tools[]; .category == "audit") and any(.host_tools[]; .category == "system") and any(.host_tools[]; .category == "utilities") and any(.host_tools[]; .category == "development") and any(.host_tools[]; .id == "curl" and .installable == true) and any(.host_tools[]; .id == "docker-compose" and .installable == true) and any(.host_tools[]; .id == "kubectl" and .installable == true) and any(.host_tools[]; .id == "lsblk" and .installable == true) and ([.host_tools[] | select(.category == "games" or .category == "virtualization" or .command == "steam")] | length == 0)' \
        <<<"$CAPABILITIES_JSON" >/dev/null \
        || die 'el catálogo JSON de herramientas del anfitrión está incompleto'
    jq -e 'all(.host_tools[]; (.version | type == "string")) and any(.host_tools[]; .id == "parted" and .installable == true) and all(.host_tools[] | select(.id == "docker" or .id == "podman" or .id == "podman-compose" or .id == "containerd" or .id == "crictl" or .id == "kubectl" or .id == "helm" or .id == "kubeadm" or .id == "kubelet" or .id == "kind" or .id == "minikube" or .id == "k3d" or .id == "k9s"); .installable == true and (.install_package | length > 0))' \
        <<<"$CAPABILITIES_JSON" >/dev/null \
        || die 'el catálogo JSON no respeta versiones o no declara instaladores para las herramientas operativas'
fi
ok 'contrato JSON de capacidades e integración; catálogo nativo con instaladores operativos'

ACTIONS_JSON="$("$BIN" actions list --format json)"
if command -v jq >/dev/null 2>&1; then
    jq -e '.schema == "ltools-actions-v1" and .platform == "linux" and .safety.target_selection == "explicit-only" and (.actions | length >= 30) and any(.actions[]; .id == "storage.mount" and .targetPolicy == "explicit-only" and .mutating == true) and any(.actions[]; .id == "accounts.add" and .targetPolicy == "explicit-only" and .mutating == true and .confirmation != "none") and any(.actions[]; .id == "native.dns-flush" and .mutating == true and .confirmation != "none") and any(.actions[]; .id == "native.network-interface" and .mutating == true and .confirmation != "none") and any(.actions[]; .id == "wine.migrate" and .mutating == true and .confirmation != "none") and any(.actions[]; .id == "boot.status" and (.aliases | index("tboot status") != null) and .mutating == false) and all(.actions[]; (.command | IN("audit","packages","games","storage","system","accounts","native","defaults","clean","diagnostics","automation","boot","wine")))' \
        <<<"$ACTIONS_JSON" >/dev/null || die 'el registro de acciones Linux no declara políticas o acciones válidas'
else
    grep -Fq 'ltools-actions-v1' <<<"$ACTIONS_JSON" || die 'el registro de acciones Linux no se pudo generar'
    grep -Fq 'explicit-only' <<<"$ACTIONS_JSON" || die 'las acciones sensibles no exigen objetivo explícito'
fi
ACTION_DRY_RUN="$("$BIN" --dry-run actions run storage.mount /dev/synthetic-ltools)"
grep -Fq 'udisksctl mount' <<<"$ACTION_DRY_RUN" || grep -Fq 'mount /dev/synthetic-ltools' <<<"$ACTION_DRY_RUN" || die 'actions run no delegó el montaje con dry-run'
ACTION_NETWORK_DRY_RUN="$("$BIN" --dry-run actions run native.network-interface 'lo up')"
grep -Fq 'ip link set dev lo up' <<<"$ACTION_NETWORK_DRY_RUN" || die 'actions run no mapeó la interfaz de red'
ACTION_WINE_DRY_RUN="$("$BIN" --dry-run actions run wine.create "$TMP_DIR/action-prefix")"
grep -Fq 'no se ejecutaría wineboot' <<<"$ACTION_WINE_DRY_RUN" || die 'actions run no mapeó la creación de prefijos'
ok 'registro de acciones guiadas, política de objetivo explícito y dry-run'

NATIVE_TOOLS_OUTPUT="$("$BIN" native tools)"
for native_tool in ssh scp sftp adb docker kubectl; do
    grep -Fq "$native_tool" <<<"$NATIVE_TOOLS_OUTPUT" || die "native tools no enumeró $native_tool"
done
ok 'catálogo de SSH, ADB, Docker y Kubernetes en la consulta nativa'
bash "$ROOT_DIR/tests/linux/native-help-e2e.sh" --binary "$BIN" >"$TMP_DIR/native-help-e2e.out" 2>&1 || {
    sed -n '1,220p' "$TMP_DIR/native-help-e2e.out" >&2
    die 'la auditoría de ayudas nativas o la correspondencia GUI falló'
}
ok 'ayudas nativas reales y correspondencia de operaciones GUI'
UTILITIES_OUTPUT="$("$BIN" native utilities status)"
for utility in curl file openssl gpg; do
    grep -Fq "$utility" <<<"$UTILITIES_OUTPUT" || die "native utilities no enumeró $utility"
done
printf 'q\n' | "$BIN" native utilities menu >/dev/null || die 'native utilities menu no volvió con q'
ok 'catálogo y menú propio de utilidades del sistema'
NETWORK_INTERFACE_DRY_RUN="$("$BIN" --dry-run native network set-interface --interface lo --state up --yes)"
grep -Fq 'ip link set dev lo up' <<<"$NETWORK_INTERFACE_DRY_RUN" || die 'network set-interface no generó el plan esperado'
NETWORK_CONNECTION_DRY_RUN="$("$BIN" --dry-run native network connection-up --connection synthetic --yes)"
grep -Fq 'nmcli connection up synthetic' <<<"$NETWORK_CONNECTION_DRY_RUN" || die 'network connection-up no generó el plan esperado'
WINE_CREATE_DRY_RUN="$("$BIN" --dry-run prefix create --dest "$TMP_DIR/prefix-gui" --arch win64 --yes)"
grep -Fq 'no se ejecutaría wineboot' <<<"$WINE_CREATE_DRY_RUN" || die 'prefix create no respetó dry-run'
ok 'gestión de red y prefijos Wine/Proton con parámetros, confirmación y dry-run'
DEPENDENCIES_MENU_OUTPUT="$("$BIN" menu-dependencies <<< 'q')"
grep -Eqi 'Instalar una dependencia|Install a dependency' <<<"$DEPENDENCIES_MENU_OUTPUT" ||
    die 'el menú central de dependencias no ofrece instalación guiada'
if grep -Eqi 'Instalar una dependencia|Install a dependency' <(printf '%s\n' "$("$BIN" native tools menu <<< 'q')"); then
    die 'el menú operativo conserva un botón duplicado de instalación'
fi
ok 'dependencias centralizadas y menú operativo sin duplicados'

CONTAINERS_OUTPUT="$(timeout 30 "$BIN" native containers status)" || die 'native containers status falló'
grep -Fq 'solo lectura' <<<"$CONTAINERS_OUTPUT" || die 'containers status no declaró su política de solo lectura'
KUBERNETES_OUTPUT="$(timeout 30 "$BIN" native kubernetes status)" || die 'native kubernetes status falló'
grep -Fq 'No se aplican manifiestos' <<<"$KUBERNETES_OUTPUT" || die 'kubernetes status no declaró su política sin mutaciones'
ok 'flujos guiados de Docker/Podman y Kubernetes sin cambios'
for container_action in container-inspect container-stats container-top container-port container-diff container-pause container-unpause container-kill container-rename container-cp container-prune image-build image-tag image-remove image-prune volume-list volume-inspect volume-create volume-remove volume-prune network-list network-inspect network-create network-remove network-prune system-info system-df system-prune container-compose; do
    grep -Fq "\"$container_action\"" "$ROOT_DIR/rust/src/native/linux.rs" || die "falta la acción nativa $container_action"
done
grep -Fq 'compose up|down|start|stop|restart|ps|logs|pull|build|config|images|top|run|exec|rm|pause|unpause' "$ROOT_DIR/rust/src/native/mod.rs" || die 'ayuda nativa sin operaciones completas de Compose'
ok 'acciones Docker/Podman y Compose cubren ciclo de vida, recursos y mantenimiento'

PARTITION_GUIDE_OUTPUT="$("$BIN" storage guide)"
grep -Fq 'parted print' <<<"$PARTITION_GUIDE_OUTPUT" || die 'la guía Linux no documentó parted print'
grep -Fq 'Objetivos protegidos' <<<"$PARTITION_GUIDE_OUTPUT" || die 'la guía Linux no documentó objetivos protegidos'
PARTITION_DRY_RUN="$("$BIN" --dry-run storage partition-table /dev/synthetic-ltools)"
grep -Fq 'se consultarían lsblk, parted print, fdisk -l y sfdisk --dump' <<<"$PARTITION_DRY_RUN" || die 'partition-table dry-run no mostró el plan seguro'
ok 'flujo guiado de tabla de particiones Linux y dry-run sin acceso al dispositivo'

INSTALL_STUB_DIR="$TMP_DIR/install-stub"
INSTALL_OUTPUT="$TMP_DIR/install-output.log"
mkdir -p "$INSTALL_STUB_DIR" "$TMP_DIR/home-install" "$TMP_DIR/state-install"
cat > "$INSTALL_STUB_DIR/pacman" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$INSTALL_STUB_DIR/pacman"
set +e
printf 'n\n' | HOME="$TMP_DIR/home-install" XDG_STATE_HOME="$TMP_DIR/state-install" PATH="$INSTALL_STUB_DIR" "$BIN" doctor --install rsync >"$INSTALL_OUTPUT" 2>&1
INSTALL_STATUS=$?
set -e
[[ "$INSTALL_STATUS" -ne 0 ]] || die 'doctor --install aceptó una instalación cancelada'
grep -Fq 'pacman -S --needed rsync' "$INSTALL_OUTPUT" || die 'doctor --install no mostró el comando concreto'
grep -Fq 'Instalación cancelada para la dependencia «rsync» (rsync); no se modifica el sistema.' "$INSTALL_OUTPUT" || die 'doctor --install no identificó la dependencia cancelada de forma segura'
ok 'instalación explícita muestra comando y respeta la cancelación'
TERMINAL_JSON="$("$BIN" capabilities --format terminal-json)"
grep -Fq '"schema": "ltools-terminal-integration-v1"' <<<"$TERMINAL_JSON" ||
    die 'el descriptor específico de terminal no se pudo generar'
grep -Fq '"required_terminal_capability": "lterminal-startup-v1"' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no declara la capacidad requerida'
grep -Fq '"standalone_releases_require_it": false' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no declara su carácter opcional'
grep -Fq '"exclusive_host_family": "lterminal"' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no limita su integración a LTerminal'
grep -Fq 'WinSlim Terminal' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no declara WinSlim Terminal'
grep -Fq '"action_catalog"' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no enlaza el catálogo de acciones'
if command -v jq >/dev/null 2>&1; then
    jq -e '(.action_catalog.schema == "ltools-actions-v1") and (.action_catalog.shell == "none") and (.actions | length >= 20) and any(.actions[]; .id == "package-search") and any(.actions[]; .id == "package-install") and any(.actions[]; .id == "git-pull") and all(.actions[]; .id and .executable and (.args | type == "array") and .terminal == true and .shell == "none" and (.supports | index("dry-run") != null))' \
        <<<"$TERMINAL_JSON" >/dev/null || die 'las acciones declarativas no tienen el contrato esperado'
fi
ok 'descriptor JSON específico para terminales'
EN_HELP="$(LTOOLS_LANG=en "$BIN" --help)"
grep -Fq 'Usage: ltools' <<<"$EN_HELP" || die 'el idioma inglés no se aplicó al backend Rust'
DE_HELP="$("$BIN" --lang de --help)"
grep -Fq 'Verwendung:' <<<"$DE_HELP" || die 'la opción --lang no se aplicó al backend Rust'
declare -A LANGUAGE_MARKERS=(
    [es]='Uso:' [en]='Usage:' [de]='Verwendung:' [fr]='Utilisation'
    [pt]='Uso:' [it]='Uso:' [pl]='Użycie:'
    [ar]='Uso:' [hi]='Uso:' [ja]='Uso:' [ko]='Uso:' [ro]='Uso:'
    [ru]='Uso:' [uk]='Uso:' [zh]='Uso:'
)
for language in "${!LANGUAGE_MARKERS[@]}"; do
    translated_help="$(LTOOLS_LANG="$language" "$BIN" --help)"
    [[ -n "$translated_help" ]] || die "la ayuda quedó vacía para el idioma Rust $language"
    grep -Fq "${LANGUAGE_MARKERS[$language]}" <<<"$translated_help" ||
        die "el idioma Rust $language no se aplicó a la ayuda"
done
ok 'los 15 idiomas del contrato se aceptan y la ayuda permanece operativa'
THEMED_MENU="$(printf 'q\n' | LTOOLS_LANG=en LTERMINAL_THEME=amber LTOOLS_COLOR=always LTOOLS_NO_CLEAR=1 "$BIN" menu 2>&1)"
grep -Fq $'\033[' <<<"$THEMED_MENU" || die 'la CLI no aplica color ANSI al tema recibido de la terminal'
grep -Fq 'Usage:' <<<"$(env -u LTOOLS_LANG LTERMINAL_LANG=en LTOOLS_NO_CLEAR=1 "$BIN" --help)" ||
    die 'la CLI no recibe el idioma alternativo de la terminal'
MACHINE_WITH_THEME="$(LTOOLS_THEME=matrix LTOOLS_COLOR=always "$BIN" capabilities --format json)"
if command -v jq >/dev/null 2>&1; then
    jq empty <<<"$MACHINE_WITH_THEME" >/dev/null || die 'el tema no debe contaminar la salida JSON con ANSI'
fi
! grep -Fq $'\033[' <<<"$MACHINE_WITH_THEME" || die 'la salida JSON contiene secuencias ANSI'
ok 'contexto CLI de idioma/tema heredado y salida máquina limpia'
DOCTOR_OUTPUT="$(HOME="$TMP_DIR/home" XDG_STATE_HOME="$TMP_DIR/state" "$BIN" doctor)"
grep -Fq 'LTools host diagnostics' <<<"$DOCTOR_OUTPUT" || die 'doctor no funciona como operación de solo lectura'
grep -Fq '[audit]' <<<"$DOCTOR_OUTPUT" || die 'doctor no agrupa herramientas de auditoría'
grep -Fq '[system]' <<<"$DOCTOR_OUTPUT" || die 'doctor no agrupa herramientas de sistema'
grep -Fq '[prefix]' <<<"$DOCTOR_OUTPUT" || die 'doctor no agrupa herramientas de prefijos'
if grep -Fq '[packages]' <<<"$DOCTOR_OUTPUT"; then
    die 'doctor volvió a mezclar gestores de paquetes como dependencias'
fi
if find "$TMP_DIR/state" -type f -name '*.tsv' -print -quit 2>/dev/null | grep -q .; then
    die 'doctor de solo lectura creó un plan innecesario'
fi
ok 'doctor Rust sin crear planes ni modificar el estado'

QUERY_OUT="$TMP_DIR/query-report"
QUERY_STATE="$TMP_DIR/query-state"
HOME="$TMP_DIR/home-query" XDG_STATE_HOME="$QUERY_STATE" "$BIN" packages --packages-only --out "$QUERY_OUT" >/dev/null
if find "$QUERY_STATE" -type f -name '*.tsv' -print -quit 2>/dev/null | grep -q .; then
    die 'el inventario de paquetes creó un plan innecesario'
fi
ok 'consultas directas no generan planes de estado innecesarios'

mkdir -p "$TMP_DIR/root/demo-prefix/drive_c"
printf 'synthetic-prefix\n' > "$TMP_DIR/root/demo-prefix/system.reg"
printf 'demo\n' > "$TMP_DIR/root/demo-prefix/drive_c/demo.exe"
PLAN="$TMP_DIR/list-plan.tsv"
LIST_OUTPUT="$("$BIN" --dry-run --plan "$PLAN" prefix list --root "$TMP_DIR/root")"
[[ -s "$PLAN" ]] || die 'no se creó el plan del listado'
grep -Fq 'demo-prefix' <<<"$LIST_OUTPUT" || die 'el listado no detectó el prefijo sintético'
ok 'listado aislado de un prefijo sintético'
STORAGE_OUTPUT="$("$BIN" storage tools)"
grep -Fq 'Herramientas de almacenamiento' <<<"$STORAGE_OUTPUT" || die 'storage no responde'
STORAGE_MOUNTS="$("$BIN" storage mounts)"
grep -Fq 'Montajes activos' <<<"$STORAGE_MOUNTS" || die 'storage mounts no responde'
STORAGE_PARTITIONS="$("$BIN" storage partitions)"
grep -Fq 'Discos y particiones Linux' <<<"$STORAGE_PARTITIONS" || die 'storage partitions no responde'
STORAGE_DRY_RUN="$("$BIN" --dry-run storage mount /dev/synthetic-ltools)"
grep -Fq 'udisksctl mount' <<<"$STORAGE_DRY_RUN" || grep -Fq 'mount /dev/synthetic-ltools' <<<"$STORAGE_DRY_RUN" || die 'storage mount no genera plan dry-run'
if "$BIN" storage inspect '/dev/synthetic;invalid' >"$TMP_DIR/storage-invalid.out" 2>&1; then
    die 'storage aceptó un dispositivo inválido'
fi
grep -Fq 'dispositivo' "$TMP_DIR/storage-invalid.out" || die 'storage no explicó el objetivo inválido'
ok 'gestor de discos: montajes, particiones, simulación y validación'
STORAGE_USAGE="$("$BIN" storage usage)"
grep -Fq 'Uso de espacio' <<<"$STORAGE_USAGE" || die 'storage usage no responde'
STORAGE_FILESYSTEMS="$("$BIN" storage filesystems)"
grep -Fq 'Sistemas de archivos' <<<"$STORAGE_FILESYSTEMS" || die 'storage filesystems no responde'
STORAGE_STACK="$("$BIN" storage volume-stack)"
grep -Fq 'Capas de almacenamiento' <<<"$STORAGE_STACK" || die 'storage volume-stack no responde'
ok 'consultas avanzadas de almacenamiento y capas nativas'
ACCOUNT_OUTPUT="$("$BIN" accounts identity)"
grep -Eq '^UID[[:space:]]+' <<<"$ACCOUNT_OUTPUT" || die 'accounts identity no responde o no está tabulada'
ACCOUNT_LIST_OUTPUT="$("$BIN" accounts list --human)"
grep -Eq '^USUARIO[[:space:]]+' <<<"$ACCOUNT_LIST_OUTPUT" || die 'accounts list no está tabulada'
ACCOUNT_GROUPS_OUTPUT="$("$BIN" accounts groups --user "$(id -un)")"
grep -Eq '^USUARIO[[:space:]]+GRUPO' <<<"$ACCOUNT_GROUPS_OUTPUT" || die 'accounts groups no está tabulada'
ACCOUNT_CREATE_DRY="$("$BIN" --dry-run accounts create --user ltools-e2e-user --home "$TMP_DIR/home-user" --shell /bin/bash --groups users,developers --system --no-create-home)"
grep -Fq 'useradd' <<<"$ACCOUNT_CREATE_DRY" || die 'accounts create no expone su orden en dry-run'
ACCOUNT_MODIFY_DRY="$("$BIN" --dry-run accounts modify --user ltools-e2e-user --shell /bin/zsh --groups developers --append --lock)"
grep -Fq 'usermod' <<<"$ACCOUNT_MODIFY_DRY" || die 'accounts modify no expone su orden en dry-run'
ACCOUNT_GROUP_DRY="$("$BIN" --dry-run accounts group-add --user ltools-e2e-user --group developers)"
grep -Fq 'usermod --append --groups developers' <<<"$ACCOUNT_GROUP_DRY" || die 'accounts group-add no admite campos separados'
ACCOUNT_EXPIRE_DRY="$("$BIN" --dry-run accounts expire --user ltools-e2e-user --date 2030-01-01 --maxdays 90)"
grep -Fq 'chage' <<<"$ACCOUNT_EXPIRE_DRY" || die 'accounts expire no expone su orden en dry-run'
ACCOUNT_ACTIONS="$("$BIN" --dry-run actions run accounts.add ltools-e2e-user)"
grep -Fq 'Acción: accounts.add' <<<"$ACCOUNT_ACTIONS" || die 'accounts.add no se publica en el catálogo'
ok 'usuarios, grupos, membresías, caducidad y salidas tabuladas'
REGISTRY_OUTPUT="$("$BIN" registry status)"
grep -Fq 'Registros y configuración Linux' <<<"$REGISTRY_OUTPUT" || die 'registry Linux no responde'
ok 'módulos Linux de almacenamiento y configuración'

# Ejecuta cada acción nativa directamente. El menú E2E comprueba la navegación,
# pero estas comprobaciones garantizan además que cada subacción devuelve un
# estado estable aunque una utilidad opcional esté ausente, enmascarada o
# requiera privilegios para mostrar parte de su información.
NATIVE_NETWORK="$("$BIN" native network status 2>&1)" || die 'native network status terminó con error'
grep -Fq 'Red Linux' <<<"$NATIVE_NETWORK" || die 'native network status no mostró su sección Linux'
NATIVE_HARDWARE="$("$BIN" native hardware status 2>&1)" || die 'native hardware status terminó con error'
grep -Fq 'Hardware Linux' <<<"$NATIVE_HARDWARE" || die 'native hardware status no mostró su sección Linux'
NATIVE_POWER="$("$BIN" native power status 2>&1)" || die 'native power status terminó con error por una dependencia opcional'
grep -Fq 'Energía Linux' <<<"$NATIVE_POWER" || die 'native power status no mostró su sección Linux'
NATIVE_SECURITY="$("$BIN" native security status 2>&1)" || die 'native security status terminó con error por permisos o herramienta opcional'
grep -Fq 'Firewall Linux' <<<"$NATIVE_SECURITY" || die 'native security status no mostró su sección Linux'
NATIVE_DNS_DRY_RUN="$("$BIN" --dry-run native network flush-dns 2>&1)" || die 'native network flush-dns dry-run terminó con error'
grep -Fq 'resolvectl flush-caches' <<<"$NATIVE_DNS_DRY_RUN" || die 'native network flush-dns no generó el plan esperado'
ok 'acciones nativas Linux directas: red, hardware, energía, seguridad y DNS'
BOOT_STATUS="$("$BIN" boot status 2>&1)" || die 'boot status Linux terminó con error'
grep -Fq 'Arranque Linux' <<<"$BOOT_STATUS" || die 'boot status Linux no mostró su sección nativa'
BOOT_ALIAS="$("$BIN" tboot status 2>&1)" || die 'alias tboot status terminó con error'
grep -Fq 'Arranque Linux' <<<"$BOOT_ALIAS" || die 'tboot no se tradujo al módulo boot Linux'
BOOT_PLAN="$("$BIN" --dry-run boot plan 2>&1)" || die 'boot plan Linux terminó con error'
grep -Fq 'No se modificarán' <<<"$BOOT_PLAN" || die 'boot plan Linux no confirmó su modo de solo lectura'
for network_action in interfaces routes dns listening connections; do
    "$BIN" native network "$network_action" >/dev/null 2>&1 ||
        die "native network $network_action terminó con error"
done
ok 'arranque Linux: estado, alias, submenú y red separada sin modificaciones'

DIAGNOSTICS_JSON="$("$BIN" diagnostics health --format json)"
if command -v jq >/dev/null 2>&1; then
    jq -e '.schema == "ltools-diagnostics-v1" and .platform == "linux" and (.probes | length >= 3) and all(.probes[]; .key and (.available | type == "boolean") and (.installed | type == "boolean") and (.timed_out | type == "boolean") and (.output | type == "string") and (.error | type == "string"))' \
        <<<"$DIAGNOSTICS_JSON" >/dev/null || die 'diagnostics no generó un JSON válido o incompleto'
else
    grep -Fq 'ltools-diagnostics-v1' <<<"$DIAGNOSTICS_JSON" || die 'diagnostics no generó su esquema JSON'
fi
DIAGNOSTICS_TSV="$("$BIN" diagnostics network --format tsv)"
grep -Fq $'key\tcommand\tavailable\tinstalled\tstatus_code\ttimed_out\toutput\terror' <<<"$DIAGNOSTICS_TSV" || die 'diagnostics no generó cabecera TSV completa'
for diagnostic_action in health network hardware users; do
    "$BIN" diagnostics "$diagnostic_action" >/dev/null || die "diagnostics $diagnostic_action terminó con error"
done
ok 'diagnóstico nativo Linux en salud, red, hardware, usuarios y formatos JSON/TSV'

RELEASE_FIXTURE_DIR="$TMP_DIR/release-assets"
mkdir -p "$RELEASE_FIXTURE_DIR"
printf 'synthetic-appimage\n' > "$RELEASE_FIXTURE_DIR/ltools-$VERSION-linux-x86_64.AppImage"
RELEASE_MANIFEST="$TMP_DIR/ltools-release.json"
"$BIN" release-manifest \
    --output "$RELEASE_MANIFEST" \
    --repository Darkeiser003/Tools \
    --tag "v$VERSION" \
    --artifacts-dir "$RELEASE_FIXTURE_DIR" >/dev/null
grep -Fq '"schema": "ltools-release-v1"' "$RELEASE_MANIFEST" ||
    die 'el manifiesto de release no declara su esquema'
grep -Fq '"platform":"linux"' "$RELEASE_MANIFEST" ||
    die 'el manifiesto de release no detectó Linux'
grep -Eq '"sha256":"[a-f0-9]{64}"' "$RELEASE_MANIFEST" ||
    die 'el manifiesto de release no contiene un SHA-256 válido'
ok 'manifiesto GitHub de release con tamaño y SHA-256'

if [[ -n "$APPIMAGE_PATH" ]]; then
    [[ -x "$APPIMAGE_PATH" ]] || die "AppImage no ejecutable: $APPIMAGE_PATH"
    case "$APPIMAGE_PATH" in
        *-cli.AppImage)
            die '--appimage debe apuntar al AppImage GUI; el perfil CLI se detecta automáticamente como vecino'
            ;;
    esac
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
    if [[ -c /dev/fuse ]] &&
        { command -v fusermount3 >/dev/null 2>&1 || command -v fusermount >/dev/null 2>&1; }; then
        set +e
        timeout 30 "$APPIMAGE_PATH" --doctor >> "$DIRECT_LOG" 2>&1
        DIRECT_STATUS=$?
        set -e
        if [[ "$DIRECT_STATUS" -eq 0 ]]; then
            APPIMAGE_FUSE_WORKS=1
            ok "AppImage montado y ejecutado directamente con FUSE; log: $DIRECT_LOG"
        elif grep -Eiq 'Cannot mount AppImage|mount failed:|Operation not permitted|Cannot access /dev/fuse|fusermount.*failed' "$DIRECT_LOG"; then
            if [[ "$REQUIRE_FUSE" -eq 1 ]]; then
                sed -n '1,160p' "$DIRECT_LOG" >&2
                die 'se exigió FUSE, pero el host rechazó el montaje real del AppImage'
            fi
            printf 'El host anuncia soporte FUSE, pero el montaje real fue denegado; se probará el fallback.\n' >> "$DIRECT_LOG"
            set +e
            APPIMAGE_EXTRACT_AND_RUN=1 timeout 30 "$APPIMAGE_PATH" --doctor >> "$DIRECT_LOG" 2>&1
            FALLBACK_STATUS=$?
            set -e
            if [[ "$FALLBACK_STATUS" -ne 0 ]]; then
                sed -n '1,180p' "$DIRECT_LOG" >&2
                die "falló el fallback AppImage por extracción (código $FALLBACK_STATUS); log: $DIRECT_LOG"
            fi
            skip 'montaje FUSE no autorizado por el host; fallback de extracción verificado'
        else
            sed -n '1,160p' "$DIRECT_LOG" >&2
            die "el AppImage no se pudo abrir directamente (código $DIRECT_STATUS); log: $DIRECT_LOG"
        fi
    else
        [[ "$REQUIRE_FUSE" -eq 0 ]] || die 'se exigió FUSE, pero faltan /dev/fuse o fusermount/fusermount3'
        # En hosts sin FUSE la ejecución directa del runtime AppImage no
        # puede montar su SquashFS. El contrato del builder ofrece extracción
        # como fallback, así que la prueba debe ejercer ese mismo camino.
        set +e
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 30 "$APPIMAGE_PATH" --doctor >> "$DIRECT_LOG" 2>&1
        FALLBACK_STATUS=$?
        set -e
        if [[ "$FALLBACK_STATUS" -ne 0 ]]; then
            printf 'Salida de la ejecución por extracción:\n' >&2
            sed -n '1,160p' "$DIRECT_LOG" >&2
            die "el AppImage no se pudo abrir por extracción (código $FALLBACK_STATUS); log: $DIRECT_LOG"
        fi
        ok "AppImage probado por extracción (FUSE ausente); log: $DIRECT_LOG"
    fi

    if command -v xvfb-run >/dev/null 2>&1; then
        if timeout 10 xvfb-run -a true >/dev/null 2>&1; then
            set +e
            APPIMAGE_GUI_OUTPUT="$(timeout 30 xvfb-run -a env GDK_BACKEND=x11 \
                LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0 \
                LTOOLS_TERMINAL=auto \
                HOME="$TMP_DIR/appimage-gui-home" XDG_STATE_HOME="$TMP_DIR/appimage-gui-state" \
                APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" 2>&1)"
            APPIMAGE_GUI_STATUS=$?
            set -e
            if [[ "$APPIMAGE_GUI_STATUS" -ne 0 ]]; then
                printf 'Salida del AppImage GUI (código %s):\n%s\n' "$APPIMAGE_GUI_STATUS" "$APPIMAGE_GUI_OUTPUT" >&2
                die 'el AppImage normal no pudo abrir y cerrar su GUI sin argumentos'
            fi
            grep -Fq 'LTools se cerró correctamente' <<<"$APPIMAGE_GUI_OUTPUT" ||
                die 'el AppImage normal no confirmó el cierre limpio de la GUI'
            ok 'AppImage normal abre y cierra su GUI sin argumentos'

            RESPONSIVE_MARKER="$TMP_DIR/gui-responsive-layout.marker"
            rm -f -- "$RESPONSIVE_MARKER"
            set +e
            RESPONSIVE_GUI_OUTPUT="$(timeout 30 xvfb-run -a env GDK_BACKEND=x11 \
                LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0 \
                LTOOLS_TERMINAL=auto \
                LTOOLS_GUI_WIDTH=640 LTOOLS_GUI_HEIGHT=480 \
                LTOOLS_GUI_SMOKE_NAV_MARKER="$RESPONSIVE_MARKER" \
                HOME="$TMP_DIR/appimage-gui-small-home" XDG_STATE_HOME="$TMP_DIR/appimage-gui-small-state" \
                APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" 2>&1)"
            RESPONSIVE_GUI_STATUS=$?
            set -e
            if [[ "$RESPONSIVE_GUI_STATUS" -ne 0 ]]; then
                printf 'Salida de la GUI pequeña (código %s):\n%s\n' "$RESPONSIVE_GUI_STATUS" "$RESPONSIVE_GUI_OUTPUT" >&2
                die 'la GUI no pudo abrirse en una ventana pequeña de prueba'
            fi
            grep -Fq 'LTools se cerró correctamente' <<<"$RESPONSIVE_GUI_OUTPUT" ||
                die 'la GUI pequeña no confirmó un cierre limpio'
            grep -Fq 'responsive-layout=vertical' "$RESPONSIVE_MARKER" ||
                die 'la GUI pequeña no activó el layout responsive de una columna'
            ok 'GUI responsiva validada en 640x480 con layout de una columna y sin clipping horizontal'

            # La GUI independiente debe conservar la misma estructura con las
            # paletas soportadas. Se prueban colores representativos de la
            # paleta normal, alto contraste, verde y violeta en una ventana
            # pequeña; así se detectan errores de CSS, cierres tempranos o
            # dependencias accidentales del tema de la terminal anfitriona.
            for gui_theme in ocean contrast matrix violet; do
                THEME_GUI_OUTPUT="$(timeout 20 xvfb-run -a env GDK_BACKEND=x11 \
                    LTOOLS_GUI_THEME="$gui_theme" \
                    LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0 \
                    LTOOLS_TERMINAL=auto LTOOLS_GUI_WIDTH=640 LTOOLS_GUI_HEIGHT=480 \
                    HOME="$TMP_DIR/appimage-gui-$gui_theme-home" \
                    XDG_STATE_HOME="$TMP_DIR/appimage-gui-$gui_theme-state" \
                    APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" 2>&1)"
                THEME_GUI_STATUS=$?
                if [[ "$THEME_GUI_STATUS" -ne 0 ]] ||
                    ! grep -Fq 'LTools se cerró correctamente' <<<"$THEME_GUI_OUTPUT"; then
                    printf 'Salida de la GUI con tema %s (código %s):\n%s\n' \
                        "$gui_theme" "$THEME_GUI_STATUS" "$THEME_GUI_OUTPUT" >&2
                    die "la GUI no pudo completar el smoke con el tema $gui_theme"
                fi
            done
            ok 'temas GUI representativos validados en 640x480'

            # Verifica la transición estructural entre el menú global y una
            # sección. El hook solo existe en modo smoke y deja una marca
            # explícita cuando el dashboard es reemplazado; así el E2E no depende de
            # coordenadas del escritorio para detectar la duplicación que
            # antes quedaba visible.
            NAV_MARKER="$TMP_DIR/gui-navigation.marker"
            set +e
            timeout 20 xvfb-run -a env GDK_BACKEND=x11 \
                LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_SMOKE_HOLD_MS=1200 \
                LTOOLS_GUI_SMOKE_NAV_PAGE=1 LTOOLS_GUI_SMOKE_NAV_MARKER="$NAV_MARKER" \
                LTOOLS_DISABLE_GUI=0 LTOOLS_TERMINAL=auto \
                LTOOLS_GUI_WIDTH=640 LTOOLS_GUI_HEIGHT=480 \
                HOME="$TMP_DIR/gui-navigation-home" \
                XDG_CONFIG_HOME="$TMP_DIR/gui-navigation-config" \
                XDG_STATE_HOME="$TMP_DIR/gui-navigation-state" \
                APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" >/dev/null 2>&1
            NAV_STATUS=$?
            set -e
            [[ "$NAV_STATUS" -eq 0 ]] || die "la navegación GUI estructural terminó con código $NAV_STATUS"
            grep -Fq 'navigation-dashboard-hidden' "$NAV_MARKER" ||
                die 'la navegación GUI no ocultó el menú global al entrar en una sección'
            ok 'navegación GUI reemplaza el menú global por la sección activa'

            if command -v xdotool >/dev/null 2>&1; then
                RESPONSIVE_ACTION_LOG="$TMP_DIR/gui-responsive-action-output.log"
                RESPONSIVE_STARTED=$SECONDS
                set +e
                # El AppImage ya tiene pruebas propias de arranque, cierre,
                # resolución, temas y navegación justo arriba. Para esta
                # prueba temporal de interacción GTK usamos el mismo backend
                # compilado directamente: así el lanzador no puede competir
                # con la ventana de Konsole mientras medimos exclusivamente
                # la modal, el bloqueo de controles y el trabajo asíncrono.
                GUI_BINARY="$BIN" APPIMAGE_PATH="$APPIMAGE_PATH" SMOKE_TMP_DIR="$TMP_DIR" timeout 45 xvfb-run -a bash -c '
                    set -Eeuo pipefail
                    # xvfb-run solo es observable por xdotool si GTK no
                    # hereda el Wayland del escritorio anfitrión. Sin esta
                    # selección explícita la GUI puede abrirse fuera del
                    # display virtual y la prueba ve únicamente la ventana
                    # auxiliar 10x10.
                    export GDK_BACKEND=x11
                    unset WAYLAND_DISPLAY WAYLAND_SOCKET
                    export GTK_USE_PORTAL=0
                    export LTOOLS_GUI_SMOKE=1
                    export LTOOLS_GUI_REQUIRED=1
                    # The AppImage normally integrates with the host terminal
                    # when it is launched without arguments.  A GUI smoke
                    # must be independent from that host preference: select
                    # the autonomous GUI path explicitly.  Do not set
                    # LTOOLS_NO_AUTO_TERMINAL here: in the AppRun contract
                    # that flag intentionally means “run the CLI menu”.
                    export LTOOLS_DISABLE_GUI=0
                    export LTOOLS_TERMINAL=auto
                    # Keep the window alive while the synthetic child action
                    # is running.  The test closes it through the window
                    # manager afterwards; a synchronous callback would not
                    # process that close event until the 5-second child
                    # finishes.
                    export LTOOLS_GUI_SMOKE_HOLD_MS=10000
                    export LTOOLS_GUI_SMOKE_ACTION_DELAY_MS=5000
                    export LTOOLS_GUI_SMOKE_ACTION_AUTO_QUIT=1
                    export LTOOLS_GUI_WIDTH=640
                    export LTOOLS_GUI_HEIGHT=480
                    # Nunca uses la cuenta real para los datos de la prueba:
                    # una ejecución anterior no debe dejar preferencias,
                    # planes ni estado de Konsole que alteren la siguiente.
                    export HOME="$SMOKE_TMP_DIR/gui-responsive-action-home"
                    export XDG_CONFIG_HOME="$HOME/gui-responsive-action-config"
                    export XDG_STATE_HOME="$HOME/gui-responsive-action-state"
                    export LTOOLS_GUI_SMOKE_ACTION_MARKER="$HOME/gui-responsive-action.clicked"
                    export LTOOLS_GUI_LOG="$HOME/gui-responsive-action-gui.log"
                    export LTOOLS_LAUNCH_LOG="$HOME/gui-responsive-action-launch.log"
                    mkdir -p "$HOME" "$XDG_CONFIG_HOME" "$XDG_STATE_HOME"
                    rm -f "$LTOOLS_GUI_SMOKE_ACTION_MARKER"
                    "$GUI_BINARY" >"$HOME/gui-responsive-action.log" 2>&1 &
                    app_pid=$!
                    window=''
                    # AppImages may need several seconds to extract before
                    # creating the GTK window.  A one-second probe made the
                    # old test report a false failure on slower disks.
                    for _ in $(seq 1 300); do
                        # GTK crea una ventana auxiliar 10x10 con el nombre
                        # «ltools». Seleccionar únicamente el título visible
                        # evita enviar los clics al helper del toolkit.
                        window="$(xdotool search --name "^LTools [0-9]" 2>/dev/null | head -n1 || true)"
                        [[ -n "$window" ]] && break
                        sleep 0.1
                    done
                    if [[ -z "$window" ]]; then
                        printf "No se encontró la ventana LTools tras 30 s.\\n"
                        printf "Ventanas detectadas:\\n"
                        xdotool search --name ".*" 2>/dev/null | while read -r id; do
                            printf "  %s: " "$id"
                            xdotool getwindowname "$id" 2>/dev/null || true
                        done
                        printf "Procesos LTools/AppRun:\\n"
                        ps -ef | grep -E "ltools|AppRun" | grep -v grep || true
                        printf "Log del AppImage:\\n"
                        sed -n "1,160p" "$HOME/gui-responsive-action.log" 2>/dev/null || true
                        kill "$app_pid" 2>/dev/null || true
                        exit 21
                    fi
                    # La GUI dispara el callback GTK real del primer botón de
                    # acción cuando se establece este marcador. El archivo se
                    # crea antes, al dibujar la ventana, así que esperar solo
                    # a que exista introduce una carrera con el clic. No usamos
                    # coordenadas: el rail y el layout son responsivos y la
                    # prueba debe seguir siendo válida al redimensionar.
                    # La extracción de un AppImage y la inicialización GTK
                    # pueden tardar bastante más que la aparición de la
                    # ventana, especialmente en discos lentos. Esperar el
                    # estado observable, no un plazo arbitrario de 2 s,
                    # evita falsos negativos sin aceptar una acción ausente.
                    for _ in $(seq 1 300); do
                        if grep -Fq "action-button-trigger" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" 2>/dev/null &&
                           grep -Fq "clicked" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" 2>/dev/null &&
                           grep -Fq "busy-begin" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" 2>/dev/null &&
                           grep -Fq "modal-open" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" 2>/dev/null; then
                            break
                        fi
                        sleep 0.1
                    done
                    [[ -f "$LTOOLS_GUI_SMOKE_ACTION_MARKER" ]] || exit 22
                    grep -Fq "action-button-trigger" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" || {
                        printf "La GUI no pulsó el botón de acción real.\n"
                        cat "$LTOOLS_GUI_SMOKE_ACTION_MARKER"
                        exit 27
                    }
                    grep -Fq "clicked" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" || {
                        printf "El callback GTK del botón no recibió el clic.\n"
                        cat "$LTOOLS_GUI_SMOKE_ACTION_MARKER"
                        exit 28
                    }
                    grep -Fq "busy-begin" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" || {
                        printf "La GUI no registró el estado ocupado al iniciar la acción.\n"
                        cat "$LTOOLS_GUI_SMOKE_ACTION_MARKER"
                        exit 24
                    }
                    grep -Fq "modal-open" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" || {
                        printf "La GUI no abrió el modal de trabajo al iniciar la acción.\n"
                        cat "$LTOOLS_GUI_SMOKE_ACTION_MARKER"
                        exit 26
                    }
                    # No cerramos la ventana mientras el trabajo está en
                    # curso: la GUI debe conservar el contexto y bloquear
                    # nuevas acciones hasta recibir busy-end. Así evitamos
                    # dejar callbacks GTK pendientes contra una ventana ya
                    # destruida y comprobamos el ciclo completo.
                    for _ in $(seq 1 300); do
                        grep -Fq "busy-end" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" && break
                        sleep 0.1
                    done
                    grep -Fq "busy-end" "$LTOOLS_GUI_SMOKE_ACTION_MARKER" || {
                        printf "La GUI no finalizó la acción en segundo plano.\n"
                        cat "$LTOOLS_GUI_SMOKE_ACTION_MARKER"
                        exit 25
                    }
                    action_started_ms=$(date +%s%3N)
                    wait "$app_pid"
                    cat "$HOME/gui-responsive-action.log"
                    action_elapsed_ms=$(( $(date +%s%3N) - action_started_ms ))
                    printf "GUI_ACTION_ELAPSED_MS=%s\n" "$action_elapsed_ms"
                    (( action_elapsed_ms < 3000 )) || exit 23
                ' >"$RESPONSIVE_ACTION_LOG" 2>&1
                RESPONSIVE_ACTION_STATUS=$?
                set -e
                RESPONSIVE_ACTION_OUTPUT="$(cat "$RESPONSIVE_ACTION_LOG")"
                if (( RESPONSIVE_ACTION_STATUS != 0 )); then
                    printf 'Salida de la prueba de acción GUI:\n%s\n' "$RESPONSIVE_ACTION_OUTPUT" >&2
                    die "la GUI no pudo procesar una acción lenta en segundo plano (código $RESPONSIVE_ACTION_STATUS)"
                fi
            grep -Fq 'GUI_ACTION_ELAPSED_MS=' <<<"$RESPONSIVE_ACTION_OUTPUT" ||
                die 'la GUI no confirmó el cierre limpio durante la prueba de acción lenta'
            ok "acciones GUI en segundo plano: la ventana sigue respondiendo ($(grep -o 'GUI_ACTION_ELAPSED_MS=[0-9]*' <<<"$RESPONSIVE_ACTION_OUTPUT" | tail -n1))"

            CANCEL_HOME="$TMP_DIR/gui-cancel-home"
            CANCEL_STATE="$TMP_DIR/gui-cancel-state"
            CANCEL_MARKER="$TMP_DIR/gui-cancel.marker"
            CANCEL_LOG="$TMP_DIR/gui-cancel.log"
            rm -f "$CANCEL_MARKER"
            set +e
            timeout 25 xvfb-run -a env GDK_BACKEND=x11 \
                LTOOLS_GUI_SMOKE=1 LTOOLS_GUI_REQUIRED=1 \
                LTOOLS_DISABLE_GUI=0 LTOOLS_TERMINAL=auto \
                LTOOLS_GUI_SMOKE_HOLD_MS=10000 \
                LTOOLS_GUI_SMOKE_ACTION_DELAY_MS=10000 \
                LTOOLS_GUI_SMOKE_ACTION_CANCEL=1 \
                LTOOLS_GUI_SMOKE_ACTION_AUTO_QUIT=1 \
                LTOOLS_GUI_SMOKE_ACTION_MARKER="$CANCEL_MARKER" \
                LTOOLS_GUI_LOG="$CANCEL_HOME/gui.log" \
                HOME="$CANCEL_HOME" XDG_STATE_HOME="$CANCEL_STATE" \
                APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" >"$CANCEL_LOG" 2>&1
            CANCEL_STATUS=$?
            set -e
            if (( CANCEL_STATUS != 0 )); then
                printf 'Salida de la prueba de cancelación GUI:\n%s\n' "$(cat "$CANCEL_LOG")" >&2
                [[ -f "$CANCEL_MARKER" ]] && cat "$CANCEL_MARKER" >&2 || true
                die "la GUI no pudo cancelar una acción en segundo plano (código $CANCEL_STATUS)"
            fi
            [[ -f "$CANCEL_MARKER" ]] || die 'la prueba de cancelación GUI no produjo su informe'
            grep -Fq 'cancel-button-trigger' "$CANCEL_MARKER" || die 'la GUI no activó el botón Cancelar'
            grep -Fq 'cancel-requested' "$CANCEL_MARKER" || die 'la GUI no registró la petición de cancelación'
            grep -Fq 'busy-end' "$CANCEL_MARKER" || die 'la GUI no liberó el estado ocupado tras cancelar'
            ok 'cancelación GUI: Cancelar durante la acción y Volver al finalizar'

            GUI_SUITE_HOME="$TMP_DIR/gui-all-buttons-home"
            GUI_SUITE_STATE="$TMP_DIR/gui-all-buttons-state"
            GUI_SUITE_MARKER="$TMP_DIR/gui-all-buttons.marker"
            rm -f "$GUI_SUITE_MARKER"
            set +e
            # La suite ejecuta cada ruta segura mediante un proceso hijo. En
            # máquinas lentas, la extracción del AppImage y las consultas
            # nativas pueden superar un minuto sin que exista un bloqueo.
            timeout 120 xvfb-run -a env GDK_BACKEND=x11 \
                LTOOLS_GUI_SMOKE_ALL_BUTTONS_MARKER="$GUI_SUITE_MARKER" \
                LTOOLS_DISABLE_GUI=0 LTOOLS_TERMINAL=auto \
                LTOOLS_NO_MOUNTS=1 HOME="$GUI_SUITE_HOME" XDG_STATE_HOME="$GUI_SUITE_STATE" \
                APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" >"$TMP_DIR/gui-all-buttons.log" 2>&1
            GUI_SUITE_STATUS=$?
            set -e
            if (( GUI_SUITE_STATUS != 0 )); then
                sed -n '1,180p' "$TMP_DIR/gui-all-buttons.log" >&2 || true
                die "la suite de botones GUI terminó con código $GUI_SUITE_STATUS"
            fi
            [[ -f "$GUI_SUITE_MARKER" ]] || {
                sed -n '1,180p' "$TMP_DIR/gui-all-buttons.log" >&2 || true
                die 'la suite de botones GUI no produjo su informe'
            }
            grep -Fq 'GUI_SAFE_ACTIONS_BEGIN' "$GUI_SUITE_MARKER" || die 'la suite GUI no comenzó correctamente'
            grep -Fq 'GUI_SAFE_ACTIONS_END' "$GUI_SUITE_MARKER" || die 'la suite GUI no terminó correctamente'
            if grep -Fq $'FAIL\t' "$GUI_SUITE_MARKER"; then
                cat "$GUI_SUITE_MARKER" >&2
                die 'una ruta segura de botón GUI falló'
            fi
            safe_button_count="$(grep -c $'^OK\t' "$GUI_SUITE_MARKER")"
            [[ "$safe_button_count" -ge 20 ]] || die "la suite GUI solo validó $safe_button_count rutas seguras"
            ok "rutas seguras de botones GUI probadas una por una ($safe_button_count OK; externas/formularios protegidos como SKIP)"
            else
                (( REQUIRE_GUI == 0 )) || die '--require-gui exige xdotool para comprobar interacción GUI'
                skip 'GUI responsiva durante acción: xdotool no está disponible'
            fi
        else
            (( REQUIRE_GUI == 0 )) || die '--require-gui no pudo iniciar un display Xvfb para el AppImage'
            skip 'GUI AppImage: Xvfb no puede crear un display aislado'
        fi
    else
        (( REQUIRE_GUI == 0 )) || die '--require-gui exige xvfb-run para probar el AppImage GUI'
        skip 'GUI AppImage: xvfb-run no está disponible'
    fi

    CLI_APPIMAGE_PATH="${APPIMAGE_PATH%.AppImage}-cli.AppImage"
    if [[ -x "$CLI_APPIMAGE_PATH" ]]; then
        CLI_APPIMAGE_OUTPUT="$(env APPIMAGE_EXTRACT_AND_RUN=1 timeout 30 "$CLI_APPIMAGE_PATH" 2>&1)" ||
            die 'el AppImage CLI terminó con error sin argumentos'
        grep -Fq 'Uso: ltools' <<<"$CLI_APPIMAGE_OUTPUT" ||
            die 'el AppImage CLI sin argumentos no mostró la ayuda'
        ok 'AppImage CLI sin argumentos muestra ayuda'
    fi

    TERMINAL_STUB_DIR="$TMP_DIR/terminal-stub"
    TERMINAL_STUB_LOG="$TMP_DIR/terminal-stub.log"
    mkdir -p "$TERMINAL_STUB_DIR"
    LTERMINAL_STUB_DIR="$TMP_DIR/lterminal-stub"
    LTERMINAL_STUB_LOG="$TMP_DIR/lterminal-stub.log"
    mkdir -p "$LTERMINAL_STUB_DIR"
cat > "$LTERMINAL_STUB_DIR/lterminal" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == --ltools-capabilities ]]; then
    printf 'lterminal-startup-v1\n'
    exit 0
fi
printf '%s\n' "$*" >> "$LTOOLS_LTERMINAL_STUB_LOG"
sleep 2
EOF
    chmod +x "$LTERMINAL_STUB_DIR/lterminal"
cat > "$TERMINAL_STUB_DIR/x-terminal-emulator" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$LTOOLS_TERMINAL_STUB_LOG"
sleep 2
EOF
    chmod +x "$TERMINAL_STUB_DIR/x-terminal-emulator"
    LTERMINAL_OUTPUT_FILE="$TMP_DIR/lterminal-output.log"
    LTERMINAL_LAUNCH_LOG="$TMP_DIR/lterminal-launch.log"
    : > "$LTERMINAL_OUTPUT_FILE"
    env -u LTOOLS_NO_AUTO_TERMINAL -u LTOOLS_TERMINAL_LAUNCH \
        LTOOLS_DISABLE_GUI=1 LTOOLS_TERMINAL=lterminal \
        HOME="$TMP_DIR/home-lterminal" XDG_STATE_HOME="$TMP_DIR/state-lterminal" \
        LTOOLS_LAUNCH_LOG="$LTERMINAL_LAUNCH_LOG" \
        LTOOLS_LTERMINAL="$LTERMINAL_STUB_DIR/lterminal" \
        PATH="$LTERMINAL_STUB_DIR:$TERMINAL_STUB_DIR:$PATH" \
        LTOOLS_LTERMINAL_STUB_LOG="$LTERMINAL_STUB_LOG" \
        LTOOLS_TERMINAL_STUB_LOG="$TERMINAL_STUB_LOG" \
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 10 "$APPIMAGE_PATH" >"$LTERMINAL_OUTPUT_FILE" 2>&1 &
    LTERMINAL_PID=$!
    LTERMINAL_READY=0
    for _ in {1..100}; do
        if grep -Fq 'Menú abierto en LTerminal.' "$LTERMINAL_OUTPUT_FILE" &&
            grep -Fq 'started pid=' "$LTERMINAL_LAUNCH_LOG" 2>/dev/null; then
            LTERMINAL_READY=1
            break
        fi
        if ! kill -0 "$LTERMINAL_PID" 2>/dev/null; then
            break
        fi
        sleep 0.1
    done
    if [[ "$LTERMINAL_READY" -ne 1 ]]; then
        set +e
        wait "$LTERMINAL_PID"
        LTERMINAL_STATUS=$?
        set -e
        sed -n '1,160p' "$LTERMINAL_OUTPUT_FILE" >&2
        [[ -f "$LTERMINAL_LAUNCH_LOG" ]] && sed -n '1,160p' "$LTERMINAL_LAUNCH_LOG" >&2
        die "el lanzador no seleccionó LTerminal con protocolo compatible (código $LTERMINAL_STATUS)"
    fi
    kill "$LTERMINAL_PID" 2>/dev/null || true
    wait "$LTERMINAL_PID" 2>/dev/null || true
    grep -Fq -- '--command' "$LTERMINAL_STUB_LOG" || die 'LTerminal no recibió --command'
    grep -Fq -- '-- menu' "$LTERMINAL_STUB_LOG" || die 'LTerminal no recibió el argumento menu'
    ok 'LTerminal compatible se prioriza y recibe el comando de LTools'

    INCOMPATIBLE_LTERMINAL_DIR="$TMP_DIR/incompatible-lterminal"
    INCOMPATIBLE_TERMINAL_DIR="$TMP_DIR/incompatible-terminal"
    INCOMPATIBLE_TERMINAL_LOG="$TMP_DIR/incompatible-terminal.log"
    mkdir -p "$INCOMPATIBLE_LTERMINAL_DIR" "$INCOMPATIBLE_TERMINAL_DIR"
    cat > "$INCOMPATIBLE_LTERMINAL_DIR/lterminal" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == --ltools-capabilities ]]; then
    printf '{"schema":"unsupported-terminal-capabilities"}\n'
    exit 0
fi
EOF
    chmod +x "$INCOMPATIBLE_LTERMINAL_DIR/lterminal"
cat > "$INCOMPATIBLE_TERMINAL_DIR/x-terminal-emulator" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$LTOOLS_INCOMPATIBLE_TERMINAL_LOG"
sleep 2
EOF
    chmod +x "$INCOMPATIBLE_TERMINAL_DIR/x-terminal-emulator"
    INCOMPATIBLE_OUTPUT="$TMP_DIR/incompatible-lterminal-output.log"
    # La extracción del runtime puede tardar varios segundos cuando se
    # ejecutan varios AppImage seguidos; el límite debe cubrir también su
    # limpieza, no solo la respuesta de AppRun.  Mantener los comentarios
    # fuera de la orden continuada es importante: un comentario después de
    # una barra invertida puede sacar el resto de las variables del entorno.
    set +e
    env -u LTOOLS_ALLOW_TERMINAL_FALLBACK -u LTOOLS_NO_AUTO_TERMINAL -u LTOOLS_TERMINAL_LAUNCH \
        LTOOLS_DISABLE_GUI=1 LTOOLS_TERMINAL=lterminal \
        HOME="$TMP_DIR/home-incompatible" XDG_STATE_HOME="$TMP_DIR/state-incompatible" \
        LTOOLS_LAUNCH_LOG="$TMP_DIR/incompatible-lterminal-launch.log" \
        LTOOLS_LTERMINAL="$INCOMPATIBLE_LTERMINAL_DIR/lterminal" \
        LTOOLS_INCOMPATIBLE_TERMINAL_LOG="$INCOMPATIBLE_TERMINAL_LOG" \
        PATH="$INCOMPATIBLE_LTERMINAL_DIR:$INCOMPATIBLE_TERMINAL_DIR:$PATH" \
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 30 "$APPIMAGE_PATH" >"$INCOMPATIBLE_OUTPUT" 2>&1
    INCOMPATIBLE_STATUS=$?
    set -e
    [[ "$INCOMPATIBLE_STATUS" -eq 3 ]] || die "LTerminal incompatible terminó con código inesperado: $INCOMPATIBLE_STATUS"
    grep -Fq 'Se solicitó LTerminal, pero no se encontró una instalación compatible' "$INCOMPATIBLE_OUTPUT" ||
        die 'LTools no explicó que la integración explícita de LTerminal no está disponible'
    grep -Fq 'lterminal-startup-v1' "$INCOMPATIBLE_OUTPUT" ||
        die 'LTools no indicó el protocolo requerido para LTerminal'
    [[ ! -s "$INCOMPATIBLE_TERMINAL_LOG" ]] ||
        die 'se abrió una terminal alternativa sin autorización explícita'
    ok 'LTerminal incompatible bloquea el fallback silencioso y deja diagnóstico'

    FALLBACK_OUTPUT="$TMP_DIR/explicit-fallback-output.log"
    set +e
    env -u LTOOLS_NO_AUTO_TERMINAL -u LTOOLS_TERMINAL_LAUNCH \
        LTOOLS_DISABLE_GUI=1 LTOOLS_ALLOW_TERMINAL_FALLBACK=1 \
        LTOOLS_TERMINAL=x-terminal-emulator \
        HOME="$TMP_DIR/home-fallback" XDG_STATE_HOME="$TMP_DIR/state-fallback" \
        LTOOLS_LAUNCH_LOG="$TMP_DIR/fallback-launch.log" \
        LTOOLS_LTERMINAL="$INCOMPATIBLE_LTERMINAL_DIR/lterminal" \
        LTOOLS_INCOMPATIBLE_TERMINAL_LOG="$INCOMPATIBLE_TERMINAL_LOG" \
        PATH="$INCOMPATIBLE_LTERMINAL_DIR:$INCOMPATIBLE_TERMINAL_DIR:$PATH" \
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 10 "$APPIMAGE_PATH" >"$FALLBACK_OUTPUT" 2>&1
    FALLBACK_STATUS=$?
    set -e
    [[ "$FALLBACK_STATUS" -eq 0 ]] || die "fallback explícito terminó con código inesperado: $FALLBACK_STATUS"
    grep -Fq 'Menú autónomo abierto en x-terminal-emulator usando' "$FALLBACK_OUTPUT" ||
        die 'el fallback explícito no abrió la terminal seleccionada'
    grep -Fq 'menu' "$INCOMPATIBLE_TERMINAL_LOG" ||
        die 'el fallback explícito no recibió el menú'
    ok 'fallback de terminal disponible solo con autorización explícita'

    AUTO_LAUNCH_LOG="$TMP_DIR/auto-launch.log"
    AUTO_TERMINAL_OUTPUT_FILE="$TMP_DIR/auto-terminal-output.log"
    : > "$AUTO_TERMINAL_OUTPUT_FILE"
    # Keep this check independent from FUSE mount cleanup.  The AppImage is
    # still exercised, but extraction mode prevents a terminal child from
    # keeping the runtime's captured pipe open on some desktop environments.
    env -u LTOOLS_NO_AUTO_TERMINAL -u LTOOLS_TERMINAL_LAUNCH \
        LTOOLS_DISABLE_GUI=1 HOME="$TMP_DIR/home" XDG_STATE_HOME="$TMP_DIR/state" LTOOLS_LAUNCH_LOG="$AUTO_LAUNCH_LOG" \
        LTOOLS_LTERMINAL="$INCOMPATIBLE_LTERMINAL_DIR/lterminal" LTOOLS_TERMINAL=auto \
        PATH="$TERMINAL_STUB_DIR:$PATH" LTOOLS_TERMINAL_STUB_LOG="$TERMINAL_STUB_LOG" \
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 10 "$APPIMAGE_PATH" >"$AUTO_TERMINAL_OUTPUT_FILE" 2>&1 &
    AUTO_LAUNCH_PID=$!
    AUTO_LAUNCH_READY=0
    for _ in {1..100}; do
        if grep -Fq 'Menú autónomo abierto en x-terminal-emulator usando' "$AUTO_TERMINAL_OUTPUT_FILE" &&
            grep -Fq 'started pid=' "$AUTO_LAUNCH_LOG" 2>/dev/null; then
            AUTO_LAUNCH_READY=1
            break
        fi
        if ! kill -0 "$AUTO_LAUNCH_PID" 2>/dev/null; then
            break
        fi
        sleep 0.1
    done
    if [[ "$AUTO_LAUNCH_READY" -ne 1 ]]; then
        set +e
        wait "$AUTO_LAUNCH_PID"
        AUTO_LAUNCH_STATUS=$?
        set -e
        printf 'Salida del lanzador sin TTY (código %s):\n' "$AUTO_LAUNCH_STATUS" >&2
        sed -n '1,160p' "$AUTO_TERMINAL_OUTPUT_FILE" >&2
        [[ -f "$AUTO_LAUNCH_LOG" ]] && sed -n '1,160p' "$AUTO_LAUNCH_LOG" >&2
        die 'el AppImage no pudo solicitar una terminal gráfica sin TTY'
    fi
    # Evidence is already recorded; do not make the build wait for a desktop
    # terminal or a wrapper that intentionally remains open.
    kill "$AUTO_LAUNCH_PID" 2>/dev/null || true
    wait "$AUTO_LAUNCH_PID" 2>/dev/null || true
    AUTO_TERMINAL_OUTPUT="$(<"$AUTO_TERMINAL_OUTPUT_FILE")"
    grep -Fq 'Menú autónomo abierto en x-terminal-emulator usando' <<<"$AUTO_TERMINAL_OUTPUT" ||
        die 'el AppImage no intentó abrir una terminal al iniciarse sin argumentos'
    grep -Fq 'menu' "$TERMINAL_STUB_LOG" ||
        die 'el lanzador no pasó el comando menu a la terminal gráfica'
    grep -Fq 'Registro del lanzador:' <<<"$AUTO_TERMINAL_OUTPUT" ||
        die 'el lanzador no informó el registro de apertura gráfica'
    grep -Fq 'started pid=' "$AUTO_LAUNCH_LOG" ||
        die 'el registro del lanzador no confirmó el proceso de terminal'
    ok 'apertura sin argumentos redirige el menú a una terminal gráfica'

    NOARGS_OUTPUT="$(printf 'q\n' | timeout 30 env APPIMAGE_EXTRACT_AND_RUN=1 LTOOLS_NO_AUTO_TERMINAL=1 "$APPIMAGE_PATH" 2>&1)"
    grep -Fq 'Elige una opción' <<<"$NOARGS_OUTPUT" || die 'el menú interactivo no se mostró al iniciar sin argumentos'
    ok 'menú interactivo del AppImage'
    APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --version >/dev/null
    ok 'AppImage responde usando extracción temporal'
    APPIMAGE_ZH_HELP="$(APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --lang zh --help)" ||
        die 'la ayuda Rust del AppImage no pudo ejecutarse con --lang zh'
    grep -Fq 'Uso:' <<<"$APPIMAGE_ZH_HELP" ||
        die 'la ayuda Rust del AppImage no respeta --lang zh'
    APPIMAGE_PL_HELP="$(APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --lang pl --help)" ||
        die 'la ayuda Rust del AppImage no pudo ejecutarse con --lang pl'
    grep -Fq 'Użycie:' <<<"$APPIMAGE_PL_HELP" ||
        die 'la ayuda Rust del AppImage no respeta --lang pl'
    ok 'idiomas del catálogo LTerminal en la CLI del AppImage'
    APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --doctor >/dev/null
    ok 'diagnóstico del AppImage'
    if [[ "$APPIMAGE_FUSE_WORKS" -eq 1 ]]; then
        ok 'el montaje real de AppImage con FUSE se verificó durante la prueba inicial'
    else
        skip 'montaje FUSE normal: no operativo en este host; se validó el fallback por extracción'
    fi
    set +e
    FUSE_OUTPUT="$(APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --fuse-check 2>&1)"
    FUSE_STATUS=$?
    set -e
    [[ "$FUSE_STATUS" -eq 0 || "$FUSE_STATUS" -eq 1 ]] || die '--fuse-check terminó con un código inesperado'
    grep -Fq 'FUSE' <<<"$FUSE_OUTPUT" || die '--fuse-check no mostró diagnóstico FUSE'
    grep -Eq 'actual mount permission unverified|device/helper missing' <<<"$FUSE_OUTPUT" ||
        die '--fuse-check afirmó disponibilidad sin distinguir detección de permiso real de montaje'
    ok 'diagnóstico FUSE'
    if [[ -n "$RUNNER_PATH" ]]; then
        [[ -x "$RUNNER_PATH" ]] || die "lanzador no ejecutable: $RUNNER_PATH"
        set +e
        RUNNER_OUTPUT="$(timeout 30 "$RUNNER_PATH" --appimage "$APPIMAGE_PATH" --doctor 2>&1)"
        RUNNER_STATUS=$?
        set -e
        if [[ "$RUNNER_STATUS" -ne 0 ]]; then
            printf 'Salida del lanzador AppImage (código %s):\n%s\n' "$RUNNER_STATUS" "$RUNNER_OUTPUT" >&2
            die 'el lanzador no recuperó la ejecución si el montaje FUSE falla'
        fi
        ok 'lanzador AppImage prueba FUSE y recupera por extracción si el host lo deniega'
        LTOOLS_FORCE_EXTRACT=1 "$RUNNER_PATH" --version >/dev/null
        ok 'lanzador externo con fallback forzado sin FUSE'
    fi
else
    skip 'AppImage: no se proporcionó --appimage'
fi

printf 'Smoke tests completados correctamente.\n'
