#!/usr/bin/env bash

# disk-audit.sh - read-only inventory for CachyOS/Arch desktops and related stores.
# It never removes, moves, or changes files.

set -uo pipefail
export LC_ALL=C

VERSION="0.3.0"
SCRIPT_NAME="$(basename "$0")"
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd -P)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib/ltools-plan.sh"
HOSTNAME_SHORT="$(hostname -s 2>/dev/null || hostname 2>/dev/null || printf 'unknown')"
HOME_DIR="${HOME:-$(getent passwd "$(id -u)" 2>/dev/null | cut -d: -f6)}"
XDG_CACHE_DIR="${XDG_CACHE_HOME:-$HOME_DIR/.cache}"
XDG_DATA_DIR="${XDG_DATA_HOME:-$HOME_DIR/.local/share}"
XDG_DOWNLOAD_DIR="$HOME_DIR/Downloads"
USER_DIRS_FILE="${XDG_CONFIG_HOME:-$HOME_DIR/.config}/user-dirs.dirs"
if [[ -r "$USER_DIRS_FILE" ]]; then
    XDG_DOWNLOAD_DIR="$(sed -n 's/^XDG_DOWNLOAD_DIR="\(.*\)"/\1/p' "$USER_DIRS_FILE" | head -1)"
    XDG_DOWNLOAD_DIR="${XDG_DOWNLOAD_DIR/\$HOME/$HOME_DIR}"
    [[ -n "$XDG_DOWNLOAD_DIR" ]] || XDG_DOWNLOAD_DIR="$HOME_DIR/Downloads"
fi
PACKAGE_SCAN_ROOTS=()

# These can be overridden in ~/.config/disk-audit.conf or with command-line flags.
AUTO_MOUNT_ROOTS=1
INCLUDE_HOME=1
DUPLICATE_MIN_MB=50
TOP_N=100
SCAN_ROOTS=()
EXTRA_SCAN_ROOTS=()

MODE="quick"
DO_DUPLICATES=0
PACKAGES_ONLY=0
OUT_DIR=""
DRY_RUN=0
PLAN_PATH=""
CONFIG_FILE="${XDG_CONFIG_HOME:-$HOME_DIR/.config}/disk-audit.conf"

# Test/portable mode: callers can disable discovery of host mount points while
# still supplying explicit roots with --root. This keeps automated checks fast
# and prevents an isolated audit from touching unrelated game disks.
[[ "${LTOOLS_NO_MOUNTS:-0}" =~ ^(1|true|yes|si|sí)$ ]] && AUTO_MOUNT_ROOTS=0

usage() {
    cat <<EOF
Uso: $SCRIPT_NAME [opciones]

Auditoría de almacenamiento, aplicaciones y gestores para CachyOS/Arch y otros sistemas. Solo lee datos.

Opciones:
  --full                    Escanea más rutas y genera listados más completos.
  --packages-only           Solo inventario de gestores, paquetes y artefactos.
  --duplicates              Busca duplicados mediante tamaño y SHA-256.
  --min-size-mb N           Tamaño mínimo para duplicados (por defecto: $DUPLICATE_MIN_MB).
  --root RUTA               Añade una ruta al escaneo; se puede repetir.
  --no-home                 No añade el directorio personal ni sus datos conocidos.
  --no-mounts               No añade automáticamente montajes bajo /mnt, /media o /run/media.
  --out RUTA                Directorio del informe.
  --config RUTA             Carga otra configuración.
  --dry-run                 Mantiene explícita la auditoría de solo lectura.
  --plan FICHERO            Registra la auditoría en un plan.
  --version                 Muestra la versión.
  -h, --help                Muestra esta ayuda.

Ejemplos:
  $SCRIPT_NAME
  $SCRIPT_NAME --full --duplicates --min-size-mb 100
  $SCRIPT_NAME --root /mnt/JuegosLinux --out ~/Informes/disco

El modo --packages-only limita el escaneo a almacenes de paquetes conocidos,
rutas XDG, descargas habituales, /tmp y las rutas extra indicadas con --root.

La configuración opcional se busca en:
  $CONFIG_FILE
EOF
}

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 2
}

have() {
    command -v "$1" >/dev/null 2>&1
}

human_bytes() {
    local bytes="${1:-0}"
    if have numfmt; then
        numfmt --to=iec --suffix=B "$bytes" 2>/dev/null || printf '%s B' "$bytes"
    else
        printf '%s B' "$bytes"
    fi
}

safe_name() {
    printf '%s' "$1" | sed 's#[^A-Za-z0-9_.-]#_#g'
}

add_root() {
    local candidate="$1"
    [[ -d "$candidate" ]] || return 0
    candidate="$(realpath -m -- "$candidate" 2>/dev/null || printf '%s' "$candidate")"
    local root
    for root in "${SCAN_ROOTS[@]}"; do
        [[ "$root" == "$candidate" ]] && return 0
    done
    SCAN_ROOTS+=("$candidate")
}

load_config() {
    [[ -f "$CONFIG_FILE" ]] || return 0
    # This is an optional user-owned shell config, intentionally sourced.
    # shellcheck disable=SC1090
    source "$CONFIG_FILE"
}

parse_args() {
    while (($#)); do
        case "$1" in
            --full) MODE="full"; shift ;;
            --packages-only) PACKAGES_ONLY=1; MODE="packages"; shift ;;
            --duplicates) DO_DUPLICATES=1; shift ;;
            --min-size-mb)
                [[ $# -ge 2 && "$2" =~ ^[0-9]+$ ]] || die "--min-size-mb requiere un entero"
                DUPLICATE_MIN_MB="$2"; shift 2 ;;
            --root)
                [[ $# -ge 2 ]] || die "--root requiere una ruta"
                EXTRA_SCAN_ROOTS+=("$2"); shift 2 ;;
            --no-home) INCLUDE_HOME=0; shift ;;
            --no-mounts) AUTO_MOUNT_ROOTS=0; shift ;;
            --out)
                [[ $# -ge 2 ]] || die "--out requiere una ruta"
                OUT_DIR="$2"; shift 2 ;;
            --config)
                [[ $# -ge 2 ]] || die "--config requiere una ruta"
                CONFIG_FILE="$2"; shift 2 ;;
            --dry-run) DRY_RUN=1; shift ;;
            --plan)
                [[ $# -ge 2 ]] || die "--plan requiere un fichero"
                PLAN_PATH="$2"; shift 2 ;;
            --version) printf '%s %s\n' "$SCRIPT_NAME" "$VERSION"; exit 0 ;;
            -h|--help) usage; exit 0 ;;
            *) die "opción desconocida: $1" ;;
        esac
    done
}

prepare_roots() {
    local root target
    if (( INCLUDE_HOME )); then
        add_root "$HOME_DIR"
    fi

    if [[ "$MODE" == "full" ]]; then
        add_root "/opt"
        add_root "/usr/local"
        add_root "/srv"
        add_root "/var/cache"
        add_root "/var/cache/pacman/pkg"
        add_root "/tmp"
        add_root "/var/tmp"
    fi

    for root in "${EXTRA_SCAN_ROOTS[@]}"; do
        add_root "$root"
    done

    if (( AUTO_MOUNT_ROOTS )) && have findmnt; then
        while IFS= read -r target; do
            [[ "$target" == /mnt/* || "$target" == /media/* || "$target" == /run/media/* ]] || continue
            [[ "$target" == /proc* || "$target" == /sys* || "$target" == /dev* || "$target" == /run* ]] && continue
            add_root "$target"
        done < <(findmnt -rn -o TARGET 2>/dev/null)
    fi
}

prepare_package_roots() {
    local root
    SCAN_ROOTS=()
    for root in /var/cache /var/lib/snapd/snaps /opt /usr/local/src /srv; do
        add_root "$root"
    done
    if (( INCLUDE_HOME )); then
        for root in "$XDG_CACHE_DIR" "$XDG_DATA_DIR" "$XDG_DOWNLOAD_DIR" "$HOME_DIR/Downloads" "$HOME_DIR/Descargas" "$HOME_DIR/AppImages" "$HOME_DIR/.local/src" /tmp /var/tmp; do
            add_root "$root"
        done
    fi
    # Custom --root paths remain useful for a mounted package archive or a
    # project-specific store, without recursively scanning every mount by default.
    for root in "${EXTRA_SCAN_ROOTS[@]}"; do
        add_root "$root"
    done
    for root in "${PACKAGE_SCAN_ROOTS[@]}"; do
        add_root "$root"
    done
}

write_scan_scope() {
    local root role reason mount
    printf 'root\trole\treason\treadable\n' >"$OUT_DIR/scan-scope.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        role="custom"; reason="Ruta solicitada o configurada por el usuario"
        case "$root" in
            /var/cache|/var/cache/*|/var/lib/snapd/snaps) role="system-store"; reason="Caché o revisiones de un gestor del sistema" ;;
            "$XDG_DOWNLOAD_DIR") role="downloads"; reason="Directorio de descargas configurado por XDG" ;;
            "$HOME_DIR"/.cache|"$HOME_DIR"/.cache/*) role="user-cache"; reason="Cachés y compilaciones del usuario" ;;
            "$HOME_DIR"/.local/share|"$HOME_DIR"/.local/share/*) role="user-data"; reason="Datos persistentes de aplicaciones y juegos" ;;
            "$HOME_DIR"/*/Downloads|"$HOME_DIR"/*/Descargas|"$HOME_DIR"/Downloads|"$HOME_DIR"/Descargas) role="downloads"; reason="Descargas habituales" ;;
            /mnt/*|/media/*|/run/media/*)
                role="mounted-disk"; reason="Disco o volumen montado"
                if have findmnt; then
                    mount="$(findmnt -rn -T "$root" -o TARGET 2>/dev/null | head -1)"
                    [[ "$mount" == "$root" ]] && reason="Punto de montaje detectado; se escanea sin cruzar a otros sistemas de archivos"
                fi
                ;;
            /home/*) role="user-path"; reason="Ruta dentro del directorio personal" ;;
            /opt|/opt/*|/usr/local|/usr/local/*) role="system-apps"; reason="Aplicaciones instaladas fuera de la base del gestor" ;;
            /srv|/srv/*) role="system-data"; reason="Datos de servicios del sistema" ;;
            /tmp|/tmp/*|/var/tmp|/var/tmp/*) role="temporary"; reason="Archivos temporales, incluidos posibles paquetes descargados" ;;
        esac
        if [[ -r "$root" ]]; then
            printf '%s\t%s\t%s\tsí\n' "$root" "$role" "$reason" >>"$OUT_DIR/scan-scope.tsv"
        else
            printf '%s\t%s\t%s\tno\n' "$root" "$role" "$reason" >>"$OUT_DIR/scan-scope.tsv"
        fi
    done
    {
        printf 'Ruta\tTipo\tMotivo\tLegible\n'
        tail -n +2 "$OUT_DIR/scan-scope.tsv"
    } >"$OUT_DIR/scan-scope.txt"
}

write_header() {
    local file="$1"
    {
        printf 'disk-audit %s\n' "$VERSION"
        printf 'Fecha: %s\n' "$(date --iso-8601=seconds 2>/dev/null || date)"
        printf 'Equipo: %s\n' "$HOSTNAME_SHORT"
        printf 'Usuario: %s\n' "$(id -un 2>/dev/null || printf unknown)"
        printf 'Home: %s\n' "$HOME_DIR"
        printf 'Modo: %s\n' "$MODE"
        printf 'Rutas escaneadas:\n'
        printf '  %s\n' "${SCAN_ROOTS[@]}"
    } >"$file"
}

collect_system() {
    {
        printf '## df -hT\n'
        df -hT 2>/dev/null || true
        printf '\n## lsblk\n'
        if have lsblk; then
            lsblk -e7 -o NAME,TYPE,FSTYPE,SIZE,FSAVAIL,FSUSE%,MOUNTPOINTS 2>/dev/null || true
        else
            printf 'lsblk no disponible\n'
        fi
        printf '\n## findmnt\n'
        if have findmnt; then
            findmnt -rn -o TARGET,SOURCE,FSTYPE,OPTIONS 2>/dev/null || true
        else
            printf 'findmnt no disponible\n'
        fi
        printf '\n## Memoria y kernel\n'
        uname -a 2>/dev/null || true
        if [[ -r /etc/os-release ]]; then
            cat /etc/os-release
        fi
        if have free; then free -h 2>/dev/null || true; fi
    } >"$OUT_DIR/system.txt"
}

collect_directory_usage() {
    local root file
    : >"$OUT_DIR/directory-usage.txt"
    for root in "${SCAN_ROOTS[@]}"; do
        file="$OUT_DIR/directory-usage-$(safe_name "$root").txt"
        {
            printf '# %s\n' "$root"
            if [[ -d "$root" ]]; then
                du -xhd1 -- "$root" 2>/dev/null | sort -h || true
            else
                printf 'Ruta no accesible o inexistente\n'
            fi
        } >"$file"
        printf '%s\n' "$file" >>"$OUT_DIR/directory-usage.txt"
    done

    {
        if (( INCLUDE_HOME )); then
            printf '# Rutas de usuario especialmente relevantes\n'
            for root in \
                "$HOME_DIR/.cache" \
                "$HOME_DIR/.cache/paru" \
                "$HOME_DIR/.local/share" \
                "$HOME_DIR/.config" \
                "$HOME_DIR/.var/app" \
                "$HOME_DIR/vmware" \
                "$HOME_DIR/Games" \
                "$HOME_DIR/.wine" \
                "$HOME_DIR/.local/share/Steam" \
                "$HOME_DIR/.local/share/lutris"; do
                [[ -e "$root" ]] || continue
                du -sh -- "$root" 2>/dev/null || true
            done
        fi
        printf '\n# Caché global de pacman\n'
        du -sh /var/cache/pacman/pkg 2>/dev/null || true
    } >"$OUT_DIR/key-paths.txt"
}

collect_packages() {
    : >"$OUT_DIR/packages-all.txt"
    : >"$OUT_DIR/packages-orphans.txt"
    : >"$OUT_DIR/packages-foreign.txt"
    : >"$OUT_DIR/packages-explicit.txt"
    : >"$OUT_DIR/packages-by-installed-size.txt"
    : >"$OUT_DIR/pacman-cache-dirs.txt"

    if have pacman; then
        pacman -Q 2>/dev/null | sort >"$OUT_DIR/packages-all.txt" || true
        pacman -Qdt 2>/dev/null | sort >"$OUT_DIR/packages-orphans.txt" || true
        pacman -Qm 2>/dev/null | sort >"$OUT_DIR/packages-foreign.txt" || true
        pacman -Qqe 2>/dev/null | sort >"$OUT_DIR/packages-explicit.txt" || true
    else
        printf 'pacman no disponible\n' >"$OUT_DIR/pacman-cache-dirs.txt"
    fi

    {
        printf '# Paquetes instalados: '; wc -l <"$OUT_DIR/packages-all.txt"
        printf '# Huérfanos: '; wc -l <"$OUT_DIR/packages-orphans.txt"
        printf '# Externos/AUR: '; wc -l <"$OUT_DIR/packages-foreign.txt"
        printf '# Explícitos: '; wc -l <"$OUT_DIR/packages-explicit.txt"
        printf '\n## Huérfanos\n'; cat "$OUT_DIR/packages-orphans.txt"
        printf '\n## Externos/AUR\n'; cat "$OUT_DIR/packages-foreign.txt"
    } >"$OUT_DIR/packages.txt"

    if have pacman && have pacman-conf; then
        pacman-conf CacheDir 2>/dev/null >"$OUT_DIR/pacman-cache-dirs.txt" || true
    fi

    # Installed sizes are parsed with C locale because pacman -Qi is localized otherwise.
    if have pacman; then pacman -Qi 2>/dev/null | awk '
        /^Name[[:space:]]*:/ { name=$0; sub(/^[^:]*:[[:space:]]*/, "", name) }
        /^Installed Size[[:space:]]*:/ { size=$0; sub(/^[^:]*:[[:space:]]*/, "", size); print size "\t" name }
    ' | sort -h >"$OUT_DIR/packages-by-installed-size.txt" || true; fi

    collect_package_inventory
}

collect_package_inventory() {
    local pacman_details foreign_list file
    pacman_details="$OUT_DIR/.pacman-details.tsv"
    foreign_list="$OUT_DIR/.pacman-foreign.txt"
    printf 'manager\tscope\torigin\treason\tname\tversion\tinstalled_size\n' >"$OUT_DIR/packages-inventory.tsv"

    if have pacman; then
        pacman -Qm 2>/dev/null | awk '{print $1}' | sort -u >"$foreign_list" || true
        pacman -Qi 2>/dev/null | awk '
            function emit() {
                if (name != "") print name "\t" version "\t" reason "\t" size
                name=version=reason=size=""
            }
            /^Name[[:space:]]*:/ { emit(); value=$0; sub(/^[^:]*:[[:space:]]*/, "", value); name=value }
            /^Version[[:space:]]*:/ { value=$0; sub(/^[^:]*:[[:space:]]*/, "", value); version=value }
            /^Install Reason[[:space:]]*:/ { value=$0; sub(/^[^:]*:[[:space:]]*/, "", value); reason=value }
            /^Installed Size[[:space:]]*:/ { value=$0; sub(/^[^:]*:[[:space:]]*/, "", value); size=value }
            END { emit() }
        ' >"$pacman_details" || true
        awk -F'\t' 'FILENAME==ARGV[1] {foreign[$1]=1; next} {
            origin=(foreign[$1] ? "foreign/AUR" : "official")
            reason=($3 ~ /^Explicit/ ? "explicit" : "dependency")
            print "pacman\tsystem\t" origin "\t" reason "\t" $1 "\t" $2 "\t" $4
        }' "$foreign_list" "$pacman_details" >>"$OUT_DIR/packages-inventory.tsv" || true
    fi

    if have dpkg-query; then
        dpkg-query -W -f='${Package}\t${Version}\t${Status}\n' 2>/dev/null \
            | awk -F'\t' '$3 == "install ok installed" {print "dpkg\tsystem\tdeb-installed\tunknown\t" $1 "\t" $2 "\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
    fi

    if have flatpak; then
        printf 'application\tname\torigin\tversion\tscope\n' >"$OUT_DIR/flatpak-user.tsv"
        printf 'application\tname\torigin\tversion\tscope\n' >"$OUT_DIR/flatpak-system.tsv"
        flatpak list --user --app --columns=application,name,origin,version 2>/dev/null \
            | if (( INCLUDE_HOME )); then awk -F'\t' 'NF {print $0 "\tuser"}'; else cat >/dev/null; fi >>"$OUT_DIR/flatpak-user.tsv" || true
        flatpak list --system --app --columns=application,name,origin,version 2>/dev/null \
            | awk -F'\t' 'NF {print $0 "\tsystem"}' >>"$OUT_DIR/flatpak-system.tsv" || true
        awk -F'\t' 'NR > 1 {print "flatpak\t" $5 "\t" $3 "\tuser\t" $1 "\t-\t-"}' "$OUT_DIR/flatpak-user.tsv" >>"$OUT_DIR/packages-inventory.tsv" || true
        awk -F'\t' 'NR > 1 {print "flatpak\t" $5 "\t" $3 "\tsystem\t" $1 "\t-\t-"}' "$OUT_DIR/flatpak-system.tsv" >>"$OUT_DIR/packages-inventory.tsv" || true
    fi

    # Query native databases only when their manager exists. This works on
    # Arch/CachyOS as well as Debian, Fedora, Alpine, Void, FreeBSD and
    # Homebrew systems without treating arbitrary files as installed packages.
    if have rpm; then
        rpm -qa --qf '%{NAME}\t%{VERSION}-%{RELEASE}\n' 2>/dev/null \
            | awk -F'\t' 'NF >= 2 {print "rpm\tsystem\tinstalled\tunknown\t" $1 "\t" $2 "\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
    fi
    if have apk; then
        apk info -v 2>/dev/null \
            | awk 'NF {name=$0; version="-"; if (match($0, /-[0-9][^-]*(-r[0-9]+)?$/)) {version=substr($0, RSTART+1); name=substr($0, 1, RSTART-1)} print "apk\tsystem\tinstalled\tunknown\t" name "\t" version "\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
    fi
    if have xbps-query; then
        xbps-query -l 2>/dev/null \
            | awk '$1 == "ii" {print "xbps\tsystem\tinstalled\tunknown\t" $2 "\t-\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
    fi
    if have pkg; then
        pkg info -a 2>/dev/null \
            | awk 'NF {print "pkg\tsystem\tinstalled\tunknown\t" $0 "\t-\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
    fi
    if have snap; then
        snap list 2>/dev/null \
            | awk 'NR > 1 && NF {print "snap\tsystem\tinstalled\tunknown\t" $1 "\t" $2 "\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
    fi
    if have brew && (( INCLUDE_HOME )); then
        brew list --formula --versions 2>/dev/null \
            | awk 'NF {print "brew\tuser\tformula\texplicit\t" $1 "\t" $2 "\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
        brew list --cask --versions 2>/dev/null \
            | awk 'NF {print "brew\tuser\tcask\texplicit\t" $1 "\t" $2 "\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
    fi
    if have nix-env && (( INCLUDE_HOME )); then
        nix-env -q 2>/dev/null \
            | awk 'NF {print "nix\tuser\tinstalled\texplicit\t" $0 "\t-\t-"}' \
            >>"$OUT_DIR/packages-inventory.tsv" || true
    fi

    awk -F'\t' 'NR == 1 || $2 == "system"' "$OUT_DIR/packages-inventory.tsv" >"$OUT_DIR/packages-system.tsv"
    awk -F'\t' 'NR == 1 || $2 == "user"' "$OUT_DIR/packages-inventory.tsv" >"$OUT_DIR/packages-user.tsv"

    {
        printf 'Categoría\tCantidad\n'
        awk -F'\t' 'NR > 1 {key=$1 ":" $2 ":" $3 ":" $4; count[key]++} END {for (key in count) print key "\t" count[key]}' \
            "$OUT_DIR/packages-inventory.tsv" | sort
    } >"$OUT_DIR/packages-by-scope.tsv"

    collect_package_managers
    collect_package_stores
    collect_package_artifacts
    write_package_summary
    rm -f -- "$pacman_details" "$foreign_list"
}

collect_package_artifacts() {
    local root bytes path kind scope lower_path
    printf 'kind\tscope\tbytes\thuman\tpath\n' >"$OUT_DIR/package-artifacts.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS=$'\t' read -r bytes path; do
            [[ "$bytes" =~ ^[0-9]+$ ]] || continue
            lower_path="${path,,}"
            [[ "$lower_path" == *.sig ]] && continue
            case "$lower_path" in
                *.deb|*.udeb) kind="deb" ;;
                *.rpm) kind="rpm" ;;
                *.pkg.tar.*) kind="pacman-package" ;;
                *.apk) kind="alpine-apk" ;;
                *.xbps) kind="void-xbps" ;;
                *.pisi) kind="pisi" ;;
                *.ipk) kind="ipk" ;;
                *.pkg) kind="pkg" ;;
                *.flatpak|*.flatpakref|*.flatpakrepo) kind="flatpak-file" ;;
                *.snap) kind="snap" ;;
                *.tgz|*.txz) kind="slackware-package" ;;
                *.tbz|*.tbz2) kind="gentoo-package" ;;
                *) continue ;;
            esac
            case "$path" in
                /var/cache/*|/var/lib/snapd/snaps/*) scope="system-cache" ;;
                "$HOME_DIR"/.cache/*) scope="user-cache" ;;
                "$HOME_DIR"/*) scope="user-file" ;;
                *) scope="external-file" ;;
            esac
            printf '%s\t%s\t%s\t%s\t%s\n' "$kind" "$scope" "$bytes" "$(human_bytes "$bytes")" "$path"
        done < <(find "$root" -xdev -type f \( \
            -iname '*.deb' -o -iname '*.udeb' -o -iname '*.rpm' -o -iname '*.pkg.tar.*' \
            -o -iname '*.apk' -o -iname '*.xbps' -o -iname '*.pisi' -o -iname '*.ipk' \
            -o -iname '*.pkg' -o -iname '*.flatpak' -o -iname '*.flatpakref' \
            -o -iname '*.flatpakrepo' -o -iname '*.snap' -o -iname '*.tgz' \
            -o -iname '*.txz' -o -iname '*.tbz' -o -iname '*.tbz2' \
        \) -printf '%s\t%p\n' 2>/dev/null)
    done | sort -n -k3,3 >>"$OUT_DIR/package-artifacts.tsv"
}

manager_path() {
    command -v "$1" 2>/dev/null || true
}

manager_version() {
    local manager="$1" output
    case "$manager" in
        dpkg) output="$(dpkg --version 2>/dev/null | head -1 || true)" ;;
        *) output="$($manager --version 2>/dev/null | head -1 || true)" ;;
    esac
    printf '%s' "$output" | tr '\t\n' '  '
}

collect_package_managers() {
    local manager family scope path version cache size
    printf 'manager\tfamily\tscope\texecutable\tversion\tknown_store\tstore_exists\tstore_size\n' >"$OUT_DIR/package-managers.tsv"
    for manager in pacman paru yay pikaur trizen aura pamac apt apt-get aptitude nala dpkg rpm dnf yum zypper apk xbps-query pkg snap flatpak brew nix-env emerge eopkg swupd; do
        path="$(manager_path "$manager")"
        [[ -n "$path" ]] || continue
        case "$manager" in
            pacman|pamac) family="arch"; scope="system" ;;
            paru|yay|pikaur|trizen|aura) family="arch"; scope="user/system" ;;
            apt|apt-get|aptitude|nala|dpkg) family="debian"; scope="system" ;;
            rpm|dnf|yum) family="rpm"; scope="system" ;;
            zypper) family="suse"; scope="system" ;;
            apk) family="alpine"; scope="system" ;;
            xbps-query) family="void"; scope="system" ;;
            pkg) family="freebsd"; scope="system" ;;
            snap|flatpak) family="$manager"; scope="system/user" ;;
            brew|nix-env) family="$manager"; scope="user" ;;
            *) family="other"; scope="system/user" ;;
        esac
        version="$(manager_version "$manager")"
        cache="-"
        case "$manager" in
            pacman) cache="/var/cache/pacman/pkg" ;;
            pamac) cache="/var/cache/pamac/pkg" ;;
            paru|yay|pikaur|trizen|aura) cache="$HOME_DIR/.cache/$manager" ;;
            apt|apt-get|aptitude|nala|dpkg) cache="/var/cache/apt/archives" ;;
            dnf) cache="/var/cache/dnf" ;;
            yum) cache="/var/cache/yum" ;;
            zypper) cache="/var/cache/zypp" ;;
            apk) cache="/var/cache/apk" ;;
            xbps-query) cache="/var/cache/xbps" ;;
            pkg) cache="/var/cache/pkg" ;;
            snap) cache="/var/lib/snapd/snaps" ;;
            flatpak) cache="/var/lib/flatpak" ;;
            brew) cache="$(brew --cache 2>/dev/null || printf '%s' '-')" ;;
            nix-env) cache="$HOME_DIR/.cache/nix" ;;
        esac
        if [[ "$cache" != "-" && -e "$cache" ]]; then
            size="$(du -sh -- "$cache" 2>/dev/null | awk '{print $1}')"
            printf '%s\t%s\t%s\t%s\t%s\t%s\tsí\t%s\n' "$manager" "$family" "$scope" "$path" "$version" "$cache" "$size" >>"$OUT_DIR/package-managers.tsv"
        else
            printf '%s\t%s\t%s\t%s\t%s\t%s\tno\t-\n' "$manager" "$family" "$scope" "$path" "$version" "$cache" >>"$OUT_DIR/package-managers.tsv"
        fi
    done
    {
        printf 'Gestor\tFamilia\tÁmbito\tEjecutable\tVersión\tAlmacén conocido\tExiste\tTamaño\n'
        tail -n +2 "$OUT_DIR/package-managers.tsv"
    } >"$OUT_DIR/package-managers.txt"
}

collect_package_stores() {
    local manager scope role path size
    printf 'manager\tscope\trole\tpath\texists\tsize\n' >"$OUT_DIR/package-stores.tsv"
    record_store() {
        manager="$1"; scope="$2"; role="$3"; path="$4"
        [[ -e "$path" ]] || return 0
        size="$(du -sh -- "$path" 2>/dev/null | awk '{print $1}')"
        printf '%s\t%s\t%s\t%s\tsí\t%s\n' "$manager" "$scope" "$role" "$path" "$size" >>"$OUT_DIR/package-stores.tsv"
    }
    if have pacman || [[ -e /var/lib/pacman ]]; then
        record_store pacman system cache /var/cache/pacman/pkg
        record_store pacman system database /var/lib/pacman
    fi
    if have pamac; then
        record_store pamac system cache /var/cache/pamac/pkg
        record_store pamac system database /var/lib/pamac
    fi
    for manager in paru yay pikaur trizen aura; do
        if (( INCLUDE_HOME )) && (have "$manager" || [[ -e "$HOME_DIR/.cache/$manager" ]]); then
            record_store "$manager" user cache "$HOME_DIR/.cache/$manager"
        fi
    done
    if have apt || have apt-get || have dpkg; then
        record_store apt system cache /var/cache/apt/archives
        record_store apt system database /var/lib/apt/lists
        record_store dpkg system database /var/lib/dpkg
    fi
    if have rpm || have dnf || have yum; then
        record_store rpm system cache /var/cache/dnf
        record_store rpm system cache /var/cache/yum
        record_store rpm system database /var/lib/rpm
    fi
    if have zypper; then
        record_store zypper system cache /var/cache/zypp
        record_store zypper system database /var/lib/rpm
    fi
    if have apk; then
        record_store apk system cache /var/cache/apk
        record_store apk system database /lib/apk/db
    fi
    if have xbps-query || have xbps-remove; then
        record_store xbps system cache /var/cache/xbps
        record_store xbps system database /var/db/xbps
    fi
    if have pkg; then
        record_store pkg system cache /var/cache/pkg
        record_store pkg system database /var/db/pkg
    fi
    if have snap; then
        record_store snap system package-revisions /var/lib/snapd/snaps
        record_store snap system database /var/lib/snapd
    fi
    if have flatpak; then
        record_store flatpak system store /var/lib/flatpak
        (( INCLUDE_HOME )) && record_store flatpak user store "$HOME_DIR/.local/share/flatpak"
    fi
    if have brew && (( INCLUDE_HOME )); then
        record_store brew user cache "$(brew --cache 2>/dev/null || printf '%s' "$HOME_DIR/.cache/Homebrew")"
    fi
    if have nix-env || have nix-store; then
        (( INCLUDE_HOME )) && record_store nix user cache "$HOME_DIR/.cache/nix"
        record_store nix system store /nix/store
    fi
    {
        printf 'Gestor\tÁmbito\tTipo\tRuta\tExiste\tTamaño\n'
        tail -n +2 "$OUT_DIR/package-stores.tsv"
    } >"$OUT_DIR/package-stores.txt"
}

write_package_summary() {
    local count_inventory count_artifacts count_managers count_stores kind count bytes
    count_inventory="$(awk 'NR > 1 {n++} END {print n + 0}' "$OUT_DIR/packages-inventory.tsv" 2>/dev/null)"
    count_artifacts="$(awk 'NR > 1 {n++} END {print n + 0}' "$OUT_DIR/package-artifacts.tsv" 2>/dev/null)"
    count_managers="$(awk 'NR > 1 {n++} END {print n + 0}' "$OUT_DIR/package-managers.tsv" 2>/dev/null)"
    count_stores="$(awk 'NR > 1 {n++} END {print n + 0}' "$OUT_DIR/package-stores.tsv" 2>/dev/null)"
    {
        printf 'INVENTARIO DE PAQUETES\n'
        printf '========================\n\n'
        printf 'Qué se ha comprobado:\n'
        printf '  - Ejecutables de gestores presentes en PATH.\n'
        printf '  - Bases de datos nativas de paquetes instalados.\n'
        printf '  - Cachés, almacenes y revisiones conocidas.\n'
        printf '  - Archivos de paquetes en las rutas indicadas en scan-scope.txt.\n\n'

        printf 'Resumen cuantitativo:\n'
        printf '  Gestores detectados: %s\n' "$count_managers"
        printf '  Almacenes existentes: %s\n' "$count_stores"
        printf '  Entradas instaladas: %s\n' "$count_inventory"
        printf '  Archivos descargados: %s\n\n' "$count_artifacts"

        printf 'Gestores y almacenes:\n'
        awk -F'\t' 'NR > 1 {printf "  - %s (%s): %s; almacén indicado: %s; existe: %s; tamaño: %s\n", $1, $2, $4, $6, $7, $8}' \
            "$OUT_DIR/package-managers.tsv"
        awk -F'\t' 'NR > 1 {printf "  - %s [%s/%s]: %s (%s)\n", $1, $2, $3, $4, $6}' \
            "$OUT_DIR/package-stores.tsv"
        printf '\nPaquetes instalados por gestor y ámbito:\n'
        awk -F'\t' 'NR > 1 {key=$1 "/" $2; count[key]++} END {for (key in count) print "  - " key ": " count[key]}' \
            "$OUT_DIR/packages-inventory.tsv" | sort

        printf '\nArchivos descargados por formato:\n'
        while IFS=$'\t' read -r kind count bytes; do
            printf '  - %s: %s archivos, %s\n' "$kind" "$count" "$(human_bytes "$bytes")"
        done < <(awk -F'\t' 'NR > 1 {count[$1]++; bytes[$1]+=$3} END {for (kind in count) print kind "\t" count[kind] "\t" bytes[kind]}' \
            "$OUT_DIR/package-artifacts.tsv" | sort)

        printf '\nInterpretación y seguridad:\n'
        printf '  - Un paquete explícito no es basura: solo se ofrece para revisión manual.\n'
        printf '  - La desinstalación debe hacerla el gestor que mantiene la base de datos.\n'
        printf '  - Un archivo .deb/.rpm/etc. es un instalador descargado, no prueba que siga instalado.\n'
        printf '  - Las cachés del sistema no deben borrarse como archivos normales; usa el limpiador nativo.\n'
        printf '  - Las rutas no existentes o no legibles quedan documentadas, pero no se consideran vacías.\n'
    } >"$OUT_DIR/package-summary.txt"
}

collect_flatpak() {
    if ! have flatpak; then
        printf 'flatpak no disponible\n' >"$OUT_DIR/flatpak.txt"
        return 0
    fi
    {
        printf '## Aplicaciones\n'
        flatpak list --app 2>/dev/null || true
        printf '\n## Todos los runtimes y extensiones\n'
        flatpak list --runtime 2>/dev/null || true
        printf '\n## Historial reciente\n'
        flatpak history 2>/dev/null | tail -50 || true
    } >"$OUT_DIR/flatpak.txt"
    {
        if (( INCLUDE_HOME )); then
            du -sh "$HOME_DIR/.local/share/flatpak" 2>/dev/null || true
            du -sh "$HOME_DIR/.var/app" 2>/dev/null || true
        fi
        du -sh /var/lib/flatpak 2>/dev/null || true
    } >"$OUT_DIR/flatpak-size.txt"
    [[ -f "$OUT_DIR/flatpak-user.tsv" ]] || printf 'flatpak no disponible\n' >"$OUT_DIR/flatpak-user.tsv"
    [[ -f "$OUT_DIR/flatpak-system.tsv" ]] || printf 'flatpak no disponible\n' >"$OUT_DIR/flatpak-system.tsv"
}

collect_desktop_apps() {
    local file name
    : >"$OUT_DIR/desktop-applications.tsv"
    printf 'path\tname\n' >"$OUT_DIR/desktop-applications.tsv"
    local desktop_roots=(/usr/share/applications)
    if (( INCLUDE_HOME )); then
        desktop_roots=("$HOME_DIR/.local/share/applications" "${desktop_roots[@]}")
    fi
    for root in "${desktop_roots[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' file; do
            name="$(awk -F= '/^Name=/{print substr($0,index($0,"=")+1); exit}' "$file" 2>/dev/null)"
            [[ -n "$name" ]] || name="$(basename "$file")"
            printf '%s\t%s\n' "$file" "$name" >>"$OUT_DIR/desktop-applications.tsv"
        done < <(find "$root" -maxdepth 1 -type f -name '*.desktop' -print0 2>/dev/null | sort -z)
    done

    {
        printf 'name\tcount\tpaths\n'
        awk -F'\t' '
            NR > 1 {
                key = tolower($2)
                count[key]++
                paths[key] = paths[key] "\t" $1
                names[key] = $2
            }
            END {
                for (key in count) if (count[key] > 1) print names[key] "\t" count[key] paths[key]
            }
        ' "$OUT_DIR/desktop-applications.tsv" | sort -f
    } >"$OUT_DIR/desktop-duplicate-names.tsv"
}

collect_appimages() {
    local root
    printf 'bytes\thuman\tpath\n' >"$OUT_DIR/appimages.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        find "$root" -xdev -type f \( -iname '*.appimage' -o -iname '*.AppImage' \) \
            -printf '%s\t%p\n' 2>/dev/null || true
    done | sort -n | while IFS=$'\t' read -r bytes path; do
        [[ "$bytes" =~ ^[0-9]+$ ]] || continue
        printf '%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$path"
    done >>"$OUT_DIR/appimages.tsv"
}

classify_prefix() {
    local prefix="$1"
    case "$prefix" in
        */files/share/default_pfx) printf 'runner-default' ;;
        */steamapps/compatdata/*/pfx|*/steamapps/compatdata/*/pfx/) printf 'steam-proton' ;;
        */pfx) printf 'proton-prefix' ;;
        */.wine|*/Games/*|*/bottles/*|*/lutris/*|*/Heroic/*|*/umu/*) printf 'user-game-prefix' ;;
        *) printf 'wine-prefix-unknown' ;;
    esac
}

collect_wine_prefixes() {
    local root prefix kind bytes
    printf 'bytes\thuman\ttype\tprefix\n' >"$OUT_DIR/wine-prefixes.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' prefix; do
            kind="$(classify_prefix "$prefix")"
            bytes="$(du -sx --block-size=1 -- "$prefix" 2>/dev/null | awk 'NR==1{print $1}')"
            [[ "$bytes" =~ ^[0-9]+$ ]] || bytes=0
            printf '%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$kind" "$prefix"
        done < <(find "$root" -xdev -type f -name system.reg -printf '%h\0' 2>/dev/null)
    done | sort -n >>"$OUT_DIR/wine-prefixes.tsv"
}

collect_games() {
    local path
    printf 'bytes\thuman\tpath\n' >"$OUT_DIR/game-related-paths.tsv"
    if (( INCLUDE_HOME )); then
        for path in \
            "$HOME_DIR/Games" \
            "$HOME_DIR/.local/share/Steam" \
            "$HOME_DIR/.local/share/lutris" \
            "$HOME_DIR/.config/lutris" \
            "$HOME_DIR/.config/heroic" \
            "$HOME_DIR/.var/app/com.heroicgameslauncher.hgl" \
            "$HOME_DIR/.local/share/bottles" \
            "$HOME_DIR/.var/app/com.usebottles.bottles"; do
            [[ -e "$path" ]] || continue
            local bytes
            bytes="$(du -sx --block-size=1 -- "$path" 2>/dev/null | awk 'NR==1{print $1}')"
            [[ "$bytes" =~ ^[0-9]+$ ]] || bytes=0
            printf '%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$path" >>"$OUT_DIR/game-related-paths.tsv"
        done
    fi

    {
        printf '# Steam library folders found\n'
        find "${SCAN_ROOTS[@]}" -xdev -type f -name libraryfolders.vdf -print 2>/dev/null | sort -u || true
        printf '\n# Common game directories\n'
        find "${SCAN_ROOTS[@]}" -xdev -type d \( -path '*/steamapps/common' -o -path '*/steamapps/compatdata' -o -path '*/lutris/runners' -o -path '*/Games/Heroic' \) -print 2>/dev/null | sort -u || true
    } >"$OUT_DIR/game-locations.txt"
}

collect_virtual_machines() {
    local root
    printf 'bytes\thuman\ttype\tpath\n' >"$OUT_DIR/virtual-machines.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        find "$root" -xdev -type f \( \
            -iname '*.vmx' -o -iname '*.vmdk' -o -iname '*.vdi' -o -iname '*.vhdx' \
            -o -iname '*.qcow2' -o -iname '*.ova' -o -iname '*.ovf' -o -iname '*.vhd' \
        \) -printf '%s\t%f\t%p\n' 2>/dev/null || true
    done | sort -n | while IFS=$'\t' read -r bytes name path; do
        [[ "$bytes" =~ ^[0-9]+$ ]] || continue
        printf '%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$name" "$path"
    done >>"$OUT_DIR/virtual-machines.tsv"
}

collect_installers_and_archives() {
    local root
    printf 'bytes\thuman\tpath\n' >"$OUT_DIR/installers-and-archives.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        find "$root" -xdev -type f \( \
            -iname '*.iso' -o -iname '*.img' -o -iname '*.raw' \
            -o -iname '*.zip' -o -iname '*.7z' -o -iname '*.rar' \
            -o -iname '*.tar' -o -iname '*.tar.gz' -o -iname '*.tar.xz' -o -iname '*.tar.zst' \
            -o -iname '*.deb' -o -iname '*.udeb' -o -iname '*.rpm' -o -iname '*.pkg.tar.*' \
            -o -iname '*.apk' -o -iname '*.xbps' -o -iname '*.pisi' -o -iname '*.ipk' \
            -o -iname '*.pkg' -o -iname '*.flatpak' -o -iname '*.flatpakref' \
            -o -iname '*.flatpakrepo' -o -iname '*.snap' -o -iname '*.tgz' \
            -o -iname '*.txz' -o -iname '*.tbz' -o -iname '*.tbz2' \
            -o -iname '*.exe' -o -iname '*.msi' -o -iname '*.AppImage' -o -iname '*.appimage' \
        \) -printf '%s\t%p\n' 2>/dev/null || true
    done | sort -n | while IFS=$'\t' read -r bytes path; do
        [[ "$bytes" =~ ^[0-9]+$ ]] || continue
        printf '%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$path"
    done >>"$OUT_DIR/installers-and-archives.tsv"
}

collect_build_artifacts() {
    local root path bytes
    printf 'bytes\thuman\ttype\tpath\n' >"$OUT_DIR/build-artifacts.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' path; do
            bytes="$(du -sx --block-size=1 -- "$path" 2>/dev/null | awk 'NR==1{print $1}')"
            [[ "$bytes" =~ ^[0-9]+$ ]] || bytes=0
            printf '%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$(basename "$path")" "$path"
        done < <(find "$root" -xdev -type d \( \
            -name target -o -name node_modules -o -name .gradle -o -name .cargo \
            -o -name .npm -o -name .yarn -o -name .venv -o -name __pycache__ \
            -o -name .tox -o -name build -o -name dist -o -name release \
        \) -prune -print0 2>/dev/null)
    done | sort -n >>"$OUT_DIR/build-artifacts.tsv"
}

collect_large_files() {
    local root
    printf 'bytes\thuman\tpath\n' >"$OUT_DIR/large-files.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        find "$root" -xdev -type f -size +500M -printf '%s\t%p\n' 2>/dev/null || true
    done | sort -n | tail -n "$TOP_N" | while IFS=$'\t' read -r bytes path; do
        [[ "$bytes" =~ ^[0-9]+$ ]] || continue
        printf '%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$path"
    done >>"$OUT_DIR/large-files.tsv"
}

collect_caches_and_trash() {
    local root
    printf 'bytes\thuman\tpath\n' >"$OUT_DIR/caches-and-trash.tsv"
    local cache_roots=(/var/cache/pacman/pkg)
    if (( INCLUDE_HOME )); then
        cache_roots=(
            "$HOME_DIR/.cache"
            "$HOME_DIR/.cache/paru"
            "$HOME_DIR/.cache/paru/clone"
            "$HOME_DIR/.cache/Shelly"
            "$HOME_DIR/.local/share/Trash"
            "$HOME_DIR/.cache/tauri"
            "$HOME_DIR/.cache/wine"
            "${cache_roots[@]}"
        )
    fi
    for root in "${cache_roots[@]}"; do
        [[ -e "$root" ]] || continue
        local bytes
        bytes="$(du -sx --block-size=1 -- "$root" 2>/dev/null | awk 'NR==1{print $1}')"
        [[ "$bytes" =~ ^[0-9]+$ ]] || bytes=0
        printf '%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$root" >>"$OUT_DIR/caches-and-trash.tsv"
    done
}

collect_duplicates() {
    local candidates hashed dup_sizes
    candidates="$OUT_DIR/.duplicate-candidates.tsv"
    hashed="$OUT_DIR/.duplicate-hashed.tsv"
    dup_sizes="$OUT_DIR/.duplicate-sizes.tsv"
    : >"$candidates"
    : >"$hashed"

    printf 'Buscando candidatos de al menos %s MiB...\n' "$DUPLICATE_MIN_MB"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        find "$root" -xdev -type f -size +"${DUPLICATE_MIN_MB}"M -printf '%s\t%p\n' 2>/dev/null || true
    done | sort -n >"$candidates"

    awk -F'\t' '{count[$1]++} END {for (size in count) if (count[size] > 1) print size}' \
        "$candidates" | sort -n >"$dup_sizes"

    while IFS=$'\t' read -r bytes path; do
        [[ "$bytes" =~ ^[0-9]+$ && -f "$path" ]] || continue
        local hash
        hash="$(sha256sum -- "$path" 2>/dev/null | awk '{print $1}')"
        [[ "$hash" =~ ^[[:xdigit:]]{64}$ ]] || continue
        printf '%s\t%s\t%s\n' "$hash" "$bytes" "$path"
    done < <(awk -F'\t' 'NR==FNR{wanted[$1]=1; next} wanted[$1]' "$dup_sizes" "$candidates" \
        | sort -n -k2,2) >"$hashed"

    {
        printf 'hash\tbytes\thuman\tpath\n'
        awk -F'\t' '
            { count[$1]++; rows[$1] = rows[$1] $0 ORS }
            END {
                for (hash in count) if (count[hash] > 1) {
                    n = split(rows[hash], lines, ORS)
                    for (i = 1; i <= n; i++) if (lines[i] != "") print lines[i]
                }
            }
        ' "$hashed" | while IFS=$'\t' read -r hash bytes path; do
            [[ "$bytes" =~ ^[0-9]+$ ]] || continue
            printf '%s\t%s\t%s\t%s\n' "$hash" "$bytes" "$(human_bytes "$bytes")" "$path"
        done | sort -k1,1
    } >"$OUT_DIR/duplicates.tsv"

    rm -f -- "$candidates" "$hashed" "$dup_sizes"
}

write_summary() {
    local count_orphans=0 count_foreign=0 count_prefixes=0 count_appimages=0 count_inventory=0 count_artifacts=0 count_managers=0 count_stores=0
    [[ -f "$OUT_DIR/packages-orphans.txt" ]] && count_orphans="$(wc -l <"$OUT_DIR/packages-orphans.txt")"
    [[ -f "$OUT_DIR/packages-foreign.txt" ]] && count_foreign="$(wc -l <"$OUT_DIR/packages-foreign.txt")"
    [[ -f "$OUT_DIR/wine-prefixes.tsv" ]] && count_prefixes=$(( $(wc -l <"$OUT_DIR/wine-prefixes.tsv") - 1 ))
    [[ -f "$OUT_DIR/appimages.tsv" ]] && count_appimages=$(( $(wc -l <"$OUT_DIR/appimages.tsv") - 1 ))
    [[ -f "$OUT_DIR/packages-inventory.tsv" ]] && count_inventory=$(( $(wc -l <"$OUT_DIR/packages-inventory.tsv") - 1 ))
    [[ -f "$OUT_DIR/package-artifacts.tsv" ]] && count_artifacts=$(( $(wc -l <"$OUT_DIR/package-artifacts.tsv") - 1 ))
    [[ -f "$OUT_DIR/package-managers.tsv" ]] && count_managers=$(( $(wc -l <"$OUT_DIR/package-managers.tsv") - 1 ))
    [[ -f "$OUT_DIR/package-stores.tsv" ]] && count_stores=$(( $(wc -l <"$OUT_DIR/package-stores.tsv") - 1 ))
    {
        write_header /dev/stdout
        printf '\nResumen:\n'
        printf '  Huérfanos pacman: %s\n' "$count_orphans"
        printf '  Paquetes externos/AUR: %s\n' "$count_foreign"
        printf '  Prefijos detectados: %s\n' "$count_prefixes"
        printf '  AppImages detectados: %s\n' "$count_appimages"
        printf '  Paquetes instalados detectados: %s\n' "$count_inventory"
        printf '  Archivos de paquetes detectados: %s\n' "$count_artifacts"
        printf '  Gestores de paquetes detectados: %s\n' "$count_managers"
        printf '  Almacenes/bases de datos detectados: %s\n' "$count_stores"
        printf '  Duplicados: %s\n' "$([[ "$DO_DUPLICATES" == 1 ]] && printf 'sí' || printf 'no')"
        printf '\nArchivos principales:\n'
        find "$OUT_DIR" -maxdepth 1 -type f -printf '  %f\n' 2>/dev/null | sort
    } | tee "$OUT_DIR/summary.txt"
}

main() {
    # Discover a command-line config path before loading configuration. The
    # full argument parse happens afterwards so CLI values override the file.
    local arg next_arg
    for ((arg = 1; arg <= $#; arg++)); do
        next_arg="${!arg:-}"
        if [[ "$next_arg" == "--config" ]]; then
            ((arg++))
            CONFIG_FILE="${!arg:-}"
        fi
    done
    load_config
    parse_args "$@"
    plan_init "$SCRIPT_NAME" || die "no se pudo crear el plan: ${PLAN_PATH:-desconocido}"
    [[ -n "$OUT_DIR" ]] || OUT_DIR="$PWD/disk-audit-$HOSTNAME_SHORT-$(date +%Y%m%d-%H%M%S)"
    mkdir -p -- "$OUT_DIR" || die "no se puede crear $OUT_DIR"
    if (( PACKAGES_ONLY )); then
        prepare_package_roots
    else
        prepare_roots
    fi
    printf 'Informe: %s\n' "$OUT_DIR"
    printf 'Rutas: %s\n' "${SCAN_ROOTS[*]}"

    write_header "$OUT_DIR/metadata.txt"
    write_scan_scope
    if (( PACKAGES_ONLY )); then
        collect_packages
        collect_flatpak
        write_summary
        printf '\nInventario de paquetes terminado. No se ha modificado ningún archivo.\n'
        printf 'Plan registrado en: %s\n' "$PLAN_PATH"
        exit 0
    fi
    collect_system
    collect_directory_usage
    collect_packages
    collect_flatpak
    collect_desktop_apps
    collect_appimages
    collect_wine_prefixes
    collect_games
    collect_virtual_machines
    collect_installers_and_archives
    collect_build_artifacts
    collect_large_files
    collect_caches_and_trash
    if (( DO_DUPLICATES )); then
        collect_duplicates
    else
        printf 'No ejecutado. Usa --duplicates para activar el escaneo por hash.\n' >"$OUT_DIR/duplicates.tsv"
    fi
    write_summary
    printf '\nAuditoría terminada. No se ha modificado ningún archivo.\n'
    plan_record audit "$OUT_DIR" executed yes "solo lectura" "$MODE"
    printf 'Plan registrado en: %s\n' "$PLAN_PATH"
}

main "$@"
