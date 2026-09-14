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
    local capture_path="$1" description="$2" entropy=""
    for _ in {1..30}; do
        import -window root "$capture_path"
        if command -v identify >/dev/null 2>&1; then
            entropy="$(LC_ALL=C identify -format '%[entropy]' "$capture_path" 2>/dev/null || true)"
        else
            entropy="$(LC_ALL=C magick identify -format '%[entropy]' "$capture_path" 2>/dev/null || true)"
        fi
        if awk -v value="$entropy" 'BEGIN { exit !(value >= 0.10) }'; then
            return 0
        fi
        sleep 0.2
    done
    printf 'La captura de %s no llegó a renderizar el contenido esperado (entropía %s)\n' \
        "$description" "${entropy:-no disponible}" >&2
    return 1
}

wait_for_navigation() {
    local page="$1" title="$2"
    for _ in {1..50}; do
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
    # El título Win32 es «WinSlim-Tools <versión>»; buscar «LTools» nunca
    # encuentra esta ventana bajo Wine.
    window_id="$(xdotool search --onlyvisible --name 'WinSlim-Tools' 2>/dev/null | head -n1 || true)"
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
        7) expected_title='WinSlim' ;;
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
xdotool mousemove --sync --window "$window_id" 270 269 click 1
wait_for_navigation 8 'Usuarios, grupos y sesiones'
capture_when_rendered "$accounts_capture" 'submenú de cuentas'
printf 'GUI_PAGE_OK=8\n'
xdotool windowclose "$window_id"
wait "$gui_pid"
gui_pid=""
