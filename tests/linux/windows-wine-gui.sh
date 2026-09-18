#!/usr/bin/env bash
# Interacción/capturas de la GUI Windows ejecutada dentro del Xvfb del runner.
# Separado del script principal para que Bash y el analizador de sintaxis puedan
# validar realmente este flujo, sin ocultarlo dentro de una cadena `bash -c`.
set -Eeuo pipefail

if (( $# != 10 )); then
    printf 'Uso interno: windows-wine-gui.sh RUNNER MODE EXE DASHBOARD CAPTURE_DIR NATIVE SETTINGS NAV_MARKER READY_MARKER PAGES\n' >&2
    exit 2
fi

runner="$1"
mode="$2"
executable="$3"
dashboard_capture="$4"
capture_dir="$5"
native_capture="$6"
settings_capture="$7"
accounts_capture="$capture_dir/windows-accounts-wine.png"
marker="$8"
ready_marker="$9"
IFS=, read -r -a gui_pages <<<"${10}"
gui_pid=""

cleanup_gui_process() {
    local exit_status=$?
    trap - EXIT
    if (( exit_status != 0 )) && [[ -n "$gui_pid" ]] && kill -0 "$gui_pid" 2>/dev/null; then
        kill "$gui_pid" 2>/dev/null || true
        wait "$gui_pid" 2>/dev/null || true
    fi
    exit "$exit_status"
}
trap cleanup_gui_process EXIT

capture_when_rendered() {
    local capture_path="$1" description="$2" entropy="" geometry x y width height
    geometry="$(xdotool getwindowgeometry --shell "$window_id" 2>/dev/null || true)"
    read -r x y width height < <(awk -F= '
        $1 == "X" { x = $2 }
        $1 == "Y" { y = $2 }
        $1 == "WIDTH" { width = $2 }
        $1 == "HEIGHT" { height = $2 }
        END { print x, y, width, height }
    ' <<<"$geometry")
    if [[ ! "$x" =~ ^[0-9]+$ || ! "$y" =~ ^[0-9]+$ ||
        ! "$width" =~ ^[1-9][0-9]*$ || ! "$height" =~ ^[1-9][0-9]*$ ]]; then
        printf 'No se pudo obtener la geometría visible de la ventana %s\n' "$description" >&2
        return 1
    fi
    for _ in {1..30}; do
        import -window root "$capture_path"
        if command -v identify >/dev/null 2>&1; then
            entropy="$(LC_ALL=C identify -crop "${width}x${height}+${x}+${y}" \
                -format '%[entropy]' "$capture_path" 2>/dev/null || true)"
        else
            entropy="$(LC_ALL=C magick identify -crop "${width}x${height}+${x}+${y}" \
                -format '%[entropy]' "$capture_path" 2>/dev/null || true)"
        fi
        if awk -v value="$entropy" 'BEGIN { exit !(value >= 0.10) }'; then
            return 0
        fi
        sleep 0.2
    done
    printf 'La ventana de %s siguió vacía o sin controles renderizados (entropía %s)\n' \
        "$description" "${entropy:-no disponible}" >&2
    return 1
}

wait_for_navigation() {
    local page="$1" title="$2"
    # Hardware/CIM y powercfg pueden arrancar un proceso Windows bastante más
    # lento que una consulta de red. Esperar hasta 15 s evita falsos negativos
    # sin dejar que una acción colgada bloquee indefinidamente la E2E.
    for _ in {1..150}; do
        if [[ -s "$marker" ]] &&
            grep -Fq "navigation-page=$page" "$marker" &&
            grep -Fqx "navigation-title=$title" "$marker"; then
            return 0
        fi
        sleep 0.1
    done
    printf 'No abrió la página Win32 %s (%s)\n' "$page" "$title" >&2
    return 1
}

if [[ "$mode" == proton ]]; then
    "$runner" run "$executable" &
else
    "$runner" "$executable" &
fi
gui_pid=$!
window_id=""
for _ in {1..100}; do
    # El título Win32 es «WTools <versión>»; buscar «LTools» nunca
    # encuentra esta ventana bajo Wine.
    window_id="$(xdotool search --onlyvisible --name 'WTools' 2>/dev/null | head -n1 || true)"
    [[ -n "$window_id" && -s "$ready_marker" ]] && break
    sleep 0.1
done
[[ -n "$window_id" ]] || { printf 'No apareció la ventana Win32 de LTools\n' >&2; exit 1; }
[[ -s "$ready_marker" ]] || { printf 'La ventana Win32 no completó su primer render\n' >&2; exit 1; }
xdotool windowraise "$window_id"
capture_when_rendered "$dashboard_capture" 'panel principal'

for page in "${gui_pages[@]}"; do
    # La GUI coloca las categorías en una columna con 50 px entre centros.
    button_y=$((87 + page * 50))
    xdotool mousemove --sync --window "$window_id" 520 "$button_y" click 1
    case "$page" in
        0) expected_title='Auditar / Inventariar' ;;
        1) expected_title='Herramientas nativas' ;;
        2) expected_title='Dependencias' ;;
        3) expected_title='Rutas predeterminadas' ;;
        4) expected_title='Herramientas instalables' ;;
        5) expected_title='Automatización' ;;
        6) expected_title='Ajustes de LTools' ;;
        7) expected_title='WTools' ;;
        8) expected_title='Usuarios, grupos y sesiones' ;;
        *) printf 'Página Win32 desconocida: %s\n' "$page" >&2; exit 1 ;;
    esac
    wait_for_navigation "$page" "$expected_title"
    printf 'GUI_PAGE_OK=%s\n' "$page"
    capture="$capture_dir/windows-menu-pages/windows-menu-page-$page.png"
    [[ "$page" -ne 1 ]] || capture="$native_capture"
    [[ "$page" -ne 6 ]] || capture="$settings_capture"
    capture_when_rendered "$capture" "categoría $page"
    if [[ "$page" -ne 6 ]]; then
        # El botón Volver está centrado junto al borde inferior.
        xdotool mousemove --sync --window "$window_id" 520 643 click 1
        sleep 0.2
    fi
done

# Cuentas es un submenú real abierto desde «Herramientas nativas».
xdotool mousemove --sync --window "$window_id" 520 643 click 1
sleep 0.2
xdotool mousemove --sync --window "$window_id" 520 137 click 1
wait_for_navigation 1 'Herramientas nativas'
xdotool mousemove --sync --window "$window_id" 270 234 click 1
wait_for_navigation 8 'Usuarios, grupos y sesiones'
capture_when_rendered "$accounts_capture" 'submenú de cuentas'
printf 'GUI_PAGE_OK=8\n'

# The native page has additional read-only queries. Exercise the real Win32 buttons
# and dismiss each result dialog so a label that merely renders cannot pass as
# a functional action. The normal 1280x900 smoke geometry uses 42 px rows;
# the application switches to 30px compact rows only for shorter windows.
close_result_dialog() {
    local active_window dialog_window
    for _ in {1..50}; do
        # UMU/Wine exposes MessageBoxW with the product title but with the
        # runner's own X11 class, not the usual Win32 #32770 class.
        dialog_window="$(xdotool search --onlyvisible --name '^WTools$' 2>/dev/null | tail -n1 || true)"
        if [[ -n "$dialog_window" && "$dialog_window" != "$window_id" ]]; then
            # Wine may destroy the modal between search and key delivery;
            # that is a normal close race, not a failed LTools action.
            xdotool key --window "$dialog_window" Return 2>/dev/null || true
            return 0
        fi
        active_window="$(xdotool getactivewindow 2>/dev/null || true)"
        if [[ -n "$active_window" && "$active_window" != "$window_id" ]]; then
            xdotool key --window "$active_window" Return 2>/dev/null || true
            return 0
        fi
        sleep 0.1
    done
    return 1
}
xdotool mousemove --sync --window "$window_id" 520 643 click 1
sleep 0.2
xdotool mousemove --sync --window "$window_id" 520 137 click 1
wait_for_navigation 1 'Herramientas nativas'
for action_index in 10 11 12 13 14 15 16 17 18 19 20 21; do
    action_column=$((action_index % 2))
    action_row=$((action_index / 2))
    action_x=$((270 + action_column * 480))
    # xdotool coordinates are relative to the outer Win32 window; the
    # client origin is offset by the non-client frame. This lands near the
    # center of the compact button in the fixed smoke geometry.
    action_y=$((52 + 38 + action_row * 42 + 18))
    xdotool mousemove --sync --window "$window_id" "$action_x" "$action_y" click 1
    close_result_dialog || {
        printf 'La acción GUI Windows %s no mostró diálogo de resultado\n' "$action_index" >&2
        exit 1
    }
    if (( action_index <= 12 )); then
        printf 'GUI_ACTION_OK=storage-%s\n' "$action_index"
    elif (( action_index <= 17 )); then
        printf 'GUI_ACTION_OK=native-network-%s\n' "$action_index"
    else
        printf 'GUI_ACTION_OK=native-system-%s\n' "$action_index"
    fi
done

# Leave the scripted interaction on the accounts page, preserving the
# existing navigation assertion and its capture.
xdotool mousemove --sync --window "$window_id" 520 643 click 1
sleep 0.2
xdotool mousemove --sync --window "$window_id" 520 137 click 1
wait_for_navigation 1 'Herramientas nativas'
xdotool mousemove --sync --window "$window_id" 270 234 click 1
wait_for_navigation 8 'Usuarios, grupos y sesiones'
xdotool windowclose "$window_id"
wait "$gui_pid"
gui_pid=""
