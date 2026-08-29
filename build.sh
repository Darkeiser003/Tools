#!/usr/bin/env bash
#
# Build reproducible de LTools.
#
# Produce un tar.gz y, opcionalmente, un AppImage autocontenido con el
# lanzador, módulos Bash, backend Rust release y documentación. El AppImage
# usa las herramientas del sistema y ofrece instalar las que falten.

if [[ -z "${BASH_VERSION:-}" ]]; then
    echo "ERROR: este script necesita bash. Ejecútalo como ./build.sh o bash build.sh." >&2
    exit 1
fi

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
MANIFEST="$ROOT_DIR/rust/Cargo.toml"
OUTPUT_DIR="$ROOT_DIR/dist"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$MANIFEST" | head -n1)"
ARCH="$(uname -m)"

CLEAN=0
FAST=0
CHECKS=1
TESTS=1
SMOKE=1
E2E=1
MENU_E2E=1
PACKAGE=1
APPIMAGE=1
APPIMAGE_REQUIRED=0
FUSE_REQUIRED=0
NO_RUN=1
OFFLINE=0
NON_INTERACTIVE=0
EXPLICIT_OPTIONS=0
JOBS="${CARGO_BUILD_JOBS:-2}"
CURRENT_STEP="inicio"
BUILD_STARTED=$SECONDS

ok() { printf '    \033[32mOK:\033[0m %s\n' "$1"; }
warn() { printf '    \033[33mAVISO:\033[0m %s\n' "$1" >&2; }
die() { printf '    \033[31mERROR:\033[0m %s\n' "$1" >&2; exit 1; }
step() { CURRENT_STEP="$1"; printf '\n\033[36m==> %s\033[0m\n' "$1"; }

on_error() {
    local line="$1"
    printf '\n\033[31mLa build falló\033[0m en «%s», línea %s.\n' "$CURRENT_STEP" "$line" >&2
    exit 1
}
trap 'on_error "$LINENO"' ERR

usage() {
    cat <<EOF
Uso: $0 [opciones]

Compila LTools y genera un tar.gz y, cuando appimagetool está
disponible, un AppImage autocontenido.

Opciones:
  --clean              Limpia rust/target antes de compilar.
  --fast               Perfil release rápido e incremental.
  --skip-checks        Omite fmt, Clippy y comprobación de sintaxis Bash.
  --no-tests           No ejecuta cargo test.
  --no-smoke           No ejecuta los smoke tests posteriores al empaquetado.
  --no-e2e             No ejecuta la prueba E2E aislada de migración/rollback.
  --no-menu-e2e        No ejecuta la E2E de menús y funciones con fixtures aislados.
  --offline            Usa Cargo en modo offline.
  --no-package         Compila, pero no genera el tar.gz.
  --appimage           Exige y genera el AppImage.
  --no-appimage        No genera el AppImage.
  --require-fuse       Falla si el equipo no puede montar AppImages con FUSE.
  --output DIR         Directorio de salida (por defecto: ./dist).
  --jobs N             Paralelismo de Cargo (por defecto: 2).
  --non-interactive    No muestra preguntas de configuración.
  --no-run             Alias de compatibilidad; no se ejecuta la aplicación.
  -h, --help           Muestra esta ayuda.
  --version            Muestra la versión del proyecto.

Sin opciones, en una terminal interactiva, permite elegir limpieza, perfil y
validaciones. En CI o con cualquier opción explícita es no interactivo.
EOF
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "falta la herramienta «$1» en PATH"
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
        "${CI:-}" =~ ^(1|true|yes)$ || ! -t 0 || ! -t 1 ]]; then
        return
    fi
    printf '\n\033[36mConfiguración de build (Enter conserva el valor actual):\033[0m\n'
    if ask_yes_no 'Limpiar rust/target antes de compilar' "$CLEAN"; then CLEAN=1; else CLEAN=0; fi
    if ask_yes_no 'Usar perfil release rápido' "$FAST"; then FAST=1; else FAST=0; fi
    if ask_yes_no 'Ejecutar validaciones fmt, Clippy y Bash' "$CHECKS"; then CHECKS=1; else CHECKS=0; fi
    if ask_yes_no 'Ejecutar cargo test' "$TESTS"; then TESTS=1; else TESTS=0; fi
    if ask_yes_no 'Ejecutar smoke tests' "$SMOKE"; then SMOKE=1; else SMOKE=0; fi
    if ask_yes_no 'Ejecutar prueba E2E de migración y rollback' "$E2E"; then E2E=1; else E2E=0; fi
    if ask_yes_no 'Ejecutar E2E de menús y funciones aisladas' "$MENU_E2E"; then MENU_E2E=1; else MENU_E2E=0; fi
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
            --no-e2e) E2E=0; MENU_E2E=0 ;;
            --no-menu-e2e) MENU_E2E=0 ;;
            --offline) OFFLINE=1 ;;
            --no-package) PACKAGE=0 ;;
            --appimage) APPIMAGE=1; APPIMAGE_REQUIRED=1 ;;
            --no-appimage) APPIMAGE=0 ;;
            --require-fuse) FUSE_REQUIRED=1; APPIMAGE=1; APPIMAGE_REQUIRED=1 ;;
            --no-run) NO_RUN=1 ;;
            --non-interactive) NON_INTERACTIVE=1 ;;
            --output)
                (($# >= 2)) || die '--output necesita un directorio'
                OUTPUT_DIR="$2"; shift ;;
            --jobs)
                (($# >= 2)) || die '--jobs necesita un número'
                [[ "$2" =~ ^[1-9][0-9]*$ ]] || die '--jobs necesita un número positivo'
                JOBS="$2"; shift ;;
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
configure_interactive

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
        warn 'Para habilitar ejecución directa en CachyOS/Arch: instala fuse2 y carga «sudo modprobe fuse».'
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
    cargo clean --manifest-path "$MANIFEST"
    ok 'rust/target limpiado'
fi

if [[ "$CHECKS" -eq 1 ]]; then
    step 'Validando formato Rust'
    cargo fmt --manifest-path "$MANIFEST" -- --check
    ok 'rustfmt correcto'

    step 'Validando Clippy'
    cargo clippy --manifest-path "$MANIFEST" "${cargo_args[@]}" -- -D warnings
    ok 'Clippy sin avisos'

    step 'Validando scripts Bash'
    while IFS= read -r -d '' file; do
        bash -n "$file"
    done < <(find "$ROOT_DIR" -maxdepth 3 -type f -name '*.sh' -print0)
    ok 'sintaxis Bash correcta'
fi

if [[ "$TESTS" -eq 1 ]]; then
    step 'Ejecutando pruebas Rust'
    cargo test --manifest-path "$MANIFEST" "${cargo_args[@]}"
    ok 'cargo test correcto'
fi

step 'Validando contratos LTools'
"$ROOT_DIR/tests/contracts.sh"
ok 'contratos LTools correctos'

step 'Compilando backend Rust release'
cargo build --manifest-path "$MANIFEST" "${cargo_args[@]}" --release
BIN="$ROOT_DIR/rust/target/release/ltools"
[[ -x "$BIN" ]] || die "Cargo terminó, pero no apareció $BIN"
"$BIN" --version >/dev/null
ok "binario generado: $BIN"

if [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]]; then
    step 'Construyendo paquete distribuible'
    mkdir -p "$OUTPUT_DIR"
    STAGING="$(mktemp -d "$OUTPUT_DIR/.ltools-build.XXXXXX")"
    trap 'rm -rf -- "$STAGING"' EXIT
    PACKAGE_NAME="ltools-$VERSION-linux-$ARCH"
    PACKAGE_DIR="$STAGING/$PACKAGE_NAME"
    mkdir -p "$PACKAGE_DIR/rust/target/release" "$PACKAGE_DIR/scripts" "$PACKAGE_DIR/docs"

    copy_file() {
        local source="$1"
        [[ -e "$ROOT_DIR/$source" ]] || die "falta el archivo de distribución: $source"
        cp -a -- "$ROOT_DIR/$source" "$PACKAGE_DIR/$source"
    }

    for file in ltools.sh rust-tools.sh rust-audit.sh build.sh; do
        copy_file "$file"
    done
    cp -a -- "$ROOT_DIR/scripts"/*.sh "$PACKAGE_DIR/scripts/"
    cp -a -- "$ROOT_DIR/scripts/lib" "$PACKAGE_DIR/scripts/lib"
    cp -a -- "$BIN" "$PACKAGE_DIR/rust/target/release/ltools"
    copy_file README.md
    for file in docs/ARCHITECTURE.md docs/ROADMAP.md docs/LTERMINAL-INTEGRATION.md; do
        copy_file "$file"
    done
    cp -a -- "$ROOT_DIR/rust/README.md" "$PACKAGE_DIR/rust/README.md"
    cp -a -- "$ROOT_DIR/tests" "$PACKAGE_DIR/tests"
    chmod +x "$PACKAGE_DIR"/*.sh "$PACKAGE_DIR/tests"/*.sh "$PACKAGE_DIR/rust/target/release/ltools"
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
        tar -C "$STAGING" -czf "$ARTIFACT" "$PACKAGE_NAME"
        [[ -s "$ARTIFACT" ]] || die 'no se pudo crear el paquete tar.gz'
        tar -tzf "$ARTIFACT" >/dev/null
        ok "paquete generado: $ARTIFACT"
    fi
    if [[ "$APPIMAGE" -eq 1 ]]; then
        step 'Construyendo AppImage'
        APPDIR="$STAGING/AppDir"
        mkdir -p "$APPDIR"
        cp -a -- "$PACKAGE_DIR/." "$APPDIR/"
        cp -a -- "$ROOT_DIR/appimage/AppRun" "$APPDIR/AppRun"
        cp -a -- "$ROOT_DIR/appimage/ltools.desktop" "$APPDIR/ltools.desktop"
        cp -a -- "$ROOT_DIR/appimage/ltools.svg" "$APPDIR/ltools.svg"
        mkdir -p "$APPDIR/usr/share/applications"
        cp -a -- "$ROOT_DIR/appimage/ltools.desktop" "$APPDIR/usr/share/applications/ltools.desktop"
        chmod +x "$APPDIR/AppRun"
        APPIMAGE_ARTIFACT="$OUTPUT_DIR/$PACKAGE_NAME.AppImage"
        rm -f -- "$APPIMAGE_ARTIFACT"
        appimagetool "$APPDIR" "$APPIMAGE_ARTIFACT"
        if ! chmod +x "$APPIMAGE_ARTIFACT"; then
            if command -v sudo >/dev/null 2>&1 && [[ -t 0 && -t 1 ]]; then
                warn 'no se pudo aplicar el permiso ejecutable como usuario; se solicitará sudo.'
                sudo chmod +x "$APPIMAGE_ARTIFACT"
            else
                die "no se pudo aplicar el permiso ejecutable a $APPIMAGE_ARTIFACT"
            fi
        fi
        [[ -s "$APPIMAGE_ARTIFACT" ]] || die 'no se pudo crear el AppImage'
        ok "AppImage generado: $APPIMAGE_ARTIFACT"
        RUNNER_ARTIFACT="$OUTPUT_DIR/run-ltools.sh"
        cp -a -- "$ROOT_DIR/appimage/run-ltools.sh" "$RUNNER_ARTIFACT"
        chmod +x "$RUNNER_ARTIFACT"
        ok "Lanzador con fallback FUSE generado: $RUNNER_ARTIFACT"
    fi
else
    warn 'Se omitió la generación de artefactos distribuibles.'
fi

if [[ "$SMOKE" -eq 1 ]]; then
    step 'Ejecutando smoke tests'
    smoke_args=(--binary "$BIN")
    if [[ "$APPIMAGE" -eq 1 ]]; then
        smoke_args+=(--appimage "$APPIMAGE_ARTIFACT" --runner "$RUNNER_ARTIFACT" \
            --log "$OUTPUT_DIR/appimage-smoke.log")
    fi
    "$ROOT_DIR/tests/smoke.sh" "${smoke_args[@]}"
    ok 'smoke tests correctos'
fi

if [[ "$E2E" -eq 1 ]]; then
    step 'Ejecutando prueba E2E aislada'
    e2e_args=(--binary "$BIN")
    [[ "$APPIMAGE" -eq 1 ]] && e2e_args+=(--appimage "$APPIMAGE_ARTIFACT")
    "$ROOT_DIR/tests/e2e.sh" "${e2e_args[@]}"
    ok 'prueba E2E correcta'
fi

if [[ "$MENU_E2E" -eq 1 ]]; then
    step 'Ejecutando E2E de menús y funciones'
    menu_e2e_args=(--binary "$BIN")
    "$ROOT_DIR/tests/menu-e2e.sh" "${menu_e2e_args[@]}"
    ok 'E2E de menús y funciones correcta'
fi

if [[ "$NO_RUN" -eq 1 ]]; then
    printf '\nBuild terminada correctamente en %ss.\n' "$((SECONDS - BUILD_STARTED))"
    printf 'Backend: %s\n' "$BIN"
    [[ "$PACKAGE" -eq 1 ]] && printf 'Tarball: %s\n' "$ARTIFACT"
    [[ "$APPIMAGE" -eq 1 ]] && printf 'AppImage: %s\n' "$APPIMAGE_ARTIFACT"
    [[ "$APPIMAGE" -eq 1 ]] && printf 'Lanzador recomendado: %s\n' "$RUNNER_ARTIFACT"
fi
