#!/usr/bin/env bash
# Allowlist estricta para temporales creados por el builder y sus pruebas.

ltools_temp_name_is_owned() {
    local name="$1"
    [[ "$name" =~ ^(cachyos-(e2e|smoke)|ltools-(native-help|tarball-e2e|publish-release|menu-e2e|software-git-e2e|windows-wine))\.[[:alnum:]]{6}$ ]] ||
        [[ "$name" =~ ^ltools-windows-gui\.[[:alnum:]]{6}\.log$ ]] ||
        [[ "$name" =~ ^ltools-(windows-disabled-alias-[0-9]+\.log|registry-dry-run-[0-9]+\.reg|alias-test-[0-9]+\.tsv)$ ]] ||
        [[ "$name" =~ ^ltools-(timestamped-reports|identity-failures|linux-cross-tools|windows-cross-tools|broad-install|host-catalog-forbidden|windows-cross-catalog|shell-backend|windows-fragile-runner|windows-array-match|wine-tests)\.txt$ ]] ||
        [[ "$name" =~ ^ltools-storage-map-[0-9]+-[0-9]+-[0-9]+$ ]] ||
        [[ "$name" =~ ^ltools-storage-map-(depth-zero|denied)-[0-9]+-[0-9]+$ ]] ||
        [[ "$name" =~ ^ltools-storage-copy-[[:alnum:]_-]+-[0-9]+-[0-9]+$ ]] ||
        [[ "$name" =~ ^ltools-plan-[[:alnum:]_-]+-[0-9]+-[0-9]+$ ]] ||
        [[ "$name" =~ ^ltools-cleaner-[[:alnum:]_-]+-[0-9]+-[0-9]+$ ]] ||
        [[ "$name" =~ ^ltools-(flatpak-installations|release-test|git-test|not-a-git-repo|git-recovery-dry-run)-[0-9]+-[0-9]+$ ]]
}

ltools_temp_embedded_pid() {
    local name="$1"
    if [[ "$name" =~ ^ltools-(storage-map|storage-map-depth-zero|storage-map-denied|storage-copy-[[:alnum:]_-]+|plan-[[:alnum:]_-]+|flatpak-installations|release-test|git-test|not-a-git-repo|git-recovery-dry-run)-([0-9]+)- ]] ||
        [[ "$name" =~ ^ltools-(windows-disabled-alias|registry-dry-run|alias-test)-([0-9]+)\.(log|reg|tsv)$ ]] ||
        [[ "$name" =~ ^ltools-(cleaner-[[:alnum:]_-]+)-([0-9]+)- ]]; then
        printf '%s' "${BASH_REMATCH[2]}"
    fi
}

# Comprueba una entrada directa de una raíz temporal explícita. Devuelve:
# 0 segura, 1 ruta/nombre no permitido, 2 pertenece a otro usuario, 3 en uso.
ltools_temp_candidate_check() {
    local path="$1" root="$2" parent resolved owner embedded_pid
    [[ "$root" == /tmp || "$root" == /var/tmp ]] || return 1
    [[ -d "$root" && ! -L "$root" ]] || return 1
    [[ "$(realpath -e -- "$root" 2>/dev/null)" == "$root" ]] || return 1
    parent="$(dirname -- "$path")"
    [[ "$parent" == "$root" ]] || return 1
    ltools_temp_name_is_owned "${path##*/}" || return 1
    [[ ! -L "$path" && ( -d "$path" || -f "$path" ) ]] || return 1
    resolved="$(realpath -e -- "$path" 2>/dev/null)" || return 1
    [[ "$resolved" == "$path" ]] || return 1
    owner="$(stat -c '%u' -- "$path" 2>/dev/null)" || return 1
    [[ "$owner" == "$EUID" ]] || return 2
    embedded_pid="$(ltools_temp_embedded_pid "${path##*/}")"
    if [[ -n "$embedded_pid" ]] && kill -0 "$embedded_pid" 2>/dev/null; then
        return 3
    fi
    if command -v fuser >/dev/null 2>&1 && fuser -s -- "$path" >/dev/null 2>&1; then
        return 3
    fi
    return 0
}
