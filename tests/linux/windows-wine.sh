#!/usr/bin/env bash
# Build y smoke del ejecutable Windows usando un runner Wine/Proton.
#
# Es una herramienta de validación del host Linux, no el backend de LTools.
# Nunca usa el prefijo Wine del usuario salvo que se pase explícitamente
# --prefix. Por defecto crea un prefijo temporal.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
TARGET="${LTOOLS_WINDOWS_TARGET:-x86_64-pc-windows-gnu}"
RUNNER="${LTOOLS_WINE_RUNNER:-}"
PREFIX=""
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
        --keep-prefix) KEEP_PREFIX=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ "$TARGET" =~ ^[A-Za-z0-9_\.-]+$ ]] || die "target Rust inválido: $TARGET"

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
    if [[ "$KEEP_PREFIX" -eq 0 && -n "$PREFIX" && -d "$PREFIX" ]]; then
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

if [[ -z "$PREFIX" ]]; then
    PREFIX="$(mktemp -d "${TMPDIR:-/tmp}/ltools-windows-wine.XXXXXX")"
    if [[ "$KEEP_PREFIX" -eq 0 ]]; then
        trap cleanup_prefix EXIT
    else
        printf 'Prefijo conservado en: %s\n' "$PREFIX"
    fi
else
    PREFIX="$(realpath -m -- "$PREFIX")"
    mkdir -p -- "$PREFIX"
    warn "se usará el prefijo explícito: $PREFIX"
fi

if [[ -z "$LOG_PATH" ]]; then
    LOG_PATH="$PREFIX/windows-wine.log"
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

WINEXE="$ROOT_DIR/rust/target/$TARGET/release/ltools.exe"
[[ -f "$WINEXE" ]] || die "no existe el ejecutable Windows: $WINEXE"

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
    mkdir -p -- "$ARTIFACT_DIR"
    local package_arch="${TARGET%%-*}"
    local artifact="$ARTIFACT_DIR/ltools-$VERSION-windows-$package_arch.exe"
    local metadata="$ARTIFACT_DIR/ltools-$VERSION-windows-$package_arch-wine.json"
    cp -a -- "$WINEXE" "$artifact"
    cat > "$metadata" <<EOF
{
  "application": "LTools",
  "version": "$VERSION",
  "platform": "windows",
  "architecture": "$package_arch",
  "artifact": "$(basename -- "$artifact")",
  "validation": "wine-proton",
  "runner": "$(basename -- "$RUNNER")",
  "target": "$TARGET"
}
EOF
    ok "artefacto Windows bajo Wine copiado: $artifact"
}

run_windows_timeout() {
    if [[ "$RUNNER_MODE" == "proton" ]]; then
        timeout 30 "$RUNNER" run "$WINEXE" "$@"
    else
        timeout 30 "$RUNNER" "$WINEXE" "$@"
    fi
}

run_probe() {
    if [[ "$RUNNER_MODE" == "proton" ]]; then
        timeout 30 "$RUNNER" run cmd /c echo LTOOLS_WINE_OK
    else
        timeout 30 "$RUNNER" cmd /c echo LTOOLS_WINE_OK
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

    if ! run_probe 2>&1 | tee -a "$LOG_PATH" | grep -Fq 'LTOOLS_WINE_OK'; then
        die "el runner no pudo iniciar una consola Windows; revisa $LOG_PATH"
    fi
    ok 'runner Windows inicia una consola aislada'

    run_case() {
        local name="$1"
        shift
        printf '\n--- %s ---\n' "$name" | tee -a "$LOG_PATH"
        run_windows_timeout "$@" 2>&1 | tee -a "$LOG_PATH"
    }

    run_case 'version' --version
    run_case 'help' --help
    CAPABILITIES="$(run_windows_timeout capabilities --format json 2>>"$LOG_PATH")" ||
        die 'capabilities --format json falló'
    printf '%s\n' "$CAPABILITIES" | tee -a "$LOG_PATH" | grep -Fq '"platform": "windows"' ||
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
    if ! printf 'q\n' | run_windows_timeout menu 2>&1 | tee -a "$LOG_PATH" | grep -Fq 'Elige una opción'; then
        die 'el menú Windows no se abrió correctamente'
    fi
    ok 'menú Windows abre y sale con q'
fi

package_windows_artifact

printf '\nWindows bajo Wine/Proton completado correctamente.\n' | tee -a "$LOG_PATH"
printf 'Log: %s\n' "$LOG_PATH"
