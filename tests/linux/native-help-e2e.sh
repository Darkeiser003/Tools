#!/usr/bin/env bash
# Audita las ayudas reales de las herramientas nativas y su superficie GUI.

set -Eeuo pipefail
export LTOOLS_LANG=es

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ltools-native-help.XXXXXX")"
trap 'rm -rf -- "$TMP_DIR"' EXIT

die() { printf 'NATIVE HELP E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }
# Algunas herramientas imprimen bytes de control en su ayuda (por ejemplo
# mkfs.xfs); tratar todos los informes como texto evita falsos avisos de grep
# y mantiene el parseo de opciones estable.
grep() { command grep -a "$@"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        -h|--help)
            printf 'Uso: %s [--binary RUTA]\n' "$0"
            exit 0
            ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ -x "$BIN" ]] || die "no existe el binario ejecutable: $BIN"

export HOME="$TMP_DIR/home"
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_STATE_HOME="$HOME/.local/state"
export TMPDIR="$TMP_DIR"
export WINEPREFIX="$TMP_DIR/wineprefix"
export KUBECONFIG="$TMP_DIR/kubeconfig"
export DOCKER_CONFIG="$TMP_DIR/docker"
export GIT_CONFIG_NOSYSTEM=1
mkdir -p "$HOME" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_STATE_HOME"

run_help() {
    local output="$1" tool="$2" allow_nonzero=0
    shift 2
    : >"$output"
    local status
    case "$tool" in
        # Algunos comandos publican ayuda completa con código no cero: los
        # clientes OpenSSH sin destino (1/255) y iproute2 `ip/bridge --help`
        # (255).
        # Las excepciones están acotadas y la salida sigue debiendo ser
        # reconocible; ninguna otra herramienta convierte un error en ayuda.
        ssh|scp|sftp|ip|bridge|openvpn|ssh-copy-id|ssh-keygen) allow_nonzero=1 ;;
    esac
    set +e
    timeout 12 "$tool" "$@" >"$output" 2>&1
    status=$?
    set -e
    [[ -s "$output" ]] || return 1
    if [[ "$status" -eq 124 ]]; then
        return 1
    fi
    if [[ "$status" -ne 0 && "$allow_nonzero" -eq 0 ]]; then
        return 1
    fi
    if [[ "$status" -ne 0 ]] && ! grep -Eiq 'usage|options|commands|help|opciones|comandos' "$output"; then
        return 1
    fi
    ! grep -Eiq 'unknown command|unrecognized command|invalid choice|not a valid command|unknown option|unrecognized option|invalid option|illegal option|bad option' "$output"
}

help_for() {
    local tool="$1" output="$2" status
    case "$tool" in
        adb) run_help "$output" "$tool" help || run_help "$output" "$tool" --help ;;
        # OpenSSH clients have no portable help switch. With no arguments they
        # print their complete usage and exit nonzero; do not fake a query with
        # -h/--help, which these clients report as an unknown option.
        ssh|scp|sftp) run_help "$output" "$tool" ;;
        ssh-copy-id)
            # ssh-copy-id imprime el uso completo con `-h` y código 1.
            if timeout 12 "$tool" -h >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage:' "$output" && grep -Eq -- '\[-f\]|\[-n\]|\[-i' "$output"
            ;;
        ssh-keygen)
            # `-h` o una invocación sin argumentos pueden generar una clave;
            # `--help` es seguro, imprime el contrato y termina con código 1.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'usage: ssh-keygen' "$output" &&
                grep -Fq -- '-t ecdsa|ecdsa-sk|ed25519' "$output"
            ;;
        # e2fsprogs exposes emergency usage for an invalid option. The error
        # text is part of its native help path; require both its Usage and the
        # detailed Emergency help section to reject unrelated failures.
        e2fsck)
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: e2fsck' "$output" &&
                grep -Eiq 'Emergency help:|Ayuda de emergencia:' "$output"
            ;;
        xfs_admin)
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: xfs_admin' "$output"
            ;;
        xhost)
            # xhost documents its syntax with the historical single-dash
            # spelling. Its default invocation tries to contact DISPLAY and
            # is not a help query.
            if timeout 12 "$tool" -help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'usage: xhost' "$output"
            ;;
        exfatlabel)
            # exfatprogs publica una ayuda completa con código 1 incluso
            # usando --help; exigir el encabezado y sus opciones evita
            # confundir un fallo de acceso al dispositivo con documentación.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: exfatlabel' "$output" &&
                grep -Eq -- '--volume-serial|--version|--help' "$output"
            ;;
        gradle)
            if run_help "$output" "$tool" --help; then return 0; fi
            # Con HOME/XDG aislados, algunas instalaciones de Gradle no
            # pueden cargar su libnative-platform local. No es una ayuda
            # válida, pero sí una limitación del entorno de la E2E; cualquier
            # otro error de Gradle debe seguir fallando la auditoría.
            grep -Fq "Failed to load native library 'libnative-platform.so'" "$output" && return 2
            return 1
            ;;
        inxi)
            if run_help "$output" "$tool" --help; then return 0; fi
            # inxi se niega a mostrar --help cuando no detecta una tty y
            # clasifica la ejecución como cliente IRC. La E2E no debe
            # falsificar una tty, así que se informa como limitación del
            # runner solo ante ese mensaje exacto.
            grep -Fq "You can't run option help in an IRC client!" "$output" && return 2
            return 1
            ;;
        kpartx)
            # multipath-tools imprime su ayuda con `--help` y código 1; no
            # usar el fallback `help`, que intenta hablar con device-mapper.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage:' "$output" && grep -Eq -- '-a add partition|-d del partition|-l list partitions' "$output"
            ;;
        lspci)
            # pciutils usa opciones cortas, imprime la ayuda al recibir una
            # opción desconocida y termina con 1; validar varias secciones
            # reales evita aceptar solo el texto de error.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: lspci' "$output" &&
                grep -Fq 'Basic display modes:' "$output" &&
                grep -Fq 'Selection of devices:' "$output"
            ;;
        memtester)
            # Sin tamaño de memoria, memtester muestra el contrato de uso y
            # termina con 1 después de imprimir datos inocuos del sistema;
            # nunca se debe lanzar una prueba real desde esta E2E.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: memtester' "$output" && grep -Fq 'memtester version' "$output"
            ;;
        mkfs.exfat)
            # exfatprogs imprime su contrato de formato con código 1 ante
            # --help; no ejecutar el fallback `help`, que intenta abrirlo
            # como un dispositivo de salida.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: mkfs.exfat' "$output" &&
                grep -Eq -- '--volume-label|--volume-guid|--help' "$output"
            ;;
        mkfs.ext4)
            # Los alias mkfs.ext4/mke2fs documentan opciones cortas, pero
            # imprimen el uso al rechazar --help con código 1.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: mkfs.ext4' "$output" &&
                grep -Eq -- '-L volume-label|-O feature|-U UUID' "$output"
            ;;
        mkfs.xfs)
            # xfsprogs también imprime el uso al rechazar --help con código
            # 1; se valida que el dispositivo no llegue a abrirse.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -aFq 'Usage: mkfs.xfs' "$output" &&
                grep -aEq -- '\[-b size=num\]|\[-L label|devicename' "$output"
            ;;
        openvpn)
            # OpenVPN 2.7 publica toda la ayuda con código 1 incluso ante
            # --help; no debe probarse el fallback `help` como configuración.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'General Options:' "$output" &&
                grep -Fq -- '--config file' "$output" &&
                grep -Fq -- '--help' "$output"
            ;;
        resize2fs)
            # e2fsprogs usa `-h` y devuelve 1 al mostrar el uso; no pasar un
            # nombre ficticio al backend para no abrir dispositivos reales.
            if timeout 12 "$tool" -h >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: resize2fs' "$output" &&
                grep -Eq -- 'device|new_size|RAID-stride' "$output"
            ;;
        socat)
            # socat solo acepta la ayuda corta `-h`; `--help` es un error de
            # opción y su mensaje puede parecer ayuda sin serlo.
            run_help "$output" "$tool" -h
            ;;
        tune2fs)
            # tune2fs no tiene una opción --help dedicada; al rechazarla
            # publica igualmente su sintaxis completa con código 1.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: tune2fs' "$output" &&
                grep -Eq -- '-L volume_label|-U UUID|device' "$output"
            ;;
        xfs_growfs)
            # xfs_growfs documenta sus opciones cortas al rechazar --help
            # con código 2; no se debe pasar un punto de montaje inventado.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: xfs_growfs' "$output" &&
                grep -Fq $'\t-d' "$output" && grep -Fq $'\t-l' "$output"
            ;;
        xfs_info)
            # xfs_info solo tiene `-V` como opción informativa; las opciones
            # de ayuda inválidas imprimen su contrato con código 2.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: xfs_info' "$output" && grep -Fq 'mountpoint' "$output"
            ;;
        xfs_repair)
            # xfs_repair imprime su sintaxis al rechazar --help con código 1;
            # no ejecutar ninguna reparación ni abrir un dispositivo.
            if timeout 12 "$tool" --help >"$output" 2>&1; then status=0; else status=$?; fi
            [[ "$status" -ne 124 ]] || return 1
            grep -Fq 'Usage: xfs_repair' "$output" &&
                grep -Fq 'No modify mode' "$output" && grep -Fq 'device' "$output"
            ;;
        # These launch graphical partition editors rather than exposing a
        # CLI contract. Calling them with generic help switches can invoke
        # pkexec or abort before GTK/Qt finds a usable display.
        gparted|partitionmanager)
            printf 'Herramienta gráfica; no publica ayuda CLI.\n' >"$output"
            return 2
            ;;
        wineboot)
            # Incluso las consultas de ayuda pueden crear/inicializar un
            # prefijo y arrancar wineserver. La suite no debe convertir una
            # comprobación de documentación en una acción de inicialización.
            printf 'Inicializador de prefijos Wine; se omite para no crear ni modificar un prefijo durante la consulta de ayuda.\n' >"$output"
            return 2
            ;;
        trash)
            printf 'Integración de papelera; el backend llama a gio/trash-put y no existe un comando trash propio.\n' >"$output"
            return 2
            ;;
        nft)
            if run_help "$output" "$tool" --help; then return 0; fi
            grep -Eiq 'Netlink socket: Operation not permitted|Operation not permitted' "$output" && return 2
            return 1
            ;;
        bridge)
            if run_help "$output" "$tool" --help; then return 0; fi
            grep -Eiq 'Netlink socket: Operation not permitted|Operation not permitted' "$output" && return 2
            return 1
            ;;
        ip)
            if run_help "$output" "$tool" --help; then return 0; fi
            grep -Eiq 'Netlink socket: Operation not permitted|Operation not permitted' "$output" && return 2
            return 1
            ;;
        udisksctl)
            if run_help "$output" "$tool" help; then return 0; fi
            if run_help "$output" "$tool" --help; then return 0; fi
            grep -Eiq 'Could not connect: Operation not permitted|Operation not permitted' "$output" && return 2
            return 1
            ;;
        useradd|userdel|usermod)
            if run_help "$output" "$tool" --help; then return 0; fi
            grep -Fq 'Cannot open audit interface - aborting.' "$output" && return 2
            return 1
            ;;
        # `git help -a` publica el catálogo real de subcomandos sin abrir un
        # pager/man interactivo, por lo que sirve mejor como superficie de
        # contraste que `git --help`.
        git) run_help "$output" "$tool" help -a || run_help "$output" "$tool" --help ;;
        gh|kubectl) run_help "$output" "$tool" help || run_help "$output" "$tool" --help ;;
        docker-compose)
            if command -v docker-compose >/dev/null 2>&1; then
                run_help "$output" docker-compose --help
            else
                run_help "$output" docker compose --help
            fi
            ;;
        dig)
            if run_help "$output" "$tool" -h; then return 0; fi
            # Some sandboxed hosts deny socket() even while dig initializes
            # its help path. This is an environment restriction, not evidence
            # that the invalid `dig --help` fallback is a valid help screen.
            if grep -Eiq 'Operation not permitted|can.t find either v4 or v6 networking' "$output"; then
                return 2
            fi
            return 1
            ;;
        nslookup)
            # BIND nslookup rejects the common -h spelling as an invalid
            # option; try common long spellings first. Some builds also need
            # socket initialization before printing anything, so restricted
            # network namespaces are reported as an environmental skip.
            for help_switch in -help --help -h; do
                if run_help "$output" "$tool" "$help_switch"; then return 0; fi
                if grep -Eiq 'Operation not permitted|can.t find either v4 or v6 networking' "$output"; then
                    return 2
                fi
            done
            if grep -Eiq 'invalid option|unknown option|unrecognized option' "$output"; then
                printf 'Esta implementación no ofrece una opción de ayuda CLI no interactiva verificable.\n' >"$output"
                return 2
            fi
            return 1
            ;;
        podman|podman-compose)
            if run_help "$output" "$tool" --help; then return 0; fi
            # Podman puede fallar antes de imprimir su ayuda cuando el host de
            # la E2E monta /run/user como solo lectura o conserva un pause.pid
            # de otro namespace. Es una limitación del runtime, no una ayuda
            # inválida; cualquier otro error sigue siendo un fallo.
            if grep -Eiq 'read-only file system|set sticky bit|pause\.pid|operation not permitted' "$output"; then
                return 2
            fi
            return 1
            ;;
        lsof|tcpdump) run_help "$output" "$tool" -h || run_help "$output" "$tool" --help ;;
        lvm) run_help "$output" "$tool" help || run_help "$output" "$tool" --help ;;
        7z|unzip|zip|zstd) run_help "$output" "$tool" -h || run_help "$output" "$tool" --help ;;
        *) run_help "$output" "$tool" --help || run_help "$output" "$tool" help ;;
    esac
}

option_count() {
    local file="$1"
    grep -Eo -- '(^|[^[:alnum:]_-])(--[[:alnum:]][[:alnum:]_-]*|-[[:alpha:]])([^[:alnum:]_-]|$)' "$file" |
        sed -E 's/^[^[:alnum:]-]*//' | sed -E 's/[^[:alnum:]_-]*$//' |
        sort -u | wc -l | tr -d ' ' || true
}

has_word() {
    local file="$1" word="$2"
    grep -Eiq -- "(^|[^[:alnum:]_-])${word}([^[:alnum:]_-]|$)" "$file"
}

native_has_operation() {
    local tool="$1" root_help="$2" operation="$3" sub_help
    has_word "$root_help" "$operation" && return 0
    sub_help="$TMP_DIR/${tool//\//_}-${operation}.out"
    if [[ "$operation" == "prune" ]]; then
        run_help "$sub_help" "$tool" container prune --help &&
            has_word "$sub_help" prune &&
            ! grep -Eiq 'unknown command|unrecognized command|invalid choice|not a valid command|unknown option|unrecognized option|invalid option|illegal option|bad option' "$sub_help" &&
            return 0
    fi
    run_help "$sub_help" "$tool" "$operation" --help ||
        run_help "$sub_help" "$tool" "$operation" help ||
        run_help "$sub_help" "$tool" help "$operation" || return 1
    has_word "$sub_help" "$operation" &&
        ! grep -Eiq 'unknown command|unrecognized command|invalid choice|not a valid command|unknown option|unrecognized option|invalid option|illegal option|bad option' "$sub_help"
}

check_surface() {
    local tool="$1" guide_topic="$2" expected_count="$3" root_help="$4"
    shift 4
    local guide_output="$TMP_DIR/guide-${guide_topic}.out" operation label native_count=0
    local -a expected=()
    while (($#)); do
        expected+=("$1"); shift
    done
    [[ "${#expected[@]}" -eq "$expected_count" ]] ||
        die "$tool: contrato de auditoría mal definido (${#expected[@]} != $expected_count)"
    "$BIN" guide gui "$guide_topic" >"$guide_output" 2>&1 ||
        die "$tool: no se pudo abrir la guía GUI/$guide_topic"
    for operation in "${expected[@]}"; do
        label="${operation#*:}"
        operation="${operation%%:*}"
        native_has_operation "$tool" "$root_help" "$operation" ||
            die "$tool: la ayuda nativa no expone la operación '$operation'"
        grep -Fq -- "$label" "$guide_output" ||
            die "$tool: la guía GUI/$guide_topic no documenta '$label'"
        native_count=$((native_count + 1))
    done
    [[ "$native_count" -eq "$expected_count" ]] ||
        die "$tool: la GUI ofrece $native_count operaciones y el contrato esperaba $expected_count"
    ok "$tool: $native_count/$expected_count operaciones nativas comprobadas contra GUI/$guide_topic"
}

printf 'E2E: descubriendo herramientas del catálogo instalado...\n'
HELP_DIR="$TMP_DIR/help"
mkdir -p "$HELP_DIR"
CAPABILITIES_JSON="$TMP_DIR/capabilities.json"
"$BIN" capabilities --format json >"$CAPABILITIES_JSON" 2>"$TMP_DIR/capabilities.err" ||
    die 'no se pudo enumerar el catálogo real de herramientas del anfitrión'

# El catálogo de botones/acciones es otra superficie pública: si una GUI o
# una terminal consume este JSON, cada entrada debe tener un backend conocido,
# una política de seguridad y un ID único. Esto detecta acciones huérfanas aun
# cuando la ayuda de la herramienta nativa siga siendo correcta.
ACTIONS_JSON="$TMP_DIR/actions.json"
"$BIN" actions list --format json >"$ACTIONS_JSON" 2>"$TMP_DIR/actions.err" ||
    die 'no se pudo enumerar el catálogo JSON de acciones guiadas'
if command -v jq >/dev/null 2>&1; then
    jq -e '
        .schema == "ltools-actions-v1" and
        .platform == "linux" and
        (.actions | length > 0) and
        (([.actions[].id] | length) == ([.actions[].id] | unique | length)) and
        (([.actions[].actionKey] | length) == ([.actions[].actionKey] | unique | length)) and
        (([.actions[].operation] | length) == ([.actions[].operation] | unique | length)) and
        (([.actions[].label] | length) == ([.actions[].label] | unique | length)) and
        (([.actions[].shortLabel] | length) == ([.actions[].shortLabel] | unique | length)) and
        (([.actions[].qualifiedActionKey] | length) == ([.actions[].qualifiedActionKey] | unique | length)) and
        (([.actions[].canonicalKey] | length) == ([.actions[].canonicalKey] | unique | length)) and
        all(.actions[];
            (.id | type == "string" and length > 0) and
            (.id == .actionId) and (.legacyId | type == "string" and length > 0) and
            (.actionKey | type == "string" and test("^[a-z0-9]+(?:[.-][a-z0-9]+)+$")) and
            (.qualifiedActionKey == ("linux." + .actionKey)) and
            (.canonicalKey == .qualifiedActionKey) and
            (.scope | type == "string" and length > 0) and
            (.operation | type == "string" and length > 0) and
            (.operation == (.actionKey | gsub("\\."; "-"))) and
            (.label | type == "string" and length > 0) and
            (.shortLabel | type == "string" and length > 0) and
            (.displayName | type == "string" and length > 0) and
            (.menuPath | type == "array" and length == 2) and
            (.description | type == "string" and length > 0) and
            (.group | type == "string" and length > 0) and
            (.invocation.executable == "ltools") and
            (.invocation.args == ["actions", "run", .actionId]) and
            (.invocation.target == .target) and
            (.command | IN("audit", "packages", "games", "storage", "snapshots", "system", "accounts", "native", "defaults", "clean", "diagnostics", "automation", "boot", "wine")) and
            (.targetPolicy | type == "string" and length > 0) and
            (.profile | type == "string" and length > 0) and
            (.mutating | type == "boolean") and
            (.confirmation | type == "string") and
            (.args | type == "array" and all(.[]; type == "string"))
        )
    ' "$ACTIONS_JSON" >/dev/null ||
        die 'el catálogo JSON de acciones contiene IDs duplicados, campos incompletos o backends desconocidos'
    ok 'catálogo JSON de acciones: IDs, backends y políticas verificados'
else
    grep -Fq '"schema":"ltools-actions-v1"' "$ACTIONS_JSON" ||
        die 'el catálogo JSON de acciones no declara su esquema'
    grep -Fq '"actions":[' "$ACTIONS_JSON" ||
        die 'el catálogo JSON de acciones no contiene acciones'
    ok 'catálogo JSON de acciones disponible (jq no instalado; validación estructural reducida)'
fi
mapfile -t tools < <(
    sed -n 's/.*"command":"\([^"]*\)".*"available":true.*/\1/p' "$CAPABILITIES_JSON" |
        sort -u
)
(( ${#tools[@]} > 0 )) || die 'el catálogo disponible no contiene comandos para auditar'
printf 'E2E: ejecutando las ayudas nativas reales disponibles (%s comandos)...\n' "${#tools[@]}"
checked=0
declare -A help_skipped=()
for tool in "${tools[@]}"; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        if [[ "$tool" == docker-compose ]] && command -v docker >/dev/null 2>&1; then
            : # El catálogo considera válido el subcomando docker compose.
        elif [[ "$tool" == trash ]]; then
            : # Adaptador de plataforma, no un ejecutable homónimo.
        else
            die "el catálogo marca $tool como disponible, pero no se resuelve en el PATH de la E2E"
        fi
    fi
    help_file="$HELP_DIR/${tool//\//_}.out"
    if help_for "$tool" "$help_file"; then
        :
    else
        help_status=$?
        if [[ "$help_status" -eq 2 ]]; then
            reason="$(head -n 1 "$help_file" | tr -d '\r')"
            printf '  SKIP  %s: %s\n' "$tool" "${reason:-el entorno impidió consultar la ayuda}"
            help_skipped["$tool"]=1
            continue
        fi
        printf 'Salida recibida de %s (código de ayuda inválida):\n' "$tool" >&2
        sed -n '1,40p' "$help_file" >&2
        die "$tool está instalado pero no devuelve ninguna ayuda válida"
    fi
    count="$(option_count "$help_file")"
    if [[ "$count" -gt 0 ]]; then
        printf '  OK    %s: ayuda disponible (%s opciones)\n' "$tool" "$count"
    else
        printf '  OK    %s: ayuda disponible (sin opciones parseables; se conserva la salida)\n' "$tool"
    fi
    checked=$((checked + 1))
done
((checked > 0)) || die 'no se encontró ninguna herramienta nativa auditable'

if command -v adb >/dev/null 2>&1; then
    adb_help="$HELP_DIR/adb.out"
    # Este es el contrato solicitado: la fuente de verdad es exactamente adb help.
    run_help "$adb_help" adb help || die 'adb help no devolvió ayuda'
    check_surface adb adb 5 "$adb_help" \
        'shell:Abrir shell ADB' 'install:Instalar APK' 'push:Enviar archivo ADB' \
        'pull:Extraer archivo ADB' 'reboot:Reiniciar dispositivo ADB'
fi

if command -v git >/dev/null 2>&1; then
    check_surface git git 10 "$HELP_DIR/git.out" \
        'status:Estado del repositorio Git' 'clone:Clonar un repositorio Git' \
        'fetch:Descargar referencias Git remotas' 'pull:Descargar e integrar cambios Git' \
        'log:Ver historial de commits' 'add:Preparar cambios' 'commit:Crear commit' \
        'push:Subir cambios' 'branch:Listar o cambiar de rama' 'tag:Listar o crear tag'
fi

if command -v gh >/dev/null 2>&1; then
    check_surface gh git 4 "$HELP_DIR/gh.out" \
        'auth:Estado de autenticación GitHub' 'repo:Repositorio de GitHub' \
        'pr:Pull requests de GitHub' 'release:Releases de GitHub'
fi

if command -v ssh >/dev/null 2>&1 && command -v scp >/dev/null 2>&1 && command -v sftp >/dev/null 2>&1; then
    transfer_guide="$TMP_DIR/guide-ssh.out"
    "$BIN" guide gui ssh >"$transfer_guide" 2>&1 || die 'no se pudo abrir la guía GUI/ssh'
    for marker in 'Conectar por SSH' 'Copiar con SCP' 'Abrir SFTP'; do
        grep -Fq -- "$marker" "$transfer_guide" || die "la guía SSH no documenta '$marker'"
    done
    for spec in 'ssh:destination' 'scp:source' 'sftp:destination'; do
        tool="${spec%%:*}"
        marker="${spec#*:}"
        grep -Fqi -- "$marker" "$HELP_DIR/$tool.out" ||
            die "$tool: la ayuda nativa no muestra el argumento '$marker'"
    done
    grep -Eiq 'source.*target' "$HELP_DIR/scp.out" ||
        die 'scp: la ayuda nativa no muestra la relación origen/destino'
    ok 'SSH/SCP/SFTP: acciones GUI y argumentos de transferencia contrastados con ayuda nativa'
fi

if command -v kubectl >/dev/null 2>&1; then
    check_surface kubectl kubernetes 5 "$HELP_DIR/kubectl.out" \
        'apply:Aplicar manifiesto' 'delete:Eliminar recurso' 'scale:Escalar deployment' \
        'rollout:Reiniciar rollout' 'port-forward:Abrir port-forward'
fi

for engine in docker podman; do
    if command -v "$engine" >/dev/null 2>&1; then
        if [[ "${help_skipped[$engine]:-0}" == 1 ]]; then
            printf '  SKIP  %s: la ayuda base quedó omitida por una restricción del runtime del host\n' "$engine"
            continue
        fi
        check_surface "$engine" containers-lifecycle 19 "$HELP_DIR/$engine.out" \
            'pull:Descargar imagen' 'run:Crear y ejecutar contenedor' 'start:Iniciar contenedor' \
            'stop:Detener contenedor' 'restart:Reiniciar contenedor' 'rm:Eliminar contenedor' \
            'logs:Ver logs del contenedor' 'exec:Ejecutar comando en contenedor' \
            'inspect:Inspeccionar contenedor' 'stats:Estadísticas de contenedor' \
            'top:Procesos del contenedor' 'port:Puertos publicados' 'diff:Cambios del contenedor' \
            'pause:Pausar contenedor' 'unpause:Reanudar contenedor' 'kill:Terminar contenedor' \
            'rename:Renombrar contenedor' 'cp:Copiar archivos' 'prune:Limpiar contenedores detenidos'
    fi
done

printf 'Auditoría nativa completada: %s herramienta(s), ayuda real y contratos GUI verificados.\n' "$checked"
