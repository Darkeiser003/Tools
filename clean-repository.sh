#!/usr/bin/env bash
# Limpieza segura del checkout de LTools.
#
# Por defecto solo muestra el plan. Usa --apply para retirar artefactos
# regenerables del repositorio; nunca usa un limpiador global del checkout.

set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || true)"

die() {
    printf 'ERROR: %s\n' "$1" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Uso: ./clean-repository.sh [opciones]

Limpia únicamente salidas regenerables conocidas del checkout de LTools.
Sin --apply no modifica nada y solo muestra el plan.

Opciones:
  --dry-run       Solo mostrar lo que se limpiaría (predeterminado).
  --apply         Retirar los artefactos regenerables detectados.
  --yes           No pedir confirmación al usar --apply.
  --help          Mostrar esta ayuda.

Se consideran regenerables: dist/, release/, rust/target/, target/,
windows/{target,bin,obj}/, build/, out/, artifacts/, caches de herramientas,
y temporales de AppImage. Los archivos no ignorados por Git nunca se borran.
EOF
}

[[ -n "$ROOT_DIR" ]] || die 'no se encontró la raíz Git del proyecto.'
[[ "$ROOT_DIR" == "$SCRIPT_DIR" ]] || die 'el script debe estar en la raíz del repositorio.'

MODE='dry-run'
ASSUME_YES=0
for arg in "$@"; do
    case "$arg" in
        --dry-run) MODE='dry-run' ;;
        --apply) MODE='apply' ;;
        --yes) ASSUME_YES=1 ;;
        --help|-h) usage; exit 0 ;;
        *) die "opción desconocida: $arg (usa --help)." ;;
    esac
done

# Lista deliberadamente explícita: no se hace una búsqueda destructiva de todo
# lo ignorado. distribution/ y fuentes no
# están aquí porque forman parte del proyecto y sí deben conservarse.
GENERATED_DIRS=(
    'dist'
    'release'
    'rust/target'
    'target'
    'windows/target'
    'windows/bin'
    'windows/obj'
    'build'
    'out'
    'artifacts'
    '.appimage-builder'
    '.pytest_cache'
    'coverage'
    'node_modules'
)

declare -a CANDIDATES=()
declare -a SKIPPED=()

is_safe_child() {
    local path="$1"
    [[ "$path" == "$ROOT_DIR"/* ]]
}

for relative in "${GENERATED_DIRS[@]}"; do
    path="$ROOT_DIR/$relative"
    [[ -e "$path" || -L "$path" ]] || continue
    is_safe_child "$path" || die "ruta de limpieza insegura: $path"

    # Si una fuente llegara a versionarse dentro de una carpeta candidata,
    # se conserva toda esa carpeta y se informa para revisión manual.
    if git -C "$ROOT_DIR" ls-files -- "$relative" | grep -q .; then
        SKIPPED+=("$relative (contiene archivos versionados)")
        continue
    fi
    if [[ -L "$path" ]]; then
        SKIPPED+=("$relative (es un enlace simbólico; no se toca)")
        continue
    fi
    CANDIDATES+=("$relative")
done

# El builder puede dejar un staging con este patrón en la raíz si se
# interrumpe antes de crear dist/. Solo se consideran entradas directas.
while IFS= read -r -d '' path; do
    relative="${path#"$ROOT_DIR/"}"
    if git -C "$ROOT_DIR" ls-files -- "$relative" | grep -q .; then
        SKIPPED+=("$relative (archivo versionado)")
    elif [[ -L "$path" ]]; then
        SKIPPED+=("$relative (es un enlace simbólico; no se toca)")
    else
        CANDIDATES+=("$relative")
    fi
done < <(find "$ROOT_DIR" -mindepth 1 -maxdepth 1 -name '.ltools-build.*' -print0)

printf 'LTools — limpieza del repositorio (%s)\n' "$MODE"
printf 'Raíz: %s\n\n' "$ROOT_DIR"

if ((${#CANDIDATES[@]} == 0)); then
    printf 'No se detectaron artefactos regenerables para retirar.\n'
else
    printf 'Candidatos regenerables (%d):\n' "${#CANDIDATES[@]}"
    for relative in "${CANDIDATES[@]}"; do
        path="$ROOT_DIR/$relative"
        printf '  %-24s %s\n' "$relative" "$(du -sh -- "$path" 2>/dev/null | awk '{print $1}' || printf '?')"
    done
fi

if ((${#SKIPPED[@]} > 0)); then
    printf '\nProtegidos (requieren revisión manual):\n'
    printf '  %s\n' "${SKIPPED[@]}"
fi

printf '\nNo se borran fuentes, documentación, tests, Cargo.lock ni archivos no ignorados por Git.\n'
if [[ "$MODE" != 'apply' ]]; then
    printf 'Simulación terminada; no se ha modificado ningún archivo.\n'
else
    if ((${#CANDIDATES[@]} == 0)); then
        exit 0
    fi
    if ((ASSUME_YES == 0)); then
        printf '¿Retirar estos artefactos regenerables? [y/N] '
        read -r answer
        [[ "$answer" =~ ^([yY][eE][sS]|[yY])$ ]] || { printf 'Limpieza cancelada.\n'; exit 0; }
    fi
    for relative in "${CANDIDATES[@]}"; do
        path="$ROOT_DIR/$relative"
        [[ -d "$path" || -f "$path" ]] || die "el candidato dejó de existir o no es regular: $path"
        case "$path" in
            "$ROOT_DIR"/*) ;;
            *) die "protección de ruta activada: $path" ;;
        esac
        printf 'Retirando: %s\n' "$relative"
        rm -rf -- "$path"
    done
    printf 'Limpieza terminada. Los artefactos se regeneran con ./build.sh.\n'
fi

untracked="$(git -C "$ROOT_DIR" ls-files --others --exclude-standard || true)"
if [[ -n "$untracked" ]]; then
    printf '\nArchivos no ignorados que se conservan para revisión (no se borran):\n'
    printf '%s\n' "$untracked" | sed -n '1,80p'
fi
