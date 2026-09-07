#!/usr/bin/env bash
# Contratos de distribución: identidad, assets y superficie de integración.

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
failures=0
fail() { printf 'CONTRACT ERROR: %s\n' "$1" >&2; failures=$((failures + 1)); }
ok() { printf '  OK    %s\n' "$1"; }

[[ -f "$ROOT_DIR/ltools.sh" ]] || fail 'falta ltools.sh'
[[ -f "$ROOT_DIR/ltools-cli.sh" ]] || fail 'falta el lanzador CLI Linux'
if [[ ! -x "$ROOT_DIR/ltools" && ! -x "$ROOT_DIR/ltools.sh" ]]; then
    fail 'falta el entrypoint portable ltools o su lanzador Linux'
fi
if [[ ! -x "$ROOT_DIR/ltools-cli" && ! -x "$ROOT_DIR/ltools-cli.sh" ]]; then
    fail 'falta el entrypoint portable ltools-cli o su lanzador Linux'
fi
[[ -x "$ROOT_DIR/build.sh" ]] || fail 'falta el punto de entrada build.sh'
[[ -f "$ROOT_DIR/platform/linux/build.sh" ]] || fail 'falta el builder Linux'
[[ -f "$ROOT_DIR/appimage/ltools.desktop" ]] || fail 'falta el descriptor LTools'
[[ -f "$ROOT_DIR/appimage/ltools.svg" ]] || fail 'falta el icono LTools'
[[ -f "$ROOT_DIR/windows/build.ps1" ]] || fail 'falta el builder Windows'
[[ -f "$ROOT_DIR/windows/build.cmd" ]] || fail 'falta el lanzador build.cmd Windows'
[[ -f "$ROOT_DIR/windows/ltools.ps1" ]] || fail 'falta el lanzador PowerShell Windows'
[[ -f "$ROOT_DIR/windows/ltools.cmd" ]] || fail 'falta el lanzador CMD Windows'
[[ -f "$ROOT_DIR/windows/ltools-cli.ps1" ]] || fail 'falta el lanzador CLI PowerShell Windows'
[[ -f "$ROOT_DIR/windows/ltools-cli.cmd" ]] || fail 'falta el lanzador CLI CMD Windows'
[[ -f "$ROOT_DIR/windows/tests/smoke.ps1" ]] || fail 'falta el smoke Windows'
[[ -f "$ROOT_DIR/windows/tests/e2e.ps1" ]] || fail 'falta la E2E Windows'
[[ -f "$ROOT_DIR/windows/tests/native-process.ps1" ]] || fail 'falta el capturador nativo común de Windows'
[[ -x "$ROOT_DIR/tests/encoding.sh" ]] || fail 'falta la auditoría de codificaciones'
[[ -x "$ROOT_DIR/tests/release-e2e.sh" ]] || fail 'falta la E2E de publicación release'
[[ -f "$ROOT_DIR/rust/src/storage/linux.rs" ]] || fail 'falta almacenamiento Linux separado'
[[ -f "$ROOT_DIR/rust/src/storage/windows.rs" ]] || fail 'falta almacenamiento Windows separado'
[[ -f "$ROOT_DIR/rust/src/registry/linux.rs" ]] || fail 'falta registro Linux separado'
[[ -f "$ROOT_DIR/rust/src/registry/windows.rs" ]] || fail 'falta registro Windows separado'
[[ -f "$ROOT_DIR/rust/src/diagnostics/linux.rs" ]] || fail 'falta diagnóstico Linux separado'
[[ -f "$ROOT_DIR/rust/src/diagnostics/windows.rs" ]] || fail 'falta diagnóstico Windows separado'
[[ -x "$ROOT_DIR/tests/linux/windows-wine.sh" ]] || fail 'falta el smoke Windows bajo Wine/Proton'
[[ -x "$ROOT_DIR/tests/linux/software-git-e2e.sh" ]] || fail 'falta la E2E de stores y Git'
[[ -f "$ROOT_DIR/appimage/org.ltools.LTools.metainfo.xml.in" ]] || fail 'falta el manifiesto AppStream'
[[ -f "$ROOT_DIR/appimage/ltools-capabilities.schema.json" ]] || fail 'falta el esquema JSON de capacidades'
[[ -f "$ROOT_DIR/appimage/ltools-terminal.schema.json" ]] || fail 'falta el esquema JSON de integración de terminal'
[[ -f "$ROOT_DIR/distribution/ltools-project.json" ]] || fail 'falta el descriptor declarativo del proyecto'
[[ -f "$ROOT_DIR/distribution/ltools-project.schema.json" ]] || fail 'falta el esquema del descriptor de proyecto'
[[ -f "$ROOT_DIR/distribution/ltools-release.schema.json" ]] || fail 'falta el esquema del manifiesto de release'
[[ -f "$ROOT_DIR/docs/lterminal-integration.md" ]] || fail 'falta el contrato documentado de integración con LTerminal'
[[ -x "$ROOT_DIR/appimage/AppRun-cli" ]] || fail 'falta el AppRun del perfil CLI'
[[ -f "$ROOT_DIR/appimage/ltools-cli.desktop" ]] || fail 'falta el descriptor del perfil CLI'
grep -Fq 'appstreamcli validate --no-net' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin validación explícita AppStream'
grep -Fq 'appimagetool --no-appstream' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin modo AppStream explícito'
[[ -f "$ROOT_DIR/README.md" ]] || fail 'falta el README del proyecto'
[[ -x "$ROOT_DIR/clean-repository.sh" ]] || fail 'falta el limpiador seguro del checkout'
grep -Fq -- '--dry-run' "$ROOT_DIR/clean-repository.sh" || fail 'limpiador sin modo simulación predeterminado'
grep -Fq -- '--plans-only' "$ROOT_DIR/clean-repository.sh" || fail 'limpiador sin modo aislado para planes legacy'
if grep -Fq -- 'git clean' "$ROOT_DIR/clean-repository.sh"; then
    fail 'limpiador no debe delegar en git clean'
fi
grep -Fq -- 'No se borran fuentes' "$ROOT_DIR/clean-repository.sh" || fail 'limpiador sin protección de fuentes'
grep -Fq 'gtk_widget_set_size_request' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin tamaño mínimo de controles'
grep -Fq 'let controls_scroll = gtk_scrolled_window_new' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin scroll independiente de controles'
if ! grep -Fq 'gtk_paned_pack1(split, controls, 1, 1)' "$ROOT_DIR/rust/src/gui.rs" \
    && ! grep -Fq 'gtk_box_pack_start(root, controls_scroll, 1, 1, 0)' "$ROOT_DIR/rust/src/gui.rs"; then
    fail 'GUI sin área desplazable visible'
fi
grep -Fq 'show_page(&*navigation, None);' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI no restaura la página inicial tras mostrar la ventana'
grep -Fq 'gui-preferences-v1' "$ROOT_DIR/rust/src/gui_preferences.rs" || fail 'GUI sin preferencias persistentes versionadas'
grep -Fq 'LTOOLS_GUI_THEME' "$ROOT_DIR/rust/src/gui_preferences.rs" || fail 'preferencias GUI sin precedencia de tema'
grep -Fq 'settings_button' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin acceso visible a ajustes'
if grep -Fq '"文"' "$ROOT_DIR/rust/src/gui.rs"; then
    fail 'GUI conserva un acceso de idioma duplicado junto a Ajustes'
fi
grep -Fq 'crate::i18n::gui_text("settings_button")' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin botón visible de Ajustes'
grep -Fq 'on_settings_toggle' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin alternancia para cerrar Ajustes'
grep -Fq 'add_back_button_at(settings_page, navigation, 0)' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin salida visible al entrar en Ajustes'
grep -Fq 'gtk_widget_set_halign(preference_bar, 2)' "$ROOT_DIR/rust/src/gui.rs" || fail 'barra de preferencias GTK no alineada a la derecha'
grep -Fq 'settings_visibility' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin controles de visibilidad de categorías'
grep -Fq 'SETTINGS_APPLY_ID' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Windows sin aplicación explícita de ajustes'
grep -Fq 'gtk_grid_attach((*navigation).pages[6], search' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin submenú de búsqueda de paquetes'
grep -Fq 'on_install_package' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI de paquetes sin flujo de instalación guiada'
grep -Fq 'PACKAGE_INSTALL_FIELDS' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI de paquetes sin selección de candidato'
grep -Fq 'fs::rename(&temporary, &path)' "$ROOT_DIR/rust/src/automation.rs" || fail 'registro de automatizaciones sin guardado atómico'
grep -Fq 'parse_limit' "$ROOT_DIR/rust/src/software.rs" || fail 'búsqueda de paquetes sin límite de resultados'
grep -Fq 'stable_plan_name' "$ROOT_DIR/rust/src/common.rs" || fail 'planes sin nombres estables por módulo'
grep -Fq 'fn finalize' "$ROOT_DIR/rust/src/common.rs" || fail 'planes automáticos sin eliminación de archivos vacíos'
grep -Fq 'reusable_explicit_plan' "$ROOT_DIR/rust/src/common.rs" || fail 'planes explícitos sin reutilización segura'
grep -Fq 'requested_plan_path.clone()' "$ROOT_DIR/rust/src/main.rs" || fail 'menú sin reutilización de la ruta de plan explícita'
grep -Fq '"native" | "native-tools"' "$ROOT_DIR/rust/src/main.rs" || fail 'alias native-tools fuera de la política de planes'
grep -Fq '"clean" | "cleanup"' "$ROOT_DIR/rust/src/main.rs" || fail 'alias cleanup fuera de la política de planes'
grep -Fq 'default_report_dir' "$ROOT_DIR/rust/src/audit.rs" || fail 'auditoría sin directorio de informe estable'
grep -Fq 'default_report_dir' "$ROOT_DIR/rust/src/games.rs" || fail 'juegos sin directorio de informe estable'
grep -Fq 'default_report_dir' "$ROOT_DIR/rust/src/packages.rs" || fail 'paquetes sin directorio de informe estable'
grep -Fq 'reset_default_report_dir' "$ROOT_DIR/rust/src/common.rs" || fail 'informes predeterminados sin limpieza controlada'
if rg -n 'format!\("(rust-(audit|games|package-audit)-|ltools-windows-games-)' \
    "$ROOT_DIR/rust/src" >/tmp/ltools-timestamped-reports.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-timestamped-reports.txt >&2
    fail 'quedan informes predeterminados con nombre fechado o en el checkout'
else
    ok 'informes predeterminados sin directorios fechados en el proyecto'
fi
if grep -Fq 'SystemTime' "$ROOT_DIR/rust/src/common.rs"; then
    fail 'common.rs sigue generando planes con reloj/PID'
else
    ok 'planes sin nombres fechados por ejecución'
fi
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
grep -Fq '"ar", "de", "en", "es", "fr", "hi", "it", "ja", "ko", "pl", "pt", "ro", "ru", "uk", "zh"' "$ROOT_DIR/rust/src/i18n.rs" \
    || fail 'catálogo Rust sin los locales adicionales de LTerminal'
grep -Fq '"uk", "zh"' "$ROOT_DIR/rust/src/i18n.rs" \
    || fail 'catálogo Rust sin los últimos locales de LTerminal'
grep -Fq '"silver", "winslim", "ocean", "forest"' "$ROOT_DIR/rust/src/theme.rs" \
    || fail 'catálogo Rust sin el tema compatible winslim'
grep -Fq 'exec "$BIN"' "$ROOT_DIR/ltools.sh" || fail 'la fachada no ejecuta directamente el backend Rust'
grep -Fq 'LTOOLS_CLI' "$ROOT_DIR/ltools.sh" || fail 'la fachada no conserva el perfil CLI explícito'
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
grep -Fq 'dependency_confirmation' "$ROOT_DIR/rust/src/common.rs" || fail 'instalación sin confirmación detallada de dependencia'
grep -Fq 'Paquete que se instalará' "$ROOT_DIR/rust/src/common.rs" || fail 'confirmación sin paquete de dependencia'
grep -Fq 'tests/encoding.sh' "$ROOT_DIR/platform/linux/build.sh" || fail 'build sin auditoría de codificaciones'
grep -Fq 'tests/release-e2e.sh' "$ROOT_DIR/platform/linux/build.sh" || fail 'build sin E2E de publicación release'
grep -Fq 'release_e2e_args+=(--no-package)' "$ROOT_DIR/platform/linux/build.sh" || fail 'build sin propagar --no-package a la E2E release'
grep -Fq 'release_e2e_args+=(--no-appimage)' "$ROOT_DIR/platform/linux/build.sh" || fail 'build sin propagar --no-appimage a la E2E release'
grep -Fq 'docker-compose-primary-installer' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin instalador primario de Compose'
grep -Fq 'kubernetes-primary-client-installer' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin instalador primario de Kubernetes'
grep -Fq 'Docker.DockerCompose' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin instalador primario de Compose'
grep -Fq 'Kubernetes.kubectl' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin instalador primario de Kubernetes'
grep -Fq '"lsblk"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin inventario de discos'
grep -Fq '"parted"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin alternativa de particionado'
grep -Fq '"podman"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin Podman'
grep -Fq '"cri-tools"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin instalador de crictl'
for tool_id in wipefs sgdisk testdisk ddrescue restic borg rclone ethtool iperf3 inxi sensors jq yq git-lfs buildah skopeo kustomize trivy cosign; do
    grep -Fq "\"$tool_id\"" "$ROOT_DIR/rust/src/platform/linux.rs" \
        || fail "catálogo Linux sin herramienta ampliada: $tool_id"
done
for tool_id in chkdsk.exe fsutil.exe manage-bde.exe sfc.exe pnputil.exe schtasks.exe netsh.exe icacls.exe vssadmin.exe wbadmin.exe dotnet.exe; do
    grep -Fq "\"$tool_id\"" "$ROOT_DIR/rust/src/platform/windows.rs" \
        || fail "catálogo Windows sin herramienta ampliada: $tool_id"
done
grep -Fq '"install-dependency"' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'CLI Linux sin acción de instalación de dependencias nativas'
grep -Fq '"tools_install"' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin botón de instalación de dependencias nativas'
grep -Fq 'container-prune' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'Linux sin limpieza guiada de contenedores'
grep -Fq 'image-build' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'Linux sin construcción guiada de imágenes'
grep -Fq 'volume-prune' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'Linux sin limpieza guiada de volúmenes'
grep -Fq 'network-prune' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'Linux sin limpieza guiada de redes'
grep -Fq 'container-compose' "$ROOT_DIR/rust/src/native/windows.rs" || fail 'Windows sin flujo guiado de Compose'
grep -Fq 'native-container-compose' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor de terminal sin acción Compose'
grep -Fq '"diskpart"' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin DiskPart'
grep -Fq '"winget"' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin winget'
if rg -n 'tool\("(winget|choco|scoop|diskpart|sc\.exe)"' "$ROOT_DIR/rust/src/platform/linux.rs" >/tmp/ltools-linux-cross-tools.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-linux-cross-tools.txt >&2
    fail 'el catálogo Linux incluye herramientas nativas Windows'
else
    ok 'catálogo Linux sin gestores ni comandos Windows'
fi
if rg -n 'tool\("(pacman|apt-get|apt|parted|gparted|systemctl|wine|wineboot)"' "$ROOT_DIR/rust/src/platform/windows.rs" >/tmp/ltools-windows-cross-tools.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-windows-cross-tools.txt >&2
    fail 'el catálogo Windows incluye gestores o comandos Linux'
else
    ok 'catálogo Windows sin gestores ni comandos Linux'
fi
for catalog in "$ROOT_DIR/rust/src/platform/linux.rs" "$ROOT_DIR/rust/src/platform/windows.rs"; do
    duplicate_ids="$({
        perl -0777 -ne 'while (/tool\(\s*"([^"]+)"/g) { print "$1\n" }' "$catalog" || true
    } | sort | uniq -d)"
    if [[ -n "$duplicate_ids" ]]; then
        printf '%s\n' "$duplicate_ids" >&2
        fail "catálogo con herramientas duplicadas: $catalog"
    fi
done
ok 'catálogos de herramientas sin identificadores duplicados'
grep -Fq 'doctor --install TOOL' "$ROOT_DIR/rust/src/main.rs" || fail 'doctor sin instalación explícita de una herramienta'
if rg -n -- '--install-missing|install_all' "$ROOT_DIR/rust/src" "$ROOT_DIR/README.md" >/tmp/ltools-broad-install.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-broad-install.txt >&2
    fail 'existe una ruta de instalación masiva'
else
    ok 'instalación únicamente explícita y contextual'
fi
if rg -n '"games"|"virtualization"|tool\("(steam|lutris|heroic|bottles|umu-run|wine|winetricks|virsh|qemu-system-x86_64|VBoxManage|vmware)"' \
    "$ROOT_DIR/rust/src/platform/linux.rs" "$ROOT_DIR/rust/src/platform/windows.rs" \
    >/tmp/ltools-host-catalog-forbidden.txt 2>/dev/null; then
    sed -n '1,80p' /tmp/ltools-host-catalog-forbidden.txt >&2
    fail 'el catálogo del anfitrión incluye juegos/Wine o virtualización'
else
    ok 'catálogo del anfitrión sin juegos/Wine ni virtualización'
fi
grep -Fq 'ltools-terminal-integration-v1' "$ROOT_DIR/rust/src/compat.rs" || fail 'backend sin descriptor JSON específico de terminal'
grep -Fq 'ltools-terminal-windows.json' "$ROOT_DIR/distribution/ltools-project.json" || fail 'catálogo de proyecto sin descriptor Windows explícito'
grep -Fq 'package-search' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin acción de búsqueda de paquetes'
grep -Fq 'native-tools-install' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin acción de instalación de dependencias nativas'
grep -Fq 'git-pull' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin acción Git'
grep -Fq '"Git-version-control"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin instalación contextual de Git'
grep -Fq '"GitHub-CLI"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin instalación contextual de GitHub CLI'
grep -Fq 'fn require_git(ctx: &Context)' "$ROOT_DIR/rust/src/git.rs" || fail 'Git sin contexto de dependencias y dry-run'
grep -Fq 'fn require_gh(ctx: &Context)' "$ROOT_DIR/rust/src/git.rs" || fail 'GitHub CLI sin contexto de dependencias y dry-run'
grep -Fq 'git-commit' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin acción Git commit'
grep -Fq 'git-push' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin acción Git push'
grep -Fq 'gh-release' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin acción GitHub release'
grep -Fq 'on_git_action' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin callback de acciones Git'
grep -Fq 'git_fields' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Git sin campos de entrada'
grep -Fq 'gh-auth-status' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin consulta de autenticación GitHub'
grep -Fq 'SERVICES_BACK_ROW' "$ROOT_DIR/rust/src/gui.rs" || fail 'submenú de servicios sin fila de retorno separada'
grep -Fq 'services layout: back-row=' "$ROOT_DIR/rust/src/gui.rs" || fail 'smoke GUI sin validación de layout de Servicios'
grep -Fq 'ltools-section-heading' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin encabezados modulares de sección'
grep -Fq 'ltools-content' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin área de contenido modular'
grep -Fq 'ltools-topbar' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin barra superior persistente'
grep -Fq 'ltools-context' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin contexto de sección'
grep -Fq 'dashboard_hint' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin dashboard inicial'
grep -Fq 'mod software;' "$ROOT_DIR/rust/src/main.rs" || fail 'backend sin módulo Rust de software'
grep -Fq 'mod git;' "$ROOT_DIR/rust/src/main.rs" || fail 'backend sin módulo Rust de Git'
grep -Fq 'mod diagnostics;' "$ROOT_DIR/rust/src/main.rs" || fail 'backend sin módulo Rust de diagnóstico nativo'
grep -Fq 'native-diagnostics' "$ROOT_DIR/rust/src/compat.rs" || fail 'contrato sin diagnóstico nativo'
grep -Fq 'ltools-diagnostics-v1' "$ROOT_DIR/rust/src/diagnostics/mod.rs" || fail 'diagnóstico sin esquema JSON propio'
grep -Fq 'standalone_releases_require_it' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin independencia del host de terminal'
grep -Fq 'WinSlim Terminal' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin host Windows WinSlim Terminal'
grep -Fq 'requiresCommands' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin requisitos declarativos de acciones'
grep -Fq 'workingDirectory' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin directorio de trabajo declarativo'
grep -Fq 'supports' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin capacidades de acción'
[[ -f "$ROOT_DIR/rust/src/automation.rs" ]] || fail 'falta el registro Rust de automatizaciones'
grep -Fq 'menu-audit-inventory' "$ROOT_DIR/rust/src/main.rs" || fail 'menú sin categoría de auditoría/inventario'
grep -Fq 'menu-import' "$ROOT_DIR/rust/src/main.rs" || fail 'menú sin categoría Importar'
grep -Fq 'menu-settings' "$ROOT_DIR/rust/src/main.rs" || fail 'menú sin categoría Ajustes'
grep -Fq 'mod cli_ui;' "$ROOT_DIR/rust/src/main.rs" || fail 'CLI sin capa visual compartida'
grep -Fq 'cli_ui::header' "$ROOT_DIR/rust/src/main.rs" || fail 'CLI sin cabecera/contexto persistente'
grep -Fq 'winslim_available' "$ROOT_DIR/rust/src/main.rs" || fail 'menú Windows sin detección condicional de WinSlim'
grep -Fq 'standalone_releases_require_it' "$ROOT_DIR/rust/src/compat.rs" || fail 'contrato sin independencia de LTerminal'
grep -Fq 'capabilities --format json' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin descriptor JSON generado'
grep -Fq 'capabilities --format json' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin descriptor JSON generado'
grep -Fq 'capabilities --format terminal-json' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin descriptor JSON de terminal'
grep -Fq 'capabilities --format terminal-json' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin descriptor JSON de terminal'
grep -Fq 'release-manifest' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin manifiesto verificable de release'
grep -Fq 'release-checksums' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin SHA256SUMS reproducible'
grep -Fq 'release-signature' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin firma Ed25519'
grep -Fq 'SIGNING_REQUIRED=1' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux no exige firma por defecto'
grep -Fq '$SigningRequired = $true' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows no exige firma por defecto'
grep -Fq 'copy_launcher()' "$ROOT_DIR/platform/linux/build.sh" || fail 'paquete Linux depende de lanzadores no versionados'
grep -Fq 'fallback="${name}.sh"' "$ROOT_DIR/platform/linux/build.sh" || fail 'paquete Linux sin fallback para checkout limpio'
grep -Fq -- "-name 'ltools-*.tar.gz'" "$ROOT_DIR/platform/linux/build.sh" || fail 'release no limpia tarballs antiguos'
grep -Fq -- '--release-dir' "$ROOT_DIR/platform/linux/build.sh" || fail 'build Linux sin carpeta de publicación configurable'
grep -Fq -- '--windows-wine' "$ROOT_DIR/platform/linux/build.sh" || fail 'build raíz sin opción Windows bajo Wine/Proton'
grep -Fq 'windows-wine.sh' "$ROOT_DIR/platform/linux/build.sh" || fail 'pipeline sin etapa Windows bajo Wine/Proton'
grep -Fq -- '--windows-wine-runner' "$ROOT_DIR/platform/linux/build.sh" || fail 'pipeline sin runner Windows configurable'
grep -Fq -- '--windows-wine-prefix' "$ROOT_DIR/platform/linux/build.sh" || fail 'pipeline sin prefijo Windows configurable'
grep -Fq 'WINDOWS_LAUNCHERS' "$ROOT_DIR/rust/src/games.rs" || fail 'inventario Windows sin catálogo nativo de lanzadores'
grep -Fq 'games-windows-native' "$ROOT_DIR/rust/src/games.rs" || fail 'inventario Windows sin modo nativo explícito'
grep -Fq 'No se buscan prefijos Wine' "$ROOT_DIR/rust/src/games.rs" || fail 'inventario Windows no documenta la exclusión de Wine'
grep -Fq 'LTOOLS_CLI' "$ROOT_DIR/rust/src/main.rs" || fail 'backend sin perfil CLI explícito'
grep -Fq 'LTOOLS_CLI' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun no conserva el perfil CLI sin argumentos'
grep -Fq 'storage' "$ROOT_DIR/rust/src/main.rs" || fail 'backend sin módulo de almacenamiento'
grep -Fq 'registry' "$ROOT_DIR/rust/src/main.rs" || fail 'backend sin módulo de registro/configuración'
grep -Fq 'native-tools-menu' "$ROOT_DIR/rust/src/compat.rs" || fail 'descriptor sin flujo operativo de herramientas nativas'
grep -Fq 'fn tools_direct' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'Linux sin acciones nativas no interactivas'
grep -Fq 'fn tools_direct' "$ROOT_DIR/rust/src/native/windows.rs" || fail 'Windows sin acciones nativas no interactivas'
grep -Fq 'NativeActionData' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin modal de parámetros para herramientas nativas'
grep -Fq 'gtk_dialog_get_content_area' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin formulario modal de herramientas nativas'
grep -Fq '"adb"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin adb'
grep -Fq '"ssh"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin ssh'
grep -Fq '"scp"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin scp'
grep -Fq '"sftp"' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin sftp'
grep -Fq '"adb"' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin adb'
grep -Fq 'partition_guide' "$ROOT_DIR/rust/src/storage/linux.rs" || fail 'almacenamiento Linux sin guía segura'
grep -Fq '+SI:localuser:root' "$ROOT_DIR/rust/src/storage/linux.rs" || fail 'GParted sin autorización X11 acotada para root'
grep -Fq -- '-SI:localuser:root' "$ROOT_DIR/rust/src/storage/linux.rs" || fail 'GParted sin revocación del acceso X11 temporal'
grep -Fq 'Stdio::null()' "$ROOT_DIR/rust/src/storage/linux.rs" || fail 'gestores gráficos Linux heredan pipes del modal'
grep -Fq 'diskpart_guide' "$ROOT_DIR/rust/src/storage/windows.rs" || fail 'almacenamiento Windows sin guía DiskPart'
grep -Fq 'native_tools_label' "$ROOT_DIR/rust/src/i18n.rs" || fail 'GUI sin etiqueta localizada de herramientas nativas'
grep -Fq 'action_buttons: [[HWND; 11]' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Win32 sin capacidad para acciones ampliadas'
grep -Fq 'const ACTION_STRIDE: i32 = 16' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Win32 sin stride seguro para IDs de acciones'
grep -Fq 'for index in 0..4' "$ROOT_DIR/rust/src/gui.rs" || fail 'layout Win32 recorre campos fuera de rango'
grep -Fq 'gtk_alignment_new' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin contenedor central adaptable'
grep -Fq 'gtk_text_view_set_wrap_mode(output, 0)' "$ROOT_DIR/rust/src/gui.rs" || fail 'salida GUI sigue partiendo líneas largas'
grep -Fq 'gtk_scrolled_window_set_policy(scrolled, 1, 1)' "$ROOT_DIR/rust/src/gui.rs" || fail 'salida GUI sin scroll vertical y horizontal automático'
grep -Fq 'has_horizontal_overflow' "$ROOT_DIR/rust/src/gui.rs" || fail 'salida GUI sin detección de líneas largas'
grep -Fq 'horizontal_scroll_hint' "$ROOT_DIR/rust/src/i18n.rs" || fail 'aviso de scroll horizontal sin traducciones'
grep -Fq 'gtk_widget_set_halign' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin alineación responsiva de controles'
grep -Fq 'gtk_scrolled_window_set_policy(controls_scroll, 1, 1)' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin scroll horizontal y vertical automáticos para ventanas estrechas'
grep -Fq 'gtk_scrolled_window_set_policy(scrolled, 1, 1)' "$ROOT_DIR/rust/src/gui.rs" || fail 'salida GTK sin scroll horizontal automático'
grep -Fq 'gtk_text_view_set_wrap_mode(output, 0)' "$ROOT_DIR/rust/src/gui.rs" || fail 'salida GTK sin líneas intactas para lectura horizontal'
grep -Fq 'gtk_grid_attach(grid, button, 0, row, 2, 1)' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin filas de acciones responsivas'
grep -Fq 'ltools-section-heading' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin grupos visuales para Git y acciones largas'
grep -Fq 'LTOOLS_GUI_WIDTH' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin tamaño de prueba configurable'
grep -Fq 'std::thread::spawn(move ||' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK ejecuta acciones en el hilo visual'
grep -Fq 'g_idle_add' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin retorno seguro de resultados al hilo visual'
grep -Fq 'gtk_spinner_start' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin indicador de acción en curso'
grep -Fq 'gtk_progress_bar_pulse' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin progreso visual pulsante'
if [[ "$(rg -c '\.env\("LTOOLS_FRONTEND", "gui"\)' "$ROOT_DIR/rust/src/gui.rs" 2>/dev/null || true)" -lt 3 ]]; then
    fail 'acciones GUI sin contexto gráfico en todos sus lanzadores'
fi
grep -Fq 'gtk_widget_set_sensitive(controls' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK no bloquea acciones concurrentes'
grep -Fq 'busy-begin' "$ROOT_DIR/rust/src/gui.rs" || fail 'smoke GUI sin evidencia del estado ocupado'
grep -Fq 'GUI_MODAL_CANCEL' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin control de cancelación modal'
grep -Fq 'cancel-button-trigger' "$ROOT_DIR/rust/src/gui.rs" || fail 'smoke GUI sin ruta de cancelación modal'
grep -Fq 'cancel-requested' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin registro de cancelación solicitada'
grep -Fq 'LTOOLS_GUI_SMOKE_ACTION_CANCEL' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'smoke Linux sin prueba de Cancelar durante una acción'
grep -Fq 'command.starts_with("menu-")' "$ROOT_DIR/rust/src/main.rs" || fail 'menús mutables sin frontera de plan'
grep -Fq '"tools" | "quick-actions"' "$ROOT_DIR/rust/src/main.rs" || fail 'menú de herramientas sin política de plan'
grep -Fq 'stdout_reader = child.stdout.take()' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'captura nativa Linux susceptible a bloqueo por pipes llenos'
grep -Fq 'fn record(' "$ROOT_DIR/rust/src/accounts/linux.rs" || fail 'cuentas Linux sin registro transaccional'
grep -Fq 'fn record(' "$ROOT_DIR/rust/src/accounts/windows.rs" || fail 'cuentas Windows sin registro transaccional'
grep -Fq 'findmnt' "$ROOT_DIR/rust/src/storage/linux.rs" || fail 'desmontaje Linux sin resolver dispositivo a punto de montaje'
grep -Fq 'fn requires_privilege' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'acciones nativas Linux sin política de elevación'
grep -Fq '"firewall-disable"' "$ROOT_DIR/rust/src/native/linux.rs" || fail 'desactivación UFW sin operación transaccional propia'
grep -Fq '"networkmanager-connection"' "$ROOT_DIR/rust/src/main.rs" || fail 'red NetworkManager mutable sin política de plan'
grep -Fq '"ssh-keygen"' "$ROOT_DIR/rust/src/main.rs" || fail 'generación de claves SSH sin política de plan'
grep -Fq '"container-exec"' "$ROOT_DIR/rust/src/main.rs" || fail 'ejecución en contenedor sin política de plan'
grep -Fq 'button:focus' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin estado de foco visible'
grep -Fq 'button:disabled' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin estado deshabilitado visible'
grep -Fq 'progressbar trough' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin canal de progreso estilizado'
grep -Fq 'scrollbar slider:hover' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI GTK sin scrollbar visible al pasar el cursor'
grep -Fq 'fn layout_window' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Windows sin layout recalculable'
grep -Fq 'WM_SIZE' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Windows sin respuesta al redimensionado'
if rg -n 'Heroic-json|tool\("(wine|lutris|heroic|umu-run)"' "$ROOT_DIR/rust/src/platform/windows.rs" >/tmp/ltools-windows-cross-catalog.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-windows-cross-catalog.txt >&2
    fail 'el catálogo Windows conserva herramientas de Wine/Lutris/Heroic/UMU'
else
    ok 'catálogo Windows sin herramientas Wine/Lutris/Heroic/UMU'
fi
grep -Fq 'release-manifest' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin manifiesto verificable de release'
grep -Fq 'release-checksums' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin SHA256SUMS reproducible'
grep -Fq 'release-signature' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin firma Ed25519'
grep -Fq 'ReleaseOutput' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin carpeta de publicación configurable'
grep -Fq 'Carpeta release Windows preparada' "$ROOT_DIR/windows/build.ps1" || fail 'build Windows sin publicación en release'
grep -Fq 'sha256' "$ROOT_DIR/distribution/ltools-release.schema.json" || fail 'esquema de release sin SHA-256'
grep -Fq 'ed25519' "$ROOT_DIR/distribution/ltools-project.json" || fail 'descriptor de proyecto sin firma Ed25519'
grep -Fq 'SHA256SUMS.txt.sig' "$ROOT_DIR/distribution/ltools-project.schema.json" || fail 'esquema de proyecto sin firma separada'
if command -v jq >/dev/null 2>&1; then
    jq -e '.schema == "ltools-project-v1" and .repository == "Darkeiser003/Tools" and .platforms.linux and .platforms.windows' \
        "$ROOT_DIR/distribution/ltools-project.json" >/dev/null \
        || fail 'descriptor declarativo del proyecto inválido'
    jq -e '.verification.signature_supported == true and
        .verification.signature.algorithm == "ed25519" and
        .verification.signature.manifest == "SHA256SUMS.txt" and
        .verification.signature.detached_signature == "SHA256SUMS.txt.sig"' \
        "$ROOT_DIR/distribution/ltools-project.json" >/dev/null \
        || fail 'descriptor declarativo sin contrato de firma Ed25519'
    jq -e '.integration.hosts == ["LTerminal", "WinSlim Terminal"] and
        .distribution.source == "github-releases" and
        .distribution.update_strategy == "stage-verify-promote" and
        .platforms.linux.install.entrypoint == "ltools" and
        .platforms.windows.install.entrypoint == "ltools.exe"' \
        "$ROOT_DIR/distribution/ltools-project.json" >/dev/null \
        || fail 'descriptor declarativo sin flujo común de instalación/actualización'
    jq -e '((.required | index("distribution")) != null) and
        .properties.distribution and
        .properties.integration.properties.hosts and
        ."$defs".platform.properties.install.properties.entrypoint' \
        "$ROOT_DIR/distribution/ltools-project.schema.json" >/dev/null \
        || fail 'esquema de proyecto sin contrato de distribución/instalación'
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
grep -Fq 'LTOOLS_WINDOWS_CARGO_TARGET_DIR' "$ROOT_DIR/platform/linux/build.sh" || fail 'builder Linux no aísla el target Windows bajo Wine'
grep -Fq 'WINDOWS_CARGO_TARGET_DIR="${LTOOLS_WINDOWS_CARGO_TARGET_DIR:-' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'builder Wine no tiene target Windows aislado'
grep -Fq 'WINEXE="$CARGO_TARGET_DIR/$TARGET/release/ltools.exe"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'builder Wine no usa su target aislado para localizar el ejecutable'
grep -Fq 'windows\tests\e2e.ps1' "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows no ejecuta la E2E nativa'
grep -Fq 'native-process.ps1' "$ROOT_DIR/windows/tests/smoke.ps1" || fail 'smoke Windows sin capturador nativo estable'
grep -Fq 'native-process.ps1' "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'E2E Windows sin capturador nativo estable'
grep -Fq 'ReadToEndAsync' "$ROOT_DIR/windows/tests/native-process.ps1" || fail 'capturador Windows sin lectura asíncrona de pipes'
grep -Fq 'StandardOutputEncoding' "$ROOT_DIR/windows/tests/native-process.ps1" || fail 'capturador Windows sin UTF-8 explícito'
grep -Fq 'ValidateRange(1, 3600)' "$ROOT_DIR/windows/tests/native-process.ps1" || fail 'capturador Windows sin límite de timeout'
if rg -n '& \$Binary|LASTEXITCODE' "$ROOT_DIR/windows/tests" >/tmp/ltools-windows-fragile-runner.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-windows-fragile-runner.txt >&2
    fail 'los tests Windows conservan captura directa o LASTEXITCODE stale'
else
    ok 'tests Windows sin runner de procesos frágil'
fi
grep -Fq "Where-Object { \$_ -match '^Steam\\tmanifest\\t' }" "$ROOT_DIR/windows/tests/smoke.ps1" \
    || fail 'smoke Windows valida Steam con una comparación incorrecta sobre arrays'
if rg -n '\$launcherRows -notmatch' "$ROOT_DIR/windows/tests/smoke.ps1" >/tmp/ltools-windows-array-match.txt 2>/dev/null; then
    sed -n '1,20p' /tmp/ltools-windows-array-match.txt >&2
    fail 'smoke Windows conserva una aserción -notmatch vulnerable en arrays'
else
    ok 'smoke Windows valida inventarios sin falsos negativos por arrays'
fi
grep -Fq 'ltools-cli.exe' "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows no empaqueta el perfil CLI'
grep -Fq 'exe-cli' "$ROOT_DIR/rust/src/release.rs" || fail 'manifiesto sin tipo de ejecutable CLI Windows'
grep -Fq 'PackageArch' "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows no adapta el nombre a la arquitectura'
grep -Fq "ErrorActionPreference = 'Continue'" "$ROOT_DIR/windows/build.ps1" || fail 'builder Windows trata stderr normal de Cargo como fallo'
grep -Fq 'dist\windows' "$ROOT_DIR/windows/ltools.ps1" || fail 'lanzador PowerShell Windows no busca el output del builder'
grep -Fq "Join-Path \$root 'release'" "$ROOT_DIR/windows/ltools.ps1" || fail 'lanzador PowerShell Windows no busca la release común'
grep -Fq 'ltools.ps1' "$ROOT_DIR/windows/ltools.cmd" || fail 'lanzador CMD Windows no delega en el lanzador PowerShell'
grep -Fq 'ltools-cli.ps1' "$ROOT_DIR/windows/ltools-cli.cmd" || fail 'lanzador CLI CMD Windows no delega en PowerShell'
grep -Fq "Join-Path \$root 'release'" "$ROOT_DIR/windows/ltools-cli.ps1" || fail 'lanzador CLI Windows no busca la release común'
grep -Fq 'pause' "$ROOT_DIR/windows/build.cmd" || fail 'build.cmd no conserva el error visible al abrirse desde Explorer'
grep -Fq 'UMU-Latest' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'smoke Wine no prioriza UMU-Wine'
grep -Fq -- '--install-mono' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'smoke Wine no ofrece preparación opcional de Mono'
grep -Fq -- '--output DIR' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'helper Wine sin salida de artefactos'
grep -Fq -- '--no-tests' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'helper Wine sin modo build-only'
grep -Fq 'run_with_sudo(program' "$ROOT_DIR/rust/src/packages.rs" || fail 'limpieza no respeta el gestor seleccionado'
grep -Fq 'LC_ALL", "C' "$ROOT_DIR/rust/src/packages.rs" || fail 'consultas de paquetes no fijan locale estable'
if rg -n '(^|[[:space:]])wine([[:space:]]|$).*ltools|wine\.exe.*ltools|wine[[:space:]]+"' \
    "$ROOT_DIR/platform/linux/build.sh" "$ROOT_DIR/tests" "$ROOT_DIR/windows" "$ROOT_DIR/appimage" >/tmp/ltools-wine-tests.txt 2>/dev/null; then
    sed -n '1,40p' /tmp/ltools-wine-tests.txt >&2
    fail 'el pipeline contiene una ejecución Wine no encapsulada en el helper'
else
    ok 'ejecución Wine encapsulada y opt-in'
fi
ok 'contratos de identidad, assets e idiomas'

if ((failures)); then
    exit 1
fi
printf 'Contratos LTools correctos.\n'
