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
    ok 'Git status, log, fetch, clone, add, commit, push, branch y tag con dry-run'
    if command -v gh >/dev/null 2>&1; then
        PATH="$REAL_PATH" "$BIN" --dry-run git release --repo example/example --tag e2e-v1 --title 'E2E' --notes 'Prueba' --yes > "$TMP_DIR/gh-release.out"
        grep -Fq 'gh release create' "$TMP_DIR/gh-release.out" || die 'gh release no mostró su plan'
        PATH="$REAL_PATH" "$BIN" --dry-run git gh repo --repo example/example > "$TMP_DIR/gh-repo.out"
        grep -Fq 'gh repo view' "$TMP_DIR/gh-repo.out" || die 'gh repo no mostró su plan'
        PATH="$REAL_PATH" "$BIN" --dry-run git gh prs --repo example/example > "$TMP_DIR/gh-prs.out"
        grep -Fq 'gh pr list' "$TMP_DIR/gh-prs.out" || die 'gh prs no mostró su plan'
        PATH="$REAL_PATH" "$BIN" --dry-run git gh releases --repo example/example > "$TMP_DIR/gh-releases.out"
        grep -Fq 'gh release list' "$TMP_DIR/gh-releases.out" || die 'gh releases no mostró su plan'
        PATH="$REAL_PATH" "$BIN" --dry-run git gh auth-status > "$TMP_DIR/gh-auth.out"
        grep -Fq 'gh auth status' "$TMP_DIR/gh-auth.out" || die 'gh auth status no mostró su plan'
        ok 'GitHub CLI: release, repo, pull requests, releases y autenticación con dry-run'
    else
        printf '  SKIP  gh no está instalado; se omiten las rutas GitHub\n'
    fi
else
    printf '  SKIP  Git no está instalado en el entorno de prueba\n'
fi

printf 'Software/Git E2E completado correctamente.\n'
