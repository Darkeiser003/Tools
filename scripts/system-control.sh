#!/usr/bin/env bash

# system-control.sh - cautious systemd/process/journal module.
# Queries are read-only. Mutating actions require an explicit confirmation.

set -uo pipefail
export LC_ALL=C

VERSION="0.1.0"
SCRIPT_NAME="$(basename "$0")"
SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd -P)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib/ltools-plan.sh"
USER_SCOPE=0
ACTION="menu"
UNIT=""
PID=""
JOURNAL_UNIT=""
JOURNAL_LINES=100
DRY_RUN=0
PLAN_PATH=""
ROLLBACK_PATH=""

usage() {
    cat <<EOF
Uso: $SCRIPT_NAME [acción] [opciones]

Acciones:
  menu                         Abre el menú interactivo.
  status                       Estado general de systemd y unidades fallidas.
  services                     Lista servicios del sistema.
  user-services               Lista servicios de usuario (--user).
  service ACCIÓN UNIDAD        status/start/stop/restart/enable/disable/mask/unmask.
  processes                    Lista procesos con mayor consumo.
  process ACCIÓN PID           status/stop/kill para un proceso.
  journal                      Muestra avisos recientes del arranque actual.

Opciones:
  --user                       Usa el systemd del usuario para services/service.
  --unit UNIDAD                Filtra el journal por unidad.
  --lines N                    Número de líneas del journal (por defecto: 100).
  --dry-run                    Simula cambios de servicios/procesos sin ejecutarlos.
  --plan FICHERO               Guarda el plan en una ruta concreta.
  --rollback FICHERO           Revierte las operaciones reversibles del plan.
  -h, --help                   Muestra esta ayuda.
  --version                    Muestra la versión.

Ejemplos:
  $SCRIPT_NAME status
  $SCRIPT_NAME services
  $SCRIPT_NAME --user services
  $SCRIPT_NAME service restart NetworkManager.service
  $SCRIPT_NAME process stop 1234
  $SCRIPT_NAME journal --unit systemd-logind.service --lines 80
EOF
}

die() { printf 'Error: %s\n' "$*" >&2; exit 2; }
have() { command -v "$1" >/dev/null 2>&1; }

confirm() {
    local answer
    printf '%s [y/N] ' "$1"
    read -r answer || return 1
    [[ "$answer" =~ ^([yY]|[sS])$ ]]
}

systemctl_query() {
    if (( USER_SCOPE )); then
        systemctl --user "$@"
    else
        systemctl "$@"
    fi
}

systemctl_mutate() {
    if (( USER_SCOPE )); then
        systemctl --user "$@"
    elif (( EUID == 0 )); then
        systemctl "$@"
    elif have sudo; then
        sudo systemctl "$@"
    else
        printf 'Se necesita sudo para modificar servicios del sistema.\n' >&2
        return 1
    fi
}

validate_unit() {
    [[ "$1" != -* && "$1" =~ ^[A-Za-z0-9_.@:-]+$ ]] || die "unidad no válida: $1"
}

validate_pid() {
    [[ "$1" =~ ^[0-9]+$ && "$1" != 0 ]] || die "PID no válido: $1"
    [[ -d "/proc/$1" ]] || die "no existe el PID: $1"
}

show_status() {
    have systemctl || die "systemctl no está instalado"
    printf '\n=== Estado del sistema ===\n'
    printf 'systemd: '
    systemctl is-system-running 2>/dev/null || true
    printf '\nUnidades fallidas:\n'
    systemctl --failed --no-legend --no-pager 2>/dev/null || printf '  No disponible sin acceso al systemd del sistema.\n'
    printf '\nServicios activos principales:\n'
    systemctl list-units --type=service --state=running --no-legend --no-pager 2>/dev/null | head -30 || true
    printf '\nServicios habilitados (primeros 30):\n'
    systemctl list-unit-files --type=service --state=enabled --no-legend --no-pager 2>/dev/null | head -30 || true
}

list_services() {
    have systemctl || die "systemctl no está instalado"
    printf '\n=== Servicios %s ===\n' "$([[ "$USER_SCOPE" == 1 ]] && printf 'del usuario' || printf 'del sistema')"
    systemctl_query list-units --type=service --all --no-legend --no-pager 2>/dev/null || \
        printf 'No se pudo consultar este gestor de servicios.\n'
}

service_action() {
    local operation="$1" unit="$2" label
    have systemctl || die "systemctl no está instalado"
    validate_unit "$unit"
    case "$operation" in
        status)
            systemctl_query status "$unit" --no-pager
            ;;
        start|stop|restart|enable|disable|mask|unmask)
            label="$operation $unit"
            printf 'Se va a ejecutar sobre %s: systemctl %s %s\n' \
                "$([[ "$USER_SCOPE" == 1 ]] && printf 'servicios de usuario' || printf 'servicios del sistema')" "$operation" "$unit"
            if [[ "$operation" == stop || "$operation" == restart || "$operation" == disable || "$operation" == mask ]]; then
                confirm "¿Confirmas $label?" || { printf 'Operación cancelada.\n'; return 0; }
            else
                confirm "¿Ejecutar $label?" || { printf 'Operación cancelada.\n'; return 0; }
            fi
            if (( DRY_RUN )); then
                plan_record service-change "$unit" planned no "$operation" "$([[ "$USER_SCOPE" == 1 ]] && printf user || printf system)"
                printf 'Simulación: no se ejecutaría el cambio de servicio.\n'
            elif systemctl_mutate "$operation" "$unit"; then
                plan_record service-change "$unit" executed no "$operation" "$([[ "$USER_SCOPE" == 1 ]] && printf user || printf system)"
            else
                return 1
            fi
            ;;
        daemon-reload)
            confirm '¿Recargar la configuración de systemd?' || return 0
            if (( DRY_RUN )); then
                plan_record daemon-reload systemd planned no "systemctl daemon-reload" "$([[ "$USER_SCOPE" == 1 ]] && printf user || printf system)"
                printf 'Simulación: no se recargaría systemd.\n'
            elif systemctl_mutate daemon-reload; then
                plan_record daemon-reload systemd executed no "systemctl daemon-reload" "$([[ "$USER_SCOPE" == 1 ]] && printf user || printf system)"
            else
                return 1
            fi
            ;;
        *) die "acción de servicio no válida: $operation" ;;
    esac
}

list_processes() {
    have ps || die "ps no está disponible"
    printf '\n=== Procesos por memoria ===\n'
    ps -eo pid=,user=,%cpu=,%mem=,stat=,etime=,comm=,args= --sort=-%mem 2>/dev/null | head -40 || true
    printf '\n=== Procesos por CPU ===\n'
    ps -eo pid=,user=,%cpu=,%mem=,stat=,etime=,comm=,args= --sort=-%cpu 2>/dev/null | head -40 || true
}

process_action() {
    local operation="$1" pid="$2"
    validate_pid "$pid"
    case "$operation" in
        status)
            ps -o pid,ppid,user,%cpu,%mem,stat,etime,comm,args -p "$pid"
            if [[ -r "/proc/$pid/cgroup" ]]; then
                printf '\ncgroup:\n'
                sed -n '1,20p' "/proc/$pid/cgroup"
            fi
            ;;
        stop|kill)
            printf 'Proceso seleccionado:\n'
            ps -o pid,ppid,user,%cpu,%mem,stat,etime,comm,args -p "$pid"
            if [[ "$operation" == stop ]]; then
                confirm "¿Enviar SIGTERM al proceso $pid?" || { printf 'Operación cancelada.\n'; return 0; }
                if (( DRY_RUN )); then
                    plan_record process-signal "/proc/$pid" planned no SIGTERM "$pid"
                    printf 'Simulación: no se enviaría SIGTERM al PID %s.\n' "$pid"
                elif kill -TERM "$pid" 2>/dev/null || { have sudo && sudo kill -TERM "$pid"; }; then
                    plan_record process-signal "/proc/$pid" executed no SIGTERM "$pid"
                else
                    return 1
                fi
            else
                confirm "¿Enviar SIGKILL al proceso $pid?" || { printf 'Operación cancelada.\n'; return 0; }
                if (( DRY_RUN )); then
                    plan_record process-signal "/proc/$pid" planned no SIGKILL "$pid"
                    printf 'Simulación: no se enviaría SIGKILL al PID %s.\n' "$pid"
                elif kill -KILL "$pid" 2>/dev/null || { have sudo && sudo kill -KILL "$pid"; }; then
                    plan_record process-signal "/proc/$pid" executed no SIGKILL "$pid"
                else
                    return 1
                fi
            fi
            ;;
        *) die "acción de proceso no válida: $operation" ;;
    esac
}

show_journal() {
    have journalctl || die "journalctl no está disponible"
    local -a args=(--since today --no-pager -n "$JOURNAL_LINES" -p warning..alert -b)
    [[ -n "$JOURNAL_UNIT" ]] && args+=(--unit "$JOURNAL_UNIT")
    printf '\n=== Avisos del journal ===\n'
    journalctl "${args[@]}" 2>/dev/null || printf 'No se pudo leer el journal con el usuario actual.\n'
}

menu_services() {
    local choice unit operation answer
    while :; do
        printf '\n=== Servicios y daemons ===\n'
        printf '  1) Estado general del sistema\n'
        printf '  2) Listar servicios del sistema\n'
        printf '  3) Listar servicios del usuario\n'
        printf '  4) Consultar una unidad\n'
        printf '  5) Iniciar/detener/reiniciar una unidad\n'
        printf '  6) Habilitar/deshabilitar o enmascarar una unidad\n'
        printf '  7) Recargar daemons\n'
        printf '  q) Volver\n'
        printf 'Elige una opción: '
        read -r choice || return 0
        case "$choice" in
            1) show_status ;;
            2) USER_SCOPE=0; list_services ;;
            3) USER_SCOPE=1; list_services ;;
            4)
                printf 'Unidad: '; read -r unit || return 0
                USER_SCOPE=0; service_action status "$unit"
                ;;
            5)
                printf 'Unidad: '; read -r unit || return 0
                printf 'Acción (start/stop/restart): '; read -r operation || return 0
                USER_SCOPE=0; service_action "$operation" "$unit"
                ;;
            6)
                printf 'Unidad: '; read -r unit || return 0
                printf 'Acción (enable/disable/mask/unmask): '; read -r operation || return 0
                USER_SCOPE=0; service_action "$operation" "$unit"
                ;;
            7) USER_SCOPE=0; service_action daemon-reload systemd ;;
            q|Q) return 0 ;;
            *) printf 'Opción no válida.\n' ;;
        esac
    done
}

menu() {
    local choice operation value
    while :; do
        printf '\n=== Administración del sistema ===\n'
        printf '  1) Estado de systemd y servicios fallidos\n'
        printf '  2) Servicios y daemons\n'
        printf '  3) Procesos\n'
        printf '  4) Journal (avisos)\n'
        printf '  q) Volver\n'
        printf 'Elige una opción: '
        read -r choice || return 0
        case "$choice" in
            1) show_status ;;
            2) menu_services ;;
            3)
                while :; do
                    list_processes
                    printf '\nAcción: s=estado, t=terminar, k=forzar, q=volver: '
                    read -r operation || return 0
                    [[ "$operation" == q ]] && break
                    printf 'PID: '; read -r value || return 0
                    case "$operation" in
                        s) process_action status "$value" ;;
                        t) process_action stop "$value" ;;
                        k) process_action kill "$value" ;;
                        *) printf 'Acción no válida.\n' ;;
                    esac
                done
                ;;
            4) show_journal ;;
            q|Q) return 0 ;;
            *) printf 'Opción no válida.\n' ;;
        esac
    done
}

parse_args() {
    while (($#)); do
        case "$1" in
            --user) USER_SCOPE=1; shift ;;
            --unit) [[ $# -ge 2 ]] || die '--unit requiere una unidad'; JOURNAL_UNIT="$2"; shift 2 ;;
            --lines) [[ $# -ge 2 && "$2" =~ ^[0-9]+$ ]] || die '--lines requiere un número'; JOURNAL_LINES="$2"; shift 2 ;;
            --dry-run) DRY_RUN=1; shift ;;
            --plan) [[ $# -ge 2 ]] || die '--plan requiere un fichero'; PLAN_PATH="$2"; shift 2 ;;
            --rollback) [[ $# -ge 2 ]] || die '--rollback requiere un fichero'; ROLLBACK_PATH="$2"; shift 2 ;;
            -h|--help) usage; exit 0 ;;
            --version) printf '%s %s\n' "$SCRIPT_NAME" "$VERSION"; exit 0 ;;
            status|services|user-services|processes|journal|menu) ACTION="$1"; shift ;;
            service|process)
                ACTION="$1"; shift
                [[ $# -ge 2 ]] || die "$ACTION requiere acción y argumento"
                OPERATION="$1"; TARGET="$2"; shift 2
                ;;
            *) die "opción o acción desconocida: $1" ;;
        esac
    done
}

main() {
    local OPERATION="" TARGET=""
    parse_args "$@"
    if [[ -n "$ROLLBACK_PATH" ]]; then
        rollback_plan "$ROLLBACK_PATH"
        exit $?
    fi
    plan_init "$SCRIPT_NAME" || die "no se pudo crear el plan: ${PLAN_PATH:-desconocido}"
    case "$ACTION" in
        menu) menu ;;
        status) show_status ;;
        services) list_services ;;
        user-services) USER_SCOPE=1; list_services ;;
        service) service_action "$OPERATION" "$TARGET" ;;
        processes) list_processes ;;
        process) process_action "$OPERATION" "$TARGET" ;;
        journal) show_journal ;;
    esac
    printf '\nPlan registrado en: %s\n' "$PLAN_PATH"
}

main "$@"
