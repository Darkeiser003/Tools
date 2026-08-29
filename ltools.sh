#!/usr/bin/env bash

# ltools.sh - unified entry point for LTools disk, game and Wine tools.
# The feature modules remain beside this file so their tested behaviour is
# preserved; users interact with this single parser and menu.

set -uo pipefail

VERSION="0.3.0"
SCRIPT_NAME="$(basename "$0")"
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd -P)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/scripts/lib/ltools-i18n.sh"
export LC_ALL=C
HOME_DIR="${HOME:-$(getent passwd "$(id -u)" 2>/dev/null | cut -d: -f6)}"
MODULE=""
MODULE_ARGS=()
GLOBAL_ARGS=()
RUST_BACKEND=0
RUST_COMMAND=""
PACKAGE_MODE=0
IN_MENU=0
READ_VALUE=""
PREFIX_VALUE=""
PREFIX_PATHS=()
PREFIX_TYPES=()
PREFIX_SIZES=()
PREFIX_NESTED_PARENTS=()
PREFIX_SELECTED_PATHS=()

usage() {
    cat <<EOF
Uso:
  $SCRIPT_NAME                         Abre el menú interactivo.
  $SCRIPT_NAME menu                    Abre el menú interactivo.
  $SCRIPT_NAME audit [opciones]        Auditoría general de discos y paquetes.
  $SCRIPT_NAME packages [opciones]     Inventario de gestores, paquetes y cachés.
  $SCRIPT_NAME games [opciones]        Auditoría de juegos, Wine y Proton.
  $SCRIPT_NAME clean [opciones]        Limpieza protegida e interactiva.
  $SCRIPT_NAME prefix ACCIÓN [opciones] Crear, listar, clonar o migrar prefijos Wine.
  $SCRIPT_NAME prefix inspect            Inspecciona el contenido de un prefijo.
  $SCRIPT_NAME defaults                 Muestra rutas efectivas de Wine, Proton y gestores.
  $SCRIPT_NAME system                   Consulta y administra servicios, procesos y journal.
  $SCRIPT_NAME rollback --plan FICHERO  Revierte operaciones reversibles de un plan.
  $SCRIPT_NAME rust-audit [opciones]   Auditoría nativa Rust.
  $SCRIPT_NAME --rust COMANDO          Usa el CLI Rust para el comando indicado.

Comandos:
  menu, m                              Menú principal.
  audit, disk-audit                    Equivale a disk-audit.sh.
  packages, pkg-audit                  Inventario de paquetes sin escanear todos los discos.
  games, game-audit, wine-audit        Equivale a game-wine-audit.sh.
  clean, cleanup                       Equivale a disk-clean.sh.
  prefix, wine                         Equivale a wine-prefix-manager.sh.
  defaults, paths, status              Estado de rutas y defaults detectados.
  system, services, systemctl          Servicios, daemons, procesos y journal.
  doctor, diagnose, fuse                Diagnóstico de dependencias, FUSE y entorno.
  rollback, undo                       Revierte operaciones reversibles registradas.
  rust-audit, native-audit             Auditoría nativa Rust.

Opciones globales:
  --help, -h                           Muestra esta ayuda.
  --version                            Muestra la versión del lanzador.
  --module-help NOMBRE                 Muestra la ayuda del módulo indicado.
  --dry-run                            Simula la operación del comando elegido.
  --plan FICHERO                       Guarda el plan de la operación.
  --lang IDIOMA                        Idioma: auto, es, en, de, fr, pt o it.
  --rust                               Usa la implementación Rust en vez de Bash.
  --                                  Permite separar el comando de sus argumentos.

Ejemplos:
  $SCRIPT_NAME audit --full --duplicates --min-size-mb 100 \\
      --out "$HOME/Informes/disk-audit-$(date +%Y%m%d-%H%M)"
  $SCRIPT_NAME games --full --root /mnt/JuegosLinux
  $SCRIPT_NAME packages --out "$HOME/Informes/package-audit-$(date +%Y%m%d-%H%M)"
  $SCRIPT_NAME clean --menu --report "$HOME/Informes/disk-audit-..."
  $SCRIPT_NAME prefix migrate --source "$HOME/.wine" \\
      --dest /mnt/JuegosLinux/prefixes/wine-main --select \\
      --rewrite-configs --set-defaults --update-launchers --remove-source
  $SCRIPT_NAME prefix list
  $SCRIPT_NAME prefix batch
  $SCRIPT_NAME system status
  $SCRIPT_NAME doctor
  $SCRIPT_NAME clean --dry-run --path "$HOME/.cache/paru" --plan /tmp/limpieza.tsv
  $SCRIPT_NAME rollback --plan /tmp/limpieza.tsv

Para ver las opciones específicas:
  $SCRIPT_NAME audit --help
  $SCRIPT_NAME games --help
  $SCRIPT_NAME clean --help
  $SCRIPT_NAME prefix --help
EOF
}

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 2
}

have() {
    command -v "$1" >/dev/null 2>&1
}

module_path() {
    case "$1" in
        audit) printf '%s/scripts/disk-audit.sh' "$SCRIPT_DIR" ;;
        games) printf '%s/scripts/game-wine-audit.sh' "$SCRIPT_DIR" ;;
        clean) printf '%s/scripts/disk-clean.sh' "$SCRIPT_DIR" ;;
        prefix) printf '%s/scripts/wine-prefix-manager.sh' "$SCRIPT_DIR" ;;
        system) printf '%s/scripts/system-control.sh' "$SCRIPT_DIR" ;;
        host) printf '%s/scripts/host-tools.sh' "$SCRIPT_DIR" ;;
        rollback) printf '%s/scripts/rollback.sh' "$SCRIPT_DIR" ;;
        rust-audit) printf '%s/rust-audit.sh' "$SCRIPT_DIR" ;;
        rust) printf '%s/rust-tools.sh' "$SCRIPT_DIR" ;;
        *) return 1 ;;
    esac
}

check_module() {
    local module="$1" path
    path="$(module_path "$module")" || die "módulo desconocido: $module"
    [[ -f "$path" ]] || die "falta el módulo $path. Mantén los scripts juntos o reinstala el paquete de LTools."
    [[ -x "$path" ]] || die "el módulo no es ejecutable: $path"
}

run_module() {
    local module="$1" path status
    shift
    check_module "$module"
    path="$(module_path "$module")"
    if (( IN_MENU )); then
        "$path" "$@"
        status=$?
        printf '\nEl módulo terminó con código %s.\n' "$status"
        return "$status"
    fi
    exec "$path" "$@"
}

module_help() {
    local module="$1"
    case "$module" in
        audit|disk-audit|packages|pkg-audit|package-audit) module="audit" ;;
        games|game-audit|wine-audit) module="games" ;;
        clean|cleanup) module="clean" ;;
        prefix|wine) module="prefix" ;;
        system|services|systemctl|sys) module="system" ;;
        rollback|undo) module="rollback" ;;
        rust-audit|native-audit) module="rust-audit" ;;
        *) die "módulo desconocido: $module" ;;
    esac
    run_module "$module" --help
}

command_location() {
    local command_name="$1"
    if command -v "$command_name" >/dev/null 2>&1; then
        command -v "$command_name"
    else
        printf 'no instalado o no está en PATH'
    fi
}

show_defaults() {
    local wine_prefix default_file managed_line compat_path root library_file heroic_file heroic_parent heroic_prefix heroic_runner lutris_config lutris_game_path
    local -a steam_roots=() seen_roots=()
    wine_prefix="${WINEPREFIX:-$HOME_DIR/.wine}"
    default_file="$HOME_DIR/.config/wine-prefix-manager/default-prefix.sh"

    printf '\n=== Rutas y defaults detectados ===\n'
    printf '\nWine:\n'
    printf '  Ejecutable: %s\n' "$(command_location wine)"
    printf '  wineboot:   %s\n' "$(command_location wineboot)"
    if [[ -n "${WINEPREFIX:-}" ]]; then
        printf '  WINEPREFIX activo en esta terminal: %s\n' "$WINEPREFIX"
    else
        printf '  WINEPREFIX activo: no definido\n'
        printf '  Fallback de Wine: %s\n' "$wine_prefix"
    fi
    if [[ -d "$wine_prefix" ]]; then
        printf '  Estado: existe\n'
    else
        printf '  Estado: todavía no existe\n'
    fi

    printf '\nWinetricks:\n'
    printf '  Ejecutable: %s\n' "$(command_location winetricks)"
    printf '  Prefijo usado: %s\n' "$wine_prefix"
    printf '  Nota: winetricks sigue WINEPREFIX; no tiene un prefijo independiente.\n'

    printf '\nConfiguración centralizada del gestor:\n'
    if [[ -f "$default_file" ]]; then
        managed_line="$(grep -m1 '^export WINEPREFIX=' "$default_file" 2>/dev/null || true)"
        printf '  Archivo: %s\n' "$default_file"
        printf '  %s\n' "${managed_line:-WINEPREFIX no definido en el archivo}"
        if grep -q 'proton-prefix()' "$default_file" 2>/dev/null; then
            printf '  Proton: wrapper proton-prefix() configurado\n'
        else
            printf '  Proton: sin wrapper global (correcto para Wine normal)\n'
        fi
    else
        printf '  No existe %s\n' "$default_file"
        printf '  Usa --set-defaults tras una migración para generarlo.\n'
    fi

    printf '\nProton/Steam:\n'
    printf '  proton: %s\n' "$(command_location proton)"
    printf '  steam:  %s\n' "$(command_location steam)"
    if [[ -n "${STEAM_COMPAT_DATA_PATH:-}" ]]; then
        printf '  STEAM_COMPAT_DATA_PATH activo: %s\n' "$STEAM_COMPAT_DATA_PATH"
    elif [[ -n "${PROTON_COMPAT_DATA_PATH:-}" ]]; then
        printf '  PROTON_COMPAT_DATA_PATH configurado: %s\n' "$PROTON_COMPAT_DATA_PATH"
    else
        printf '  Default Proton global: ninguno\n'
        printf '  Steam asigna compatdata por AppID; esto es lo esperado.\n'
    fi
    for root in \
        "$HOME_DIR/.local/share/Steam" \
        "$HOME_DIR/Games/Steam" \
        /mnt/JuegosLinux \
        /mnt/JuegosWindows; do
        [[ -d "$root/steamapps" ]] || continue
        [[ " ${seen_roots[*]} " == *" $root "* ]] && continue
        seen_roots+=("$root")
        steam_roots+=("$root")
    done
    for library_file in \
        "$HOME_DIR/.local/share/Steam/steamapps/libraryfolders.vdf" \
        "$HOME_DIR/.steam/steam/steamapps/libraryfolders.vdf" \
        "$HOME_DIR/.local/share/Steam/config/libraryfolders.vdf"; do
        [[ -f "$library_file" ]] || continue
        while IFS= read -r root; do
            [[ -d "$root/steamapps" ]] || continue
            [[ " ${seen_roots[*]} " == *" $root "* ]] && continue
            seen_roots+=("$root")
            steam_roots+=("$root")
        done < <(awk -F'"' '/"path"[[:space:]]*"/ {print $4}' "$library_file" 2>/dev/null | sed 's#\\\\#\\#g')
    done
    if ((${#steam_roots[@]})); then
        printf '  Librerías Steam detectadas:\n'
        printf '    %s\n' "${steam_roots[@]}"
    else
        printf '  No se detectaron librerías Steam en rutas conocidas.\n'
    fi

    printf '\nLanzadores y gestores:\n'
    printf '  lutris: %s\n' "$(command_location lutris)"
    printf '  heroic: %s\n' "$(command_location heroic)"
    printf '  Datos Lutris: %s\n' "$HOME_DIR/.local/share/lutris"
    printf '  Datos Heroic: %s\n' "$HOME_DIR/.config/heroic"
    printf '  Datos UMU:    %s\n' "$HOME_DIR/.local/share/umu"
    printf '\nDefaults internos detectados:\n'
    heroic_file="$HOME_DIR/.config/heroic/config.json"
    if [[ -f "$heroic_file" ]]; then
        printf '  Heroic (config.json): %s\n' "$heroic_file"
        if have jq; then
            heroic_parent="$(jq -r '.defaultSettings.defaultWinePrefix // .settings.defaultWinePrefix // empty' "$heroic_file" 2>/dev/null)"
            heroic_prefix="$(jq -r '.defaultSettings.winePrefix // .settings.winePrefix // empty' "$heroic_file" 2>/dev/null)"
            heroic_runner="$(jq -r '.defaultSettings.wineVersion.bin // .settings.wineVersion.bin // empty' "$heroic_file" 2>/dev/null)"
            printf '    Carpeta de prefijos: %s\n' "${heroic_parent:-no detectada}"
            printf '    Prefijo compartido:  %s\n' "${heroic_prefix:-no detectado}"
            printf '    Runner seleccionado: %s\n' "${heroic_runner:-no detectado}"
        else
            heroic_parent="$(grep -m1 '"defaultWinePrefix"[[:space:]]*:' "$heroic_file" 2>/dev/null | sed -E 's/.*:[[:space:]]*"([^"]*)".*/\1/')"
            heroic_prefix="$(grep -m1 '"winePrefix"[[:space:]]*:' "$heroic_file" 2>/dev/null | sed -E 's/.*:[[:space:]]*"([^"]*)".*/\1/')"
            heroic_runner="$(grep -m1 '"bin"[[:space:]]*:' "$heroic_file" 2>/dev/null | sed -E 's/.*:[[:space:]]*"([^"]*)".*/\1/')"
            printf '    Carpeta de prefijos: %s\n' "${heroic_parent:-no detectada (instala jq para una lectura completa)}"
            printf '    Prefijo compartido:  %s\n' "${heroic_prefix:-no detectado}"
            printf '    Runner seleccionado: %s\n' "${heroic_runner:-no detectado}"
        fi
    else
        printf '  Heroic: no se encontró config.json global.\n'
    fi
    lutris_config="$HOME_DIR/.local/share/lutris/system.yml"
    if [[ -f "$lutris_config" ]]; then
        lutris_game_path="$(awk -F': ' '$1 ~ /game_path/ {print $2; exit}' "$lutris_config" 2>/dev/null)"
        printf '  Lutris carpeta de juegos: %s\n' "${lutris_game_path:-no detectada}"
        printf '  Lutris prefijo por defecto: no global; cada juego lo define en su YAML.\n'
    else
        printf '  Lutris: no se encontró system.yml.\n'
    fi
    printf '  UMU: usa WINEPREFIX/PROTONPATH por ejecución; no se fuerza un default global.\n'
    printf '  Para verificar una migración: ejecuta «%s defaults» y revisa también el informe games.\n' "$SCRIPT_NAME"
}

read_nonempty() {
    local prompt="$1" value
    while :; do
        printf '%s' "$prompt"
        read -r value || return 1
        if [[ -n "$value" ]]; then
            READ_VALUE="$value"
            return 0
        fi
        printf 'La ruta no puede estar vacía.\n'
    done
}

ask_phrase() {
    local expected="$1" answer
    printf 'Escribe "%s" para confirmar: ' "$expected"
    read -r answer || return 1
    [[ "$answer" == "$expected" ]]
}

confirm() {
    local prompt="$1" answer
    printf '%s [y/N] ' "$prompt"
    read -r answer || return 1
    [[ "$answer" =~ ^([yY][eE][sS]|[yY]|[sS]|[sS][iI])$ ]]
}

menu_audit() {
    local mode duplicates root out
    mode="quick"
    duplicates=0
    root=""
    out=""
    printf '\nAuditoría general\n'
    printf '¿Escaneo completo? [y/N] '
    read -r answer || return 0
    [[ "$answer" =~ ^([yY]|[sS])$ ]] && mode="full"
    printf '¿Buscar duplicados por SHA-256? [y/N] '
    read -r answer || return 0
    [[ "$answer" =~ ^([yY]|[sS])$ ]] && duplicates=1
    printf 'Ruta adicional (vacío para ninguna): '
    read -r root || return 0
    printf 'Directorio de informe (vacío para el valor predeterminado): '
    read -r out || return 0
    local -a args=()
    [[ "$mode" == full ]] && args+=(--full)
    (( duplicates )) && args+=(--duplicates)
    [[ -n "$root" ]] && args+=(--root "$root")
    [[ -n "$out" ]] && args+=(--out "$out")
    run_module audit "${args[@]}"
}

menu_games() {
    local mode root out
    mode="quick"
    root=""
    out=""
    printf '\nAuditoría de juegos, Wine y Proton\n'
    printf '¿Escaneo completo? [y/N] '
    read -r answer || return 0
    [[ "$answer" =~ ^([yY]|[sS])$ ]] && mode="full"
    printf 'Ruta adicional (vacío para ninguna): '
    read -r root || return 0
    printf 'Directorio de informe (vacío para el valor predeterminado): '
    read -r out || return 0
    local -a args=()
    [[ "$mode" == full ]] && args+=(--full)
    [[ -n "$root" ]] && args+=(--root "$root")
    [[ -n "$out" ]] && args+=(--out "$out")
    run_module games "${args[@]}"
}

menu_clean() {
    local report
    printf '\nLimpieza protegida\n'
    printf 'Informe de auditoría opcional (vacío para ninguno): '
    read -r report || return 0
    if [[ -n "$report" ]]; then
        run_module clean --menu --report "$report"
    else
        run_module clean --menu
    fi
}

menu_defaults() {
    show_defaults
    printf '\nPulsa Enter para volver al menú: '
    read -r _ || true
}

menu_packages() {
    local out root
    printf '\nInventario de paquetes y almacenes\n'
    printf 'Se comprobarán los gestores instalados y sus cachés conocidas.\n'
    printf 'Ruta adicional opcional para buscar archivos de paquetes (vacío para ninguna): '
    read -r root || return 0
    printf 'Directorio de informe (vacío para el valor predeterminado): '
    read -r out || return 0
    local -a args=(--packages-only)
    [[ -n "$root" ]] && args+=(--root "$root")
    [[ -n "$out" ]] && args+=(--out "$out")
    run_module audit "${args[@]}"
}

prefix_kind_label() {
    local path="$1"
    if [[ -f "$path/lutris.json" ]]; then
        printf 'Lutris'
        return 0
    fi
    if [[ -f "$path/config_info" ]]; then
        local runner_path
        runner_path="$(sed -n '2p' "$path/config_info" 2>/dev/null)"
        case "$runner_path" in
            */lutris/*) printf 'Lutris'; return 0 ;;
            */umu/*|*/compatibilitytools/*) printf 'UMU'; return 0 ;;
            */heroic/*) printf 'Heroic'; return 0 ;;
            */steam/*|*/Steam/*|*/steamapps/*|*/compatibilitytools.d/*) printf 'Proton'; return 0 ;;
        esac
    fi
    case "$path" in
        */files/share/default_pfx) printf 'plantilla Proton/Wine' ;;
        */steamapps/compatdata/*/pfx) printf 'Steam/Proton' ;;
        */.wine) printf 'Wine predeterminado' ;;
        */lutris/*|*/Lutris/*|*/Lutrs/*|*/.local/share/lutris/*|*/.config/lutris/*) printf 'Lutris' ;;
        */Heroic/*|*/heroic/*|*/.config/heroic/*) printf 'Heroic' ;;
        */bottles/*|*/.var/app/com.usebottles.bottles/*) printf 'Bottles' ;;
        */umu/*|*/.local/share/umu/*) printf 'UMU' ;;
        *) printf 'Wine/prefijo' ;;
    esac
}

prefix_parent_path() {
    local path="$1" parent
    parent="$(dirname -- "$path")"
    while [[ "$parent" != / && "$parent" != . ]]; do
        if [[ -f "$parent/system.reg" && "$parent" != "$path" ]]; then
            if ! have findmnt || [[ "$(findmnt -rn -T "$parent" -o TARGET 2>/dev/null | head -1)" != "$parent" ]]; then
                printf '%s' "$parent"
                return 0
            fi
        fi
        path="$parent"
        parent="$(dirname -- "$parent")"
    done
}

prefix_architecture_label() {
    local path="$1"
    local value
    value="$(grep -m1 '^#arch=' "$path/system.reg" 2>/dev/null | sed 's/^#arch=//' || true)"
    [[ -n "$value" ]] && { printf '%s' "$value"; return 0; }
    [[ -d "$path/drive_c/Program Files (x86)" || -d "$path/drive_c/windows/syswow64" ]] && { printf 'probable-win64'; return 0; }
    printf 'no detectada'
}

prefix_windows_label() {
    local path="$1" value
    value="$(grep -m1 '"ProductName"=' "$path/system.reg" 2>/dev/null | sed 's/.*"ProductName"=//' | sed 's/^str://' | sed 's/^"//; s/".*$//' || true)"
    printf '%s' "${value:-no detectada}"
}

prefix_runner_label() {
    local path="$1" runner_path runner_name
    if [[ "$path" =~ /steamapps/compatdata/([0-9]+)/pfx$ ]]; then
        printf 'Steam/Proton (AppID %s)' "${BASH_REMATCH[1]}"
    elif [[ "$path" =~ /compatibilitytools\.d/([^/]+)/files/share/default_pfx$ ]]; then
        printf 'Steam runner (%s)' "${BASH_REMATCH[1]}"
    elif [[ "$path" =~ /tools/proton/([^/]+)/files/share/default_pfx$ ]]; then
        printf 'Heroic runner (%s)' "${BASH_REMATCH[1]}"
    elif [[ "$path" == */runners/wine/* ]]; then
        printf 'Lutris runner (%s)' "$(basename "$path")"
    elif [[ "$path" == */compatibilitytools/* ]]; then
        printf 'UMU runner (%s)' "$(basename "$path")"
    elif [[ -f "$path/config_info" ]]; then
        runner_path="$(sed -n '2p' "$path/config_info" 2>/dev/null)"
        runner_name="$(basename "$(dirname "$(dirname "$(dirname "$runner_path")")")")"
        printf 'Runner detectado (%s)' "$runner_name"
    else
        printf 'Wine del sistema/externo'
    fi
}

inspect_prefix() {
    local path="$PREFIX_VALUE" lock_count exe_count msi_count dir size refs
    select_existing_prefix || return 0
    path="$PREFIX_VALUE"
    bytes="$(du -sx --block-size=1 -- "$path" 2>/dev/null | awk 'NR==1{print $1}')"
    printf '\n=== Inspección del prefijo ===\n'
    printf 'Ruta:          %s\n' "$path"
    printf 'Tipo:          %s\n' "$(prefix_kind_label "$path")"
    printf 'Tamaño total:  %s\n' "$(prefix_size "$path")"
    printf 'Arquitectura:  %s\n' "$(prefix_architecture_label "$path")"
    printf 'Windows:       %s\n' "$(prefix_windows_label "$path")"
    printf 'Runner:        %s\n' "$(prefix_runner_label "$path")"
    printf 'system.reg:    %s\n' "$([[ -s "$path/system.reg" ]] && printf 'presente' || printf 'ausente/vacío')"
    printf 'user.reg:      %s\n' "$([[ -s "$path/user.reg" ]] && printf 'presente' || printf 'ausente/vacío')"
    printf 'dosdevices:    %s\n' "$([[ -e "$path/dosdevices" ]] && printf 'presente' || printf 'ausente')"
    printf 'drive_c:       %s\n' "$([[ -d "$path/drive_c" ]] && printf '%s' "$(prefix_size "$path/drive_c")" || printf 'ausente')"
    lock_count="$(find "$path" -maxdepth 4 -type f \( -name '*.lock' -o -name '*.lck' -o -name lock \) -print 2>/dev/null | wc -l)"
    exe_count="$(find "$path/drive_c" -type f -iname '*.exe' -print 2>/dev/null | wc -l)"
    msi_count="$(find "$path/drive_c" -type f -iname '*.msi' -print 2>/dev/null | wc -l)"
    printf 'Bloqueos:      %s\n' "$lock_count"
    printf 'Ejecutables:   %s .exe / %s .msi\n' "$exe_count" "$msi_count"
    printf '\nCarpetas de contenido:\n'
    for dir in 'Program Files' 'Program Files (x86)' users windows; do
        [[ -d "$path/drive_c/$dir" ]] || continue
        size="$(prefix_size "$path/drive_c/$dir")"
        printf '  %-22s %s\n' "$dir" "$size"
    done
    printf '\nEjecutables principales detectados:\n'
    find "$path/drive_c" -type f -iname '*.exe' -printf '%f\n' 2>/dev/null | sort -fu | head -25 | sed 's/^/  /'
    refs=""
    if command -v rg >/dev/null 2>&1; then
        refs="$(rg -F -l --hidden --no-messages --glob '!cache/**' --glob '!Cache/**' -- "$path" "$HOME_DIR/.config" "$HOME_DIR/.local/share" 2>/dev/null | head -20 || true)"
    fi
    if [[ -n "$refs" ]]; then
        printf '\nAplicaciones que referencian esta ruta:\n  %s\n' "$refs"
    else
        printf '\nNo se encontraron referencias de texto a esta ruta en la configuración de usuario.\n'
    fi
}

prefix_size() {
    local bytes
    bytes="$(du -sx --block-size=1 -- "$1" 2>/dev/null | awk 'NR==1{print $1}')"
    [[ "$bytes" =~ ^[0-9]+$ ]] || { printf '?'; return 0; }
    if command -v numfmt >/dev/null 2>&1; then
        numfmt --to=iec --suffix=B "$bytes" 2>/dev/null || printf '%sB' "$bytes"
    else
        printf '%sB' "$bytes"
    fi
}

discover_prefixes() {
    local -a roots=() found=() unique=()
    local root path resolved item already nested_parent
    PREFIX_PATHS=()
    PREFIX_TYPES=()
    PREFIX_SIZES=()
    PREFIX_NESTED_PARENTS=()
    for root in "$HOME_DIR" /mnt /media /run/media; do
        if [[ "${LTOOLS_NO_MOUNTS:-0}" =~ ^(1|true|yes|si|sí)$ && "$root" != "$HOME_DIR" ]]; then
            continue
        fi
        [[ -d "$root" ]] && roots+=("$root")
    done
    if [[ ! "${LTOOLS_NO_MOUNTS:-0}" =~ ^(1|true|yes|si|sí)$ ]] && command -v findmnt >/dev/null 2>&1; then
        while IFS= read -r root; do
            [[ "$root" == /mnt/* || "$root" == /media/* || "$root" == /run/media/* || "$root" == "$HOME_DIR"/* ]] || continue
            if [[ "$root" == "$HOME_DIR"/* ]]; then
                [[ "$(dirname -- "$root")" == "$HOME_DIR" ]] || continue
            fi
            [[ -d "$root" ]] && roots+=("$root")
        done < <(findmnt -rn -o TARGET 2>/dev/null)
    fi
    ((${#roots[@]})) || return 0
    mapfile -d '' -t found < <(
        find "${roots[@]}" -xdev \( \
            -type f -name system.reg -printf '%h\0' -o \
            -type d -name drive_c -printf '%h\0' \
        \) 2>/dev/null | sort -zu
    )
    for path in "${found[@]}"; do
        resolved="$(realpath -e -- "$path" 2>/dev/null || printf '%s' "$path")"
        case "$resolved" in
            */files/share/default_pfx|*/.Trash-*/*|*/Trash/*) continue ;;
        esac
        if command -v findmnt >/dev/null 2>&1; then
            [[ "$(findmnt -rn -T "$resolved" -o TARGET 2>/dev/null | head -1)" == "$resolved" ]] && continue
        fi
        already=0
        for item in "${unique[@]}"; do
            [[ "$item" == "$resolved" ]] && already=1
        done
        (( already )) && continue
        unique+=("$resolved")
        PREFIX_PATHS+=("$resolved")
        nested_parent="$(prefix_parent_path "$resolved")"
        if [[ -n "$nested_parent" ]]; then
            PREFIX_TYPES+=("anidado/$(prefix_kind_label "$resolved")")
        else
            PREFIX_TYPES+=("$(prefix_kind_label "$resolved")")
        fi
        PREFIX_NESTED_PARENTS+=("$nested_parent")
        PREFIX_SIZES+=("$(prefix_size "$resolved")")
    done
}

print_prefix_table() {
    local index
    printf '\nPrefijos detectados:\n'
    for index in "${!PREFIX_PATHS[@]}"; do
        printf '  %2d) %-18s %-8s %s\n' \
            "$((index + 1))" "${PREFIX_TYPES[index]}" "${PREFIX_SIZES[index]}" "${PREFIX_PATHS[index]}"
        [[ -n "${PREFIX_NESTED_PARENTS[index]}" ]] && printf '      dentro de: %s\n' "${PREFIX_NESTED_PARENTS[index]}"
    done
}

select_existing_prefix() {
    local choice index path
    printf '\nBuscando prefijos en tu home y discos montados...\n'
    discover_prefixes
    if ((${#PREFIX_PATHS[@]} == 0)); then
        printf 'No se encontraron prefijos automáticamente.\n'
        read_nonempty 'Ruta del prefijo origen: ' || return 1
        PREFIX_VALUE="$READ_VALUE"
        return 0
    fi
    print_prefix_table
    printf 'Elige un número, "m" para introducir una ruta o "q" para cancelar: '
    read -r choice || return 1
    case "$choice" in
        q|Q) return 1 ;;
        m|M)
            read_nonempty 'Ruta del prefijo origen: ' || return 1
            PREFIX_VALUE="$READ_VALUE"
            ;;
        *)
            [[ "$choice" =~ ^[0-9]+$ ]] || { printf 'Selección no válida.\n'; return 1; }
            index=$((choice - 1))
            (( index >= 0 && index < ${#PREFIX_PATHS[@]} )) || { printf 'Número fuera de rango.\n'; return 1; }
            PREFIX_VALUE="${PREFIX_PATHS[index]}"
            printf 'Origen elegido: %s\n' "$PREFIX_VALUE"
            ;;
    esac
    return 0
}

list_existing_prefixes() {
    printf 'Buscando prefijos en tu home y discos montados...\n'
    discover_prefixes
    if ((${#PREFIX_PATHS[@]} == 0)); then
        printf 'No se encontraron prefijos Wine/Proton.\n'
        return 0
    fi
    print_prefix_table
}

select_existing_prefixes() {
    local -a choices=() selected_indexes=()
    local choice index path already
    PREFIX_SELECTED_PATHS=()
    printf '\nBuscando prefijos en tu home y discos montados...\n'
    discover_prefixes
    ((${#PREFIX_PATHS[@]})) || { printf 'No se encontraron prefijos automáticamente.\n'; return 1; }
    print_prefix_table
    printf 'Elige números separados por espacios, "all" para todos o "q" para cancelar: '
    read -r -a choices || return 1
    [[ "${choices[0]:-}" != q && "${choices[0]:-}" != Q ]] || return 1
    if [[ "${choices[0]:-}" == all ]]; then
        for index in "${!PREFIX_PATHS[@]}"; do
            selected_indexes+=("$index")
        done
    else
        for choice in "${choices[@]}"; do
            [[ "$choice" =~ ^[0-9]+$ ]] || { printf 'Selección no válida: %s\n' "$choice"; return 1; }
            index=$((choice - 1))
            (( index >= 0 && index < ${#PREFIX_PATHS[@]} )) || { printf 'Número fuera de rango: %s\n' "$choice"; return 1; }
            already=0
            for path in "${selected_indexes[@]}"; do
                [[ "$path" == "$index" ]] && already=1
            done
            (( already == 0 )) && selected_indexes+=("$index")
        done
    fi
    for index in "${selected_indexes[@]}"; do
        PREFIX_SELECTED_PATHS+=("${PREFIX_PATHS[index]}")
    done
    ((${#PREFIX_SELECTED_PATHS[@]})) || { printf 'No se seleccionó ningún prefijo.\n'; return 1; }
    return 0
}

prefix_destination_name() {
    local source="$1" base
    if [[ "$source" =~ /steamapps/compatdata/([0-9]+)/pfx$ ]]; then
        printf 'steam-%s' "${BASH_REMATCH[1]}"
        return 0
    fi
    base="$(basename -- "$source")"
    [[ "$base" == .wine ]] && base="wine-default"
    [[ "$base" == pfx ]] && base="$(basename -- "$(dirname -- "$source")")-pfx"
    printf '%s' "$base" | sed 's#[^A-Za-z0-9_.-]#_#g'
}

batch_migrate_prefixes() {
    local -a destinations=() args=() failed_sources=()
    local source destination central_root central_parent answer source_index destination_index allow_steam=0
    local destination_name suffix nested_warning
    local copy_mode="full" rewrite=0 remove=0
    select_existing_prefixes || return 0

    printf '\nSe migrarán los contenidos directamente a varios prefijos independientes.\n'
    printf 'Se usará una carpeta central solo para organizar esos prefijos; no se fusionarán entre sí.\n'
    read_nonempty 'Carpeta central para los nuevos prefijos: ' || return 0
    central_root="$(realpath -m -- "$READ_VALUE" 2>/dev/null || printf '%s' "$READ_VALUE")"
    if [[ -f "$central_root/system.reg" || -d "$central_root/drive_c" ]]; then
        printf 'La carpeta central ya parece un prefijo Wine: %s\n' "$central_root"
        printf 'Elige una carpeta contenedora aparte; no se deben guardar prefijos dentro de otro prefijo.\n'
        return 0
    fi
    central_parent="$(prefix_parent_path "$central_root")"
    if [[ -n "$central_parent" ]]; then
        printf 'La carpeta central está dentro de otro prefijo Wine: %s\n' "$central_parent"
        printf 'Elige una carpeta central independiente para evitar prefijos anidados.\n'
        return 0
    fi
    nested_warning=0
    for source_index in "${!PREFIX_SELECTED_PATHS[@]}"; do
        for destination_index in "${!PREFIX_SELECTED_PATHS[@]}"; do
            (( source_index < destination_index )) || continue
            source="${PREFIX_SELECTED_PATHS[source_index]}"
            destination="${PREFIX_SELECTED_PATHS[destination_index]}"
            if [[ "$destination/" == "$source/"* || "$source/" == "$destination/"* ]]; then
                nested_warning=1
            fi
        done
    done
    if (( nested_warning )); then
        printf '\nAdvertencia: has seleccionado prefijos anidados o contenidos uno dentro de otro.\n'
        printf 'Migrarlos juntos puede duplicar contenido o copiar un prefijo dentro de otro.\n'
        confirm '¿Continuar de todos modos?' || return 0
    fi
    for source in "${PREFIX_SELECTED_PATHS[@]}"; do
        destination_name="$(prefix_destination_name "$source")"
        destination="$central_root/$destination_name"
        suffix=2
        while [[ -e "$destination" ]]; do
            destination="$central_root/${destination_name}-${suffix}"
            suffix=$((suffix + 1))
        done
        [[ "$destination" != "$source" ]] || { printf 'El destino no puede ser el mismo origen.\n'; return 0; }
        destinations+=("$destination")
    done

    printf '\nVista previa de la migración:\n'
    for source_index in "${!PREFIX_SELECTED_PATHS[@]}"; do
        printf '  %s\n      → %s\n' "${PREFIX_SELECTED_PATHS[source_index]}" "${destinations[source_index]}"
    done
    printf '¿Copiar el prefijo completo en cada destino? [Y/n] '
    read -r answer || return 0
    [[ "$answer" =~ ^([nN])$ ]] && copy_mode="select"
    if [[ "$copy_mode" == select ]]; then
        printf 'Se elegirá el contenido de cada prefijo durante su migración.\n'
    fi
    printf '¿Actualizar referencias de Lutris/Heroic/UMU para cada origen? [y/N] '
    read -r answer || return 0
    [[ "$answer" =~ ^([yY]|[sS])$ ]] && rewrite=1
    printf '¿Retirar cada origen a la papelera tras verificar su copia? [y/N] '
    read -r answer || return 0
    [[ "$answer" =~ ^([yY]|[sS])$ ]] && remove=1
    for source in "${PREFIX_SELECTED_PATHS[@]}"; do
        [[ "$source" == */steamapps/compatdata/*/pfx ]] || continue
        allow_steam=1
        break
    done
    if (( allow_steam )); then
        printf 'Hay uno o más prefijos Steam/Proton seleccionados; Steam suele gestionarlos directamente.\n'
        printf '¿Permitir la migración especial de Steam/Proton? [y/N] '
        read -r answer || return 0
        [[ "$answer" =~ ^([yY]|[sS])$ ]] || { printf 'Migración cancelada.\n'; return 0; }
    fi
    confirm "¿Iniciar la migración de los ${#PREFIX_SELECTED_PATHS[@]} prefijos?" || { printf 'Migración cancelada.\n'; return 0; }

    local success=0
    for source_index in "${!PREFIX_SELECTED_PATHS[@]}"; do
        source="${PREFIX_SELECTED_PATHS[source_index]}"
        destination="${destinations[source_index]}"
        printf '\n===== %s de %s =====\n' "$((source_index + 1))" "${#PREFIX_SELECTED_PATHS[@]}"
        args=(migrate --source "$source" --dest "$destination")
        [[ "$copy_mode" == select ]] && args+=(--select)
        (( rewrite )) && args+=(--rewrite-configs)
        (( remove )) && args+=(--remove-source)
        (( allow_steam )) && [[ "$source" == */steamapps/compatdata/*/pfx ]] && args+=(--allow-steam)
        args+=(--batch-mode)
        if run_module prefix "${args[@]}"; then
            success=$((success + 1))
        else
            failed_sources+=("$source")
            printf 'Falló esta migración.\n'
            printf '¿Continuar con el resto? [y/N] '
            read -r answer || break
            [[ "$answer" =~ ^([yY]|[sS])$ ]] || break
        fi
    done
    printf '\nResumen: %s de %s migraciones terminadas correctamente.\n' "$success" "${#PREFIX_SELECTED_PATHS[@]}"
    if ((${#failed_sources[@]})); then
        printf 'Fallidas o no ejecutadas:\n'
        printf '  %s\n' "${failed_sources[@]}"
    fi
    printf 'Para varios prefijos no se modifica un default global de Wine: elige uno y usa --set-defaults en una migración individual.\n'
}

menu_prefix() {
    local choice source destination answer
    printf '\nGestor de prefijos Wine\n'
    printf '  1) Crear un prefijo vacío\n'
    printf '  2) Clonar/migrar un prefijo existente\n'
    printf '  3) Centralizar varios prefijos en una carpeta común\n'
    printf '  4) Listar prefijos existentes\n'
    printf '  5) Inspeccionar contenido de un prefijo\n'
    printf '  q) Volver\n'
    printf 'Elige una opción: '
    read -r choice || return 0
    case "$choice" in
        1)
            read_nonempty 'Ruta del nuevo prefijo: ' || return 0
            destination="$READ_VALUE"
            run_module prefix create --dest "$destination"
            ;;
        2)
            select_existing_prefix || return 0
            source="$PREFIX_VALUE"
            read_nonempty 'Ruta del nuevo prefijo: ' || return 0
            destination="$READ_VALUE"
            local -a args=(migrate --source "$source" --dest "$destination")
            local source_mount_root=0 mount_target
            mount_target="$(findmnt -rn -T "$source" -o TARGET 2>/dev/null | head -1)"
            if [[ "$mount_target" == "$source" ]]; then
                source_mount_root=1
                args+=(--allow-mount-root)
                local rescue_item
                for rescue_item in drive_c system.reg user.reg userdef.reg dosdevices .update-timestamp; do
                    [[ -e "$source/$rescue_item" || -L "$source/$rescue_item" ]] && args+=(--include "$rescue_item")
                done
                printf 'Se ha detectado un prefijo en la raíz del disco. Solo se rescatarán sus componentes Wine conocidos; el resto del disco no se tocará.\n'
            else
                args+=(--select)
            fi
            if (( ! source_mount_root )) && [[ "$source" == */steamapps/compatdata/*/pfx ]]; then
                printf 'El origen pertenece a Steam/Proton. Steam normalmente debe gestionar estos prefijos.\n'
                printf '¿Continuar bajo tu propia responsabilidad? [y/N] '
                read -r answer || return 0
                [[ "$answer" =~ ^([yY]|[sS])$ ]] || { printf 'Migración de Steam cancelada.\n'; return 0; }
                args+=(--allow-steam)
            fi
            if (( source_mount_root )); then
                printf 'No se ofrecerá actualizar referencias globales del disco ni retirar su raíz.\n'
            else
                printf '¿Actualizar referencias de Lutris/Heroic/UMU? [y/N] '
                read -r answer || return 0
                [[ "$answer" =~ ^([yY]|[sS])$ ]] && args+=(--rewrite-configs)
                printf '¿Actualizar el default interno de Heroic? [y/N] '
                read -r answer || return 0
                [[ "$answer" =~ ^([yY]|[sS])$ ]] && args+=(--update-launchers)
            fi
            printf '¿Generar defaults para Wine/winetricks? [y/N] '
            read -r answer || return 0
            if [[ "$answer" =~ ^([yY]|[sS])$ ]]; then
                args+=(--set-defaults)
                printf '¿Activarlos en futuras terminales y aplicaciones de usuario? [y/N] '
                read -r answer || return 0
                [[ "$answer" =~ ^([yY]|[sS])$ ]] && args+=(--activate-shell)
            fi
            if (( ! source_mount_root )); then
                printf '¿Retirar el prefijo antiguo a la papelera tras verificar? [y/N] '
                read -r answer || return 0
                [[ "$answer" =~ ^([yY]|[sS])$ ]] && args+=(--remove-source)
            fi
            run_module prefix "${args[@]}"
            ;;
        3) batch_migrate_prefixes ;;
        4) list_existing_prefixes ;;
        5) inspect_prefix ;;
        q|Q) return 0 ;;
        *) printf 'Opción no válida.\n' ;;
    esac
}

menu() {
    local choice
    IN_MENU=1
    while :; do
        printf '\n%s\n' "$(ltools_t menu.title "$VERSION")"
        printf '  1) %s\n' "$(ltools_t menu.audit)"
        printf '  2) %s\n' "$(ltools_t menu.games)"
        printf '  3) %s\n' "$(ltools_t menu.clean)"
        printf '  4) %s\n' "$(ltools_t menu.prefix)"
        printf '  5) %s\n' "$(ltools_t menu.defaults)"
        printf '  6) %s\n' "$(ltools_t menu.packages)"
        printf '  7) %s\n' "$(ltools_t menu.system)"
        printf '  8) %s\n' "$(ltools_t menu.doctor)"
        printf '  h) %s\n' "$(ltools_t menu.help)"
        printf '  q) %s\n' "$(ltools_t menu.quit)"
        printf '%s' "$(ltools_t menu.prompt)"
        read -r choice || { printf '\n'; return 0; }
        case "$choice" in
            1) menu_audit ;;
            2) menu_games ;;
            3) menu_clean ;;
            4) menu_prefix ;;
            5) menu_defaults ;;
            6) menu_packages ;;
            7) run_module system menu ;;
            8) run_module host --doctor ;;
            h|H) usage ;;
            q|Q) IN_MENU=0; return 0 ;;
            *) printf '%s\n' "$(ltools_t menu.invalid)" ;;
        esac
    done
}

parse_args() {
    while (($#)); do
        case "$1" in
            --dry-run) GLOBAL_ARGS+=(--dry-run); shift ;;
            --plan)
                [[ $# -ge 2 ]] || die "--plan requiere un fichero"
                GLOBAL_ARGS+=(--plan "$2"); shift 2 ;;
            --rust) RUST_BACKEND=1; shift ;;
            --lang|--language)
                [[ $# -ge 2 ]] || die "--lang requiere un idioma"
                ltools_i18n_set "$2"
                export LTOOLS_LANG="$2"
                shift 2
                ;;
            --lang=*)
                ltools_i18n_set "${1#*=}"
                export LTOOLS_LANG="${1#*=}"
                shift
                ;;
            *) break ;;
        esac
    done
    [[ $# -gt 0 ]] || { menu; exit 0; }
    [[ "$1" == -- ]] && shift
    [[ $# -gt 0 ]] || die "falta un comando (usa --help)"
    case "$1" in
        menu|m)
            shift; [[ $# -eq 0 ]] || die "menu no acepta argumentos"
            if (( RUST_BACKEND )); then
                MODULE="rust"
                RUST_COMMAND="menu"
            else
                menu
                exit 0
            fi
            ;;
        audit|disk-audit) MODULE="audit"; shift ;;
        packages|pkg-audit|package-audit)
            if (( RUST_BACKEND )); then MODULE="packages"; else MODULE="audit"; fi
            PACKAGE_MODE=1; shift ;;
        games|game-audit|wine-audit) MODULE="games"; shift ;;
        clean|cleanup) MODULE="clean"; shift ;;
        system|services|systemctl|sys) MODULE="system"; shift ;;
        doctor|diagnose|fuse|fuse-check) MODULE="host"; shift ;;
        rollback|undo) MODULE="rollback"; shift ;;
        rust-audit|native-audit) MODULE="rust-audit"; shift ;;
        rust|native) MODULE="rust"; shift ;;
        defaults|paths|status)
            if (( RUST_BACKEND )); then
                MODULE="rust"
                RUST_COMMAND="defaults"
                shift
            else
                [[ $# -eq 1 ]] || die "defaults no acepta argumentos"
                show_defaults
                exit 0
            fi
            ;;
        prefix|wine)
            MODULE="prefix"
            shift
            if (( ! RUST_BACKEND )) && [[ "${1:-}" == list || "${1:-}" == ls ]]; then
                shift
                [[ $# -eq 0 ]] || die "prefix list no acepta argumentos"
                list_existing_prefixes
                exit 0
            fi
            if (( ! RUST_BACKEND )) && [[ "${1:-}" == batch || "${1:-}" == batch-migrate ]]; then
                shift
                [[ $# -eq 0 ]] || die "prefix batch no acepta argumentos"
                IN_MENU=1
                batch_migrate_prefixes
                exit 0
            fi
            if (( ! RUST_BACKEND )) && [[ "${1:-}" == inspect || "${1:-}" == info || "${1:-}" == details ]]; then
                shift
                [[ $# -eq 0 ]] || die "prefix inspect no acepta argumentos"
                inspect_prefix
                exit 0
            fi
            ;;
        --help|-h) usage; exit 0 ;;
        --version) printf '%s %s\n' "$SCRIPT_NAME" "$VERSION"; exit 0 ;;
        --module-help)
            [[ $# -ge 2 ]] || die "--module-help requiere un módulo"
            module_help "$2"
            ;;
        --menu) shift; [[ $# -eq 0 ]] || die "--menu no acepta argumentos"; menu; exit 0 ;;
        *) die "comando desconocido: $1 (usa --help)" ;;
    esac
    [[ "${1:-}" == -- ]] && shift
    MODULE_ARGS=("${GLOBAL_ARGS[@]}" "$@")
    if (( RUST_BACKEND )); then
        MODULE_ARGS=("${RUST_COMMAND:-$MODULE}" "${MODULE_ARGS[@]}")
        MODULE="rust"
    fi
    if (( PACKAGE_MODE )); then
        if (( RUST_BACKEND )); then
            MODULE_ARGS=("${MODULE_ARGS[0]}" --packages-only "${MODULE_ARGS[@]:1}")
        else
            MODULE_ARGS=(--packages-only "${MODULE_ARGS[@]}" )
        fi
    fi
}

main() {
    parse_args "$@"
    run_module "$MODULE" "${MODULE_ARGS[@]}"
}

main "$@"
