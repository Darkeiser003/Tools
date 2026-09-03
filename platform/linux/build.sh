#!/usr/bin/env bash
#
# Build reproducible de LTools.
#
# Produce un tar.gz y, opcionalmente, un AppImage autocontenido con el
# lanzador, backend Rust release y documentación. Los scripts de soporte quedan
# fuera del paquete y no participan en la ejecución normal.

if [[ -z "${BASH_VERSION:-}" ]]; then
    echo "ERROR: este script necesita bash. Ejecútalo como ./build.sh o bash build.sh." >&2
    exit 1
fi

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
MANIFEST="$ROOT_DIR/rust/Cargo.toml"
OUTPUT_DIR="$ROOT_DIR/dist"
RELEASE_DIR="${LTOOLS_RELEASE_DIR:-$ROOT_DIR/release}"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$MANIFEST" | head -n1)"
ARCH="$(uname -m)"

clock_ms() {
    local value
    value="$(date +%s%3N 2>/dev/null || true)"
    if [[ "$value" =~ ^[0-9]+$ ]]; then
        printf '%s' "$value"
    else
        printf '%s000' "$SECONDS"
    fi
}

duration_text() {
    local duration_ms="$1"
    printf '%d.%03d' "$((duration_ms / 1000))" "$((duration_ms % 1000))"
}

CLEAN=0
FAST=0
CHECKS=1
TESTS=1
SMOKE=1
E2E=1
MENU_E2E=1
SOFTWARE_GIT_E2E=1
PACKAGE=1
APPIMAGE=1
APPIMAGE_REQUIRED=0
FUSE_REQUIRED=0
NO_RUN=1
OFFLINE=0
NON_INTERACTIVE=0
WINDOWS_WINE=0
WINDOWS_TARGET="${LTOOLS_WINDOWS_TARGET:-x86_64-pc-windows-gnu}"
WINDOWS_WINE_RUNNER="${LTOOLS_WINE_RUNNER:-}"
WINDOWS_WINE_PREFIX="${LTOOLS_WINE_PREFIX:-}"
WINDOWS_WINE_INSTALL_MONO=0
EXPLICIT_OPTIONS=0
JOBS="${CARGO_BUILD_JOBS:-2}"
CURRENT_STEP="inicio"
BUILD_STARTED=$SECONDS
BUILD_STARTED_MS="$(clock_ms)"
BUILD_STARTED_AT="$(date --iso-8601=seconds 2>/dev/null || date)"
BUILD_ID="$(date +%Y%m%d-%H%M%S)-$$"
LOG_FILE=""
TIMINGS_FILE=""
NO_LOG=0
WINDOWS_WINE_ARTIFACT_DIR=""
WINDOWS_WINE_ARTIFACT=""
WINDOWS_WINE_LOG=""
SIGNING_REQUIRED=0
SIGNING_PRIVATE_KEY_FILE=""
SIGNING_PUBLIC_KEY_FILE=""
STEP_STARTED=$SECONDS
STEP_ACTIVE=0
# init_logging redirige stdout a tee; conserva antes el estado real de la
# terminal para que `./build.sh` sin argumentos siga mostrando su configuración
# interactiva cuando se ejecuta desde una TTY.
INTERACTIVE_TTY=0
if [[ -t 0 && -t 1 ]]; then
    INTERACTIVE_TTY=1
fi

ok() { printf '    \033[32mOK:\033[0m %s\n' "$1"; }
warn() { printf '    \033[33mAVISO:\033[0m %s\n' "$1" >&2; }
die() { printf '    \033[31mERROR:\033[0m %s\n' "$1" >&2; exit 1; }

log_timing() {
    local name="$1" duration_ms="$2" status="$3"
    [[ "$NO_LOG" -eq 1 || -z "$TIMINGS_FILE" ]] && return 0
    printf '%s\t%s\t%s\n' "$name" "$duration_ms" "$status" >>"$TIMINGS_FILE"
}

finish_step() {
    local status="${1:-completed}" now duration_ms
    (( STEP_ACTIVE )) || return 0
    now="$(clock_ms)"
    duration_ms=$((now - STEP_STARTED_MS))
    printf '[TIMING] step=%s duration_ms=%s duration_s=%s status=%s\n' \
        "$CURRENT_STEP" "$duration_ms" "$(duration_text "$duration_ms")" "$status"
    log_timing "$CURRENT_STEP" "$duration_ms" "$status"
    STEP_ACTIVE=0
}

step() {
    finish_step completed
    CURRENT_STEP="$1"
    STEP_STARTED=$SECONDS
    STEP_STARTED_MS="$(clock_ms)"
    STEP_ACTIVE=1
    printf '\n\033[36m==> %s\033[0m\n' "$1"
}

run_logged() {
    local started status now duration_ms
    started="$(clock_ms)"
    printf '[COMMAND]'
    printf ' %q' "$@"
    printf '\n'
    if "$@"; then
        status=0
    else
        status=$?
    fi
    now="$(clock_ms)"
    duration_ms=$((now - started))
    printf '[COMMAND-END] status=%s duration_ms=%s duration_s=%s\n' \
        "$status" "$duration_ms" "$(duration_text "$duration_ms")"
    return "$status"
}

init_logging() {
    local log_parent
    [[ "$NO_LOG" -eq 1 ]] && return 0
    mkdir -p -- "$OUTPUT_DIR" || die "no se puede crear el directorio de logs: $OUTPUT_DIR"
    [[ -n "$LOG_FILE" ]] || LOG_FILE="$OUTPUT_DIR/build-$BUILD_ID.log"
    log_parent="$(dirname -- "$LOG_FILE")"
    mkdir -p -- "$log_parent" || die "no se puede crear el directorio del log: $log_parent"
    TIMINGS_FILE="$OUTPUT_DIR/build-$BUILD_ID-timings.tsv"
    : >"$LOG_FILE" || die "no se puede escribir el log: $LOG_FILE"
    {
        printf '# LTools build log\n'
        printf '# started=%s\n' "$BUILD_STARTED_AT"
        printf '# build_id=%s\n' "$BUILD_ID"
        printf '# root=%s\n' "$ROOT_DIR"
        printf '# output=%s\n' "$OUTPUT_DIR"
        printf '# release=%s\n' "$RELEASE_DIR"
        printf '# version=%s arch=%s user=%s host=%s\n' "$VERSION" "$ARCH" "$(id -un 2>/dev/null || printf unknown)" "$(hostname -s 2>/dev/null || hostname 2>/dev/null || printf unknown)"
        printf '# command='
        printf '%q ' "$0" "$@"
        printf '\n\n'
    } >>"$LOG_FILE"
    printf 'step\tduration_ms\tstatus\n' >"$TIMINGS_FILE"
    # Keep the normal terminal output while preserving a complete transcript.
    exec > >(tee -a "$LOG_FILE") 2>&1
    printf '[LOG] log principal: %s\n' "$LOG_FILE"
    printf '[LOG] tabla de tiempos: %s\n' "$TIMINGS_FILE"
}

on_error() {
    local line="$1"
    finish_step failed
    printf '\n\033[31mLa build falló\033[0m en «%s», línea %s.\n' "$CURRENT_STEP" "$line" >&2
    failed_elapsed_ms="$(clock_ms)"
    failed_elapsed_ms=$((failed_elapsed_ms - BUILD_STARTED_MS))
    printf '[BUILD-FAILED] step=%s line=%s elapsed_ms=%s elapsed_s=%s\n' \
        "$CURRENT_STEP" "$line" "$failed_elapsed_ms" "$(duration_text "$failed_elapsed_ms")" >&2
    exit 1
}
trap 'on_error "$LINENO"' ERR

usage() {
    cat <<EOF
Uso: $0 [opciones]

Compila LTools y genera un tar.gz y, cuando appimagetool está
disponible, un AppImage terminal y otro perfil CLI autocontenido.

Opciones:
  --clean              Limpia rust/target antes de compilar.
  --fast               Perfil release rápido e incremental.
  --skip-checks        Omite fmt, Clippy y comprobaciones de scripts.
  --no-tests           No ejecuta cargo test.
  --no-smoke           No ejecuta los smoke tests posteriores al empaquetado.
  --no-e2e             No ejecuta la prueba E2E aislada de migración/rollback.
  --no-menu-e2e        No ejecuta la E2E de menús y funciones con fixtures aislados.
  --no-software-git-e2e
                       No ejecuta la E2E aislada de stores y operaciones Git.
  --offline            Usa Cargo en modo offline.
  --windows-wine       Compila el target Windows y ejecuta sus pruebas con Wine/Proton.
  --no-windows-wine    Desactiva la etapa Windows bajo Wine/Proton.
  --windows-target T   Target Rust Windows (por defecto: x86_64-pc-windows-gnu).
  --windows-wine-runner RUTA
                       Wine, UMU-Wine o Proton concreto para ejecutar Windows.
  --windows-wine-prefix RUTA
                       Prefijo explícito; por defecto usa uno temporal y aislado.
  --windows-wine-install-mono
                       Permite instalar wine-mono si el runner no lo incluye.
  --no-package         Compila, pero no genera el tar.gz.
  --appimage           Exige y genera el AppImage.
  --no-appimage        No genera el AppImage.
  --require-fuse       Falla si el equipo no puede montar AppImages con FUSE.
  --output DIR         Directorio de salida (por defecto: ./dist).
  --release-dir DIR    Carpeta canónica de publicación (por defecto: ./release).
  --require-signing    Exige claves Ed25519 y una firma válida para release/.
  --allow-unsigned     Permite una release local sin firma aunque CI esté activo.
  --jobs N             Paralelismo de Cargo (por defecto: 2).
  --log FICHERO        Guarda la transcripción completa en esta ruta.
  --no-log             Desactiva el log persistente y la tabla de tiempos.
  --non-interactive    No muestra preguntas de configuración.
  --no-run             Alias de compatibilidad; no se ejecuta la aplicación.
  -h, --help           Muestra esta ayuda.
  --version            Muestra la versión del proyecto.

La build AppImage genera un perfil terminal, un perfil CLI y
ltools-terminal.json para integradores de terminal, además de
ltools-release.json, SHA256SUMS.txt y su firma Ed25519 separada cuando hay
material de firma disponible.

Sin opciones, en una terminal interactiva, permite elegir limpieza, perfil y
validaciones. En CI o con cualquier opción explícita es no interactivo.
EOF
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "falta la herramienta «$1» en PATH"
}

load_signing_material() {
    local config_home
    if [[ -n "${LTOOLS_SIGNING_PRIVATE_KEY_FILE:-}" ]]; then
        SIGNING_PRIVATE_KEY_FILE="$LTOOLS_SIGNING_PRIVATE_KEY_FILE"
    elif [[ -n "${LTERMINAL_SIGNING_PRIVATE_KEY_FILE:-}" ]]; then
        SIGNING_PRIVATE_KEY_FILE="$LTERMINAL_SIGNING_PRIVATE_KEY_FILE"
    else
        config_home="${LTOOLS_CONFIG_HOME:-${XDG_CONFIG_HOME:-${HOME:-$ROOT_DIR/.config}}}"
        SIGNING_PRIVATE_KEY_FILE="$config_home/lterminal/release-signing-private.pem"
    fi
    if [[ -n "${LTOOLS_UPDATE_PUBLIC_KEY_FILE:-}" ]]; then
        SIGNING_PUBLIC_KEY_FILE="$LTOOLS_UPDATE_PUBLIC_KEY_FILE"
    elif [[ -n "${LTERMINAL_UPDATE_PUBLIC_KEY_FILE:-}" ]]; then
        SIGNING_PUBLIC_KEY_FILE="$LTERMINAL_UPDATE_PUBLIC_KEY_FILE"
    else
        config_home="${LTOOLS_CONFIG_HOME:-${XDG_CONFIG_HOME:-${HOME:-$ROOT_DIR/.config}}}"
        SIGNING_PUBLIC_KEY_FILE="$config_home/lterminal/release-signing-public.hex"
    fi
    if [[ "${LTOOLS_REQUIRE_SIGNING:-${LTERMINAL_REQUIRE_SIGNING:-${CI:-0}}}" =~ ^(1|true|yes)$ ]]; then
        SIGNING_REQUIRED=1
    fi
    if [[ -n "${LTOOLS_ALLOW_UNSIGNED:-}" && "${LTOOLS_ALLOW_UNSIGNED}" =~ ^(1|true|yes)$ ]]; then
        SIGNING_REQUIRED=0
    fi
    if [[ -n "${LTOOLS_REQUIRE_SIGNING:-}" && "${LTOOLS_REQUIRE_SIGNING}" =~ ^(1|true|yes)$ ]]; then
        SIGNING_REQUIRED=1
    fi
}

release_signature_args() {
    RELEASE_SIGNATURE_ARGS=(release-signature --manifest "$RELEASE_DIR/SHA256SUMS.txt" --signature "$RELEASE_DIR/SHA256SUMS.txt.sig")
    if [[ -r "$SIGNING_PRIVATE_KEY_FILE" ]]; then
        RELEASE_SIGNATURE_ARGS+=(--private-key-file "$SIGNING_PRIVATE_KEY_FILE")
    fi
    if [[ -r "$SIGNING_PUBLIC_KEY_FILE" ]]; then
        RELEASE_SIGNATURE_ARGS+=(--public-key-file "$SIGNING_PUBLIC_KEY_FILE")
    fi
}

prepare_release_signature() {
    local private_available=0 public_available=0
    [[ -r "$SIGNING_PRIVATE_KEY_FILE" || -n "${LTOOLS_SIGNING_PRIVATE_KEY:-${LTERMINAL_SIGNING_PRIVATE_KEY:-}}" ]] && private_available=1
    [[ -r "$SIGNING_PUBLIC_KEY_FILE" || -n "${LTOOLS_UPDATE_PUBLIC_KEY:-${LTERMINAL_UPDATE_PUBLIC_KEY:-}}" ]] && public_available=1
    run_logged "$BIN" release-checksums --output "$RELEASE_DIR/SHA256SUMS.txt" --artifacts-dir "$RELEASE_DIR"
    if (( private_available && public_available )); then
        release_signature_args
        run_logged "$BIN" "${RELEASE_SIGNATURE_ARGS[@]}"
        release_signature_args
        run_logged "$BIN" "${RELEASE_SIGNATURE_ARGS[@]}" --verify
        ok 'SHA256SUMS.txt firmado y verificado con Ed25519'
    else
        rm -f -- "$RELEASE_DIR/SHA256SUMS.txt.sig"
        if (( SIGNING_REQUIRED )); then
            die "release estricta: faltan la clave privada y/o pública Ed25519; se esperaban $SIGNING_PRIVATE_KEY_FILE y $SIGNING_PUBLIC_KEY_FILE"
        fi
        warn "release local sin firma: no se encontraron ambas claves Ed25519; se conserva SHA256SUMS.txt y se retira cualquier .sig antiguo"
    fi
    if [[ "$RELEASE_DIR" != "$OUTPUT_DIR" ]]; then
        cp -a -- "$RELEASE_DIR/SHA256SUMS.txt" "$OUTPUT_DIR/SHA256SUMS.txt"
        if [[ -s "$RELEASE_DIR/SHA256SUMS.txt.sig" ]]; then
            cp -a -- "$RELEASE_DIR/SHA256SUMS.txt.sig" "$OUTPUT_DIR/SHA256SUMS.txt.sig"
        else
            rm -f -- "$OUTPUT_DIR/SHA256SUMS.txt.sig"
        fi
    fi
}

ask_yes_no() {
    local prompt="$1" default="$2" answer hint='s/N'
    [[ "$default" -eq 1 ]] && hint='S/n'
    while true; do
        if ! read -r -p "$prompt [$hint] " answer; then
            printf '\n'
            [[ "$default" -eq 1 ]]
            return
        fi
        case "${answer,,}" in
            '') [[ "$default" -eq 1 ]]; return ;;
            s|si|sí|y|yes) return 0 ;;
            n|no) return 1 ;;
            *) warn 'Responde s/sí o n/no; Enter conserva el valor predeterminado.' ;;
        esac
    done
}

configure_interactive() {
    if [[ "$NON_INTERACTIVE" -eq 1 || "$EXPLICIT_OPTIONS" -eq 1 ||
        "${CI:-}" =~ ^(1|true|yes)$ || "$INTERACTIVE_TTY" -ne 1 ]]; then
        return
    fi
    printf '\n\033[36mConfiguración de build (Enter conserva el valor actual):\033[0m\n'
    if ask_yes_no 'Limpiar rust/target antes de compilar' "$CLEAN"; then CLEAN=1; else CLEAN=0; fi
    if ask_yes_no 'Usar perfil release rápido' "$FAST"; then FAST=1; else FAST=0; fi
    if ask_yes_no 'Ejecutar validaciones fmt, Clippy, lanzadores y tests' "$CHECKS"; then CHECKS=1; else CHECKS=0; fi
    if ask_yes_no 'Ejecutar cargo test' "$TESTS"; then TESTS=1; else TESTS=0; fi
    if ask_yes_no 'Ejecutar smoke tests' "$SMOKE"; then SMOKE=1; else SMOKE=0; fi
    if ask_yes_no 'Ejecutar prueba E2E de migración y rollback' "$E2E"; then E2E=1; else E2E=0; fi
    if ask_yes_no 'Ejecutar E2E de menús y funciones aisladas' "$MENU_E2E"; then MENU_E2E=1; else MENU_E2E=0; fi
    if ask_yes_no 'Ejecutar E2E de stores y Git' "$SOFTWARE_GIT_E2E"; then SOFTWARE_GIT_E2E=1; else SOFTWARE_GIT_E2E=0; fi
    if ask_yes_no 'Compilar y probar también Windows con Wine/Proton' "$WINDOWS_WINE"; then WINDOWS_WINE=1; else WINDOWS_WINE=0; fi
    if ask_yes_no 'Generar el paquete tar.gz' "$PACKAGE"; then PACKAGE=1; else PACKAGE=0; fi
    if ask_yes_no 'Generar también el AppImage' "$APPIMAGE"; then APPIMAGE=1; else APPIMAGE=0; fi
}

parse_args() {
    while (($#)); do
        EXPLICIT_OPTIONS=1
        case "$1" in
            --clean) CLEAN=1 ;;
            --fast) FAST=1 ;;
            --skip-checks|--no-checks) CHECKS=0 ;;
            --no-tests) TESTS=0 ;;
            --no-smoke) SMOKE=0 ;;
            --no-e2e) E2E=0; MENU_E2E=0; SOFTWARE_GIT_E2E=0 ;;
            --no-menu-e2e) MENU_E2E=0 ;;
            --no-software-git-e2e) SOFTWARE_GIT_E2E=0 ;;
            --offline) OFFLINE=1 ;;
            --windows-wine|--wine-windows) WINDOWS_WINE=1 ;;
            --no-windows-wine|--no-wine-windows) WINDOWS_WINE=0 ;;
            --windows-target)
                (($# >= 2)) || die '--windows-target necesita un target'
                WINDOWS_TARGET="$2"; shift ;;
            --windows-wine-runner|--wine-runner)
                (($# >= 2)) || die '--windows-wine-runner necesita una ruta'
                WINDOWS_WINE_RUNNER="$2"; shift ;;
            --windows-wine-prefix|--wine-prefix)
                (($# >= 2)) || die '--windows-wine-prefix necesita una ruta'
                WINDOWS_WINE_PREFIX="$2"; shift ;;
            --windows-wine-install-mono) WINDOWS_WINE_INSTALL_MONO=1 ;;
            --no-package) PACKAGE=0 ;;
            --appimage) APPIMAGE=1; APPIMAGE_REQUIRED=1 ;;
            --no-appimage) APPIMAGE=0 ;;
            --require-fuse) FUSE_REQUIRED=1; APPIMAGE=1; APPIMAGE_REQUIRED=1 ;;
            --no-run) NO_RUN=1 ;;
            --non-interactive) NON_INTERACTIVE=1 ;;
            --output)
                (($# >= 2)) || die '--output necesita un directorio'
                OUTPUT_DIR="$2"; shift ;;
        --release-dir)
            (($# >= 2)) || die '--release-dir necesita un directorio'
            RELEASE_DIR="$2"; shift ;;
        --require-signing) SIGNING_REQUIRED=1 ;;
        --allow-unsigned) SIGNING_REQUIRED=0 ;;
            --jobs)
                (($# >= 2)) || die '--jobs necesita un número'
                [[ "$2" =~ ^[1-9][0-9]*$ ]] || die '--jobs necesita un número positivo'
                JOBS="$2"; shift ;;
            --log)
                (($# >= 2)) || die '--log necesita un fichero'
                LOG_FILE="$2"; shift ;;
            --no-log) NO_LOG=1 ;;
            -h|--help) usage; exit 0 ;;
            --version) printf '%s\n' "$VERSION"; exit 0 ;;
            *) die "argumento desconocido: $1 (usa --help)" ;;
        esac
        shift
    done
}

cargo_args=()
[[ "$OFFLINE" -eq 1 ]] && cargo_args+=(--offline)

configure_cargo_profile() {
    export CARGO_BUILD_JOBS="$JOBS"
    export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"
    if [[ "$FAST" -eq 1 ]]; then
        export CARGO_PROFILE_RELEASE_OPT_LEVEL=1
        export CARGO_PROFILE_RELEASE_LTO=false
        export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=256
        export CARGO_PROFILE_RELEASE_STRIP=none
        export CARGO_PROFILE_RELEASE_DEBUG=1
        export CARGO_PROFILE_RELEASE_INCREMENTAL=true
        ok "Perfil rápido: incremental, sin LTO y con símbolos"
    else
        export CARGO_PROFILE_RELEASE_OPT_LEVEL=s
        export CARGO_PROFILE_RELEASE_LTO=true
        export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
        export CARGO_PROFILE_RELEASE_STRIP=true
        export CARGO_PROFILE_RELEASE_DEBUG=0
        export CARGO_PROFILE_RELEASE_INCREMENTAL=false
        export CARGO_PROFILE_RELEASE_PANIC=abort
        ok 'Perfil release: LTO y binario optimizado'
    fi
}

parse_args "$@"
init_logging "$@"
load_signing_material
configure_interactive
if [[ "$NO_LOG" -eq 0 ]]; then
    printf '[CONFIG] clean=%s fast=%s checks=%s tests=%s smoke=%s e2e=%s menu_e2e=%s software_git_e2e=%s package=%s appimage=%s offline=%s jobs=%s windows_wine=%s windows_target=%s release_dir=%s\n' \
        "$CLEAN" "$FAST" "$CHECKS" "$TESTS" "$SMOKE" "$E2E" "$MENU_E2E" "$SOFTWARE_GIT_E2E" "$PACKAGE" "$APPIMAGE" "$OFFLINE" "$JOBS" "$WINDOWS_WINE" "$WINDOWS_TARGET" "$RELEASE_DIR"
    printf '[CONFIG] signing_required=%s private_key_file=%s public_key_file=%s\n' "$SIGNING_REQUIRED" "$SIGNING_PRIVATE_KEY_FILE" "$SIGNING_PUBLIC_KEY_FILE"
fi

[[ -f "$MANIFEST" ]] || die "no existe $MANIFEST"
require_command cargo
require_command rustc
require_command tar
require_command sed
if [[ "$APPIMAGE" -eq 1 ]] && ! command -v appimagetool >/dev/null 2>&1; then
    if [[ "$APPIMAGE_REQUIRED" -eq 1 ]]; then
        die 'falta appimagetool; instálalo o ejecuta con --no-appimage'
    fi
    warn 'appimagetool no está disponible; se generará solo el tar.gz. Usa --appimage para exigirlo.'
    APPIMAGE=0
fi
if [[ "$APPIMAGE" -eq 1 ]]; then
    require_command appimagetool
    if command -v mksquashfs >/dev/null 2>&1; then
        ok 'mksquashfs disponible para el empaquetado'
    else
        warn 'mksquashfs no está en PATH; se confiará en el runtime interno de appimagetool.'
    fi
    if [[ -c /dev/fuse ]] &&
        { command -v fusermount3 >/dev/null 2>&1 || command -v fusermount >/dev/null 2>&1; }; then
        ok 'runtime FUSE disponible para probar el AppImage directamente'
    elif [[ "$FUSE_REQUIRED" -eq 1 ]]; then
        die 'FUSE no está disponible: falta /dev/fuse o fusermount/fusermount3'
    else
        warn 'FUSE no está disponible; el AppImage se generará, pero se probará mediante extracción.'
        warn 'Para habilitar ejecución directa en Arch Linux y derivados: instala fuse2 y carga «sudo modprobe fuse».'
    fi
fi
if [[ "$APPIMAGE" -eq 1 && "$SMOKE" -eq 1 ]]; then
    require_command timeout
    require_command stat
    require_command grep
fi
if [[ "$MENU_E2E" -eq 1 ]]; then
    require_command timeout
fi

step 'Comprobando toolchain'
printf '    Cargo: %s\n' "$(cargo --version)"
printf '    Rustc: %s\n' "$(rustc --version)"
printf '    Versión del proyecto: %s\n' "$VERSION"
[[ "$ARCH" == x86_64 || "$ARCH" == aarch64 || "$ARCH" == armv7l ]] || warn "arquitectura no probada: $ARCH"

configure_cargo_profile

if [[ "$CLEAN" -eq 1 ]]; then
    step 'Limpiando artefactos Rust'
    run_logged cargo clean --manifest-path "$MANIFEST"
    ok 'rust/target limpiado'
fi

if [[ "$CHECKS" -eq 1 ]]; then
    step 'Validando formato Rust'
    run_logged cargo fmt --manifest-path "$MANIFEST" -- --check
    ok 'rustfmt correcto'

    step 'Validando Clippy'
    run_logged cargo clippy --manifest-path "$MANIFEST" "${cargo_args[@]}" -- -D warnings
    ok 'Clippy sin avisos'

    step 'Validando lanzadores, build y tests Bash'
    run_logged "$ROOT_DIR/tests/encoding.sh"
    ok 'codificaciones UTF-8/UTF-8 BOM/ANSI correctas'
    while IFS= read -r -d '' file; do
        run_logged bash -n "$file"
    done < <(find "$ROOT_DIR" -maxdepth 5 -type f -name '*.sh' -print0)
    ok 'sintaxis de scripts correcta'

    cargo_home_dir="${CARGO_HOME:-${HOME:-/tmp}/.cargo}"
    if command -v cargo-audit >/dev/null 2>&1 && [[ -d "$cargo_home_dir/advisory-db" ]]; then
        step 'Auditando dependencias Rust'
        (cd "$ROOT_DIR/rust" && run_logged cargo audit --file Cargo.lock --no-fetch)
        ok 'cargo audit sin vulnerabilidades conocidas'
    else
        warn 'cargo-audit o su base local no está disponible; se omite la auditoría de seguridad.'
    fi
    if command -v cargo-deny >/dev/null 2>&1; then
        step 'Validando licencias y fuentes Rust'
        (cd "$ROOT_DIR/rust" && run_logged cargo deny check)
        ok 'cargo-deny correcto'
    else
        warn 'cargo-deny no está disponible; se omite la validación de licencias y fuentes.'
    fi
fi

if [[ "$TESTS" -eq 1 ]]; then
    step 'Ejecutando pruebas Rust'
    run_logged cargo test --manifest-path "$MANIFEST" "${cargo_args[@]}"
    ok 'cargo test correcto'
fi

if [[ "$CHECKS" -eq 1 ]] && command -v rustup >/dev/null 2>&1 &&
    installed_targets="$(rustup target list --installed 2>/dev/null || true)" &&
    grep -Fxq "$WINDOWS_TARGET" <<<"$installed_targets"; then
    step 'Comprobando compatibilidad cruzada Windows'
    run_logged cargo check --manifest-path "$MANIFEST" "${cargo_args[@]}" --target "$WINDOWS_TARGET" --jobs "$JOBS"
    ok "backend Rust compatible con $WINDOWS_TARGET"
else
    warn "El target $WINDOWS_TARGET no está instalado; se omite la comprobación cruzada Windows."
fi

step 'Validando contratos LTools'
run_logged "$ROOT_DIR/tests/contracts.sh"
ok 'contratos LTools correctos'

step 'Compilando backend Rust release'
run_logged cargo build --manifest-path "$MANIFEST" "${cargo_args[@]}" --release
BIN="$ROOT_DIR/rust/target/release/ltools"
[[ -x "$BIN" ]] || die "Cargo terminó, pero no apareció $BIN"
run_logged "$BIN" --version >/dev/null
ok "binario generado: $BIN"

if [[ "$WINDOWS_WINE" -eq 1 ]]; then
    step 'Compilando y probando Windows con Wine/Proton'
    WINDOWS_WINE_ARTIFACT_DIR="$OUTPUT_DIR/windows-wine"
    WINDOWS_WINE_ARTIFACT="$WINDOWS_WINE_ARTIFACT_DIR/ltools-$VERSION-windows-${WINDOWS_TARGET%%-*}.exe"
    WINDOWS_WINE_LOG="$WINDOWS_WINE_ARTIFACT_DIR/windows-wine-$BUILD_ID.log"
    wine_args=(
        --target "$WINDOWS_TARGET"
        --output "$WINDOWS_WINE_ARTIFACT_DIR"
        --jobs "$JOBS"
    )
    [[ "$CLEAN" -eq 1 ]] && wine_args+=(--clean)
    [[ "$FAST" -eq 1 ]] && wine_args+=(--fast)
    [[ "$OFFLINE" -eq 1 ]] && wine_args+=(--offline)
    [[ "$TESTS" -eq 0 || "$SMOKE" -eq 0 || "$E2E" -eq 0 || "$MENU_E2E" -eq 0 ]] && wine_args+=(--no-tests)
    [[ "$PACKAGE" -eq 0 ]] && wine_args+=(--no-package)
    [[ "$NON_INTERACTIVE" -eq 1 || "$EXPLICIT_OPTIONS" -eq 1 ]] && wine_args+=(--non-interactive)
    [[ -n "$WINDOWS_WINE_RUNNER" ]] && wine_args+=(--runner "$WINDOWS_WINE_RUNNER")
    [[ -n "$WINDOWS_WINE_PREFIX" ]] && wine_args+=(--prefix "$WINDOWS_WINE_PREFIX")
    [[ "$WINDOWS_WINE_INSTALL_MONO" -eq 1 ]] && wine_args+=(--install-mono)
    [[ "$NO_LOG" -eq 0 ]] && wine_args+=(--log "$WINDOWS_WINE_LOG")
    run_logged "$ROOT_DIR/tests/linux/windows-wine.sh" "${wine_args[@]}"
    ok 'compilación y pruebas Windows bajo Wine/Proton correctas'
fi

if [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]]; then
    step 'Construyendo paquete distribuible'
    mkdir -p "$OUTPUT_DIR"
    # La carpeta dist es salida regenerable. Retirar aquí solo artefactos
    # reconocibles evita mezclar una versión anterior con la release actual.
    find "$OUTPUT_DIR" -maxdepth 1 -type f \
        \( -name 'ltools-*.AppImage' -o -name 'ltools-*.tar.gz' \
        -o -name 'ltools-*.zip' -o -name 'ltools-*.exe' \) -delete
    STAGING="$(mktemp -d "$OUTPUT_DIR/.ltools-build.XXXXXX")"
    cleanup_staging() {
        [[ -n "${STAGING:-}" && -d "$STAGING" ]] && rm -rf -- "$STAGING"
    }
    trap cleanup_staging EXIT
    PACKAGE_NAME="ltools-$VERSION-linux-$ARCH"
    PACKAGE_DIR="$STAGING/$PACKAGE_NAME"
    mkdir -p "$PACKAGE_DIR/rust/target/release"

    copy_file() {
        local source="$1"
        [[ -e "$ROOT_DIR/$source" ]] || die "falta el archivo de distribución: $source"
        mkdir -p -- "$(dirname -- "$PACKAGE_DIR/$source")"
        cp -a -- "$ROOT_DIR/$source" "$PACKAGE_DIR/$source"
    }

    # El paquete runtime solo necesita la fachada compatible; los builders y
    # wrappers de desarrollo pertenecen al repositorio, no a la distribución.
    for file in ltools.sh ltools-cli.sh; do
        copy_file "$file"
    done
    cp -a -- "$BIN" "$PACKAGE_DIR/rust/target/release/ltools"
    copy_file README.md
    # Descriptor machine-readable generado por el mismo backend que se
    # distribuye, para que terminales y frontends conozcan las capacidades
    # reales sin interpretar la ayuda humana.
    "$BIN" capabilities --format json >"$PACKAGE_DIR/ltools-capabilities.json"
    "$BIN" capabilities --format terminal-json >"$PACKAGE_DIR/ltools-terminal.json"
    TERMINAL_DESCRIPTOR_ARTIFACT="$OUTPUT_DIR/ltools-terminal.json"
    CAPABILITIES_ARTIFACT="$OUTPUT_DIR/ltools-capabilities.json"
    TERMINAL_SCHEMA_ARTIFACT="$OUTPUT_DIR/ltools-terminal.schema.json"
    cp -a -- "$PACKAGE_DIR/ltools-terminal.json" "$TERMINAL_DESCRIPTOR_ARTIFACT"
    cp -a -- "$PACKAGE_DIR/ltools-capabilities.json" "$CAPABILITIES_ARTIFACT"
    cp -a -- "$ROOT_DIR/appimage/ltools-terminal.schema.json" "$TERMINAL_SCHEMA_ARTIFACT"
    copy_file appimage/ltools-capabilities.schema.json
    cp -a -- "$ROOT_DIR/appimage/ltools-terminal.schema.json" "$PACKAGE_DIR/ltools-terminal.schema.json"
    grep -Fq '"schema": "ltools-capabilities-v1"' "$PACKAGE_DIR/ltools-capabilities.json" \
        || die 'el descriptor JSON de capacidades no es válido'
    grep -Fq '"schema": "ltools-terminal-integration-v1"' "$PACKAGE_DIR/ltools-terminal.json" \
        || die 'el descriptor JSON de terminal no es válido'
    if command -v jq >/dev/null 2>&1; then
        jq -e '
          .schema == "ltools-capabilities-v1" and
          .application == "LTools" and
          .distribution.linux.standalone == true and
          .distribution.windows.standalone == true and
          .external_integrations.optional == true and
          .external_integrations.standalone_releases_require_it == false
        ' "$PACKAGE_DIR/ltools-capabilities.json" >/dev/null \
            || die 'el contrato JSON autónomo no supera la validación estructural'
        jq -e '
          .schema == "ltools-terminal-integration-v1" and
          .integration.optional == true and
          .integration.standalone_releases_require_it == false and
          .integration.exclusive_host_family == "lterminal" and
          (.host.known_products | index("LTerminal")) != null and
          (.host.known_products | index("WinSlim Terminal")) != null and
          .required_terminal_capability == "lterminal-startup-v1" and
          (.open_arguments | index("--command")) != null and
          (.actions | length >= 15) and
          all(.actions[]; (.id | length > 0) and (.executable | length > 0) and
            (.args | type == "array") and .terminal == true and .shell == "none")
        ' "$PACKAGE_DIR/ltools-terminal.json" >/dev/null \
            || die 'el descriptor JSON de integración no supera la validación estructural'
        jq -e . "$PACKAGE_DIR/ltools-terminal.schema.json" >/dev/null \
            || die 'el esquema JSON de integración de terminal no es válido'
        ok 'contratos JSON validados con jq'
    else
        warn 'jq no está disponible; se omite la validación estructural adicional de JSON.'
    fi
    mkdir -p "$PACKAGE_DIR/tests/linux"
    cp -a -- "$ROOT_DIR/tests/contracts.sh" "$PACKAGE_DIR/tests/"
    cp -a -- "$ROOT_DIR/tests/encoding.sh" "$PACKAGE_DIR/tests/"
    cp -a -- "$ROOT_DIR/tests/linux"/*.sh "$PACKAGE_DIR/tests/linux/"
    chmod +x "$PACKAGE_DIR"/*.sh "$PACKAGE_DIR/tests"/*.sh "$PACKAGE_DIR/tests/linux"/*.sh \
        "$PACKAGE_DIR/rust/target/release/ltools"
    cat > "$PACKAGE_DIR/BUILD-INFO.txt" <<EOF
LTools $VERSION
Arquitectura: $ARCH
Backend: Rust release
Compilado: $(date --iso-8601=seconds)
Origen: $ROOT_DIR
Uso: ./ltools.sh --rust --help
EOF

    if [[ "$PACKAGE" -eq 1 ]]; then
        ARTIFACT="$OUTPUT_DIR/$PACKAGE_NAME.tar.gz"
        rm -f -- "$ARTIFACT"
        run_logged tar -C "$STAGING" -czf "$ARTIFACT" "$PACKAGE_NAME"
        [[ -s "$ARTIFACT" ]] || die 'no se pudo crear el paquete tar.gz'
        PACKAGE_LIST="$STAGING/$PACKAGE_NAME.list"
        run_logged tar -tzf "$ARTIFACT" >"$PACKAGE_LIST"
        if grep -Eq '/(platform|windows)/|/build\.sh$' "$PACKAGE_LIST"; then
            die 'el paquete runtime contiene código de build o código de otra plataforma'
        fi
        ok "paquete generado: $ARTIFACT"
    fi
    if [[ "$APPIMAGE" -eq 1 ]]; then
        step 'Construyendo AppImage'
        build_appimage_variant() {
            local variant="$1" artifact="$2" desktop_source="$3" appdir appstream_meta
            appdir="$STAGING/AppDir-$variant"
            mkdir -p "$appdir"
            cp -a -- "$PACKAGE_DIR/." "$appdir/"
            if [[ "$variant" == cli ]]; then
                cp -a -- "$ROOT_DIR/appimage/AppRun" "$appdir/AppRun-main"
                cp -a -- "$ROOT_DIR/appimage/AppRun-cli" "$appdir/AppRun"
            else
                cp -a -- "$ROOT_DIR/appimage/AppRun" "$appdir/AppRun"
            fi
            cp -a -- "$desktop_source" "$appdir/ltools.desktop"
            cp -a -- "$ROOT_DIR/appimage/ltools.svg" "$appdir/ltools.svg"
            mkdir -p "$appdir/usr/share/applications"
            cp -a -- "$desktop_source" "$appdir/usr/share/applications/ltools.desktop"
            mkdir -p "$appdir/usr/share/metainfo"
            sed "s/@VERSION@/$VERSION/g" "$ROOT_DIR/appimage/org.ltools.LTools.metainfo.xml.in" \
                > "$appdir/usr/share/metainfo/org.ltools.LTools.metainfo.xml"
            chmod +x "$appdir/AppRun" "$appdir/AppRun-main" 2>/dev/null || true
            rm -f -- "$artifact"
            appstream_meta="$appdir/usr/share/metainfo/org.ltools.LTools.metainfo.xml"
            if command -v appstreamcli >/dev/null 2>&1; then
                run_logged appstreamcli validate --no-net "$appstream_meta"
            else
                warn 'appstreamcli no está en PATH; se omite la validación independiente del manifiesto.'
            fi
            run_logged appimagetool --no-appstream "$appdir" "$artifact"
            chmod +x "$artifact" || die "no se pudo aplicar el permiso ejecutable a $artifact"
            [[ -s "$artifact" ]] || die "no se pudo crear el AppImage $variant"
            ok "AppImage $variant generado: $artifact"
        }

        APPIMAGE_ARTIFACT="$OUTPUT_DIR/$PACKAGE_NAME.AppImage"
        build_appimage_variant terminal "$APPIMAGE_ARTIFACT" "$ROOT_DIR/appimage/ltools.desktop"
        CLI_APPIMAGE_ARTIFACT="$OUTPUT_DIR/$PACKAGE_NAME-cli.AppImage"
        build_appimage_variant cli "$CLI_APPIMAGE_ARTIFACT" "$ROOT_DIR/appimage/ltools-cli.desktop"
        CLI_SMOKE_OUTPUT="$STAGING/cli-appimage-smoke.log"
        APPIMAGE_EXTRACT_AND_RUN=1 "$CLI_APPIMAGE_ARTIFACT" >"$CLI_SMOKE_OUTPUT" 2>&1 \
            || die 'el AppImage CLI no pudo mostrar la ayuda sin argumentos'
        grep -Fq 'Uso: ltools' "$CLI_SMOKE_OUTPUT" || die 'el AppImage CLI no mostró la ayuda sin argumentos'
        ok 'AppImage CLI verificado: sin argumentos muestra la ayuda'
        RUNNER_ARTIFACT="$OUTPUT_DIR/run-ltools.sh"
        cp -a -- "$ROOT_DIR/appimage/run-ltools.sh" "$RUNNER_ARTIFACT"
        chmod +x "$RUNNER_ARTIFACT"
        ok "Lanzador con fallback FUSE generado: $RUNNER_ARTIFACT"
    fi
else
    warn 'Se omitió la generación de artefactos distribuibles.'
fi

if [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]]; then
    step 'Publicando artefactos en la carpeta release'
    mkdir -p -- "$RELEASE_DIR"
    # release/ es la carpeta que se puede subir directamente a GitHub. Se
    # limpia solo de artefactos LTools para evitar mezclar versiones, pero no
    # se toca ningún fichero ajeno que el usuario haya guardado allí.
    release_output_real="$(readlink -f -- "$OUTPUT_DIR" 2>/dev/null || realpath -- "$OUTPUT_DIR")"
    release_dir_real="$(readlink -f -- "$RELEASE_DIR" 2>/dev/null || realpath -- "$RELEASE_DIR")"
    if [[ "$release_output_real" != "$release_dir_real" ]]; then
        find "$RELEASE_DIR" -maxdepth 1 -type f \
            \( -name "ltools-$VERSION-*" -o -name 'ltools-*.json' \
            -o -name 'ltools-*.schema.json' -o -name 'run-ltools.sh' \) -delete
    fi

    copy_to_release() {
        local file="$1" destination
        [[ -f "$file" ]] || return 0
        destination="$RELEASE_DIR/$(basename -- "$file")"
        if [[ "$(readlink -f -- "$file" 2>/dev/null || realpath -- "$file")" != \
            "$(readlink -f -- "$destination" 2>/dev/null || realpath -- "$destination")" ]]; then
            cp -a -- "$file" "$destination"
        fi
    }

    # Publica los artefactos Linux recién generados y los artefactos Windows
    # que pueda haber dejado el builder nativo Windows en dist/windows.
    while IFS= read -r -d '' file; do copy_to_release "$file"; done < <(
        find "$OUTPUT_DIR" -maxdepth 1 -type f \
            \( -name "ltools-$VERSION-*" -o -name 'ltools-capabilities.json' \
            -o -name 'ltools-terminal.json' -o -name 'ltools-*.schema.json' \
            -o -name 'run-ltools.sh' \) -print0
    )
    if [[ -d "$ROOT_DIR/dist/windows" ]]; then
        while IFS= read -r -d '' file; do copy_to_release "$file"; done < <(
            find "$ROOT_DIR/dist/windows" -maxdepth 1 -type f \
                \( -name "ltools-$VERSION-windows-*" -o -name 'ltools-capabilities.json' \
                -o -name 'ltools-terminal.json' -o -name 'ltools-*.schema.json' \) -print0
        )
    fi
    for file in \
        "$ROOT_DIR/distribution/ltools-project.json" \
        "$ROOT_DIR/distribution/ltools-project.schema.json" \
        "$ROOT_DIR/distribution/ltools-release.schema.json"; do
        copy_to_release "$file"
    done
    ok "carpeta release preparada: $RELEASE_DIR"

    step 'Generando manifiesto verificable de release'
    require_command sha256sum
    RELEASE_MANIFEST_ARTIFACT="$RELEASE_DIR/ltools-release.json"
    run_logged "$BIN" release-manifest \
        --output "$RELEASE_MANIFEST_ARTIFACT" \
        --repository "${LTOOLS_GITHUB_REPOSITORY:-Darkeiser003/Tools}" \
        --tag "${LTOOLS_GITHUB_TAG:-v$VERSION}" \
        --artifacts-dir "$RELEASE_DIR"
    if [[ "$RELEASE_DIR" != "$OUTPUT_DIR" ]]; then
        cp -a -- "$RELEASE_MANIFEST_ARTIFACT" "$OUTPUT_DIR/ltools-release.json"
    fi
    cp -a -- "$ROOT_DIR/distribution/ltools-project.json" "$OUTPUT_DIR/ltools-project.json"
    cp -a -- "$ROOT_DIR/distribution/ltools-project.schema.json" "$OUTPUT_DIR/ltools-project.schema.json"
    cp -a -- "$ROOT_DIR/distribution/ltools-release.schema.json" "$OUTPUT_DIR/ltools-release.schema.json"
    if command -v jq >/dev/null 2>&1; then
        jq -e '.schema == "ltools-release-v1" and .application == "LTools" and .hash_algorithm == "sha256" and (.artifacts | length > 0)' \
            "$RELEASE_MANIFEST_ARTIFACT" >/dev/null \
            || die 'el manifiesto de release no supera la validación estructural'
        jq -e '.schema == "ltools-project-v1" and .repository == "Darkeiser003/Tools" and .platforms.linux and .platforms.windows' \
            "$OUTPUT_DIR/ltools-project.json" >/dev/null \
            || die 'el descriptor de proyecto no supera la validación estructural'
        jq -e '(.properties.schema.const == "ltools-project-v1") and ((.properties.platforms.required | index("linux")) != null) and ((.properties.platforms.required | index("windows")) != null)' \
            "$ROOT_DIR/distribution/ltools-project.schema.json" >/dev/null \
            || die 'el esquema de proyecto no supera la validación estructural'
        ok 'manifiesto de release y descriptor de proyecto validados con jq'
    else
        warn 'jq no está disponible; se omite la validación estructural adicional del manifiesto de release.'
    fi
    ok "manifiesto generado: $RELEASE_MANIFEST_ARTIFACT"
    step 'Generando y firmando comprobaciones de artefactos'
    prepare_release_signature
    step 'Ejecutando E2E de artefactos release'
    release_e2e_args=(--release-dir "$RELEASE_DIR" --version "$VERSION" --signature-verifier "$BIN")
    [[ "$APPIMAGE" -eq 0 ]] && release_e2e_args+=(--no-appimage)
    [[ "$PACKAGE" -eq 0 ]] && release_e2e_args+=(--no-package)
    if [[ -r "$SIGNING_PUBLIC_KEY_FILE" ]]; then
        release_e2e_args+=(--signature-public-key-file "$SIGNING_PUBLIC_KEY_FILE")
    fi
    run_logged "$ROOT_DIR/tests/release-e2e.sh" "${release_e2e_args[@]}"
    ok 'artefactos release verificados'
fi

if [[ "$SMOKE" -eq 1 ]]; then
    step 'Ejecutando smoke tests'
    smoke_args=(--binary "$BIN")
    if [[ "$APPIMAGE" -eq 1 ]]; then
        smoke_args+=(--appimage "$APPIMAGE_ARTIFACT" --runner "$RUNNER_ARTIFACT" \
            --log "$OUTPUT_DIR/appimage-smoke.log")
    fi
    run_logged "$ROOT_DIR/tests/linux/smoke.sh" "${smoke_args[@]}"
    ok 'smoke tests correctos'
fi

if [[ "$E2E" -eq 1 ]]; then
    step 'Ejecutando prueba E2E aislada'
    e2e_args=(--binary "$BIN")
    [[ "$APPIMAGE" -eq 1 ]] && e2e_args+=(--appimage "$APPIMAGE_ARTIFACT")
    run_logged "$ROOT_DIR/tests/linux/e2e.sh" "${e2e_args[@]}"
    ok 'prueba E2E correcta'
fi

if [[ "$MENU_E2E" -eq 1 ]]; then
    step 'Ejecutando E2E de menús y funciones'
    menu_e2e_args=(--binary "$BIN")
    [[ "$APPIMAGE" -eq 1 ]] && menu_e2e_args+=(--appimage "$APPIMAGE_ARTIFACT")
    run_logged "$ROOT_DIR/tests/linux/menu-e2e.sh" "${menu_e2e_args[@]}"
    ok 'E2E de menús y funciones correcta'
fi

if [[ "$SOFTWARE_GIT_E2E" -eq 1 ]]; then
    step 'Ejecutando E2E de stores y Git'
    run_logged "$ROOT_DIR/tests/linux/software-git-e2e.sh" --binary "$BIN"
    ok 'E2E de stores y Git correcta'
fi

if [[ "$NO_LOG" -eq 0 ]]; then
    step 'Validando logs y tiempos del build'
    [[ -s "$LOG_FILE" ]] || die "el log está vacío: $LOG_FILE"
    [[ -s "$TIMINGS_FILE" ]] || die "la tabla de tiempos está vacía: $TIMINGS_FILE"
    grep -Fq '[LOG] log principal:' "$LOG_FILE" || die 'el log no contiene su cabecera de ejecución'
    grep -Fq '[COMMAND-END] status=0 duration_ms=' "$LOG_FILE" || die 'el log no registra comandos completados'
    grep -Fq $'step\tduration_ms\tstatus' "$TIMINGS_FILE" || die 'la tabla de tiempos no tiene cabecera'
    awk -F '\t' 'NR > 1 && $2 !~ /^[0-9]+$/ { exit 1 }' "$TIMINGS_FILE" || die 'la tabla de tiempos contiene duraciones inválidas'
    finish_step completed
    build_total_ms="$(clock_ms)"
    build_total_ms=$((build_total_ms - BUILD_STARTED_MS))
    printf 'build-total\t%s\tcompleted\n' "$build_total_ms" >>"$TIMINGS_FILE"
    printf '[BUILD-END] status=0 duration_ms=%s duration_s=%s\n' \
        "$build_total_ms" "$(duration_text "$build_total_ms")"
    grep -Fq '[BUILD-END] status=0 duration_ms=' "$LOG_FILE" || die 'el log no registra el final del build'
    ok "logs y tiempos validados: $LOG_FILE"
fi

if [[ "$NO_RUN" -eq 1 ]]; then
    build_total_ms="$(clock_ms)"
    build_total_ms=$((build_total_ms - BUILD_STARTED_MS))
    printf '\nBuild terminada correctamente en %ss.\n' "$(duration_text "$build_total_ms")"
    printf 'Backend: %s\n' "$BIN"
    [[ "$PACKAGE" -eq 1 ]] && printf 'Tarball: %s\n' "$ARTIFACT"
    [[ "$APPIMAGE" -eq 1 ]] && printf 'AppImage: %s\n' "$APPIMAGE_ARTIFACT"
    [[ "$APPIMAGE" -eq 1 ]] && printf 'AppImage CLI: %s\n' "$CLI_APPIMAGE_ARTIFACT"
    [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]] && printf 'Contrato terminal: %s\n' "$TERMINAL_DESCRIPTOR_ARTIFACT"
    [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]] && printf 'Release publicable: %s\n' "$RELEASE_DIR"
    [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]] && printf 'Checksums release: %s\n' "$RELEASE_DIR/SHA256SUMS.txt"
    [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]] && [[ -s "$RELEASE_DIR/SHA256SUMS.txt.sig" ]] && printf 'Firma release: %s\n' "$RELEASE_DIR/SHA256SUMS.txt.sig"
    [[ "$APPIMAGE" -eq 1 ]] && printf 'Lanzador recomendado: %s\n' "$RUNNER_ARTIFACT"
    [[ "$WINDOWS_WINE" -eq 1 && -f "$WINDOWS_WINE_ARTIFACT" ]] && printf 'Windows validado con Wine/Proton: %s\n' "$WINDOWS_WINE_ARTIFACT"
    [[ "$WINDOWS_WINE" -eq 1 && "$NO_LOG" -eq 0 && -s "$WINDOWS_WINE_LOG" ]] && printf 'Log Windows Wine/Proton: %s\n' "$WINDOWS_WINE_LOG"
    [[ "$NO_LOG" -eq 0 ]] && printf 'Log del build: %s\n' "$LOG_FILE"
    [[ "$NO_LOG" -eq 0 ]] && printf 'Tiempos: %s\n' "$TIMINGS_FILE"
fi

# El resumen usa condiciones `&&` opcionales; fijar explícitamente el estado
# evita que `--no-log` o `--no-appimage` conviertan una build correcta en código 1.
exit 0
