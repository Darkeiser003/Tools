#!/usr/bin/env bash
# Preflight de herramientas del sistema para el AppImage de LTools.

set -uo pipefail

host_have() { command -v "$1" >/dev/null 2>&1; }

host_fuse_available() {
    [[ -c /dev/fuse ]] || return 1
    host_have fusermount3 || host_have fusermount
}

host_fuse_package() {
    local manager="${1:-}"
    case "$manager" in
        pacman) printf 'fuse2' ;;
        apt-get|dnf|zypper|apk|xbps-install|pkg) printf 'fuse3' ;;
        brew) printf 'macfuse' ;;
        *) printf 'fuse3' ;;
    esac
}

host_fuse_report() {
    local manager package
    manager="$(host_manager 2>/dev/null || true)"
    package="$(host_fuse_package "$manager")"
    printf '=== FUSE para AppImage ===\n'
    if host_fuse_available; then
        printf '  OK      /dev/fuse y fusermount disponibles\n'
        [[ -e /dev/fuse ]] && ls -l /dev/fuse 2>/dev/null || true
        return 0
    fi
    if [[ -e /dev/fuse ]]; then
        printf '  AVISO   existe /dev/fuse, pero falta fusermount/fusermount3\n'
    else
        printf '  FALTA   /dev/fuse no está disponible en el sistema anfitrión\n'
    fi
    printf '  Solución recomendada para este sistema: paquete «%s»\n' "$package"
    printf '  Si el paquete ya está instalado, carga FUSE con «sudo modprobe fuse»\n'
    printf '  o utiliza el lanzador, que activa APPIMAGE_EXTRACT_AND_RUN automáticamente.\n'
    return 1
}

host_manager() {
    local manager
    for manager in pacman apt-get dnf zypper apk xbps-install brew pkg; do
        host_have "$manager" && { printf '%s' "$manager"; return 0; }
    done
    return 1
}

host_package_for() {
    case "$1" in
        findmnt) printf 'util-linux' ;;
        sha256sum) printf 'coreutils' ;;
        rsync) printf 'rsync' ;;
        jq) printf 'jq' ;;
        perl) printf 'perl' ;;
        wine|wineboot) printf 'wine' ;;
        winetricks) printf 'winetricks' ;;
        paccache) printf 'pacman-contrib' ;;
        systemctl|journalctl) printf 'systemd' ;;
        ps|kill) printf 'procps-ng' ;;
        gio) printf 'glib2' ;;
        *) printf '%s' "$1" ;;
    esac
}

host_package_for_manager() {
    local manager="$1" tool="$2"
    case "$manager:$tool" in
        apt-get:findmnt) printf 'util-linux' ;;
        apt-get:sha256sum) printf 'coreutils' ;;
        apt-get:systemctl|apt-get:journalctl) printf 'systemd' ;;
        apt-get:ps|apt-get:kill) printf 'procps' ;;
        apt-get:gio) printf 'libglib2.0-bin' ;;
        dnf:systemctl|dnf:journalctl) printf 'systemd' ;;
        dnf:ps|dnf:kill) printf 'procps-ng' ;;
        apk:findmnt) printf 'util-linux' ;;
        apk:sha256sum) printf 'coreutils' ;;
        apk:systemctl|apk:journalctl) printf 'systemd' ;;
        xbps-install:ps|xbps-install:kill) printf 'procps-ng' ;;
        pkg:findmnt) printf 'util-linux' ;;
        *) host_package_for "$tool" ;;
    esac
}

host_install() {
    local tool="$1" package="$2" manager
    local -a command
    manager="$(host_manager 2>/dev/null || true)"
    if [[ -z "$manager" ]]; then
        printf 'No se encontró pacman, apt, dnf, zypper, apk, xbps, brew ni pkg.\n' >&2
        return 1
    fi
    case "$manager" in
        pacman) command=(sudo pacman -S --needed "$package") ;;
        apt-get) command=(sudo apt-get install "$package") ;;
        dnf) command=(sudo dnf install "$package") ;;
        zypper) command=(sudo zypper install "$package") ;;
        apk) command=(sudo apk add "$package") ;;
        xbps-install) command=(sudo xbps-install -S "$package") ;;
        brew) command=(brew install "$package") ;;
        pkg) command=(sudo pkg install "$package") ;;
        *) return 1 ;;
    esac
    printf 'Se ejecutará mediante %s: %q' "$manager" "${command[0]}"
    printf ' %q' "${command[@]:1}"
    printf '\n'
    "${command[@]}"
    if [[ "$tool" == fuse ]]; then
        host_fuse_available
    else
        host_have "$tool"
    fi
}

host_offer_fuse() {
    local dry_run="${1:-0}" manager package answer
    host_fuse_available && return 0
    manager="$(host_manager 2>/dev/null || true)"
    package="$(host_fuse_package "$manager")"
    printf 'FUSE no está disponible; AppImage puede ejecutarse mediante extracción temporal.\n' >&2
    printf 'Paquete sugerido: %s\n' "$package" >&2
    [[ "$dry_run" -eq 1 ]] && { printf 'Simulación: no se ofrecerá instalar FUSE.\n' >&2; return 0; }
    if [[ ! -t 0 || ! -t 1 ]]; then
        printf 'Modo no interactivo: no se instalará FUSE automáticamente.\n' >&2
        return 0
    fi
    printf '¿Instalar %s mediante el gestor del sistema? [y/N] ' "$package"
    read -r answer || return 1
    case "${answer,,}" in
        y|yes|s|si|sí) host_install fuse "$package" || return 1; host_fuse_report ;;
        *) printf 'Instalación de FUSE omitida; se utilizará el modo de extracción.\n' >&2; return 0 ;;
    esac
}

host_offer_missing() {
    local tool="$1" dry_run="${2:-0}" package answer manager
    host_have "$tool" && return 0
    manager="$(host_manager 2>/dev/null || true)"
    package="$(host_package_for_manager "$manager" "$tool")"
    printf 'Falta la herramienta opcional «%s» (paquete sugerido: %s).\n' "$tool" "$package" >&2
    if [[ "$dry_run" -eq 1 ]]; then
        printf 'Simulación: no se ofrecerá instalarla.\n' >&2
        return 0
    fi
    if [[ ! -t 0 || ! -t 1 ]]; then
        printf 'Modo no interactivo: no se instalará nada automáticamente.\n' >&2
        return 0
    fi
    printf '¿Instalar %s mediante el gestor del sistema? [y/N] ' "$package"
    read -r answer || return 1
    case "${answer,,}" in
        y|yes|s|si|sí) host_install "$tool" "$package" ;;
        *) printf 'Instalación omitida; la operación puede quedar limitada.\n' >&2; return 0 ;;
    esac
}

host_doctor() {
    local install_missing=0 tool package manager
    manager="$(host_manager 2>/dev/null || true)"
    for arg in "$@"; do
        [[ "$arg" == --install-missing ]] && install_missing=1
    done
    printf '=== Dependencias del sistema ===\n'
    for tool in findmnt sha256sum rsync jq perl wine wineboot winetricks paccache systemctl journalctl ps gio; do
        package="$(host_package_for_manager "$manager" "$tool")"
        if host_have "$tool"; then
            printf '  OK      %-12s %s\n' "$tool" "$(command -v "$tool")"
        else
            printf '  FALTA   %-12s paquete sugerido: %s\n' "$tool" "$package"
            [[ "$install_missing" -eq 1 ]] && host_offer_missing "$tool" 0
        fi
    done
    host_fuse_report || [[ "$install_missing" -eq 1 ]] && host_offer_fuse 0
}

host_preflight() {
    local dry_run=0 arg command='' skip_next=0
    local -a required=()
    for arg in "$@"; do
        if [[ "$skip_next" -eq 1 ]]; then
            skip_next=0
            continue
        fi
        [[ "$arg" == --dry-run ]] && dry_run=1
        case "$arg" in
            --plan|--out|--root|--min-size-mb|--source|--from|--dest|--destination|--target|--include|--exclude|--package|--path|--manager|--arch|--unit)
                skip_next=1
                continue
                ;;
        esac
        [[ -z "$command" && "$arg" != -* ]] && command="$arg"
    done
    case "$command" in
        audit|disk-audit) required+=(findmnt sha256sum) ;;
        games|game-audit|wine-audit) required+=(findmnt sha256sum) ;;
        packages|pkg-audit|package-audit) ;;
        clean|cleanup)
            required+=(findmnt)
            ;;
        prefix|wine)
            [[ " $* " == *' create '* ]] && required+=(wineboot)
            [[ " $* " == *' migrate '* || " $* " == *' clone '* ]] && required+=(rsync)
            ;;
        system|services|systemctl) required+=(systemctl) ;;
        defaults|paths|rollback|undo) ;;
    esac
    local -A seen=()
    for arg in "${required[@]}"; do
        [[ "${seen[$arg]:-0}" -eq 1 ]] && continue
        seen["$arg"]=1
        host_offer_missing "$arg" "$dry_run"
    done
    if [[ "$command" == prefix || "$command" == wine ]] &&
        [[ " $* " == *' --update-launchers '* ]] &&
        ! host_have jq && ! host_have perl; then
        host_offer_missing jq "$dry_run"
    fi
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    case "${1:-}" in
        --fuse-check|fuse|fuse-check)
            host_fuse_report
            ;;
        --doctor|doctor|"")
            shift || true
            host_doctor "$@"
            ;;
        *)
            printf 'Uso: %s [--doctor [--install-missing]|--fuse-check]\n' "$0"
            exit 2
            ;;
    esac
fi
