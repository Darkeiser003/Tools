#!/usr/bin/env bash
# Build y smoke del ejecutable Windows usando un runner Wine/Proton.
#
# Es una herramienta de validación del host Linux, no el backend de LTools.
# Nunca usa el prefijo Wine del usuario salvo que se pase explícitamente
# --prefix. Por defecto crea un prefijo temporal.

set -Eeuo pipefail
export LTOOLS_LANG=es

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
TARGET="${LTOOLS_WINDOWS_TARGET:-x86_64-pc-windows-gnu}"
RUNNER="${LTOOLS_WINE_RUNNER:-}"
# Este builder puede ejecutarse desde el builder Linux, que también usa
# rust/target. Mantener un target propio evita que --clean borre el binario
# Linux que todavía necesita el empaquetado de la release.
WINDOWS_CARGO_TARGET_DIR="${LTOOLS_WINDOWS_CARGO_TARGET_DIR:-$ROOT_DIR/rust/target/windows-wine}"
PREFIX=""
CREATED_TEMP_PREFIX=0
LOG_PATH=""
ARTIFACT_DIR=""
KEEP_PREFIX=0
DO_BUILD=1
RUN_TESTS=1
DO_PACKAGE=1
DO_CLEAN=0
FAST=0
OFFLINE=0
JOBS="${CARGO_BUILD_JOBS:-2}"
NON_INTERACTIVE=0
INSTALL_MONO=0
REQUIRE_GUI=0
WINE_TIMEOUT_SECONDS="${LTOOLS_WINE_TIMEOUT_SECONDS:-60}"
ARTIFACT_STAGING=""
PACKAGE_TEST_DIR=""

die() { printf 'WINDOWS-WINE ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }
warn() { printf '  AVISO %s\n' "$1"; }
usage() {
    cat <<'EOF'
Uso: tests/linux/windows-wine.sh [opciones]

Compila y prueba el ejecutable Windows con Wine/Proton en un prefijo aislado.

  --runner RUTA       Wine, wine64 o Proton concreto.
  --prefix RUTA       Prefijo explícito; por defecto se crea uno temporal.
  --target TARGET     Target Rust; por defecto x86_64-pc-windows-gnu.
  --log FICHERO       Guarda la salida de las pruebas.
  --output DIR        Copia aquí el ejecutable y la información del build.
  --no-build          Usa el ejecutable Windows ya compilado.
  --clean             Limpia solo los artefactos del target Windows.
  --fast              Usa el perfil release incremental y rápido.
  --offline           Usa Cargo en modo offline.
  --jobs N             Paralelismo de Cargo (por defecto: 2).
  --no-tests          Compila, pero no ejecuta Wine/Proton.
  --no-package        No copia el ejecutable a --output.
  --non-interactive   No pregunta por Wine Mono.
  --install-mono      Acepta la instalación de Wine Mono si el runner no lo trae.
  --require-gui       Falla si no se puede abrir/capturar la GUI Windows bajo Wine.
  --keep-prefix       Conserva el prefijo temporal para inspeccionarlo.
  -h, --help          Muestra esta ayuda.

También acepta LTOOLS_WINE_RUNNER y LTOOLS_WINDOWS_TARGET.
UMU-Wine se prefiere cuando está instalado porque suele evitar los bloqueos
de inicialización de Wine del sistema en equipos con Proton/Steam.
Wine Mono no es necesario para LTools: el binario es Rust y no usa .NET.
Si el runner ya trae Mono, no se instala nada; si no, se ofrece wine-mono.
EOF
}

while (($#)); do
    case "$1" in
        --runner) (($# >= 2)) || die '--runner necesita una ruta'; RUNNER="$2"; shift ;;
        --prefix) (($# >= 2)) || die '--prefix necesita una ruta'; PREFIX="$2"; shift ;;
        --target) (($# >= 2)) || die '--target necesita un target'; TARGET="$2"; shift ;;
        --log) (($# >= 2)) || die '--log necesita un fichero'; LOG_PATH="$2"; shift ;;
        --output) (($# >= 2)) || die '--output necesita un directorio'; ARTIFACT_DIR="$2"; shift ;;
        --no-build) DO_BUILD=0 ;;
        --clean) DO_CLEAN=1 ;;
        --fast) FAST=1 ;;
        --offline) OFFLINE=1 ;;
        --jobs) (($# >= 2)) || die '--jobs necesita un número'; [[ "$2" =~ ^[1-9][0-9]*$ ]] || die '--jobs necesita un número positivo'; JOBS="$2"; shift ;;
        --no-tests|--no-smoke|--no-e2e|--no-menu-e2e) RUN_TESTS=0 ;;
        --no-package) DO_PACKAGE=0 ;;
        --non-interactive) NON_INTERACTIVE=1 ;;
        --install-mono) INSTALL_MONO=1 ;;
        --require-gui) REQUIRE_GUI=1 ;;
        --keep-prefix) KEEP_PREFIX=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ "$TARGET" =~ ^[A-Za-z0-9_\.-]+$ ]] || die "target Rust inválido: $TARGET"
WINDOWS_CARGO_TARGET_DIR="$(realpath -m -- "$WINDOWS_CARGO_TARGET_DIR")"
mkdir -p -- "$WINDOWS_CARGO_TARGET_DIR"
export CARGO_TARGET_DIR="$WINDOWS_CARGO_TARGET_DIR"

find_runner() {
    if [[ -n "$RUNNER" ]]; then
        [[ -x "$RUNNER" ]] || die "el runner no es ejecutable: $RUNNER"
        return
    fi
    local -a candidates=(
        "$HOME/.local/share/umu/compatibilitytools/UMU-Latest/files/bin/wine"
        "/usr/share/steam/compatibilitytools.d/proton-cachyos-native/files/bin/wine"
        "$HOME/.local/share/lutris/runners/wine/wine-ge-8-26-x86_64/bin/wine"
    )
    local candidate
    for candidate in "${candidates[@]}"; do
        if [[ -x "$candidate" ]]; then
            RUNNER="$candidate"
            return
        fi
    done
    RUNNER="$(command -v wine 2>/dev/null || true)"
    [[ -n "$RUNNER" ]] || die 'no se encontró Wine/UMU-Wine. Usa --runner RUTA.'
}

find_runner
[[ -f "$ROOT_DIR/rust/Cargo.toml" ]] || die 'no se encontró rust/Cargo.toml'

cleanup_prefix() {
    if [[ "$KEEP_PREFIX" -eq 0 && "$CREATED_TEMP_PREFIX" -eq 1 && -n "$PREFIX" && -d "$PREFIX" ]]; then
        local wine_server
        wine_server="$(dirname -- "$RUNNER")/wineserver"
        if [[ ! -x "$wine_server" ]]; then
            wine_server="$(command -v wineserver 2>/dev/null || true)"
        fi
        if [[ -n "$wine_server" ]]; then
            WINEPREFIX="$PREFIX" timeout 10 "$wine_server" -k >/dev/null 2>&1 || true
        fi
        # wineserver puede tardar unos instantes en cerrar handles sobre el
        # prefijo. La limpieza nunca debe convertir un smoke correcto en un
        # fallo ni dejar residuos por una carrera de cierre.
        for _ in {1..20}; do
            rm -rf -- "$PREFIX" 2>/dev/null || true
            [[ ! -e "$PREFIX" ]] && break
            sleep 0.25
        done
        if [[ -e "$PREFIX" ]]; then
            warn "no se pudo retirar completamente el prefijo temporal: $PREFIX"
        fi
    fi
}

cleanup_e2e_staging() {
    if [[ -n "$PACKAGE_TEST_DIR" && -d "$PACKAGE_TEST_DIR" ]]; then
        rm -rf -- "$PACKAGE_TEST_DIR" 2>/dev/null || true
    fi
    if [[ -n "$ARTIFACT_STAGING" && -d "$ARTIFACT_STAGING" ]]; then
        rm -rf -- "$ARTIFACT_STAGING" 2>/dev/null || true
    fi
    cleanup_prefix
}
trap cleanup_e2e_staging EXIT

if [[ -z "$PREFIX" ]]; then
    PREFIX="$(mktemp -d "${TMPDIR:-/tmp}/ltools-windows-wine.XXXXXX")"
    CREATED_TEMP_PREFIX=1
    if [[ "$KEEP_PREFIX" -eq 1 ]]; then
        printf 'Prefijo conservado en: %s\n' "$PREFIX"
    fi
else
    PREFIX="$(realpath -m -- "$PREFIX")"
    mkdir -p -- "$PREFIX"
    warn "se usará el prefijo explícito: $PREFIX"
fi

if [[ -z "$LOG_PATH" ]]; then
    log_directory="${ARTIFACT_DIR:-$ROOT_DIR/dist}"
    mkdir -p -- "$log_directory"
    LOG_PATH="$log_directory/windows-wine-$$.log"
else
    mkdir -p -- "$(dirname -- "$LOG_PATH")"
fi
: > "$LOG_PATH"

configure_cargo_profile() {
    export CARGO_BUILD_JOBS="$JOBS"
    if [[ "$FAST" -eq 1 ]]; then
        export CARGO_PROFILE_RELEASE_OPT_LEVEL=1
        export CARGO_PROFILE_RELEASE_LTO=false
        export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=256
        export CARGO_PROFILE_RELEASE_STRIP=none
        export CARGO_PROFILE_RELEASE_DEBUG=1
        export CARGO_PROFILE_RELEASE_INCREMENTAL=true
        ok 'perfil release rápido aplicado al target Windows'
    else
        export CARGO_PROFILE_RELEASE_OPT_LEVEL=s
        export CARGO_PROFILE_RELEASE_LTO=true
        export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
        export CARGO_PROFILE_RELEASE_STRIP=true
        export CARGO_PROFILE_RELEASE_DEBUG=0
        export CARGO_PROFILE_RELEASE_INCREMENTAL=false
        export CARGO_PROFILE_RELEASE_PANIC=abort
    fi
}

cargo_args=()
[[ "$OFFLINE" -eq 1 ]] && cargo_args+=(--offline)

VERSION="$(sed -n 's/^version = \"\([^\"]*\)\"/\1/p' "$ROOT_DIR/rust/Cargo.toml" | head -n1)"
[[ -n "$VERSION" ]] || die 'no se pudo leer la versión de rust/Cargo.toml'
configure_cargo_profile

if [[ "$DO_BUILD" -eq 1 ]]; then
    installed_targets=""
    if command -v rustup >/dev/null 2>&1; then
        installed_targets="$(rustup target list --installed 2>/dev/null || true)"
    fi
    if [[ -n "$installed_targets" ]] &&
        ! grep -Fxq "$TARGET" <<<"$installed_targets"; then
        die "falta el target Rust $TARGET. Instálalo con: rustup target add $TARGET"
    fi
    if [[ "$TARGET" == x86_64-pc-windows-gnu ]] &&
        ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        die 'falta el linker MinGW x86_64-w64-mingw32-gcc para compilar Windows GNU'
    fi
fi

if [[ "$DO_BUILD" -eq 1 ]]; then
    command -v cargo >/dev/null 2>&1 || die 'no se encontró cargo'
    if [[ "$DO_CLEAN" -eq 1 ]]; then
        printf '$ cargo clean --manifest-path rust/Cargo.toml --target %s\n' "$TARGET" | tee -a "$LOG_PATH"
        cargo clean --manifest-path "$ROOT_DIR/rust/Cargo.toml" --target "$TARGET" 2>&1 |
            tee -a "$LOG_PATH"
    fi
    printf '$ cargo build --manifest-path rust/Cargo.toml --release --target %s --jobs %s\n' "$TARGET" "$JOBS" | tee -a "$LOG_PATH"
    cargo build --manifest-path "$ROOT_DIR/rust/Cargo.toml" "${cargo_args[@]}" --release --target "$TARGET" --jobs "$JOBS" 2>&1 |
        tee -a "$LOG_PATH"
fi

WINEXE="$CARGO_TARGET_DIR/$TARGET/release/ltools.exe"
[[ -f "$WINEXE" ]] || die "no existe el ejecutable Windows: $WINEXE"
WINEXE_GUI="$CARGO_TARGET_DIR/$TARGET/release/ltools-gui.exe"
WINEXE_CLI="$CARGO_TARGET_DIR/$TARGET/release/ltools-cli.exe"
if [[ "$DO_BUILD" -eq 1 ]]; then
    cp -a -- "$WINEXE" "$WINEXE_GUI"
    printf '$ cargo build --manifest-path rust/Cargo.toml --release --target %s --features cli --jobs %s\n' "$TARGET" "$JOBS" | tee -a "$LOG_PATH"
    cargo build --manifest-path "$ROOT_DIR/rust/Cargo.toml" "${cargo_args[@]}" --release --target "$TARGET" --features cli --jobs "$JOBS" 2>&1 |
        tee -a "$LOG_PATH"
    [[ -f "$WINEXE" ]] || die "no se generó el perfil CLI Windows: $WINEXE"
    cp -a -- "$WINEXE" "$WINEXE_CLI"
    cp -a -- "$WINEXE_GUI" "$WINEXE"
else
    [[ -f "$WINEXE_GUI" ]] || cp -a -- "$WINEXE" "$WINEXE_GUI"
    [[ -f "$WINEXE_CLI" ]] || die "no existe el perfil CLI Windows: $WINEXE_CLI"
fi

RUNNER_NAME="$(basename -- "$RUNNER")"
RUNNER_MODE="wine"
if [[ "$RUNNER_NAME" == "proton" ]]; then
    RUNNER_MODE="proton"
    export STEAM_COMPAT_DATA_PATH="$PREFIX"
    export STEAM_COMPAT_CLIENT_INSTALL_PATH="${STEAM_COMPAT_CLIENT_INSTALL_PATH:-/usr/share/steam}"
fi

export WINEPREFIX="$PREFIX"
export WINEARCH=win64
export WINEDEBUG="${WINEDEBUG:--all}"
export WINEESYNC="${WINEESYNC:-0}"
export WINEFSYNC="${WINEFSYNC:-0}"

if [[ "$RUN_TESTS" -eq 1 && "$INSTALL_MONO" -eq 0 && "$NON_INTERACTIVE" -eq 0 &&
    "$RUNNER_MODE" == "wine" && -t 0 && -t 1 ]]; then
    read -r -p '¿Instalar Wine Mono en este prefijo temporal? LTools no lo necesita [y/N] ' answer || answer=""
    case "$answer" in
        y|Y|s|S|yes|YES|si|SI|sí|SÍ) INSTALL_MONO=1 ;;
    esac
fi

install_mono_if_needed() {
    [[ "$INSTALL_MONO" -eq 1 ]] || {
        warn 'Wine Mono no se instala: LTools no usa .NET; usa --install-mono si lo necesita otra aplicación'
        return
    }
    local runner_root runner_dir package_manager
    runner_dir="$(dirname -- "$RUNNER")"
    runner_root="$(cd -- "$(dirname -- "$RUNNER")/.." && pwd -P)"
    if [[ -d "$runner_root/share/wine/mono" ||
          -d "$runner_dir/../share/wine/mono" ||
          -d "$runner_dir/../files/share/wine/mono" ]]; then
        ok 'Wine Mono ya está incluido en el runner seleccionado'
        return
    fi
    [[ "$RUNNER_MODE" == "wine" ]] ||
        die 'el runner Proton no incluye Mono y no se puede instalar Mono del host dentro de Proton'
    if command -v pacman >/dev/null 2>&1 && pacman -Q wine-mono >/dev/null 2>&1; then
        ok 'Wine Mono ya está instalado en el sistema'
        return
    fi
    if command -v pacman >/dev/null 2>&1; then
        package_manager='sudo pacman -S --needed wine-mono'
    elif command -v apt-get >/dev/null 2>&1; then
        package_manager='sudo apt-get install -y wine-mono'
    elif command -v dnf >/dev/null 2>&1; then
        package_manager='sudo dnf install -y wine-mono'
    elif command -v zypper >/dev/null 2>&1; then
        package_manager='sudo zypper --non-interactive install wine-mono'
    else
        die 'no se encontró un gestor compatible para instalar wine-mono'
    fi
    printf 'No se encontró Wine Mono en el runner.\n'
    printf 'Se ejecutará: %s\n' "$package_manager"
    if [[ "$EUID" -eq 0 ]]; then
        sh -c "$package_manager" 2>&1 | tee -a "$LOG_PATH"
    else
        command -v sudo >/dev/null 2>&1 || die 'se necesita sudo para instalar wine-mono'
        sh -c "$package_manager" 2>&1 | tee -a "$LOG_PATH"
    fi
    ok 'Wine Mono instalado mediante el gestor del sistema'
}

package_windows_artifact() {
    [[ "$DO_PACKAGE" -eq 1 && -n "$ARTIFACT_DIR" ]] || return 0
    ARTIFACT_DIR="$(realpath -m -- "$ARTIFACT_DIR")"
    mkdir -p -- "$ARTIFACT_DIR"
    local package_arch="${TARGET%%-*}"
    ARTIFACT_STAGING="$(mktemp -d "$ARTIFACT_DIR/.windows-package.XXXXXX")"
    local artifact="$ARTIFACT_STAGING/ltools-$VERSION-windows-$package_arch.exe"
    local cli_artifact="$ARTIFACT_STAGING/ltools-$VERSION-windows-$package_arch-cli.exe"
    local package_zip="$ARTIFACT_STAGING/ltools-$VERSION-windows-$package_arch.zip"
    local metadata="$ARTIFACT_STAGING/ltools-$VERSION-windows-$package_arch-wine.json"
    command -v zip >/dev/null 2>&1 || die 'zip es necesario para generar el paquete portable Windows'
    command -v unzip >/dev/null 2>&1 || die 'unzip es necesario para validar el paquete portable Windows'
    cp -a -- "$WINEXE" "$artifact"
    cp -a -- "$WINEXE_CLI" "$cli_artifact"
    local portable_dir="$ARTIFACT_STAGING/portable"
    mkdir -p -- "$portable_dir"
    cp -a -- "$WINEXE" "$portable_dir/ltools.exe"
    cp -a -- "$WINEXE_CLI" "$portable_dir/ltools-cli.exe"
    for launcher in ltools.ps1 ltools.cmd ltools-cli.ps1 ltools-cli.cmd; do
        [[ -f "$ROOT_DIR/windows/$launcher" ]] || {
            rm -rf -- "$portable_dir"
            die "falta el lanzador Windows $launcher para el ZIP portable"
        }
        cp -a -- "$ROOT_DIR/windows/$launcher" "$portable_dir/$launcher"
    done
    # Una release combinada puede tener el descriptor Linux como canónico;
    # publica además la variante Windows para WinSlim Terminal.
    if [[ -z "${CAPABILITIES:-}" ]]; then
        CAPABILITIES="$(run_windows_timeout capabilities --format json 2>>"$LOG_PATH")" \
            || die 'no se pudo generar el descriptor de capacidades Windows'
    fi
    if [[ -z "${TERMINAL_JSON:-}" ]]; then
        TERMINAL_JSON="$(run_windows_timeout capabilities --format terminal-json 2>>"$LOG_PATH")" \
            || die 'no se pudo generar el descriptor de terminal Windows'
    fi
    printf '%s\n' "$CAPABILITIES" > "$ARTIFACT_STAGING/ltools-capabilities-windows.json"
    printf '%s\n' "$TERMINAL_JSON" > "$ARTIFACT_STAGING/ltools-terminal-windows.json"
    cp -a -- "$ARTIFACT_STAGING/ltools-capabilities-windows.json" "$portable_dir/"
    cp -a -- "$ARTIFACT_STAGING/ltools-terminal-windows.json" "$portable_dir/"
    cp -a -- "$ROOT_DIR/appimage/ltools-capabilities.schema.json" "$portable_dir/"
    cp -a -- "$ROOT_DIR/appimage/ltools-terminal.schema.json" "$portable_dir/"
    cp -a -- "$ROOT_DIR/README.md" "$portable_dir/"
    cat > "$portable_dir/BUILD-INFO.txt" <<EOF
WinSlim-Tools $VERSION
Platform: Windows
Target: $TARGET
GUI: ltools.exe
CLI: ltools-cli.exe
Validated: Wine/Proton
EOF
    (cd "$portable_dir" && zip -q -r "$package_zip" .) || die 'no se pudo crear el ZIP portable Windows'
    [[ -s "$package_zip" ]] || die 'el ZIP portable Windows está vacío'
    unzip -tq "$package_zip" >/dev/null || die 'el ZIP portable Windows está corrupto'

    # La validación se hace desde el ZIP extraído dentro del prefijo aislado,
    # no desde los binarios del target. Así se comprueba el contenido real que
    # recibirá quien descargue el paquete portable.
    PACKAGE_TEST_DIR="$PREFIX/drive_c/ltools-package-e2e-$$"
    mkdir -p -- "$PACKAGE_TEST_DIR"
    unzip -q "$package_zip" -d "$PACKAGE_TEST_DIR" || die 'no se pudo extraer el ZIP portable Windows'
    for required_file in \
        ltools.exe ltools-cli.exe ltools.ps1 ltools.cmd ltools-cli.ps1 ltools-cli.cmd \
        ltools-capabilities-windows.json ltools-terminal-windows.json \
        ltools-capabilities.schema.json ltools-terminal.schema.json README.md BUILD-INFO.txt; do
        [[ -s "$PACKAGE_TEST_DIR/$required_file" ]] || die "el ZIP portable omite $required_file"
    done
    cmp -s -- "$PACKAGE_TEST_DIR/ltools.exe" "$WINEXE" || die 'el perfil GUI extraído del ZIP difiere del ejecutable probado'
    cmp -s -- "$PACKAGE_TEST_DIR/ltools-cli.exe" "$WINEXE_CLI" || die 'el perfil CLI extraído del ZIP difiere del ejecutable probado'
    local package_gui_windows='C:\ltools-package-e2e-'"$$"'\ltools.exe'
    local package_cli_windows='C:\ltools-package-e2e-'"$$"'\ltools-cli.exe'
    local package_version package_help gui_package_version
    package_version="$(run_windows_executable_timeout "$package_cli_windows" --version 2>>"$LOG_PATH")" ||
        die 'el perfil CLI extraído del ZIP no responde a --version'
    grep -Fq 'ltools-rs' <<<"$package_version" || die 'el perfil CLI del ZIP devolvió una versión inesperada'
    package_help="$(run_windows_executable_timeout "$package_cli_windows" 2>>"$LOG_PATH")" ||
        die 'el perfil CLI extraído del ZIP no muestra ayuda sin argumentos'
    grep -Fq 'Uso: ltools' <<<"$package_help" || die 'el perfil CLI del ZIP no muestra la ayuda nativa'
    gui_package_version="$(run_windows_executable_timeout "$package_gui_windows" --version 2>>"$LOG_PATH")" ||
        die 'el perfil GUI extraído del ZIP no responde a --version'
    grep -Fq 'ltools-rs' <<<"$gui_package_version" || die 'el perfil GUI del ZIP devolvió una versión inesperada'
    rm -rf -- "$PACKAGE_TEST_DIR"
    PACKAGE_TEST_DIR=""
    ok 'ZIP portable probado: integridad, contenido y perfiles GUI/CLI extraídos bajo Wine'

    cat > "$metadata" <<EOF
{
  "application": "WinSlim-Tools",
  "version": "$VERSION",
  "platform": "windows",
  "architecture": "$package_arch",
  "artifact": "$(basename -- "$artifact")",
  "cli_artifact": "$(basename -- "$cli_artifact")",
  "portable_artifact": "$(basename -- "$package_zip")",
  "validation": "wine-proton",
  "runner": "$(basename -- "$RUNNER")",
  "target": "$TARGET"
}
EOF
    # Promueve solo el conjunto ya construido y probado. Los archivos previos
    # no se borran antes de que el ZIP extraído haya superado sus comprobaciones.
    local staged_file
    while IFS= read -r -d '' staged_file; do
        mv -f -- "$staged_file" "$ARTIFACT_DIR/${staged_file##*/}"
    done < <(find "$ARTIFACT_STAGING" -maxdepth 1 -type f -print0 | sort -z)
    local published_artifact="$ARTIFACT_DIR/ltools-$VERSION-windows-$package_arch.exe"
    local published_cli_artifact="$ARTIFACT_DIR/ltools-$VERSION-windows-$package_arch-cli.exe"
    local published_package_zip="$ARTIFACT_DIR/ltools-$VERSION-windows-$package_arch.zip"
    local completed_staging="$ARTIFACT_STAGING"
    ARTIFACT_STAGING=""
    rm -rf -- "$completed_staging"
    ok "artefactos Windows bajo Wine validados y publicados: $published_artifact, $published_cli_artifact y $published_package_zip"
}

run_windows_timeout() {
    if [[ "$RUNNER_MODE" == "proton" ]]; then
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" run "$WINEXE" "$@"
    else
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" "$WINEXE" "$@"
    fi
}

run_wine_host_timeout() {
    if [[ "$RUNNER_MODE" == "proton" ]]; then
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" run "$@"
    else
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" "$@"
    fi
}

windows_path_to_prefix() {
    local windows_path="$1"
    [[ "${windows_path:1:2}" == ':\' ]] || return 1
    local drive="${windows_path:0:1}"
    drive="${drive,,}"
    local relative="${windows_path:3}"
    relative="$(printf '%s' "$relative" | tr '\\' '/')"
    printf '%s/drive_%s/%s\n' "$PREFIX" "$drive" "$relative"
}

run_cli_timeout() {
    if [[ "$RUNNER_MODE" == "proton" ]]; then
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" run "$WINEXE_CLI" "$@"
    else
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" "$WINEXE_CLI" "$@"
    fi
}

run_windows_executable_timeout() {
    local executable="$1"
    shift
    if [[ "$RUNNER_MODE" == "proton" ]]; then
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" run "$executable" "$@"
    else
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" "$executable" "$@"
    fi
}

run_probe() {
    if [[ "$RUNNER_MODE" == "proton" ]]; then
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" run cmd /c echo LTOOLS_WINE_OK
    else
        timeout "$WINE_TIMEOUT_SECONDS" "$RUNNER" cmd /c echo LTOOLS_WINE_OK
    fi
}

printf 'Runner: %s\n' "$RUNNER" | tee -a "$LOG_PATH"
printf 'Modo: %s\n' "$RUNNER_MODE" | tee -a "$LOG_PATH"
printf 'Prefijo: %s\n' "$PREFIX" | tee -a "$LOG_PATH"
printf 'Ejecutable: %s\n' "$WINEXE" | tee -a "$LOG_PATH"

if [[ "$RUN_TESTS" -eq 0 ]]; then
    warn 'pruebas Wine/Proton omitidas por configuración'
else
    install_mono_if_needed

    if ! probe_output="$(run_probe 2>&1)"; then
        # No ocultar la causa del fallo detrás del mensaje genérico: los
        # errores del wineserver/socket local son distintos de una consola
        # ausente y permiten separar límites del entorno de fallos de LTools.
        printf '%s\n' "$probe_output" | tee -a "$LOG_PATH" >&2
        die "el runner no pudo iniciar una consola Windows; revisa $LOG_PATH"
    fi
    printf '%s\n' "$probe_output" | tee -a "$LOG_PATH" >/dev/null
    if ! grep -Fq 'LTOOLS_WINE_OK' <<<"$probe_output"; then
        die "el runner no pudo iniciar una consola Windows; revisa $LOG_PATH"
    fi
    ok 'runner Windows inicia una consola aislada'

    # Solo en el prefijo temporal propio se simula C:\WSCore para cubrir el
    # menú WinSlim condicional sin tocar instalaciones ni prefijos del usuario.
    if [[ "$CREATED_TEMP_PREFIX" -eq 1 ]]; then
        mkdir -p -- "$PREFIX/drive_c/WSCore"
    fi

    run_case() {
        local name="$1"
        shift
        printf '\n--- %s ---\n' "$name" | tee -a "$LOG_PATH"
        run_windows_timeout "$@" 2>&1 | tee -a "$LOG_PATH"
    }

    run_case 'version' --version
    run_case 'help' --help
    cli_version_output="$(run_cli_timeout --version 2>>"$LOG_PATH")" ||
        die 'el perfil CLI Windows no respondió a --version'
    grep -Fq 'ltools-rs' <<<"$cli_version_output" ||
        die 'el perfil CLI Windows no devolvió su versión'
    cli_help_output="$(run_cli_timeout --help 2>>"$LOG_PATH")" ||
        die 'el perfil CLI Windows no respondió a --help'
    grep -Fq 'Uso: ltools' <<<"$cli_help_output" ||
        die 'el ejecutable CLI Windows publicado no muestra la ayuda'
    cli_noargs_output="$(run_cli_timeout 2>>"$LOG_PATH")" ||
        die 'el perfil CLI Windows no respondió sin argumentos'
    grep -Fq 'Uso: ltools' <<<"$cli_noargs_output" ||
        die 'el ejecutable CLI Windows sin argumentos no muestra la ayuda'
    ok 'perfil CLI Windows separado abre en modo consola y sin argumentos no inicia la GUI'

    # Validación real del gestor de alias dentro del prefijo Wine. Las rutas
    # son Windows deliberadamente: el registro y el lanzador deben respetar
    # APPDATA/LOCALAPPDATA y no reutilizar rutas POSIX.
    windows_alias_home='C:\ltools-alias-config'
    windows_alias_bin='C:\ltools-alias-bin'
    windows_alias_ensure="$(LTOOLS_ALIAS_HOME="$windows_alias_home" \
        LTOOLS_ALIAS_BIN="$windows_alias_bin" run_cli_timeout aliases ensure 2>>"$LOG_PATH")" ||
        die 'aliases ensure Windows bajo Wine falló'
    grep -Fq 'Alias predeterminados listos:' <<<"$windows_alias_ensure" ||
        die 'aliases ensure Windows no creó el registro'
    grep -Fq 'Lanzador creado:' <<<"$windows_alias_ensure" ||
        die 'aliases ensure Windows no creó el lanzador'
    windows_alias_list="$(LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout aliases list 2>>"$LOG_PATH")" ||
        die 'aliases list Windows bajo Wine falló'
    grep -Fq 'tnet -> native network' <<<"$windows_alias_list" ||
        die 'el registro Windows no contiene el alias nativo de red'
    LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout aliases add tguide guide gui storage >/dev/null 2>>"$LOG_PATH" ||
        die 'aliases add Windows bajo Wine falló'
    windows_alias_guide="$(LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout tguide 2>>"$LOG_PATH")" ||
        die 'la expansión de alias Windows bajo Wine falló'
    grep -Fq 'GUÍA GRÁFICA' <<<"$windows_alias_guide" ||
        die 'el alias Windows no conservó sus argumentos'
    LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout aliases disable tguide >/dev/null 2>>"$LOG_PATH" ||
        die 'aliases disable Windows bajo Wine falló'
    set +e
    windows_disabled_status=0
    LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout tguide >/tmp/ltools-windows-disabled-alias-$$.log 2>&1 ||
        windows_disabled_status=$?
    set -e
    (( windows_disabled_status != 0 )) || die 'un alias Windows desactivado se ejecutó inesperadamente'
    rm -f -- "/tmp/ltools-windows-disabled-alias-$$.log"
    ok 'gestor de alias Windows bajo Wine: registro, expansión y desactivación'
    windows_map_output="$(LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout storage map --path 'C:\Windows' --depth 0 2>>"$LOG_PATH")" ||
        die 'storage map Windows bajo Wine falló'
    grep -Fq 'MAPA DE DISCOS' <<<"$windows_map_output" ||
        die 'storage map Windows no mostró su cabecera'
    windows_map_json="$(LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout storage map --path 'C:\Windows' --depth 0 --format json 2>>"$LOG_PATH")" ||
        die 'storage map JSON Windows bajo Wine falló'
    grep -Fq 'ltools-storage-map-v1' <<<"$windows_map_json" ||
        die 'storage map JSON Windows no declaró su esquema'
    windows_explain_output="$(LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout storage explain --path 'C:\Windows' 2>>"$LOG_PATH")" ||
        die 'storage explain Windows bajo Wine falló'
    grep -Fq 'Windows' <<<"$windows_explain_output" ||
        die 'storage explain Windows no explicó la ruta estándar'
    windows_system_child="$(LTOOLS_LANG=en LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout storage explain --path 'C:\Windows\System32' 2>>"$LOG_PATH")" ||
        die 'storage explain no explicó el contenido de Windows'
    grep -Fq 'Windows system:' <<<"$windows_system_child" ||
        die 'storage explain no aplicó la explicación localizada a subrutas de Windows'
    windows_program_child="$(LTOOLS_LANG=en LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout storage explain --path 'C:\Program Files\Example' 2>>"$LOG_PATH")" ||
        die 'storage explain no explicó una aplicación instalada'
    grep -Fq 'System-managed programs and libraries' <<<"$windows_program_child" ||
        die 'storage explain no describió Program Files en el idioma solicitado'
    windows_appdata_child="$(LTOOLS_LANG=en LTOOLS_ALIAS_HOME="$windows_alias_home" run_cli_timeout storage explain --path 'C:\Users\Example\AppData\Roaming' 2>>"$LOG_PATH")" ||
        die 'storage explain no explicó AppData'
    grep -Fq 'Per-user settings and caches' <<<"$windows_appdata_child" ||
        die 'storage explain no describió los datos AppData en el idioma solicitado'
    ok 'mapa Windows bajo Wine: árbol, JSON y rutas estándar'

    # El limpiador no debe tratar todo LOCALAPPDATA como caché: allí viven
    # perfiles y datos de aplicaciones. TEMP y LOCALAPPDATA\Temp tampoco se
    # deben sumar dos veces cuando apuntan al mismo directorio.
    cleaner_local="$(run_wine_host_timeout cmd /c echo %LOCALAPPDATA% 2>>"$LOG_PATH" | tr -d '\r')"
    [[ -n "$cleaner_local" ]] || die 'Wine no informó LOCALAPPDATA para el usuario aislado'
    cleaner_temp="$cleaner_local\\Temp"
    cleaner_fixture="$(windows_path_to_prefix "$cleaner_local")" ||
        die "LOCALAPPDATA no apunta a una ruta de disco Windows reconocible: $cleaner_local"
    mkdir -p -- "$cleaner_fixture/Temp" "$cleaner_fixture/LTOOLS_UNSAFE_APP_DATA"
    printf 'temporary fixture\n' > "$cleaner_fixture/Temp/temporary-fixture.tmp"
    printf 'keep application data\n' > "$cleaner_fixture/LTOOLS_UNSAFE_APP_DATA/keep.txt"
    cleaner_preview="$(export LOCALAPPDATA="$cleaner_local" TEMP="$cleaner_temp"; \
        run_windows_timeout clean --automatic --preview 2>>"$LOG_PATH")" ||
        die 'la vista previa del limpiador Windows bajo Wine falló'
    printf '%s\n' "$cleaner_preview" >> "$LOG_PATH"
    if grep -Fq 'LTOOLS_UNSAFE_APP_DATA' <<<"$cleaner_preview"; then
        die 'el limpiador Windows clasificó datos de aplicación como caché segura'
    fi
    temp_occurrences="$(grep -F -o -- "$cleaner_temp" <<<"$cleaner_preview" | wc -l || true)"
    [[ "$temp_occurrences" -eq 1 ]] ||
        die "el limpiador Windows no mostró TEMP exactamente una vez (esperado: $cleaner_temp; apariciones: $temp_occurrences)"
    [[ -f "$cleaner_fixture/LTOOLS_UNSAFE_APP_DATA/keep.txt" ]] ||
        die 'la vista previa del limpiador modificó datos de aplicación Windows'
    ok 'limpieza Windows bajo Wine: rutas seguras, sin solape TEMP ni datos de aplicación'

    CAPABILITIES="$(run_windows_timeout capabilities --format json 2>>"$LOG_PATH")" ||
        die 'capabilities --format json falló'
    printf '%s\n' "$CAPABILITIES" | tee -a "$LOG_PATH" >/dev/null
    grep -Fq '"platform": "windows"' <<<"$CAPABILITIES" ||
        die 'el JSON Windows no declara la plataforma correcta'
    if command -v jq >/dev/null 2>&1; then
        printf '%s\n' "$CAPABILITIES" | jq -e '
            (.schema == "ltools-capabilities-v1") and
            (.platform == "windows") and
            any(.host_tools[]; .id == "docker-compose" and .installable == true) and
            any(.host_tools[]; .id == "kubectl" and .installable == true) and
            ([.host_tools[] | select((.id == "docker" or .id == "podman" or .id == "podman-compose" or .id == "helm" or .id == "kind" or .id == "minikube" or .id == "k3d" or .id == "k9s") and .installable == true)] | length == 0) and
            ([.host_tools[] | select(.command == "pacman" or .command == "apt-get" or .command == "systemctl" or .command == "wine")] | length == 0) and
            all(.host_tools[]; (.version | type == "string"))
        ' >/dev/null || die 'el catálogo Windows bajo Wine mezcla plataformas o instaladores alternativos'
        ok 'catálogo Windows nativo, versiones y primarios únicos'
        for expanded_id in \
            chkdsk.exe fsutil.exe manage-bde.exe sfc.exe pnputil.exe \
            schtasks.exe netsh.exe icacls.exe vssadmin.exe wbadmin.exe \
            dotnet.exe; do
            printf '%s\n' "$CAPABILITIES" | jq -e --arg id "$expanded_id" \
                'any(.host_tools[]; .id == $id)' >/dev/null ||
                die "falta la herramienta Windows ampliada en capacidades: $expanded_id"
        done
        ok 'catálogo Windows ampliado: disco, reparación, red, seguridad y desarrollo'
    fi
    TERMINAL_JSON="$(run_windows_timeout capabilities --format terminal-json 2>>"$LOG_PATH")" ||
        die 'capabilities --format terminal-json falló'
    printf '%s\n' "$TERMINAL_JSON" | grep -Fq 'WinSlim Terminal' ||
        die 'el descriptor terminal Windows no declara WinSlim Terminal'
    printf '%s\n' "$TERMINAL_JSON" | grep -Fq '"executable":"ltools.exe"' ||
        die 'el descriptor terminal Windows no usa ltools.exe'
    ok 'descriptor declarativo WinSlim Terminal'
    ok 'contrato JSON Windows'
    run_case 'defaults' defaults
    windows_network_guide="$(run_windows_timeout guide network 2>>"$LOG_PATH")" ||
        die 'guide network Windows bajo Wine falló'
    grep -Fq 'PERFIL WINDOWS' <<<"$windows_network_guide" ||
        die 'guide network Windows no identificó el perfil Windows'
    grep -Fq 'adaptadores' <<<"$windows_network_guide" ||
        die 'guide network Windows no documentó sus objetivos nativos'
    ! grep -Fq 'systemctl' <<<"$windows_network_guide" ||
        die 'guide network Windows mezcló conceptos Linux'
    windows_boot_guide="$(run_windows_timeout guide boot 2>>"$LOG_PATH")" ||
        die 'guide boot Windows bajo Wine falló'
    grep -Fq 'BCD/UEFI' <<<"$windows_boot_guide" ||
        die 'guide boot Windows no documentó BCD/UEFI'
    grep -Fq 'EFI/GRUB de Linux no aplican' <<<"$windows_boot_guide" ||
        die 'guide boot Windows no marcó las opciones Linux como no aplicables'
    windows_wine_guide="$(run_windows_timeout guide wine 2>>"$LOG_PATH")" ||
        die 'guide wine Windows bajo Wine falló'
    grep -Fq 'NO APLICA' <<<"$windows_wine_guide" ||
        die 'guide wine Windows no marcó la incompatibilidad nativa'
    windows_winslim_guide="$(run_windows_timeout guide gui winslim 2>>"$LOG_PATH")" ||
        die 'guide gui winslim Windows bajo Wine falló'
    windows_gui_index="$(run_windows_timeout guide gui all 2>>"$LOG_PATH")" ||
        die 'guide gui all Windows bajo Wine falló'
    if grep -Eq '^WinSlim:[[:space:]]*$' <<<"$windows_gui_index"; then
        grep -Fq 'Estado de WSCore y NSudo' <<<"$windows_winslim_guide" ||
            die 'la guía WinSlim no enumera la pantalla activa'
    else
        grep -Fq 'no tiene disponible la pantalla WinSlim/NSudo' <<<"$windows_winslim_guide" ||
            die 'la guía WinSlim no explica por qué el menú condicional no aparece'
    fi
    windows_winslim_status="$(run_windows_timeout winslim status 2>>"$LOG_PATH")" ||
        die 'winslim status Windows bajo Wine falló'
    grep -Fq 'NSudo' <<<"$windows_winslim_status" || die 'winslim status no informa la detección de NSudo'
    windows_nsudo_guide="$(run_windows_timeout winslim guide 2>>"$LOG_PATH")" ||
        die 'winslim guide Windows bajo Wine falló'
    for marker in 'TrustedInstaller' \
        '--identity current|elevated|system|trustedinstaller|process|drop-rights' \
        '--integrity system|high|medium|low' \
        '--window show|hide|maximize|minimize' \
        --all-privileges --console --wait --dry-run --yes; do
        grep -Fq -- "$marker" <<<"$windows_nsudo_guide" || die "la guía NSudo omite $marker"
    done
    windows_nsudo_plan="$(run_windows_timeout --dry-run winslim launch --identity system --program cmd.exe --arg /c --arg ver --all-privileges --integrity high --window maximize --wait --console 2>>"$LOG_PATH")" ||
        die 'el plan NSudo simulado falló en Windows bajo Wine'
    for marker in '-U:S -P:E -M:H -Wait -UseCurrentConsole' 'Modo de ventana: maximize' 'Programa: cmd.exe (2 argumento(s)' 'Simulación: no se inició'; do
        grep -Fq -- "$marker" <<<"$windows_nsudo_plan" || die "el plan NSudo no conserva $marker"
    done
    for gui_marker in 'Panel principal:' 'Resumen de espacio y montajes' \
        'Discos y particiones' 'Usuarios, grupos y sesiones' 'Aplicar ajustes'; do
        grep -Fq "$gui_marker" <<<"$windows_gui_index" ||
            die "el índice GUI Windows no enumera la opción visible: $gui_marker"
    done
    if grep -Eq '^WinSlim:[[:space:]]*$' <<<"$windows_gui_index"; then
        grep -Eq '^WinSlim:[[:space:]]*$' <<<"$windows_gui_index" ||
            die 'el índice GUI Windows no enumera la categoría WinSlim visible'
    else
        ! grep -Eq '^WinSlim:[[:space:]]*$' <<<"$windows_gui_index" ||
            die 'el índice GUI Windows anuncia WinSlim sin WSCore ni un lanzador NSudo'
    fi
    ! grep -Fq 'Crear prefijo' <<<"$windows_gui_index" ||
        die 'el índice GUI Windows anuncia gestión de prefijos Wine/Proton'
    windows_native_gui_guide="$(run_windows_timeout guide gui native 2>>"$LOG_PATH")" ||
        die 'guide gui native Windows bajo Wine falló'
    grep -Fq 'Abrir gestor nativo de particiones' <<<"$windows_native_gui_guide" ||
        die 'la guía GUI nativa Windows no coincide con los botones reales'
    ok 'guías Windows usan opciones nativas y rechazan supuestos Linux'
    native_tools_output="$(run_windows_timeout --elevate native tools status 2>>"$LOG_PATH")" ||
        die 'native tools Windows bajo Wine falló'
    printf '%s\n' "$native_tools_output" | tee -a "$LOG_PATH" >/dev/null
    if grep -Fq 'Elevación: native' <<<"$native_tools_output"; then
        die 'el inventario de herramientas, que es de solo lectura, intentó elevarse'
    fi
    for tool_name in ssh scp sftp adb docker kubectl chkdsk.exe fsutil.exe netsh.exe; do
        if ! grep -Fqi "$tool_name" <<<"$native_tools_output"; then
            printf 'Salida incompleta de «native tools» (se esperaba %s):\n%s\n' \
                "$tool_name" "$native_tools_output" >&2
            die "native tools Windows no mostró $tool_name"
        fi
    done
    ok 'inventario Windows de SSH, ADB, Docker y Kubernetes'
    disk_guide_output="$(run_windows_timeout storage guide 2>>"$LOG_PATH")" ||
        die 'storage guide Windows bajo Wine falló'
    printf '%s\n' "$disk_guide_output" | tee -a "$LOG_PATH" >/dev/null
    for guide_marker in 'list disk' 'select disk' 'detail disk' 'clean all' 'C:'; do
        grep -Fqi "$guide_marker" <<<"$disk_guide_output" ||
            die "storage guide Windows no mostró $guide_marker"
    done
    ok 'guía DiskPart Windows con protección de objetivos'
    registry_dry_run_output="/tmp/ltools-registry-dry-run-$$.reg"
    registry_output="$(run_windows_timeout registry export --key 'HKCU\\Software' --out "$registry_dry_run_output" --dry-run 2>>"$LOG_PATH")" ||
        die 'registry export --dry-run Windows bajo Wine falló'
    printf '%s\n' "$registry_output" | tee -a "$LOG_PATH" >/dev/null
    grep -Fq 'no se modificaría el Registro' <<<"$registry_output" ||
        die 'registry export --dry-run no confirmó que no escribiría'
    [[ ! -e "$registry_dry_run_output" ]] ||
        die 'registry export --dry-run creó un archivo inesperadamente'
    ok 'exportación del Registro Windows respeta dry-run y no escribe archivos'
    menu_output="$(printf 'q\n' | run_windows_timeout menu 2>&1)" || {
        die 'el menú Windows no se abrió correctamente'
    }
    printf '%s\n' "$menu_output" | tee -a "$LOG_PATH" >/dev/null
    if ! grep -Fq 'Elige una opción' <<<"$menu_output"; then
        die 'el menú Windows no se abrió correctamente'
    fi
    ok 'menú Windows abre y sale con q'

    if command -v xvfb-run >/dev/null 2>&1 && command -v xdotool >/dev/null 2>&1 &&
        command -v import >/dev/null 2>&1 &&
        timeout 10 xvfb-run -a -s '-screen 0 1280x900x24' true >/dev/null 2>&1; then
        gui_output="$(mktemp "${TMPDIR:-/tmp}/ltools-windows-gui.XXXXXX.log")"
        gui_marker_name="ltools-gui-smoke-$$.marker"
        gui_marker="$PREFIX/drive_c/windows/temp/$gui_marker_name"
        gui_marker_windows="C:\\windows\\temp\\$gui_marker_name"
        gui_capture_dir="${LTOOLS_GUI_CAPTURE_DIR:-$ROOT_DIR/dist/captures}"
        menu_capture_dir="$gui_capture_dir/windows-menu-pages"
        native_gui_capture="$gui_capture_dir/windows-native-tools-wine.png"
        settings_gui_capture="$gui_capture_dir/windows-settings-wine.png"
        gui_pages=(0 1 2 3 4 5)
        if [[ -d "$PREFIX/drive_c/WSCore" ]]; then
            gui_pages+=(7)
        fi
        gui_pages+=(6)
        mkdir -p -- "$menu_capture_dir"
        mkdir -p -- "$(dirname -- "$gui_marker")"
        rm -f -- "$gui_marker" "$native_gui_capture" "$settings_gui_capture"
        for page in "${gui_pages[@]}"; do
            rm -f -- "$menu_capture_dir/windows-menu-page-$page.png"
        done
        set +e
        timeout 60 xvfb-run -a -s '-screen 0 1280x900x24' env \
            WINEPREFIX="$PREFIX" WINEDEBUG=-all \
            LTOOLS_GUI_REQUIRED=1 LTOOLS_DISABLE_GUI=0 LTOOLS_TERMINAL=auto \
            LTOOLS_LANG=es \
            LTOOLS_GUI_SMOKE_NAV_MARKER="$gui_marker_windows" \
            LTOOLS_GUI_SMOKE_HOLD_MS=45000 bash -c '
                set -Eeuo pipefail
                runner="$1"
                mode="$2"
                executable="$3"
                capture_dir="$4"
                native_capture="$5"
                settings_capture="$6"
                marker="$7"
                IFS=, read -r -a gui_pages <<<"$8"
                if [[ "$mode" == proton ]]; then
                    "$runner" run "$executable" &
                else
                    "$runner" "$executable" &
                fi
                gui_pid=$!
                window_id=""
                for _ in {1..60}; do
                    # El título Win32 es «WinSlim-Tools <versión>»; buscar
                    # «LTools» nunca encuentra esta ventana bajo Wine.
                    window_id="$(xdotool search --onlyvisible --name "WinSlim-Tools" 2>/dev/null | head -n1 || true)"
                    [[ -n "$window_id" ]] && break
                    sleep 0.1
                done
                [[ -n "$window_id" ]] || { echo "No apareció la ventana Win32 de LTools" >&2; exit 1; }
                xdotool windowraise "$window_id"
                for page in "${gui_pages[@]}"; do
                    # La GUI coloca las categorías en una columna con 50 px
                    # entre centros; cada botón se abre con un clic real.
                    button_y=$((87 + page * 50))
                    xdotool mousemove --sync --window "$window_id" 520 "$button_y" click 1
                    opened=0
                    for _ in {1..50}; do
                        if [[ -s "$marker" ]] && grep -Fq "navigation-page=$page" "$marker"; then
                            opened=1
                            break
                        fi
                        sleep 0.1
                    done
                    [[ "$opened" -eq 1 ]] || { echo "No abrió la categoría Win32 $page" >&2; exit 1; }
                    printf 'GUI_PAGE_OK=%s\n' "$page"
                    capture="$capture_dir/windows-menu-pages/windows-menu-page-$page.png"
                    [[ "$page" -ne 1 ]] || capture="$native_capture"
                    [[ "$page" -ne 6 ]] || capture="$settings_capture"
                    # Capturar la superficie realmente visible: ImageMagick
                    # devuelve negro al leer directamente el HWND de Wine.
                    sleep 0.6
                    import -window root "$capture"
                    if [[ "$page" -ne 6 ]]; then
                        # El botón Volver está centrado junto al borde inferior.
                        xdotool mousemove --sync --window "$window_id" 520 643 click 1
                        sleep 0.2
                    fi
                done
                xdotool windowclose "$window_id"
                wait "$gui_pid"
            ' _ "$RUNNER" "$RUNNER_MODE" "$WINEXE" "$gui_capture_dir" \
            "$native_gui_capture" "$settings_gui_capture" "$gui_marker" \
            "$(IFS=,; printf '%s' "${gui_pages[*]}")" \
            >"$gui_output" 2>&1
        gui_status=$?
        set -e
        if (( gui_status != 0 )); then
            printf 'Salida de la GUI Windows bajo Wine (código %s):\n' "$gui_status" >&2
            sed -n '1,160p' "$gui_output" >&2 || true
            [[ -f "$gui_marker" ]] && cat "$gui_marker" >&2 || true
            die 'la GUI Windows bajo Wine no abrió todas las categorías con clics reales'
        fi
        for page in "${gui_pages[@]}"; do
            if [[ "$page" -eq 1 ]]; then
                page_capture="$native_gui_capture"
            elif [[ "$page" -eq 6 ]]; then
                page_capture="$settings_gui_capture"
            else
                page_capture="$menu_capture_dir/windows-menu-page-$page.png"
            fi
            [[ -s "$page_capture" ]] || die "la categoría GUI Windows $page no produjo captura"
            if command -v identify >/dev/null 2>&1; then
                page_dimensions="$(identify -format '%w %h' "$page_capture" 2>/dev/null || true)"
                read -r page_width page_height <<<"$page_dimensions"
                [[ "${page_width:-0}" -ge 800 && "${page_height:-0}" -ge 600 ]] ||
                    die "captura de GUI Windows $page incompleta: ${page_dimensions:-sin dimensiones}"
                page_colors="$(identify -format '%k' "$page_capture" 2>/dev/null || echo 0)"
                # El Win32 owner-drawn usa una paleta plana; cuatro colores
                # distinguen texto, controles, borde y fondo.
                [[ "$page_colors" -ge 4 ]] || die "captura de GUI Windows $page sin contenido visual suficiente"
            fi
        done
        for page in "${gui_pages[@]}"; do
            grep -Fq "GUI_PAGE_OK=$page" "$gui_output" ||
                die "el E2E no confirmó la categoría GUI Windows $page"
        done
        cat "$gui_output" | tee -a "$LOG_PATH"
        rm -f -- "$gui_marker"
        rm -f -- "$gui_output"
        ok "GUI Windows bajo Wine abre con clic real las categorías ${gui_pages[*]} y verifica todas las capturas"
    else
        if [[ "$REQUIRE_GUI" -eq 1 ]]; then
            if command -v xvfb-run >/dev/null 2>&1 && command -v xdotool >/dev/null 2>&1 &&
                command -v import >/dev/null 2>&1; then
                die 'GUI Windows bajo Wine requerida, pero Xvfb no pudo iniciar un display aislado'
            fi
            die 'GUI Windows bajo Wine requerida, pero falta xvfb-run, xdotool o ImageMagick import'
        fi
        warn 'GUI Windows bajo Wine omitida: faltan herramientas gráficas o Xvfb no pudo iniciar un display aislado'
    fi
fi
if [[ "$REQUIRE_GUI" -eq 1 && "$RUN_TESTS" -eq 0 ]]; then
    die 'GUI Windows bajo Wine requerida, pero las pruebas Wine están desactivadas'
fi

package_windows_artifact

printf '\nWindows bajo Wine/Proton completado correctamente.\n' | tee -a "$LOG_PATH"
printf 'Log: %s\n' "$LOG_PATH"
