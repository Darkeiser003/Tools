#!/usr/bin/env bash

# wine-prefix-manager.sh - guarded Wine prefix creation and migration.
# It does not modify anything unless the user explicitly selects an action.

set -uo pipefail
export LC_ALL=C

VERSION="0.3.0"
SCRIPT_NAME="$(basename "$0")"
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd -P)"
HOME_DIR="${HOME:-$(getent passwd "$(id -u)" 2>/dev/null | cut -d: -f6)}"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib/ltools-plan.sh"
ACTION=""
SOURCE=""
DESTINATION=""
ARCH="win64"
REWRITE_CONFIGS=0
REMOVE_SOURCE=0
ALLOW_STEAM=0
ALLOW_MOUNT_ROOT=0
FORCE=0
SELECT_MODE="all"
INCLUDE_ITEMS=()
EXCLUDE_ITEMS=()
SET_DEFAULTS=0
UPDATE_LAUNCHERS=0
ACTIVATE_SHELL=0
BATCH_MODE=0
ROLLBACK_PATH=""
SOURCE_TYPE=""
SOURCE_MOUNT_ROOT=0
SELECTED_ITEMS=()
SELECTION_PARTIAL=0

usage() {
    cat <<EOF
Uso: $SCRIPT_NAME ACCIÓN [opciones]

Acciones:
  create                    Crea un prefijo vacío usando wineboot.
  clone                     Copia un prefijo completo conservando su registro.
  migrate                   Alias de clone; permite actualizar referencias y retirar el origen.

Opciones:
  --source RUTA             Prefijo Wine origen para clone/migrate.
  --dest RUTA               Ruta del nuevo prefijo.
  --arch win32|win64        Arquitectura para create (por defecto: win64).
  --select                  Elige interactivamente qué elementos de primer nivel copiar.
  --include NOMBRE          Copia solo este elemento de primer nivel (repetible).
  --exclude NOMBRE          Excluye este elemento de primer nivel (repetible).
  --rewrite-configs         Actualiza referencias de texto al origen tras verificar la copia.
  --set-defaults            Genera una configuración de usuario para Wine/winetricks/Proton.
  --update-launchers        Actualiza defaults compatibles de Heroic y crea copias de seguridad.
  --activate-shell          Añade esa configuración a .bashrc/.zshrc tras confirmación explícita.
  --remove-source           Mueve el origen a la papelera tras verificar la copia.
  --allow-steam             Permite operar sobre compatdata de Steam, bajo confirmación.
  --allow-mount-root        Recupera un prefijo situado en la raíz de un disco, solo con --include.
  --force                   Continúa pese a referencias o advertencias no críticas.
  --batch-mode              Usa confirmaciones Y/N sencillas para el gestor unificado.
  --dry-run                 Muestra y registra el plan sin copiar ni modificar nada.
  --plan FICHERO            Guarda el plan en una ruta concreta.
  --rollback FICHERO        Revierte las operaciones reversibles de un plan.
  -h, --help                Muestra esta ayuda.

Ejemplos:
  $SCRIPT_NAME create --dest "$HOME/Games/NuevoPrefix"
  $SCRIPT_NAME migrate --source "$HOME/.wine" --dest /mnt/JuegosLinux/prefixes/wine-main
  $SCRIPT_NAME migrate --source "$HOME/Games/ea-app" \\
      --dest /mnt/JuegosLinux/Lutrs/ea-app --rewrite-configs
  $SCRIPT_NAME migrate --source "$HOME/.wine" \\
      --dest /mnt/JuegosLinux/prefixes/wine-main --select
  $SCRIPT_NAME migrate --source "$HOME/.wine" \\
      --dest /mnt/JuegosLinux/prefixes/wine-main \\
      --include drive_c --include system.reg --set-defaults

Por defecto se copia el prefijo completo. Con --select, --include o --exclude se
pueden escoger elementos de primer nivel. Los juegos instalados fuera del prefijo
no se copian automáticamente.
EOF
}

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 2
}

have() {
    command -v "$1" >/dev/null 2>&1
}

confirm() {
    local answer
    printf '%s [y/N] ' "$1"
    read -r answer || return 1
    [[ "$answer" =~ ^([yY][eE][sS]|[yY]|[sS]|[sS][iI])$ ]]
}

ask_phrase() {
    local expected="$1" answer prompt
    if (( BATCH_MODE )); then
        case "$expected" in
            CERRADO) prompt='¿Has cerrado Wine, Steam, Lutris y Heroic y quieres continuar?' ;;
            PROCESO-CERRADO) prompt='¿Has cerrado el proceso que usa este prefijo y quieres continuar?' ;;
            CONTINUAR-FS) prompt='El destino usa un sistema de archivos con posibles limitaciones. ¿Continuar?' ;;
            CONTINUAR-PARCIAL) prompt='La selección puede no ser un prefijo completo. ¿Continuar?' ;;
            COPIA-CORRECTA) prompt='La copia coincide con el origen. ¿Continuar con las acciones posteriores?' ;;
            ACTUALIZAR-REFERENCIAS) prompt='¿Actualizar las referencias de las aplicaciones?' ;;
            RETIRAR-OMITIDOS) prompt='La migración fue parcial. ¿Retirar aun así el origen?' ;;
            RETIRAR-ORIGEN) prompt='¿Mover el origen verificado a la papelera?' ;;
            RAIZ-MONTAJE) prompt='¿Autorizar el rescate selectivo de la raíz del disco?' ;;
            *) prompt='¿Continuar con este prefijo?' ;;
        esac
        confirm "$prompt"
        return $?
    fi
    printf 'Escribe "%s" para confirmar: ' "$expected"
    read -r answer || return 1
    [[ "$answer" == "$expected" ]]
}

safe_name() {
    printf '%s' "$1" | sed 's#[^A-Za-z0-9_.-]#_#g'
}

parse_args() {
    [[ $# -gt 0 ]] || { usage; exit 0; }
    case "$1" in
        --version) printf '%s %s\n' "$SCRIPT_NAME" "$VERSION"; exit 0 ;;
        create|clone|migrate) ACTION="$1"; shift ;;
        -h|--help) usage; exit 0 ;;
        *) die "acción desconocida: $1" ;;
    esac
    while (($#)); do
        case "$1" in
            --source)
                [[ $# -ge 2 ]] || die "--source requiere una ruta"
                SOURCE="$2"; shift 2 ;;
            --dest)
                [[ $# -ge 2 ]] || die "--dest requiere una ruta"
                DESTINATION="$2"; shift 2 ;;
            --arch)
                [[ $# -ge 2 && ( "$2" == win32 || "$2" == win64 ) ]] || die "--arch debe ser win32 o win64"
                ARCH="$2"; shift 2 ;;
            --select) SELECT_MODE="interactive"; shift ;;
            --include)
                [[ $# -ge 2 ]] || die "--include requiere un nombre"
                INCLUDE_ITEMS+=("$2"); shift 2 ;;
            --exclude)
                [[ $# -ge 2 ]] || die "--exclude requiere un nombre"
                EXCLUDE_ITEMS+=("$2"); shift 2 ;;
            --rewrite-configs) REWRITE_CONFIGS=1; shift ;;
            --set-defaults) SET_DEFAULTS=1; shift ;;
            --update-launchers) UPDATE_LAUNCHERS=1; shift ;;
            --activate-shell) ACTIVATE_SHELL=1; SET_DEFAULTS=1; shift ;;
            --remove-source) REMOVE_SOURCE=1; shift ;;
            --allow-steam) ALLOW_STEAM=1; shift ;;
            --allow-mount-root) ALLOW_MOUNT_ROOT=1; shift ;;
            --force) FORCE=1; shift ;;
            --batch-mode) BATCH_MODE=1; shift ;;
            --dry-run) DRY_RUN=1; shift ;;
            --version) printf '%s %s\n' "$SCRIPT_NAME" "$VERSION"; exit 0 ;;
            --plan)
                [[ $# -ge 2 ]] || die "--plan requiere un fichero"
                PLAN_PATH="$2"; shift 2 ;;
            --rollback)
                [[ $# -ge 2 ]] || die "--rollback requiere un fichero"
                ROLLBACK_PATH="$2"; shift 2 ;;
            -h|--help) usage; exit 0 ;;
            *) die "opción desconocida: $1" ;;
        esac
    done
}

require_destination() {
    [[ -n "$DESTINATION" ]] || die "falta --dest"
    DESTINATION="$(realpath -m -- "$DESTINATION" 2>/dev/null || true)"
    [[ -n "$DESTINATION" ]] || die "no se pudo resolver la ruta destino"
    case "$DESTINATION" in
        /|/home|/mnt|/media|/run/media|/usr|/opt|/var|/etc|/boot|"$HOME_DIR")
            die "la ruta destino es demasiado amplia o crítica: $DESTINATION" ;;
    esac
    if [[ -e "$DESTINATION" ]]; then
        [[ -d "$DESTINATION" ]] || die "el destino existe y no es un directorio: $DESTINATION"
        [[ -z "$(find "$DESTINATION" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]] || die "el destino debe estar vacío: $DESTINATION"
    fi
}

prefix_type() {
    local path="$1"
    case "$path" in
        */files/share/default_pfx) printf 'runner-default' ;;
        */steamapps/compatdata/*/pfx) printf 'steam-proton' ;;
        */.wine) printf 'default-wine' ;;
        */lutris/*|*/Lutris/*|*/Lutrs/*|*/.local/share/lutris/*) printf 'lutris-prefix' ;;
        */Heroic/*|*/heroic/*|*/.config/heroic/*) printf 'heroic-prefix' ;;
        */bottles/*|*/.var/app/com.usebottles.bottles/*) printf 'bottles-prefix' ;;
        *) printf 'wine-prefix' ;;
    esac
}

validate_source() {
    local resolved type mountpoint
    [[ -n "$SOURCE" ]] || die "falta --source"
    resolved="$(realpath -e -- "$SOURCE" 2>/dev/null || true)"
    [[ -n "$resolved" && -d "$resolved" ]] || die "el origen no existe o no es un directorio: $SOURCE"
    SOURCE="$resolved"
    [[ -f "$SOURCE/system.reg" || -d "$SOURCE/drive_c" ]] || die "no parece un prefijo Wine: falta system.reg o drive_c"
    if have findmnt; then
        mountpoint="$(findmnt -rn -T "$SOURCE" -o TARGET 2>/dev/null | head -1)"
        if [[ "$mountpoint" == "$SOURCE" ]]; then
            SOURCE_MOUNT_ROOT=1
            (( ALLOW_MOUNT_ROOT )) || die "el origen es la raíz de un disco; usa --allow-mount-root solo para rescatar componentes del prefijo"
            ((${#INCLUDE_ITEMS[@]} > 0)) || die "un prefijo en la raíz del disco solo puede rescatarse con --include explícitos"
            (( REMOVE_SOURCE == 0 )) || die "nunca se permite retirar la raíz completa de un disco"
            (( REWRITE_CONFIGS == 0 )) || die "no se pueden reescribir referencias globales al punto de montaje"
            ask_phrase "RAIZ-MONTAJE" || die "operación cancelada: no se ha autorizado la recuperación selectiva"
        fi
    fi
    case "$SOURCE" in
        */files/share/default_pfx|*/files/share/default_pfx/*|*/.Trash-*/*|*/Trash/*)
            die "se bloquean plantillas compartidas de Proton y prefijos dentro de la papelera" ;;
    esac
    SOURCE_TYPE="$(prefix_type "$SOURCE")"
    printf 'Tipo detectado: %s\n' "$SOURCE_TYPE"
    if [[ "$SOURCE_TYPE" == runner-default ]]; then
        die "se bloquean las plantillas default_pfx compartidas de Proton/Wine"
    fi
    if [[ "$SOURCE_TYPE" == steam-proton && "$ALLOW_STEAM" != 1 ]]; then
        die "es un prefijo compatdata de Steam; usa --allow-steam si realmente quieres operarlo"
    fi
    if [[ "$SOURCE" == "$DESTINATION" ]]; then
        die "origen y destino son la misma ruta"
    fi
    if (( SOURCE_MOUNT_ROOT )); then
        local item
        for item in "${INCLUDE_ITEMS[@]}"; do
            case "$DESTINATION/" in
                "$SOURCE/$item"/*) die "el destino está dentro del componente que se intenta rescatar: $item" ;;
            esac
        done
    else
        case "$DESTINATION/" in
            "$SOURCE"/*) die "el destino está dentro del origen" ;;
        esac
    fi
}

check_locks() {
    local lock_files process_lines
    lock_files="$(find "$SOURCE" -maxdepth 4 -type f \( -name '*.lock' -o -name '*.lck' -o -name lock \) -print 2>/dev/null | head -20)"
    if [[ -n "$lock_files" ]]; then
        printf 'Se detectaron posibles bloqueos:\n%s\n' "$lock_files"
        (( FORCE )) || { ask_phrase "CERRADO" || die "cierra Wine, Steam, Lutris y Heroic antes de continuar"; }
    fi
    process_lines="$(ps -eo pid=,comm=,args= 2>/dev/null | awk -v self="$$" -v source="$SOURCE" '
        $1 != self && $2 ~ /^(wine|wine64|wineserver|wineboot|proton|steam|lutris|heroic|bottles|bwrap)$/ && index($0, source) {
            sub(/^[[:space:]]*[0-9]+[[:space:]]+[^[:space:]]+[[:space:]]+/, "")
            print
        }
    ' )"
    if [[ -n "$process_lines" ]]; then
        printf 'Hay un proceso que menciona el origen:\n'
        printf '%s\n' "$process_lines"
        (( FORCE )) || { ask_phrase "PROCESO-CERRADO" || die "cierra el proceso antes de continuar"; }
    fi
}

size_bytes() {
    du -sx --block-size=1 -- "$1" 2>/dev/null | awk 'NR==1{print $1}'
}

selected_size_bytes() {
    local item bytes total=0
    for item in "${SELECTED_ITEMS[@]}"; do
        bytes="$(size_bytes "$SOURCE/$item")"
        [[ "$bytes" =~ ^[0-9]+$ ]] || die "no se pudo calcular el tamaño de $SOURCE/$item"
        total=$((total + bytes))
    done
    printf '%s' "$total"
}

existing_parent() {
    local path="$1"
    while [[ ! -d "$path" && "$path" != / ]]; do
        path="$(dirname "$path")"
    done
    printf '%s' "$path"
}

check_space() {
    local parent source_bytes available fstype
    parent="$(existing_parent "$(dirname "$DESTINATION")")"
    source_bytes="$(selected_size_bytes)"
    available="$(df -P -B1 -- "$parent" 2>/dev/null | awk 'NR==2{print $4}')"
    [[ "$source_bytes" =~ ^[0-9]+$ && "$available" =~ ^[0-9]+$ ]] || die "no se pudo calcular espacio libre"
    printf 'Tamaño del origen: %s\n' "$(numfmt --to=iec --suffix=B "$source_bytes" 2>/dev/null || printf '%s B' "$source_bytes")"
    printf 'Espacio disponible: %s\n' "$(numfmt --to=iec --suffix=B "$available" 2>/dev/null || printf '%s B' "$available")"
    (( available > source_bytes )) || die "no hay espacio suficiente en el destino"
    if have findmnt; then
        fstype="$(findmnt -rn -T "$parent" -o FSTYPE 2>/dev/null | head -1)"
        case "$fstype" in
            ntfs|ntfs3|fuseblk|exfat|vfat)
                printf 'Advertencia: el destino usa %s; Wine funciona mejor sobre ext4, btrfs u otro FS Linux.\n' "$fstype"
                (( FORCE )) || ask_phrase "CONTINUAR-FS" || die "operación cancelada por el tipo de sistema de archivos"
                ;;
        esac
    fi
}

item_is_selected() {
    local wanted="$1" item
    for item in "${SELECTED_ITEMS[@]}"; do
        [[ "$item" == "$wanted" ]] && return 0
    done
    return 1
}

validate_item_name() {
    local item="$1"
    [[ -n "$item" && "$item" != . && "$item" != .. && "$item" != /* && "$item" != */* ]] || die "elemento no válido: $item"
    [[ -e "$SOURCE/$item" || -L "$SOURCE/$item" ]] || die "no existe en el origen: $item"
}

select_items() {
    local -a top_level=() choices=()
    local item choice index
    mapfile -d '' -t top_level < <(find "$SOURCE" -mindepth 1 -maxdepth 1 -printf '%f\0' 2>/dev/null | sort -z)
    ((${#top_level[@]})) || die "el origen no contiene elementos copiables"

    if (( SOURCE_MOUNT_ROOT )); then
        [[ "$SELECT_MODE" == all ]] || die "un prefijo en la raíz del disco no admite selección interactiva; usa --include"
        local allowed
        for item in "${INCLUDE_ITEMS[@]}"; do
            case "$item" in
                drive_c|system.reg|user.reg|userdef.reg|dosdevices|.update-timestamp) ;;
                *) die "elemento no permitido para rescate desde la raíz del disco: $item" ;;
            esac
        done
        SELECTED_ITEMS=("${INCLUDE_ITEMS[@]}")
    elif ((${#INCLUDE_ITEMS[@]})); then
        SELECTED_ITEMS=("${INCLUDE_ITEMS[@]}")
    elif [[ "$SELECT_MODE" == interactive ]]; then
        printf '\nElementos de primer nivel del prefijo:\n'
        for index in "${!top_level[@]}"; do
            printf '  %2d) %-24s %s\n' "$((index + 1))" "${top_level[index]}" \
                "$(numfmt --to=iec --suffix=B "$(size_bytes "$SOURCE/${top_level[index]}")" 2>/dev/null || printf '?')"
        done
        printf 'Escribe números separados por espacios, "all" para todo o "q" para cancelar: '
        read -r -a choices || die "operación cancelada"
        [[ "${choices[0]:-}" != q ]] || die "operación cancelada"
        if [[ "${choices[0]:-}" == all ]]; then
            SELECTED_ITEMS=("${top_level[@]}")
        else
            for choice in "${choices[@]}"; do
                [[ "$choice" =~ ^[0-9]+$ ]] || die "selección no válida: $choice"
                index=$((choice - 1))
                (( index >= 0 && index < ${#top_level[@]} )) || die "número fuera de rango: $choice"
                SELECTED_ITEMS+=("${top_level[index]}")
            done
        fi
    else
        SELECTED_ITEMS=("${top_level[@]}")
    fi

    for item in "${EXCLUDE_ITEMS[@]}"; do
        validate_item_name "$item"
        local -a filtered=()
        for choice in "${SELECTED_ITEMS[@]}"; do
            [[ "$choice" == "$item" ]] || filtered+=("$choice")
        done
        SELECTED_ITEMS=("${filtered[@]}")
    done
    ((${#SELECTED_ITEMS[@]})) || die "la selección no contiene ningún elemento"
    local -a unique=()
    local duplicate
    for item in "${SELECTED_ITEMS[@]}"; do
        validate_item_name "$item"
        duplicate=0
        for choice in "${unique[@]}"; do
            [[ "$choice" == "$item" ]] && duplicate=1
        done
        (( duplicate == 0 )) && unique+=("$item")
    done
    SELECTED_ITEMS=("${unique[@]}")
    ((${#SELECTED_ITEMS[@]})) || die "la selección no contiene ningún elemento"

    if ((${#SELECTED_ITEMS[@]} != ${#top_level[@]})); then
        SELECTION_PARTIAL=1
    fi

    if ! item_is_selected system.reg || ! item_is_selected drive_c; then
        printf '\nAdvertencia: la selección no incluye system.reg y/o drive_c.\n'
        printf 'El resultado puede no ser un prefijo Wine funcional; los elementos omitidos seguirán en el origen.\n'
        (( FORCE )) || ask_phrase "CONTINUAR-PARCIAL" || die "operación cancelada: selecciona un prefijo completo o confirma la copia parcial"
    fi
    printf '\nElementos seleccionados (%d):\n' "${#SELECTED_ITEMS[@]}"
    printf '  %s\n' "${SELECTED_ITEMS[@]}"
}

copy_prefix() {
    printf '\nSe copiarán los elementos seleccionados:\n  Origen:  %s\n  Destino: %s\n' "$SOURCE" "$DESTINATION"
    printf 'La copia se verificará antes de cualquier acción posterior.\n'
    if (( SET_DEFAULTS )); then
        printf 'Después de verificar: se generarán las rutas predeterminadas de Wine/winetricks%s.\n' \
            "$([[ "$ACTIVATE_SHELL" == 1 ]] && printf ' y se ofrecerá activarlas en la shell' || true)"
    fi
    if (( REWRITE_CONFIGS )); then
        printf 'Después de verificar: se revisarán referencias de Lutris/Heroic/UMU y se hará copia de seguridad antes de actualizarlas.\n'
    fi
    if (( UPDATE_LAUNCHERS )); then
        printf 'Después de verificar: se actualizarán los defaults globales compatibles de Heroic, con copia de seguridad.\n'
        printf 'Lutris, UMU y Steam se mantienen por juego o por ejecución; no se sobrescribirá un default inexistente.\n'
    fi
    if (( REMOVE_SOURCE )); then
        printf 'Después de verificar: se ofrecerá mover el origen a la papelera; nunca se borra directamente.\n'
    fi
    if (( DRY_RUN )); then
        plan_record copy-prefix "$DESTINATION" planned yes "$SOURCE" "${SELECTED_ITEMS[*]}"
        printf 'Simulación: se copiaría el prefijo seleccionado y se verificaría con rsync.\n'
        return 0
    fi
    have rsync || die "rsync es necesario para migrar un prefijo de forma verificable"
    ask_phrase "MIGRAR $(basename "$SOURCE")" || die "operación cancelada"
    mkdir -p -- "$DESTINATION" || die "no se pudo crear el destino"
    local item differences
    for item in "${SELECTED_ITEMS[@]}"; do
        printf '\nCopiando: %s\n' "$item"
        rsync -aH --info=progress2 --partial -- "$SOURCE/$item" "$DESTINATION/" || die "la copia ha fallado en $item"
    done
    printf '\nVerificando que el destino coincide con el origen...\n'
    for item in "${SELECTED_ITEMS[@]}"; do
        differences="$(rsync -aHn --delete --itemize-changes -- "$SOURCE/$item" "$DESTINATION/" 2>/dev/null)"
        if [[ -n "$differences" ]]; then
            printf 'La verificación encontró diferencias en %s:\n%s\n' "$item" "$differences"
            die "no se modificará el origen mientras la copia no sea idéntica"
        fi
    done
    [[ -f "$DESTINATION/system.reg" || -d "$DESTINATION/drive_c" ]] || die "el destino no parece un prefijo después de copiar"
    printf 'Copia verificada correctamente.\n'
    printf 'La copia coincide con el origen; se pueden ejecutar ahora las acciones posteriores seleccionadas.\n'
    plan_record copy-prefix "$DESTINATION" executed yes "$SOURCE" "${SELECTED_ITEMS[*]}"
    plan_record remove-created "$DESTINATION" executed yes "" "copia migrada"
}

update_heroic_file() {
    local file="$1" parent prefix tmp stamp
    parent="$(dirname -- "$DESTINATION")"
    prefix="$DESTINATION"
    [[ -f "$file" ]] || return 0
    if (( DRY_RUN )); then
        plan_record update-heroic "$file" planned yes "se creará backup antes de actualizar" "$prefix"
        printf 'Simulación: se actualizarían las referencias compatibles de Heroic en: %s\n' "$file"
        return 0
    fi
    if ! have jq; then
        if [[ "$parent" == *'"'* || "$parent" == *'\\'* || "$prefix" == *'"'* || "$prefix" == *'\\'* ]]; then
            printf 'Heroic detectado, pero la ruta contiene caracteres que requieren jq; no se modificará %s.\n' "$file"
            return 0
        fi
        stamp="$(date +%Y%m%d-%H%M%S)"
        cp -a -- "$file" "$file.bak-$stamp" || die "no se pudo respaldar $file"
        if HEROIC_PARENT="$parent" HEROIC_PREFIX="$prefix" perl -0pi -e '
            my $parent = $ENV{HEROIC_PARENT};
            my $prefix = $ENV{HEROIC_PREFIX};
            s/("defaultWinePrefix"\s*:\s*")[^"]*(")/$1 . $parent . $2/ge;
            s/("defaultWinePrefixDir"\s*:\s*")[^"]*(")/$1 . $parent . $2/ge;
            s/("winePrefix"\s*:\s*")[^"]*(")/$1 . $prefix . $2/ge;
        ' "$file"; then
            printf 'Heroic actualizado (modo compatible): %s\n' "$file"
            plan_record restore-file "$file" executed yes "$file.bak-$stamp" "Heroic"
        else
            printf 'No se pudo actualizar Heroic; el backup se conserva: %s.bak-%s\n' "$file" "$stamp"
        fi
        return 0
    fi
    tmp="$(mktemp "${file}.tmp.XXXXXX")" || die "no se pudo crear un temporal para Heroic"
    if ! jq --arg prefix "$prefix" --arg parent "$parent" '
        def set_prefix_defaults:
            .defaultWinePrefix = $parent
            | .defaultWinePrefixDir = $parent
            | .winePrefix = $prefix;
        if has("defaultSettings") then .defaultSettings |= set_prefix_defaults
        elif has("settings") then .settings |= set_prefix_defaults
        elif has("winePrefix") or has("defaultWinePrefix") then set_prefix_defaults
        else . end
    ' "$file" >"$tmp"; then
        rm -f -- "$tmp"
        printf 'No se pudo interpretar Heroic; se conserva %s sin cambios.\n' "$file"
        return 0
    fi
    stamp="$(date +%Y%m%d-%H%M%S)"
    cp -a -- "$file" "$file.bak-$stamp" || die "no se pudo respaldar $file"
    mv -- "$tmp" "$file" || die "no se pudo instalar la configuración actualizada de Heroic"
    plan_record restore-file "$file" executed yes "$file.bak-$stamp" "Heroic"
    printf 'Heroic actualizado: %s\n' "$file"
    printf '  defaultWinePrefix/defaultWinePrefixDir: %s\n' "$parent"
    printf '  winePrefix compartido: %s\n' "$prefix"
}

update_launcher_defaults() {
    local heroic_found=0
    printf '\nActualizando defaults internos de lanzadores compatibles...\n'
    for file in \
        "$HOME_DIR/.config/heroic/config.json" \
        "$HOME_DIR/.config/heroic/store/config.json"; do
        [[ -f "$file" ]] || continue
        heroic_found=1
        update_heroic_file "$file"
    done
    if (( ! heroic_found )); then
        printf 'Heroic no tiene una configuración global conocida en este equipo.\n'
    fi
    printf 'Lutris y UMU guardan normalmente el prefijo por juego o por ejecución; no tienen un default global seguro que sustituir.\n'
    printf 'Sus referencias al origen se actualizan con --rewrite-configs, siempre con copia de seguridad.\n'
    printf 'Steam/Proton mantiene un compatdata por AppID; no se cambia como si fuera un único default global.\n'
}

write_default_environment() {
    local config_dir env_file stamp quoted_destination proton_path
    config_dir="$HOME_DIR/.config/wine-prefix-manager"
    env_file="$config_dir/default-prefix.sh"
    stamp="$(date +%Y%m%d-%H%M%S)"
    if (( DRY_RUN )); then
        plan_record write-defaults "$env_file" planned yes "se generaría un backup si ya existe" "$DESTINATION"
        printf '\nSimulación: se generaría la configuración de Wine/winetricks en: %s\n' "$env_file"
        return 0
    fi
    mkdir -p -- "$config_dir" || die "no se pudo crear $config_dir"
    if [[ -e "$env_file" ]]; then
        cp -a -- "$env_file" "$env_file.bak-$stamp" || die "no se pudo respaldar $env_file"
        plan_record restore-file "$env_file" executed yes "$env_file.bak-$stamp" "defaults"
        printf 'Configuración anterior respaldada en: %s.bak-%s\n' "$env_file" "$stamp"
    fi
    quoted_destination="$(printf '%q' "$DESTINATION")"
    {
        printf '# Generado por wine-prefix-manager.sh el %s\n' "$(date --iso-8601=seconds)"
        printf '# Activa este archivo con: source %q\n' "$env_file"
        printf 'export WINEPREFIX=%s\n' "$quoted_destination"
        printf '# No se fija WINEARCH: Wine debe conservar la arquitectura propia del prefijo migrado.\n'
        printf '\n'
        printf 'wine-prefix() { WINEPREFIX="$WINEPREFIX" wine "$@"; }\n'
        printf 'winetricks-prefix() { WINEPREFIX="$WINEPREFIX" winetricks "$@"; }\n'
        if [[ "$SOURCE_TYPE" == steam-proton ]]; then
            proton_path="$(dirname "$DESTINATION")"
            printf '\n# Este valor apunta al directorio compatdata que contiene pfx/.\n'
            printf 'export PROTON_COMPAT_DATA_PATH=%q\n' "$proton_path"
            printf 'proton-prefix() { STEAM_COMPAT_DATA_PATH="$PROTON_COMPAT_DATA_PATH" proton "$@"; }\n'
        else
            printf '\n# No se cambia STEAM_COMPAT_DATA_PATH globalmente: Steam administra un compatdata por AppID.\n'
            printf '# Para Proton usa el compatdata del juego mediante Steam o una configuración específica.\n'
        fi
    } > "$env_file" || die "no se pudo escribir $env_file"
    chmod 600 -- "$env_file" || die "no se pudieron ajustar los permisos de $env_file"
    printf '\nConfiguración de usuario generada: %s\n' "$env_file"
    printf 'Wine y winetricks usarán la nueva ruta después de ejecutar: source %q\n' "$env_file"
    if [[ "$SOURCE_TYPE" == steam-proton ]]; then
        printf 'También se generó proton-prefix() para el compatdata migrado.\n'
    else
        printf 'Proton no se ha cambiado globalmente porque Steam necesita un compatdata por AppID.\n'
    fi
}

activate_shell_defaults() {
    local env_file="$HOME_DIR/.config/wine-prefix-manager/default-prefix.sh"
    local stamp rc environment_dir environment_file environment_prefix
    if (( DRY_RUN )); then
        plan_record activate-shell "$HOME_DIR/.bashrc" planned yes "también se revisaría .zshrc y environment.d" "$env_file"
        printf 'Simulación: se ofrecería activar los defaults en las shells y en environment.d.\n'
        return 0
    fi
    [[ -f "$env_file" ]] || die "primero hay que generar la configuración con --set-defaults"
    ask_phrase "ACTIVAR-DEFAULT" || { printf 'No se modificaron .bashrc ni .zshrc.\n'; return 0; }
    stamp="$(date +%Y%m%d-%H%M%S)"
    for rc in "$HOME_DIR/.bashrc" "$HOME_DIR/.zshrc"; do
        [[ -f "$rc" ]] || continue
        if grep -Fq -- "$env_file" "$rc" 2>/dev/null; then
            printf 'Ya estaba activado en: %s\n' "$rc"
            continue
        fi
        cp -a -- "$rc" "$rc.bak-$stamp" || die "no se pudo respaldar $rc"
        plan_record restore-file "$rc" executed yes "$rc.bak-$stamp" "shell"
        {
            printf '\n# Wine prefix gestionado por wine-prefix-manager.sh\n'
            printf 'source %q\n' "$env_file"
        } >> "$rc" || die "no se pudo actualizar $rc"
        printf 'Activado en: %s (copia: %s.bak-%s)\n' "$rc" "$rc" "$stamp"
    done
    # Desktop applications launched through the user systemd session do not
    # read shell rc files. Provide an environment.d drop-in as well, without
    # exporting a dangerous global Proton compatdata path.
    environment_dir="$HOME_DIR/.config/environment.d"
    environment_file="$environment_dir/90-ltools-wine.conf"
    environment_prefix="${DESTINATION:-$(grep -m1 '^export WINEPREFIX=' "$env_file" | cut -d= -f2-)}"
    [[ "$environment_prefix" != *'"'* ]] || die "la ruta contiene comillas no compatibles con environment.d"
    mkdir -p -- "$environment_dir" || die "no se pudo crear $environment_dir"
    if [[ -e "$environment_file" ]]; then
        cp -a -- "$environment_file" "$environment_file.bak-$stamp" || die "no se pudo respaldar $environment_file"
        plan_record restore-file "$environment_file" executed yes "$environment_file.bak-$stamp" "environment.d"
    fi
    {
        printf '# Generado por wine-prefix-manager.sh\n'
        printf 'WINEPREFIX="%s"\n' "$environment_prefix"
    } >"$environment_file" || die "no se pudo escribir $environment_file"
    chmod 600 -- "$environment_file" || die "no se pudieron ajustar los permisos de $environment_file"
    printf 'Activado para aplicaciones de la sesión de usuario: %s\n' "$environment_file"
    printf 'Abre una nueva terminal o ejecuta: source %q\n' "$env_file"
}

reference_files() {
    local -a roots=() root
    local candidates file
    for root in \
        "$HOME_DIR/.config" \
        "$HOME_DIR/.local/share/applications" \
        "$HOME_DIR/.local/share/lutris" \
        "$HOME_DIR/.local/share/umu" \
        "$HOME_DIR/.config/heroic" \
        "$HOME_DIR/.var/app"; do
        [[ -e "$root" ]] && roots+=("$root")
    done
    ((${#roots[@]})) || return 0
    if have rg; then
        candidates="$(rg -F -l --hidden --no-messages \
            --glob '!cache/**' --glob '!Cache/**' --glob '!Trash/**' \
            -- "$SOURCE" "${roots[@]}" 2>/dev/null || true)"
    else
        candidates="$(grep -R -F -l -- "$SOURCE" "${roots[@]}" 2>/dev/null || true)"
    fi
    while IFS= read -r file; do
        [[ -n "$file" ]] || continue
        path_reference_exists "$file" && printf '%s\n' "$file"
    done <<<"$candidates"
}

path_reference_exists() {
    local file="$1"
    if have perl; then
        OLD_PREFIX="$SOURCE" perl -0ne '
            my $path = $ENV{OLD_PREFIX};
            my $offset = 0;
            while ((my $at = index($_, $path, $offset)) >= 0) {
                my $before = $at > 0 ? substr($_, $at - 1, 1) : "";
                my $after_at = $at + length($path);
                my $after = $after_at < length($_) ? substr($_, $after_at, 1) : "";
                exit 0 if ($before eq "" || $before !~ /[A-Za-z0-9_.-]/)
                    && ($after eq "" || $after !~ /[A-Za-z0-9_.-]/);
                $offset = $after_at;
            }
            exit 1;
        ' -- "$file"
    else
        # Conservative fallback when Perl is unavailable.
        grep -Fq -- "$SOURCE/" "$file" 2>/dev/null || grep -Fq -- "$SOURCE" "$file" 2>/dev/null
    fi
}

reference_application() {
    local file="$1"
    case "$file" in
        */.config/lutris/*|*/.local/share/lutris/*) printf 'Lutris' ;;
        */.config/heroic/*|*/.local/share/heroic/*) printf 'Heroic' ;;
        */.local/share/umu/*|*/.config/umu/*) printf 'UMU' ;;
        */.var/app/com.usebottles.bottles/*) printf 'Bottles' ;;
        */.local/share/Steam/*|*/.steam/*) printf 'Steam/Proton' ;;
        *.desktop) printf 'Lanzador .desktop' ;;
        *) printf 'Configuración general' ;;
    esac
}

print_reference_report() {
    local files="$1" file
    printf '\nAplicaciones que todavía mencionan el origen:\n'
    printf '  %-22s %s\n' 'Aplicación probable' 'Archivo'
    while IFS= read -r file; do
        [[ -n "$file" ]] || continue
        printf '  %-22s %s\n' "$(reference_application "$file")" "$file"
    done <<<"$files"
    printf '\nRuta antigua: %s\nRuta nueva:   %s\n' "$SOURCE" "$DESTINATION"
    printf 'Lutris/Heroic/UMU: comprueba que cada juego use la ruta nueva del prefijo.\n'
    printf 'Steam/Proton: revisa el juego o su biblioteca desde Steam; no se cambia un compatdata global.\n'
}

report_references() {
    local files
    if (( SOURCE_MOUNT_ROOT )); then
        printf 'No se analizarán referencias globales al punto de montaje; el disco contiene juegos y otros datos además del prefijo.\n'
        return 0
    fi
    files="$(reference_files)"
    if [[ -z "$files" ]]; then
        printf 'No se encontraron referencias de texto al origen.\n'
        return 0
    fi
    print_reference_report "$files"
    printf 'No se han modificado. Usa --rewrite-configs para actualizarlas con copia de seguridad.\n'
}

rewrite_configs() {
    local files backup_dir stamp file updated=0 skipped=0
    have perl || die "perl es necesario para actualizar referencias de configuración"
    files="$(reference_files)"
    if [[ -z "$files" ]]; then
        printf 'No se encontraron referencias de texto al origen.\n'
        return 0
    fi
    print_reference_report "$files"
    (( FORCE )) || ask_phrase "ACTUALIZAR-REFERENCIAS" || { printf 'No se actualizarán configuraciones.\n'; return 0; }
    stamp="$(date +%Y%m%d-%H%M%S)"
    backup_dir="$DESTINATION/migration-config-backup-$stamp"
    if (( DRY_RUN )); then
        while IFS= read -r file; do
            [[ -f "$file" ]] || continue
            path_reference_exists "$file" || continue
            plan_record rewrite-config "$file" planned yes "$backup_dir$file" "$DESTINATION"
            printf 'Simulación: se respaldaría y actualizaría: %s\n' "$file"
        done <<<"$files"
        printf 'Simulación: no se han modificado configuraciones.\n'
        return 0
    fi
    mkdir -p -- "$backup_dir" || die "no se pudo crear la copia de seguridad de configuraciones"
    while IFS= read -r file; do
        [[ -f "$file" ]] || continue
        if ! path_reference_exists "$file" || ! grep -Iq -- "$SOURCE" "$file" 2>/dev/null; then
            printf 'Omitido por parecer binario: %s\n' "$file"
            skipped=$((skipped + 1))
            continue
        fi
        cp -a --parents -- "$file" "$backup_dir" || die "no se pudo respaldar $file"
        plan_record restore-file "$file" executed yes "$backup_dir$file" "configuración"
        OLD_PREFIX="$SOURCE" NEW_PREFIX="$DESTINATION" perl -0pi -e '
            my $old = $ENV{OLD_PREFIX};
            my $new = $ENV{NEW_PREFIX};
            s/(?<![A-Za-z0-9_.-])\Q$old\E(?![A-Za-z0-9_.-])/$new/g;
        ' -- "$file" || die "no se pudo actualizar $file"
        printf 'Actualizado: %s\n' "$file"
        updated=$((updated + 1))
    done <<<"$files"
    printf 'Archivos actualizados: %s. Omitidos por ser binarios: %s.\n' "$updated" "$skipped"
    printf 'Copias de seguridad guardadas en: %s\n' "$backup_dir"
    printf 'Reinicia o vuelve a abrir Lutris/Heroic/UMU para que relean sus configuraciones.\n'
}

remove_source_to_trash() {
    printf '\nEl origen se conservará hasta este punto:\n  %s\n' "$SOURCE"
    if (( SELECTION_PARTIAL )); then
        printf 'La copia fue selectiva: hay elementos del origen que no se han movido y se perderían al retirarlo.\n'
        ask_phrase "RETIRAR-OMITIDOS" || { printf 'El origen se conserva porque la migración fue parcial.\n'; return 0; }
    fi
    ask_phrase "RETIRAR-ORIGEN" || { printf 'El origen se conserva.\n'; return 0; }
    if (( DRY_RUN )); then
        plan_record trash-move "$SOURCE" planned yes "se movería a la papelera" ""
        printf 'Simulación: el origen se movería a la papelera después de la verificación.\n'
        return 0
    fi
    have gio || die "gio es necesario para mover el origen a la papelera de forma reversible"
    gio trash -- "$SOURCE" || die "no se pudo mover el origen a la papelera"
    plan_record trash-move "$SOURCE" executed yes "$(plan_trash_guess "$SOURCE")" "origen migrado"
    printf 'Origen movido a la papelera.\n'
}

create_prefix() {
    require_destination
    [[ ! -e "$DESTINATION" || -z "$(find "$DESTINATION" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]] || die "el destino debe ser inexistente o estar vacío"
    local parent fstype wineboot_status
    parent="$(existing_parent "$(dirname "$DESTINATION")")"
    if have findmnt; then
        fstype="$(findmnt -rn -T "$parent" -o FSTYPE 2>/dev/null | head -1)"
        case "$fstype" in
            ntfs|ntfs3|fuseblk|exfat|vfat) printf 'Advertencia: destino sobre %s.\n' "$fstype"; (( FORCE )) || ask_phrase "CONTINUAR-FS" || die "operación cancelada" ;;
        esac
    fi
    printf 'Se creará un prefijo %s en: %s\n' "$ARCH" "$DESTINATION"
    ask_phrase "CREAR $(basename "$DESTINATION")" || die "operación cancelada"
    if (( DRY_RUN )); then
        plan_record create-prefix "$DESTINATION" planned yes "wineboot -u" "$ARCH"
        printf 'Simulación: se ejecutaría wineboot -u con WINEPREFIX=%s.\n' "$DESTINATION"
        return 0
    fi
    have wineboot || die "wineboot no está disponible"
    if WINEARCH="$ARCH" WINEPREFIX="$DESTINATION" wineboot -u; then
        wineboot_status=0
    else
        wineboot_status=$?
    fi
    if [[ ! -d "$DESTINATION/drive_c" || ! -s "$DESTINATION/system.reg" ]]; then
        die "wineboot terminó con código $wineboot_status y no creó un prefijo válido"
    fi
    if (( wineboot_status != 0 )); then
        printf 'Advertencia: wineboot terminó con código %s, pero el prefijo básico sí fue creado. Revisa los mensajes anteriores antes de instalar nada.\n' "$wineboot_status"
    fi
    printf 'Prefijo creado y validado: %s\n' "$DESTINATION"
}

migrate_prefix() {
    if [[ "$SELECT_MODE" == interactive ]] && ((${#INCLUDE_ITEMS[@]} > 0)); then
        die "usa --select o --include, no ambos"
    fi
    require_destination
    validate_source
    check_locks
    select_items
    check_space
    copy_prefix
    if (( SET_DEFAULTS )); then
        write_default_environment
        if (( ACTIVATE_SHELL )); then
            activate_shell_defaults
        fi
    fi
    if (( UPDATE_LAUNCHERS )); then
        update_launcher_defaults
    fi
    if (( REWRITE_CONFIGS )); then
        rewrite_configs
    else
        report_references
    fi
    if (( REMOVE_SOURCE )); then
        remove_source_to_trash
    fi
}

main() {
    parse_args "$@"
    if [[ -n "$ROLLBACK_PATH" ]]; then
        rollback_plan "$ROLLBACK_PATH"
        exit $?
    fi
    plan_init "$SCRIPT_NAME" || die "no se pudo crear el plan: ${PLAN_PATH:-desconocido}"
    case "$ACTION" in
        create)
            [[ -z "$SOURCE" ]] || die "create no acepta --source"
            [[ "$SELECT_MODE" == all ]] && ((${#INCLUDE_ITEMS[@]} == 0)) && ((${#EXCLUDE_ITEMS[@]} == 0)) || die "create no acepta selección de elementos"
            (( ! REWRITE_CONFIGS && ! REMOVE_SOURCE && ! SET_DEFAULTS && ! UPDATE_LAUNCHERS && ! ACTIVATE_SHELL )) || die "create no acepta opciones de migración"
            create_prefix
            ;;
        clone|migrate)
            migrate_prefix
            ;;
    esac
    printf '\nOperación terminada.\n'
}

main "$@"
