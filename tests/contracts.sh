#!/usr/bin/env bash
# Contratos de distribución: identidad, assets y superficie de integración.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
failures=0
fail() { printf 'CONTRACT ERROR: %s\n' "$1" >&2; failures=$((failures + 1)); }
ok() { printf '  OK    %s\n' "$1"; }

[[ -f "$ROOT_DIR/ltools.sh" ]] || fail 'falta ltools.sh'
[[ -x "$ROOT_DIR/legacy/bash/host-tools.sh" ]] || fail 'host-tools legacy no es ejecutable'
[[ -f "$ROOT_DIR/legacy/bash/lib/ltools-i18n.sh" ]] || fail 'falta el catálogo Bash legacy'
[[ -f "$ROOT_DIR/appimage/ltools.desktop" ]] || fail 'falta el descriptor LTools'
[[ -f "$ROOT_DIR/appimage/ltools.svg" ]] || fail 'falta el icono LTools'
[[ -f "$ROOT_DIR/windows/build.ps1" ]] || fail 'falta el builder Windows'
[[ -f "$ROOT_DIR/windows/build.cmd" ]] || fail 'falta el lanzador build.cmd Windows'
[[ -f "$ROOT_DIR/windows/ltools.ps1" ]] || fail 'falta el lanzador PowerShell Windows'
[[ -f "$ROOT_DIR/windows/ltools.cmd" ]] || fail 'falta el lanzador CMD Windows'
[[ -f "$ROOT_DIR/windows/tests/smoke.ps1" ]] || fail 'falta el smoke Windows'
[[ -f "$ROOT_DIR/windows/tests/e2e.ps1" ]] || fail 'falta la E2E Windows'
[[ -f "$ROOT_DIR/appimage/org.ltools.LTools.metainfo.xml.in" ]] || fail 'falta el manifiesto AppStream'
[[ -f "$ROOT_DIR/appimage/ltools-capabilities.schema.json" ]] || fail 'falta el esquema JSON de capacidades'
[[ -f "$ROOT_DIR/appimage/ltools-terminal.schema.json" ]] || fail 'falta el esquema JSON de integración de terminal'
[[ -x "$ROOT_DIR/appimage/AppRun-cli" ]] || fail 'falta el AppRun del perfil CLI'
[[ -f "$ROOT_DIR/appimage/ltools-cli.desktop" ]] || fail 'falta el descriptor del perfil CLI'
grep -Fq 'appstreamcli validate --no-net' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin validación explícita AppStream'
grep -Fq 'appimagetool --no-appstream' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin modo AppStream explícito'
[[ -f "$ROOT_DIR/README.md" ]] || fail 'falta el README del proyecto'
for module in disk-audit disk-clean game-wine-audit rollback system-control wine-prefix-manager; do
    grep -Fq 'lib/ltools-plan.sh' "$ROOT_DIR/legacy/bash/$module.sh" || fail "$module no usa ltools-plan.sh"
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
grep -Fq 'Exec=ltools %U' "$ROOT_DIR/appimage/ltools.desktop" || fail 'desktop autónomo no inicia LTools'
grep -Fq 'Terminal=false' "$ROOT_DIR/appimage/ltools.desktop" || fail 'desktop autónomo delega indebidamente la terminal'
grep -Fq 'Terminal=true' "$ROOT_DIR/appimage/ltools-cli.desktop" || fail 'desktop CLI no conserva la terminal del sistema'
grep -Fq 'es|en|de|fr|pt|it|ca|nl|pl' "$ROOT_DIR/legacy/bash/lib/ltools-i18n.sh" || fail 'catálogo Bash incompleto'
grep -Fq 'SUPPORTED: &[&str] = &["es", "en", "de", "fr", "pt", "it", "ca", "nl", "pl"]' \
    "$ROOT_DIR/rust/src/i18n.rs" || fail 'catálogo Rust incompleto'
grep -Fq 'exec "$BIN"' "$ROOT_DIR/ltools.sh" || fail 'la fachada no ejecuta directamente el backend Rust'
grep -Fq 'exec "$BACKEND"' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun no ejecuta directamente el backend Rust'
grep -Fq 'LTOOLS_LTERMINAL' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun sin selección de LTerminal'
grep -Fq 'terminal_mode="${LTOOLS_TERMINAL:-auto}"' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun sin modo autónomo de terminal'
grep -Fq 'LTOOLS_TERMINAL=lterminal' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'smoke sin integración LTerminal explícita'
grep -Fq 'lterminal-startup-v1' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun sin comprobación de protocolo LTerminal'
grep -Fq 'launch_standard_terminal' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun sin ventana de terminal autónoma'
grep -Fq 'select_shell' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun sin selección de shell autónoma'
grep -Fq 'INTERACTIVE_TTY=0' "$ROOT_DIR/platform/linux/build.sh" || fail 'builder sin detección TTY previa al log'
grep -Fq 'ltools-capabilities-v1' "$ROOT_DIR/rust/src/compat.rs" || fail 'backend sin contrato JSON de capacidades'
grep -Fq 'ltools-terminal-integration-v1' "$ROOT_DIR/rust/src/compat.rs" || fail 'backend sin descriptor JSON específico de terminal'
grep -Fq 'standalone_releases_require_it' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin independencia del host de terminal'
grep -Fq 'WinSlim Terminal' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin host Windows WinSlim Terminal'
grep -Fq 'standalone_releases_require_it' "$ROOT_DIR/rust/src/compat.rs" || fail 'contrato sin independencia de LTerminal'
grep -Fq 'capabilities --format json' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin descriptor JSON generado'
grep -Fq 'capabilities --format json' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin descriptor JSON generado'
grep -Fq 'capabilities --format terminal-json' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin descriptor JSON de terminal'
grep -Fq 'capabilities --format terminal-json' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin descriptor JSON de terminal'
if rg -n 'source .*legacy/bash|source .*platform/linux/scripts|run_module' \
    "$ROOT_DIR/ltools.sh" "$ROOT_DIR/appimage/AppRun" >/tmp/ltools-shell-backend.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-shell-backend.txt >&2
    fail 'la entrada normal todavía depende de un backend Bash'
else
    ok 'backend normal exclusivamente Rust'
fi
grep -Fq 'x86_64-pc-windows-msvc' "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows no usa target MSVC'
grep -Fq 'build-state.json' "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows sin estado incremental'
grep -Fq 'CARGO_TARGET_DIR' "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows no fija el directorio de target'
grep -Fq 'windows\tests\e2e.ps1' "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows no ejecuta la E2E nativa'
grep -Fq 'PackageArch' "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows no adapta el nombre a la arquitectura'
grep -Fq 'run_with_sudo(program' "$ROOT_DIR/rust/src/packages.rs" || fail 'limpieza no respeta el gestor seleccionado'
grep -Fq 'LC_ALL", "C' "$ROOT_DIR/rust/src/packages.rs" || fail 'consultas de paquetes no fijan locale estable'
if rg -n '(^|[[:space:]])wine([[:space:]]|$).*ltools|wine\.exe.*ltools|wine[[:space:]]+"' \
    "$ROOT_DIR/platform/linux/build.sh" "$ROOT_DIR/tests" "$ROOT_DIR/windows" "$ROOT_DIR/appimage" >/tmp/ltools-wine-tests.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-wine-tests.txt >&2
    fail 'el pipeline de build/tests intenta ejecutar LTools mediante Wine'
else
    ok 'pipeline sin ejecución Wine'
fi
ok 'contratos de identidad, assets e idiomas'

if ((failures)); then
    exit 1
fi
printf 'Contratos LTools correctos.\n'
