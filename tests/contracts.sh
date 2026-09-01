#!/usr/bin/env bash
# Contratos de distribución: identidad, assets y superficie de integración.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
failures=0
fail() { printf 'CONTRACT ERROR: %s\n' "$1" >&2; failures=$((failures + 1)); }
ok() { printf '  OK    %s\n' "$1"; }

[[ -f "$ROOT_DIR/ltools.sh" ]] || fail 'falta ltools.sh'
[[ -x "$ROOT_DIR/build.sh" ]] || fail 'falta el punto de entrada build.sh'
[[ -f "$ROOT_DIR/platform/linux/build.sh" ]] || fail 'falta el builder Linux'
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
[[ -f "$ROOT_DIR/distribution/ltools-project.json" ]] || fail 'falta el descriptor declarativo del proyecto'
[[ -f "$ROOT_DIR/distribution/ltools-project.schema.json" ]] || fail 'falta el esquema del descriptor de proyecto'
[[ -f "$ROOT_DIR/distribution/ltools-release.schema.json" ]] || fail 'falta el esquema del manifiesto de release'
[[ -x "$ROOT_DIR/appimage/AppRun-cli" ]] || fail 'falta el AppRun del perfil CLI'
[[ -f "$ROOT_DIR/appimage/ltools-cli.desktop" ]] || fail 'falta el descriptor del perfil CLI'
grep -Fq 'appstreamcli validate --no-net' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin validación explícita AppStream'
grep -Fq 'appimagetool --no-appstream' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin modo AppStream explícito'
[[ -f "$ROOT_DIR/README.md" ]] || fail 'falta el README del proyecto'
old_product='cachy'
old_product+='os-tools'
old_alias='chary'
old_alias+='os-tools'
old_brand='cachy'
old_brand+='os tools'
old_env='CACHYOS'
old_env+='_TOOLS'
if rg -n -i "$old_product|$old_alias|$old_brand|$old_env" \
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
grep -Fq 'host_tools' "$ROOT_DIR/rust/src/compat.rs" || fail 'contrato sin catálogo de herramientas del anfitrión'
grep -Fq 'rsync' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin migración verificada'
grep -Fq 'sc.exe' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin control de servicios'
grep -Fq 'install_package' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin metadatos de instalación'
grep -Fq 'install_package' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin metadatos de instalación'
grep -Fq 'doctor --install TOOL' "$ROOT_DIR/rust/src/main.rs" || fail 'doctor sin instalación explícita de una herramienta'
if rg -n -- '--install-missing|install_all' "$ROOT_DIR/rust/src" "$ROOT_DIR/README.md" >/tmp/ltools-broad-install.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-broad-install.txt >&2
    fail 'existe una ruta de instalación masiva'
else
    ok 'instalación únicamente explícita y contextual'
fi
if rg -n '"games"|"virtualization"|"development"|tool\("(steam|lutris|heroic|bottles|umu-run|wine|winetricks|docker|podman|virsh|qemu-system-x86_64|VBoxManage|vmware|git|cargo|node|python)"' \
    "$ROOT_DIR/rust/src/platform/linux.rs" "$ROOT_DIR/rust/src/platform/windows.rs" \
    >/tmp/ltools-host-catalog-forbidden.txt 2>/dev/null; then
    sed -n '1,80p' /tmp/ltools-host-catalog-forbidden.txt >&2
    fail 'el catálogo del anfitrión incluye juegos/Wine, virtualización o desarrollo'
else
    ok 'catálogo del anfitrión sin módulos ajenos al alcance'
fi
grep -Fq 'ltools-terminal-integration-v1' "$ROOT_DIR/rust/src/compat.rs" || fail 'backend sin descriptor JSON específico de terminal'
grep -Fq 'standalone_releases_require_it' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin independencia del host de terminal'
grep -Fq 'WinSlim Terminal' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin host Windows WinSlim Terminal'
grep -Fq 'standalone_releases_require_it' "$ROOT_DIR/rust/src/compat.rs" || fail 'contrato sin independencia de LTerminal'
grep -Fq 'capabilities --format json' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin descriptor JSON generado'
grep -Fq 'capabilities --format json' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin descriptor JSON generado'
grep -Fq 'capabilities --format terminal-json' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin descriptor JSON de terminal'
grep -Fq 'capabilities --format terminal-json' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin descriptor JSON de terminal'
grep -Fq 'release-manifest' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin manifiesto verificable de release'
grep -Fq 'release-manifest' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin manifiesto verificable de release'
grep -Fq 'sha256' "$ROOT_DIR/distribution/ltools-release.schema.json" || fail 'esquema de release sin SHA-256'
if command -v jq >/dev/null 2>&1; then
    jq -e '.schema == "ltools-project-v1" and .repository == "Darkeiser003/Tools" and .platforms.linux and .platforms.windows' \
        "$ROOT_DIR/distribution/ltools-project.json" >/dev/null \
        || fail 'descriptor declarativo del proyecto inválido'
    jq -e '.properties.schema.const == "ltools-release-v1" and .properties.hash_algorithm.const == "sha256"' \
        "$ROOT_DIR/distribution/ltools-release.schema.json" >/dev/null \
        || fail 'esquema del manifiesto de release inválido'
    jq -e '.properties.schema.const == "ltools-project-v1" and (.properties.platforms.required | index("linux")) != null and (.properties.platforms.required | index("windows")) != null' \
        "$ROOT_DIR/distribution/ltools-project.schema.json" >/dev/null \
        || fail 'esquema del descriptor de proyecto inválido'
fi
if rg -n 'source .*platform/linux/scripts|run_module' \
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
