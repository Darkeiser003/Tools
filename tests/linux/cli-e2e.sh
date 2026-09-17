#!/usr/bin/env bash
# E2E de la interfaz CLI distribuible. Solo usa un HOME y un repositorio
# sintéticos; las acciones mutables se ejecutan únicamente con --dry-run.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
CLI_BIN=""
KEEP_TEMP=0

die() { printf 'CLI E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        --cli-binary) (($# >= 2)) || die '--cli-binary necesita una ruta'; CLI_BIN="$2"; shift ;;
        --keep-temp) KEEP_TEMP=1 ;;
        -h|--help)
            printf 'Uso: %s [--binary RUTA] [--cli-binary RUTA] [--keep-temp]\n' "$0"
            exit 0
            ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ -x "$BIN" ]] || die "no existe el backend CLI/GUI: $BIN"
if [[ -n "$CLI_BIN" ]]; then
    [[ -x "$CLI_BIN" ]] || die "no existe el ejecutable CLI distribuible: $CLI_BIN"
fi
command -v jq >/dev/null 2>&1 || die 'jq es necesario para validar los contratos JSON'

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ltools-cli-e2e.XXXXXX")"
if (( KEEP_TEMP )); then
    printf 'Temporales conservados en: %s\n' "$TMP_DIR"
else
    trap 'rm -rf -- "$TMP_DIR"' EXIT
fi
export HOME="$TMP_DIR/home"
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_STATE_HOME="$HOME/.local/state"
export TMPDIR="$TMP_DIR"
export LTOOLS_LANG=es
export LTOOLS_NO_MOUNTS=1
export LTOOLS_NO_AUTO_TERMINAL=1
mkdir -p -- "$HOME" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_STATE_HOME"

run_selected() {
    local selected="$BIN"
    if [[ "${CLI_VARIANT:-0}" == 1 && -n "$CLI_BIN" ]]; then
        selected="$CLI_BIN"
    fi
    if [[ "${CLI_VARIANT:-0}" == 1 && -z "$CLI_BIN" ]]; then
        if [[ "$selected" == *.AppImage ]]; then
            env APPIMAGE_EXTRACT_AND_RUN=1 LTOOLS_CLI=1 "$selected" "$@"
        else
            env LTOOLS_CLI=1 "$selected" "$@"
        fi
        return
    fi
    if [[ "$selected" == *.AppImage ]]; then
        env APPIMAGE_EXTRACT_AND_RUN=1 "$selected" "$@"
    else
        "$selected" "$@"
    fi
}

expect_ok() {
    local name="$1"; shift
    local output="$TMP_DIR/$name.out"
    if ! run_selected "$@" >"$output" 2>&1; then
        sed -n '1,100p' "$output" >&2
        die "$name terminó con error"
    fi
    [[ -s "$output" ]] || die "$name no produjo salida"
    ok "$name"
}

expect_fail() {
    local name="$1"; shift
    local output="$TMP_DIR/$name.out" status=0
    run_selected "$@" >"$output" 2>&1 || status=$?
    (( status != 0 )) || die "$name devolvió éxito inesperadamente"
    ok "$name rechaza entradas inválidas (código $status)"
}

printf 'E2E CLI: contrato del binario y opciones globales...\n'
expect_ok version --version
grep -Eq 'ltools-rs [0-9]+\.[0-9]+\.[0-9]+' "$TMP_DIR/version.out" || die '--version no tiene formato estable'
expect_ok help --help
grep -Fq 'Comandos:' "$TMP_DIR/help.out" || die '--help no contiene el índice de comandos'
expect_ok help-before --no-color --lang en --help
grep -Fq 'Usage:' "$TMP_DIR/help-before.out" || die 'las opciones globales antes del comando no se aplicaron'
if grep -Fq $'\033[' "$TMP_DIR/help-before.out"; then
    die '--no-color dejó secuencias ANSI en la salida CLI'
fi
CLI_VARIANT=1 expect_ok cli-profile
grep -Eq 'Uso: ltools|Usage: ltools' "$TMP_DIR/cli-profile.out" || die 'el perfil CLI distribuible sin argumentos no muestra ayuda'
ok 'perfil CLI distribuible sin argumentos'

printf 'E2E CLI: contratos JSON/TSV y ausencia de estado incidental...\n'
expect_ok capabilities capabilities --format json
jq -e '.schema == "ltools-capabilities-v1" and (.entrypoints.cli.no_arguments == "shows-help") and (.actions | type == "array") and (.host_tools | type == "array")' \
    "$TMP_DIR/capabilities.out" >/dev/null || die 'capabilities no cumple el contrato JSON'
expect_ok actions actions list --format json
jq -e '.schema == "ltools-actions-v1" and (.actions | length > 10) and all(.actions[]; .id and (.args | type == "array") and (.mutating | type == "boolean"))' \
    "$TMP_DIR/actions.out" >/dev/null || die 'actions no cumple el contrato JSON'
expect_ok diagnostics-json diagnostics health --format json
jq -e '.schema == "ltools-diagnostics-v1" and (.probes | length > 0) and all(.probes[]; .key and (.status_code == null or (.status_code | type == "number")))' \
    "$TMP_DIR/diagnostics-json.out" >/dev/null || die 'diagnostics JSON no conserva el resultado de los sondeos'
expect_ok diagnostics-tsv diagnostics network --format tsv
grep -Fq $'key\tcommand\tavailable\tinstalled\tstatus_code\ttimed_out\toutput\terror' "$TMP_DIR/diagnostics-tsv.out" ||
    die 'diagnostics TSV no tiene todas las columnas'
if find "$XDG_STATE_HOME" -type f -print -quit | grep -q .; then
    die 'las consultas CLI crearon planes o estado antes de una acción explícita'
fi
ok 'consultas JSON/TSV sin planes ni estado incidental'

printf 'E2E CLI: familias principales con fixtures aislados...\n'
FIXTURE="$TMP_DIR/fixture"
REPORT="$TMP_DIR/report"
REPO="$TMP_DIR/repository"
mkdir -p -- "$FIXTURE" "$REPO"
printf 'fixture\n' > "$FIXTURE/file.txt"
git -C "$REPO" init -q
git -C "$REPO" config user.name 'LTools CLI E2E'
git -C "$REPO" config user.email 'ltools-cli@example.invalid'
printf 'repo\n' > "$REPO/file.txt"
git -C "$REPO" add file.txt
git -C "$REPO" commit -q -m fixture

expect_ok audit audit --root "$FIXTURE" --out "$REPORT/audit" --no-mounts --min-size-mb 0
expect_ok games games --root "$FIXTURE" --out "$REPORT/games" --no-mounts
expect_ok packages packages --out "$REPORT/packages"
expect_ok software software stores
expect_ok report report view --path "$REPORT/packages/inventory.tsv"
expect_ok git-status git status --repo "$REPO"
expect_ok guide guide cli all
grep -Fq 'ÍNDICE DETALLADO DE FAMILIAS CLI' "$TMP_DIR/guide.out" || die 'guide cli all no recorrió el índice completo'
expect_ok aliases aliases list
expect_ok automation automation list
expect_ok clean-preview clean --path "$FIXTURE" --preview
expect_ok prefix-list prefix list --root "$TMP_DIR"
expect_ok defaults defaults
expect_ok system-status system status
expect_ok boot-status boot status
expect_ok accounts-identity accounts identity
expect_ok native-network native network status
expect_ok diagnostics-health diagnostics health
expect_ok storage-map storage map --path "$FIXTURE" --depth 1 --max-children 10 --format json
jq -e '.schema == "ltools-storage-map-v1" and (.roots | length == 1)' "$TMP_DIR/storage-map.out" >/dev/null ||
    die 'storage map CLI no devolvió su esquema JSON'
expect_ok registry registry status
expect_ok privileges privileges
expect_ok doctor doctor
ok 'familias CLI de auditoría, diagnóstico, almacenamiento, Git, cuentas y sistema'

printf 'E2E CLI: frontera de argumentos, planes y acciones seguras...\n'
expect_ok dry-run-storage --dry-run storage mount /dev/synthetic-ltools --yes
grep -Eq 'Simulación|Plan:' "$TMP_DIR/dry-run-storage.out" || die 'storage --dry-run no informó simulación ni plan'
expect_ok dry-run-account --dry-run accounts create --user ltools-cli-user --home "$TMP_DIR/user" --shell /bin/bash --no-create-home
grep -Fq 'useradd' "$TMP_DIR/dry-run-account.out" || die 'accounts --dry-run no generó el plan nativo'
expect_ok dry-run-network --dry-run native network set-interface --interface lo --state up --yes
grep -Eq 'Simulación|Plan:' "$TMP_DIR/dry-run-network.out" || die 'native --dry-run no informó simulación ni plan'
expect_fail unknown-command does-not-exist
grep -Fq 'comando desconocido: does-not-exist' "$TMP_DIR/unknown-command.out" || die 'el error de comando desconocido no es accionable'
expect_fail missing-format diagnostics health --format
expect_fail malformed-depth storage map --path "$FIXTURE" --depth invalid
expect_fail missing-action actions run
expect_fail missing-plan rollback
unexpected_state=0
while IFS= read -r state_file; do
    case "$state_file" in
        "$XDG_STATE_HOME"/ltools/plans/plan-rust-accounts.tsv|"$XDG_STATE_HOME"/ltools/plans/plan-rust-native.tsv|"$XDG_STATE_HOME"/ltools/plans/plan-rust-storage.tsv) ;;
        *) printf 'Estado inesperado: %s\n' "$state_file" >&2; unexpected_state=1 ;;
    esac
done < <(find "$XDG_STATE_HOME" -type f -print)
(( unexpected_state == 0 )) || die 'la CLI dejó estado fuera de los planes explícitos de dry-run'
ok 'errores de argumentos, --dry-run y rollback sin mutaciones'

printf 'E2E CLI completado correctamente.\n'
