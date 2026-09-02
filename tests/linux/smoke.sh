#!/usr/bin/env bash
# Smoke tests seguros: no recorren los discos reales ni modifican la cuenta.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/rust/Cargo.toml" | head -n1)"
APPIMAGE_PATH=""
RUNNER_PATH=""
LOG_PATH=""
KEEP_TEMP=0

die() { printf 'SMOKE ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }
skip() { printf '  SKIP  %s\n' "$1"; }
usage() { printf 'Uso: %s [--binary RUTA] [--appimage RUTA] [--runner RUTA] [--log RUTA] [--keep-temp]\n' "$0"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        --appimage) (($# >= 2)) || die '--appimage necesita una ruta'; APPIMAGE_PATH="$2"; shift ;;
        --runner) (($# >= 2)) || die '--runner necesita una ruta'; RUNNER_PATH="$2"; shift ;;
        --log) (($# >= 2)) || die '--log necesita una ruta'; LOG_PATH="$2"; shift ;;
        --keep-temp) KEEP_TEMP=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ -x "$BIN" ]] || die "no existe el binario ejecutable: $BIN"
[[ -f "$ROOT_DIR/ltools-cli.sh" ]] || die 'no existe el lanzador CLI Linux'
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cachyos-smoke.XXXXXX")"
if [[ "$KEEP_TEMP" -eq 0 ]]; then
    trap 'rm -rf -- "$TMP_DIR"' EXIT
else
    printf 'Temporales conservados en: %s\n' "$TMP_DIR"
fi

printf 'Smoke tests de LTools\n'
while IFS= read -r -d '' file; do
    bash -n "$file"
done < <(find "$ROOT_DIR" -maxdepth 4 -type f -name '*.sh' -print0)
ok 'sintaxis de todos los scripts Bash'

"$BIN" --version >/dev/null
ok 'backend Rust responde a --version'
HELP_OUTPUT="$($BIN --help)"
grep -Fq 'doctor --install TOOL' <<<"$HELP_OUTPUT" || die 'la ayuda no documenta la instalación explícita'
ok 'backend Rust responde a --help'
CLI_HELP_OUTPUT="$(LTOOLS_CLI=1 "$BIN")"
grep -Fq 'Uso: ltools' <<<"$CLI_HELP_OUTPUT" || die 'el perfil CLI sin argumentos no mostró la ayuda'
ok 'perfil CLI sin argumentos muestra ayuda sin abrir menú'
CLI_WRAPPER_OUTPUT="$("$ROOT_DIR/ltools-cli.sh")"
grep -Fq 'Uso: ltools' <<<"$CLI_WRAPPER_OUTPUT" || die 'ltools-cli.sh sin argumentos no mostró la ayuda'
ok 'lanzador CLI Linux conserva el modo sin argumentos'
CAPABILITIES_JSON="$($BIN capabilities --format json)"
grep -Fq '"schema": "ltools-capabilities-v1"' <<<"$CAPABILITIES_JSON" ||
    die 'el contrato JSON de capacidades no se pudo generar'
grep -Fq 'lterminal-startup-v1' <<<"$CAPABILITIES_JSON" ||
    die 'el contrato JSON no declara integración de terminal'
if command -v jq >/dev/null 2>&1; then
    jq -e '(.host_tools | length >= 10) and any(.host_tools[]; .category == "audit") and any(.host_tools[]; .category == "system") and any(.host_tools[]; .installable == true) and any(.host_tools[]; .id == "docker-compose" and .installable == true) and any(.host_tools[]; .id == "kubectl" and .installable == true) and any(.host_tools[]; .id == "lsblk" and .installable == true) and ([.host_tools[] | select(.category == "games" or .category == "virtualization" or .category == "development" or .command == "steam" or .command == "git")] | length == 0)' \
        <<<"$CAPABILITIES_JSON" >/dev/null \
        || die 'el catálogo JSON de herramientas del anfitrión está incompleto'
    jq -e 'all(.host_tools[]; (.version | type == "string")) and ([.host_tools[] | select((.id == "parted" or .id == "gparted" or .id == "fdisk" or .id == "podman" or .id == "podman-compose" or .id == "helm" or .id == "kind" or .id == "minikube" or .id == "k3d" or .id == "k9s") and .installable == true)] | length == 0)' \
        <<<"$CAPABILITIES_JSON" >/dev/null \
        || die 'el catálogo JSON no respeta versiones e instalador único por capacidad'
fi
ok 'contrato JSON de capacidades e integración; catálogo nativo con instaladores únicos'

INSTALL_STUB_DIR="$TMP_DIR/install-stub"
INSTALL_OUTPUT="$TMP_DIR/install-output.log"
mkdir -p "$INSTALL_STUB_DIR" "$TMP_DIR/home-install" "$TMP_DIR/state-install"
cat > "$INSTALL_STUB_DIR/pacman" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$INSTALL_STUB_DIR/pacman"
set +e
printf 'n\n' | HOME="$TMP_DIR/home-install" XDG_STATE_HOME="$TMP_DIR/state-install" PATH="$INSTALL_STUB_DIR" "$BIN" doctor --install rsync >"$INSTALL_OUTPUT" 2>&1
INSTALL_STATUS=$?
set -e
[[ "$INSTALL_STATUS" -ne 0 ]] || die 'doctor --install aceptó una instalación cancelada'
grep -Fq 'pacman -S --needed rsync' "$INSTALL_OUTPUT" || die 'doctor --install no mostró el comando concreto'
grep -Fq 'Instalación cancelada; no se modifica el sistema.' "$INSTALL_OUTPUT" || die 'doctor --install no confirmó la cancelación segura'
ok 'instalación explícita muestra comando y respeta la cancelación'
TERMINAL_JSON="$($BIN capabilities --format terminal-json)"
grep -Fq '"schema": "ltools-terminal-integration-v1"' <<<"$TERMINAL_JSON" ||
    die 'el descriptor específico de terminal no se pudo generar'
grep -Fq '"required_terminal_capability": "lterminal-startup-v1"' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no declara la capacidad requerida'
grep -Fq '"standalone_releases_require_it": false' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no declara su carácter opcional'
grep -Fq '"exclusive_host_family": "lterminal"' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no limita su integración a LTerminal'
grep -Fq 'WinSlim Terminal' <<<"$TERMINAL_JSON" ||
    die 'el descriptor de terminal no declara WinSlim Terminal'
if command -v jq >/dev/null 2>&1; then
    jq -e '(.actions | length >= 15) and all(.actions[]; .id and .executable and (.args | type == "array") and .terminal == true and .shell == "none" and (.supports | index("dry-run") != null))' \
        <<<"$TERMINAL_JSON" >/dev/null || die 'las acciones declarativas no tienen el contrato esperado'
fi
ok 'descriptor JSON específico para terminales'
EN_HELP="$(LTOOLS_LANG=en "$BIN" --help)"
grep -Fq 'Usage: ltools' <<<"$EN_HELP" || die 'el idioma inglés no se aplicó al backend Rust'
DE_HELP="$("$BIN" --lang de --help)"
grep -Fq 'Verwendung:' <<<"$DE_HELP" || die 'la opción --lang no se aplicó al backend Rust'
declare -A LANGUAGE_MARKERS=(
    [es]='Uso:' [en]='Usage:' [de]='Verwendung:' [fr]='Utilisation'
    [pt]='Uso:' [it]='Uso:' [ca]='Ús:' [nl]='Gebruik:' [pl]='Użycie:'
)
for language in "${!LANGUAGE_MARKERS[@]}"; do
    translated_help="$(LTOOLS_LANG="$language" "$BIN" --help)"
    grep -Fq "${LANGUAGE_MARKERS[$language]}" <<<"$translated_help" ||
        die "el idioma Rust $language no se aplicó a la ayuda"
    if [[ "$language" != es ]] && grep -Fq 'Auditoría de discos, paquetes y aplicaciones' <<<"$translated_help"; then
        die "el idioma Rust $language está usando el fallback español en la ayuda"
    fi
done
ok 'todos los idiomas en el backend Rust'
DOCTOR_OUTPUT="$(HOME="$TMP_DIR/home" XDG_STATE_HOME="$TMP_DIR/state" "$BIN" doctor)"
grep -Fq 'LTools host diagnostics' <<<"$DOCTOR_OUTPUT" || die 'doctor no funciona como operación de solo lectura'
grep -Fq '[audit]' <<<"$DOCTOR_OUTPUT" || die 'doctor no agrupa herramientas de auditoría'
grep -Fq '[system]' <<<"$DOCTOR_OUTPUT" || die 'doctor no agrupa herramientas de sistema'
grep -Fq '[prefix]' <<<"$DOCTOR_OUTPUT" || die 'doctor no agrupa herramientas de prefijos'
if grep -Fq '[packages]' <<<"$DOCTOR_OUTPUT"; then
    die 'doctor volvió a mezclar gestores de paquetes como dependencias'
fi
ok 'doctor Rust sin crear planes ni modificar el estado'

mkdir -p "$TMP_DIR/root/demo-prefix/drive_c"
printf 'synthetic-prefix\n' > "$TMP_DIR/root/demo-prefix/system.reg"
printf 'demo\n' > "$TMP_DIR/root/demo-prefix/drive_c/demo.exe"
PLAN="$TMP_DIR/list-plan.tsv"
LIST_OUTPUT="$("$BIN" --dry-run --plan "$PLAN" prefix list --root "$TMP_DIR/root")"
[[ -s "$PLAN" ]] || die 'no se creó el plan del listado'
grep -Fq 'demo-prefix' <<<"$LIST_OUTPUT" || die 'el listado no detectó el prefijo sintético'
ok 'listado aislado de un prefijo sintético'
STORAGE_OUTPUT="$("$BIN" storage tools)"
grep -Fq 'Herramientas de almacenamiento' <<<"$STORAGE_OUTPUT" || die 'storage no responde'
REGISTRY_OUTPUT="$("$BIN" registry status)"
grep -Fq 'Registros y configuración Linux' <<<"$REGISTRY_OUTPUT" || die 'registry Linux no responde'
ok 'módulos Linux de almacenamiento y configuración'

RELEASE_FIXTURE_DIR="$TMP_DIR/release-assets"
mkdir -p "$RELEASE_FIXTURE_DIR"
printf 'synthetic-appimage\n' > "$RELEASE_FIXTURE_DIR/ltools-$VERSION-linux-x86_64.AppImage"
RELEASE_MANIFEST="$TMP_DIR/ltools-release.json"
"$BIN" release-manifest \
    --output "$RELEASE_MANIFEST" \
    --repository Darkeiser003/Tools \
    --tag "v$VERSION" \
    --artifacts-dir "$RELEASE_FIXTURE_DIR" >/dev/null
grep -Fq '"schema": "ltools-release-v1"' "$RELEASE_MANIFEST" ||
    die 'el manifiesto de release no declara su esquema'
grep -Fq '"platform":"linux"' "$RELEASE_MANIFEST" ||
    die 'el manifiesto de release no detectó Linux'
grep -Eq '"sha256":"[a-f0-9]{64}"' "$RELEASE_MANIFEST" ||
    die 'el manifiesto de release no contiene un SHA-256 válido'
ok 'manifiesto GitHub de release con tamaño y SHA-256'

if [[ -n "$APPIMAGE_PATH" ]]; then
    [[ -x "$APPIMAGE_PATH" ]] || die "AppImage no ejecutable: $APPIMAGE_PATH"
    DIRECT_LOG="${LOG_PATH:-$TMP_DIR/appimage-direct.log}"
    mkdir -p "$(dirname -- "$DIRECT_LOG")"
    : > "$DIRECT_LOG"
    {
        printf 'LTools: prueba directa del AppImage\n'
        printf 'Fecha: %s\n' "$(date --iso-8601=seconds)"
        printf 'AppImage: %s\n' "$APPIMAGE_PATH"
        printf 'Permisos: %s\n\n' "$(stat -c '%A %a %U:%G' "$APPIMAGE_PATH" 2>/dev/null || printf 'desconocidos')"
        printf '$ %q --doctor\n' "$APPIMAGE_PATH"
    } >> "$DIRECT_LOG"
    set +e
    timeout 30 "$APPIMAGE_PATH" --doctor >> "$DIRECT_LOG" 2>&1
    DIRECT_STATUS=$?
    set -e
    if [[ "$DIRECT_STATUS" -ne 0 ]]; then
        printf 'Salida de la ejecución directa:\n' >&2
        sed -n '1,160p' "$DIRECT_LOG" >&2
        die "el AppImage no se pudo abrir directamente (código $DIRECT_STATUS); log: $DIRECT_LOG"
    fi
    ok "ejecución directa del AppImage; log: $DIRECT_LOG"

    CLI_APPIMAGE_PATH="${APPIMAGE_PATH%.AppImage}-cli.AppImage"
    if [[ -x "$CLI_APPIMAGE_PATH" ]]; then
        CLI_APPIMAGE_OUTPUT="$(env APPIMAGE_EXTRACT_AND_RUN=1 timeout 30 "$CLI_APPIMAGE_PATH" 2>&1)" ||
            die 'el AppImage CLI terminó con error sin argumentos'
        grep -Fq 'Uso: ltools' <<<"$CLI_APPIMAGE_OUTPUT" ||
            die 'el AppImage CLI sin argumentos no mostró la ayuda'
        ok 'AppImage CLI sin argumentos muestra ayuda'
    fi

    TERMINAL_STUB_DIR="$TMP_DIR/terminal-stub"
    TERMINAL_STUB_LOG="$TMP_DIR/terminal-stub.log"
    mkdir -p "$TERMINAL_STUB_DIR"
    LTERMINAL_STUB_DIR="$TMP_DIR/lterminal-stub"
    LTERMINAL_STUB_LOG="$TMP_DIR/lterminal-stub.log"
    mkdir -p "$LTERMINAL_STUB_DIR"
cat > "$LTERMINAL_STUB_DIR/lterminal" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == --ltools-capabilities ]]; then
    printf 'lterminal-startup-v1\n'
    exit 0
fi
printf '%s\n' "$*" >> "$LTOOLS_LTERMINAL_STUB_LOG"
sleep 2
EOF
    chmod +x "$LTERMINAL_STUB_DIR/lterminal"
cat > "$TERMINAL_STUB_DIR/x-terminal-emulator" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$LTOOLS_TERMINAL_STUB_LOG"
sleep 2
EOF
    chmod +x "$TERMINAL_STUB_DIR/x-terminal-emulator"
    LTERMINAL_OUTPUT_FILE="$TMP_DIR/lterminal-output.log"
    LTERMINAL_LAUNCH_LOG="$TMP_DIR/lterminal-launch.log"
    : > "$LTERMINAL_OUTPUT_FILE"
    env -u LTOOLS_NO_AUTO_TERMINAL -u LTOOLS_TERMINAL_LAUNCH \
        LTOOLS_TERMINAL=lterminal \
        HOME="$TMP_DIR/home-lterminal" XDG_STATE_HOME="$TMP_DIR/state-lterminal" \
        LTOOLS_LAUNCH_LOG="$LTERMINAL_LAUNCH_LOG" \
        LTOOLS_LTERMINAL="$LTERMINAL_STUB_DIR/lterminal" \
        PATH="$LTERMINAL_STUB_DIR:$TERMINAL_STUB_DIR:$PATH" \
        LTOOLS_LTERMINAL_STUB_LOG="$LTERMINAL_STUB_LOG" \
        LTOOLS_TERMINAL_STUB_LOG="$TERMINAL_STUB_LOG" \
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 10 "$APPIMAGE_PATH" >"$LTERMINAL_OUTPUT_FILE" 2>&1 &
    LTERMINAL_PID=$!
    LTERMINAL_READY=0
    for _ in {1..100}; do
        if grep -Fq 'Menú abierto en LTerminal.' "$LTERMINAL_OUTPUT_FILE" &&
            grep -Fq 'started pid=' "$LTERMINAL_LAUNCH_LOG" 2>/dev/null; then
            LTERMINAL_READY=1
            break
        fi
        if ! kill -0 "$LTERMINAL_PID" 2>/dev/null; then
            break
        fi
        sleep 0.1
    done
    if [[ "$LTERMINAL_READY" -ne 1 ]]; then
        set +e
        wait "$LTERMINAL_PID"
        LTERMINAL_STATUS=$?
        set -e
        sed -n '1,160p' "$LTERMINAL_OUTPUT_FILE" >&2
        [[ -f "$LTERMINAL_LAUNCH_LOG" ]] && sed -n '1,160p' "$LTERMINAL_LAUNCH_LOG" >&2
        die "el lanzador no seleccionó LTerminal con protocolo compatible (código $LTERMINAL_STATUS)"
    fi
    kill "$LTERMINAL_PID" 2>/dev/null || true
    wait "$LTERMINAL_PID" 2>/dev/null || true
    grep -Fq -- '--command' "$LTERMINAL_STUB_LOG" || die 'LTerminal no recibió --command'
    grep -Fq -- '-- menu' "$LTERMINAL_STUB_LOG" || die 'LTerminal no recibió el argumento menu'
    ok 'LTerminal compatible se prioriza y recibe el comando de LTools'

    INCOMPATIBLE_LTERMINAL_DIR="$TMP_DIR/incompatible-lterminal"
    INCOMPATIBLE_TERMINAL_DIR="$TMP_DIR/incompatible-terminal"
    INCOMPATIBLE_TERMINAL_LOG="$TMP_DIR/incompatible-terminal.log"
    mkdir -p "$INCOMPATIBLE_LTERMINAL_DIR" "$INCOMPATIBLE_TERMINAL_DIR"
    cat > "$INCOMPATIBLE_LTERMINAL_DIR/lterminal" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == --ltools-capabilities ]]; then
    printf '{"schema":"unsupported-terminal-capabilities"}\n'
    exit 0
fi
EOF
    chmod +x "$INCOMPATIBLE_LTERMINAL_DIR/lterminal"
cat > "$INCOMPATIBLE_TERMINAL_DIR/x-terminal-emulator" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$LTOOLS_INCOMPATIBLE_TERMINAL_LOG"
sleep 2
EOF
    chmod +x "$INCOMPATIBLE_TERMINAL_DIR/x-terminal-emulator"
    INCOMPATIBLE_OUTPUT="$TMP_DIR/incompatible-lterminal-output.log"
    # La extracción del runtime puede tardar varios segundos cuando se
    # ejecutan varios AppImage seguidos; el límite debe cubrir también su
    # limpieza, no solo la respuesta de AppRun.  Mantener los comentarios
    # fuera de la orden continuada es importante: un comentario después de
    # una barra invertida puede sacar el resto de las variables del entorno.
    set +e
    env -u LTOOLS_ALLOW_TERMINAL_FALLBACK -u LTOOLS_NO_AUTO_TERMINAL -u LTOOLS_TERMINAL_LAUNCH \
        LTOOLS_TERMINAL=lterminal \
        HOME="$TMP_DIR/home-incompatible" XDG_STATE_HOME="$TMP_DIR/state-incompatible" \
        LTOOLS_LAUNCH_LOG="$TMP_DIR/incompatible-lterminal-launch.log" \
        LTOOLS_LTERMINAL="$INCOMPATIBLE_LTERMINAL_DIR/lterminal" \
        LTOOLS_INCOMPATIBLE_TERMINAL_LOG="$INCOMPATIBLE_TERMINAL_LOG" \
        PATH="$INCOMPATIBLE_LTERMINAL_DIR:$INCOMPATIBLE_TERMINAL_DIR:$PATH" \
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 30 "$APPIMAGE_PATH" >"$INCOMPATIBLE_OUTPUT" 2>&1
    INCOMPATIBLE_STATUS=$?
    set -e
    [[ "$INCOMPATIBLE_STATUS" -eq 3 ]] || die "LTerminal incompatible terminó con código inesperado: $INCOMPATIBLE_STATUS"
    grep -Fq 'Se solicitó LTerminal, pero no se encontró una instalación compatible' "$INCOMPATIBLE_OUTPUT" ||
        die 'LTools no explicó que la integración explícita de LTerminal no está disponible'
    grep -Fq 'lterminal-startup-v1' "$INCOMPATIBLE_OUTPUT" ||
        die 'LTools no indicó el protocolo requerido para LTerminal'
    [[ ! -s "$INCOMPATIBLE_TERMINAL_LOG" ]] ||
        die 'se abrió una terminal alternativa sin autorización explícita'
    ok 'LTerminal incompatible bloquea el fallback silencioso y deja diagnóstico'

    FALLBACK_OUTPUT="$TMP_DIR/explicit-fallback-output.log"
    set +e
    env -u LTOOLS_NO_AUTO_TERMINAL -u LTOOLS_TERMINAL_LAUNCH \
        LTOOLS_ALLOW_TERMINAL_FALLBACK=1 \
        LTOOLS_TERMINAL=x-terminal-emulator \
        HOME="$TMP_DIR/home-fallback" XDG_STATE_HOME="$TMP_DIR/state-fallback" \
        LTOOLS_LAUNCH_LOG="$TMP_DIR/fallback-launch.log" \
        LTOOLS_LTERMINAL="$INCOMPATIBLE_LTERMINAL_DIR/lterminal" \
        LTOOLS_INCOMPATIBLE_TERMINAL_LOG="$INCOMPATIBLE_TERMINAL_LOG" \
        PATH="$INCOMPATIBLE_LTERMINAL_DIR:$INCOMPATIBLE_TERMINAL_DIR:$PATH" \
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 10 "$APPIMAGE_PATH" >"$FALLBACK_OUTPUT" 2>&1
    FALLBACK_STATUS=$?
    set -e
    [[ "$FALLBACK_STATUS" -eq 0 ]] || die "fallback explícito terminó con código inesperado: $FALLBACK_STATUS"
    grep -Fq 'Menú autónomo abierto en x-terminal-emulator usando' "$FALLBACK_OUTPUT" ||
        die 'el fallback explícito no abrió la terminal seleccionada'
    grep -Fq 'menu' "$INCOMPATIBLE_TERMINAL_LOG" ||
        die 'el fallback explícito no recibió el menú'
    ok 'fallback de terminal disponible solo con autorización explícita'

    AUTO_LAUNCH_LOG="$TMP_DIR/auto-launch.log"
    AUTO_TERMINAL_OUTPUT_FILE="$TMP_DIR/auto-terminal-output.log"
    : > "$AUTO_TERMINAL_OUTPUT_FILE"
    # Keep this check independent from FUSE mount cleanup.  The AppImage is
    # still exercised, but extraction mode prevents a terminal child from
    # keeping the runtime's captured pipe open on some desktop environments.
    env -u LTOOLS_NO_AUTO_TERMINAL -u LTOOLS_TERMINAL_LAUNCH \
        HOME="$TMP_DIR/home" XDG_STATE_HOME="$TMP_DIR/state" LTOOLS_LAUNCH_LOG="$AUTO_LAUNCH_LOG" \
        LTOOLS_LTERMINAL="$INCOMPATIBLE_LTERMINAL_DIR/lterminal" LTOOLS_TERMINAL=auto \
        PATH="$TERMINAL_STUB_DIR:$PATH" LTOOLS_TERMINAL_STUB_LOG="$TERMINAL_STUB_LOG" \
        APPIMAGE_EXTRACT_AND_RUN=1 timeout 10 "$APPIMAGE_PATH" >"$AUTO_TERMINAL_OUTPUT_FILE" 2>&1 &
    AUTO_LAUNCH_PID=$!
    AUTO_LAUNCH_READY=0
    for _ in {1..100}; do
        if grep -Fq 'Menú autónomo abierto en x-terminal-emulator usando' "$AUTO_TERMINAL_OUTPUT_FILE" &&
            grep -Fq 'started pid=' "$AUTO_LAUNCH_LOG" 2>/dev/null; then
            AUTO_LAUNCH_READY=1
            break
        fi
        if ! kill -0 "$AUTO_LAUNCH_PID" 2>/dev/null; then
            break
        fi
        sleep 0.1
    done
    if [[ "$AUTO_LAUNCH_READY" -ne 1 ]]; then
        set +e
        wait "$AUTO_LAUNCH_PID"
        AUTO_LAUNCH_STATUS=$?
        set -e
        printf 'Salida del lanzador sin TTY (código %s):\n' "$AUTO_LAUNCH_STATUS" >&2
        sed -n '1,160p' "$AUTO_TERMINAL_OUTPUT_FILE" >&2
        [[ -f "$AUTO_LAUNCH_LOG" ]] && sed -n '1,160p' "$AUTO_LAUNCH_LOG" >&2
        die 'el AppImage no pudo solicitar una terminal gráfica sin TTY'
    fi
    # Evidence is already recorded; do not make the build wait for a desktop
    # terminal or a wrapper that intentionally remains open.
    kill "$AUTO_LAUNCH_PID" 2>/dev/null || true
    wait "$AUTO_LAUNCH_PID" 2>/dev/null || true
    AUTO_TERMINAL_OUTPUT="$(<"$AUTO_TERMINAL_OUTPUT_FILE")"
    grep -Fq 'Menú autónomo abierto en x-terminal-emulator usando' <<<"$AUTO_TERMINAL_OUTPUT" ||
        die 'el AppImage no intentó abrir una terminal al iniciarse sin argumentos'
    grep -Fq 'menu' "$TERMINAL_STUB_LOG" ||
        die 'el lanzador no pasó el comando menu a la terminal gráfica'
    grep -Fq 'Registro del lanzador:' <<<"$AUTO_TERMINAL_OUTPUT" ||
        die 'el lanzador no informó el registro de apertura gráfica'
    grep -Fq 'started pid=' "$AUTO_LAUNCH_LOG" ||
        die 'el registro del lanzador no confirmó el proceso de terminal'
    ok 'apertura sin argumentos redirige el menú a una terminal gráfica'

    NOARGS_OUTPUT="$(printf 'q\n' | timeout 30 env LTOOLS_NO_AUTO_TERMINAL=1 "$APPIMAGE_PATH" 2>&1)"
    grep -Fq 'Elige una opción' <<<"$NOARGS_OUTPUT" || die 'el menú interactivo no se mostró al iniciar sin argumentos'
    ok 'menú interactivo del AppImage'
    APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --version >/dev/null
    ok 'AppImage responde usando extracción temporal'
    APPIMAGE_CA_HELP="$(APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --lang ca --help)" ||
        die 'la ayuda Rust del AppImage no pudo ejecutarse con --lang ca'
    grep -Fq 'Ús:' <<<"$APPIMAGE_CA_HELP" ||
        die 'la ayuda Rust del AppImage no respeta --lang ca'
    APPIMAGE_PL_HELP="$(APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --lang pl --help)" ||
        die 'la ayuda Rust del AppImage no pudo ejecutarse con --lang pl'
    grep -Fq 'Użycie:' <<<"$APPIMAGE_PL_HELP" ||
        die 'la ayuda Rust del AppImage no respeta --lang pl'
    ok 'idiomas nuevos en la CLI del AppImage'
    APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --doctor >/dev/null
    ok 'diagnóstico del AppImage'
    if [[ -c /dev/fuse ]] &&
        { command -v fusermount3 >/dev/null 2>&1 || command -v fusermount >/dev/null 2>&1; }; then
        "$APPIMAGE_PATH" --version >/dev/null
        ok 'AppImage responde con montaje FUSE normal'
    else
        skip 'montaje FUSE normal: no disponible; se validó el fallback por extracción'
    fi
    set +e
    FUSE_OUTPUT="$(APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGE_PATH" --fuse-check 2>&1)"
    FUSE_STATUS=$?
    set -e
    [[ "$FUSE_STATUS" -eq 0 || "$FUSE_STATUS" -eq 1 ]] || die '--fuse-check terminó con un código inesperado'
    grep -Fq 'FUSE' <<<"$FUSE_OUTPUT" || die '--fuse-check no mostró diagnóstico FUSE'
    ok 'diagnóstico FUSE'
    if [[ -n "$RUNNER_PATH" ]]; then
        [[ -x "$RUNNER_PATH" ]] || die "lanzador no ejecutable: $RUNNER_PATH"
        LTOOLS_FORCE_EXTRACT=1 "$RUNNER_PATH" --version >/dev/null
        ok 'lanzador externo con fallback forzado sin FUSE'
    fi
else
    skip 'AppImage: no se proporcionó --appimage'
fi

printf 'Smoke tests completados correctamente.\n'
