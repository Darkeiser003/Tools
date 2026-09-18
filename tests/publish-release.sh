#!/usr/bin/env bash
# Pruebas aisladas del staging/intercambio de release, sin compilar LTools.
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
source "$ROOT_DIR/scripts/lib/publish.sh"
workspace="$(mktemp -d "${TMPDIR:-/tmp}/ltools-publish-release.XXXXXX")"
trap 'ltools_cleanup_release_staging; rm -rf -- "$workspace"' EXIT

# El wrapper se define antes de cualquier llamada para que ShellCheck pueda
# analizar también las ramas que simulan un fallo de `mv`. En el resto del
# fixture delega íntegramente en el comando real.
simulate_move_failure=0
mv() {
    if [[ "$simulate_move_failure" == 1 && ( "${1:-}" == --exchange || "${1:-}" == -Tn ) ]]; then
        return 1
    fi
    command mv "$@"
}

destination="$workspace/release"
mkdir -p -- "$destination/user-data"
printf 'old\n' >"$destination/old-release.txt"
printf 'keep\n' >"$destination/user-data/keep.txt"
printf 'hidden\n' >"$destination/.user-note"

ltools_create_release_staging "$destination"
stage="$LTOOLS_RELEASE_STAGING"
[[ -f "$destination/old-release.txt" && -f "$stage/old-release.txt" ]] || {
    printf 'ERROR: staging alteró o no copió la release previa.\n' >&2; exit 1;
}
printf 'new\n' >"$stage/new-release.txt"
ltools_promote_release_staging
[[ -f "$destination/new-release.txt" && -f "$destination/user-data/keep.txt" && -f "$destination/.user-note" ]] || {
    printf 'ERROR: promoción no conservó los datos ajenos o los nuevos artefactos.\n' >&2; exit 1;
}
[[ ! -e "$stage" ]] || { printf 'ERROR: staging anterior no se retiró tras promover.\n' >&2; exit 1; }

# Un reemplazo concurrente del directorio debe rechazarse sin sobrescribirlo.
ltools_create_release_staging "$destination"
stage="$LTOOLS_RELEASE_STAGING"
command mv -- "$destination" "$workspace/original-release"
mkdir -- "$destination"
printf 'concurrent\n' >"$destination/concurrent.txt"
if ltools_promote_release_staging; then
    printf 'ERROR: se aceptó un destino release sustituido concurrentemente.\n' >&2; exit 1
fi
[[ -f "$destination/concurrent.txt" ]] || { printf 'ERROR: se alteró el destino concurrente.\n' >&2; exit 1; }
ltools_cleanup_release_staging
rm -rf -- "$destination"
command mv -- "$workspace/original-release" "$destination"

# Fuerza la ruta de fallback y simula un fallo de instalación: debe restaurar
# la release anterior y dejar el staging nuevo recuperable.
ltools_create_release_staging "$destination"
stage="$LTOOLS_RELEASE_STAGING"
printf 'failed-new\n' >"$stage/failed-new.txt"
simulate_move_failure=1
if ltools_promote_release_staging; then
    printf 'ERROR: se informó éxito a pesar del fallo de promoción simulado.\n' >&2; exit 1
fi
simulate_move_failure=0
[[ -f "$destination/old-release.txt" && -f "$destination/new-release.txt" &&
    ! -f "$destination/failed-new.txt" && -f "$stage/failed-new.txt" ]] || {
    printf 'ERROR: el fallo no restauró la release previa y conservó el staging nuevo.\n' >&2; exit 1;
}
ltools_cleanup_release_staging

linked_destination="$workspace/release-with-link"
mkdir -- "$linked_destination"
printf 'do not follow\n' >"$workspace/external-target"
ln -s -- "$workspace/external-target" "$linked_destination/ltools-9.8.7-linux-x86_64.tar.gz"
if ltools_create_release_staging "$linked_destination"; then
    printf 'ERROR: se aceptó una release con enlaces simbólicos.\n' >&2; exit 1
fi
[[ "$(cat -- "$workspace/external-target")" == 'do not follow' ]] || {
    printf 'ERROR: la comprobación de enlaces alteró un destino externo.\n' >&2; exit 1;
}

new_destination="$workspace/new-release"
ltools_create_release_staging "$new_destination"
printf 'first\n' >"$LTOOLS_RELEASE_STAGING/first.txt"
ltools_promote_release_staging
[[ -f "$new_destination/first.txt" ]] || { printf 'ERROR: no se publicó un destino nuevo.\n' >&2; exit 1; }

# La E2E del contrato también debe aceptar un fichero oculto ajeno (sin barra
# ni traversal) en la lista de checksums y detectar cualquier manipulación.
if command -v jq >/dev/null 2>&1; then
    release_fixture="$workspace/release-e2e"
    mkdir -- "$release_fixture"
    fixture_version='9.8.7'
    fixture_artifact="ltools-$fixture_version-linux-x86_64.tar.gz"
    cp -- "$ROOT_DIR/LICENSE" "$release_fixture/LICENSE"
    printf 'binary fixture\n' >"$release_fixture/$fixture_artifact"
    printf 'user data\n' >"$release_fixture/.user-note"
    printf 'ignored temp\n' >"$release_fixture/.build-fragment.tmp"
    printf 'ignored backup\n' >"$release_fixture/.previous.bak"
    printf 'case-sensitive uppercase\n' >"$release_fixture/case-sensitive.TMP"
    for json in ltools-capabilities.json ltools-actions.json ltools-terminal.json ltools-project.json \
        ltools-capabilities.schema.json ltools-actions.schema.json ltools-terminal.schema.json \
        ltools-project.schema.json ltools-release.schema.json; do
        printf '{}\n' >"$release_fixture/$json"
    done
    jq -n '{schema:"ltools-capabilities-v1", application:"LTools", version:$version, platform:"linux", actions:[{id:"linux.fixture.check", legacyId:"fixture-check", actionId:"linux.fixture.check", actionKey:"fixture.check", qualifiedActionKey:"linux.fixture.check", canonicalKey:"linux.fixture.check", scope:"fixture", operation:"fixture-check", label:"Fixture check", shortLabel:"Fixture", displayName:"Linux · Fixture · Fixture check", menuPath:["Fixture","Fixture check"], group:"Fixture", description:"Acción de prueba", command:"fixture", executable:"ltools", args:[], shell:"none", workingDirectory:"current", terminal:true, interactive:false, requiresAdmin:false, confirmation:"none", safe:true, supports:["dry-run"], requiresCommands:[]}]}' --arg version "$fixture_version" \
        >"$release_fixture/ltools-capabilities.json"
    jq -n '{schema:"ltools-terminal-integration-v1", application:"LTools", version:$version, platform:"linux", actions:[{id:"linux.fixture.check", legacyId:"fixture-check", actionId:"linux.fixture.check", actionKey:"fixture.check", qualifiedActionKey:"linux.fixture.check", canonicalKey:"linux.fixture.check", scope:"fixture", operation:"fixture-check", label:"Fixture check", shortLabel:"Fixture", displayName:"Linux · Fixture · Fixture check", menuPath:["Fixture","Fixture check"], group:"Fixture", description:"Acción de prueba", command:"fixture", executable:"ltools", args:[], shell:"none", workingDirectory:"current", terminal:true, interactive:false, requiresAdmin:false, confirmation:"none", safe:true, supports:["dry-run"], requiresCommands:[]}]}' --arg version "$fixture_version" \
        >"$release_fixture/ltools-terminal.json"
    jq -n '{schema:"ltools-actions-v1", platform:"linux", safety:{excluded_defaults:[], target_selection:"explicit-only"}, actions:[{id:"linux.fixture.check", legacyId:"fixture.check", actionId:"linux.fixture.check", actionKey:"fixture.check", qualifiedActionKey:"linux.fixture.check", canonicalKey:"linux.fixture.check", scope:"fixture", operation:"fixture-check", label:"Fixture check", shortLabel:"Fixture", displayName:"Linux · Fixture · Fixture check", menuPath:["Fixture","Fixture check"], group:"fixture", description:"Acción de prueba", category:"fixture", command:"fixture", args:[], target:"none", targetPolicy:"none", invocation:{executable:"ltools",args:["actions","run","linux.fixture.check"],target:"none"}, mutating:false, confirmation:"none", safe:true, profile:"safe-default", aliases:[], supports:["dry-run","plan"]}]}' \
        >"$release_fixture/ltools-actions.json"
    jq -n '{additionalProperties:false, properties:{schema:{}, application:{}, version:{}, platform:{}, actions:{}, environment:{required:["cli_profile"]}, terminal_integration:{required:["working_directory"]}}, "$defs":{ui_context:{}, entrypoint:{}, distribution_entry:{}}}' \
        >"$release_fixture/ltools-capabilities.schema.json"
    jq -n '{additionalProperties:false, properties:{schema:{}, application:{}, version:{}, platform:{}, actions:{}}}' \
        >"$release_fixture/ltools-terminal.schema.json"
    fixture_hash="$(sha256sum -- "$release_fixture/$fixture_artifact" | awk '{print $1}')"
    fixture_size="$(stat -c '%s' -- "$release_fixture/$fixture_artifact")"
    jq -n --arg version "$fixture_version" --arg filename "$fixture_artifact" \
        --arg hash "$fixture_hash" --argjson size "$fixture_size" \
        '{schema:"ltools-release-v1",application:"LTools",version:$version,hash_algorithm:"sha256",artifacts:[{platform:"linux",architecture:"x86_64",kind:"tarball",filename:$filename,size_bytes:$size,sha256:$hash,executable:false}]}' \
        >"$release_fixture/ltools-release.json"
    while IFS= read -r -d '' file; do
        printf '%s  %s\n' "$(sha256sum -- "$file" | awk '{print $1}')" "${file##*/}"
    done < <(find "$release_fixture" -maxdepth 1 -type f ! -name 'SHA256SUMS.txt' ! -name 'SHA256SUMS.txt.sig' ! -name 'SHA256SUMS.txt.sshsig' ! -name '*.tmp' ! -name '*.bak' -print0 | sort -z) \
        >"$release_fixture/SHA256SUMS.txt"
    bash "$ROOT_DIR/tests/release-e2e.sh" --release-dir "$release_fixture" \
        --version "$fixture_version" --no-appimage --no-package >/dev/null
    ltools_create_release_staging "$release_fixture"
    printf 'broken staging\n' >>"$LTOOLS_RELEASE_STAGING/$fixture_artifact"
    if bash "$ROOT_DIR/tests/release-e2e.sh" --release-dir "$LTOOLS_RELEASE_STAGING" \
        --version "$fixture_version" --no-appimage --no-package >/dev/null 2>&1; then
        printf 'ERROR: E2E aceptó un staging manipulado.\n' >&2; exit 1
    fi
    ltools_cleanup_release_staging
    [[ "$(sha256sum -- "$release_fixture/$fixture_artifact" | awk '{print $1}')" == "$fixture_hash" ]] || {
        printf 'ERROR: fallar la E2E cambió la release final anterior.\n' >&2; exit 1;
    }
    ln -s -- "$release_fixture" "$workspace/release-e2e-link"
    if bash "$ROOT_DIR/tests/release-e2e.sh" --release-dir "$workspace/release-e2e-link" \
        --version "$fixture_version" --no-appimage --no-package >/dev/null 2>&1; then
        printf 'ERROR: E2E de release aceptó un directorio release que es enlace simbólico.\n' >&2; exit 1
    fi
    mv -- "$release_fixture/$fixture_artifact" "$workspace/release-artifact-original"
    ln -s -- "$workspace/release-artifact-original" "$release_fixture/$fixture_artifact"
    if bash "$ROOT_DIR/tests/release-e2e.sh" --release-dir "$release_fixture" \
        --version "$fixture_version" --no-appimage --no-package >/dev/null 2>&1; then
        printf 'ERROR: E2E de release aceptó un artefacto que es enlace simbólico.\n' >&2; exit 1
    fi
    rm -- "$release_fixture/$fixture_artifact"
    mv -- "$workspace/release-artifact-original" "$release_fixture/$fixture_artifact"
    printf 'tampered\n' >>"$release_fixture/$fixture_artifact"
    if bash "$ROOT_DIR/tests/release-e2e.sh" --release-dir "$release_fixture" \
        --version "$fixture_version" --no-appimage --no-package >/dev/null 2>&1; then
        printf 'ERROR: E2E de release aceptó un artefacto manipulado.\n' >&2; exit 1
    fi
else
    printf 'SKIP: jq no está instalado; se omitió el fixture E2E de release.\n'
fi

printf 'Staging, preservación, concurrencia, rollback, enlaces y E2E de hashes validados.\n'
