#!/usr/bin/env bash
# Contratos de distribución: identidad, assets y superficie de integración.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
failures=0
fail() { printf 'CONTRACT ERROR: %s\n' "$1" >&2; failures=$((failures + 1)); }
ok() { printf '  OK    %s\n' "$1"; }

[[ -f "$ROOT_DIR/ltools.sh" ]] || fail 'falta ltools.sh'
[[ -x "$ROOT_DIR/scripts/host-tools.sh" ]] || fail 'host-tools.sh no es ejecutable'
[[ -f "$ROOT_DIR/scripts/lib/ltools-i18n.sh" ]] || fail 'falta el catálogo Bash'
[[ -f "$ROOT_DIR/appimage/ltools.desktop" ]] || fail 'falta el descriptor LTools'
[[ -f "$ROOT_DIR/appimage/ltools.svg" ]] || fail 'falta el icono LTools'
for module in disk-audit disk-clean game-wine-audit rollback system-control wine-prefix-manager; do
    grep -Fq 'lib/ltools-plan.sh' "$ROOT_DIR/scripts/$module.sh" || fail "$module no usa ltools-plan.sh"
done

legacy_product='cachy'
legacy_product+='os-tools'
legacy_alias='chary'
legacy_alias+='os-tools'
legacy_brand='cachy'
legacy_brand+='os tools'
legacy_env='CACHYOS'
legacy_env+='_TOOLS'
if rg -n -i "$legacy_product|$legacy_alias|$legacy_brand|$legacy_env" \
    --glob '!dist/**' --glob '!reports/**' --glob '!rust/target/**' \
    --glob '!tests/contracts.sh' "$ROOT_DIR" >/tmp/ltools-identity-failures.txt 2>/dev/null; then
    sed -n '1,80p' /tmp/ltools-identity-failures.txt >&2
    fail 'quedan referencias de la identidad antigua'
else
    ok 'identidad antigua ausente del código distribuible'
fi

grep -Fq 'name = "ltools"' "$ROOT_DIR/rust/Cargo.toml" || fail 'Cargo no usa el nombre ltools'
grep -Fq 'Name=LTools' "$ROOT_DIR/appimage/ltools.desktop" || fail 'desktop no usa LTools'
grep -Fq 'Exec=ltools menu' "$ROOT_DIR/appimage/ltools.desktop" || fail 'desktop no inicia el menú LTools'
grep -Fq 'es|en|de|fr|pt|it' "$ROOT_DIR/scripts/lib/ltools-i18n.sh" || fail 'catálogo Bash incompleto'
grep -Fq 'SUPPORTED: &[&str] = &["es", "en", "de", "fr", "pt", "it"]' \
    "$ROOT_DIR/rust/src/i18n.rs" || fail 'catálogo Rust incompleto'
ok 'contratos de identidad, assets e idiomas'

if ((failures)); then
    exit 1
fi
printf 'Contratos LTools correctos.\n'
