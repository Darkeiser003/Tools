#!/usr/bin/env bash
# Punto de entrada único para ejecutar, probar, compilar y limpiar LTools.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
source "$ROOT_DIR/scripts/lib/publish.sh"
source "$ROOT_DIR/scripts/lib/signing.sh"
source "$ROOT_DIR/scripts/lib/ssh-signing.sh"
source "$ROOT_DIR/scripts/lib/third-party-licenses.sh"
source "$ROOT_DIR/scripts/lib/temp-clean.sh"
LINUX_BUILDER="$ROOT_DIR/scripts/build.sh"
CLEANER="$ROOT_DIR/scripts/build.sh"
MANIFEST="$ROOT_DIR/rust/Cargo.toml"
RELEASE_BIN="$ROOT_DIR/rust/target/release/ltools"
DEBUG_BIN="$ROOT_DIR/rust/target/debug/ltools"

usage() {
    cat <<'EOF'
Uso: bash scripts/build.sh [opciones del builder Linux]
     bash scripts/build.sh clean [--dry-run|--apply] [--keep-release] [--plans-only] [--tmp]
                                 [--only targets|dist|release|all|build-tmp]

Sin opciones abre el menú Preview, Pruebas, Build y Limpieza.
Usa --help para ver todas las opciones del pipeline Linux.
EOF
}

project_path_has_symlink_ancestor() {
    local candidate="$1"
    [[ "$candidate" == "$ROOT_DIR/"* ]] || return 0
    while [[ "$candidate" != "$ROOT_DIR" ]]; do
        [[ ! -L "$candidate" ]] || return 0
        candidate="${candidate%/*}"
    done
    return 1
}

project_cleanup_path_is_safe() {
    local resolved
    project_path_has_symlink_ancestor "$1" && return 1
    resolved="$(realpath -e -- "$1" 2>/dev/null)" || return 1
    [[ "$resolved" == "$1" ]]
}

clean_repo() {
    local mode='dry-run' assume_yes=0 include_plans=0 plans_only=0 keep_release=0 include_tmp=0 only='all'
    local arg relative path name size temp_root temp_status
    local -a generated=() candidates=() skipped=()
    while (($#)); do
        case "$1" in
            --dry-run) mode='dry-run' ;;
            --apply) mode='apply' ;;
            --plans) include_plans=1 ;;
            --plans-only) include_plans=1; plans_only=1 ;;
            --tmp) include_tmp=1 ;;
            --keep-release) keep_release=1 ;;
            --yes) assume_yes=1 ;;
            --only) (($# >= 2)) || { printf 'clean: --only necesita targets, dist, release, build-tmp o all.\n' >&2; return 2; }; only="$2"; shift ;;
            -h|--help) printf 'clean: --dry-run (predeterminado), --apply, --keep-release, --plans, --plans-only, --tmp (alias), --yes, --only targets|dist|release|build-tmp|all\n'; return 0 ;;
            *) printf 'Opción de limpieza desconocida: %s\n' "$1" >&2; return 2 ;;
        esac
        shift
    done
    case "$only" in all|targets|dist|release|build-tmp|tmp) ;; *) printf 'Grupo de limpieza no válido: %s\n' "$only" >&2; return 2 ;; esac
    # "tmp" se conserva como alias histórico; el nombre nuevo deja claro que
    # solo se retiran temporales de build/pruebas identificados por allowlist.
    [[ "$only" != tmp ]] || only='build-tmp'
    if [[ "$only" == build-tmp ]]; then include_tmp=1; fi
    if [[ "$only" == build-tmp && ( "$plans_only" == 1 || "$include_plans" == 1 ) ]]; then
        printf 'clean: --only build-tmp no se combina con --plans.\n' >&2
        return 2
    fi
    if ((plans_only == 0)) && [[ "$only" != build-tmp ]]; then
        if ! command -v git >/dev/null 2>&1 ||
            ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1 ||
            ! git -C "$ROOT_DIR" ls-files >/dev/null 2>&1 ||
            ! git -C "$ROOT_DIR" ls-files --others --exclude-standard >/dev/null 2>&1; then
            printf 'Limpieza cancelada: Git no puede verificar qué archivos del proyecto deben protegerse.\n' >&2
            return 1
        fi
    fi
    if ((plans_only == 0)) && [[ "$only" != build-tmp ]]; then
        case "$only" in
            all) generated=(dist rust/dist release rust/target fuzz/target fuzz/artifacts reports target windows/target windows/bin windows/obj build out artifacts .appimage-builder .pytest_cache coverage node_modules) ;;
            targets) generated=(rust/target fuzz/target target windows/target windows/bin windows/obj) ;;
            dist) generated=(dist rust/dist) ;;
            release) generated=(release) ;;
        esac
        for relative in "${generated[@]}"; do
            [[ "$relative" == release && "$keep_release" -eq 1 ]] && { skipped+=("release (conservado)"); continue; }
            path="$ROOT_DIR/$relative"
            [[ -e "$path" || -L "$path" ]] || continue
            if ! project_cleanup_path_is_safe "$path"; then
                skipped+=("$relative (ruta atravesada por enlace simbólico)")
                continue
            fi
            if git -C "$ROOT_DIR" ls-files -- "$relative" | grep -q . ||
                git -C "$ROOT_DIR" ls-files --others --exclude-standard -- "$relative" | grep -q .; then
                skipped+=("$relative (contiene archivos protegidos por Git)"); continue
            fi
            if [[ -L "$path" ]]; then skipped+=("$relative (es un enlace simbólico)"); continue; fi
            candidates+=("$relative")
        done
        while IFS= read -r -d '' path; do
            relative="${path#"$ROOT_DIR/"}"
            if ! project_cleanup_path_is_safe "$path" ||
                git -C "$ROOT_DIR" ls-files -- "$relative" | grep -q . ||
                git -C "$ROOT_DIR" ls-files --others --exclude-standard -- "$relative" | grep -q . || [[ -L "$path" ]]; then
                skipped+=("$relative (protegido por Git o enlace simbólico)")
            else candidates+=("$relative"); fi
        done < <(find "$ROOT_DIR" -mindepth 1 -maxdepth 1 -name '.ltools-build.*' -print0)
        for pattern in 'rust-audit-*' 'rust-games-*' 'rust-package-audit-*'; do
            for path in "$ROOT_DIR"/$pattern "$ROOT_DIR/rust"/$pattern; do
                [[ -d "$path" && ! -L "$path" ]] || continue
                relative="${path#"$ROOT_DIR/"}"
                if ! project_cleanup_path_is_safe "$path" ||
                    git -C "$ROOT_DIR" ls-files -- "$relative" | grep -q . ||
                    git -C "$ROOT_DIR" ls-files --others --exclude-standard -- "$relative" | grep -q .; then
                    skipped+=("$relative (contiene archivos protegidos por Git)")
                else candidates+=("$relative"); fi
            done
        done
    fi
    if ((include_tmp == 1)); then
        for temp_root in /tmp /var/tmp; do
            [[ -d "$temp_root" && ! -L "$temp_root" ]] || continue
            if [[ "$(realpath -e -- "$temp_root" 2>/dev/null)" != "$temp_root" ]]; then
                skipped+=("$temp_root (raíz temporal no canónica o enlace simbólico)")
                continue
            fi
            while IFS= read -r -d '' path; do
                name="${path##*/}"
                ltools_temp_name_is_owned "$name" || continue
                if ltools_temp_candidate_check "$path" "$temp_root"; then
                    candidates+=("$path")
                else
                    temp_status=$?
                    case "$temp_status" in
                        2) skipped+=("$path (pertenece a otro usuario)") ;;
                        3) skipped+=("$path (parece estar en uso)") ;;
                        *) skipped+=("$path (enlace, tipo o ruta no segura)") ;;
                    esac
                fi
            done < <(find "$temp_root" -mindepth 1 -maxdepth 1 -print0)
        done
    fi
    local plans_dir="${XDG_STATE_HOME:-$HOME/.local/state}/ltools/plans"
    if ((include_plans == 1)) && [[ -d "$plans_dir" ]]; then
        while IFS= read -r -d '' path; do
            name="${path##*/}"
            [[ "$name" =~ ^plan-([0-9]{6,}-|-[0-9]+-|[0-9]+-[0-9]+-).*\.tsv$ ]] && candidates+=("$path")
        done < <(find "$plans_dir" -maxdepth 1 -type f -name 'plan-*.tsv' -print0)
    fi
    printf 'LTools — limpieza (%s)\nRaíz: %s\n\n' "$mode" "$ROOT_DIR"
    if ((${#candidates[@]} == 0)); then printf 'No se detectaron artefactos regenerables para retirar.\n'; else
        printf 'Candidatos regenerables (%d):\n' "${#candidates[@]}"
        for relative in "${candidates[@]}"; do
            path="$relative"; [[ "$relative" == /* ]] || path="$ROOT_DIR/$relative"
            size="$(du -sh -- "$path" 2>/dev/null | awk '{print $1}' || printf '?')"
            printf '  %-72s %s\n' "${relative/#$HOME/~}" "$size"
        done
    fi
    if ((${#skipped[@]} > 0)); then printf '\nProtegidos:\n'; printf '  %s\n' "${skipped[@]}"; fi
    printf '\nNo se borran fuentes, documentación, tests, Cargo.lock ni archivos no ignorados por Git.\n'
    if ((include_tmp == 1)); then
        printf 'Temporales: solo formatos allowlist de build/E2E de LTools en /tmp y /var/tmp; se excluyen artefactos ajenos, otras rutas y usuarios.\n'
    fi
    if [[ "$mode" != apply ]]; then printf 'Simulación terminada; no se ha modificado ningún archivo.\n'; return 0; fi
    ((${#candidates[@]} > 0)) || return 0
    if ((assume_yes == 0)); then
        printf '¿Retirar estos artefactos regenerables? [y/N] '
        read -r answer || answer=''
        [[ "$answer" =~ ^([yY][eE][sS]|[yY])$ ]] || { printf 'Limpieza cancelada.\n'; return 0; }
    fi
    for relative in "${candidates[@]}"; do
        if [[ "$relative" == /* ]]; then
            case "$relative" in
                "$plans_dir"/plan-*.tsv) rm -f -- "$relative" ;;
                /tmp/*|/var/tmp/*)
                    temp_root="${relative%/*}"
                    if ! ltools_temp_candidate_check "$relative" "$temp_root"; then
                        printf 'No se retira un temporal que cambió o ya no es seguro: %s\n' "$relative" >&2
                        return 1
                    fi
                    printf 'Retirando temporal propio: %s\n' "$relative"
                    if [[ -d "$relative" ]]; then rm -rf -- "$relative"; else rm -f -- "$relative"; fi
                    ;;
                *) printf 'Ruta temporal o de plan protegida: %s\n' "$relative" >&2; return 1 ;;
            esac
        else
            path="$ROOT_DIR/$relative"
            case "$path" in "$ROOT_DIR"/*) ;; *) printf 'Ruta insegura: %s\n' "$path" >&2; return 1 ;; esac
            project_cleanup_path_is_safe "$path" || {
                printf 'No se retira una ruta atravesada por enlaces simbólicos: %s\n' "$relative" >&2
                return 1
            }
            printf 'Retirando: %s\n' "$relative"
            rm -rf -- "$path"
        fi
    done
    printf 'Limpieza terminada.\n'
}

pause() {
    [[ -t 0 ]] || return 0
    printf '\nPulsa Enter para volver al menú...'
    IFS= read -r _ || true
}

run_and_pause() {
    printf '\n$'
    printf ' %q' "$@"
    printf '\n\n'
    if "$@"; then
        printf '\nTarea terminada correctamente.\n'
    else
        local status=$?
        printf '\nLa tarea terminó con código %s.\n' "$status" >&2
    fi
    pause
    return 0
}

run_suite_test() {
    local label="$1" status
    shift
    printf '\n==> %s\n' "$label"
    if "$@"; then
        printf '  OK    %s\n' "$label"
        return 0
    else
        status=$?
        printf '  ERROR %s (código %s)\n' "$label" "$status" >&2
        return "$status"
    fi
}

run_all_e2e() {
    local failures=0 selected captures
    run_suite_test 'Smoke del backend release' "$ROOT_DIR/tests/linux/smoke.sh" --require-gui --binary "$RELEASE_BIN" || failures=$((failures + 1))
    run_suite_test 'CLI distribuible y contratos' "$ROOT_DIR/tests/linux/cli-e2e.sh" --binary "$RELEASE_BIN" || failures=$((failures + 1))
    run_suite_test 'Migración y rollback' "$ROOT_DIR/tests/linux/e2e.sh" --require-dependencies --binary "$RELEASE_BIN" || failures=$((failures + 1))
    run_suite_test 'Menús y acciones' "$ROOT_DIR/tests/linux/menu-e2e.sh" --require-gui --binary "$RELEASE_BIN" || failures=$((failures + 1))
    run_suite_test 'Stores y Git' "$ROOT_DIR/tests/linux/software-git-e2e.sh" --binary "$RELEASE_BIN" || failures=$((failures + 1))
    mkdir -p -- "$ROOT_DIR/dist"
    captures="$(mktemp -d "$ROOT_DIR/dist/storage-map-gui.XXXXXX")"
    run_suite_test 'Acciones reales del mapa GUI' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" \
        --require-gui --binary "$RELEASE_BIN" --tmp "$captures" --captures "$captures/captures" || failures=$((failures + 1))
    run_suite_test 'Ayudas nativas y correspondencia GUI' "$ROOT_DIR/tests/linux/native-help-e2e.sh" --binary "$RELEASE_BIN" || failures=$((failures + 1))
    if selected="$(choose_artifact 'ltools-*.tar.gz' 'tarball Linux')"; then
        run_suite_test 'Tarball extraído' "$ROOT_DIR/tests/linux/tarball-e2e.sh" --tarball "$selected" --require-gui || failures=$((failures + 1))
    else
        printf '  SKIP  no hay tarball Linux disponible\n'
    fi
    if ((failures)); then
        printf '\n%s E2E fallaron; revisa los errores anteriores.\n' "$failures" >&2
        return 1
    fi
    printf '\nTodas las E2E disponibles finalizaron correctamente.\n'
    return 0
}

choose_artifact() {
    local pattern="$1" label="$2" selected
    local -a candidates=()
    while IFS= read -r -d '' selected; do candidates+=("$selected"); done \
        < <(find "$ROOT_DIR/dist" "$ROOT_DIR/release" -maxdepth 1 -type f -name "$pattern" -print0 2>/dev/null | sort -zV -r)
    if ((${#candidates[@]} == 0)); then
        printf 'No hay %s existentes en dist/ ni release/.\n' "$label" >&2
        return 1
    fi
    if ((${#candidates[@]} == 1)); then
        printf '%s\n' "${candidates[0]}"
        return 0
    fi
    printf '\n%s disponibles:\n' "$label" >&2
    local index=1
    for selected in "${candidates[@]}"; do printf '  %s) %s\n' "$index" "${selected##*/}" >&2; index=$((index + 1)); done
    printf '  0) Volver\n' >&2
    read -r -p 'Elige uno: ' index || return 1
    [[ "$index" =~ ^[1-9][0-9]*$ ]] || return 1
    ((index <= ${#candidates[@]})) || return 1
    printf '%s\n' "${candidates[index-1]}"
}

preview_menu() {
    while true; do
        printf '\nLTools — Preview y ejecución\n'
        printf '  1) Abrir la GUI release ya compilada\n'
        printf '  2) Abrir la GUI en modo desarrollo (Cargo puede compilar cambios)\n'
        printf '  3) Mostrar la ayuda CLI del backend existente\n'
        printf '  4) Probar el perfil CLI aislado\n'
        printf '  5) Ejecutar el AppImage GUI disponible\n'
        printf '  6) Mostrar artefactos disponibles\n'
        printf '  7) Preview GUI vigilado y aislado (recarga al cambiar código)\n'
        printf '  0) Volver\n'
        read -r -p 'Selecciona una opción: ' choice || return
        case "$choice" in
            1)
                if [[ -x "$RELEASE_BIN" ]]; then run_and_pause env LTOOLS_BINARY="$RELEASE_BIN" "$ROOT_DIR/ltools.sh"
                else printf 'No existe %s. Compila el backend release desde Build.\n' "$RELEASE_BIN"; pause; fi ;;
            2) run_and_pause cargo run --manifest-path "$MANIFEST" -- ;;
            3)
                if [[ -x "$RELEASE_BIN" || -x "$DEBUG_BIN" ]]; then run_and_pause "$ROOT_DIR/ltools-cli.sh" --help
                else printf 'No hay backend compilado. Usa Build → Compilar backend.\n'; pause; fi ;;
            4)
                CLI_PREVIEW_BIN="$ROOT_DIR/rust/target/cli-preview/release/ltools"
                if [[ -x "$CLI_PREVIEW_BIN" ]]; then run_and_pause env LTOOLS_BINARY="$CLI_PREVIEW_BIN" "$ROOT_DIR/ltools-cli.sh" --help
                else printf 'No hay perfil CLI aislado. Usa Build → Compilar el perfil CLI.\n'; pause; fi ;;
            5)
                if selected="$(choose_artifact 'ltools-*.AppImage' 'AppImages')"; then
                    case "$selected" in
                        *-cli.AppImage)
                            printf 'Ese es el perfil CLI. Elige un AppImage GUI.\n'
                            pause
                            ;;
                        *)
                            if [[ -c /dev/fuse ]] &&
                                { command -v fusermount3 >/dev/null 2>&1 || command -v fusermount >/dev/null 2>&1; }; then
                                run_and_pause "$selected"
                            else
                                printf 'FUSE no está disponible; se ejecutará mediante la extracción oficial del AppImage.\n'
                                run_and_pause env APPIMAGE_EXTRACT_AND_RUN=1 "$selected"
                            fi
                            ;;
                    esac
                else pause; fi ;;
            6) run_and_pause bash -c 'find "$1" "$2" -maxdepth 1 -type f \( -name "ltools-*" -o -name "SHA256SUMS.txt*" \) -printf "%p\n" 2>/dev/null || true' _ "$ROOT_DIR/dist" "$ROOT_DIR/release" ;;
            7) run_and_pause bash "$ROOT_DIR/scripts/live-preview.sh" ;;
            0|'') return ;;
            *) printf 'Opción no válida.\n'; pause ;;
        esac
    done
}

test_menu() {
    while true; do
        printf '\nLTools — Pruebas sin volver a compilar\n'
        printf '  1) Smoke del backend release existente\n'
        printf '  2) E2E aislada de migración y rollback\n'
        printf '  3) E2E de menús GUI y flujos funcionales\n'
        printf '  4) E2E de stores simuladas y Git\n'
        printf '  5) Contratos y codificación de scripts\n'
        printf '  6) Ejecutar todas las E2E Linux disponibles\n'
        printf '  7) E2E de ayudas nativas\n'
        printf '  8) E2E del tarball Linux\n'
        printf '  9) E2E de acciones reales del mapa GUI\n'
        printf ' 10) E2E de la CLI distribuible\n'
        printf '  0) Volver\n'
        read -r -p 'Selecciona una opción: ' choice || return
        if [[ "$choice" =~ ^[1-4]$ || "$choice" == 6 || "$choice" == 7 || "$choice" == 9 || "$choice" == 10 ]]; then
            if [[ ! -x "$RELEASE_BIN" ]]; then printf 'No existe el backend release. Estas pruebas usan un binario ya compilado.\n'; pause; continue; fi
        fi
        case "$choice" in
            1) run_and_pause "$ROOT_DIR/tests/linux/smoke.sh" --require-gui --binary "$RELEASE_BIN" ;;
            2) run_and_pause "$ROOT_DIR/tests/linux/e2e.sh" --require-dependencies --binary "$RELEASE_BIN" ;;
            3) run_and_pause "$ROOT_DIR/tests/linux/menu-e2e.sh" --require-gui --binary "$RELEASE_BIN" ;;
            4) run_and_pause "$ROOT_DIR/tests/linux/software-git-e2e.sh" --binary "$RELEASE_BIN" ;;
            5) run_and_pause bash -c 'set -e; "$1"; "$2"; bash "$3"; bash "$4"' _ "$ROOT_DIR/tests/contracts.sh" "$ROOT_DIR/tests/encoding.sh" "$ROOT_DIR/tests/scripts-syntax.sh" "$ROOT_DIR/tests/temp-cleaner.sh" ;;
            6) if run_all_e2e; then :; else printf 'Una o más E2E fallaron; revisa los resultados anteriores.\n' >&2; fi; pause ;;
            7) run_and_pause "$ROOT_DIR/tests/linux/native-help-e2e.sh" --binary "$RELEASE_BIN" ;;
            8)
                if selected="$(choose_artifact 'ltools-*.tar.gz' 'tarball Linux')"; then
                    run_and_pause "$ROOT_DIR/tests/linux/tarball-e2e.sh" --tarball "$selected" --require-gui
                else pause; fi ;;
            9)
                mkdir -p -- "$ROOT_DIR/dist"
                captures="$(mktemp -d "$ROOT_DIR/dist/storage-map-gui.XXXXXX")"
                run_and_pause "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" \
                    --require-gui --binary "$RELEASE_BIN" --tmp "$captures" --captures "$captures/captures" ;;
            10) run_and_pause "$ROOT_DIR/tests/linux/cli-e2e.sh" --binary "$RELEASE_BIN" ;;
            0|'') return ;;
            *) printf 'Opción no válida.\n'; pause ;;
        esac
    done
}

build_menu() {
    while true; do
        printf '\nLTools — Build\n'
        printf '  1) Compilar solo el backend GUI (release)\n'
        printf '  2) Compilar el perfil CLI en un target aislado\n'
        printf '  3) Validar backend: formato, Clippy, tests y contratos\n'
        printf '  4) Crear tarball Linux local\n'
        printf '  5) Crear AppImages Linux locales\n'
        printf '  6) Crear tarball y AppImages Linux locales\n'
        printf '  7) Release Linux firmada y validada (incluye Wine/Proton)\n'
        printf '  8) Release Linux firmada sin etapa Wine/Proton\n'
        printf '  9) Build rápida de desarrollo: sin paquetes, tests, smoke, E2E ni Wine\n'
        printf '  0) Volver\n'
        read -r -p 'Selecciona una opción: ' choice || return
        case "$choice" in
            1) run_and_pause cargo build --manifest-path "$MANIFEST" --release ;;
            2)
                run_and_pause env CARGO_TARGET_DIR="$ROOT_DIR/rust/target/cli-preview" cargo build --manifest-path "$MANIFEST" --features cli --release ;;
            3)
                run_and_pause bash -c 'set -e; cargo fmt --manifest-path "$1" --all -- --check; cargo clippy --manifest-path "$1" --workspace --all-targets -- -D warnings; cargo test --manifest-path "$1" --workspace --all-targets; "$2"; "$3"; bash "$4"' _ \
                    "$MANIFEST" "$ROOT_DIR/tests/contracts.sh" "$ROOT_DIR/tests/encoding.sh" "$ROOT_DIR/tests/scripts-syntax.sh" ;;
            4) run_and_pause "$LINUX_BUILDER" --non-interactive --no-appimage --no-windows-wine --allow-unsigned --output "$ROOT_DIR/dist/local" --release-dir "$ROOT_DIR/dist/local-release" ;;
            5) run_and_pause "$LINUX_BUILDER" --non-interactive --appimage --no-package --no-windows-wine --allow-unsigned --output "$ROOT_DIR/dist/local" --release-dir "$ROOT_DIR/dist/local-release" ;;
            6) run_and_pause "$LINUX_BUILDER" --non-interactive --no-windows-wine --allow-unsigned --output "$ROOT_DIR/dist/local" --release-dir "$ROOT_DIR/dist/local-release" ;;
            7) run_and_pause "$LINUX_BUILDER" --non-interactive --appimage --windows-wine ;;
            8) run_and_pause "$LINUX_BUILDER" --non-interactive --appimage --no-windows-wine ;;
            9) run_and_pause "$LINUX_BUILDER" --non-interactive --fast --no-tests --no-package --no-appimage --no-windows-wine --no-smoke --no-e2e --no-menu-e2e --no-software-git-e2e --allow-unsigned ;;
            0|'') return ;;
            *) printf 'Opción no válida.\n'; pause ;;
        esac
    done
}

clean_menu() {
    while true; do
        printf '\nLTools — Limpiar artefactos\n'
        printf '  1) Revisar qué se puede limpiar (simulación)\n'
        printf '  2) Limpiar solo los artefactos de compilación Rust\n'
        printf '  3) Limpiar staging y cachés, conservando release/\n'
        printf '  4) Limpiar salidas regenerables, incluida release/\n'
        printf '  5) Revisar planes legacy fuera del repositorio\n'
        printf '  6) Retirar planes legacy revisados\n'
        printf '  7) Revisar temporales de build/pruebas LTools (simulación)\n'
        printf '  8) Retirar temporales LTools confirmados en /tmp y /var/tmp\n'
        printf '  0) Volver\n'
        read -r -p 'Selecciona una opción: ' choice || return
        case "$choice" in
            1) run_and_pause "$CLEANER" clean --dry-run ;;
            2)
                read -r -p 'Esto elimina rust/target. ¿Continuar? [s/N] ' answer || answer=''
                case "${answer,,}" in
                    s|si|sí|y|yes)
                        if project_path_has_symlink_ancestor "$ROOT_DIR/rust/target"; then
                            printf 'Limpieza cancelada: rust/target o uno de sus directorios padre es un enlace simbólico.\n' >&2
                            pause
                        else
                            run_and_pause env CARGO_TARGET_DIR="$ROOT_DIR/rust/target" cargo clean --manifest-path "$MANIFEST"
                        fi ;;
                    *) printf 'Cancelado.\n'; pause ;;
                esac ;;
            3)
                run_and_pause "$CLEANER" clean --apply --keep-release ;;
            4)
                run_and_pause "$CLEANER" clean --apply ;;
            5) run_and_pause "$CLEANER" clean --plans-only --dry-run ;;
            6)
                run_and_pause "$CLEANER" clean --plans-only --apply ;;
            7) run_and_pause "$CLEANER" clean --only build-tmp --dry-run ;;
            8) run_and_pause "$CLEANER" clean --only build-tmp --apply ;;
            0|'') return ;;
            *) printf 'Opción no válida.\n'; pause ;;
        esac
    done
}

main_menu() {
    while true; do
        printf '\nLTools — Desarrollo y distribución\n'
        printf '  1) Preview y ejecutar\n'
        printf '  2) Probar lo ya compilado (sin recompilar)\n'
        printf '  3) Build y distribución\n'
        printf '  4) Limpiar artefactos\n'
        printf '  5) Ver ayuda de las opciones avanzadas\n'
        printf '  0) Salir\n'
        read -r -p 'Selecciona una opción: ' choice || return
        case "$choice" in
            1) preview_menu ;;
            2) test_menu ;;
            3) build_menu ;;
            4) clean_menu ;;
            5) run_and_pause "$LINUX_BUILDER" --help ;;
            0|'') return ;;
            *) printf 'Opción no válida.\n'; pause ;;
        esac
    done
}

if (($# == 0)) || [[ "${1:-}" == --menu ]]; then
    [[ "${1:-}" == --menu ]] && shift
    if [[ -t 0 && -t 1 ]]; then main_menu; else usage >&2; exit 2; fi
    exit 0
fi
if [[ "$1" == clean ]]; then shift; clean_repo "$@"; exit $?; fi
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
COMPONENT='all'
TEST_EXISTING=''
PREVIEW=0
CHECKS=1
STRICT_SECURITY=0
SECURITY_REVIEW=0
AUTO_FIX=0
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
# La validación Windows bajo Wine/Proton forma parte del build completo por
# defecto. Puede desactivarse explícitamente con --no-windows-wine o con "n".
WINDOWS_WINE=1
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
WINDOWS_WINE_CLI_ARTIFACT=""
WINDOWS_WINE_LOG=""
CLI_ONLY=0
# Las publicaciones se firman por defecto. La excepción local debe ser
# explícita mediante --allow-unsigned o LTOOLS_ALLOW_UNSIGNED=1.
SIGNING_REQUIRED=1
SIGNING_PRIVATE_KEY_FILE=""
SIGNING_PUBLIC_KEY_FILE=""
SIGNING_PUBLIC_KEY_ENV_ACTIVE=0
SSH_SIGNING_AVAILABLE=0
STEP_STARTED=$SECONDS
STEP_ACTIVE=0
# init_logging redirige stdout a tee; conserva antes el estado real de la
# terminal para mantener separadas las preguntas del pipeline y el menú
# interactivo de desarrollo de la parte superior.
INTERACTIVE_TTY=0
if [[ -t 0 && -t 1 ]]; then
    INTERACTIVE_TTY=1
fi

ok() { printf '    \033[32mOK:\033[0m %s\n' "$1"; }
warn() { printf '    \033[33mAVISO:\033[0m %s\n' "$1" >&2; }
die() { printf '    \033[31mERROR:\033[0m %s\n' "$1" >&2; exit 1; }

validate_output_paths() {
    command -v realpath >/dev/null 2>&1 || die 'realpath es necesario para validar rutas de salida seguras'
    local root_real output_real release_real tmp_real system_tmp_real var_tmp_real
    root_real="$(realpath -e -- "$ROOT_DIR")" || die "no se pudo resolver la raíz del proyecto: $ROOT_DIR"
    output_real="$(realpath -m -- "$OUTPUT_DIR")" || die "no se pudo resolver la salida: $OUTPUT_DIR"
    release_real="$(realpath -m -- "$RELEASE_DIR")" || die "no se pudo resolver la release: $RELEASE_DIR"
    tmp_real="$(realpath -m -- "${TMPDIR:-/tmp}")" || die 'no se pudo resolver TMPDIR'
    system_tmp_real="$(realpath -m -- /tmp)" || die 'no se pudo resolver /tmp'
    var_tmp_real="$(realpath -m -- /var/tmp)" || die 'no se pudo resolver /var/tmp'

    if [[ "$tmp_real" == "$root_real" || "$tmp_real" == "$root_real/"* ]]; then
        die "TMPDIR no puede estar dentro del repositorio: $tmp_real (los tests crean repositorios Git temporales; usa /tmp u otra ruta externa)"
    fi

    for entry in "output:$output_real" "release:$release_real"; do
        local label="${entry%%:*}" candidate="${entry#*:}"
        [[ "$candidate" != / ]] || die "la ruta de $label no puede ser la raíz del sistema"
        [[ "$candidate" != "$root_real" ]] || die "la ruta de $label no puede ser la raíz del proyecto"
        [[ "$root_real" != "$candidate/"* ]] || die "la ruta de $label no puede contener la raíz del proyecto: $candidate"
        [[ "$candidate" != "$tmp_real" && "$candidate" != "$system_tmp_real" && "$candidate" != "$var_tmp_real" ]] || die "la ruta de $label no puede ser una carpeta temporal compartida: $candidate"
    done
    if [[ "$output_real" == "$release_real" || "$output_real" == "$release_real/"* || "$release_real" == "$output_real/"* ]]; then
        die "las rutas de staging y publicación no pueden coincidir ni contenerse entre sí: output=$output_real release=$release_real"
    fi
    # Todas las fases pueden cambiar de directorio. Conserva rutas absolutas
    # para que los staging temporales, CARGO_HOME y los logs no se resuelvan
    # accidentalmente relativos al subdirectorio actual.
    OUTPUT_DIR="$output_real"
    RELEASE_DIR="$release_real"
}

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

rust_source_hashes() {
    find "$ROOT_DIR/rust" -type f -name '*.rs' -not -path "$ROOT_DIR/rust/target/*" -print0 |
        sort -z | xargs -0 -r sha256sum
}

report_rust_autofix_changes() {
    local before="$1" after changed path
    after="$(rust_source_hashes)"
    changed="$(diff -u <(printf '%s\n' "$before") <(printf '%s\n' "$after") 2>/dev/null || true)"
    changed="$(sed -nE 's/^[+-][0-9a-f]{64}  (.*)$/\1/p' <<<"$changed" | sort -u)"
    if [[ -z "$changed" ]]; then
        ok '[AUTO-FIX] no persistieron cambios de archivo.'
        return
    fi
    while IFS= read -r path; do
        [[ -n "$path" ]] && ok "[AUTO-FIXED] ${path#"$ROOT_DIR"/}"
    done <<<"$changed"
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
  --strict-security   Exige cargo-audit/cargo-deny y ShellCheck/actionlint; reescanea.
  --security-review   Exige ShellCheck y actionlint para scripts y workflows.
  --auto-fix           Corrige formato Rust y sugerencias mecánicas de Clippy; luego reescanea.
  --no-tests           No ejecuta cargo test.
  --no-smoke           No ejecuta los smoke tests posteriores al empaquetado.
  --no-e2e             Omite las E2E de migración, menús/mapa y stores/Git.
  --no-menu-e2e        Omite la E2E de menús, mapa GUI, capturas y ayudas nativas.
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
  --component NAME     Ejecuta solo backend|frontend|cli|tarball|appimage|windows o todo.
  --test-existing PATH Ejecuta la matriz aplicable sobre un binario, AppImage o tarball ya creado.
  --preview            Abre el preview GUI vigilado en un target aislado, sin empaquetar.
  --no-package         Compila, pero no genera el tar.gz.
  --appimage           Exige y genera el AppImage.
  --no-appimage        No genera el AppImage.
  --require-fuse       Falla si el equipo no puede montar AppImages con FUSE.
  --output DIR         Directorio de salida (por defecto: ./dist).
  --release-dir DIR    Carpeta canónica de publicación (por defecto: ./release).
  --require-signing    Exige firmas Ed25519 y OpenSSH válidas para release/ (predeterminado).
  --allow-unsigned     Excepción explícita: permite una release local sin firma.
  --jobs N             Paralelismo de Cargo (por defecto: 2).
  --log FICHERO        Guarda la transcripción completa en esta ruta.
  --no-log             Desactiva el log persistente y la tabla de tiempos.
  --non-interactive    No muestra preguntas de configuración.
  --no-run             Alias de compatibilidad; no se ejecuta la aplicación.
  -h, --help           Muestra esta ayuda.
  --workflow-help      Alias de --help para el pipeline Linux.
  --version            Muestra la versión del proyecto.

La build AppImage genera un perfil terminal, un perfil CLI y
ltools-terminal.json para integradores de terminal, además de
ltools-release.json, SHA256SUMS.txt, la firma Ed25519 para el actualizador y
la firma OpenSSH para la procedencia de la release. Usa --allow-unsigned solo
para una build local que no vaya a publicarse.

Sin argumentos, scripts/build.sh abre el menú de tareas. Para ejecutar el
pipeline Linux directamente, indica una o más opciones; con ellas no se hacen
preguntas interactivas.

Componentes: backend y frontend compilan el binario GUI; cli compila el
perfil CLI aislado; tarball y appimage empaquetan solo ese formato; windows
compila el ejecutable Windows mediante la etapa Wine/Proton; todo ejecuta el
pipeline completo. --test-existing no recompila.
EOF
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "falta la herramienta «$1» en PATH"
}

load_signing_material() {
    local config_home update_key_value
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
    # La clave pública no es secreta: se fija dentro de los binarios firmados
    # para que las instalaciones publicadas puedan verificar futuras releases
    # sin configuración manual. build.rs la normaliza y Cargo invalida el
    # artefacto cuando cambia; nunca se imprime su contenido en el log.
    if update_key_value="$(ltools_effective_update_public_key)"; then
        SIGNING_PUBLIC_KEY_ENV_ACTIVE=1
        export LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY="$update_key_value"
    elif [[ -r "$SIGNING_PUBLIC_KEY_FILE" ]]; then
        export LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY="$(tr -d '[:space:]' <"$SIGNING_PUBLIC_KEY_FILE")"
    else
        unset LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY
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
    if ltools_ssh_signing_config; then
        SSH_SIGNING_AVAILABLE=1
    fi
}

release_signature_args() {
    RELEASE_SIGNATURE_ARGS=(release-signature --manifest "$RELEASE_DIR/SHA256SUMS.txt" --signature "$RELEASE_DIR/SHA256SUMS.txt.sig")
    if [[ -r "$SIGNING_PRIVATE_KEY_FILE" ]]; then
        RELEASE_SIGNATURE_ARGS+=(--private-key-file "$SIGNING_PRIVATE_KEY_FILE")
    fi
    # Una clave pública indicada por variable de entorno tiene precedencia
    # también en signature::run; no pasar un --public-key-file que la anule.
    if [[ "$SIGNING_PUBLIC_KEY_ENV_ACTIVE" -eq 0 && -r "$SIGNING_PUBLIC_KEY_FILE" ]]; then
        RELEASE_SIGNATURE_ARGS+=(--public-key-file "$SIGNING_PUBLIC_KEY_FILE")
    fi
}

prepare_release_signature() {
    local private_available=0 public_available=0 ssh_signature="$RELEASE_DIR/SHA256SUMS.txt.sshsig"
    [[ -r "$SIGNING_PRIVATE_KEY_FILE" || -n "${LTOOLS_SIGNING_PRIVATE_KEY:-${LTERMINAL_SIGNING_PRIVATE_KEY:-}}" ]] && private_available=1
    [[ -r "$SIGNING_PUBLIC_KEY_FILE" || "$SIGNING_PUBLIC_KEY_ENV_ACTIVE" -eq 1 ]] && public_available=1
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
    if (( SSH_SIGNING_AVAILABLE )); then
        run_logged ltools_ssh_sign_manifest "$RELEASE_DIR/SHA256SUMS.txt" "$ssh_signature"
        run_logged ltools_ssh_verify_manifest "$RELEASE_DIR/SHA256SUMS.txt" "$ssh_signature"
        ok 'SHA256SUMS.txt firmado y verificado con la clave SSH configurada'
    else
        rm -f -- "$ssh_signature"
        if (( SIGNING_REQUIRED )); then
            die 'release estricta: falta la clave SSH de firma (LTOOLS_SSH_SIGNING_KEY_FILE o ~/.ssh/id_ed25519) o ssh-keygen'
        fi
        warn 'release local sin firma SSH; use una clave OpenSSH para habilitar SHA256SUMS.txt.sshsig'
    fi
    if [[ "$RELEASE_DIR" != "$OUTPUT_DIR" ]]; then
        cp -a -- "$RELEASE_DIR/SHA256SUMS.txt" "$OUTPUT_DIR/SHA256SUMS.txt"
        if [[ -s "$RELEASE_DIR/SHA256SUMS.txt.sig" ]]; then
            cp -a -- "$RELEASE_DIR/SHA256SUMS.txt.sig" "$OUTPUT_DIR/SHA256SUMS.txt.sig"
        else
            rm -f -- "$OUTPUT_DIR/SHA256SUMS.txt.sig"
        fi
        if [[ -s "$ssh_signature" ]]; then
            cp -a -- "$ssh_signature" "$OUTPUT_DIR/SHA256SUMS.txt.sshsig"
        else
            rm -f -- "$OUTPUT_DIR/SHA256SUMS.txt.sshsig"
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
    if ask_yes_no 'Exigir auditoría estricta (Cargo audit/deny, ShellCheck y actionlint)' "$STRICT_SECURITY"; then STRICT_SECURITY=1; else STRICT_SECURITY=0; fi
    if ask_yes_no 'Exigir revisión estática de scripts y workflows (ShellCheck/actionlint)' "$SECURITY_REVIEW"; then SECURITY_REVIEW=1; else SECURITY_REVIEW=0; fi
    if ask_yes_no 'Aplicar autocorrecciones Rust seguras y volver a escanear' "$AUTO_FIX"; then AUTO_FIX=1; else AUTO_FIX=0; fi
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
            --component)
                (($# >= 2)) || die '--component necesita backend, frontend, cli, tarball, appimage, windows o all'
                COMPONENT="${2,,}"; shift ;;
            --test-existing)
                (($# >= 2)) || die '--test-existing necesita la ruta de un binario, AppImage o tarball'
                TEST_EXISTING="$2"; shift ;;
            --preview) PREVIEW=1 ;;
            --skip-checks|--no-checks) CHECKS=0 ;;
            --strict-security) STRICT_SECURITY=1; SECURITY_REVIEW=1 ;;
            --security-review) SECURITY_REVIEW=1 ;;
            --auto-fix) AUTO_FIX=1 ;;
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
            -h|--help|--workflow-help) usage; exit 0 ;;
            --version) printf '%s\n' "$VERSION"; exit 0 ;;
            *) die "argumento desconocido: $1 (usa --help)" ;;
        esac
        shift
    done
}

apply_component_defaults() {
    case "$COMPONENT" in
        all) ;;
        backend|frontend)
            PACKAGE=0; APPIMAGE=0; WINDOWS_WINE=0; TESTS=0; SMOKE=0; E2E=0
            MENU_E2E=0; SOFTWARE_GIT_E2E=0 ;;
        cli)
            CLI_ONLY=1; PACKAGE=0; APPIMAGE=0; WINDOWS_WINE=0; TESTS=0; SMOKE=0; E2E=0
            MENU_E2E=0; SOFTWARE_GIT_E2E=0 ;;
        tarball)
            PACKAGE=1; APPIMAGE=0; WINDOWS_WINE=0; TESTS=0; SMOKE=0; E2E=0
            MENU_E2E=0; SOFTWARE_GIT_E2E=0 ;;
        appimage)
            PACKAGE=0; APPIMAGE=1; WINDOWS_WINE=0; TESTS=0; SMOKE=0; E2E=0
            MENU_E2E=0; SOFTWARE_GIT_E2E=0 ;;
        windows)
            PACKAGE=0; APPIMAGE=0; WINDOWS_WINE=1; TESTS=0; SMOKE=0; E2E=0
            MENU_E2E=0; SOFTWARE_GIT_E2E=0 ;;
        *) die "componente desconocido: $COMPONENT (usa backend, frontend, cli, tarball, appimage, windows o all)" ;;
    esac
    if (( CLI_ONLY )) && [[ "$WINDOWS_WINE" -eq 1 ]]; then
        die '--component cli no se combina con la etapa Windows; usa --component windows para el CLI Windows'
    fi
}

test_existing_artifact() {
    local artifact="$1" failures=0
    [[ -e "$artifact" ]] || die "no existe el artefacto a probar: $artifact"
    if [[ "$artifact" == *.tar.gz ]]; then
        run_suite_test 'Tarball existente' "$ROOT_DIR/tests/linux/tarball-e2e.sh" --tarball "$artifact" --require-gui || failures=$((failures + 1))
    elif [[ "$artifact" == *-cli.AppImage ]]; then
        run_suite_test 'CLI AppImage existente' "$ROOT_DIR/tests/linux/cli-e2e.sh" --binary "$artifact" || failures=$((failures + 1))
    else
        run_suite_test 'Smoke del artefacto existente' "$ROOT_DIR/tests/linux/smoke.sh" --require-gui --binary "$artifact" || failures=$((failures + 1))
        run_suite_test 'CLI del artefacto existente' "$ROOT_DIR/tests/linux/cli-e2e.sh" --binary "$artifact" || failures=$((failures + 1))
        run_suite_test 'E2E funcional del artefacto existente' "$ROOT_DIR/tests/linux/e2e.sh" --require-dependencies --binary "$artifact" || failures=$((failures + 1))
        run_suite_test 'Menús del artefacto existente' "$ROOT_DIR/tests/linux/menu-e2e.sh" --require-gui --binary "$artifact" || failures=$((failures + 1))
        run_suite_test 'Stores y Git del artefacto existente' "$ROOT_DIR/tests/linux/software-git-e2e.sh" --binary "$artifact" || failures=$((failures + 1))
        run_suite_test 'Ayudas nativas del artefacto existente' "$ROOT_DIR/tests/linux/native-help-e2e.sh" --binary "$artifact" || failures=$((failures + 1))
    fi
    (( failures == 0 )) || return 1
}

cargo_args=(--locked)
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
apply_component_defaults
if (( PREVIEW )); then
    exec bash "$ROOT_DIR/scripts/live-preview.sh"
fi
if [[ -n "$TEST_EXISTING" ]]; then
    test_existing_artifact "$TEST_EXISTING"
    exit $?
fi
if [[ "$STRICT_SECURITY" -eq 1 || "$SECURITY_REVIEW" -eq 1 || "$AUTO_FIX" -eq 1 ]]; then CHECKS=1; fi
validate_output_paths
init_logging "$@"
load_signing_material
configure_interactive
if [[ "$STRICT_SECURITY" -eq 1 ]]; then SECURITY_REVIEW=1; fi
if [[ "$STRICT_SECURITY" -eq 1 || "$SECURITY_REVIEW" -eq 1 || "$AUTO_FIX" -eq 1 ]]; then CHECKS=1; fi
if [[ "$NO_LOG" -eq 0 ]]; then
    printf '[CONFIG] component=%s test_existing=%s preview=%s clean=%s fast=%s checks=%s strict_security=%s security_review=%s auto_fix=%s tests=%s smoke=%s e2e=%s menu_e2e=%s software_git_e2e=%s package=%s appimage=%s offline=%s jobs=%s windows_wine=%s windows_target=%s release_dir=%s\n' \
        "$COMPONENT" "$TEST_EXISTING" "$PREVIEW" "$CLEAN" "$FAST" "$CHECKS" "$STRICT_SECURITY" "$SECURITY_REVIEW" "$AUTO_FIX" "$TESTS" "$SMOKE" "$E2E" "$MENU_E2E" "$SOFTWARE_GIT_E2E" "$PACKAGE" "$APPIMAGE" "$OFFLINE" "$JOBS" "$WINDOWS_WINE" "$WINDOWS_TARGET" "$RELEASE_DIR"
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
if [[ "$STRICT_SECURITY" -eq 1 || "$SECURITY_REVIEW" -eq 1 ]]; then
    missing_security_tools=()
    required_security_tools=(shellcheck actionlint)
    if [[ "$STRICT_SECURITY" -eq 1 ]]; then
        required_security_tools+=(cargo-audit cargo-deny)
    fi
    for security_tool in "${required_security_tools[@]}"; do
        if ! command -v "$security_tool" >/dev/null 2>&1; then
            missing_security_tools+=("$security_tool")
        fi
    done
    if ((${#missing_security_tools[@]})); then
        die "la revisión estática solicitada no puede empezar; faltan: ${missing_security_tools[*]}"
    fi
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
        ok 'se detectó el dispositivo y el helper FUSE; el montaje real se comprobará en el smoke'
    elif [[ "$FUSE_REQUIRED" -eq 1 ]]; then
        die 'FUSE no está disponible: falta /dev/fuse o fusermount/fusermount3'
    else
        warn 'FUSE no está disponible; el AppImage se generará, pero se probará mediante extracción.'
        warn 'Para habilitar ejecución directa en Arch Linux y derivados: instala fuse2 y carga «sudo modprobe fuse».'
    fi
fi
if [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]]; then
    require_command jq
fi
if [[ "$APPIMAGE" -eq 1 && "$SMOKE" -eq 1 ]]; then
    require_command timeout
    require_command stat
    require_command grep
fi
if [[ "$MENU_E2E" -eq 1 ]]; then
    require_command timeout
fi
if [[ "$PACKAGE" -eq 1 || "$SMOKE" -eq 1 || "$MENU_E2E" -eq 1 ]]; then
    require_command timeout
    for gui_command in xvfb-run xdotool import identify file xdpyinfo; do
        require_command "$gui_command"
    done
    timeout 10 xvfb-run -a -s "-screen 0 1280x900x24" xdpyinfo >/dev/null 2>&1 ||
        die 'Xvfb está instalado, pero no puede iniciar un display: no se comenzará la build con E2E GUI incompletas'
fi
if [[ "$E2E" -eq 1 ]]; then
    require_command rsync
    if ! command -v gio >/dev/null 2>&1 && ! command -v trash-put >/dev/null 2>&1; then
        die 'la E2E de migración/rollback requiere gio o trash-put; no se iniciará una build que omita esa prueba'
    fi
fi

step 'Comprobando toolchain'
printf '    Cargo: %s\n' "$(cargo --version)"
printf '    Rustc: %s\n' "$(rustc --version)"
printf '    Versión del proyecto: %s\n' "$VERSION"
[[ "$ARCH" == x86_64 || "$ARCH" == aarch64 || "$ARCH" == armv7l ]] || warn "arquitectura no probada: $ARCH"

configure_cargo_profile

if (( CLI_ONLY )); then
    step 'Compilando perfil CLI Rust aislado'
    CLI_TARGET_DIR="$ROOT_DIR/rust/target/cli-preview"
    run_logged env CARGO_TARGET_DIR="$CLI_TARGET_DIR" cargo build --manifest-path "$MANIFEST" \
        "${cargo_args[@]}" --features cli --release
    CLI_BIN_ARTIFACT="$CLI_TARGET_DIR/release/ltools"
    [[ -x "$CLI_BIN_ARTIFACT" ]] || die "Cargo terminó, pero no apareció $CLI_BIN_ARTIFACT"
    run_logged "$CLI_BIN_ARTIFACT" --help >/dev/null
    ok "perfil CLI generado: $CLI_BIN_ARTIFACT"
    exit 0
fi

if [[ "$CLEAN" -eq 1 ]]; then
    step 'Limpiando artefactos Rust'
    run_logged cargo clean --manifest-path "$MANIFEST"
    ok 'rust/target limpiado'
fi

if [[ "$CHECKS" -eq 1 ]]; then
    step 'Validando formato Rust'
    if ! run_logged cargo fmt --manifest-path "$MANIFEST" --all -- --check; then
        if [[ "$AUTO_FIX" -ne 1 ]]; then
            die 'rustfmt detectó diferencias; corrígelas o vuelve a ejecutar con --auto-fix para aplicar formato y reescanear.'
        fi
        step 'Autocorrigiendo formato Rust'
        rust_fix_before="$(rust_source_hashes)"
        run_logged cargo fmt --manifest-path "$MANIFEST" --all
        ok '[AUTO-FIX] rustfmt aplicó el formato; se repite el escaneo antes de continuar.'
        report_rust_autofix_changes "$rust_fix_before"
        step 'Reescaneando formato Rust'
        run_logged cargo fmt --manifest-path "$MANIFEST" --all -- --check
    fi
    ok 'rustfmt correcto'

    step 'Validando Clippy'
    if ! run_logged cargo clippy --manifest-path "$MANIFEST" --workspace --all-features --all-targets "${cargo_args[@]}" -- -D warnings; then
        if [[ "$AUTO_FIX" -ne 1 ]]; then
            die 'Clippy encontró avisos; corrígelos o vuelve a ejecutar con --auto-fix para aplicar solo sugerencias mecánicas y reescanear.'
        fi
        step 'Aplicando autocorrecciones mecánicas de Clippy'
        rust_fix_before="$(rust_source_hashes)"
        run_logged cargo clippy --fix --allow-dirty --allow-staged --manifest-path "$MANIFEST" --workspace --all-features --all-targets "${cargo_args[@]}"
        ok '[AUTO-FIX] Clippy aplicó sugerencias mecánicas; se volverán a ejecutar formato y Clippy estricto.'
        step 'Normalizando formato después de Clippy'
        run_logged cargo fmt --manifest-path "$MANIFEST" --all
        ok '[AUTO-FIX] rustfmt normalizó el resultado de Clippy antes del reescaneo.'
        report_rust_autofix_changes "$rust_fix_before"
        step 'Reescaneando formato después de Clippy'
        run_logged cargo fmt --manifest-path "$MANIFEST" --all -- --check
        step 'Reescaneando Clippy estricto'
        run_logged cargo clippy --manifest-path "$MANIFEST" --workspace --all-features --all-targets "${cargo_args[@]}" -- -D warnings
    fi
    ok 'Clippy sin avisos en todos los targets (incluidos tests)'

    step 'Validando lanzadores, build y tests Bash'
    run_logged "$ROOT_DIR/tests/encoding.sh"
    ok 'codificaciones UTF-8/UTF-8 BOM/ANSI correctas'
    run_logged bash "$ROOT_DIR/tests/scripts-syntax.sh"
    ok 'sintaxis Bash/PowerShell correcta (PowerShell cuando pwsh está disponible)'
    step 'Revisando estáticamente scripts y workflows'
    if [[ "$SECURITY_REVIEW" -eq 1 ]]; then
        run_logged bash "$ROOT_DIR/tests/static-security.sh" --strict
    else
        run_logged bash "$ROOT_DIR/tests/static-security.sh"
    fi
    ok 'revisión estática completada; los analizadores omitidos constan explícitamente en el log'
    if command -v ssh-keygen >/dev/null 2>&1; then
        step 'Probando firma OpenSSH de checksums'
        run_logged bash "$ROOT_DIR/tests/ssh-signing.sh"
        ok 'firma SSH validada y manipulación de manifiesto rechazada'
    else
        warn 'ssh-keygen no está instalado; no se puede ejecutar la prueba de firma SSH.'
    fi
    if [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]]; then
        step 'Validando inclusión de licencias y avisos de terceros'
        run_logged bash "$ROOT_DIR/tests/third-party-licenses.sh"
        ok 'avisos de terceros completos y destinos de paquete protegidos'
    fi
    run_logged bash "$ROOT_DIR/tests/temp-cleaner.sh"
    ok 'limpieza temporal limitada a artefactos propios, con rutas y uso activo verificados'

    cargo_home_dir="${CARGO_HOME:-${HOME:-/tmp}/.cargo}"
    if command -v cargo-audit >/dev/null 2>&1 && { [[ "$OFFLINE" -eq 0 ]] || [[ -d "$cargo_home_dir/advisory-db" ]]; }; then
        step 'Auditando dependencias Rust'
        run_cargo_audit() {
            local lockfile="$1" audit_home='' status=0
            local -a audit_args=(audit --file "$lockfile")
            [[ "$OFFLINE" -eq 1 ]] && audit_args+=(--no-fetch)
            if [[ "$OFFLINE" -eq 1 && -d "$cargo_home_dir/advisory-db" && -d "$cargo_home_dir/registry" ]]; then
                audit_home="$OUTPUT_DIR/.cargo-audit-$BUILD_ID"
                mkdir -p -- "$audit_home"
                cp -a -- "$cargo_home_dir/advisory-db" "$audit_home/"
                ln -s -- "$cargo_home_dir/registry" "$audit_home/registry"
                CARGO_HOME="$audit_home" run_logged cargo "${audit_args[@]}" || status=$?
                rm -rf -- "$audit_home"
                return "$status"
            fi
            run_logged cargo "${audit_args[@]}"
        }
        (cd "$ROOT_DIR/rust" && run_cargo_audit Cargo.lock)
        run_cargo_audit "$ROOT_DIR/fuzz/Cargo.lock"
        ok 'cargo audit sin vulnerabilidades conocidas'
    elif [[ "$STRICT_SECURITY" -eq 1 ]]; then
        die 'la auditoría estricta requiere cargo-audit y, en modo offline, una base local de advisories'
    else
        warn 'cargo-audit no está disponible o falta su base local en modo offline; se omite la auditoría de seguridad.'
    fi
    if command -v cargo-deny >/dev/null 2>&1; then
        step 'Validando licencias y fuentes Rust'
        cargo_home_dir="${CARGO_HOME:-${HOME:-/tmp}/.cargo}"
        if [[ "$OFFLINE" -eq 0 ]] && [[ -w "$cargo_home_dir/advisory-dbs" ]]; then
            (cd "$ROOT_DIR/rust" && run_logged cargo deny --config "$ROOT_DIR/deny.toml" check)
            run_logged cargo deny --manifest-path "$ROOT_DIR/fuzz/Cargo.toml" --config "$ROOT_DIR/fuzz/deny.toml" check
        elif [[ -d "$cargo_home_dir/advisory-dbs" && -d "$cargo_home_dir/registry" ]]; then
            # Algunos entornos de CI/sandbox montan ~/.cargo como solo
            # lectura. cargo-deny necesita un lock exclusivo incluso en modo
            # offline; usa una copia efímera de sus datos locales para
            # conservar la validación de licencias, fuentes y advisories.
            deny_home="$OUTPUT_DIR/.cargo-deny-$BUILD_ID"
            mkdir -p -- "$deny_home"
            cp -a -- "$cargo_home_dir/advisory-dbs" "$deny_home/"
            ln -s -- "$cargo_home_dir/registry" "$deny_home/registry"
            deny_status=0
            (cd "$ROOT_DIR/rust" && CARGO_HOME="$deny_home" run_logged cargo deny --offline --config "$ROOT_DIR/deny.toml" check) || deny_status=$?
            if (( deny_status == 0 )); then
                (cd "$ROOT_DIR" && CARGO_HOME="$deny_home" run_logged cargo deny --offline --manifest-path "$ROOT_DIR/fuzz/Cargo.toml" --config "$ROOT_DIR/fuzz/deny.toml" check) || deny_status=$?
            fi
            rm -rf -- "$deny_home"
            (( deny_status == 0 )) || exit "$deny_status"
        else
            die 'cargo-deny está instalado, pero no hay una base local de advisories y no se puede preparar un entorno offline'
        fi
        ok 'cargo-deny correcto'
    elif [[ "$STRICT_SECURITY" -eq 1 ]]; then
        die 'la auditoría estricta requiere cargo-deny instalado'
    else
        warn 'cargo-deny no está disponible; se omite la validación de licencias y fuentes.'
    fi
fi

if [[ "$TESTS" -eq 1 ]]; then
    step 'Ejecutando pruebas Rust'
    run_logged cargo test --manifest-path "$MANIFEST" --workspace --all-features --all-targets "${cargo_args[@]}"
    ok 'cargo test correcto (incluidas todas las features)'
fi

if [[ "$CHECKS" -eq 1 ]] && command -v rustup >/dev/null 2>&1 &&
    installed_targets="$(rustup target list --installed 2>/dev/null || true)" &&
    grep -Fxq "$WINDOWS_TARGET" <<<"$installed_targets"; then
    step 'Clippy cruzado Windows'
    run_logged cargo clippy --manifest-path "$MANIFEST" --workspace --all-features --target "$WINDOWS_TARGET" --all-targets "${cargo_args[@]}" --jobs "$JOBS" -- -D warnings
    ok "Clippy y todos los targets Rust correctos para $WINDOWS_TARGET"
elif [[ "$CHECKS" -eq 1 && "$WINDOWS_WINE" -eq 1 ]]; then
    step 'Clippy cruzado Windows'
    run_logged cargo clippy --manifest-path "$MANIFEST" --workspace --target "$WINDOWS_TARGET" --all-targets "${cargo_args[@]}" --jobs "$JOBS" -- -D warnings
    ok "Clippy y todos los targets Rust correctos para $WINDOWS_TARGET"
elif [[ "$CHECKS" -eq 1 ]]; then
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
    WINDOWS_WINE_CARGO_TARGET_DIR="$ROOT_DIR/rust/target/windows-wine"
    WINDOWS_WINE_TARGET_EXE="$WINDOWS_WINE_CARGO_TARGET_DIR/$WINDOWS_TARGET/release/ltools.exe"
    WINDOWS_WINE_TARGET_CLI_EXE="$WINDOWS_WINE_CARGO_TARGET_DIR/$WINDOWS_TARGET/release/ltools-cli.exe"
    WINDOWS_WINE_ARTIFACT="$WINDOWS_WINE_ARTIFACT_DIR/ltools-$VERSION-windows-${WINDOWS_TARGET%%-*}.exe"
    WINDOWS_WINE_CLI_ARTIFACT="$WINDOWS_WINE_ARTIFACT_DIR/ltools-$VERSION-windows-${WINDOWS_TARGET%%-*}-cli.exe"
    WINDOWS_WINE_ZIP_ARTIFACT="$WINDOWS_WINE_ARTIFACT_DIR/ltools-$VERSION-windows-${WINDOWS_TARGET%%-*}.zip"
    WINDOWS_WINE_LOG="$WINDOWS_WINE_ARTIFACT_DIR/windows-wine-$BUILD_ID.log"
    wine_args=(
        --target "$WINDOWS_TARGET"
        --output "$WINDOWS_WINE_ARTIFACT_DIR"
        --jobs "$JOBS"
    )
    WINDOWS_WINE_TESTS=1
    [[ "$CLEAN" -eq 1 ]] && wine_args+=(--clean)
    [[ "$FAST" -eq 1 ]] && wine_args+=(--fast)
    [[ "$OFFLINE" -eq 1 ]] && wine_args+=(--offline)
    if [[ "$TESTS" -eq 0 ]]; then
        WINDOWS_WINE_TESTS=0
        wine_args+=(--no-tests)
    else
        # Las pruebas de CLI Windows/Wine son independientes de las suites
        # GUI Linux. Solo se exige la captura GUI Windows cuando se mantienen
        # activas las suites gráficas completas del pipeline.
        WINDOWS_WINE_TESTS=1
        if [[ "$SMOKE" -eq 1 && "$MENU_E2E" -eq 1 ]]; then
            wine_args+=(--require-gui)
        fi
    fi
    [[ "$PACKAGE" -eq 0 ]] && wine_args+=(--no-package)
    [[ "$NON_INTERACTIVE" -eq 1 || "$EXPLICIT_OPTIONS" -eq 1 ]] && wine_args+=(--non-interactive)
    [[ -n "$WINDOWS_WINE_RUNNER" ]] && wine_args+=(--runner "$WINDOWS_WINE_RUNNER")
    [[ -n "$WINDOWS_WINE_PREFIX" ]] && wine_args+=(--prefix "$WINDOWS_WINE_PREFIX")
    [[ "$WINDOWS_WINE_INSTALL_MONO" -eq 1 ]] && wine_args+=(--install-mono)
    [[ "$NO_LOG" -eq 0 ]] && wine_args+=(--log "$WINDOWS_WINE_LOG")
    mkdir -p -- "$OUTPUT_DIR"
    run_logged env \
        "TMPDIR=$OUTPUT_DIR" \
        "LTOOLS_WINDOWS_CARGO_TARGET_DIR=$WINDOWS_WINE_CARGO_TARGET_DIR" \
        "LTOOLS_GUI_CAPTURE_DIR=$WINDOWS_WINE_ARTIFACT_DIR/captures" \
        "$ROOT_DIR/tests/linux/windows-wine.sh" "${wine_args[@]}"
    if [[ "$PACKAGE" -eq 1 ]]; then
        [[ -s "$WINDOWS_WINE_ARTIFACT" ]] || die 'la validación Windows no produjo el ejecutable principal'
        [[ -s "$WINDOWS_WINE_CLI_ARTIFACT" ]] || die 'la validación Windows no produjo el ejecutable CLI'
        [[ -s "$WINDOWS_WINE_ZIP_ARTIFACT" ]] || die 'la validación Windows no produjo el ZIP portable'
    else
        [[ -s "$WINDOWS_WINE_TARGET_EXE" ]] || die 'la compilación Windows no produjo el ejecutable principal'
        [[ -s "$WINDOWS_WINE_TARGET_CLI_EXE" ]] || die 'la compilación Windows no produjo el ejecutable CLI'
    fi
    [[ -x "$BIN" ]] || die 'la fase Windows alteró el target Linux; se esperaba conservar el binario Linux'
    if [[ "$WINDOWS_WINE_TESTS" -eq 1 ]]; then
        ok 'compilación y pruebas Windows bajo Wine/Proton correctas'
    else
        ok 'compilación Windows correcta; pruebas bajo Wine omitidas por configuración'
    fi
fi

if [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]]; then
    step 'Construyendo paquete distribuible'
    mkdir -p "$OUTPUT_DIR"
    PACKAGE_NAME="ltools-$VERSION-linux-$ARCH"
    STAGING="$(mktemp -d "$OUTPUT_DIR/.ltools-build.XXXXXX")"
    cleanup_staging() {
        [[ -n "${STAGING:-}" && -d "$STAGING" ]] && rm -rf -- "$STAGING"
        ltools_cleanup_release_staging
    }
    trap cleanup_staging EXIT
    LOCAL_PUBLISH_DIR="$STAGING/local-publish"
    mkdir -p -- "$LOCAL_PUBLISH_DIR"
    PACKAGE_DIR="$STAGING/$PACKAGE_NAME"
    mkdir -p "$PACKAGE_DIR/rust/target/release"
    RUST_HOST_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
    [[ -n "$RUST_HOST_TARGET" ]] || die 'no se pudo identificar la plataforma Cargo para generar avisos de licencias'
    ltools_create_third_party_license_bundle \
        "$PACKAGE_DIR/THIRD-PARTY-LICENSES" "$RUST_HOST_TARGET" \
        || die 'no se pudieron incluir las licencias y avisos de las dependencias Linux'
    [[ -s "$ROOT_DIR/LICENSE" ]] || die 'falta la licencia MIT del proyecto'
    cp -a -- "$ROOT_DIR/LICENSE" "$PACKAGE_DIR/LICENSE"

    copy_file() {
        local source="$1"
        [[ -e "$ROOT_DIR/$source" ]] || die "falta el archivo de distribución: $source"
        mkdir -p -- "$(dirname -- "$PACKAGE_DIR/$source")"
        cp -a -- "$ROOT_DIR/$source" "$PACKAGE_DIR/$source"
    }

    copy_launcher() {
        local name="$1"
        local fallback="${name}.sh"
        local source
        if [[ -e "$ROOT_DIR/$name" ]]; then
            source="$name"
        elif [[ -e "$ROOT_DIR/$fallback" ]]; then
            # Los nombres cortos son la interfaz publicada. El fallback evita
            # que un checkout limpio dependa de wrappers no versionados.
            source="$fallback"
        else
            die "falta el lanzador de distribución: $name (ni $fallback)"
        fi
        mkdir -p -- "$(dirname -- "$PACKAGE_DIR/$name")"
        cp -a -- "$ROOT_DIR/$source" "$PACKAGE_DIR/$name"
    }

    # El paquete runtime solo necesita la fachada compatible; los builders y
    # wrappers de desarrollo pertenecen al repositorio, no a la distribución.
    copy_launcher ltools
    copy_launcher ltools-cli
    copy_file ltools.sh
    copy_file ltools-cli.sh
    cp -a -- "$BIN" "$PACKAGE_DIR/rust/target/release/ltools"
    copy_file README.md
    # Descriptor machine-readable generado por el mismo backend que se
    # distribuye, para que terminales y frontends conozcan las capacidades
    # reales sin interpretar la ayuda humana.
    "$BIN" capabilities --format json >"$PACKAGE_DIR/ltools-capabilities.json"
    "$BIN" capabilities --format terminal-json >"$PACKAGE_DIR/ltools-terminal.json"
    TERMINAL_DESCRIPTOR_ARTIFACT="$LOCAL_PUBLISH_DIR/ltools-terminal.json"
    CAPABILITIES_ARTIFACT="$LOCAL_PUBLISH_DIR/ltools-capabilities.json"
    CAPABILITIES_SCHEMA_ARTIFACT="$LOCAL_PUBLISH_DIR/ltools-capabilities.schema.json"
    TERMINAL_SCHEMA_ARTIFACT="$LOCAL_PUBLISH_DIR/ltools-terminal.schema.json"
    cp -a -- "$PACKAGE_DIR/ltools-terminal.json" "$TERMINAL_DESCRIPTOR_ARTIFACT"
    cp -a -- "$PACKAGE_DIR/ltools-capabilities.json" "$CAPABILITIES_ARTIFACT"
    cp -a -- "$ROOT_DIR/appimage/ltools-capabilities.schema.json" "$CAPABILITIES_SCHEMA_ARTIFACT"
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
    chmod +x "$PACKAGE_DIR"/ltools "$PACKAGE_DIR"/ltools-cli "$PACKAGE_DIR"/*.sh \
        "$PACKAGE_DIR/tests"/*.sh "$PACKAGE_DIR/tests/linux"/*.sh \
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
        ARTIFACT="$LOCAL_PUBLISH_DIR/$PACKAGE_NAME.tar.gz"
        run_logged tar -C "$STAGING" -czf "$ARTIFACT" "$PACKAGE_NAME"
        [[ -s "$ARTIFACT" ]] || die 'no se pudo crear el paquete tar.gz'
        PACKAGE_LIST="$STAGING/$PACKAGE_NAME.list"
        run_logged tar -tzf "$ARTIFACT" >"$PACKAGE_LIST"
        if grep -Eq '/(platform|windows)/|/build\.sh$' "$PACKAGE_LIST"; then
            die 'el paquete runtime contiene código de build o código de otra plataforma'
        fi
        ok "paquete generado: $ARTIFACT"
        tarball_e2e_args=(--tarball "$ARTIFACT")
        tarball_e2e_args+=(--require-gui)
        step 'Probando el tarball extraído'
        run_logged "$ROOT_DIR/tests/linux/tarball-e2e.sh" "${tarball_e2e_args[@]}"
        ok 'el tarball extraído ejecuta su CLI y GUI'
    fi
    if [[ "$APPIMAGE" -eq 1 ]]; then
        step 'Construyendo AppImage'
        build_appimage_variant() {
            local variant="$1" artifact="$2" desktop_source="$3" appdir appstream_meta
            appdir="$STAGING/AppDir-$variant"
            mkdir -p "$appdir"
            cp -a -- "$PACKAGE_DIR/." "$appdir/"
            [[ -s "$appdir/THIRD-PARTY-LICENSES/INDEX.txt" ]] || die 'AppImage sin índice de licencias de terceros'
            grep -Fq 'ISC' "$appdir/THIRD-PARTY-LICENSES/INDEX.txt" || die 'AppImage sin avisos ISC de dependencias'
            grep -Fq 'CDLA-Permissive-2.0' "$appdir/THIRD-PARTY-LICENSES/INDEX.txt" || die 'AppImage sin aviso CDLA de raíces TLS'
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
            # appimagetool puede ser un AppImage o necesitar FUSE para
            # localizar su runtime interno. Forzar extracción lo hace
            # utilizable en builders sin FUSE; ARCH evita que dependa de que
            # pueda inferir la arquitectura desde un binario del AppDir.
            run_logged env ARCH="$ARCH" APPIMAGE_EXTRACT_AND_RUN=1 \
                appimagetool --no-appstream "$appdir" "$artifact"
            chmod +x "$artifact" || die "no se pudo aplicar el permiso ejecutable a $artifact"
            [[ -s "$artifact" ]] || die "no se pudo crear el AppImage $variant"
            ok "AppImage $variant generado: $artifact"
        }

        APPIMAGE_ARTIFACT="$LOCAL_PUBLISH_DIR/$PACKAGE_NAME.AppImage"
        build_appimage_variant terminal "$APPIMAGE_ARTIFACT" "$ROOT_DIR/appimage/ltools.desktop"
        CLI_APPIMAGE_ARTIFACT="$LOCAL_PUBLISH_DIR/$PACKAGE_NAME-cli.AppImage"
        build_appimage_variant cli "$CLI_APPIMAGE_ARTIFACT" "$ROOT_DIR/appimage/ltools-cli.desktop"
        CLI_SMOKE_OUTPUT="$STAGING/cli-appimage-smoke.log"
        APPIMAGE_EXTRACT_AND_RUN=1 "$CLI_APPIMAGE_ARTIFACT" >"$CLI_SMOKE_OUTPUT" 2>&1 \
            || die 'el AppImage CLI no pudo mostrar la ayuda sin argumentos'
        grep -Fq 'Uso: ltools' "$CLI_SMOKE_OUTPUT" || die 'el AppImage CLI no mostró la ayuda sin argumentos'
        ok 'AppImage CLI verificado: sin argumentos muestra la ayuda'
    fi

    # Publica solo después de que el empaquetado y las pruebas específicas
    # hayan terminado. Cada archivo se crea primero en el mismo filesystem;
    # rename sustituye el destino de forma atómica y conserva la copia previa
    # si falla la generación o validación del artefacto.
    while IFS= read -r -d '' staged_file; do
        run_logged mv -f -- "$staged_file" "$OUTPUT_DIR/${staged_file##*/}"
    done < <(find "$LOCAL_PUBLISH_DIR" -maxdepth 1 -type f -print0 | sort -z)
    TERMINAL_DESCRIPTOR_ARTIFACT="$OUTPUT_DIR/ltools-terminal.json"
    CAPABILITIES_ARTIFACT="$OUTPUT_DIR/ltools-capabilities.json"
    CAPABILITIES_SCHEMA_ARTIFACT="$OUTPUT_DIR/ltools-capabilities.schema.json"
    TERMINAL_SCHEMA_ARTIFACT="$OUTPUT_DIR/ltools-terminal.schema.json"
    [[ "$PACKAGE" -eq 0 ]] || ARTIFACT="$OUTPUT_DIR/$PACKAGE_NAME.tar.gz"
    if [[ "$APPIMAGE" -eq 1 ]]; then
        APPIMAGE_ARTIFACT="$OUTPUT_DIR/$PACKAGE_NAME.AppImage"
        CLI_APPIMAGE_ARTIFACT="$OUTPUT_DIR/$PACKAGE_NAME-cli.AppImage"
    fi
else
    warn 'Se omitió la generación de artefactos distribuibles.'
fi

if [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]]; then
    step 'Publicando artefactos en la carpeta release'
    FINAL_RELEASE_DIR="$RELEASE_DIR"
    ltools_create_release_staging "$FINAL_RELEASE_DIR" || die 'no se pudo preparar el staging de release; el destino previo permanece intacto'
    RELEASE_DIR="$LTOOLS_RELEASE_STAGING"
    # Se modifica únicamente la copia temporal; el destino final no cambia
    # hasta que los manifiestos, firmas y E2E de release terminen bien.
    release_output_real="$(readlink -f -- "$OUTPUT_DIR" 2>/dev/null || realpath -- "$OUTPUT_DIR")"
    release_dir_real="$(readlink -f -- "$RELEASE_DIR" 2>/dev/null || realpath -- "$RELEASE_DIR")"
    if [[ "$release_output_real" != "$release_dir_real" ]]; then
        find "$RELEASE_DIR" -maxdepth 1 -type f \
            \( -name 'ltools-*.AppImage' -o -name 'ltools-*.tar.gz' \
            -o -name 'ltools-*.zip' -o -name 'ltools-*.exe' \
            -o -name 'ltools-capabilities.json' -o -name 'ltools-capabilities-windows.json' \
            -o -name 'ltools-terminal.json' -o -name 'ltools-terminal-windows.json' \
            -o -name 'ltools-release.json' -o -name 'ltools-project.json' \
            -o -name 'ltools-capabilities.schema.json' -o -name 'ltools-terminal.schema.json' \
            -o -name 'ltools-project.schema.json' -o -name 'ltools-release.schema.json' \
            -o -name 'THIRD-PARTY-LICENSES-windows.zip' \
            -o -name 'SHA256SUMS.txt' \
            -o -name 'SHA256SUMS.txt.sig' -o -name 'SHA256SUMS.txt.sshsig' \
            -o -name 'LICENSE' \) -delete
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
            -o -name 'ltools-terminal.json' -o -name 'ltools-*.schema.json' \) -print0
    )
    if [[ -d "$ROOT_DIR/dist/windows" ]]; then
        while IFS= read -r -d '' file; do copy_to_release "$file"; done < <(
            find "$ROOT_DIR/dist/windows" -maxdepth 1 -type f \
                \( -name "ltools-$VERSION-windows-*" -o -name 'ltools-capabilities.json' \
                -o -name 'ltools-terminal.json' -o -name 'ltools-capabilities-windows.json' \
                -o -name 'ltools-terminal-windows.json' -o -name 'ltools-*.schema.json' \
                -o -name 'THIRD-PARTY-LICENSES-windows.zip' \) -print0
        )
    fi
    if [[ "$WINDOWS_WINE" -eq 1 && -d "$WINDOWS_WINE_ARTIFACT_DIR" ]]; then
        while IFS= read -r -d '' file; do copy_to_release "$file"; done < <(
        find "$WINDOWS_WINE_ARTIFACT_DIR" -maxdepth 1 -type f \
            \( -name "ltools-$VERSION-windows-*.exe" \
                -o -name "ltools-$VERSION-windows-*.zip" \
                -o -name 'THIRD-PARTY-LICENSES-windows.zip' \
                -o -name 'ltools-capabilities-windows.json' \
                -o -name 'ltools-terminal-windows.json' \) -print0
        )
        ok 'perfiles Windows GUI y CLI bajo Wine publicados en release/'
    fi
    for file in \
        "$ROOT_DIR/LICENSE" \
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
        jq -e '.platforms.linux.runtime.appimage_requires_fuse == true and
            .platforms.linux.runtime.appimage_extract_override == "APPIMAGE_EXTRACT_AND_RUN=1" and
            (.platforms.windows.runtime // null) == null' \
            "$OUTPUT_DIR/ltools-project.json" >/dev/null \
            || die 'el descriptor de proyecto no declara correctamente el runtime AppImage por plataforma'
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
    release_e2e_args+=(--linux-arch "$ARCH")
    [[ "$APPIMAGE" -eq 0 ]] && release_e2e_args+=(--no-appimage)
    [[ "$PACKAGE" -eq 0 ]] && release_e2e_args+=(--no-package)
    if [[ "$WINDOWS_WINE" -eq 1 ]]; then
        release_e2e_args+=(--windows-arch "${WINDOWS_TARGET%%-*}")
        if [[ "$PACKAGE" -eq 1 ]]; then
            release_e2e_args+=(--require-windows)
        else
            release_e2e_args+=(--require-windows-executables)
        fi
    fi
    if [[ "$SIGNING_PUBLIC_KEY_ENV_ACTIVE" -eq 0 && -r "$SIGNING_PUBLIC_KEY_FILE" ]]; then
        release_e2e_args+=(--signature-public-key-file "$SIGNING_PUBLIC_KEY_FILE")
    fi
    if (( SSH_SIGNING_AVAILABLE )); then
        release_e2e_args+=(--require-ssh-signature
            --ssh-public-key-file "$LTOOLS_SSH_SIGNING_PUBLIC_KEY_FILE"
            --ssh-identity "$LTOOLS_SSH_SIGNING_IDENTITY")
    fi
    run_logged "$ROOT_DIR/tests/release-e2e.sh" "${release_e2e_args[@]}"
    ok 'artefactos release verificados'
    ltools_promote_release_staging || die 'falló la promoción de release; se conservó o restauró la publicación previa'
    RELEASE_DIR="$FINAL_RELEASE_DIR"
    ok "release verificada y promovida: $RELEASE_DIR"
fi

if [[ "$SMOKE" -eq 1 ]]; then
    step 'Ejecutando smoke tests'
    smoke_args=(--binary "$BIN")
    if [[ "$APPIMAGE" -eq 1 ]]; then
        smoke_args+=(--appimage "$APPIMAGE_ARTIFACT" \
            --log "$OUTPUT_DIR/appimage-smoke.log")
    fi
    [[ "$FUSE_REQUIRED" -eq 1 ]] && smoke_args+=(--require-fuse)
    LTOOLS_GUI_CAPTURE_DIR="$OUTPUT_DIR/captures" run_logged "$ROOT_DIR/tests/linux/smoke.sh" --require-gui "${smoke_args[@]}"
    ok 'smoke tests correctos'
fi

if [[ "$E2E" -eq 1 ]]; then
    step 'Ejecutando prueba E2E aislada'
    e2e_args=(--binary "$BIN")
    [[ "$APPIMAGE" -eq 1 ]] && e2e_args+=(--appimage "$APPIMAGE_ARTIFACT")
    run_logged "$ROOT_DIR/tests/linux/e2e.sh" --require-dependencies "${e2e_args[@]}"
    ok 'prueba E2E correcta'
    step 'Ejecutando E2E de la CLI distribuible'
    cli_e2e_args=(--binary "$BIN")
    [[ "$APPIMAGE" -eq 1 ]] && cli_e2e_args+=(--cli-binary "$CLI_APPIMAGE_ARTIFACT")
    run_logged "$ROOT_DIR/tests/linux/cli-e2e.sh" "${cli_e2e_args[@]}"
    ok 'E2E de la CLI distribuible correcta'
fi

if [[ "$MENU_E2E" -eq 1 ]]; then
    step 'Ejecutando E2E de menús y funciones'
    menu_e2e_args=(--binary "$BIN")
    [[ "$APPIMAGE" -eq 1 ]] && menu_e2e_args+=(--appimage "$APPIMAGE_ARTIFACT")
    run_logged "$ROOT_DIR/tests/linux/menu-e2e.sh" --require-gui "${menu_e2e_args[@]}"
    ok 'E2E de menús y funciones correcta'

    if [[ "$SMOKE" -eq 0 ]]; then
        step 'Ejecutando E2E de acciones reales del mapa GUI'
        STORAGE_MAP_TMP="$OUTPUT_DIR/storage-map-$BUILD_ID"
        mkdir -p -- "$STORAGE_MAP_TMP"
        run_logged "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" \
            --require-gui --binary "$BIN" --tmp "$STORAGE_MAP_TMP" --captures "$STORAGE_MAP_TMP/captures"
        ok "acciones reales del mapa GUI correctas; capturas: $STORAGE_MAP_TMP/captures"
    else
        ok 'acciones reales del mapa GUI cubiertas por el smoke; se omite la repetición'
    fi

    step 'Ejecutando E2E de ayudas nativas y correspondencia GUI'
    run_logged "$ROOT_DIR/tests/linux/native-help-e2e.sh" --binary "$BIN"
    ok 'ayudas nativas y correspondencia GUI correctas'
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
    [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]] && [[ -s "$RELEASE_DIR/SHA256SUMS.txt.sig" ]] && printf 'Firma Ed25519: %s\n' "$RELEASE_DIR/SHA256SUMS.txt.sig"
    [[ "$PACKAGE" -eq 1 || "$APPIMAGE" -eq 1 ]] && [[ -s "$RELEASE_DIR/SHA256SUMS.txt.sshsig" ]] && printf 'Firma OpenSSH: %s\n' "$RELEASE_DIR/SHA256SUMS.txt.sshsig"
    [[ "$WINDOWS_WINE" -eq 1 && -f "$WINDOWS_WINE_ARTIFACT" ]] && printf 'Windows validado con Wine/Proton: %s\n' "$WINDOWS_WINE_ARTIFACT"
    [[ "$WINDOWS_WINE" -eq 1 && "$NO_LOG" -eq 0 && -s "$WINDOWS_WINE_LOG" ]] && printf 'Log Windows Wine/Proton: %s\n' "$WINDOWS_WINE_LOG"
    [[ "$NO_LOG" -eq 0 ]] && printf 'Log del build: %s\n' "$LOG_FILE"
    [[ "$NO_LOG" -eq 0 ]] && printf 'Tiempos: %s\n' "$TIMINGS_FILE"
fi

# El resumen usa condiciones `&&` opcionales; fijar explícitamente el estado
# evita que `--no-log` o `--no-appimage` conviertan una build correcta en código 1.
exit 0
