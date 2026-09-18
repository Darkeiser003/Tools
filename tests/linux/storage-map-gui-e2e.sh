#!/usr/bin/env bash
# Prueba acciones reales del mapa GTK con datos efímeros y papelera aislada.
set -Eeuo pipefail

BIN=""
TMP_DIR=""
CAPTURE_DIR=""
REQUIRE_GUI=0
SCREEN="1280x900"
LAYOUT_ONLY=0
SKIP_COMPACT_CHECK=0
while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || { echo "--binary necesita una ruta" >&2; exit 2; }; BIN="$2"; shift ;;
        --tmp) (($# >= 2)) || { echo "--tmp necesita una ruta" >&2; exit 2; }; TMP_DIR="$2"; shift ;;
        --captures) (($# >= 2)) || { echo "--captures necesita una ruta" >&2; exit 2; }; CAPTURE_DIR="$2"; shift ;;
        --require-gui) REQUIRE_GUI=1 ;;
        --screen) (($# >= 2)) || { echo "--screen necesita ANCHOxALTO" >&2; exit 2; }; SCREEN="$2"; shift ;;
        --layout-only) LAYOUT_ONLY=1 ;;
        --skip-compact-check) SKIP_COMPACT_CHECK=1 ;;
        -h|--help) printf 'Uso: %s --binary RUTA --tmp RUTA --captures RUTA [--require-gui] [--screen ANCHOxALTO] [--layout-only]\n' "$0"; exit 0 ;;
        *) echo "Opción desconocida: $1" >&2; exit 2 ;;
    esac
    shift
done

[[ -x "$BIN" ]] || { echo "No existe el binario GUI: $BIN" >&2; exit 2; }
[[ -n "$TMP_DIR" && -n "$CAPTURE_DIR" ]] || {
    echo "Se requieren --tmp y --captures" >&2
    exit 2
}
command -v realpath >/dev/null 2>&1 || { echo 'Se necesita realpath para normalizar las rutas E2E.' >&2; exit 2; }
TMP_DIR="$(realpath -m -- "$TMP_DIR")"
CAPTURE_DIR="$(realpath -m -- "$CAPTURE_DIR")"
[[ "$SCREEN" =~ ^[0-9]+x[0-9]+$ ]] || { echo "Pantalla inválida: $SCREEN" >&2; exit 2; }
mkdir -p -- "$TMP_DIR" "$CAPTURE_DIR"
if ! command -v xvfb-run >/dev/null || ! command -v xdpyinfo >/dev/null \
    || ! command -v xdotool >/dev/null \
    || ! command -v timeout >/dev/null \
    || { ! command -v import >/dev/null && ! command -v magick >/dev/null; } \
    || { ! command -v identify >/dev/null && ! command -v magick >/dev/null; }; then
    if (( REQUIRE_GUI )); then
        echo "ERROR: se requieren xvfb-run, xdpyinfo, xdotool, timeout e ImageMagick (import y identify, o magick)" >&2
        exit 2
    fi
    echo "SKIP: faltan xvfb-run, xdpyinfo, xdotool, timeout o ImageMagick para acciones/capturas GUI"
    exit 0
fi
if ! timeout 10 xvfb-run -a -s "-screen 0 1280x900x24" xdpyinfo >/dev/null 2>&1; then
    if (( REQUIRE_GUI )); then
        echo "ERROR: Xvfb no acepta conexiones X11; se cancela la E2E GUI del mapa" >&2
        exit 2
    fi
    echo "SKIP: Xvfb inició pero no acepta clientes X11"
    exit 0
fi

if [[ "${LTOOLS_STORAGE_MAP_GUI_E2E_INNER:-0}" != 1 ]]; then
    inner_args=(--binary "$BIN" --tmp "$TMP_DIR" --captures "$CAPTURE_DIR")
    if (( REQUIRE_GUI )); then inner_args+=(--require-gui); fi
    inner_args+=(--screen "$SCREEN")
    if (( LAYOUT_ONLY )); then inner_args+=(--layout-only); fi
    if (( SKIP_COMPACT_CHECK )); then inner_args+=(--skip-compact-check); fi
    if [[ "$SCREEN" == "1280x900" ]] && (( ! LAYOUT_ONLY && ! SKIP_COMPACT_CHECK )); then
        compact_args=(--binary "$BIN" --tmp "$TMP_DIR" --captures "$CAPTURE_DIR" \
            --screen 640x480 --layout-only --skip-compact-check)
        if (( REQUIRE_GUI )); then compact_args+=(--require-gui); fi
        bash "$0" "${compact_args[@]}"
    fi
    exec timeout 75 xvfb-run -a -s "-screen 0 ${SCREEN}x24" \
        env LTOOLS_STORAGE_MAP_GUI_E2E_INNER=1 bash "$0" "${inner_args[@]}"
fi

RUN_DIR="$(mktemp -d "$TMP_DIR/storage-map-gui-e2e.XXXXXX")"
CONFIRM_ACTION="action"
HOME_DIR="$RUN_DIR/home"
SOURCE="$HOME_DIR/.cache/ltools-map/source.txt"
TREE_ROOT="$RUN_DIR/tree-root"
COPY="$RUN_DIR/copy.txt"
MOVED="$RUN_DIR/moved.txt"
mkdir -p -- "${SOURCE%/*}"
printf 'LTools GUI map action fixture\n' >"$SOURCE"
mkdir -p -- "$TREE_ROOT/child/grandchild"
printf 'LTools GUI map expansion fixture\n' >"$TREE_ROOT/child/grandchild/nested.txt"
GUI_PID=""
MAP_WINDOW=""
ACTION_ROW_Y=0
ACTION_TREE_ROW_Y=0
ACTION_COPY_X=330
ACTION_MOVE_X=410
ACTION_TRASH_X=520

close_gui() {
    if [[ -n "$GUI_PID" ]]; then
        kill "$GUI_PID" 2>/dev/null || true
        wait "$GUI_PID" 2>/dev/null || true
        GUI_PID=""
    fi
}

cleanup() {
    local status=$?
    close_gui
    if [[ -n "$RUN_DIR" && -d "$RUN_DIR" ]]; then
        if (( status == 0 )); then
            rm -rf -- "$RUN_DIR"
        else
            printf 'E2E fallida; se conservan fixture y logs para diagnóstico: %s\n' "$RUN_DIR" >&2
        fi
    fi
    return "$status"
}
trap cleanup EXIT

capture_screen() {
    local destination="$1"
    if command -v import >/dev/null; then
        import -window root "$destination" >/dev/null 2>&1
    else
        magick import -window root "$destination" >/dev/null 2>&1
    fi
    [[ -s "$destination" ]] || {
        echo "No se pudo guardar la captura GUI: $destination" >&2
        return 1
    }
}

capture_window() {
    local window="$1" destination="$2"
    if command -v import >/dev/null; then
        import -window "$window" "$destination" >/dev/null 2>&1
    else
        magick import -window "$window" "$destination" >/dev/null 2>&1
    fi
    [[ -s "$destination" ]] || {
        echo "No se pudo guardar la captura del diálogo: $destination" >&2
        return 1
    }
}

image_geometry() {
    local image="$1"
    if command -v identify >/dev/null; then
        identify -format '%wx%h' "$image"
    else
        magick identify -format '%wx%h' "$image"
    fi
}

wait_for_tree_layout() {
    local marker="$1" require_compact="$2"
    for _ in {1..40}; do
        if [[ -s "$marker" ]] && awk -F '\t' -v compact="$require_compact" '
            $1 == "STORAGE_TREE_LAYOUT" && $2 ~ /^[0-9]+$/ && $3 ~ /^[0-9]+$/ && $4 ~ /^[0-9]+$/ &&
            $3 <= $2 && $4 < $3 && $4 >= 132 && (!compact || $2 < 700) { valid = 1 }
            END { exit !valid }
        ' "$marker"; then
            return 0
        fi
        sleep 0.05
    done
    echo "La columna y el ajuste de texto no se adaptaron al viewport GTK: $marker" >&2
    [[ -f "$marker" ]] && cat "$marker" >&2
    return 1
}

launch_map() {
    local path="$1" tag="$2" home="$HOME_DIR"
    local state="$RUN_DIR/$2-state" data="$RUN_DIR/$2-data" config="$RUN_DIR/$2-config"
    local ready_marker="$RUN_DIR/$tag.ready"
    local layout_marker="$RUN_DIR/$tag.layout"
    mkdir -p -- "$home" "$state" "$data" "$config" "$home/.local/share/Trash"
    export GDK_BACKEND=x11 GTK_USE_PORTAL=0
    unset WAYLAND_DISPLAY WAYLAND_SOCKET
    unset LTOOLS_GUI_TREE_CANCEL LTOOLS_GUI_TREE_CANCEL_DELAY_MS \
        LTOOLS_GUI_TREE_CANCEL_REQUESTED_MARKER LTOOLS_GUI_TREE_CANCELLED_MARKER
    export LTOOLS_NO_MOUNTS=1 LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0
    export LTOOLS_GUI_TREE_SMOKE=1 LTOOLS_GUI_TREE_SMOKE_HOLD_MS=45000
    export LTOOLS_GUI_TREE_PATH="$path" LTOOLS_GUI_TREE_MARKER="$RUN_DIR/$tag.marker"
    export LTOOLS_GUI_TREE_READY_MARKER="$ready_marker"
    export LTOOLS_GUI_AUDIT_MARKER="$layout_marker"
    export LTOOLS_LANG=es HOME="$home" XDG_STATE_HOME="$state" XDG_DATA_HOME="$data" XDG_CONFIG_HOME="$config"
    "$BIN" >"$RUN_DIR/$tag.log" 2>&1 &
    GUI_PID=$!
    MAP_WINDOW=""
    for _ in {1..100}; do
        MAP_WINDOW="$(xdotool search --onlyvisible --name "Mapa interactivo" 2>/dev/null | head -n1 || true)"
        [[ -n "$MAP_WINDOW" ]] && break
        sleep 0.1
    done
    if [[ -z "$MAP_WINDOW" ]]; then
        cat "$RUN_DIR/$tag.log" >&2 || true
        echo "No apareció el mapa interactivo ($tag)" >&2
        return 1
    fi
    for _ in {1..100}; do
        [[ -s "$ready_marker" ]] && break
        sleep 0.1
    done
    if [[ ! -s "$ready_marker" ]]; then
        cat "$RUN_DIR/$tag.log" >&2 || true
        echo "El mapa no terminó de cargar ($tag)" >&2
        return 1
    fi
    wait_for_tree_layout "$layout_marker" 0
    grep -Fq $'STORAGE_TREE_CONTROLS_LAYOUT\trows=2\tbuttons=7' "$layout_marker" || {
        echo 'El mapa no registró las dos filas completas de acciones; puede haber recorte horizontal' >&2
        cat "$layout_marker" >&2
        return 1
    }
    if [[ "$path" == */.cache/* ]] &&
        ! grep -Fq $'STORAGE_TREE_EXPLANATION\tCaché de usuario; suele poder limpiarse' "$layout_marker"; then
        echo "El árbol GTK no registró la explicación estándar de la ruta fixture: $layout_marker" >&2
        cat "$layout_marker" >&2
        return 1
    fi
    sleep 0.2
    if (( LAYOUT_ONLY )); then
        wait_for_tree_layout "$RUN_DIR/$tag.layout" 1
        local compact_geometry compact_x compact_y compact_width compact_height screen_width screen_height
        compact_geometry="$(xdotool getwindowgeometry --shell "$MAP_WINDOW")"
        read -r screen_width screen_height < <(xdotool getdisplaygeometry)
        compact_x="$(awk -F= '$1 == "X" { print $2 }' <<<"$compact_geometry")"
        compact_y="$(awk -F= '$1 == "Y" { print $2 }' <<<"$compact_geometry")"
        compact_width="$(awk -F= '$1 == "WIDTH" { print $2 }' <<<"$compact_geometry")"
        compact_height="$(awk -F= '$1 == "HEIGHT" { print $2 }' <<<"$compact_geometry")"
        local compact_capture="$CAPTURE_DIR/linux-storage-map-$tag-${SCREEN}-display.png"
        local compact_window_capture="$CAPTURE_DIR/linux-storage-map-$tag-${SCREEN}-window.png"
        capture_screen "$compact_capture"
        capture_window "$MAP_WINDOW" "$compact_window_capture"
        local capture_geometry window_capture_geometry
        capture_geometry="$(image_geometry "$compact_capture")"
        window_capture_geometry="$(image_geometry "$compact_window_capture")"
        if [[ "$screen_width"x"$screen_height" != "$SCREEN" ||
            "$capture_geometry" != "$SCREEN" ||
            "$window_capture_geometry" != "${compact_width}x${compact_height}" ]]; then
            echo "Las capturas no corresponden a las geometrías esperadas: display=${screen_width}x${screen_height}; imagen=$capture_geometry; ventana=$window_capture_geometry; esperado=$SCREEN; geometría=$compact_width x $compact_height" >&2
            return 1
        fi
        if [[ ! "$compact_x" =~ ^-?[0-9]+$ || ! "$compact_y" =~ ^-?[0-9]+$ ||
            ! "$compact_width" =~ ^[0-9]+$ || ! "$compact_height" =~ ^[0-9]+$ ]] ||
            (( compact_x < 0 || compact_y < 0 || compact_x + compact_width > screen_width ||
                compact_y + compact_height > screen_height )); then
            echo "El diálogo del mapa no cabe en ${SCREEN}: $compact_geometry; display=${screen_width}x${screen_height}" >&2
            return 1
        fi
        printf 'OK: mapa cargado y completo dentro de %s: display=%s; %s\n' \
            "$SCREEN" "$capture_geometry" "$compact_geometry"
        return 0
    fi
    xdotool windowsize --sync "$MAP_WINDOW" 800 640
    sleep 0.35
    xdotool windowsize --sync "$MAP_WINDOW" 640 480
    sleep 0.2
    wait_for_tree_layout "$RUN_DIR/$tag.layout" 1
    local small_geometry small_width small_height large_geometry large_height
    small_geometry="$(xdotool getwindowgeometry --shell "$MAP_WINDOW")"
    small_width="$(awk -F= '$1 == "WIDTH" { print $2 }' <<<"$small_geometry")"
    small_height="$(awk -F= '$1 == "HEIGHT" { print $2 }' <<<"$small_geometry")"
    if [[ ! "$small_width" =~ ^[0-9]+$ || ! "$small_height" =~ ^[0-9]+$ ]] ||
        ((small_width > 640 || small_height > 480)); then
        echo "El diálogo del mapa no cabe en 640x480: $small_geometry" >&2
        return 1
    fi
    capture_screen "$CAPTURE_DIR/linux-storage-map-$tag-resized-640x480-on-${SCREEN}.png"
    xdotool windowsize --sync "$MAP_WINDOW" 800 640
    sleep 0.25
    capture_screen "$CAPTURE_DIR/linux-storage-map-$tag-screen.png"
    large_geometry="$(xdotool getwindowgeometry --shell "$MAP_WINDOW")"
    large_height="$(awk -F= '$1 == "HEIGHT" { print $2 }' <<<"$large_geometry")"
    [[ "$large_height" =~ ^[0-9]+$ ]] || {
        echo "No se pudo obtener la altura del mapa para ubicar sus acciones: $large_geometry" >&2
        return 1
    }
    # La fila de acciones permanece anclada al pie del área de contenido; su
    # coordenada se deriva del tamaño real, no de una altura absoluta antigua.
    ACTION_ROW_Y=$((large_height - 110))
    # La fila inferior contiene siempre las acciones de archivos. En la
    # disposición de dos filas no reutilizar las coordenadas de la antigua
    # fila única: el botón largo de elevación ocupa la primera fila.
    ACTION_COPY_X=65
    ACTION_MOVE_X=140
    ACTION_TRASH_X=245
    ACTION_TREE_ROW_Y=$((ACTION_ROW_Y - 50))
    xdotool mousemove --window "$MAP_WINDOW" 120 110 click 1
    sleep 0.15
}

launch_cancel_map() {
    local path="$1" tag="$2" home="$HOME_DIR"
    local state="$RUN_DIR/$2-state" data="$RUN_DIR/$2-data" config="$RUN_DIR/$2-config"
    local requested_marker="$RUN_DIR/$tag.requested"
    local cancelled_marker="$RUN_DIR/$tag.cancelled"
    mkdir -p -- "$home" "$state" "$data" "$config" "$home/.local/share/Trash"
    export GDK_BACKEND=x11 GTK_USE_PORTAL=0
    unset WAYLAND_DISPLAY WAYLAND_SOCKET
    export LTOOLS_NO_MOUNTS=1 LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0
    export LTOOLS_GUI_TREE_SMOKE=1 LTOOLS_GUI_TREE_SMOKE_HOLD_MS=45000
    export LTOOLS_GUI_TREE_CANCEL=1 LTOOLS_GUI_TREE_CANCEL_DELAY_MS=250
    export LTOOLS_GUI_TREE_PATH="$path" LTOOLS_GUI_TREE_MARKER="$RUN_DIR/$tag.marker"
    export LTOOLS_GUI_TREE_CANCEL_REQUESTED_MARKER="$requested_marker"
    export LTOOLS_GUI_TREE_CANCELLED_MARKER="$cancelled_marker"
    export LTOOLS_GUI_TREE_READY_MARKER="$RUN_DIR/$tag.ready"
    export LTOOLS_GUI_AUDIT_MARKER="$RUN_DIR/$tag.layout"
    export LTOOLS_LANG=es HOME="$home" XDG_STATE_HOME="$state" XDG_DATA_HOME="$data" XDG_CONFIG_HOME="$config"
    "$BIN" >"$RUN_DIR/$tag.log" 2>&1 &
    GUI_PID=$!
    MAP_WINDOW=""
    for _ in {1..100}; do
        MAP_WINDOW="$(xdotool search --onlyvisible --name "Mapa interactivo" 2>/dev/null | head -n1 || true)"
        [[ -n "$MAP_WINDOW" ]] && break
        sleep 0.1
    done
    if [[ -z "$MAP_WINDOW" ]]; then
        cat "$RUN_DIR/$tag.log" >&2 || true
        echo "No apareció el mapa interactivo para la cancelación ($tag)" >&2
        return 1
    fi
    for _ in {1..100}; do
        [[ -s "$cancelled_marker" ]] && break
        sleep 0.1
    done
    if [[ ! -s "$requested_marker" || ! -s "$cancelled_marker" ]]; then
        cat "$RUN_DIR/$tag.log" >&2 || true
        echo "El mapa no registró correctamente la cancelación ($tag)" >&2
        return 1
    fi
    capture_screen "$CAPTURE_DIR/linux-storage-map-$tag-screen.png"
    capture_window "$MAP_WINDOW" "$CAPTURE_DIR/linux-storage-map-$tag-window.png"
}

open_path_form() {
    local x="$1" title="$2" destination="$3" form=""
    xdotool mousemove --window "$MAP_WINDOW" "$x" "$ACTION_ROW_Y" click 1
    for _ in {1..60}; do
        form="$(xdotool search --onlyvisible --name "$title" 2>/dev/null | tail -n1 || true)"
        [[ -n "$form" ]] && break
        sleep 0.1
    done
    [[ -n "$form" ]] || { echo "No apareció el formulario: $title" >&2; return 1; }
    xdotool windowfocus --sync "$form"
    xdotool mousemove --window "$form" 350 95 click 1
    xdotool key --clearmodifiers ctrl+a
    xdotool type --clearmodifiers --delay 1 "$destination"
    xdotool mousemove --window "$form" 435 210 click 1
    sleep 0.65
}

confirm_yes() {
    # El diálogo GTK es una ventana top-level separada; getwindowfocus puede
    # devolver la raíz cuando Xvfb no tiene gestor de ventanas. Se encuentra el
    # diálogo entre las ventanas visibles sin título y se omiten la raíz, el
    # mapa y las ventanas diminutas auxiliares.
    local screen_width screen_height dialog title geometry width height click_x click_y candidate candidate_area dialog_area
    read -r screen_width screen_height < <(xdotool getdisplaygeometry)
    dialog=""
    dialog_area=0
    while IFS= read -r candidate; do
        [[ "$candidate" != "$MAP_WINDOW" ]] || continue
        title="$(xdotool getwindowname "$candidate" 2>/dev/null || true)"
        [[ -z "$title" ]] || continue
        geometry="$(xdotool getwindowgeometry --shell "$candidate" 2>/dev/null || true)"
        width="$(awk -F= '$1 == "WIDTH" { print $2 }' <<<"$geometry")"
        height="$(awk -F= '$1 == "HEIGHT" { print $2 }' <<<"$geometry")"
        if [[ "$width" =~ ^[0-9]+$ && "$height" =~ ^[0-9]+$ ]] &&
            (( width >= 200 && height >= 100 && width < screen_width && height < screen_height )); then
            # Con un gestor de ventanas también aparecen ventanas GTK padre
            # sin nombre (por ejemplo, la ventana principal). No asumir que el
            # primer top-level sin título es el modal: elegir el candidato
            # visible más pequeño excluye raíz/padre y conserva el diálogo.
            candidate_area=$((width * height))
            if [[ -z "$dialog" ]] || (( candidate_area < dialog_area )); then
                dialog="$candidate"
                dialog_area="$candidate_area"
            fi
        fi
    done < <(xdotool search --onlyvisible --name '.*' 2>/dev/null || true)
    [[ -n "$dialog" ]] || { echo 'No se identificó la ventana del diálogo de confirmación' >&2; return 1; }
    geometry="$(xdotool getwindowgeometry --shell "$dialog")"
    width="$(awk -F= '$1 == "WIDTH" { print $2 }' <<<"$geometry")"
    height="$(awk -F= '$1 == "HEIGHT" { print $2 }' <<<"$geometry")"
    [[ "$width" =~ ^[0-9]+$ && "$height" =~ ^[0-9]+$ ]] || {
        echo "No se pudo leer la geometría del diálogo de confirmación: $geometry" >&2
        return 1
    }
    (( width >= 120 && height >= 60 )) || {
        echo "Ventana enfocada no parece el diálogo de confirmación: $geometry" >&2
        return 1
    }
    click_x=$((width * 3 / 4))
    click_y=$((height - 22))
    xdotool windowraise "$dialog"
    xdotool mousemove --sync --window "$dialog" "$click_x" "$click_y" click 1
    sleep 0.3
    capture_screen "$CAPTURE_DIR/linux-storage-map-${CONFIRM_ACTION}-after-confirm.png"
}

wait_for_file() {
    local path="$1"
    for _ in {1..100}; do
        [[ -f "$path" ]] && return 0
        sleep 0.1
    done
    echo "No se creó el resultado GUI esperado: $path" >&2
    return 1
}

# La vista debe permitir expandir y contraer una carpeta individual, además de
# ofrecer navegación por teclado y controles globales. El marcador se
# emite desde las señales reales de GtkTreeView, no desde el propio test.
launch_map "$TREE_ROOT" map-tree-expansion
if (( LAYOUT_ONLY )); then
    close_gui
    echo "OK: layout compacto del mapa validado en ${SCREEN}"
    exit 0
fi

# Pulsa la flecha triangular de la primera carpeta para verificar la
# interacción gráfica individual, no solo la navegación por teclado.
xdotool mousemove --sync --window "$MAP_WINDOW" 18 110 click 1
for _ in {1..30}; do
    grep -Fq 'STORAGE_TREE_ROW_EXPANDED' "$RUN_DIR/map-tree-expansion.layout" && break
    sleep 0.05
done
grep -Fq 'STORAGE_TREE_ROW_EXPANDED' "$RUN_DIR/map-tree-expansion.layout" || {
    echo 'La flecha triangular no expandió la carpeta seleccionada del mapa' >&2
    exit 41
}
capture_screen "$CAPTURE_DIR/linux-storage-map-tree-expanded-es.png"
xdotool mousemove --sync --window "$MAP_WINDOW" 18 110 click 1
for _ in {1..30}; do
    [[ "$(grep -Fc 'STORAGE_TREE_ROW_COLLAPSED' "$RUN_DIR/map-tree-expansion.layout" || true)" -ge 1 ]] && break
    sleep 0.05
done
[[ "$(grep -Fc 'STORAGE_TREE_ROW_COLLAPSED' "$RUN_DIR/map-tree-expansion.layout" || true)" -ge 1 ]] || {
    echo 'La flecha triangular no contrajo la carpeta seleccionada del mapa' >&2
    exit 42
}
capture_screen "$CAPTURE_DIR/linux-storage-map-tree-collapsed-es.png"

# También se conservan las flechas de teclado accesibles de GtkTreeView.
xdotool windowfocus --sync "$MAP_WINDOW"
xdotool mousemove --sync --window "$MAP_WINDOW" 120 110 click 1
xdotool key --clearmodifiers Right
for _ in {1..30}; do
    [[ "$(grep -Fc 'STORAGE_TREE_ROW_EXPANDED' "$RUN_DIR/map-tree-expansion.layout" || true)" -ge 2 ]] && break
    sleep 0.05
done
[[ "$(grep -Fc 'STORAGE_TREE_ROW_EXPANDED' "$RUN_DIR/map-tree-expansion.layout" || true)" -ge 2 ]] || {
    echo 'La flecha derecha no expandió la carpeta seleccionada del mapa' >&2
    focus_window="$(xdotool getwindowfocus 2>/dev/null || true)"
    printf 'Diagnóstico: mapa=%s foco=%s título_foco=%s ratón=%s\n' \
        "$MAP_WINDOW" "$focus_window" "$(xdotool getwindowname "$focus_window" 2>/dev/null || true)" \
        "$(xdotool getmouselocation --shell 2>/dev/null | tr '\n' ' ' || true)" >&2
    exit 43
}
xdotool key --clearmodifiers Left
for _ in {1..30}; do
    [[ "$(grep -Fc 'STORAGE_TREE_ROW_COLLAPSED' "$RUN_DIR/map-tree-expansion.layout" || true)" -ge 2 ]] && break
    sleep 0.05
done
[[ "$(grep -Fc 'STORAGE_TREE_ROW_COLLAPSED' "$RUN_DIR/map-tree-expansion.layout" || true)" -ge 2 ]] || {
    echo 'La flecha izquierda no contrajo la carpeta seleccionada del mapa' >&2
    exit 44
}
xdotool mousemove --window "$MAP_WINDOW" 55 "$ACTION_TREE_ROW_Y" click 1
for _ in {1..30}; do
    grep -Fq 'STORAGE_TREE_EXPAND_ALL' "$RUN_DIR/map-tree-expansion.layout" && break
    sleep 0.05
done
grep -Fq 'STORAGE_TREE_EXPAND_ALL' "$RUN_DIR/map-tree-expansion.layout" || {
    echo 'El botón Expandir todo no activó el árbol' >&2
    exit 45
}
xdotool mousemove --window "$MAP_WINDOW" 185 "$ACTION_TREE_ROW_Y" click 1
for _ in {1..30}; do
    grep -Fq 'STORAGE_TREE_COLLAPSE_ALL' "$RUN_DIR/map-tree-expansion.layout" && break
    sleep 0.05
done
grep -Fq 'STORAGE_TREE_COLLAPSE_ALL' "$RUN_DIR/map-tree-expansion.layout" || {
    echo 'El botón Colapsar todo no activó el árbol' >&2
    exit 46
}
close_gui
echo 'OK: mapa GUI: flechas triangulares, navegación por teclado y expansión global'

# Cancelación: el fixture tiene muchas entradas y el retardo controlado solo
# para E2E mantiene el escaneo activo el tiempo suficiente para pulsar el
# botón GTK real. Se verifican tanto la petición como el estado final.
CANCEL_ROOT="$RUN_DIR/cancel-root"
mkdir -p -- "$CANCEL_ROOT"
for index in $(seq 1 240); do
    mkdir -p -- "$CANCEL_ROOT/group-$index/nested"
    printf 'cancel fixture %s\n' "$index" >"$CANCEL_ROOT/group-$index/nested/item.txt"
done
launch_cancel_map "$CANCEL_ROOT" map-cancel
grep -Fq 'STORAGE_TREE_CANCEL_REQUESTED' "$RUN_DIR/map-cancel.layout" || {
    echo 'La auditoría GUI no registró la petición de cancelación del mapa' >&2
    exit 47
}
grep -Fq 'STORAGE_TREE_CANCELLED' "$RUN_DIR/map-cancel.layout" || {
    echo 'La auditoría GUI no registró la finalización cancelada del mapa' >&2
    exit 48
}
close_gui
echo 'OK: mapa GUI: cancelación durante el escaneo y estado final verificables'

# Copiar: Enter debe cancelar; solo un clic explícito en Sí ejecuta la acción.
launch_map "$SOURCE" map-copy
# La fila inferior contiene Copiar; el test usa la coordenada que publica el
# layout estable de dos filas y no depende del ancho del texto de los botones.
open_path_form "$ACTION_COPY_X" "Copiar ruta seleccionada" "$COPY"
capture_screen "$CAPTURE_DIR/linux-storage-map-confirm-no-es.png"
xdotool key Return
sleep 0.3
[[ ! -e "$COPY" ]] || { echo "Enter no respetó la negativa predeterminada" >&2; exit 31; }
open_path_form "$ACTION_COPY_X" "Copiar ruta seleccionada" "$COPY"
capture_screen "$CAPTURE_DIR/linux-storage-map-confirm-yes-es.png"
CONFIRM_ACTION=copy
confirm_yes
wait_for_file "$COPY"
cmp -s "$SOURCE" "$COPY"
close_gui

# Mover en una sesión limpia para que ningún modal de resultado intercepte la
# siguiente pulsación; se comprueba la desaparición del origen y los bytes.
launch_map "$SOURCE" map-move
xdotool mousemove --window "$MAP_WINDOW" "$ACTION_MOVE_X" "$ACTION_ROW_Y" click 1
form=""
for _ in {1..60}; do
    form="$(xdotool search --onlyvisible --name "Mover ruta seleccionada" 2>/dev/null | tail -n1 || true)"
    [[ -n "$form" ]] && break
    sleep 0.1
done
[[ -n "$form" ]] || { echo "No apareció el formulario Mover" >&2; exit 32; }
xdotool windowfocus --sync "$form"
xdotool mousemove --window "$form" 350 95 click 1
xdotool key --clearmodifiers ctrl+a
xdotool type --clearmodifiers --delay 1 "$MOVED"
xdotool mousemove --window "$form" 435 210 click 1
sleep 0.65
CONFIRM_ACTION=move
confirm_yes
wait_for_file "$MOVED"
[[ ! -e "$SOURCE" ]]
cmp -s "$COPY" "$MOVED"
close_gui

echo "OK: mapa GUI: No predeterminado, Copiar y Mover"

# Papelera: usa un XDG_DATA_HOME aislado; nunca toca la papelera real.
if command -v gio >/dev/null || command -v trash-put >/dev/null; then
    launch_map "$MOVED" map-trash
    xdotool mousemove --window "$MAP_WINDOW" "$ACTION_TRASH_X" "$ACTION_ROW_Y" click 1
    sleep 0.65
    capture_screen "$CAPTURE_DIR/linux-storage-map-confirm-trash-es.png"
    CONFIRM_ACTION=trash
    confirm_yes
    for _ in {1..100}; do [[ ! -e "$MOVED" ]] && break; sleep 0.1; done
    [[ ! -e "$MOVED" ]]
    TRASHED="$(find "$RUN_DIR/map-trash-data" "$HOME_DIR/.local/share/Trash" \
        -type f -name "$(basename "$MOVED")" -print -quit 2>/dev/null || true)"
    [[ -n "$TRASHED" && -f "$TRASHED" ]]
    grep -Fq "LTools GUI map action fixture" "$TRASHED"
    close_gui
    echo "OK: papelera del mapa GUI en XDG_DATA_HOME aislado"
else
    if (( REQUIRE_GUI )); then
        echo 'ERROR: se necesita gio o trash-put para probar el botón de papelera del mapa' >&2
        exit 2
    fi
    echo "SKIP: papelera GUI sin gio/trash-put"
fi
