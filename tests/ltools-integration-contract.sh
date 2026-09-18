#!/usr/bin/env bash
# Contrato de proveedor para LTerminal/WTools.
#
# Puede ejecutarse sin compilar (audita el catálogo y los esquemas) o contra
# un binario real/AppImage. No abre la GUI ni ejecuta acciones mutables.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
BIN="${LTOOLS_TEST_BINARY:-}"
RELEASE_DIR="${LTOOLS_TEST_RELEASE_DIR:-$ROOT_DIR/release}"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ltools-integration-contract.XXXXXX")"
trap 'rm -rf -- "$TMP_DIR"' EXIT

die() { printf 'INTEGRATION CONTRACT ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }
skip() { printf '  SKIP  %s\n' "$1"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        --release-dir) (($# >= 2)) || die '--release-dir necesita una ruta'; RELEASE_DIR="$2"; shift ;;
        --static-only) BIN='' ;;
        -h|--help)
            printf 'Uso: %s [--binary RUTA] [--release-dir RUTA] [--static-only]\n' "$0"
            exit 0
            ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

command -v jq >/dev/null 2>&1 || die 'jq es necesario para validar contratos JSON'

json_file() {
    local path="$1" label="$2"
    [[ -s "$path" ]] || die "falta el JSON $label: $path"
    jq empty "$path" >/dev/null || die "JSON inválido en $label: $path"
}

PROJECT="$ROOT_DIR/distribution/ltools-project.json"
PROJECT_SCHEMA="$ROOT_DIR/distribution/ltools-project.schema.json"
TERMINAL_SCHEMA="$ROOT_DIR/appimage/ltools-terminal.schema.json"
ACTION_SCHEMA="$ROOT_DIR/appimage/ltools-actions.schema.json"
json_file "$PROJECT" 'catálogo de proyecto'
json_file "$PROJECT_SCHEMA" 'esquema de proyecto'
json_file "$TERMINAL_SCHEMA" 'esquema de integración terminal'
json_file "$ACTION_SCHEMA" 'esquema del catálogo de acciones'
jq -e '
    .schema == "ltools-project-v1" and .id == "ltools" and .name == "LTools" and
    (.integration.optional == true) and (.integration.standalone_releases_require_it == false) and
    (.integration.hosts | index("LTerminal")) and (.integration.hosts | index("WTools")) and
    (.platforms.linux.preferred_kind == "appimage") and
    (.platforms.windows.preferred_kind == "exe") and
    (.action_catalog.catalog == "ltools-actions.json") and
    (.action_catalog.schema == "ltools-actions.schema.json") and
    (.action_catalog.descriptors.linux == "ltools-actions.json") and
    (.action_catalog.descriptors.windows == "ltools-actions-windows.json") and
    (.verification.hash_algorithm == "sha256") and
    (.verification.signature.algorithm == "ed25519") and
    (.verification.additional_signatures | any(.[]; .algorithm == "openssh-sshsig"))
' "$PROJECT" >/dev/null || die 'el catálogo no cumple el contrato multiplataforma de LTools'
ok 'catálogo de proyecto, plataformas, actualización y firmas'

if [[ -d "$RELEASE_DIR" ]]; then
    for descriptor in ltools-capabilities.json ltools-actions.json ltools-terminal.json ltools-release.json; do
        path="$RELEASE_DIR/$descriptor"
        [[ -e "$path" ]] || continue
        json_file "$path" "release/$descriptor"
    done
    for descriptor in ltools-capabilities-windows.json ltools-actions-windows.json ltools-terminal-windows.json; do
        path="$RELEASE_DIR/$descriptor"
        [[ -e "$path" ]] || continue
        json_file "$path" "release/$descriptor"
    done
    if [[ -s "$RELEASE_DIR/ltools-terminal.json" ]]; then
        jq -e '
            .schema == "ltools-terminal-integration-v1" and
            .integration.optional == true and
            .integration.standalone_releases_require_it == false and
            .host.family == "lterminal" and
            .host.id == "lterminal" and
            (.host.product | IN("LTerminal", "WTools")) and
            (.entrypoint.args == ["menu"]) and
            (.open_arguments | index("--open-path")) and
            (.open_arguments | index("--command")) and
            .action_catalog.shell == "none" and
            .action_catalog.safe_defaults == true and
            (.actions | length >= 10) and
            ([.actions[].actionKey] | length == (unique | length)) and
            ([.actions[].operation] | length == (unique | length)) and
            all(.actions[];
                (.id | type == "string") and (.actionKey | type == "string") and
                (.scope | type == "string") and (.operation | type == "string") and
                (.operation == (.actionKey | gsub("\\."; "-"))) and
                (.command | type == "string") and
                (.executable | type == "string") and (.args | type == "array") and
                .shell == "none" and (.safe | type == "boolean") and
                (.requiresAdmin | type == "boolean"))
        ' "$RELEASE_DIR/ltools-terminal.json" >/dev/null ||
            die 'ltools-terminal.json no cumple el contrato que consume LTerminal'
        ok 'descriptor de integración Linux/WTools compatible con LTerminal'
    fi
    if [[ -s "$RELEASE_DIR/ltools-terminal-windows.json" ]]; then
        jq -e '
            .schema == "ltools-terminal-integration-v1" and .platform == "windows" and
            .application == "WTools" and .host.product == "WTools" and
            .host.id == "wtools" and
            .entrypoint.command == "ltools.exe" and .action_catalog.shell == "none"
        ' "$RELEASE_DIR/ltools-terminal-windows.json" >/dev/null ||
            die 'ltools-terminal-windows.json no identifica WTools correctamente'
        ok 'descriptor Windows identifica WTools y su entrypoint nativo'
    fi
    if [[ -s "$RELEASE_DIR/ltools-actions-windows.json" ]]; then
        jq -e '.schema == "ltools-actions-v1" and .platform == "windows" and (.actions | length >= 1)' \
            "$RELEASE_DIR/ltools-actions-windows.json" >/dev/null ||
            die 'ltools-actions-windows.json no identifica el catálogo WTools correctamente'
        ok 'descriptor Windows de acciones identifica WTools'
    fi
fi

if [[ -z "$BIN" ]]; then
    skip 'binario real no indicado; contrato estático completado'
    exit 0
fi
[[ -x "$BIN" ]] || die "no existe el binario ejecutable: $BIN"

export HOME="$TMP_DIR/home"
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_STATE_HOME="$HOME/.local/state"
export TMPDIR="$TMP_DIR"
export LTOOLS_LANG=es
export LTOOLS_NO_AUTO_TERMINAL=1
mkdir -p -- "$HOME" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_STATE_HOME"

run_binary() {
    if [[ "$BIN" == *.AppImage ]]; then
        APPIMAGE_EXTRACT_AND_RUN=1 "$BIN" "$@"
    else
        "$BIN" "$@"
    fi
}

run_binary capabilities --format json >"$TMP_DIR/capabilities.json" 2>"$TMP_DIR/capabilities.err" ||
    die "el binario no publica capabilities JSON: $(sed -n '1p' "$TMP_DIR/capabilities.err")"
run_binary actions list --format json >"$TMP_DIR/actions.json" 2>"$TMP_DIR/actions.err" ||
    die "el binario no publica actions JSON: $(sed -n '1p' "$TMP_DIR/actions.err")"
run_binary capabilities --format terminal-json >"$TMP_DIR/terminal.json" 2>"$TMP_DIR/terminal.err" ||
    die "el binario no publica terminal JSON: $(sed -n '1p' "$TMP_DIR/terminal.err")"
json_file "$TMP_DIR/capabilities.json" 'capabilities del binario'
json_file "$TMP_DIR/actions.json" 'actions del binario'
json_file "$TMP_DIR/terminal.json" 'descriptor terminal del binario'

platform='linux'
command='ltools'
if [[ "$BIN" == *.exe || "${LTOOLS_TEST_PLATFORM:-}" == windows ]]; then
    platform='windows'
    command='ltools.exe'
fi
jq -e --arg platform "$platform" --arg command "$command" '
    .schema == "ltools-capabilities-v1" and
    (.application | IN("LTools", "WTools")) and .platform == $platform and
    .entrypoints.cli.no_arguments == "shows-help" and
    .terminal_integration.schema == "lterminal-startup-v1" and
    .terminal_integration.capability_request == ["--ltools-capabilities", "--format", "json"] and
        (.actions | length >= 10) and
    ([.actions[].actionKey] | length == (unique | length)) and
    ([.actions[].actionId] | length == (unique | length)) and
    ([.actions[].qualifiedActionKey] | length == (unique | length)) and
    ([.actions[].canonicalKey] | length == (unique | length)) and
    ([.actions[].displayName] | length == (unique | length)) and
    ([.actions[].operation] | length == (unique | length)) and
    all(.actions[]; (.actionKey | test("^[a-z0-9]+(?:[.-][a-z0-9]+)+$"))) and
    all(.actions[];
        (.id | type == "string") and (.legacyId | type == "string") and
        (.actionId == .canonicalKey) and (.actionId == .qualifiedActionKey) and
        (.command | type == "string") and
        (.actionKey | type == "string") and (.scope | type == "string") and
        (.qualifiedActionKey == (($platform + ".") + .actionKey)) and
        (.canonicalKey == .qualifiedActionKey) and
        (.operation | type == "string") and
        (.operation == (.actionKey | gsub("\\."; "-"))) and
        (.displayName | type == "string" and length > 0) and
        (.menuPath | type == "array" and length == 2) and
        (.executable == $command) and
        (.args | type == "array") and .shell == "none" and
        (.safe | type == "boolean") and (.requiresAdmin | type == "boolean") and
        (.requiresCommands | type == "array"))
' "$TMP_DIR/capabilities.json" >/dev/null || die 'capabilities del binario no cumple el contrato de consumo'
jq -e --arg platform "$platform" '
    .schema == "ltools-terminal-integration-v1" and
    .platform == $platform and
    (.host.family == "lterminal") and
    (.host.product == (if $platform == "windows" then "WTools" else "LTerminal" end)) and
    (.host.id == (if $platform == "windows" then "wtools" else "lterminal" end)) and
    (.entrypoint.command == (if $platform == "windows" then "ltools.exe" else "ltools" end)) and
    (.entrypoint.args == ["menu"]) and
    ([.actions[].actionId] | length == (unique | length)) and
    all(.actions[]; (.id == .actionId) and (.legacyId | type == "string") and
        (.actionId == .canonicalKey) and (.actionId == .qualifiedActionKey) and
        (.qualifiedActionKey == (($platform + ".") + .actionKey)))
' "$TMP_DIR/terminal.json" >/dev/null || die 'el descriptor terminal del binario no declara la identidad canónica de la plataforma'
jq -e --arg platform "$platform" --arg command "$command" '
    .schema == "ltools-actions-v1" and .platform == $platform and
    (.actions | length >= 10) and
    ([.actions[].actionKey] | length == (unique | length)) and
    ([.actions[].actionId] | length == (unique | length)) and
    ([.actions[].operation] | length == (unique | length)) and
    ([.actions[].label] | length == (unique | length)) and
    ([.actions[].shortLabel] | length == (unique | length)) and
    ([.actions[].qualifiedActionKey] | length == (unique | length)) and
    ([.actions[].canonicalKey] | length == (unique | length)) and
    all(.actions[]; (.actionKey | test("^[a-z0-9]+(?:[.-][a-z0-9]+)+$"))) and
    all(.actions[];
        (.id | type == "string") and (.legacyId | type == "string") and
        (.actionId == .canonicalKey) and (.actionId == .qualifiedActionKey) and
        (.command | type == "string") and
        (.id == .actionId) and (.legacyId | type == "string" and length > 0) and
        (.actionKey | type == "string") and (.scope | type == "string") and
        (.qualifiedActionKey == (($platform + ".") + .actionKey)) and
        (.canonicalKey == .qualifiedActionKey) and
        (.operation | type == "string") and
        (.operation == (.actionKey | gsub("\\."; "-"))) and
        (.label | type == "string" and length > 0) and
        (.shortLabel | type == "string" and length > 0) and
        (.displayName | type == "string" and length > 0) and
        (.menuPath | type == "array" and length == 2) and
        (.description | type == "string" and length > 0) and
        (.group | type == "string" and length > 0) and
        (.invocation.executable == $command) and
        (.invocation.args == ["actions", "run", .actionId]) and
        (.invocation.target == .target) and
        (.args | type == "array") and (.target | type == "string") and
        (.targetPolicy | IN("none", "explicit-only")) and
        (.mutating | type == "boolean") and (.supports | index("dry-run"))
    ) and
    any(.actions[]; .mutating == true and .targetPolicy == "explicit-only")
' "$TMP_DIR/actions.json" >/dev/null || die 'actions del binario no conserva seguridad, objetivos y dry-run'
ok "binario real publica contrato de capacidades y catálogo de acciones ($platform)"

printf 'Contrato de integración LTools verificado completamente.\n'
