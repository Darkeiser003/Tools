#!/usr/bin/env bash

# disk-clean.sh - interactive, guarded cleanup for CachyOS/Arch.
# Package operations use pacman. User files go to the desktop trash by default.

set -uo pipefail
export LC_ALL=C

VERSION="0.3.0"
SCRIPT_NAME="$(basename "$0")"
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd -P)"
HOME_DIR="${HOME:-$(getent passwd "$(id -u)" 2>/dev/null | cut -d: -f6)}"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib/ltools-plan.sh"
REPORT_DIR=""
FORCE=0
MENU=0
PACMAN_CACHE=0
FLATPAK_UNUSED=0
ORPHANS=0
FOREIGN=0
EXPLICIT=0
PACKAGE_CACHES=0
PACKAGE_ARTIFACTS=0
ROLLBACK_PATH=""
declare -a PACKAGE_REQUESTS=()
declare -a PATH_REQUESTS=()

usage() {
    cat <<EOF
Uso: $SCRIPT_NAME [opciones]

Limpiador interactivo y protegido para CachyOS/Arch.
No borra nada por defecto: los archivos se mueven a la papelera.

Acciones:
  --menu                    Abre el menú interactivo.
  --package PAQUETE         Solicita la eliminación de un paquete; se puede repetir.
  --orphans                 Selecciona paquetes huérfanos detectados actualmente.
  --foreign                 Selecciona paquetes AUR/manuales detectados actualmente.
  --explicit                Selecciona paquetes explícitos de pacman para revisión.
  --path RUTA               Selecciona una ruta concreta para moverla a la papelera.
  --report DIRECTORIO       Usa un informe de disk-audit.sh en el menú de rutas.
  --pacman-cache             Ejecuta paccache -rk2 tras confirmar.
  --flatpak-unused           Retira runtimes Flatpak sin uso tras confirmar.
  --package-caches           Detecta y ofrece limpiar cachés de los gestores presentes.
  --package-artifacts        Revisa archivos de paquetes del informe indicado.
  --dry-run                  Solo calcula y guarda el plan; no modifica nada.
  --plan FICHERO             Guarda el plan en una ruta concreta.
  --rollback FICHERO         Revierte las operaciones reversibles de un plan.
  --force                    Permite continuar pese a referencias de configuración,
                            pero mantiene las confirmaciones y bloqueos críticos.
  -h, --help                Muestra esta ayuda.

Ejemplos:
  $SCRIPT_NAME --menu --report "$HOME/Informes/disk-audit-..."
  $SCRIPT_NAME --package electron41
  $SCRIPT_NAME --path "$HOME/.cache/paru/clone"
  $SCRIPT_NAME --orphans
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
    local expected="$1" answer
    printf 'Escribe "%s" para confirmar: ' "$expected"
    read -r answer || return 1
    [[ "$answer" == "$expected" ]]
}

parse_args() {
    while (($#)); do
        case "$1" in
            --menu) MENU=1; shift ;;
            --package)
                [[ $# -ge 2 ]] || die "--package requiere un nombre"
                PACKAGE_REQUESTS+=("$2"); shift 2 ;;
            --path)
                [[ $# -ge 2 ]] || die "--path requiere una ruta"
                PATH_REQUESTS+=("$2"); shift 2 ;;
            --report)
                [[ $# -ge 2 ]] || die "--report requiere un directorio"
                REPORT_DIR="$2"; shift 2 ;;
            --orphans) ORPHANS=1; shift ;;
            --foreign) FOREIGN=1; shift ;;
            --explicit) EXPLICIT=1; shift ;;
            --pacman-cache) PACMAN_CACHE=1; shift ;;
            --flatpak-unused) FLATPAK_UNUSED=1; shift ;;
            --package-caches) PACKAGE_CACHES=1; shift ;;
            --package-artifacts) PACKAGE_ARTIFACTS=1; shift ;;
            --dry-run) DRY_RUN=1; shift ;;
            --plan)
                [[ $# -ge 2 ]] || die "--plan requiere un fichero"
                PLAN_PATH="$2"; shift 2 ;;
            --rollback)
                [[ $# -ge 2 ]] || die "--rollback requiere un fichero"
                ROLLBACK_PATH="$2"; shift 2 ;;
            --force) FORCE=1; shift ;;
            -h|--help) usage; exit 0 ;;
            *) die "opción desconocida: $1" ;;
        esac
    done
}

validate_package_name() {
    [[ "$1" =~ ^[[:alnum:]@._+:-]+$ ]]
}

package_required_by() {
    pacman -Qi -- "$1" 2>/dev/null | awk -F: '
        /^Required By[[:space:]]*:/ {
            value=$2
            sub(/^[[:space:]]*/, "", value)
            print value
            exit
        }
    '
}

remove_package() {
    local package="$1" required
    validate_package_name "$package" || { printf 'Paquete ignorado por nombre no válido: %s\n' "$package" >&2; return 1; }
    have pacman || { printf 'pacman no está disponible.\n' >&2; return 1; }
    pacman -Q -- "$package" >/dev/null 2>&1 || { printf 'No está instalado: %s\n' "$package"; return 1; }

    printf '\nPaquete solicitado: %s\n' "$package"
    pacman -Qi -- "$package" 2>/dev/null | awk -F: '/^(Name|Version|Install Reason|Installed Size|Required By)[[:space:]]*:/ {print}'
    required="$(package_required_by "$package")"

    if [[ -n "$required" && "$required" != "None" ]]; then
        printf '\nHay paquetes que dependen de %s:\n' "$package"
        if have pactree; then
            pactree -r -- "$package" 2>/dev/null || true
        else
            printf '%s\n' "$required"
        fi
        printf '\nNo se eliminará automáticamente un paquete con dependientes.\n'
        printf 'Opciones: cancela, elimina primero los dependientes, o usa una cascada explícita.\n'
        if (( DRY_RUN )); then
            plan_record package-remove "$package" planned no cascade
            printf 'Simulación: se necesitaría una cascada explícita; no se ejecutará pacman.\n'
            return 0
        fi
        if ! ask_phrase "CASCADE $package"; then
            printf 'Cancelado: %s\n' "$package"
            return 0
        fi
        printf 'La cascada puede eliminar también paquetes explícitos que dependan de él.\n'
        confirm "¿Quieres mostrar y aceptar la transacción de pacman?" || { printf 'Cancelado.\n'; return 0; }
        if sudo pacman -Rns -c -- "$package"; then
            plan_record package-remove "$package" executed no cascade
            return 0
        fi
        return 1
    fi

    printf 'No hay dependientes registrados por pacman. Esto no garantiza que un programa externo no lo use directamente.\n'
    if (( DRY_RUN )); then
        plan_record package-remove "$package" planned no "pacman -Rns"
        printf 'Simulación: se propondría eliminar %s y dependencias no requeridas.\n' "$package"
        return 0
    fi
    if confirm "¿Eliminar $package y dependencias que queden sin uso?"; then
        if sudo pacman -Rns -- "$package"; then
            plan_record package-remove "$package" executed no "pacman -Rns"
            return 0
        fi
        return 1
    else
        printf 'Cancelado: %s\n' "$package"
    fi
}

select_package_list() {
    local title="$1" query="$2" line answer token index
    local -a packages=() choices=()
    mapfile -t packages < <(pacman $query 2>/dev/null | sort -u)
    ((${#packages[@]})) || { printf '\n%s: ninguno.\n' "$title"; return 0; }

    printf '\n%s:\n' "$title"
    for index in "${!packages[@]}"; do
        printf '  %d) %s\n' "$((index + 1))" "${packages[index]}"
    done
    printf 'Selecciona números separados por espacios, "all" o "q": '
    read -r answer || return 0
    [[ "$answer" == q || "$answer" == Q ]] && return 0
    if [[ "$answer" == all ]]; then
        choices=($(seq 1 "${#packages[@]}"))
    else
        read -ra choices <<<"$answer"
    fi
    for token in "${choices[@]}"; do
        [[ "$token" =~ ^[0-9]+$ ]] || { printf 'Selección ignorada: %s\n' "$token"; continue; }
        index=$((token - 1))
        (( index >= 0 && index < ${#packages[@]} )) || { printf 'Número fuera de rango: %s\n' "$token"; continue; }
        remove_package "${packages[index]}"
    done
}

report_path() {
    local file="$1"
    [[ -n "$REPORT_DIR" && -f "$REPORT_DIR/$file" ]] && printf '%s' "$REPORT_DIR/$file"
}

report_line_path() {
    local file="$1" line="$2"
    case "$(basename "$file")" in
        wine-prefixes.tsv)
            # Both audit formats keep the complete prefix in column 10 and
            # drive_c in the final column. Always select the complete prefix.
            printf '%s' "$(printf '%s\n' "$line" | awk -F'\t' '{print $10}')"
            ;;
        *)
            printf '%s' "${line##*$'\t'}"
            ;;
    esac
}

find_references() {
    local path="$1" root
    local -a roots=()
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
        rg -F -l --hidden --no-messages \
            --glob '!cache/**' --glob '!Cache/**' --glob '!Trash/**' \
            -- "$path" "${roots[@]}" 2>/dev/null | head -30
    else
        grep -R -F -l -- "$path" "${roots[@]}" 2>/dev/null | head -30
    fi
}

safe_realpath() {
    realpath -e -- "$1" 2>/dev/null
}

is_critical_path() {
    local path="$1" mountpoint
    case "$path" in
        /|/home|"$HOME_DIR"|/mnt|/media|/run/media|/opt|/var|/usr|/bin|/lib|/lib64|/etc|/boot)
            return 0 ;;
        /home/*|/mnt/*|/media/*|/run/media/*|/tmp/*)
            ;;
        *)
            return 0 ;;
    esac
    case "$path" in
        */files/share/default_pfx|*/files/share/default_pfx/*)
            return 0 ;;
        */steamapps/compatdata|*/steamapps/common|*/steamapps|*/steamapps/)
            return 0 ;;
    esac
    if have findmnt; then
        mountpoint="$(findmnt -rn -T "$path" -o TARGET 2>/dev/null | head -1)"
        [[ "$mountpoint" == "$path" ]] && return 0
    fi
    return 1
}

move_path_to_trash() {
    local requested="$1" path references trash_copy
    path="$(safe_realpath "$requested")" || { printf 'No existe o no es legible: %s\n' "$requested"; return 1; }
    [[ -L "$requested" ]] && { printf 'Bloqueado: no se siguen enlaces simbólicos: %s\n' "$requested"; return 1; }
    if is_critical_path "$path"; then
        printf 'Bloqueado por seguridad: ruta crítica, punto de montaje o runtime compartido: %s\n' "$path"
        return 1
    fi
    case "$path" in
        /var/cache/*|/var/lib/snapd/snaps/*)
            printf 'Bloqueado: es un almacén de paquetes del sistema. Usa --package-caches para que el gestor lo limpie con su método nativo: %s\n' "$path"
            return 1
            ;;
    esac

    printf '\nRuta seleccionada:\n  %s\n' "$path"
    references="$(find_references "$path")"
    if [[ -n "$references" ]]; then
        printf 'Se encontraron posibles referencias en:\n%s\n' "$references"
        if (( ! FORCE )); then
            printf 'No se mueve. Revisa esas referencias o repite con --force.\n'
            return 1
        fi
        printf 'Se continuará porque se indicó --force.\n'
    fi

    if (( DRY_RUN )); then
        plan_record trash-move "$path" planned yes "$(dirname -- "$path")" ""
        printf 'Simulación: se movería a la papelera: %s\n' "$path"
        return 0
    fi

    if ! have gio && ! have trash-put; then
        printf 'No hay gio ni trash-put; no se elimina nada.\n'
        return 1
    fi
    if ! confirm "¿Mover a la papelera?"; then
        printf 'Cancelado.\n'
        return 0
    fi
    if have gio; then
        gio trash -- "$path" || return 1
    else
        trash-put -- "$path" || return 1
    fi
    trash_copy="$(plan_trash_guess "$path")"
    plan_record trash-move "$path" executed yes "$trash_copy" ""
}

select_report_paths() {
    local title="$1" filename="$2" file line path answer token index kind human scope
    local -a lines=() choices=()
    file="$(report_path "$filename")"
    [[ -n "$file" ]] || { printf 'No está disponible: %s\n' "$filename"; return 0; }
    mapfile -t lines < <(tail -n +2 "$file")
    ((${#lines[@]})) || { printf '\n%s: ninguno.\n' "$title"; return 0; }
    printf '\n%s (%s):\n' "$title" "$filename"
    for index in "${!lines[@]}"; do
        path="$(report_line_path "$file" "${lines[index]}")"
        if [[ "$filename" == package-artifacts.tsv ]]; then
            IFS=$'\t' read -r kind scope _ human path <<<"${lines[index]}"
            printf '  %d) %-20s %-12s %s\n' "$((index + 1))" "$kind" "$human" "$path"
        else
            printf '  %d) %s\n' "$((index + 1))" "$path"
        fi
    done
    printf 'Selecciona números separados por espacios, "all" o "q": '
    read -r answer || return 0
    [[ "$answer" == q || "$answer" == Q ]] && return 0
    if [[ "$answer" == all ]]; then
        choices=($(seq 1 "${#lines[@]}"))
    else
        read -ra choices <<<"$answer"
    fi
    for token in "${choices[@]}"; do
        [[ "$token" =~ ^[0-9]+$ ]] || { printf 'Selección ignorada: %s\n' "$token"; continue; }
        index=$((token - 1))
        (( index >= 0 && index < ${#lines[@]} )) || { printf 'Número fuera de rango: %s\n' "$token"; continue; }
        path="$(report_line_path "$file" "${lines[index]}")"
        move_path_to_trash "$path"
    done
}

report_menu() {
    [[ -n "$REPORT_DIR" && -d "$REPORT_DIR" ]] || { printf 'Indica --report con un informe de disk-audit.sh.\n'; return 0; }
    while true; do
        cat <<'EOF'

Rutas del informe:
  1) Artefactos de desarrollo
  2) AppImages
  3) Instaladores y archivos comprimidos
  4) Máquinas virtuales
  5) Cachés y papeleras
  6) Archivos grandes
  7) Duplicados exactos
  8) Prefijos Wine/Proton (revisión cuidadosa)
  9) Archivos de paquetes descargados
  0) Volver
EOF
        local choice
        printf 'Opción: '
        read -r choice || return 0
        case "$choice" in
            1) select_report_paths "Artefactos de desarrollo" build-artifacts.tsv ;;
            2) select_report_paths "AppImages" appimages.tsv ;;
            3) select_report_paths "Instaladores y archivos" installers-and-archives.tsv ;;
            4) select_report_paths "Máquinas virtuales" virtual-machines.tsv ;;
            5) select_report_paths "Cachés y papeleras" caches-and-trash.tsv ;;
            6) select_report_paths "Archivos grandes" large-files.tsv ;;
            7) select_report_paths "Duplicados exactos: elige solo las copias que quieras conservar fuera" duplicates.tsv ;;
            8) select_report_paths "Prefijos Wine/Proton" wine-prefixes.tsv ;;
            9) select_report_paths "Archivos de paquetes descargados" package-artifacts.tsv ;;
            0) return 0 ;;
            *) printf 'Opción no válida.\n' ;;
        esac
    done
}

print_store_status() {
    local label path size
    label="$1"; path="$2"
    [[ -e "$path" ]] || return 0
    size="$(du -sh -- "$path" 2>/dev/null | awk '{print $1}')"
    printf '  %-12s %8s  %s\n' "$label" "${size:--}" "$path"
}

run_package_caches() {
    local cache manager
    printf '\nGestores y almacenes detectados en este equipo:\n'
    for manager in pacman paru yay pikaur trizen aura pamac apt apt-get aptitude nala dpkg rpm dnf yum zypper apk xbps-query xbps-remove pkg snap flatpak brew nix-env; do
        if have "$manager"; then
            printf '  - %s: instalado\n' "$manager"
        fi
    done
    printf '\nAlmacenes existentes (ámbito, tamaño y ruta):\n'
    print_store_status 'pacman/sistema' /var/cache/pacman/pkg
    print_store_status 'pamac/sistema' /var/cache/pamac/pkg
    print_store_status 'apt/sistema' /var/cache/apt/archives
    print_store_status 'dnf/sistema' /var/cache/dnf
    print_store_status 'yum/sistema' /var/cache/yum
    print_store_status 'zypper/sistema' /var/cache/zypp
    print_store_status 'apk/sistema' /var/cache/apk
    print_store_status 'xbps/sistema' /var/cache/xbps
    print_store_status 'pkg/sistema' /var/cache/pkg
    print_store_status 'snap/sistema' /var/lib/snapd/snaps
    print_store_status 'flatpak/sistema' /var/lib/flatpak
    if [[ -d "$HOME_DIR/.local/share/flatpak" ]]; then
        print_store_status 'flatpak/usuario' "$HOME_DIR/.local/share/flatpak"
    fi
    for manager in paru yay pikaur trizen aura; do
        print_store_status "$manager/usuario" "$HOME_DIR/.cache/$manager"
    done
    if have brew; then
        print_store_status 'brew/usuario' "$(brew --cache 2>/dev/null || printf '%s' "$HOME_DIR/.cache/Homebrew")"
    fi

    if (( DRY_RUN )); then
        for cache in /var/cache/pacman/pkg /var/cache/apt/archives /var/cache/dnf /var/cache/yum /var/cache/zypp /var/cache/apk /var/cache/xbps /var/cache/pkg /var/lib/flatpak "$HOME_DIR/.cache/paru" "$HOME_DIR/.cache/yay"; do
            [[ -e "$cache" ]] || continue
            plan_record package-cache-clean "$cache" planned no native-manager
        done
        printf 'Simulación: se han añadido al plan las cachés detectadas; no se ejecutará ningún gestor.\n'
        return 0
    fi

    if have paccache; then
        if confirm '¿Limpiar la caché de pacman conservando las 2 últimas versiones?'; then
            if sudo paccache -rk2; then
                plan_record package-cache-clean /var/cache/pacman/pkg executed no paccache
            fi
        fi
    elif have pacman; then
        printf 'paccache no está instalado; no se tocará /var/cache/pacman/pkg automáticamente.\n'
    fi

    for manager in paru yay pikaur trizen aura; do
        cache="$HOME_DIR/.cache/$manager"
        [[ -d "$cache" ]] || continue
        printf '\nCaché de construcción de %s: %s\n' "$manager" "$cache"
        if confirm '¿Mover esta caché a la papelera?'; then
            move_path_to_trash "$cache"
        fi
    done

    if have apt-get && [[ -d /var/cache/apt/archives ]]; then
        if confirm '¿Ejecutar la limpieza de caché de APT?' && sudo apt-get clean; then
            plan_record package-cache-clean /var/cache/apt/archives executed no apt-get-clean
        fi
    fi
    if have dnf; then
        if confirm '¿Ejecutar la limpieza de caché de DNF?' && sudo dnf clean all; then
            plan_record package-cache-clean /var/cache/dnf executed no dnf-clean
        fi
    elif have yum; then
        if confirm '¿Ejecutar la limpieza de caché de YUM?' && sudo yum clean all; then
            plan_record package-cache-clean /var/cache/yum executed no yum-clean
        fi
    fi
    if have zypper; then
        if confirm '¿Ejecutar la limpieza de caché de Zypper?' && sudo zypper clean --all; then
            plan_record package-cache-clean /var/cache/zypp executed no zypper-clean
        fi
    fi
    if have apk && [[ -d /var/cache/apk ]]; then
        if confirm '¿Limpiar la caché de APK?' && sudo apk cache clean; then
            plan_record package-cache-clean /var/cache/apk executed no apk-clean
        fi
    fi
    if have xbps-remove; then
        if confirm '¿Limpiar paquetes descargados de XBPS?' && sudo xbps-remove -O; then
            plan_record package-cache-clean /var/cache/xbps executed no xbps-clean
        fi
    fi
    if have pkg && [[ -d /var/cache/pkg ]]; then
        if confirm '¿Limpiar la caché de pkg?' && sudo pkg clean -a; then
            plan_record package-cache-clean /var/cache/pkg executed no pkg-clean
        fi
    fi
    if have brew; then
        if confirm '¿Ejecutar brew cleanup para sus paquetes de usuario?' && brew cleanup -s; then
            plan_record package-cache-clean "$(brew --cache 2>/dev/null || printf '%s' "$HOME_DIR/.cache/Homebrew")" executed no brew-clean
        fi
    fi
    if have snap && [[ -d /var/lib/snapd/snaps ]]; then
        printf 'Snap conserva revisiones instaladas; no se borrarán manualmente. Usa la gestión de Snap si quieres retirarlas.\n'
    fi
    if have flatpak; then
        if confirm '¿Eliminar runtimes Flatpak sin uso?' && flatpak uninstall --unused; then
            plan_record flatpak-unused flatpak executed no "flatpak uninstall --unused"
        fi
    fi
    if have nix-store; then
        printf 'Nix detectado: no se ejecutará una recolección automática sin revisar generaciones.\n'
    fi
}

run_pacman_cache() {
    have paccache || { printf 'paccache no está disponible. Instala pacman-contrib si lo deseas.\n'; return 1; }
    if (( DRY_RUN )); then
        plan_record package-cache-clean /var/cache/pacman/pkg planned no paccache
        printf 'Simulación: se ejecutaría paccache -rk2.\n'
        return 0
    fi
    confirm "¿Limpiar la caché de pacman conservando las 2 últimas versiones?" || { printf 'Cancelado.\n'; return 0; }
    sudo paccache -rk2
}

run_flatpak_unused() {
    have flatpak || { printf 'flatpak no está disponible.\n'; return 1; }
    if (( DRY_RUN )); then
        plan_record flatpak-unused flatpak planned no "flatpak uninstall --unused"
        printf 'Simulación: se ejecutarían las eliminaciones de runtimes Flatpak sin uso.\n'
        return 0
    fi
    confirm "¿Eliminar runtimes y extensiones Flatpak sin uso?" || { printf 'Cancelado.\n'; return 0; }
    flatpak uninstall --unused
}

main_menu() {
    while true; do
        cat <<'EOF'

disk-clean: menú principal
  1) Paquetes huérfanos
  2) Paquetes AUR/manuales
  3) Paquetes explícitos de pacman
  4) Rutas del informe
  5) Introducir paquete concreto
  6) Introducir ruta concreta
  7) Limpiar almacenes/cachés detectados
  8) Limpiar runtimes Flatpak sin uso
  0) Salir
EOF
        local choice value
        printf 'Opción: '
        read -r choice || return 0
        case "$choice" in
            1) select_package_list "Paquetes huérfanos" "-Qdtq" ;;
            2) select_package_list "Paquetes AUR/manuales" "-Qmq" ;;
            3) select_package_list "Paquetes explícitos de pacman" "-Qqe" ;;
            4) report_menu ;;
            5) printf 'Nombre del paquete: '; read -r value && remove_package "$value" ;;
            6) printf 'Ruta: '; read -r value && move_path_to_trash "$value" ;;
            7) run_package_caches ;;
            8) run_flatpak_unused ;;
            0) return 0 ;;
            *) printf 'Opción no válida.\n' ;;
        esac
    done
}

main() {
    parse_args "$@"
    if [[ -n "${ROLLBACK_PATH:-}" ]]; then
        rollback_plan "$ROLLBACK_PATH"
        exit $?
    fi
    plan_init "$SCRIPT_NAME" || die "no se pudo crear el plan: ${PLAN_PATH:-desconocido}"
    if (( MENU )); then main_menu; fi
    if (( ORPHANS )); then select_package_list "Paquetes huérfanos" "-Qdtq"; fi
    if (( FOREIGN )); then select_package_list "Paquetes AUR/manuales" "-Qmq"; fi
    if (( EXPLICIT )); then select_package_list "Paquetes explícitos de pacman" "-Qqe"; fi
    for package in "${PACKAGE_REQUESTS[@]}"; do remove_package "$package"; done
    for path in "${PATH_REQUESTS[@]}"; do move_path_to_trash "$path"; done
    if (( PACMAN_CACHE )); then run_pacman_cache; fi
    if (( FLATPAK_UNUSED )); then run_flatpak_unused; fi
    if (( PACKAGE_CACHES )); then run_package_caches; fi
    if (( PACKAGE_ARTIFACTS )); then select_report_paths "Archivos de paquetes descargados" package-artifacts.tsv; fi
    if (( ! MENU && ! ORPHANS && ! FOREIGN && ! EXPLICIT && ${#PACKAGE_REQUESTS[@]} == 0 && ${#PATH_REQUESTS[@]} == 0 && ! PACMAN_CACHE && ! FLATPAK_UNUSED && ! PACKAGE_CACHES && ! PACKAGE_ARTIFACTS )); then
        usage
    fi
    printf '\nPlan registrado en: %s\n' "$PLAN_PATH"
}

main "$@"
