#!/usr/bin/env bash

# game-wine-audit.sh - focused inventory of Wine/Proton and Linux game stores.
# Read-only: it never removes, moves, or changes files.

set -uo pipefail
export LC_ALL=C

VERSION="0.3.0"
SCRIPT_NAME="$(basename "$0")"
HOSTNAME_SHORT="$(hostname -s 2>/dev/null || hostname 2>/dev/null || printf 'unknown')"
HOME_DIR="${HOME:-$(getent passwd "$(id -u)" 2>/dev/null | cut -d: -f6)}"
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd -P)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib/ltools-plan.sh"

AUTO_MOUNT_ROOTS=1
INCLUDE_HOME=1
MODE="quick"
OUT_DIR=""
DRY_RUN=0
PLAN_PATH=""
SCAN_ROOTS=()
EXTRA_SCAN_ROOTS=()
CONFIG_FILE="${XDG_CONFIG_HOME:-$HOME_DIR/.config}/disk-audit.conf"

[[ "${LTOOLS_NO_MOUNTS:-0}" =~ ^(1|true|yes|si|sí)$ ]] && AUTO_MOUNT_ROOTS=0

usage() {
    cat <<EOF
Uso: $SCRIPT_NAME [opciones]

Auditoría especializada de juegos, Wine y Proton para CachyOS/Arch.
Solo lee datos y no borra nada.

Opciones:
  --full                    Incluye más rutas conocidas de runners y datos.
  --root RUTA               Añade una ruta al escaneo; se puede repetir.
  --no-home                 No añade el directorio personal ni sus datos conocidos.
  --no-mounts               No descubre montajes bajo /mnt, /media y /run/media.
  --out RUTA                Directorio del informe.
  --config RUTA             Carga otra configuración.
  --dry-run                 Auditoría de solo lectura explícita; no cambia datos.
  --plan FICHERO            Registra la auditoría en un plan sin operaciones de escritura.
  --version                 Muestra la versión.
  -h, --help                Muestra esta ayuda.

Ejemplos:
  $SCRIPT_NAME --full --out "$HOME/Informes/juegos-wine-$(date +%Y%m%d-%H%M)"
  $SCRIPT_NAME --root /mnt/JuegosLinux --root /mnt/JuegosWindows
  $SCRIPT_NAME --no-home --no-mounts --root /mnt/JuegosLinux
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
    local candidate="$1" root
    [[ -d "$candidate" ]] || return 0
    candidate="$(realpath -m -- "$candidate" 2>/dev/null || printf '%s' "$candidate")"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ "$root" == "$candidate" ]] && return 0
    done
    SCAN_ROOTS+=("$candidate")
}

load_config() {
    [[ -f "$CONFIG_FILE" ]] || return 0
    # Optional user-owned shell configuration.
    # shellcheck disable=SC1090
    source "$CONFIG_FILE"
}

parse_args() {
    while (($#)); do
        case "$1" in
            --full) MODE="full"; shift ;;
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
    for root in "${EXTRA_SCAN_ROOTS[@]}"; do
        add_root "$root"
    done
    if [[ "$MODE" == full ]]; then
        # Useful locations outside HOME on Arch/CachyOS; opt-in because they
        # can be slower to scan and may require elevated permissions.
        for root in /opt /usr/local/share /var/lib/flatpak /var/lib/steam /srv; do
            add_root "$root"
        done
    fi
    if (( AUTO_MOUNT_ROOTS )) && have findmnt; then
        while IFS= read -r target; do
            [[ "$target" == /mnt/* || "$target" == /media/* || "$target" == /run/media/* ]] || continue
            add_root "$target"
        done < <(findmnt -rn -o TARGET 2>/dev/null)
    fi
}

write_header() {
    local file="$1"
    {
        printf 'game-wine-audit %s\n' "$VERSION"
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
        fi
        printf '\n## Montajes de interés\n'
        if have findmnt; then
            findmnt -rn -o TARGET,SOURCE,FSTYPE,OPTIONS 2>/dev/null \
                | awk '$1 ~ /^\/home|^\/mnt|^\/media|^\/run\/media/ {print}' || true
        fi
    } >"$OUT_DIR/system.txt"
}

prefix_kind() {
    local prefix="$1"
    if [[ -f "$prefix/lutris.json" ]]; then
        printf 'lutris-prefix'
        return 0
    fi
    if [[ -f "$prefix/config_info" ]]; then
        local runner_path
        runner_path="$(sed -n '2p' "$prefix/config_info" 2>/dev/null)"
        case "$runner_path" in
            */lutris/*) printf 'lutris-prefix'; return 0 ;;
            */umu/*) printf 'umu-prefix'; return 0 ;;
            */heroic/*) printf 'heroic-prefix'; return 0 ;;
            */steam/*|*/Steam/*|*/steamapps/*|*/compatibilitytools.d/*) printf 'proton-prefix'; return 0 ;;
            */compatibilitytools/*) printf 'umu-prefix'; return 0 ;;
        esac
    fi
    case "$prefix" in
        */.Trash-*/*|*/Trash/*) printf 'trash-prefix' ;;
        */files/share/default_pfx) printf 'runner-default' ;;
        */steamapps/compatdata/*/pfx) printf 'steam-proton' ;;
        */.wine) printf 'default-wine' ;;
        */lutris/*|*/Lutris/*|*/Lutrs/*|*/.local/share/lutris/*|*/.config/lutris/*) printf 'lutris-prefix' ;;
        */Heroic/*|*/heroic/*|*/.config/heroic/*) printf 'heroic-prefix' ;;
        */bottles/*|*/.var/app/com.usebottles.bottles/*) printf 'bottles-prefix' ;;
        */umu/*|*/.local/share/umu/*) printf 'umu-prefix' ;;
        *) printf 'wine-prefix-unknown' ;;
    esac
}

prefix_appid() {
    local prefix="$1"
    if [[ "$prefix" =~ /steamapps/compatdata/([0-9]+)/pfx$ ]]; then
        printf '%s' "${BASH_REMATCH[1]}"
    fi
}

size_of() {
    local path="$1" bytes
    bytes="$(du -sx --block-size=1 -- "$path" 2>/dev/null | awk 'NR==1{print $1}')"
    [[ "$bytes" =~ ^[0-9]+$ ]] || bytes=0
    printf '%s' "$bytes"
}

collect_prefixes() {
    local root prefix kind appid root_bytes drive_bytes reported_bytes mountpoint scope tmp
    local marker markers
    declare -A seen_prefixes=()
    tmp="$OUT_DIR/.prefix-rows.tsv"
    : >"$tmp"
    printf 'prefix\tmountpoint\tdrive_c_size\troot_size\tmarkers\treason\n' >"$OUT_DIR/wine-mount-root-candidates.tsv"
    printf 'prefix\troot_size\tdrive_c_size\treason\n' >"$OUT_DIR/wine-trash-prefixes.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' prefix; do
            # Both system.reg and drive_c identify a normal prefix. Deduplicate
            # here so every canonical path gets one row only.
            prefix="$(realpath -e -- "$prefix" 2>/dev/null || printf '%s' "$prefix")"
            [[ -n "${seen_prefixes[$prefix]+present}" ]] && continue
            seen_prefixes["$prefix"]=1
            kind="$(prefix_kind "$prefix")"
            if [[ "$kind" == trash-prefix ]]; then
                printf '%s\t%s\t%s\ttrash-prefix\n' "$prefix" "$(human_bytes "$(size_of "$prefix")")" "$(human_bytes "$(size_of "$prefix/drive_c")")" \
                    >>"$OUT_DIR/wine-trash-prefixes.tsv"
                continue
            fi
            [[ -f "$prefix/system.reg" ]] || kind="candidate-drive_c"
            appid="$(prefix_appid "$prefix")"
            drive_c="$prefix/drive_c"
            root_bytes="$(size_of "$prefix")"
            drive_bytes=0
            [[ -d "$drive_c" ]] && drive_bytes="$(size_of "$drive_c")"
            reported_bytes="$root_bytes"
            scope="prefix-root"
            mountpoint=""
            if have findmnt; then
                mountpoint="$(findmnt -rn -T "$prefix" -o TARGET 2>/dev/null | head -1)"
            fi
            if [[ -n "$mountpoint" && "$mountpoint" == "$prefix" ]]; then
                # A Wine-looking directory at the root of a mounted disk is
                # reported separately. It must never enter generic migration.
                markers=""
                for marker in system.reg user.reg userdef.reg dosdevices drive_c; do
                    [[ -e "$prefix/$marker" || -L "$prefix/$marker" ]] || continue
                    [[ -n "$markers" ]] && markers+=';'
                    markers+="$marker"
                done
                printf '%s\t%s\t%s\t%s\t%s\tmount-root-wine-data\n' \
                    "$prefix" "$mountpoint" "$(human_bytes "$drive_bytes")" "$(human_bytes "$root_bytes")" "${markers:--}" \
                    >>"$OUT_DIR/wine-mount-root-candidates.tsv"
                continue
            fi
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
                "$reported_bytes" "$(human_bytes "$reported_bytes")" "$root_bytes" "$(human_bytes "$root_bytes")" \
                "$drive_bytes" "$(human_bytes "$drive_bytes")" "$scope" "$kind" "${appid:--}" "$prefix" "$drive_c" >>"$tmp"
        done < <(find "$root" -xdev \( -type f -name system.reg -printf '%h\0' -o -type d -name drive_c -printf '%h\0' \) 2>/dev/null)
    done
    {
        printf 'reported_bytes\treported_human\troot_bytes\troot_human\tdrive_c_bytes\tdrive_c_human\tsize_scope\ttype\tsteam_appid\tprefix\tdrive_c\n'
        sort -n -k1,1 "$tmp"
    } >"$OUT_DIR/wine-prefixes.tsv"
    rm -f -- "$tmp"

    printf 'prefix_candidate\tstatus\n' >"$OUT_DIR/wine-drive-c-candidates.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' drive_c; do
            prefix="$(dirname "$drive_c")"
            [[ -f "$prefix/system.reg" ]] && continue
            printf '%s\tno-system.reg\n' "$prefix" >>"$OUT_DIR/wine-drive-c-candidates.tsv"
        done < <(find "$root" -xdev -type d -name drive_c -print0 2>/dev/null)
    done

    {
        printf 'type\tcount\treported_bytes\thuman\n'
        awk -F'\t' 'NR>1 {count[$8]++; bytes[$8]+=$1} END {for (type in count) print type "\t" count[type] "\t" bytes[type]}' \
            "$OUT_DIR/wine-prefixes.tsv" | while IFS=$'\t' read -r type count bytes; do
                printf '%s\t%s\t%s\t%s\n' "$type" "$count" "$bytes" "$(human_bytes "$bytes")"
            done | sort
    } >"$OUT_DIR/wine-prefix-summary.tsv"

    {
        printf 'prefix_a\tprefix_b\n'
        awk -F'\t' 'NR>1 {paths[++n]=$10; types[n]=$8} END {for (i=1;i<=n;i++) for (j=1;j<=n;j++) if (i!=j && paths[i] != paths[j] && types[i] != "runner-default" && index(paths[j] "/", paths[i] "/") == 1) print paths[i] "\t" paths[j]}' \
            "$OUT_DIR/wine-prefixes.tsv" | sort -u
    } >"$OUT_DIR/wine-prefix-overlaps.tsv"
}

prefix_architecture() {
    local prefix="$1" value
    value="$(grep -m1 '^#arch=' "$prefix/system.reg" 2>/dev/null | sed 's/^#arch=//' || true)"
    [[ -n "$value" ]] && { printf '%s' "$value"; return 0; }
    [[ -d "$prefix/drive_c/Program Files (x86)" ]] && { printf 'probable-win64'; return 0; }
    [[ -d "$prefix/drive_c/windows/syswow64" ]] && { printf 'probable-win64'; return 0; }
    printf 'no-detectada'
}

prefix_windows_version() {
    local prefix="$1" value
    value="$(grep -m1 '"ProductName"=' "$prefix/system.reg" 2>/dev/null | sed 's/.*"ProductName"=//' | sed 's/^str://' | sed 's/^"//; s/".*$//' || true)"
    printf '%s' "${value:-no-detectada}"
}

prefix_runner() {
    local prefix="$1" config_runner runner_name
    if [[ "$prefix" =~ /steamapps/compatdata/([0-9]+)/pfx$ ]]; then
        printf 'Steam/Proton (AppID %s)' "${BASH_REMATCH[1]}"
        return 0
    fi
    if [[ "$prefix" =~ /compatibilitytools\.d/([^/]+)/files/share/default_pfx$ ]]; then
        printf 'Steam runner (%s)' "${BASH_REMATCH[1]}"
        return 0
    fi
    if [[ "$prefix" =~ /steamapps/common/([^/]+)/files/share/default_pfx$ ]]; then
        printf 'Steam/Proton runner (%s)' "${BASH_REMATCH[1]}"
        return 0
    fi
    if [[ "$prefix" =~ /tools/proton/([^/]+)/files/share/default_pfx$ ]]; then
        printf 'Heroic runner (%s)' "${BASH_REMATCH[1]}"
        return 0
    fi
    if [[ "$prefix" =~ /runners/wine/([^/]+)/files/share/default_pfx$ ]]; then
        printf 'Lutris runner (%s)' "${BASH_REMATCH[1]}"
        return 0
    fi
    if [[ "$prefix" =~ /compatibilitytools/([^/]+)/files/share/default_pfx$ ]]; then
        printf 'UMU runner (%s)' "${BASH_REMATCH[1]}"
        return 0
    fi
    if [[ -f "$prefix/config_info" ]]; then
        config_runner="$(sed -n '2p' "$prefix/config_info" 2>/dev/null)"
        runner_name="$(basename "$(dirname "$(dirname "$(dirname "$config_runner")")")")"
        case "$config_runner" in
            */lutris/*) printf 'Lutris/Wine (%s)' "$runner_name"; return 0 ;;
            */umu/*) printf 'UMU/Proton (%s)' "$runner_name"; return 0 ;;
            */heroic/*) printf 'Heroic/Proton (%s)' "$runner_name"; return 0 ;;
            */steam/*|*/Steam/*|*/steamapps/*|*/compatibilitytools.d/*) printf 'Steam/Proton (%s)' "$runner_name"; return 0 ;;
        esac
    fi
    case "$prefix" in
        */runners/wine/*) printf 'Lutris runner (%s)' "$(basename "$(dirname "$prefix")")" ;;
        */compatibilitytools/*) printf 'UMU runner (%s)' "$(basename "$(dirname "$prefix")")" ;;
        *) printf 'Wine del sistema/externo' ;;
    esac
}

prefix_program_dirs() {
    local prefix="$1" dir size result=""
    for dir in 'Program Files' 'Program Files (x86)' users windows; do
        [[ -d "$prefix/drive_c/$dir" ]] || continue
        size="$(size_of "$prefix/drive_c/$dir")"
        [[ -n "$result" ]] && result+='; '
        result+="$dir ($(human_bytes "$size"))"
    done
    printf '%s' "${result:--}"
}

registry_program_names() {
    local prefix="$1" file
    for file in "$prefix/system.reg" "$prefix/user.reg"; do
        [[ -f "$file" ]] || continue
        awk -F= '
            /^"DisplayName"=(str:)?/ {
                value=$0
                sub(/^[^=]*=/, "", value)
                sub(/^str:/, "", value)
                sub(/^"/, "", value)
                sub(/"[[:space:]]*$/, "", value)
                gsub(/\\"/, "\"", value)
                gsub(/[\t\r\n]/, " ", value)
                if (value != "") print value
            }
        ' "$file" 2>/dev/null | sort -fu
    done
}

collect_prefix_programs() {
    local reported_bytes reported_human root_bytes root_human drive_bytes drive_human size_scope type appid prefix drive_c
    local name file
    printf 'prefix\ttype\tdisplay_name\tregistry\n' >"$OUT_DIR/wine-prefix-programs.tsv"
    while IFS=$'\t' read -r reported_bytes reported_human root_bytes root_human drive_bytes drive_human size_scope type appid prefix drive_c; do
        [[ -n "$prefix" ]] || continue
        for file in "$prefix/system.reg" "$prefix/user.reg"; do
            [[ -f "$file" ]] || continue
            while IFS= read -r name; do
                [[ -n "$name" ]] || continue
                printf '%s\t%s\t%s\t%s\n' "$prefix" "$type" "$name" "$file" >>"$OUT_DIR/wine-prefix-programs.tsv"
            done < <(awk -F= '
                /^"DisplayName"=(str:)?/ {
                    value=$0
                    sub(/^[^=]*=/, "", value)
                    sub(/^str:/, "", value)
                    sub(/^"/, "", value)
                    sub(/"[[:space:]]*$/, "", value)
                    gsub(/\\"/, "\"", value)
                    gsub(/[\t\r\n]/, " ", value)
                    if (value != "") print value
                }
            ' "$file" 2>/dev/null | sort -fu)
        done
    done < <(tail -n +2 "$OUT_DIR/wine-prefixes.tsv")
}

collect_prefix_content() {
    local reported_bytes reported_human root_bytes root_human drive_bytes drive_human size_scope type appid prefix drive_c
    local item path bytes category name
    printf 'prefix\tcategory\tbytes\thuman\tpath\n' >"$OUT_DIR/wine-prefix-content.tsv"
    while IFS=$'\t' read -r reported_bytes reported_human root_bytes root_human drive_bytes drive_human size_scope type appid prefix drive_c; do
        [[ -n "$prefix" && -d "$drive_c" ]] || continue
        for item in 'Program Files' 'Program Files (x86)'; do
            path="$drive_c/$item"
            [[ -d "$path" ]] || continue
            while IFS= read -r -d '' name; do
                bytes="$(size_of "$name")"
                printf '%s\twindows-program\t%s\t%s\t%s\n' "$prefix" "$bytes" "$(human_bytes "$bytes")" "$name" >>"$OUT_DIR/wine-prefix-content.tsv"
            done < <(find "$path" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)
        done
        for item in users windows Games; do
            path="$drive_c/$item"
            [[ -d "$path" ]] || continue
            bytes="$(size_of "$path")"
            case "$item" in
                users) category="user-data" ;;
                windows) category="windows-system" ;;
                *) category="windows-games" ;;
            esac
            printf '%s\t%s\t%s\t%s\t%s\n' "$prefix" "$category" "$bytes" "$(human_bytes "$bytes")" "$path" >>"$OUT_DIR/wine-prefix-content.tsv"
        done
        # Non-standard directories beside drive_c are often where an
        # accidental centralization or an embedded second prefix is visible.
        while IFS= read -r -d '' path; do
            name="$(basename "$path")"
            case "$name" in
                drive_c|dosdevices|system.reg|system.reg.old|user.reg|userdef.reg|version|*.lock|*.lck) continue ;;
            esac
            bytes="$(size_of "$path")"
            printf '%s\tprefix-root-directory\t%s\t%s\t%s\n' "$prefix" "$bytes" "$(human_bytes "$bytes")" "$path" >>"$OUT_DIR/wine-prefix-content.tsv"
        done < <(find "$prefix" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)
    done < <(tail -n +2 "$OUT_DIR/wine-prefixes.tsv")
}

collect_prefix_binaries() {
    local reported_bytes reported_human root_bytes root_human drive_bytes drive_human size_scope type appid prefix drive_c
    local record bytes path kind
    printf 'prefix\tkind\tbytes\thuman\tname\tpath\n' >"$OUT_DIR/wine-prefix-binaries.tsv"
    while IFS=$'\t' read -r reported_bytes reported_human root_bytes root_human drive_bytes drive_human size_scope type appid prefix drive_c; do
        [[ -d "$drive_c" ]] || continue
        while IFS= read -r -d '' record; do
            bytes="${record%%$'\t'*}"
            path="${record#*$'\t'}"
            case "$path" in
                *.exe|*.EXE) kind="exe" ;;
                *) kind="msi" ;;
            esac
            printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$prefix" "$kind" "$bytes" "$(human_bytes "$bytes")" "$(basename "$path")" "$path" >>"$OUT_DIR/wine-prefix-binaries.tsv"
        done < <(find "$drive_c" -type f \( -iname '*.exe' -o -iname '*.msi' \) -printf '%s\t%p\0' 2>/dev/null)
    done < <(tail -n +2 "$OUT_DIR/wine-prefixes.tsv")
}

collect_prefix_details() {
    local reported_bytes reported_human root_bytes root_human drive_bytes drive_human size_scope type appid prefix drive_c
    local architecture windows runner lock_count exe_count msi_count components marker registered_programs
    printf 'prefix\ttype\tsteam_appid\treported_size\tdrive_c_size\tarchitecture\twindows_version\trunner\tlocks\texecutables\tinstallers\tregistered_programs\tcomponents\tmarkers\n' >"$OUT_DIR/wine-prefix-details.tsv"
    while IFS=$'\t' read -r reported_bytes reported_human root_bytes root_human drive_bytes drive_human size_scope type appid prefix drive_c; do
        [[ -n "$prefix" ]] || continue
        architecture="$(prefix_architecture "$prefix")"
        windows="$(prefix_windows_version "$prefix")"
        runner="$(prefix_runner "$prefix")"
        lock_count="$(find "$prefix" -maxdepth 4 -type f \( -name '*.lock' -o -name '*.lck' -o -name lock \) -print 2>/dev/null | wc -l)"
        exe_count="$(find "$drive_c" -type f -iname '*.exe' -print 2>/dev/null | wc -l)"
        msi_count="$(find "$drive_c" -type f -iname '*.msi' -print 2>/dev/null | wc -l)"
        registered_programs="$(awk -F'\t' -v wanted="$prefix" 'NR>1 && $1 == wanted {count++} END {print count+0}' "$OUT_DIR/wine-prefix-programs.tsv")"
        components="$(prefix_program_dirs "$prefix")"
        markers=""
        for marker in system.reg user.reg userdef.reg dosdevices drive_c; do
            [[ -e "$prefix/$marker" || -L "$prefix/$marker" ]] || continue
            [[ -n "$markers" ]] && markers+=';'
            markers+="$marker"
        done
        printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
            "$prefix" "$type" "${appid:--}" "$reported_human" "$drive_human" "$architecture" "$windows" "$runner" \
            "$lock_count" "$exe_count" "$msi_count" "$registered_programs" "$components" "${markers:--}" >>"$OUT_DIR/wine-prefix-details.tsv"
    done < <(tail -n +2 "$OUT_DIR/wine-prefixes.tsv")

    {
        printf 'Prefijo\tTipo\tAppID\tTamaño\tdrive_c\tArquitectura\tWindows\tRunner\tBloqueos\tEXE\tMSI\tProgramas registrados\tComponentes\tMarcadores\n'
        tail -n +2 "$OUT_DIR/wine-prefix-details.tsv"
    } >"$OUT_DIR/wine-prefix-details.txt"
}

steam_field() {
    local file="$1" key="$2"
    awk -F'"' -v wanted="$key" '$2 == wanted {print $4; exit}' "$file" 2>/dev/null
}

collect_steam() {
    local root manifest library appid name installdir disk_size game_dir bytes
    printf 'bytes\thuman\tappid\tname\tinstalldir\tlibrary\tmanifest\n' >"$OUT_DIR/steam-games.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' manifest; do
            library="$(dirname "$(dirname "$manifest")")"
            appid="$(steam_field "$manifest" appid)"
            name="$(steam_field "$manifest" name)"
            installdir="$(steam_field "$manifest" installdir)"
            disk_size="$(steam_field "$manifest" SizeOnDisk)"
            game_dir="$library/steamapps/common/$installdir"
            bytes="$(size_of "$game_dir")"
            [[ "$bytes" == 0 && "$disk_size" =~ ^[0-9]+$ ]] && bytes="$disk_size"
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
                "$bytes" "$(human_bytes "$bytes")" "${appid:--}" "${name:--}" \
                "${installdir:--}" "$library" "$manifest"
        done < <(find "$root" -xdev -type f -name 'appmanifest_*.acf' -print0 2>/dev/null)
    done | sort -n -k1,1 >>"$OUT_DIR/steam-games.tsv"

    printf 'bytes\thuman\tpath\n' >"$OUT_DIR/steam-unmanaged-directories.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' game_dir; do
            bytes="$(size_of "$game_dir")"
            printf '%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$game_dir"
        done < <(find "$root" -xdev -type d -path '*/steamapps/common/*' -prune -print0 2>/dev/null)
    done | sort -n >>"$OUT_DIR/steam-unmanaged-directories.tsv"

    {
        printf 'appid\tcount\tnames\tmanifests\n'
        awk -F'\t' 'NR>1 {count[$3]++; names[$3]=names[$3] " | " $4; paths[$3]=paths[$3] " | " $7} END {for (id in count) if (count[id]>1) print id "\t" count[id] "\t" names[id] "\t" paths[id]}' \
            "$OUT_DIR/steam-games.tsv" | sort -n
    } >"$OUT_DIR/steam-duplicate-appids.tsv"
}

yaml_value() {
    local file="$1" key="$2"
    awk -v wanted="$key" '$0 ~ "^[[:space:]]*" wanted ":[[:space:]]*" {value=$0; sub(/^[^:]*:[[:space:]]*/, "", value); sub(/^\047/, "", value); sub(/\047$/, "", value); sub(/^\042/, "", value); sub(/\042$/, "", value); print value; exit}' "$file" 2>/dev/null
}

yaml_wine_version() {
    local file="$1"
    awk '
        /^wine:[[:space:]]*$/ {in_wine=1; next}
        in_wine && /^[^[:space:]]/ {in_wine=0}
        in_wine && /^[[:space:]]+version:[[:space:]]*/ {
            value=$0
            sub(/^[^:]*:[[:space:]]*/, "", value)
            sub(/^"/, "", value); sub(/"$/, "", value)
            print value; exit
        }
    ' "$file" 2>/dev/null
}

collect_lutris() {
    local root file name slug runner prefix exe appid prefix_exists resolved_prefix bytes
    printf 'bytes\thuman\tname\tslug\tappid\trunner\tprefix\tprefix_status\texe\tconfig\tprefix_resolved\n' >"$OUT_DIR/lutris-games.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' file; do
            name="$(yaml_value "$file" name)"
            [[ -n "$name" ]] || name="$(basename "$file" .yml | sed -E 's/-[0-9]+$//' )"
            slug="$(yaml_value "$file" game_slug)"
            runner="$(yaml_value "$file" runner)"
            [[ -n "$runner" ]] || runner="$(yaml_wine_version "$file")"
            prefix="$(yaml_value "$file" prefix)"
            exe="$(yaml_value "$file" exe)"
            appid="$(yaml_value "$file" appid)"
            bytes=0
            prefix_exists="missing"
            resolved_prefix=""
            if [[ -d "$prefix" ]]; then
                bytes="$(size_of "$prefix")"
                prefix_exists="exists"
                resolved_prefix="$(realpath -e -- "$prefix" 2>/dev/null || printf '%s' "$prefix")"
                [[ "$resolved_prefix" != "$prefix" ]] && prefix_exists="alias"
            elif [[ -z "$prefix" ]]; then
                prefix_exists="not-declared"
            fi
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
                "$bytes" "$(human_bytes "$bytes")" "${name:--}" "${slug:--}" "${appid:--}" \
                "${runner:--}" "${prefix:--}" "$prefix_exists" "${exe:--}" "$file" "${resolved_prefix:--}"
        done < <(find "$root" -xdev -type f -path '*/lutris/games/*.yml' -print0 2>/dev/null)
    done | sort -n -k1,1 >>"$OUT_DIR/lutris-games.tsv"

    {
        printf 'slug\tcount\tnames\tprefixes\tconfigs\n'
        awk -F'\t' 'NR>1 && $4 != "-" {count[$4]++; names[$4]=names[$4] " | " $3; prefixes[$4]=prefixes[$4] " | " $7; configs[$4]=configs[$4] " | " $10} END {for (slug in count) if (count[slug]>1) print slug "\t" count[slug] "\t" names[slug] "\t" prefixes[slug] "\t" configs[slug]}' \
            "$OUT_DIR/lutris-games.tsv" | sort -f
    } >"$OUT_DIR/lutris-duplicate-slugs.tsv"
}

collect_heroic() {
    local root file name install_path prefix bytes prefix_scope
    printf 'bytes\thuman\tname\tinstall_path\twine_prefix\tsize_scope\tconfig\n' >"$OUT_DIR/heroic-configs.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' file; do
            name=""; install_path=""; prefix=""
            name="$(basename "$file" .json)"
            if have jq; then
                install_path="$(jq -r '.. | objects | .installPath? // .install_path? // empty' "$file" 2>/dev/null | head -1)"
                prefix="$(jq -r '.. | objects | .winePrefix? // .wine_prefix? // empty' "$file" 2>/dev/null | head -1)"
            fi
            bytes=0
            prefix_scope="none"
            if [[ -d "$prefix" ]]; then
                bytes="$(size_of "$prefix")"
                prefix_scope="wine-prefix"
                if have findmnt && [[ -d "$prefix/drive_c" ]] && [[ "$(findmnt -rn -T "$prefix" -o TARGET 2>/dev/null | head -1)" == "$prefix" ]]; then
                    bytes="$(size_of "$prefix/drive_c")"
                    prefix_scope="drive_c-on-mount"
                fi
            fi
            [[ "$bytes" == 0 && -d "$install_path" ]] && { bytes="$(size_of "$install_path")"; prefix_scope="install-path"; }
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
                "$bytes" "$(human_bytes "$bytes")" "${name:--}" "${install_path:--}" \
                "${prefix:--}" "$prefix_scope" "$file"
        done < <(find "$root" -xdev -type f -ipath '*/GamesConfig/*.json' -print0 2>/dev/null)
    done | sort -n -k1,1 >>"$OUT_DIR/heroic-configs.tsv"

    printf 'name\tmetadata\n' >"$OUT_DIR/heroic-library.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' file; do
            name="$(basename "$file" .json)"
            if have jq; then
                parsed="$(jq -r '.. | objects | .title? // .app_title? // empty' "$file" 2>/dev/null | head -1)"
                [[ -n "$parsed" ]] && name="$parsed"
            fi
            printf '%s\t%s\n' "$name" "$file"
        done < <(find "$root" -xdev -type f -ipath '*/legendary/metadata/*.json' -print0 2>/dev/null)
    done | sort -f >>"$OUT_DIR/heroic-library.tsv"

    printf 'bytes\thuman\tpath\n' >"$OUT_DIR/heroic-game-directories.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' install_path; do
            bytes="$(size_of "$install_path")"
            printf '%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$install_path"
        done < <(find "$root" -xdev -type d -path '*/Games/Heroic/*' -prune -print0 2>/dev/null)
    done | sort -n >>"$OUT_DIR/heroic-game-directories.tsv"
}

collect_bottles() {
    local root file bottle bytes
    printf 'bytes\thuman\tbottle\tconfig\n' >"$OUT_DIR/bottles.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' file; do
            bottle="$(dirname "$file")"
            bytes="$(size_of "$bottle")"
            printf '%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$bottle" "$file"
        done < <(find "$root" -xdev -type f \( -name bottle.yml -o -name bottle.conf \) -print0 2>/dev/null)
    done | sort -n -k1,1 >>"$OUT_DIR/bottles.tsv"
}

config_clean() {
    printf '%s' "${1:-}" | tr '\t\r\n' '   '
}

config_app_for_path() {
    case "$1" in
        */.config/heroic/*|*/.local/share/heroic/*|*/Games/Heroic/*|*/GamesConfig/*) printf 'Heroic' ;;
        */.config/lutris/*|*/.local/share/lutris/*|*/lutris/games/*) printf 'Lutris' ;;
        */.local/share/umu/*|*/.config/umu/*|*/compatibilitytools/*) printf 'UMU' ;;
        */.local/share/Steam/*|*/.steam/*|*/steamapps/*) printf 'Steam' ;;
        *) printf 'general' ;;
    esac
}

config_path_is_known() {
    case "$1" in
        */.config/heroic/*|*/.config/lutris/*|*/.config/umu/*|\
        */.local/share/heroic/*|*/.local/share/lutris/games/*|\
        */.local/share/umu/*/config_info|*/.local/share/umu/*/version|\
        */.local/share/umu/*/toolmanifest.vdf|\
        */.local/share/Steam/config/*|*/.local/share/Steam/steamapps/libraryfolders.vdf|\
        */.local/share/Steam/steamapps/appmanifest_*.acf|*/.steam/steam/steamapps/*|\
        */Games/Heroic/GamesConfig/*|*/.var/app/*/config/*|*/.var/app/*/data/*/config/*)
            return 0 ;;
    esac
    return 1
}

collect_configuration_files() {
    local root file bytes format app scope magic candidate
    local -a config_roots=()
    printf 'bytes\thuman\tformat\tapp\tscope\ttype\tpath\n' >"$OUT_DIR/configuration-files.tsv"
    printf 'bytes\thuman\tformat\tapp\tscope\tsqlite_header\ttype\tpath\n' >"$OUT_DIR/configuration-databases.tsv"
    printf 'bytes\thuman\tformat\tapp\ttype\tfile_type\tpath\n' >"$OUT_DIR/configuration-binaries.tsv"
    # Configuration is deliberately scanned from configuration roots, not
    # from every game file on a mounted disk. This keeps --full responsive on
    # Steam libraries containing millions of assets.
    for candidate in \
        "$HOME_DIR/.config/heroic" "$HOME_DIR/.config/lutris" "$HOME_DIR/.config/umu" \
        "$HOME_DIR/.local/share/lutris/games" \
        "$HOME_DIR/.local/share/Steam/config" "$HOME_DIR/.local/share/Steam/steamapps" \
        "$HOME_DIR/.steam/steam/steamapps" "$HOME_DIR/Games/Heroic/GamesConfig"; do
        [[ -d "$candidate" ]] || continue
        config_roots+=("$candidate")
    done
    for root in "${config_roots[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' file; do
            config_path_is_known "$file" || continue
            case "$file" in
                *.sqlite|*.sqlite3|*.db|*.db-wal|*.db-shm) format="sqlite/db" ;;
                *.vdf) format="vdf" ;;
                *.acf) format="steam-acf" ;;
                *.json) format="json" ;;
                *.yml|*.yaml) format="yaml" ;;
                *.toml) format="toml" ;;
                *.ini|*.conf) format="ini/conf" ;;
                *.bin|*.dat) format="binary/dat" ;;
                *) continue ;;
            esac
            bytes="$(stat -c '%s' -- "$file" 2>/dev/null || printf 0)"
            scope="user"
            [[ "$file" == /var/* || "$file" == /opt/* ]] && scope="system"
            app="$(config_app_for_path "$file")"
            if have file; then
                format="$(file -b -- "$file" 2>/dev/null | tr '\t\r\n' ' ' | cut -c1-120)"
            fi
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$format" "$app" "$scope" "$(file -b -- "$file" 2>/dev/null | tr '\t\r\n' ' ' | cut -c1-100)" "$file" >>"$OUT_DIR/configuration-files.tsv"
            case "$file" in
                *.sqlite|*.sqlite3|*.db|*.db-wal|*.db-shm)
                    magic="no"
                    if dd if="$file" bs=1 count=16 2>/dev/null | grep -a -q '^SQLite format 3'; then
                        magic="yes"
                    fi
                    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$format" "$app" "$scope" "$magic" "$(file -b -- "$file" 2>/dev/null | tr '\t\r\n' ' ' | cut -c1-100)" "$file" >>"$OUT_DIR/configuration-databases.tsv"
                    ;;
            esac
            case "$file" in
                *.bin|*.dat|*.vdf|*.acf|*.db|*.sqlite|*.sqlite3)
                    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$format" "$app" "$format" "$(file -b -- "$file" 2>/dev/null | tr '\t\r\n' ' ' | cut -c1-120)" "$file" >>"$OUT_DIR/configuration-binaries.tsv"
                    ;;
            esac
        done < <(find "$root" -xdev -type f \( -iname '*.sqlite' -o -iname '*.sqlite3' -o -iname '*.db' -o -iname '*.db-wal' -o -iname '*.db-shm' -o -iname '*.vdf' -o -iname '*.acf' -o -iname '*.json' -o -iname '*.yml' -o -iname '*.yaml' -o -iname '*.toml' -o -iname '*.ini' -o -iname '*.conf' -o -iname '*.bin' -o -iname '*.dat' \) -print0 2>/dev/null)
    done
    for file in configuration-files.tsv configuration-databases.tsv configuration-binaries.tsv; do
        { head -n 1 "$OUT_DIR/$file"; tail -n +2 "$OUT_DIR/$file" | sort -nr -k1,1; } >"$OUT_DIR/$file.tmp"
        mv -- "$OUT_DIR/$file.tmp" "$OUT_DIR/$file"
    done
}

validation_row() {
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$(config_clean "$1")" "$(config_clean "$2")" "$(config_clean "$3")" \
        "$(config_clean "$4")" "$(config_clean "$5")" "$(config_clean "$6")" >>"$OUT_DIR/configuration-validation.tsv"
}

validate_json_config() {
    local app file key value status
    app="$1"; file="$2"
    [[ -f "$file" ]] || return 0
    if have jq; then
        if jq empty "$file" >/dev/null 2>&1; then
            validation_row "$app" "$file" valid syntax - 'JSON válido'
            while IFS=$'\t' read -r key value; do
                [[ -n "$key" && -n "$value" ]] || continue
                status="literal"
                if [[ "$value" == /* ]]; then
                    [[ -e "$value" ]] && status="path-exists" || status="path-missing"
                fi
                validation_row "$app" "$file" "$status" "$key" "$value" 'campo detectado'
            done < <(jq -r '.. | objects | to_entries[]? | select(.key|test("(?i)(wineprefix|wine_prefix|defaultwineprefix|defaultwineprefixdir|installpath|defaultinstallpath|defaultsteampath|runner|winepath|gamepath|^bin$)")) | [.key, (.value|tostring)] | @tsv' "$file" 2>/dev/null)
        else
            validation_row "$app" "$file" invalid syntax - 'JSON no válido o incompleto'
        fi
    elif have python3; then
        if CONFIG_JSON_FILE="$file" python3 -c 'import json,os; json.load(open(os.environ["CONFIG_JSON_FILE"], encoding="utf-8"))' >/dev/null 2>&1; then
            validation_row "$app" "$file" valid syntax - 'JSON válido; instala jq para revisar campos internos'
        else
            validation_row "$app" "$file" invalid syntax - 'JSON no válido o incompleto'
        fi
    else
        validation_row "$app" "$file" unknown syntax - 'no hay jq ni python3 para validar JSON'
    fi
}

collect_configuration_validation() {
    local file root prefix runner game_path library count status
    local resolved_file
    declare -A seen_steam_configs=()
    printf 'app\tconfig\tstatus\tfield\tvalue\tnote\n' >"$OUT_DIR/configuration-validation.tsv"
    for file in \
        "$HOME_DIR/.config/heroic/config.json" \
        "$HOME_DIR/.config/heroic/store/config.json"; do
        validate_json_config Heroic "$file"
    done
    local -a heroic_roots=("$HOME_DIR/Games/Heroic/GamesConfig" "$HOME_DIR/.config/heroic")
    for root in "${heroic_roots[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' file; do
            validate_json_config Heroic "$file"
        done < <(find "$root" -xdev -type f -path '*/GamesConfig/*.json' -print0 2>/dev/null)
    done

    for file in "$HOME_DIR/.local/share/lutris/system.yml" "$HOME_DIR/.config/lutris/system.yml"; do
        [[ -f "$file" ]] || continue
        game_path="$(yaml_value "$file" game_path)"
        [[ -n "$game_path" ]] && [[ -d "$game_path" ]] && status="path-exists" || status="path-missing"
        validation_row Lutris "$file" "$status" game_path "${game_path:--}" 'directorio global de juegos'
    done
    while IFS=$'\t' read -r _ _ name slug appid runner prefix prefix_status exe config resolved; do
        [[ -n "$config" ]] || continue
        validation_row Lutris "$config" "$prefix_status" prefix "${prefix:--}" "runner=${runner:--}; exe=${exe:--}"
    done < <(tail -n +2 "$OUT_DIR/lutris-games.tsv")

    for root in "$HOME_DIR/.local/share/umu" "$HOME_DIR/.config/umu"; do
        [[ -d "$root" ]] || continue
        validation_row UMU "$root" exists root "$root" 'directorio de configuración/herramientas'
        while IFS= read -r -d '' file; do
            runner="$(sed -n '2p' "$file" 2>/dev/null)"
            [[ -n "$runner" ]] && [[ -e "$runner" ]] && status="runner-exists" || status="runner-missing"
            validation_row UMU "$file" "$status" runner "${runner:--}" 'config_info del prefijo'
        done < <(find "$root" -xdev -type f -name config_info -print0 2>/dev/null)
    done
    for file in \
        "$HOME_DIR/.local/share/Steam/steamapps/libraryfolders.vdf" \
        "$HOME_DIR/.steam/steam/steamapps/libraryfolders.vdf" \
        "$HOME_DIR/.local/share/Steam/config/libraryfolders.vdf"; do
        [[ -f "$file" ]] || continue
        resolved_file="$(realpath -e -- "$file" 2>/dev/null || printf '%s' "$file")"
        [[ -n "${seen_steam_configs[$resolved_file]+present}" ]] && continue
        seen_steam_configs["$resolved_file"]=1
        validation_row Steam "$file" valid libraryfolders path "$(steam_field "$file" path)" 'biblioteca declarada'
        while IFS= read -r library; do
            [[ -n "$library" ]] || continue
            [[ -d "$library/steamapps" ]] && status="library-exists" || status="library-missing"
            count="$(find "$library/steamapps" -maxdepth 1 -type f -name 'appmanifest_*.acf' 2>/dev/null | wc -l)"
            validation_row Steam "$file" "$status" library "$library" "manifiestos=$count"
        done < <(awk -F'"' '/"path"[[:space:]]*"/ {print $4}' "$file" 2>/dev/null | sed 's#\\\\#\\#g')
    done
}

runner_kind() {
    case "$1" in
        */steamapps/common/*) printf 'steam-proton-tool' ;;
        */compatibilitytools.d/*) printf 'steam-compatibility-tool' ;;
        */runners/wine/*) printf 'lutris-wine-runner' ;;
        */tools/proton/*) printf 'heroic-proton-tool' ;;
        */compatibilitytools/*) printf 'umu-compatibility-tool' ;;
        *) printf 'runner' ;;
    esac
}

collect_runners() {
    local root parent runner bytes
    printf 'bytes\thuman\ttype\trunner\tparent\n' >"$OUT_DIR/runners.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' parent; do
            while IFS= read -r -d '' runner; do
                if [[ "$parent" == */steamapps/common ]]; then
                    # That directory also contains ordinary games. Only keep
                    # entries with Proton/runtime markers, not game folders.
                    [[ -f "$runner/toolmanifest.vdf" || -x "$runner/proton" || \
                        "$(basename "$runner")" == Proton* || "$(basename "$runner")" == SteamLinuxRuntime* ]] || continue
                fi
                bytes="$(size_of "$runner")"
                printf '%s\t%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" \
                    "$(runner_kind "$runner")" "$runner" "$parent"
            done < <(find "$parent" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)
        done < <(find "$root" -xdev -type d \( -name compatibilitytools.d -o -path '*/runners/wine' -o -path '*/tools/proton' -o -path '*/compatibilitytools' -o -path '*/steamapps/common' \) -print0 2>/dev/null)
    done | sort -n -k1,1 >>"$OUT_DIR/runners.tsv"
}

collect_game_roots() {
    local root path bytes
    printf 'bytes\thuman\ttype\tpath\n' >"$OUT_DIR/game-roots.tsv"
    for root in "${SCAN_ROOTS[@]}"; do
        [[ -d "$root" ]] || continue
        while IFS= read -r -d '' path; do
            bytes="$(size_of "$path")"
            case "$path" in
                */steamapps) kind="steam" ;;
                */Lutris|*/lutris) kind="lutris" ;;
                */Heroic|*/heroic) kind="heroic" ;;
                */Games|*/games) kind="games" ;;
                *) kind="game-related" ;;
            esac
            printf '%s\t%s\t%s\t%s\n' "$bytes" "$(human_bytes "$bytes")" "$kind" "$path"
        done < <(find "$root" -xdev -type d \( -name steamapps -o -name Lutris -o -name lutris -o -name Heroic -o -name heroic -o -name Games -o -name games \) -prune -print0 2>/dev/null)
    done | sort -n -k1,1 >>"$OUT_DIR/game-roots.tsv"
}

write_summary() {
    local prefixes mount_candidates steam lutris runners bottles appimages validation_issues config_dbs
    prefixes=$(( $(wc -l <"$OUT_DIR/wine-prefixes.tsv") - 1 ))
    mount_candidates=$(( $(wc -l <"$OUT_DIR/wine-mount-root-candidates.tsv") - 1 ))
    steam=$(( $(wc -l <"$OUT_DIR/steam-games.tsv") - 1 ))
    lutris=$(( $(wc -l <"$OUT_DIR/lutris-games.tsv") - 1 ))
    runners=$(( $(wc -l <"$OUT_DIR/runners.tsv") - 1 ))
    bottles=$(( $(wc -l <"$OUT_DIR/bottles.tsv") - 1 ))
    validation_issues="$(awk -F'\t' 'NR>1 && ($3 ~ /missing|invalid|unknown/) {count++} END {print count+0}' "$OUT_DIR/configuration-validation.tsv" 2>/dev/null)"
    config_dbs=$(( $(wc -l <"$OUT_DIR/configuration-databases.tsv") - 1 ))
    appimages=0
    {
        write_header /dev/stdout
        printf '\nResumen especializado:\n'
        printf '  Prefijos Wine/Proton catalogados: %s\n' "$prefixes"
        printf '  Raíces de montaje con restos Wine (revisión): %s\n' "$mount_candidates"
        printf '  Juegos Steam con manifiesto: %s\n' "$steam"
        printf '  Juegos Lutris con configuración: %s\n' "$lutris"
        printf '  Runners detectados: %s\n' "$runners"
        printf '  Botellas Bottles: %s\n' "$bottles"
        printf '  Configuraciones SQLite/DB: %s\n' "$config_dbs"
        printf '  Incidencias de configuración detectadas: %s\n' "$validation_issues"
        printf '\nInformes clave:\n'
        printf '  wine-prefixes.tsv: prefijos ordenados por tamaño y clasificados\n'
        printf '  wine-prefix-details.tsv: arquitectura, registro, runner, bloqueos y contenido\n'
        printf '  wine-prefix-programs.tsv: programas registrados en los desinstaladores de Wine\n'
        printf '  wine-prefix-content.tsv: carpetas de programas, usuarios, sistema y datos anidados\n'
        printf '  wine-prefix-binaries.tsv: ejecutables EXE/MSI encontrados y sus rutas\n'
        printf '  configuration-validation.tsv: validación de Heroic, Lutris, UMU y Steam\n'
        printf '  configuration-databases.tsv: SQLite/DB y cabeceras detectadas\n'
        printf '  configuration-binaries.tsv: VDF/ACF/DB/binarios de configuración\n'
        printf '  wine-drive-c-candidates.tsv: carpetas drive_c sin system.reg, marcadas como candidatas\n'
        printf '  wine-mount-root-candidates.tsv: raíces de disco con restos de un prefijo Wine\n'
        printf '  wine-prefix-summary.tsv: totales por tipo\n'
        printf '  steam-games.tsv: appid, nombre, biblioteca y tamaño\n'
        printf '  lutris-games.tsv: nombre, runner, prefijo y ejecutable\n'
        printf '  lutris-duplicate-slugs.tsv: configuraciones Lutris repetidas\n'
        printf '  heroic-configs.tsv y heroic-library.tsv: configuraciones y biblioteca Heroic\n'
        printf '  runners.tsv: Proton/Wine compartidos\n'
        printf '  game-roots.tsv: directorios principales de juegos\n'
        printf '  wine-prefix-overlaps.tsv: prefijos anidados que requieren revisión\n'
        printf '  steam-duplicate-appids.tsv: el mismo AppID en varias bibliotecas\n'
    } | tee "$OUT_DIR/summary.txt"
}

main() {
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
    [[ -n "$OUT_DIR" ]] || OUT_DIR="$PWD/game-wine-audit-$HOSTNAME_SHORT-$(date +%Y%m%d-%H%M%S)"
    mkdir -p -- "$OUT_DIR" || die "no se puede crear $OUT_DIR"
    prepare_roots
    printf 'Informe: %s\n' "$OUT_DIR"
    printf 'Rutas: %s\n' "${SCAN_ROOTS[*]}"

    write_header "$OUT_DIR/metadata.txt"
    collect_system
    collect_prefixes
    collect_prefix_programs
    collect_prefix_content
    collect_prefix_binaries
    collect_prefix_details
    collect_steam
    collect_lutris
    collect_heroic
    collect_bottles
    collect_configuration_files
    collect_configuration_validation
    collect_runners
    collect_game_roots
    write_summary
    printf '\nAuditoría terminada. No se ha modificado ningún archivo.\n'
    plan_record audit "$OUT_DIR" executed yes "solo lectura" "$MODE"
    printf 'Plan registrado en: %s\n' "$PLAN_PATH"
}

main "$@"
