#!/usr/bin/env bash

# Shared planning/rollback primitives for LTools modules.
# The file is sourced by modules and is intentionally dependency-light.

PLAN_PATH="${PLAN_PATH:-}"
DRY_RUN="${DRY_RUN:-0}"
PLAN_READY="${PLAN_READY:-0}"

plan_default_path() {
    local base="${XDG_STATE_HOME:-${HOME:-/tmp}/.local/state}/ltools/plans"
    mkdir -p -- "$base" 2>/dev/null || base="${TMPDIR:-/tmp}"
    printf '%s/plan-%s-%s.tsv' "$base" "$(date +%Y%m%d-%H%M%S)" "$$"
}

plan_init() {
    local module="$1"
    [[ "$PLAN_READY" == 1 ]] && return 0
    [[ -n "$PLAN_PATH" ]] || PLAN_PATH="$(plan_default_path)"
    if [[ "$PLAN_PATH" != /dev/stdout ]]; then
        mkdir -p -- "$(dirname -- "$PLAN_PATH")" || return 1
    fi
    {
        printf '# ltools-plan-v1\n'
        printf '# module=%s\n' "$module"
        printf '# created=%s\n' "$(date --iso-8601=seconds 2>/dev/null || date)"
        printf 'operation\ttarget\tstatus\treversible\tdata1\tdata2\n'
    } >"$PLAN_PATH" || return 1
    PLAN_READY=1
}

plan_clean_field() {
    printf '%s' "${1:-}" | tr '\t\r\n' '   '
}

plan_record() {
    local operation target status reversible data1 data2
    operation="$(plan_clean_field "${1:-}")"
    target="$(plan_clean_field "${2:-}")"
    status="$(plan_clean_field "${3:-executed}")"
    reversible="$(plan_clean_field "${4:-no}")"
    data1="$(plan_clean_field "${5:-}")"
    data2="$(plan_clean_field "${6:-}")"
    [[ "$PLAN_READY" == 1 ]] || plan_init "${SCRIPT_NAME:-ltools}"
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$operation" "$target" "$status" "$reversible" "$data1" "$data2" >>"$PLAN_PATH"
}

plan_announce() {
    local message="$1"
    printf '%s\n' "$message"
    [[ "$DRY_RUN" == 1 ]] && printf 'Simulación: no se aplicarán cambios.\n'
    [[ -n "$PLAN_PATH" ]] && printf 'Plan: %s\n' "$PLAN_PATH"
}

plan_trash_guess() {
    local path="$1" base candidate
    base="$(basename -- "$path")"
    for candidate in \
        "${XDG_DATA_HOME:-${HOME:-/tmp}/.local/share}/Trash/files/$base" \
        "$(dirname -- "$path")/.Trash-$(id -u 2>/dev/null || printf 1000)/files/$base"; do
        [[ -e "$candidate" ]] && { printf '%s' "$candidate"; return 0; }
    done
}

rollback_plan() {
    local file="$1" operation target status reversible data1 data2 answer restored=0 skipped=0
    [[ -f "$file" ]] || { printf 'No existe el plan: %s\n' "$file" >&2; return 2; }
    printf 'Plan de rollback: %s\n' "$file"
    printf 'Solo se revertirán operaciones marcadas como reversibles y ejecutadas.\n'
    printf '¿Continuar? [y/N] '
    read -r answer || return 1
    [[ "$answer" =~ ^([yY]|[sS])$ ]] || { printf 'Rollback cancelado.\n'; return 0; }
    while IFS=$'\t' read -r operation target status reversible data1 data2; do
        [[ "$operation" == operation || "$operation" == \#* || -z "$operation" ]] && continue
        [[ "$status" == executed && "$reversible" == yes ]] || { skipped=$((skipped + 1)); continue; }
        case "$operation" in
            restore-file)
                if [[ -e "$data1" ]]; then
                    # Keep the current version recoverable before restoring the
                    # known backup. This also handles edited JSON/rc files,
                    # where the target naturally still exists.
                    if [[ -e "$target" ]]; then
                        if command -v gio >/dev/null 2>&1; then
                            gio trash -- "$target" || { printf 'No se pudo apartar la versión actual: %s\n' "$target"; skipped=$((skipped + 1)); continue; }
                        else
                            printf 'No se restaura sobre un archivo existente sin gio: %s\n' "$target"
                            skipped=$((skipped + 1))
                            continue
                        fi
                    fi
                    mkdir -p -- "$(dirname -- "$target")" && cp -a -- "$data1" "$target" && { printf 'Restaurado: %s\n' "$target"; restored=$((restored + 1)); }
                else
                    printf 'No se puede restaurar automáticamente: %s\n' "$target"
                    skipped=$((skipped + 1))
                fi
                ;;
            trash-move)
                if [[ -e "$data1" && ! -e "$target" ]]; then
                    mkdir -p -- "$(dirname -- "$target")" && mv -- "$data1" "$target" && { printf 'Restaurado desde papelera: %s\n' "$target"; restored=$((restored + 1)); }
                else
                    printf 'No se encontró una copia recuperable en papelera para: %s\n' "$target"
                    skipped=$((skipped + 1))
                fi
                ;;
            remove-created)
                if [[ -e "$target" ]]; then
                    if command -v gio >/dev/null 2>&1; then
                        gio trash -- "$target" && { printf 'Destino retirado a papelera: %s\n' "$target"; restored=$((restored + 1)); }
                    else
                        printf 'No se retira sin gio: %s\n' "$target"
                        skipped=$((skipped + 1))
                    fi
                fi
                ;;
            *)
                printf 'Operación no soportada automáticamente: %s (%s)\n' "$operation" "$target"
                skipped=$((skipped + 1))
                ;;
        esac
    done < <(tail -n +5 "$file")
    printf 'Rollback terminado: %s restauradas, %s omitidas/no reversibles.\n' "$restored" "$skipped"
}
