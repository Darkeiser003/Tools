#!/usr/bin/env bash
# E2E aislada de búsqueda de software y operaciones Git.
# Usa gestores simulados y un repositorio temporal; nunca instala paquetes ni
# toca el repositorio desde el que se ejecuta la prueba.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
BIN="$ROOT_DIR/rust/target/release/ltools"
TMP_DIR=""
die() { printf 'SOFTWARE/GIT E2E ERROR: %s\n' "$1" >&2; exit 1; }
ok() { printf '  OK    %s\n' "$1"; }

while (($#)); do
    case "$1" in
        --binary) (($# >= 2)) || die '--binary necesita una ruta'; BIN="$2"; shift ;;
        -h|--help) printf 'Uso: %s [--binary RUTA]\n' "$0"; exit 0 ;;
        *) die "opción desconocida: $1" ;;
    esac
    shift
done

[[ -x "$BIN" ]] || die "no existe el binario: $BIN"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ltools-software-git-e2e.XXXXXX")"
trap 'rm -rf -- "$TMP_DIR"' EXIT
export HOME="$TMP_DIR/home"
export XDG_STATE_HOME="$HOME/.local/state"
export LTOOLS_LANG=es
mkdir -p "$HOME" "$XDG_STATE_HOME"

STUB_DIR="$TMP_DIR/stores"
mkdir -p "$STUB_DIR"
cat > "$STUB_DIR/pacman" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
if [[ "${1:-}" == "-Ss" ]]; then
    printf 'extra/fake-tool 1.2.3-1\nextra/fake-tool - paquete sintético\n'
    exit 0
fi
printf '%s\n' "${LTOOLS_FAKE_INSTALL_LOG:?}" > /dev/null
touch "$LTOOLS_FAKE_INSTALL_LOG"
EOF
chmod +x "$STUB_DIR/pacman"

REAL_PATH="${PATH:-/usr/bin:/bin}"
SEARCH_JSON="$TMP_DIR/search.json"
PATH="$STUB_DIR:$REAL_PATH" "$BIN" software search fake-tool --manager pacman --format json > "$SEARCH_JSON"
grep -Fq 'ltools-package-search-v1' "$SEARCH_JSON" || die 'la búsqueda no generó el contrato JSON'
grep -Fq 'fake-tool' "$SEARCH_JSON" || die 'la búsqueda no devolvió el candidato sintético'
if command -v jq >/dev/null 2>&1; then
    jq -e '.schema == "ltools-package-search-v1" and (.candidates | length >= 1) and .candidates[0].manager == "pacman" and .candidates[0].id == "extra/fake-tool"' "$SEARCH_JSON" >/dev/null || die 'el JSON de búsqueda no supera la validación estructural'
fi
ok 'búsqueda en store simulada y contrato JSON'

INSTALL_LOG="$TMP_DIR/install.marker"
PATH="$STUB_DIR:$REAL_PATH" LTOOLS_FAKE_INSTALL_LOG="$INSTALL_LOG" "$BIN" --dry-run software install fake-tool --manager pacman --candidate 1 --yes > "$TMP_DIR/install.out"
grep -Fq 'pacman -S --needed' "$TMP_DIR/install.out" || die 'dry-run no mostró el comando nativo de instalación'
[[ ! -e "$INSTALL_LOG" ]] || die 'dry-run ejecutó el gestor simulado'
ok 'instalación seleccionada con --dry-run no modifica el sistema'

MANAGER_STUB_DIR="$TMP_DIR/package-managers"
mkdir -p "$MANAGER_STUB_DIR"
for manager_name in brew flatpak pamac; do
    cat > "$MANAGER_STUB_DIR/$manager_name" <<'EOF'
#!/bin/sh
printf '%s %s\n' "${0##*/}" "$*" >> "${LTOOLS_FAKE_MANAGER_LOG:?}"
if [ "${0##*/}" = flatpak ] && [ "${1:-}" = list ]; then
    case "${2:-}" in
        --user) printf '%s\n' "${LTOOLS_FLATPAK_USER_APP:-}" ;;
        --system) printf '%s\n' "${LTOOLS_FLATPAK_SYSTEM_APP:-}" ;;
    esac
    exit 0
fi
if [ "${0##*/}" = flatpak ] && [ "${1#--installation=}" != "$1" ]; then
    shift
    if [ "${1:-}" = list ]; then
        printf '%s\n' "${LTOOLS_FLATPAK_CUSTOM_APP:-}"
        exit 0
    fi
fi
exit "${LTOOLS_FAKE_MANAGER_STATUS:-0}"
EOF
    chmod +x "$MANAGER_STUB_DIR/$manager_name"
done
cat > "$MANAGER_STUB_DIR/sudo" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "${LTOOLS_FAKE_SUDO_LOG:?}"
exit 0
EOF
chmod +x "$MANAGER_STUB_DIR/sudo"

MANAGER_LOG="$TMP_DIR/package-manager.marker"
SUDO_LOG="$TMP_DIR/sudo.marker"
: > "$MANAGER_LOG"
set +e
env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    LTOOLS_FAKE_SUDO_LOG="$SUDO_LOG" "$BIN" clean --package ambiguous-name \
    > "$TMP_DIR/manager-ambiguous.out" 2>&1
MANAGER_AMBIGUOUS_STATUS=$?
set -e
[[ "$MANAGER_AMBIGUOUS_STATUS" -ne 0 ]] ||
    die 'la limpieza eligió un gestor de paquetes sin que el usuario lo indicara'
grep -Fq 'especifica --manager' "$TMP_DIR/manager-ambiguous.out" ||
    die 'la limpieza no explicó que debe elegirse el gestor'
! grep -Fq 'uninstall' "$MANAGER_LOG" ||
    die 'la limpieza ejecutó una desinstalación con gestor ambiguo'

printf 'y\n' | env PATH="$MANAGER_STUB_DIR" \
    LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" LTOOLS_FAKE_SUDO_LOG="$SUDO_LOG" \
    "$BIN" clean --package fake-formula --manager brew > "$TMP_DIR/brew-remove.out"
grep -Fq 'brew uninstall -- fake-formula' "$MANAGER_LOG" ||
    die 'la desinstalación Brew no ejecutó su gestor'
[[ ! -e "$SUDO_LOG" ]] || die 'Brew se ejecutó como root en vez de conservar el perfil del usuario'

: > "$MANAGER_LOG"
printf 'y\n' | env PATH="$MANAGER_STUB_DIR" \
    LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" LTOOLS_FAKE_SUDO_LOG="$SUDO_LOG" \
    LTOOLS_FLATPAK_USER_APP=fake.app \
    "$BIN" clean --package fake.app --manager flatpak > "$TMP_DIR/flatpak-remove.out"
grep -Fq 'flatpak uninstall --delete-data --user -- fake.app' "$MANAGER_LOG" ||
    die 'la desinstalación Flatpak no seleccionó la instalación de usuario detectada'
[[ ! -e "$SUDO_LOG" ]] || die 'Flatpak se ejecutó como root en vez de conservar la instalación del usuario'
ok 'desinstalación Brew y Flatpak conserva identidad y gestor nativo'

: > "$MANAGER_LOG"
printf 'y\n' | env PATH="$MANAGER_STUB_DIR" \
    LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" LTOOLS_FAKE_SUDO_LOG="$SUDO_LOG" \
    "$BIN" clean --package fake-pamac-package --manager pamac > "$TMP_DIR/pamac-remove.out"
grep -Fq 'pamac remove -- fake-pamac-package' "$MANAGER_LOG" ||
    die 'la desinstalación Pamac no ejecutó su interfaz nativa'
[[ ! -e "$SUDO_LOG" ]] || die 'Pamac se ejecutó como root en vez de conservar su autorización nativa'
ok 'Pamac conserva su ámbito de usuario y su autorización nativa'

: > "$MANAGER_LOG"
rm -f -- "$SUDO_LOG"
printf 'y\ny\n' | env PATH="$MANAGER_STUB_DIR" \
    LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" LTOOLS_FAKE_SUDO_LOG="$SUDO_LOG" \
    "$BIN" clean --package-caches > "$TMP_DIR/pamac-cache-clean.out"
grep -Fq 'pamac clean --keep 3' "$MANAGER_LOG" ||
    die 'la limpieza no usa la opción conservadora de caché de Pamac'
[[ ! -e "$SUDO_LOG" ]] || die 'la limpieza de Pamac se ejecutó como root'
ok 'Pamac limpia su caché conservando las tres últimas versiones y su identidad'

: > "$MANAGER_LOG"
set +e
env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    "$BIN" clean --package fake-formula --manager brew --cascade \
    > "$TMP_DIR/cascade-unsupported.out" 2>&1
CASCADE_STATUS=$?
set -e
[[ "$CASCADE_STATUS" -ne 0 ]] || die '--cascade fue aceptado para Brew aunque solo aplica a Pacman'
grep -Fq -- '--cascade solo se admite con --manager pacman' "$TMP_DIR/cascade-unsupported.out" ||
    die 'el rechazo de --cascade no explicó la restricción del gestor'
! grep -Fq 'uninstall' "$MANAGER_LOG" || die '--cascade ejecutó una desinstalación con un gestor no compatible'
ok '--cascade incompatible se rechaza antes de ejecutar un gestor'

: > "$MANAGER_LOG"
set +e
env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    "$BIN" clean --manager brew > "$TMP_DIR/manager-without-package.out" 2>&1
MANAGER_WITHOUT_PACKAGE_STATUS=$?
set -e
[[ "$MANAGER_WITHOUT_PACKAGE_STATUS" -ne 0 ]] ||
    die '--manager sin --package/--orphans devolvió éxito'
grep -Fq -- '--manager solo se puede usar con --package o --orphans' \
    "$TMP_DIR/manager-without-package.out" ||
    die '--manager sin operación no explica el uso correcto'
! grep -Fq 'uninstall' "$MANAGER_LOG" ||
    die '--manager sin paquete ejecutó una operación'

: > "$MANAGER_LOG"
printf 'y\n' | env PATH="$MANAGER_STUB_DIR" \
    LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" LTOOLS_FAKE_SUDO_LOG="$SUDO_LOG" \
    LTOOLS_FLATPAK_SYSTEM_APP=system.app \
    "$BIN" clean --package system.app --manager flatpak --scope system \
    > "$TMP_DIR/flatpak-system-remove.out"
grep -Fq 'flatpak uninstall --delete-data --system -- system.app' "$MANAGER_LOG" ||
    die 'Flatpak no respetó --scope system al desinstalar'
[[ ! -e "$SUDO_LOG" ]] || die 'Flatpak system se ejecutó como root en vez de delegar en Polkit'

: > "$MANAGER_LOG"
set +e
env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    LTOOLS_FLATPAK_USER_APP=duplicate.app LTOOLS_FLATPAK_SYSTEM_APP=duplicate.app \
    "$BIN" clean --package duplicate.app --manager flatpak \
    > "$TMP_DIR/flatpak-ambiguous.out" 2>&1
FLATPAK_AMBIGUOUS_STATUS=$?
set -e
[[ "$FLATPAK_AMBIGUOUS_STATUS" -ne 0 ]] ||
    die 'Flatpak eliminó una referencia presente en ambos ámbitos sin pedir selección'
grep -Fq 'especifica --scope' "$TMP_DIR/flatpak-ambiguous.out" ||
    die 'Flatpak no explicó cómo resolver la referencia ambigua'
! grep -Fq 'flatpak uninstall' "$MANAGER_LOG" ||
    die 'Flatpak ejecutó la eliminación ambigua en algún ámbito'
ok 'referencias Flatpak duplicadas exigen un ámbito explícito'

set +e
printf 'y\n' | env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    LTOOLS_FAKE_MANAGER_STATUS=23 "$BIN" clean --package broken-formula --manager brew \
    > "$TMP_DIR/brew-remove-failed.out" 2>&1
REMOVE_STATUS=$?
set -e
[[ "$REMOVE_STATUS" -ne 0 ]] ||
    die 'una desinstalación Brew fallida devolvió estado de éxito'
grep -Fq 'no se completó la eliminación' "$TMP_DIR/brew-remove-failed.out" ||
    die 'el fallo de desinstalación Brew no se explicó al usuario'
grep -Fq $'package-remove\tbroken-formula\tfailed' \
    "$XDG_STATE_HOME/ltools/plans/plan-rust-clean.tsv" ||
    die 'el plan no registró como fallida la desinstalación Brew'
ok 'fallo de desinstalación propaga error al CLI'

: > "$MANAGER_LOG"
printf 'y\n' | env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    LTOOLS_FAKE_SUDO_LOG="$SUDO_LOG" "$BIN" clean --package-caches \
    > "$TMP_DIR/brew-cache.out"
grep -Fq 'brew cleanup' "$MANAGER_LOG" ||
    die 'la limpieza de caché Brew no ejecutó su gestor'
[[ ! -e "$SUDO_LOG" ]] || die 'la limpieza de caché Brew se ejecutó como root'

set +e
printf 'y\n' | env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    LTOOLS_FAKE_MANAGER_STATUS=23 "$BIN" clean --package-caches \
    > "$TMP_DIR/brew-cache-failed.out" 2>&1
CACHE_STATUS=$?
set -e
[[ "$CACHE_STATUS" -ne 0 ]] ||
    die 'una limpieza de caché Brew fallida devolvió estado de éxito'
grep -Fq $'package-cache-clean\tbrew-cache\tfailed' \
    "$XDG_STATE_HOME/ltools/plans/plan-rust-clean.tsv" ||
    die 'el plan no registró como fallida la limpieza de caché Brew'
ok 'limpieza Brew conserva identidad y propaga fallos'

: > "$MANAGER_LOG"
printf 'y\n' | env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    LTOOLS_FAKE_SUDO_LOG="$SUDO_LOG" "$BIN" clean --flatpak-unused \
    > "$TMP_DIR/flatpak-clean.out"
grep -Fq 'flatpak uninstall --unused --user' "$MANAGER_LOG" ||
    die 'Flatpak no limpió runtimes sin uso de usuario'
grep -Fq 'flatpak uninstall --unused --system' "$MANAGER_LOG" ||
    die 'Flatpak no limpió runtimes sin uso del sistema'
[[ ! -e "$SUDO_LOG" ]] || die 'la limpieza Flatpak se ejecutó como root'
ok 'limpieza Flatpak cubre instalaciones de usuario y sistema sin cambiar de identidad'

: > "$MANAGER_LOG"
set +e
printf 'y\n' | env PATH="$MANAGER_STUB_DIR" LTOOLS_FAKE_MANAGER_LOG="$MANAGER_LOG" \
    LTOOLS_FAKE_MANAGER_STATUS=23 "$BIN" clean --flatpak-unused \
    > "$TMP_DIR/flatpak-clean-failed.out" 2>&1
FLATPAK_STATUS=$?
set -e
[[ "$FLATPAK_STATUS" -ne 0 ]] ||
    die 'la limpieza Flatpak fallida devolvió estado de éxito'
grep -Fq $'flatpak-unused\tflatpak:user\tfailed' \
    "$XDG_STATE_HOME/ltools/plans/plan-rust-clean.tsv" ||
    die 'el plan no registró como fallida la limpieza Flatpak de usuario'
grep -Fq $'flatpak-unused\tflatpak:system\tfailed' \
    "$XDG_STATE_HOME/ltools/plans/plan-rust-clean.tsv" ||
    die 'el plan no registró como fallida la limpieza Flatpak de sistema'
ok 'limpieza de Flatpak propaga fallos al CLI'

if command -v git >/dev/null 2>&1; then
    REPO="$TMP_DIR/repository"
    git init -q "$REPO"
    git -C "$REPO" config user.name 'LTools E2E'
    git -C "$REPO" config user.email 'ltools-e2e@example.invalid'
    printf 'fixture\n' > "$REPO/file.txt"
    git -C "$REPO" add file.txt
    git -C "$REPO" commit -q -m fixture
    git -C "$REPO" status --porcelain >/dev/null
    PATH="$REAL_PATH" "$BIN" git status --repo "$REPO" > "$TMP_DIR/git-status.out"
    grep -Fq '##' "$TMP_DIR/git-status.out" || die 'git status no mostró la rama'
    PATH="$REAL_PATH" "$BIN" --dry-run git fetch --repo "$REPO" --prune --yes > "$TMP_DIR/git-fetch.out"
    grep -Fq 'git -C' "$TMP_DIR/git-fetch.out" || die 'git fetch dry-run no mostró su plan'
    PATH="$REAL_PATH" "$BIN" --dry-run git clone https://github.com/example/example.git "$TMP_DIR/clone" --yes > "$TMP_DIR/git-clone.out"
    grep -Fq 'git clone' "$TMP_DIR/git-clone.out" || die 'git clone dry-run no mostró su plan'
    PATH="$REAL_PATH" "$BIN" --dry-run git log --repo "$REPO" --limit 5 > "$TMP_DIR/git-log.out"
    grep -Fq 'fixture' "$TMP_DIR/git-log.out" || die 'git log no devolvió el historial del repositorio'
    PATH="$REAL_PATH" "$BIN" --dry-run git add --repo "$REPO" --all --yes > "$TMP_DIR/git-add.out"
    grep -Fq 'git -C' "$TMP_DIR/git-add.out" || die 'git add no mostró su plan'
    PATH="$REAL_PATH" "$BIN" --dry-run git commit --repo "$REPO" --message 'commit e2e' --all --yes > "$TMP_DIR/git-commit.out"
    grep -Fq 'git -C' "$TMP_DIR/git-commit.out" || die 'git commit no mostró su plan'
    PATH="$REAL_PATH" "$BIN" --dry-run git push --repo "$REPO" --remote origin --branch main --yes > "$TMP_DIR/git-push.out"
    grep -Fq 'git -C' "$TMP_DIR/git-push.out" || die 'git push no mostró su plan'
    PATH="$REAL_PATH" "$BIN" --dry-run git branch --repo "$REPO" --switch main --yes > "$TMP_DIR/git-branch.out"
    grep -Fq 'git -C' "$TMP_DIR/git-branch.out" || die 'git branch no mostró su plan'
    PATH="$REAL_PATH" "$BIN" --dry-run git tag --repo "$REPO" --name e2e-v1 --message 'tag e2e' --yes > "$TMP_DIR/git-tag.out"
    grep -Fq 'git -C' "$TMP_DIR/git-tag.out" || die 'git tag no mostró su plan'
    PATH="$REAL_PATH" "$BIN" git diagnose --repo "$REPO" > "$TMP_DIR/git-diagnose.out"
    grep -Fq 'fsck correcto' "$TMP_DIR/git-diagnose.out" || die 'el diagnóstico Git no validó objetos'

    INDEX_REPO="$TMP_DIR/index-repair"
    git clone -q "$REPO" "$INDEX_REPO"
    rm -- "$INDEX_REPO/.git/index"
    PATH="$REAL_PATH" "$BIN" --dry-run git repair --repo "$INDEX_REPO" > "$TMP_DIR/git-repair-dry-run.out"
    [[ ! -e "$INDEX_REPO/.git/index" ]] || die 'git repair dry-run modificó el índice'
    PATH="$REAL_PATH" "$BIN" git repair --repo "$INDEX_REPO" --yes > "$TMP_DIR/git-repair.out"
    [[ -s "$INDEX_REPO/.git/index" ]] || die 'git repair no reconstruyó el índice ausente'
    grep -Fq 'Índice reconstruido' "$TMP_DIR/git-repair.out" || die 'git repair no informó verificación del índice'
    [[ "$(cat "$INDEX_REPO/file.txt")" == fixture ]] || die 'git repair alteró el árbol de trabajo'
    ok 'Git diagnose y reconstrucción del índice con respaldo/validación sin checkout'

    # Se envuelve solo el transporte del fixture para probar la recuperación
    # completa desde un remoto HTTPS simulado, sin red ni cambios externos.
    REMOTE="$TMP_DIR/remote.git"
    git clone --bare -q "$REPO" "$REMOTE"
    RECOVERY="$TMP_DIR/recovery-worktree"
    mkdir -p "$RECOVERY"
    printf 'local modification\n' > "$RECOVERY/file.txt"
    printf 'untracked data\n' > "$RECOVERY/local-only.txt"
    GIT_BRIDGE="$TMP_DIR/git-bridge"
    mkdir -p "$GIT_BRIDGE"
    cat > "$GIT_BRIDGE/git" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
if [[ "${1:-}" == clone ]]; then
    shift
    clone_args=()
    while (($#)); do
        case "$1" in
            --no-checkout) clone_args+=("$1"); shift ;;
            --origin|--branch) clone_args+=("$1" "$2"); shift 2 ;;
            https://example.invalid/org/project.git) clone_args+=("$LTOOLS_LOCAL_REMOTE"); shift ;;
            *) clone_args+=("$1"); shift ;;
        esac
    done
    exec "$LTOOLS_REAL_GIT" clone "${clone_args[@]}"
fi
exec "$LTOOLS_REAL_GIT" "$@"
EOF
    chmod +x "$GIT_BRIDGE/git"
    RECOVERY_URL='https://example.invalid/org/project.git'
    env PATH="$GIT_BRIDGE:$REAL_PATH" LTOOLS_REAL_GIT="$(command -v git)" \
        LTOOLS_LOCAL_REMOTE="$REMOTE" "$BIN" --dry-run git repair \
        --repo "$RECOVERY" --remote "$RECOVERY_URL" > "$TMP_DIR/git-directory-repair-dry-run.out"
    [[ ! -e "$RECOVERY/.git" ]] || die 'la recuperación .git dry-run instaló metadata'
    env PATH="$GIT_BRIDGE:$REAL_PATH" LTOOLS_REAL_GIT="$(command -v git)" \
        LTOOLS_LOCAL_REMOTE="$REMOTE" "$BIN" git repair --repo "$RECOVERY" \
        --remote "$RECOVERY_URL" --yes > "$TMP_DIR/git-directory-repair.out"
    [[ -d "$RECOVERY/.git" ]] || die 'no se recuperó .git desde el remoto'
    [[ "$(cat "$RECOVERY/file.txt")" == $'local modification' ]] || die 'la recuperación .git sobrescribió un archivo local'
    [[ "$(cat "$RECOVERY/local-only.txt")" == $'untracked data' ]] || die 'la recuperación .git eliminó un archivo sin seguimiento'
    grep -Fq ' M file.txt' "$TMP_DIR/git-directory-repair.out" || die 'la recuperación no hizo visibles los cambios locales'
    [[ -z "$(find "$TMP_DIR" -maxdepth 1 -name '.ltools-git-recovery-*' -print -quit)" ]] || die 'quedó un staging de recuperación tras el éxito'
    ok 'recuperación .git desde remoto explícito valida metadata y preserva todo el árbol local'

    GH_STUB="$TMP_DIR/gh-stub"
    mkdir -p "$GH_STUB"
    cat > "$GH_STUB/gh" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
printf '%s\n' "$@" >> "${LTOOLS_GH_ARGV_LOG:?}"
if [[ "${1:-}" == help || "${1:-}" == --help ]]; then
    printf 'GitHub CLI test stub help\n'
fi
EOF
    chmod +x "$GH_STUB/gh"
    GH_ARGV_LOG="$TMP_DIR/gh-argv.log"
    : > "$GH_ARGV_LOG"
    env PATH="$GH_STUB:$REAL_PATH" LTOOLS_GH_ARGV_LOG="$GH_ARGV_LOG" \
        "$BIN" git gh native project list --owner example --yes --ltools-confirmed \
        > "$TMP_DIR/gh-native.out"
    expected_gh_args="$TMP_DIR/gh-expected-argv.log"
    printf '%s\n' project list --owner example --yes > "$expected_gh_args"
    cmp -s "$expected_gh_args" "$GH_ARGV_LOG" || die 'gh native alteró o eliminó argumentos nativos de la versión instalada'
    grep -Fq 'gh project …' "$TMP_DIR/gh-native.out" || die 'gh native no identificó el comando avanzado'
    : > "$GH_ARGV_LOG"
    set +e
    env PATH="$GH_STUB:$REAL_PATH" LTOOLS_GH_ARGV_LOG="$GH_ARGV_LOG" \
        "$BIN" git gh native auth token > "$TMP_DIR/gh-token-blocked.out" 2>&1
    GH_TOKEN_STATUS=$?
    set -e
    [[ "$GH_TOKEN_STATUS" -ne 0 && ! -s "$GH_ARGV_LOG" ]] || die 'gh native permitió mostrar el token de autenticación'
    grep -Fq 'bloquea `gh auth token`' "$TMP_DIR/gh-token-blocked.out" || die 'el bloqueo de gh auth token no explicó el motivo'
    ok 'GitHub CLI passthrough conserva argumentos versionados y bloquea extracción de tokens'

    PATH="$REAL_PATH" "$BIN" --dry-run git release --repo example/example --tag e2e-v1 --title 'E2E' --notes 'Prueba' --yes > "$TMP_DIR/gh-release.out"
    grep -Fq 'gh release create' "$TMP_DIR/gh-release.out" || die 'gh release no mostró su plan'
    ok 'Git status/log/fetch/clone/add/commit/push/branch/tag/release con planes de prueba'
else
    printf '  SKIP  Git no está instalado en el entorno de prueba\n'
fi

printf 'Software/Git E2E completado correctamente.\n'
