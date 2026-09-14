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

run_help() {
    local output="$1" tool="$2"
    shift 2
    : >"$output"
    local status
    set +e
    timeout 12 "$tool" "$@" >"$output" 2>&1
    status=$?
    set -e
    [[ -s "$output" ]] || return 1
    if [[ "$status" -eq 124 ]]; then
        return 1
    fi
    if [[ "$status" -ne 0 ]] && ! grep -Eiq 'usage|options|commands|help|opciones|comandos' "$output"; then
        return 1
    fi
    ! grep -Eiq 'unknown command|unrecognized command|invalid choice|not a valid command|unknown option|unrecognized option|invalid option|illegal option|bad option' "$output"
}

help_for() {
    local tool="$1" output="$2"
    case "$tool" in
        adb) run_help "$output" "$tool" help || run_help "$output" "$tool" --help ;;
        # OpenSSH clients have no portable help switch. With no arguments they
        # print their complete usage and exit nonzero; do not fake a query with
        # -h/--help, which these clients report as an unknown option.
        ssh|scp|sftp) run_help "$output" "$tool" ;;
        # `git help -a` publica el catálogo real de subcomandos sin abrir un
        # pager/man interactivo, por lo que sirve mejor como superficie de
        # contraste que `git --help`.
        git) run_help "$output" "$tool" help -a || run_help "$output" "$tool" --help ;;
        gh|kubectl) run_help "$output" "$tool" help || run_help "$output" "$tool" --help ;;
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

printf 'E2E: ejecutando las ayudas nativas reales disponibles...\n'
HELP_DIR="$TMP_DIR/help"
mkdir -p "$HELP_DIR"
tools=(
    adb docker podman kubectl helm git gh ssh scp sftp nmcli ip ss systemctl
    efibootmgr parted lsblk findmnt mount umount fsck cryptsetup lvm btrfs zpool
    mdadm curl wget file tree htop btop lsof strace tcpdump dig nmap openssl gpg
    7z unzip zip zstd tmux python3 make cmake gcc gdb
)
checked=0
for tool in "${tools[@]}"; do
    command -v "$tool" >/dev/null 2>&1 || continue
    help_file="$HELP_DIR/${tool//\//_}.out"
    if help_for "$tool" "$help_file"; then
        :
    else
        help_status=$?
        if [[ "$tool" == dig && "$help_status" -eq 2 ]]; then
            printf '  SKIP  dig: el sandbox bloqueó socket() antes de mostrar ayuda\n'
            continue
        fi
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
