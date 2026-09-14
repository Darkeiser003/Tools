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
[[ -x "$ROOT_DIR/scripts/build.sh" ]] || fail 'falta el menú unificado Linux en scripts/build.sh'
[[ -f "$ROOT_DIR/scripts/build.ps1" ]] || fail 'falta el builder/menú unificado Windows en scripts/build.ps1'
grep -Fq 'Preview y ejecutar' "$ROOT_DIR/scripts/build.sh" || fail 'menú Linux sin sección de preview'
grep -Fq 'Probar lo ya compilado' "$ROOT_DIR/scripts/build.sh" || fail 'menú Linux sin sección de pruebas sin recompilar'
grep -Fq 'E2E de menús GUI y flujos funcionales' "$ROOT_DIR/scripts/build.sh" || fail 'menú de pruebas Linux no distingue la auditoría nativa independiente'
grep -Fq -- '--require-gui --binary "$RELEASE_BIN"' "$ROOT_DIR/scripts/build.sh" || fail 'pruebas manuales Linux permiten omitir la GUI'
grep -Fq -- '--require-dependencies --binary "$RELEASE_BIN"' "$ROOT_DIR/scripts/build.sh" || fail 'pruebas manuales Linux permiten omitir dependencias'
grep -Fq 'tests/scripts-syntax.sh' "$ROOT_DIR/scripts/build.sh" || fail 'builder/menú Linux no ofrece parseo de scripts sin compilar'
grep -Fq 'third-party-licenses.sh' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux no integra el paquete de licencias de dependencias'
grep -Fq 'ssh-signing.sh' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux no integra firmas SSH de release'
[[ -f "$ROOT_DIR/LICENSE" ]] || fail 'falta la licencia MIT del proyecto'
grep -Fq 'https://github.com/Darkeiser003' "$ROOT_DIR/LICENSE" || fail 'la licencia del proyecto no enlaza al perfil de GitHub del titular'
grep -Fq 'SHA256SUMS.txt.sshsig' "$ROOT_DIR/tests/release-e2e.sh" || fail 'E2E de release no verifica la firma SSH'
[[ -f "$ROOT_DIR/tests/ssh-signing.sh" ]] || fail 'falta la prueba funcional de firma SSH de release'
[[ -f "$ROOT_DIR/tests/third-party-licenses.sh" ]] || fail 'falta la prueba funcional del paquete de licencias'
grep -Fq 'THIRD-PARTY-LICENSES/INDEX.txt' "$ROOT_DIR/tests/linux/tarball-e2e.sh" || fail 'E2E del tarball no valida los avisos de licencias incluidos'
grep -Fq 'LICENSE' "$ROOT_DIR/tests/linux/tarball-e2e.sh" || fail 'E2E del tarball no valida la licencia del proyecto'
grep -Fq 'New-LToolsThirdPartyLicenseBundle' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no genera los avisos de dependencias'
grep -Fq 'THIRD-PARTY-LICENSES-windows.zip' "$ROOT_DIR/scripts/lib/build-state.ps1" || fail 'estado incremental Windows no vigila el archivo de licencias'
grep -Fq 'crate = "webpki-roots", allow = ["CDLA-Permissive-2.0"]' "$ROOT_DIR/deny.toml" || fail 'cargo-deny no limita CDLA a los datos de raíces TLS'
grep -Fxq '    "ISC",' "$ROOT_DIR/deny.toml" || fail 'cargo-deny no contempla ISC requerido por rustls-webpki/ring'
[[ -f "$ROOT_DIR/tests/scripts-syntax.sh" ]] || fail 'falta el comprobador de sintaxis de scripts'
grep -Fq 'Compilar solo el backend GUI' "$ROOT_DIR/scripts/build.sh" || fail 'menú Linux sin build de backend aislada'
grep -Fq 'Build rápida de desarrollo: sin paquetes, tests, smoke, E2E ni Wine' "$ROOT_DIR/scripts/build.sh" || fail 'menú Linux no advierte que la build rápida omite las pruebas'
grep -Fq -- '--fast --no-tests --no-package' "$ROOT_DIR/scripts/build.sh" || fail 'perfil rápido del menú no desactiva explícitamente cargo test'
grep -Fq 'Limpiar artefactos Windows' "$ROOT_DIR/scripts/build.ps1" || fail 'menú Windows sin limpieza de artefactos'
grep -Fq 'perfil rápido, conserva las pruebas' "$ROOT_DIR/scripts/build.ps1" || fail 'menú Windows confunde perfil rápido con omitir pruebas'
grep -Fq 'El smoke también necesita ltools-cli.exe' "$ROOT_DIR/scripts/build.ps1" || fail 'menú Windows no verifica el perfil CLI previo al smoke'
grep -Fxq '/dist/' "$ROOT_DIR/.gitignore" || fail 'Git no excluye la carpeta de salida predeterminada dist'
grep -Fxq '/release/' "$ROOT_DIR/.gitignore" || fail 'Git no excluye la carpeta de releases locales predeterminada'
for github_workflow in ci.yml workflow-security.yml dependency-review.yml; do
    [[ -f "$ROOT_DIR/.github/workflows/$github_workflow" ]] ||
        fail "falta el workflow GitHub Actions $github_workflow"
done
grep -Fq 'branches: [main]' "$ROOT_DIR/.github/workflows/ci.yml" || fail 'CI no se ejecuta en main'
grep -Fq -- '--windows-wine --no-appimage --allow-unsigned' "$ROOT_DIR/.github/workflows/ci.yml" || fail 'CI Linux no ejecuta build completa con Wine'
grep -Fq -- '-Force -AllowUnsigned -NonInteractive' "$ROOT_DIR/.github/workflows/ci.yml" || fail 'CI Windows no fuerza build y pruebas nativas completas'
grep -Fq 'cargo audit --file rust/Cargo.lock' "$ROOT_DIR/.github/workflows/ci.yml" || fail 'CI no audita advisories del lockfile Rust'
grep -Fq 'cargo deny --manifest-path rust/Cargo.toml --config deny.toml check' "$ROOT_DIR/.github/workflows/ci.yml" || fail 'CI no valida licencias/fuentes con deny.toml'
grep -Fq 'actionlint -color' "$ROOT_DIR/.github/workflows/workflow-security.yml" || fail 'falta la validación estática de GitHub Actions'
grep -Fq 'shellcheck --severity=error' "$ROOT_DIR/.github/workflows/workflow-security.yml" || fail 'falta ShellCheck para scripts Bash'
grep -Fq 'gitleaks/gitleaks-action@v3' "$ROOT_DIR/.github/workflows/workflow-security.yml" || fail 'falta el escaneo de secretos compatible con Node 24'
grep -Fq "GITLEAKS_ENABLE_COMMENTS: 'false'" "$ROOT_DIR/.github/workflows/workflow-security.yml" || fail 'Gitleaks puede comentar pull requests con resultados'
if grep -Fq 'security-events: write' "$ROOT_DIR/.github/workflows/workflow-security.yml"; then
    fail 'la auditoría de workflows solicita permisos de escritura en PR no confiables'
fi
grep -Fq 'advanced-security: false' "$ROOT_DIR/.github/workflows/workflow-security.yml" || fail 'zizmor necesita salida local y permisos de escritura innecesarios'
grep -Fq 'actions/dependency-review-action@v5' "$ROOT_DIR/.github/workflows/dependency-review.yml" || fail 'la revisión de dependencias no usa la acción Node 24'
grep -Fq 'fail-on-severity: high' "$ROOT_DIR/.github/workflows/dependency-review.yml" || fail 'la revisión de dependencias no bloquea vulnerabilidades altas/críticas'
grep -Fq 'package-ecosystem: cargo' "$ROOT_DIR/.github/dependabot.yml" || fail 'Dependabot no mantiene las dependencias Rust'
grep -Fq 'package-ecosystem: github-actions' "$ROOT_DIR/.github/dependabot.yml" || fail 'Dependabot no mantiene las acciones GitHub'
if git -C "$ROOT_DIR" check-ignore -q .github/workflows/ci.yml; then
    fail '.gitignore oculta los workflows GitHub del repositorio'
fi
if grep -Eq '^[*].*filter=lfs' "$ROOT_DIR/.gitattributes"; then
    fail 'Git LFS se activa globalmente para todos los ficheros'
fi
grep -Fq '* text=auto eol=lf' "$ROOT_DIR/.gitattributes" || fail 'faltan finales de línea consistentes en .gitattributes'
grep -Fq '*.png binary' "$ROOT_DIR/.gitattributes" || fail '.gitattributes no marca recursos PNG como binarios'
if rg -n '^\*\.(AppImage|appimage|tar\.gz|tar\.xz|tar\.zst|zip|exe|dll|so|log)$' "$ROOT_DIR/.gitignore"; then
    fail '.gitignore oculta globalmente extensiones que podrían pertenecer a fuentes o fixtures'
fi
grep -Fq 'previousCliMode' "$ROOT_DIR/scripts/build.ps1" || fail 'preview CLI Windows no conserva la variable de entorno previa'
grep -Fq '[switch]$NoSmoke' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no permite omitir solo el smoke'
grep -Fq '[switch]$NoE2E' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no permite omitir solo la E2E'
grep -Fq '$needSmoke = -not ($NoTests -or $NoRun -or $NoSmoke)' "$ROOT_DIR/scripts/build.ps1" || fail 'el skip smoke Windows mezcla flags o no conserva los alias NoRun/NoTests'
grep -Fq '$needE2E = -not ($NoTests -or $NoRun -or $NoE2E)' "$ROOT_DIR/scripts/build.ps1" || fail 'el skip E2E Windows mezcla flags o no conserva los alias NoRun/NoTests'
grep -Fq 'Alias compatible: omite smoke y E2E, pero conserva cargo test.' "$ROOT_DIR/scripts/build.ps1" || fail 'ayuda Windows documenta mal NoRun/NoTests'
grep -Fq 'storage-map-gui-e2e.sh' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline sin E2E de acciones reales del mapa GUI'
[[ -x "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" ]] || fail 'la E2E del mapa GUI no se puede ejecutar desde el pipeline'
grep -Fq -- '--require-gui' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline sin exigir la E2E real/capturas del mapa GUI'
grep -Fq 'smoke.sh" --require-gui' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline puede aceptar smoke GUI omitido'
grep -Fq 'menu-e2e.sh" --require-gui' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline puede aceptar menú GUI omitido'
grep -Fq 'e2e.sh" --require-dependencies' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline puede aceptar E2E de rollback omitida'
grep -Fq 'tarball-e2e.sh" --tarball "$selected" --require-gui' "$ROOT_DIR/scripts/build.sh" || fail 'menú puede aceptar GUI de tarball omitida'
grep -Fq 'dist/local-release' "$ROOT_DIR/scripts/build.sh" || fail 'menú Linux sin carpeta de publicación local separada'
grep -Fq -- '--output "$ROOT_DIR/dist/local"' "$ROOT_DIR/scripts/build.sh" || fail 'menú Linux local anida el staging dentro de la publicación'
grep -Fq 'REQUIRE_GUI=1' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'smoke no ofrece modo GUI obligatorio'
grep -Fq 'REQUIRE_GUI=1' "$ROOT_DIR/tests/linux/menu-e2e.sh" || fail 'E2E de menú no ofrece modo GUI obligatorio'
grep -Fq 'REQUIRE_DEPENDENCIES=1' "$ROOT_DIR/tests/linux/e2e.sh" || fail 'E2E migración/rollback no ofrece modo dependencias obligatorias'
grep -Fq 'native-help-e2e.sh' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline sin la E2E de ayudas nativas'
grep -Fq 'unknown option|unrecognized option|invalid option' "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'E2E Windows puede aceptar una opción de ayuda nativa inválida'
grep -Fq "Tool = 'ssh'; Args = @()" "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'E2E Windows invoca una opción de ayuda no portable de OpenSSH'
grep -Fq "Tool = 'git'; Args = @('help', '-a')" "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'E2E Windows puede abrir un pager/navegador en vez de mostrar comandos Git'
grep -Fq '"winslim" => Some((7, "winslim"' "$ROOT_DIR/rust/src/guides.rs" || fail 'guía Windows no reconoce la pantalla WinSlim condicional'
grep -Fq "gui_pages+=(7)" "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Wine no abre/captura la página WinSlim cuando WSCore existe'
grep -Fq 'no tiene disponible la pantalla WinSlim' "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'E2E Windows no valida el caso WinSlim sin WSCore'
grep -Fq 'xdotool search --onlyvisible --name "WinSlim-Tools"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine busca un título distinto del producto Windows'
grep -Fq 'button_y=$((87 + page * 50))' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine no deriva la coordenada real de cada categoría'
grep -Fq 'GUI_PAGE_OK=%s' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine no registra la apertura individual de cada categoría'
grep -Fq 'C:\\windows\\temp\\$gui_marker_name' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine guarda el marcador en una ruta visible desde Win32'
grep -Fq 'for page in "${gui_pages[@]}"; do' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine no recorre todas las categorías visibles'
grep -Fq '[[ "$page_colors" -ge 4 ]]' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine exige una paleta mayor que la GUI plana realmente usa'
grep -Fq 'windows-settings-wine.png' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine no valida una captura Win32 de Ajustes'
grep -Fq 'import -window root "$capture"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine captura el HWND negro en vez de la pantalla visible'
grep -Fq 'xdotool windowclose "$window_id"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine no cierra explícitamente la ventana temporal'
grep -Fq 'wine_args+=(--require-gui)' "$ROOT_DIR/scripts/build.sh" || fail 'build completa puede aceptar la E2E GUI Windows/Wine omitida'
grep -Fq 'REQUIRE_GUI=1' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'smoke Windows/Wine no ofrece exigir capturas y GUI'
grep -Fq 'LTOOLS_GUI_CAPTURE_DIR=$WINDOWS_WINE_ARTIFACT_DIR/captures' "$ROOT_DIR/scripts/build.sh" || fail 'capturas Windows/Wine pueden sobrescribir capturas ajenas al build'
grep -Fq '1 => Some("native_tools")' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Win32 cruza el ajuste Herramientas nativas con Dependencias'
grep -Fq '2 => Some("dependencies")' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Win32 cruza el ajuste Dependencias con Herramientas nativas'
grep -Fq 'pub(crate) fn windows_menu_labels' "$ROOT_DIR/rust/src/gui.rs" || fail 'guía Windows no puede leer el catálogo real de botones GUI'
grep -Fq 'crate::gui::windows_menu_labels(page)' "$ROOT_DIR/rust/src/guides.rs" || fail 'índice de guías Windows duplica opciones en vez de usar el catálogo GUI'
grep -Fq 'windows_gui_unavailable(topic)' "$ROOT_DIR/rust/src/guides.rs" || fail 'faltan guías Windows explícitas para temas sin pantalla propia'
grep -Fq 'native_action_text("adb_devices")' "$ROOT_DIR/rust/src/gui.rs" || fail 'botón ADB Win32 no obtiene su etiqueta del catálogo nativo'
grep -Fq 'crate::i18n::automation_text(key)' "$ROOT_DIR/rust/src/gui.rs" || fail 'botones de automatización Win32 pueden quedar sin etiqueta'
grep -Fq 'every_visible_win32_action_has_a_label' "$ROOT_DIR/rust/src/gui.rs" || fail 'tests Rust no exigen etiquetas para todos los botones Win32'
grep -Fq 'let edit_x = 248;' "$ROOT_DIR/rust/src/gui.rs" || fail 'formulario de automatización Win32 no reserva ancho para las etiquetas'
grep -Fq '«\s*»' "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'E2E Windows no rechaza opciones GUI sin etiqueta'
grep -Fq 'no se pudo ejecutar {shell}: {error}' "$ROOT_DIR/rust/src/diagnostics/windows.rs" || fail 'sondeo PowerShell Windows descarta errores de lanzamiento'
grep -Fq 'no está integrado en la' "$ROOT_DIR/rust/src/guides.rs" || fail 'guía GUI Windows anuncia el mapa interactivo ausente'
grep -Fq 'Resumen de espacio y montajes' "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'E2E Windows no compara la guía con botones GUI reales'
grep -Fq 'windows-native-tools-wine.png' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Windows/Wine no captura la categoría abierta mediante clic'
grep -Fq '"scripts"' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows sin vigilar cambios de scripts'
grep -Fq "^rust/(src/|tests/|Cargo" "$ROOT_DIR/scripts/lib/build-state.ps1" || fail 'builder Windows sin clasificar cambios de pruebas Rust'
grep -Fq 'foreach ($key in (Get-LToolsMapKeys $Old))' "$ROOT_DIR/scripts/lib/build-state.ps1" || fail 'builder Windows no incorpora archivos borrados a la matriz de impacto'
grep -Fq '$rustTestDeletion = ' "$ROOT_DIR/tests/build-state.ps1" || fail 'la matriz no prueba que borrar pruebas Rust vuelva a ejecutarlas'
grep -Fq "path -match '^distribution/'" "$ROOT_DIR/scripts/lib/build-state.ps1" || fail 'cambios del descriptor de distribución no fuerzan el empaquetado'
grep -Fq "path -match '^scripts/build\.ps1$'" "$ROOT_DIR/scripts/lib/build-state.ps1" || fail 'un cambio del propio builder Windows no fuerza el empaquetado'
grep -Fq '$readmeChange = ' "$ROOT_DIR/tests/build-state.ps1" || fail 'el cambio del README empaquetado no está probado'
grep -Fq 'rust/target|windows/(target|bin|obj)|dist' "$ROOT_DIR/scripts/build.ps1" || fail 'fingerprint Windows incluye targets y staging generados'
[[ -f "$ROOT_DIR/appimage/ltools.desktop" ]] || fail 'falta el descriptor LTools'
[[ -f "$ROOT_DIR/appimage/ltools.svg" ]] || fail 'falta el icono LTools'
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
[[ -x "$ROOT_DIR/tests/linux/native-help-e2e.sh" ]] || fail 'falta la E2E ejecutable de ayudas nativas'
grep -Fq 'capture_screen' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa sin capturas verificadas'
for localized_suite in smoke.sh menu-e2e.sh e2e.sh software-git-e2e.sh native-help-e2e.sh tarball-e2e.sh windows-wine.sh; do
    grep -Fq 'export LTOOLS_LANG=es' "$ROOT_DIR/tests/linux/$localized_suite" || fail "E2E $localized_suite hereda un idioma externo pese a comparar mensajes españoles"
done
grep -Fq 'capture_window "$MAP_WINDOW"' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no captura la ventana independientemente del fondo'
grep -Fq 'windowsize --sync "$MAP_WINDOW" 640 480' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no comprueba una ventana compacta 640x480'
grep -Fq 'STORAGE_TREE_LAYOUT' "$ROOT_DIR/rust/src/gui.rs" || fail 'el mapa no reajusta el ancho de columna al viewport real'
grep -Fq 'wait_for_tree_layout "$RUN_DIR/$tag.layout" 1' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no detecta recorte horizontal de texto al redimensionar'
grep -Fq 'TMP_DIR="$(realpath -m -- "$TMP_DIR")"' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no normaliza rutas absolutas para revisar rutas estándar'
grep -Fq 'STORAGE_TREE_EXPLANATION' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no exige explicación completa de rutas estándar'
grep -Fq 'storage_tree_text_width_tracks_the_actual_viewport' "$ROOT_DIR/rust/src/gui.rs" || fail 'el ancho adaptable del mapa no tiene prueba unitaria'
grep -Fq 'small_height > 480' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no detecta recorte vertical en pantallas compactas'
grep -Fq -- '--screen 640x480 --layout-only' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no abre en una pantalla real de 640x480'
grep -Fq 'LTOOLS_GUI_TREE_READY_MARKER' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no espera al árbol poblado antes de validar el layout'
grep -Fq 'ACTION_ROW_Y=$((large_height - 90))' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa usa una coordenada obsoleta para la fila de acciones'
grep -Fq 'capture_geometry" != "$SCREEN' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa no comprueba la resolución real de la captura compacta'
grep -Fq 'resized-640x480-on-${SCREEN}' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'captura redimensionada mal etiquetada como pantalla 640x480'
grep -Fq 'xdotool getdisplaygeometry' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa depende de coordenadas absolutas en el diálogo'
grep -Fq 'conservan fixture y logs para diagnóstico' "$ROOT_DIR/tests/linux/storage-map-gui-e2e.sh" || fail 'E2E del mapa elimina evidencia al fallar'
grep -Fq 'Smoke fallido; temporales conservados para diagnóstico' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'smoke elimina la evidencia temporal al fallar'
grep -Fq 'body_scrolled' "$ROOT_DIR/rust/src/gui.rs" || fail 'el mapa no mantiene el contenido en un viewport acotado'
grep -Fq 'gtk_box_pack_start(content, outer, 1, 1, 0)' "$ROOT_DIR/rust/src/gui.rs" || fail 'el contenido del mapa no se expande dentro del viewport'
grep -Fq 'if screen_width < 820' "$ROOT_DIR/rust/src/gui.rs" || fail 'los controles del mapa no se reorganizan en pantallas estrechas'
grep -Fq 'check_surface git git' "$ROOT_DIR/tests/linux/native-help-e2e.sh" || fail 'ayudas nativas sin contrato GUI para Git'
grep -Fq 'check_surface gh git' "$ROOT_DIR/tests/linux/native-help-e2e.sh" || fail 'ayudas nativas sin contrato GUI para GitHub CLI'
grep -Fq 'ssh|scp|sftp)' "$ROOT_DIR/tests/linux/native-help-e2e.sh" || fail 'la E2E usa el argumento de ayuda nativo de OpenSSH'
grep -Fq 'unknown option|unrecognized option|invalid option' "$ROOT_DIR/tests/linux/native-help-e2e.sh" || fail 'ayudas nativas pueden aprobar una opción de ayuda inválida'
grep -Fq '[[ -s "$output" ]] || return 1' "$ROOT_DIR/tests/linux/native-help-e2e.sh" || fail 'ayudas nativas pueden aprobar una salida vacía'
grep -Fq 'el sandbox bloqueó socket()' "$ROOT_DIR/tests/linux/native-help-e2e.sh" || fail 'ayuda de dig no diferencia los límites del sandbox de una salida válida'
grep -Fq 'SSH/SCP/SFTP' "$ROOT_DIR/tests/linux/native-help-e2e.sh" || fail 'ayudas nativas sin contrato de transferencia SSH'
grep -Fq 'SETTINGS_GUIDE_LABELS' "$ROOT_DIR/tests/linux/menu-e2e.sh" || fail 'E2E sin cobertura de la guía Ajustes multilingüe'
grep -Fq 'linux-settings-bottom.png' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'smoke GUI sin captura del final de Ajustes Linux'
grep -Fq 'windows-settings-wine.png' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Wine sin captura de Ajustes Windows'
[[ -f "$ROOT_DIR/appimage/org.ltools.LTools.metainfo.xml.in" ]] || fail 'falta el manifiesto AppStream'
[[ -f "$ROOT_DIR/appimage/ltools-capabilities.schema.json" ]] || fail 'falta el esquema JSON de capacidades'
[[ -f "$ROOT_DIR/appimage/ltools-terminal.schema.json" ]] || fail 'falta el esquema JSON de integración de terminal'
[[ -f "$ROOT_DIR/distribution/ltools-project.json" ]] || fail 'falta el descriptor declarativo del proyecto'
[[ -f "$ROOT_DIR/distribution/ltools-project.schema.json" ]] || fail 'falta el esquema del descriptor de proyecto'
[[ -f "$ROOT_DIR/distribution/ltools-release.schema.json" ]] || fail 'falta el esquema del manifiesto de release'
[[ -x "$ROOT_DIR/appimage/AppRun-cli" ]] || fail 'falta el AppRun del perfil CLI'
[[ -f "$ROOT_DIR/appimage/ltools-cli.desktop" ]] || fail 'falta el descriptor del perfil CLI'
grep -Fq 'appstreamcli validate --no-net' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin validación explícita AppStream'
grep -Fq 'appimagetool --no-appstream' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin modo AppStream explícito'
[[ -f "$ROOT_DIR/README.md" ]] || fail 'falta el README del proyecto'
grep -Fq -- '--dry-run' "$ROOT_DIR/scripts/build.sh" || fail 'menú sin modo de simulación predeterminado'
grep -Fq -- '--plans-only' "$ROOT_DIR/scripts/build.sh" || fail 'limpiador sin modo aislado para planes legacy'
if grep -Fq -- 'git clean' "$ROOT_DIR/scripts/build.sh"; then
    fail 'limpiador no debe delegar en git clean'
fi
grep -Fq -- 'No se borran fuentes' "$ROOT_DIR/scripts/build.sh" || fail 'limpiador sin protección de fuentes'
grep -Fq 'Git no puede verificar qué archivos del proyecto deben protegerse' "$ROOT_DIR/scripts/build.sh" || fail 'limpiador Linux puede borrar salidas sin verificar archivos no versionados'
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
grep -Fq 'set_elevate_by_default' "$ROOT_DIR/rust/src/gui_preferences.rs" || fail 'preferencias sin elevación por defecto'
grep -Fq 'fn classify' "$ROOT_DIR/rust/src/privilege.rs" || fail 'acciones sin clasificación de privilegios'
grep -Fq -- '--elevate' "$ROOT_DIR/rust/src/main.rs" || fail 'CLI sin opción de elevación explícita'
grep -Fq 'elevated_args.extend(["--plan"' "$ROOT_DIR/rust/src/main.rs" || fail 'relanzamiento elevado sin conservar el plan'
grep -Fq 'gui_privilege_available' "$ROOT_DIR/rust/src/platform/mod.rs" || fail 'GUI sin detección centralizada de autorización'
grep -Fq 'sudo_available_without_prompt' "$ROOT_DIR/rust/src/storage_map.rs" || fail 'mapa elevado sin alternativa sudo no interactiva'
grep -Fq 'drop(Box::from_raw(pointer as *mut StorageTreeLoadUi))' "$ROOT_DIR/rust/src/gui.rs" || fail 'timeout del mapa GUI fuga el estado del callback'
grep -Fq 'selected_storage_tree_path' "$ROOT_DIR/rust/src/gui.rs" || fail 'mapa sin selección de rutas para acciones contextuales'
grep -Fq 'operation=manage-copy' "$ROOT_DIR/rust/src/gui.rs" || fail 'mapa sin acción contextual de copia'
grep -Fq 'operation=manage-move' "$ROOT_DIR/rust/src/gui.rs" || fail 'mapa sin acción contextual de movimiento'
grep -Fq 'operation=manage-delete' "$ROOT_DIR/rust/src/gui.rs" || fail 'mapa sin acción contextual de papelera'
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
grep -Fq 'explicit && existing_plan' "$ROOT_DIR/rust/src/common.rs" || fail 'los planes explícitos válidos no se reutilizan sin truncarse'
grep -Fq 'plan_creation_never_truncates_an_unrecognized_existing_file' "$ROOT_DIR/rust/src/common.rs" || fail 'plan --plan puede sobrescribir archivos que no son planes LTools'
grep -Fq 'plan_creation_preserves_a_file_with_only_the_plan_marker' "$ROOT_DIR/rust/src/common.rs" || fail 'plan --plan no protege archivos parciales que solo imitan la marca LTools'
grep -Fq 'plan_creation_preserves_a_plan_with_a_malformed_record' "$ROOT_DIR/rust/src/common.rs" || fail 'plan --plan no protege un registro truncado antes de reutilizarlo'
grep -Fq 'fn is_ltools_plan(path: &Path)' "$ROOT_DIR/rust/src/common.rs" || fail 'plan --plan no verifica la cabecera completa antes de truncar o reutilizar'
grep -Fq 'ensure_private_plan_directory' "$ROOT_DIR/rust/src/common.rs" || fail 'planes automáticos sin directorio privado en Unix'
grep -Fq 'options.mode(0o600)' "$ROOT_DIR/rust/src/common.rs" || fail 'planes nuevos pueden ser legibles por otros usuarios'
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
grep -Fq 'native-help-e2e.sh' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'smoke sin auditoría de ayudas nativas'
grep -Fq 'montaje real se comprobará en el smoke' "$ROOT_DIR/scripts/build.sh" || fail 'preflight FUSE afirma capacidad sin probar montaje'
grep -Fq 'montaje FUSE no autorizado por el host' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'smoke no distingue FUSE detectado de montaje realmente permitido'
grep -Fq 'APPIMAGE_EXTRACT_AND_RUN=1 timeout 30 "$APPIMAGE_PATH" --doctor' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'smoke sin fallback AppImage cuando el montaje FUSE falla'
grep -Fq 'smoke_args+=(--require-fuse)' "$ROOT_DIR/scripts/build.sh" || fail 'build no propaga la exigencia explícita de FUSE al smoke'
grep -Fq 'run_with_fuse_fallback()' "$ROOT_DIR/appimage/run-ltools.sh" || fail 'lanzador AppImage sin fallback cuando FUSE está detectado pero denegado'
grep -Fq 'run_with_fuse_fallback()' "$ROOT_DIR/release/run-ltools.sh" || fail 'lanzador de release sin fallback cuando FUSE está detectado pero denegado'
grep -Fq 'FUSE está anunciado, pero el montaje fue rechazado' "$ROOT_DIR/appimage/run-ltools.sh" || fail 'lanzador AppImage no diagnostica el rechazo real de FUSE'
grep -Fq 'actual mount permission unverified' "$ROOT_DIR/rust/src/main.rs" || fail 'doctor presenta los requisitos FUSE como capacidad de montaje verificada'
grep -Fq 'lterminal-startup-v1' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun sin comprobación de protocolo LTerminal'
grep -Fq 'launch_standard_terminal' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun sin ventana de terminal autónoma'
grep -Fq 'select_shell' "$ROOT_DIR/appimage/AppRun" || fail 'AppRun sin selección de shell autónoma'
grep -Fq 'INTERACTIVE_TTY=0' "$ROOT_DIR/scripts/build.sh" || fail 'builder sin detección TTY previa al log'
grep -Fq 'ltools-capabilities-v1' "$ROOT_DIR/rust/src/compat.rs" || fail 'backend sin contrato JSON de capacidades'
grep -Fq 'host_tools' "$ROOT_DIR/rust/src/compat.rs" || fail 'contrato sin catálogo de herramientas del anfitrión'
grep -Fq 'rsync' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin migración verificada'
grep -Fq 'sc.exe' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin control de servicios'
grep -Fq 'install_package' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'catálogo Linux sin metadatos de instalación'
grep -Fq 'install_package' "$ROOT_DIR/rust/src/platform/windows.rs" || fail 'catálogo Windows sin metadatos de instalación'
grep -Fq 'dependency_confirmation' "$ROOT_DIR/rust/src/common.rs" || fail 'instalación sin confirmación detallada de dependencia'
grep -Fq 'Paquete que se instalará' "$ROOT_DIR/rust/src/common.rs" || fail 'confirmación sin paquete de dependencia'
grep -Fq 'tests/encoding.sh' "$ROOT_DIR/scripts/build.sh" || fail 'build sin auditoría de codificaciones'
grep -Fq 'tests/release-e2e.sh' "$ROOT_DIR/scripts/build.sh" || fail 'build sin E2E de publicación release'
grep -Fq 'ltools_create_release_staging' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux modifica release sin staging previo'
grep -Fq 'ltools_promote_release_staging' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux no espera al E2E antes de promover release'
grep -Fq 'scripts/lib/publish.sh' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin utilidades de promoción recuperable'
[[ -f "$ROOT_DIR/tests/publish-release.sh" ]] || fail 'falta la prueba aislada de promoción de release'
grep -Fq -- '--exchange -T --no-copy' "$ROOT_DIR/scripts/lib/publish.sh" || fail 'promoción Linux sin intercambio atómico cuando está disponible'
grep -Fq 'restauró la release previa' "$ROOT_DIR/scripts/lib/publish.sh" || fail 'fallback Linux no restaura release ante fallo'
linux_release_e2e_line="$(grep -nF 'run_logged "$ROOT_DIR/tests/release-e2e.sh"' "$ROOT_DIR/scripts/build.sh" | head -n1 | cut -d: -f1)"
linux_release_promote_line="$(grep -nF 'ltools_promote_release_staging ||' "$ROOT_DIR/scripts/build.sh" | head -n1 | cut -d: -f1)"
[[ -n "$linux_release_e2e_line" && -n "$linux_release_promote_line" && "$linux_release_e2e_line" -lt "$linux_release_promote_line" ]] || fail 'builder Linux promueve release antes del E2E'
grep -Fq 'release_e2e_args+=(--no-package)' "$ROOT_DIR/scripts/build.sh" || fail 'build sin propagar --no-package a la E2E release'
grep -Fq 'release_e2e_args+=(--no-appimage)' "$ROOT_DIR/scripts/build.sh" || fail 'build sin propagar --no-appimage a la E2E release'
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
grep -Fq '"modify" | "edit" | "update"' "$ROOT_DIR/rust/src/automation.rs" || fail 'automatizaciones sin edición CLI'
grep -Fq 'automation-modify' "$ROOT_DIR/rust/src/automation.rs" || fail 'automatizaciones sin registro transaccional de edición'
grep -Fq 'on_automation_entry_action' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin acciones por script registrado'
grep -Fq 'Scripts registrados' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin submenú de scripts registrados'
grep -Fq '(5, 4) => Some(("automation", &[], "remove"))' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Windows sin borrar automatizaciones'
grep -Fq 'menu-audit-inventory' "$ROOT_DIR/rust/src/main.rs" || fail 'menú sin categoría de auditoría/inventario'
grep -Fq 'menu-import' "$ROOT_DIR/rust/src/main.rs" || fail 'menú sin categoría Importar'
grep -Fq 'menu-settings' "$ROOT_DIR/rust/src/main.rs" || fail 'menú sin categoría Ajustes'
grep -Fq 'mod cli_ui;' "$ROOT_DIR/rust/src/main.rs" || fail 'CLI sin capa visual compartida'
grep -Fq 'cli_ui::header' "$ROOT_DIR/rust/src/main.rs" || fail 'CLI sin cabecera/contexto persistente'
grep -Fq 'winslim_available' "$ROOT_DIR/rust/src/main.rs" || fail 'menú Windows sin detección condicional de WinSlim'
grep -Fq 'standalone_releases_require_it' "$ROOT_DIR/rust/src/compat.rs" || fail 'contrato sin independencia de LTerminal'
grep -Fq 'capabilities --format json' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin descriptor JSON generado'
grep -Fq 'capabilities --format json' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows sin descriptor JSON generado'
grep -Fq 'capabilities --format terminal-json' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin descriptor JSON de terminal'
grep -Fq 'capabilities --format terminal-json' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows sin descriptor JSON de terminal'
grep -Fq 'CAPABILITIES_SCHEMA_ARTIFACT=' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux no publica el esquema de capacidades'
grep -Fq "ltools-capabilities.schema.json') -Destination \$packageStageDir" "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows no prepara el esquema de capacidades en staging'
grep -Fq 'ltools-capabilities.schema.json' "$ROOT_DIR/tests/release-e2e.sh" || fail 'E2E release no exige el esquema de capacidades'
grep -Fq 'release-manifest' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin manifiesto verificable de release'
grep -Fq 'release-checksums' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin SHA256SUMS reproducible'
grep -Fq -- '--linux-arch "$ARCH"' "$ROOT_DIR/scripts/build.sh" || fail 'E2E release Linux asume x86_64 aunque el builder admite ARM'
grep -Fq -- '--windows-arch "${WINDOWS_TARGET%%-*}"' "$ROOT_DIR/scripts/build.sh" || fail 'E2E release Windows asume x86_64 aunque el target Wine es configurable'
grep -Fq 'strip_prefix(&version_prefix)' "$ROOT_DIR/rust/src/release.rs" || fail 'manifiesto release no reconoce versiones prerelease'
grep -Fq 'file_type()' "$ROOT_DIR/rust/src/release.rs" || fail 'manifiesto/checksums siguen dereferenciando symlinks de artefactos'
grep -Fq 'release_coordinates_reject_empty_and_path_like_values' "$ROOT_DIR/rust/src/release.rs" || fail 'coordenadas de release sin regresiones de seguridad'
grep -Fq 'release-signature' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin firma Ed25519'
grep -Fq 'SIGNING_REQUIRED=1' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux no exige firma por defecto'
grep -Fq '$SigningRequired = $true' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows no exige firma por defecto'
grep -Fq 'copy_launcher()' "$ROOT_DIR/scripts/build.sh" || fail 'paquete Linux depende de lanzadores no versionados'
grep -Fq 'fallback="${name}.sh"' "$ROOT_DIR/scripts/build.sh" || fail 'paquete Linux sin fallback para checkout limpio'
grep -Fq -- "-name 'ltools-*.tar.gz'" "$ROOT_DIR/scripts/build.sh" || fail 'release no limpia tarballs antiguos'
grep -Fq -- '--release-dir' "$ROOT_DIR/scripts/build.sh" || fail 'build Linux sin carpeta de publicación configurable'
grep -Fq -- '--windows-wine' "$ROOT_DIR/scripts/build.sh" || fail 'build raíz sin opción Windows bajo Wine/Proton'
grep -Fq 'windows-wine.sh' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline sin etapa Windows bajo Wine/Proton'
grep -Fq -- '--windows-wine-runner' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline sin runner Windows configurable'
grep -Fq -- '--windows-wine-prefix' "$ROOT_DIR/scripts/build.sh" || fail 'pipeline sin prefijo Windows configurable'
grep -Fq 'WINDOWS_WINE_TARGET_EXE="$WINDOWS_WINE_CARGO_TARGET_DIR/$WINDOWS_TARGET/release/ltools.exe"' "$ROOT_DIR/scripts/build.sh" || fail 'build sin validar el EXE fuente cuando se omite empaquetar Windows'
grep -Fq 'WINDOWS_WINE_TARGET_CLI_EXE="$WINDOWS_WINE_CARGO_TARGET_DIR/$WINDOWS_TARGET/release/ltools-cli.exe"' "$ROOT_DIR/scripts/build.sh" || fail 'build sin validar el CLI fuente cuando se omite empaquetar Windows'
grep -Fq '[[ -s "$WINDOWS_WINE_TARGET_EXE" ]]' "$ROOT_DIR/scripts/build.sh" || fail 'build exige artefactos copiados aunque Windows use --no-package'
grep -Fq 'pruebas bajo Wine omitidas por configuración' "$ROOT_DIR/scripts/build.sh" || fail 'build no distingue la compilación Windows de las pruebas Wine omitidas'
grep -Fq 'run_windows_timeout --elevate native tools' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Wine no comprueba que el inventario nativo siga siendo de solo lectura con elevación predeterminada'
grep -Fq 'WINDOWS_LAUNCHERS' "$ROOT_DIR/rust/src/games.rs" || fail 'inventario Windows sin catálogo nativo de lanzadores'
grep -Fq 'games-windows-native' "$ROOT_DIR/rust/src/games.rs" || fail 'inventario Windows sin modo nativo explícito'
grep -Fq 'No se buscan prefijos Wine' "$ROOT_DIR/rust/src/games.rs" || fail 'inventario Windows no documenta la exclusión de Wine'
grep -Fq 'LTOOLS_CLI' "$ROOT_DIR/rust/src/main.rs" || fail 'backend sin perfil CLI explícito'
grep -Fq 'cli = []' "$ROOT_DIR/rust/Cargo.toml" || fail 'Cargo sin feature CLI separada'
grep -Fq -- '--features cli' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'builder Wine no compila el perfil CLI separado'
grep -Fq -- "'--features', 'cli'" "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no compila el perfil CLI separado'
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
grep -Fq 'action_buttons: [[HWND; ACTION_COUNT]' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Win32 sin capacidad para acciones ampliadas'
grep -Fq 'const ACTION_STRIDE: i32 = 18' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Win32 sin stride suficiente para IDs de acciones'
grep -Fq 'const PAGE_COUNT: usize = 9' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Win32 no reserva la página de usuarios y grupos'
grep -Fq 'const ACCOUNT_PAGE: usize = 8' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI Win32 sin índice estable para usuarios y grupos'
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
grep -Fq 'crate::formatting::table' "$ROOT_DIR/rust/src/accounts/linux.rs" || fail 'cuentas Linux sin salidas tabuladas'
grep -Fq 'Format-Table' "$ROOT_DIR/rust/src/accounts/windows.rs" || fail 'cuentas Windows sin salidas tabuladas'
grep -Fq 'ACCOUNT_CREATE_FIELDS' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin formulario de creación de cuentas'
grep -Fq 'ACCOUNT_MEMBERSHIP_FIELDS' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin formulario de membresías'
grep -Fq 'on_account_password' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin cambio de contraseña seguro'
grep -Fq 'Usuarios, grupos y sesiones' "$ROOT_DIR/rust/src/gui.rs" || fail 'GUI sin submenú propio de cuentas'
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
grep -Fq 'release-manifest' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows sin manifiesto verificable de release'
grep -Fq 'release-checksums' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows sin SHA256SUMS reproducible'
grep -Fq 'release-signature' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows sin firma Ed25519'
grep -Fq 'Invoke-LToolsSshManifestSigning' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows sin firma OpenSSH de release'
grep -Fq 'SHA256SUMS.txt.sshsig' "$ROOT_DIR/rust/src/release.rs" || fail 'checksums Rust no excluye la firma SSH para evitar circularidad'
grep -Fq 'tests/ssh-signing.sh' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux sin prueba funcional de la firma SSH'
grep -Fq 'ReleaseOutput' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows sin carpeta de publicación configurable'
grep -Fq 'Publish-StagedDirectory $releaseStageDir $PublishDir' "$ROOT_DIR/scripts/build.ps1" || fail 'build Windows publica directamente antes de validar'
grep -Fq 'tests\release-e2e.ps1' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows sin E2E de release antes de promover'
grep -Fq 'New-StagedDirectory $PublishDir' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no copia la release previa a staging'
grep -Fq 'function New-StagedDirectory' "$ROOT_DIR/scripts/lib/publish.ps1" || fail 'faltan helpers de staging de release Windows'
[[ -f "$ROOT_DIR/windows/tests/release-e2e.ps1" ]] || fail 'falta la E2E nativa de release Windows'
windows_release_e2e_line="$(grep -nF "Invoke-Step 'Validando manifiesto, checksums y firma de release'" "$ROOT_DIR/scripts/build.ps1" | head -n1 | cut -d: -f1)"
windows_release_promote_line="$(grep -nF 'Publish-StagedDirectory $releaseStageDir $PublishDir' "$ROOT_DIR/scripts/build.ps1" | head -n1 | cut -d: -f1)"
[[ -n "$windows_release_e2e_line" && -n "$windows_release_promote_line" && "$windows_release_e2e_line" -lt "$windows_release_promote_line" ]] || fail 'builder Windows promueve release antes del E2E'
grep -Fq 'sha256' "$ROOT_DIR/distribution/ltools-release.schema.json" || fail 'esquema de release sin SHA-256'
grep -Fq 'ed25519' "$ROOT_DIR/distribution/ltools-project.json" || fail 'descriptor de proyecto sin firma Ed25519'
grep -Fq 'openssh-sshsig' "$ROOT_DIR/distribution/ltools-project.json" || fail 'descriptor de proyecto sin firma SSH'
grep -Fq 'SHA256SUMS.txt.sig' "$ROOT_DIR/distribution/ltools-project.schema.json" || fail 'esquema de proyecto sin firma separada'
if command -v jq >/dev/null 2>&1; then
    jq -e '.schema == "ltools-project-v1" and .repository == "Darkeiser003/Tools" and .platforms.linux and .platforms.windows' \
        "$ROOT_DIR/distribution/ltools-project.json" >/dev/null \
        || fail 'descriptor declarativo del proyecto inválido'
    jq -e '.verification.signature_supported == true and
        .verification.signature.algorithm == "ed25519" and
        .verification.signature.manifest == "SHA256SUMS.txt" and
        .verification.signature.detached_signature == "SHA256SUMS.txt.sig" and
        .verification.additional_signatures[0].algorithm == "openssh-sshsig" and
        .verification.additional_signatures[0].detached_signature == "SHA256SUMS.txt.sshsig"' \
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
grep -Fq 'x86_64-pc-windows-msvc' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no usa target MSVC'
grep -Fq 'build-state.json' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows sin estado incremental'
grep -Fq 'Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows detecta cambios solo por tamaño/fecha'
grep -Fq 'Test-BuildProfileChanged $oldState $BuildProfile' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no recompila al cambiar fast/release'
grep -Fq 'profile = $BuildProfile' "$ROOT_DIR/scripts/build.ps1" || fail 'estado incremental Windows no guarda el perfil usado'
grep -Fq '$StatePath = Join-Path $CargoReleaseDir ".build-state.json"' "$ROOT_DIR/scripts/build.ps1" || fail 'estado de perfil Windows no es compartido entre rutas de salida'
grep -Fq "[Environment]::SetEnvironmentVariable(\$name, \$previousCargoProfile[\$name], 'Process')" "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows deja variables de perfil rápido contaminando builds posteriores'
grep -Fq 'Assert-SafeOutputPath $OutputDir' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows permite borrar desde una ruta de salida raíz/peligrosa'
grep -Fq "Assert-SafeOutputPath \$TargetDir 'Cargo target Windows'" "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows permite compilar o limpiar a través de una junction del target'
grep -Fq 'Assert-DisjointOutputPaths $OutputDir $PublishDir' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows permite staging y publicación solapados'
grep -Fq 'no puede atravesar un enlace o punto de reanálisis' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no protege las rutas de salida contra junctions/symlinks'
grep -Fq "Assert-SafeOutputPath \$path 'Limpieza Windows'" "$ROOT_DIR/scripts/build.ps1" || fail 'limpiador Windows no comprueba los padres contra junctions/symlinks'
grep -Fq '$unsignedRelease))' "$ROOT_DIR/scripts/build.ps1" || fail 'limpiador Windows omite la publicación local sin firma'
grep -Fq 'Test-PowerShellSyntax' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no analiza los scripts PowerShell antes de compilar'
grep -Fq 'dist\windows-local-release' "$ROOT_DIR/scripts/build.ps1" || fail 'build local Windows conserva staging y publicación anidados'
grep -Fq 'Output y ReleaseOutput no pueden coincidir ni contenerse entre sí' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no explica rutas de salida solapadas'
grep -Fq "Target -notmatch '^(x86_64|aarch64|i686)-pc-windows-(msvc|gnu)$'" "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no valida arquitectura/target antes de compilar'
grep -Fq 'localArtifactPrefix = "ltools-$Version-windows-$PackageArch"' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows borra artefactos de otras arquitecturas/versiones del staging'
grep -Fq '"ltools-$Version-windows-$PackageArch*"' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows sustituye todos los perfiles Windows al publicar una arquitectura'
grep -Fq 'validate_output_paths' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux no valida rutas de salida antes de crear/borrar artefactos'
if preflight_output="$(TMPDIR="$ROOT_DIR/dist/contracts-tmp" bash "$ROOT_DIR/scripts/build.sh" --non-interactive --no-package --no-appimage --no-windows-wine --output "$ROOT_DIR/dist/contracts-output" --release-dir "$ROOT_DIR/dist/contracts-release" 2>&1)"; then
    fail 'builder Linux aceptó TMPDIR dentro del repositorio'
fi
grep -Fq 'TMPDIR no puede estar dentro del repositorio' <<<"$preflight_output" || fail 'builder Linux no explica por qué TMPDIR debe quedar fuera del repositorio'
grep -Fq 'project_cleanup_path_is_safe' "$ROOT_DIR/scripts/build.sh" || fail 'limpiador Linux no rechaza enlaces simbólicos en padres de destinos'
grep -Fq 'project_path_has_symlink_ancestor' "$ROOT_DIR/scripts/build.sh" || fail 'cargo clean del menú Linux no valida enlaces en directorios padre'
grep -Fq 'staging y publicación no pueden coincidir ni contenerse' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux permite staging y publicación solapados'
grep -Fq 'carpeta temporal compartida' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux permite usar /tmp compartido como destino de limpieza'
grep -Fq 'PACKAGE_NAME="ltools-$VERSION-linux-$ARCH"' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux no delimita la limpieza por versión/arquitectura'
grep -Fq 'LOCAL_PUBLISH_DIR="$STAGING/local-publish"' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux no prepara artefactos locales en staging'
if grep -Fq -- '-name "$PACKAGE_NAME.tar.gz" -o' "$ROOT_DIR/scripts/build.sh"; then
    fail 'builder Linux borra artefactos anteriores antes de validar sus reemplazos'
fi
grep -Fq 'run_logged "$ROOT_DIR/tests/linux/tarball-e2e.sh"' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux no prueba el tarball antes de publicarlo'
grep -Fq 'run_logged mv -f -- "$staged_file" "$OUTPUT_DIR/${staged_file##*/}"' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux no promueve artefactos locales desde staging'
grep -Fq 'CARGO_TARGET_DIR' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no fija el directorio de target'
grep -Fq 'LTOOLS_WINDOWS_CARGO_TARGET_DIR' "$ROOT_DIR/scripts/build.sh" || fail 'builder Linux no aísla el target Windows bajo Wine'
grep -Fq 'WINDOWS_CARGO_TARGET_DIR="${LTOOLS_WINDOWS_CARGO_TARGET_DIR:-' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'builder Wine no tiene target Windows aislado'
grep -Fq 'WINEXE="$CARGO_TARGET_DIR/$TARGET/release/ltools.exe"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'builder Wine no usa su target aislado para localizar el ejecutable'
grep -Fq 'windows\tests\e2e.ps1' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no ejecuta la E2E nativa'
grep -Fq "lib\\publish.ps1" "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows sin publicación segura de artefactos'
grep -Fq 'Publish-StagedDirectory $portableStage $portable' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no valida el paquete antes de promover la carpeta portable'
grep -Fq "ltools-capabilities.json') -Destination (Join-Path \$packageStageDir" "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no prepara el descriptor de capacidades para la release plana'
grep -Fq "ltools-terminal.json') -Destination (Join-Path \$packageStageDir" "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no prepara el descriptor terminal para la release plana'
grep -Fq 'tests\build-publish.ps1' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no prueba sus utilidades de publicación'
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
grep -Fq 'ltools-cli.exe' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no empaqueta el perfil CLI'
grep -Fq 'Compress-Archive' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no genera el ZIP portable'
grep -Fq 'exe-cli' "$ROOT_DIR/rust/src/release.rs" || fail 'manifiesto sin tipo de ejecutable CLI Windows'
grep -Fq 'portable-zip' "$ROOT_DIR/rust/src/release.rs" || fail 'manifiesto sin tipo de ZIP portable Windows'
grep -Fq 'PackageArch' "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows no adapta el nombre a la arquitectura'
grep -Fq "ErrorActionPreference = 'Continue'" "$ROOT_DIR/scripts/build.ps1" || fail 'builder Windows trata stderr normal de Cargo como fallo'
grep -Fq 'dist\windows' "$ROOT_DIR/windows/ltools.ps1" || fail 'lanzador PowerShell Windows no busca el output del builder'
grep -Fq "Join-Path \$root 'release'" "$ROOT_DIR/windows/ltools.ps1" || fail 'lanzador PowerShell Windows no busca la release común'
grep -Fq 'ltools.ps1' "$ROOT_DIR/windows/ltools.cmd" || fail 'lanzador CMD Windows no delega en el lanzador PowerShell'
grep -Fq 'ltools-cli.ps1' "$ROOT_DIR/windows/ltools-cli.cmd" || fail 'lanzador CLI CMD Windows no delega en PowerShell'
grep -Fq "Join-Path \$root 'release'" "$ROOT_DIR/windows/ltools-cli.ps1" || fail 'lanzador CLI Windows no busca la release común'
grep -Fq 'pause' "$ROOT_DIR/windows/build.cmd" || fail 'build.cmd no conserva el error visible al abrirse desde Explorer'
grep -Fq 'UMU-Latest' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'smoke Wine no prioriza UMU-Wine'
grep -Fq -- '--install-mono' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'smoke Wine no ofrece preparación opcional de Mono'
grep -Fq -- '--output DIR' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'helper Wine sin salida de artefactos'
grep -Fq 'zip es necesario para generar el paquete portable Windows' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'helper Wine no valida la herramienta ZIP'
grep -Fq 'unzip es necesario para validar el paquete portable Windows' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'helper Wine no valida la herramienta de extracción del ZIP'
grep -Fq 'ltools-$VERSION-windows-$package_arch.zip' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'helper Wine no genera el ZIP portable'
grep -Fq 'ZIP portable probado: integridad, contenido y perfiles GUI/CLI extraídos bajo Wine' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'E2E Wine no prueba los binarios extraídos del ZIP portable'
grep -Fq 'CREATED_TEMP_PREFIX" -eq 1' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'limpieza Wine puede borrar un prefijo explícito del usuario'
grep -Fq 'ARTIFACT_STAGING="$(mktemp -d "$ARTIFACT_DIR/.windows-package.XXXXXX")"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'paquetes Wine sobrescriben salidas antes de validar el ZIP'
grep -Fq 'LOG_PATH="$log_directory/windows-wine-$$.log"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'log Wine por defecto queda dentro del prefijo temporal que se elimina'
grep -Fq 'TMPDIR=$OUTPUT_DIR' "$ROOT_DIR/scripts/build.sh" || fail 'build integrada coloca el prefijo temporal Wine en tmpfs por defecto'
grep -Fq 'TEMP_ROOT="$(realpath -m -- "${TMPDIR:-/tmp}")"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'prefijos Wine con TMPDIR relativo pueden no resolverse como rutas absolutas'
grep -Fq 'mktemp -d "$TEMP_ROOT/ltools-windows-wine.XXXXXX"' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'el prefijo temporal Wine no usa un directorio absoluto'
grep -Fq -- '--no-tests' "$ROOT_DIR/tests/linux/windows-wine.sh" || fail 'helper Wine sin modo build-only'
grep -Fq 'run_with_sudo(program' "$ROOT_DIR/rust/src/packages.rs" || fail 'limpieza no respeta el gestor seleccionado'
grep -Fq 'manager_requires_process_elevation' "$ROOT_DIR/rust/src/packages.rs" || fail 'limpieza no separa gestores del sistema y de usuario'
grep -Fq 'tool_install_uses_privilege_wrapper' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'Pamac/AUR no conservan su autorización/compilación como usuario'
grep -Fq 'gui_install_flags("pamac").is_empty()' "$ROOT_DIR/rust/src/platform/linux.rs" || fail 'instalador Pamac recibe una opción de confirmación de Pacman no compatible'
grep -Fq 'pamac clean --keep 3' "$ROOT_DIR/tests/linux/software-git-e2e.sh" || fail 'limpieza de paquetes sin E2E de caché Pamac conservadora'
grep -Fq 'cascade_removal_is_pacman_only' "$ROOT_DIR/rust/src/packages.rs" || fail '--cascade carece de restricción comprobada al gestor Pacman'
grep -Fq 'package_removal_never_guesses_between_multiple_managers' "$ROOT_DIR/rust/src/packages.rs" || fail 'paquetes no prueban la selección explícita entre gestores'
grep -Fq 'flatpak_scope_for_package' "$ROOT_DIR/rust/src/packages.rs" || fail 'desinstalación Flatpak no conserva ni valida el ámbito de instalación'
grep -Fq 'resolve_flatpak_scope(None, &both)' "$ROOT_DIR/rust/src/packages.rs" || fail 'tests no rechazan Flatpak duplicado sin ámbito explícito'
grep -Fq 'parse_flatpak_installations' "$ROOT_DIR/rust/src/packages.rs" || fail 'limpieza Flatpak no reconoce instalaciones personalizadas declaradas'
grep -Fq -- '--installation={scope}' "$ROOT_DIR/rust/src/packages.rs" || fail 'operaciones Flatpak no seleccionan instalación personalizada'
grep -Fq 'LTOOLS_FLATPAK_USER_APP' "$ROOT_DIR/tests/linux/software-git-e2e.sh" || fail 'E2E no cubre la selección de instalación Flatpak'
grep -Fq 'desinstalación Brew y Flatpak conserva identidad' "$ROOT_DIR/tests/linux/software-git-e2e.sh" || fail 'E2E no comprueba managers de paquetes de usuario'
grep -Fq 'Pamac conserva su ámbito de usuario' "$ROOT_DIR/tests/linux/software-git-e2e.sh" || fail 'E2E no comprueba que Pamac no se relance como root'
grep -Fq 'la limpieza ejecutó una desinstalación con gestor ambiguo' "$ROOT_DIR/tests/linux/software-git-e2e.sh" || fail 'E2E permite que limpieza adivine el gestor'
grep -Fq 'flatpak uninstall --unused --user' "$ROOT_DIR/tests/linux/software-git-e2e.sh" || fail 'E2E no comprueba la limpieza Flatpak user'
grep -Fq 'flatpak uninstall --unused --system' "$ROOT_DIR/tests/linux/software-git-e2e.sh" || fail 'E2E no comprueba la limpieza Flatpak system'
grep -Fq 'WINDOWS_WINE" -eq 0' "$ROOT_DIR/scripts/build.sh" || fail 'el builder Linux repite cargo check antes de su build Windows/Wine'
grep -Fq 'LC_ALL", "C' "$ROOT_DIR/rust/src/packages.rs" || fail 'consultas de paquetes no fijan locale estable'
grep -Fq 'mod updater;' "$ROOT_DIR/rust/src/main.rs" || fail 'el backend no integra el módulo de actualización'
grep -Fq 'verified-update-download' "$ROOT_DIR/rust/src/compat.rs" || fail 'el contrato JSON no anuncia la descarga verificada'
grep -Fq 'automatic_install": false' "$ROOT_DIR/rust/src/compat.rs" || fail 'el contrato promete o no niega la sustitución automática'
grep -Fq 'Comprobar y descargar actualizaciones' "$ROOT_DIR/README.md" || fail 'el README no explica la función del actualizador'
grep -Fq 'local_artifact_download_checks_size_and_writes_only_to_reserved_path' "$ROOT_DIR/rust/src/updater.rs" || fail 'el actualizador no prueba una descarga local sin sobrescritura'
grep -Fq 'errores de actualizador GUI visibles antes de cerrar la consola' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'el smoke no detecta que los errores GUI del actualizador desaparezcan al cerrar'
grep -Fq 'pausedUpdateFailure' "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'la E2E Windows no verifica el error de la consola de actualizaciones pausada'
grep -Fq 'if [[ -n "$SIGNATURE_PUBLIC_KEY_FILE" ]]; then' "$ROOT_DIR/tests/release-e2e.sh" || fail 'la E2E de firma puede pasar una ruta vacía cuando solo se configura la clave en entorno'
grep -Fq 'SIGNING_PUBLIC_KEY_ENV_ACTIVE=1' "$ROOT_DIR/scripts/build.sh" || fail 'el builder Linux no fija precedencia consistente para la clave de entorno'
grep -Fq 'ltools_effective_update_public_key' "$ROOT_DIR/scripts/build.sh" || fail 'el builder Linux no usa el selector de clave pública con fallback en blanco'
[[ -x "$ROOT_DIR/tests/update-signing.sh" ]] || fail 'falta la prueba de precedencia y fallback de claves públicas Linux'
grep -Fq 'if [[ "$SIGNING_PUBLIC_KEY_ENV_ACTIVE" -eq 0 && -r "$SIGNING_PUBLIC_KEY_FILE" ]]; then' "$ROOT_DIR/scripts/build.sh" || fail 'el builder Linux puede firmar con una clave distinta de la incrustada'
grep -Fq '$script:PublicKeyEnvironmentActive = $null -ne $publicKeyEnvironment' "$ROOT_DIR/scripts/build.ps1" || fail 'el builder Windows no fija precedencia consistente para la clave de entorno'
grep -Fq 'if (-not $PublicKeyEnvironmentActive -and' "$ROOT_DIR/scripts/build.ps1" || fail 'el builder Windows puede firmar con una clave distinta de la incrustada'
grep -Fq 'windows_settings_exposes_both_update_actions' "$ROOT_DIR/rust/src/gui.rs" || fail 'las acciones del actualizador Win32 no tienen test de menú'
grep -Fq "Run @('update', '--help')" "$ROOT_DIR/windows/tests/e2e.ps1" || fail 'la E2E Windows no ejecuta la ayuda del actualizador'
grep -Fq 'update --help' "$ROOT_DIR/tests/linux/smoke.sh" || fail 'el smoke Linux no ejecuta la ayuda del actualizador'
if rg -n '(^|[[:space:]])wine([[:space:]]|$).*ltools|wine\.exe.*ltools|wine[[:space:]]+"' \
    "$ROOT_DIR/scripts/build.sh" "$ROOT_DIR/tests" "$ROOT_DIR/windows" "$ROOT_DIR/appimage" >/tmp/ltools-wine-tests.txt 2>/dev/null; then
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
