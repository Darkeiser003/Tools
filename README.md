# LTools

Herramientas modulares para auditar, organizar y mantener un equipo Linux,
con especial atención a CachyOS/Arch, juegos, Wine, Proton y prefijos
distribuidos entre varios discos. LTools usa un backend Rust nativo; los
scripts Bash se limitan a lanzadores, builders y pruebas.
El usuario puede utilizar el menú o el mismo conjunto de comandos desde una
terminal, un AppImage o un paquete portable de Windows.

| | |
|---|---|
| Versión | La definida en `rust/Cargo.toml` |
| Plataformas | Linux x86_64 · Windows x86_64 portable |
| Runtime | Rust 2021 · Bash solo para lanzadores, build y tests |
| Distribución | AppImage terminal, AppImage CLI, tarball Linux y ZIP Windows |
| Licencia | MIT |
| Idiomas | Español, inglés, alemán, francés, portugués, italiano, catalán, neerlandés y polaco |
| Proyecto | [Darkeiser003/Tools](https://github.com/Darkeiser003/Tools) |

LTools no es una herramienta de borrado ciego. Sus operaciones de limpieza y
migración comprueban rutas críticas, dependencias, espacio disponible,
bloqueos y contenido. Las operaciones modificadoras piden confirmación,
admiten `--dry-run` y generan planes reversibles cuando corresponde.

## Índice

- [Qué hace](#qué-hace)
- [Requisitos](#requisitos)
- [Instalación y primer uso](#instalación-y-primer-uso)
- [Comandos](#comandos)
- [Auditoría de Wine, Proton y juegos](#auditoría-de-wine-proton-y-juegos)
- [Migración de prefijos](#migración-de-prefijos)
- [Paquetes y limpieza](#paquetes-y-limpieza)
- [Salud y gestión del sistema](#salud-y-gestión-del-sistema)
- [Build y distribución](#build-y-distribución)
- [Descarga desde GitHub y manifiesto de release](#descarga-desde-github-y-manifiesto-de-release)
- [Integración JSON y terminal](#integración-json-y-terminal)
- [Arquitectura](#arquitectura)
- [Idiomas](#idiomas)
- [Logs, planes y rollback](#logs-planes-y-rollback)
- [Pruebas](#pruebas)
- [Seguridad y límites](#seguridad-y-límites)

## Qué hace

- Audita discos, directorios, archivos grandes, duplicados, caches, basura,
  instaladores, AppImages, máquinas virtuales y artefactos de compilación.
- Localiza juegos y prefijos Wine/Proton en Home, Steam, Lutris, Heroic, UMU,
  Bottles y bibliotecas montadas, evitando confundir un punto de montaje con
  un prefijo.
- Inspecciona cada prefijo: tamaño, arquitectura, versión Windows, runners,
  bloqueos, ejecutables, MSI, programas instalados y contenido relevante.
- Muestra las rutas efectivas de Wine, `wineboot`, winetricks, Proton, Steam,
  Heroic, Lutris y UMU, además de las variables activas.
- Inventaría gestores y formatos de paquetes del sistema y del usuario para
  detectar el backend nativo disponible: pacman/AUR, dpkg/apt, rpm/dnf/yum/
  zypper, apk, XBPS, pkg, Homebrew, Flatpak, Snap y Pamac. No es una tienda ni
  mantiene un catálogo de software de terceros.
- Limpia huérfanos, caches y rutas seleccionadas sin ejecutarlas si el usuario
  no confirma la operación.
- Gestiona servicios, daemons, procesos y journal mediante `systemctl` en
  Linux, y usa PowerShell, `sc.exe`, `tasklist`, `taskkill` y `wevtutil` en
  Windows, con diagnósticos que distinguen capacidades no disponibles.
- Detecta un catálogo amplio de herramientas nativas del anfitrión, agrupadas
  por almacenamiento, hardware, red, paquetes, servicios, procesos, archivos
  y escritorio. Wine, juegos, virtualización y desarrollo pertenecen a sus
  módulos propios o a la terminal, no a este catálogo. El catálogo se publica
  también en `capabilities --format json` para frontends.
- Genera AppImage con fallback de extracción si FUSE no está disponible y un
  ZIP portable nativo para Windows.

## Requisitos

### Para utilizarlo

En Linux, el AppImage lleva el backend de LTools. Las funciones dependen de
las herramientas que existan en el equipo; `doctor` las enumera y explica las
limitaciones. LTools no instala paquetes durante una auditoría ni actúa como
tienda. Si una operación concreta necesita una herramienta básica que falta,
el módulo puede mostrar una propuesta explícita tras comprobar alternativas;
la instalación siempre requiere confirmación del usuario.

El diagnóstico enumera solo dependencias que LTools puede utilizar en una
acción automatizada: `findmnt`, `sha256sum`, `rsync`, `df`, `wineboot`, `jq`,
`perl`, `rg`, la papelera nativa, `paccache`, `flatpak`, `systemctl`,
`journalctl`, `ps` y `kill`. No es un inventario general de herramientas del
sistema. Los gestores de paquetes se consultan dentro del módulo de paquetes
como backends nativos y no se ofrecen como una tienda de LTools.
En Windows se limita a PowerShell, `sc.exe`, `tasklist`, `taskkill`,
`wevtutil`, `Get-CimInstance`, la papelera nativa y `jq`, que son las
dependencias que las acciones automatizadas realmente pueden ejecutar. Usa
UAC únicamente para las acciones que lo necesitan.

### Política de herramientas del anfitrión

LTools prioriza siempre esta secuencia:

1. Usar una herramienta nativa que ya esté instalada.
2. Usar una alternativa equivalente que ya exista, aunque tenga menos
   funciones.
3. Explicar qué función queda limitada y por qué.
4. Proponer una única herramienta básica y oficial solo si es imprescindible,
   mostrando el gestor y el comando antes de instalarla.
5. Pedir confirmación explícita; nunca instalar en segundo plano.

La instalación es contextual. Por ejemplo, una migración que necesite
`rsync`, una reescritura de configuración que necesite `jq`, o una limpieza
que necesite `rg` muestran primero la dependencia, el paquete y el comando
del gestor nativo disponible. Solo después de una confirmación explícita se
intenta instalar esa única dependencia. Si no hay un gestor oficial o la
herramienta es propia de la plataforma, se informa y la acción se cancela de
forma segura.

Los gestores de paquetes se usan exclusivamente como mecanismo nativo para
consultar inventarios y ejecutar una limpieza que el usuario haya pedido, o
para una dependencia básica concreta previamente justificada. LTools no
incluye una tienda, no recomienda listas de aplicaciones de terceros y no
instala paquetes opcionales por iniciativa propia. Wine, juegos,
virtualización y herramientas de desarrollo no forman parte del catálogo
general; Wine solo se ofrece como dependencia contextual de la creación de un
prefijo.

### Para compilar

- Rust y Cargo, con el proyecto en edición 2021.
- Bash, `tar` y las utilidades habituales del sistema.
- `appimagetool` para generar AppImage.
- `appstreamcli` se usa para validar los metadatos de escritorio cuando está
  instalado.
- En Windows nativo: PowerShell 5.1+, Rust mediante rustup, MSVC/Visual
  Studio Build Tools y Windows SDK.
- Para compilación cruzada Windows desde Linux: target GNU de Rust y MinGW;
  la ejecución y las pruebas Windows se reservan a Windows nativo.

## Instalación y primer uso

### AppImage Linux

```bash
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' rust/Cargo.toml | head -n1)"
chmod +x "ltools-$VERSION-linux-x86_64.AppImage"
"./ltools-$VERSION-linux-x86_64.AppImage"
```

Si el AppImage se distribuye junto al lanzador auxiliar, este detecta FUSE y
activa automáticamente la extracción temporal cuando haga falta:

```bash
./run-ltools.sh
./run-ltools.sh --doctor
./run-ltools.sh games --full
```

El lanzador auxiliar es opcional y no forma parte del tarball runtime.

### Tarball Linux

Descomprime el tarball conservando su estructura y ejecuta `./ltools.sh`. El
paquete runtime contiene únicamente la fachada, el backend Rust, documentación
y tests; no incluye builders ni código de otra plataforma.

### Windows portable

Descomprime el ZIP y ejecuta `ltools.exe`, `ltools.cmd` o:

```powershell
.\ltools.ps1
.\ltools.ps1 doctor
```

La release Windows no incluye scripts Bash, FUSE, Wine, Proton ni comandos
Linux. Las capacidades no aplicables se muestran como tales.

## Comandos

Sin argumentos se abre el menú interactivo:

```bash
./ltools.sh
```

Rust es el backend normal y único. `--rust` se conserva como opción compatible
del lanzador, pero ya no selecciona una implementación alternativa:

```bash
./ltools.sh --rust audit --full --duplicates --min-size-mb 100
./ltools.sh games --full --root /mnt/JuegosLinux
./ltools.sh packages
./ltools.sh defaults
./ltools.sh doctor
./ltools.sh doctor --install rsync
./ltools.sh system status
./ltools.sh system services
./ltools.sh system processes
./ltools.sh rollback --plan /tmp/ltools-plan.tsv
./ltools.sh capabilities --format json
```

| Comando | Función |
|---|---|
| `audit` | Discos, aplicaciones, archivos grandes y duplicados |
| `games` | Juegos, runners, Wine, Proton, Steam, Lutris, Heroic y UMU |
| `packages` | Gestores, paquetes instalados, AUR y archivos descargados |
| `clean` | Limpieza protegida de paquetes, caches y rutas |
| `prefix` | Listar, inspeccionar, crear y migrar prefijos |
| `defaults` | Rutas efectivas y defaults de las herramientas |
| `system` | Servicios, procesos, daemons y journal |
| `doctor` | Dependencias, FUSE y diagnóstico del anfitrión |
| `rollback` | Recuperar operaciones registradas en un plan |

Cada módulo ofrece ayuda propia:

```bash
./ltools.sh audit --help
./ltools.sh games --help
./ltools.sh packages --help
./ltools.sh clean --help
./ltools.sh prefix --help
./ltools.sh system --help
./ltools.sh --lang en --rust --help
```

## Integración JSON y terminal

LTools puede describir sus capacidades con un contrato estable y legible por
máquinas:

```bash
./ltools.sh capabilities --format json > ltools-capabilities.json
```

El descriptor se incluye también en el tarball Linux, el AppImage y el ZIP
portable Windows junto con `ltools-capabilities.schema.json`. Un frontend puede
usar `entrypoints.menu` para abrir el menú y `terminal_integration` para saber
qué protocolo necesita la terminal anfitriona.

Para una integración directa basta con distribuir también
`ltools-terminal.json`; es la versión reducida del contrato destinada única y
exclusivamente a la integración con la familia LTerminal. Es opcional: no se
lee ni se necesita para ejecutar el AppImage, el AppImage CLI, el tarball ni el
`.exe` portable. `ltools-terminal.schema.json` permite validar el descriptor
antes de instalarlo.

El mismo contrato sirve para las dos variantes del host: el descriptor Linux
identifica `LTerminal` y usa `ltools`; el descriptor generado por el builder
Windows identifica `WinSlim Terminal` y usa `ltools.exe`. Esto prepara la
compatibilidad cruzada sin mezclar los lanzamientos autónomos con la
integración del host.

LTools no puede invocar una API nativa de una terminal que no la exponga. Para
integrarse directamente con LTerminal, esta debe implementar la consulta
`--ltools-capabilities --format json` y la apertura con
`--open-path RUTA --command COMANDO -- menu`, declarando `lterminal-startup-v1`.
AppRun lo detecta y registra el diagnóstico. Si LTerminal está instalado pero
no ofrece todavía ese protocolo, LTools no cambia silenciosamente a Konsole:
informa del motivo y termina para evitar abrir la aplicación en la terminal
equivocada. El fallback puede autorizarse conscientemente con
`LTOOLS_ALLOW_TERMINAL_FALLBACK=1`, o puede seleccionarse una terminal concreta
con `LTOOLS_TERMINAL=konsole`. `LTOOLS_TERMINAL=lterminal` exige LTerminal y
nunca usa otra terminal. LTools no modifica LTerminal.

## Salud y gestión del sistema

El módulo `system` separa diagnóstico de acciones. No considera un servicio
`not-found` como fallo real, explica los `inactive/dead` normales de servicios
`oneshot`, identifica `active/exited` como tareas terminadas y avisa cuando un
servicio está `masked` (por ejemplo `power-profiles-daemon`, que no se cambia
automáticamente).

```bash
./ltools.sh system status
./ltools.sh system failed --journal
./ltools.sh system services --scope both --filter noteworthy
./ltools.sh system services --filter all --category docker --search container
./ltools.sh system processes --sort memory --limit 20
./ltools.sh system journal --level error --hours 6 --limit 100
./ltools.sh system dependencies --unit docker.service --reverse
./ltools.sh system export --scope both --format json --out /tmp/ltools-system.json
```

Los servicios se muestran en columnas completas, sin truncar descripciones, y
se agrupan por Steam, KDE, Docker, VMware, Wine, red, audio, sesión y sistema.
Las acciones disponibles son `start`, `stop`, `restart`, `enable`, `disable`,
`mask` y `unmask`:

```bash
./ltools.sh --dry-run --plan /tmp/service-plan.tsv \
  system service restart docker.service
```

Siempre se pide confirmación. Las acciones del sistema requieren `sudo` cuando
corresponde; las consultas no modifican nada. En el menú se encuentran en
«Gestionar servicios, procesos y journal», junto con filtros interactivos,
dependencias y exportación TSV/JSON.

El AppImage principal es autónomo. Al abrirlo sin argumentos, desde el gestor de
archivos o desde otra terminal, busca un emulador de terminal del sistema, abre
una ventana nueva y ejecuta el menú Rust con `LTOOLS_SHELL`, `$SHELL` cuando es
compatible, o Bash/sh como fallback. No necesita LTerminal y no se bloquea porque LTerminal esté
ausente, desactualizado o sea incompatible. `LTOOLS_TERMINAL` permite escoger un
emulador concreto; `LTOOLS_TERMINAL=lterminal` activa deliberadamente la
integración externa.

## Auditoría de Wine, Proton y juegos

La auditoría busca rutas habituales y también bibliotecas indicadas por
configuraciones de Steam, Heroic, Lutris y UMU. Clasifica los resultados por
origen y evita contar como prefijo independiente:

- puntos de montaje completos;
- `default_pfx` que pertenece a un runner;
- directorios ya contenidos dentro de otro prefijo detectado;
- rutas inexistentes o inaccesibles.

En modo completo revisa Home, `/opt`, `/usr/local/share`, caches, bibliotecas
Steam y las rutas montadas que se indiquen con `--root`.

```bash
./ltools.sh games --full --root "$HOME" --root /mnt/JuegosLinux
./ltools.sh prefix list
./ltools.sh prefix list --include-mount-roots
./ltools.sh prefix inspect --path "$HOME/.wine"
./ltools.sh defaults
```

## Migración de prefijos

La migración mueve el contenido de un prefijo a un destino dado; no mete un
prefijo dentro de otro ni fusiona varios `drive_c`. Para varios orígenes crea
un destino independiente dentro de la carpeta central.

```bash
./ltools.sh prefix migrate \
  --source "$HOME/.wine" \
  --dest /mnt/JuegosLinux/prefixes/wine-main \
  --select \
  --rewrite-configs \
  --set-defaults \
  --update-launchers \
  --remove-source
```

Antes de copiar comprueba que el destino no sea peligroso, que haya espacio y
que no existan bloqueos activos. Después compara el contenido. Solo tras la
verificación y la confirmación del usuario ofrece retirar el origen a la
papelera. Si una referencia de Lutris, Heroic, UMU o Steam no se puede
actualizar automáticamente, se informa de la aplicación, el fichero y la ruta
que debe revisarse.

`--set-defaults` configura el `WINEPREFIX` persistente y hace backup. También
puede activarse en la shell con `--activate-shell`. Steam/Proton no tiene un
default global único seguro: Steam gestiona un `compatdata` por AppID.

## Paquetes y limpieza

`packages` sondea los gestores disponibles en lugar de recorrer el disco sin
criterio. Separa paquetes del sistema, externos/AUR, Flatpak, Snap y archivos
descargados. La eliminación usa el gestor que el usuario seleccione y no está
fijada accidentalmente a pacman. Antes de retirar un paquete se comprueban
dependencias y, si existen, se ofrece cancelar o resolverlas mediante el
gestor correspondiente.

Ejemplos seguros:

```bash
./ltools.sh packages --out "$HOME/Informes/ltools-packages"
./ltools.sh clean --dry-run --package-caches --plan /tmp/ltools-clean.tsv
./ltools.sh clean --dry-run --path "$HOME/.cache/paru" --plan /tmp/ltools-cache.tsv
./ltools.sh rollback --plan /tmp/ltools-cache.tsv
```

El modo de limpieza no incluye automáticamente bibliotecas de juegos, máquinas
virtuales, prefijos ni puntos de montaje. Esas rutas requieren selección
explícita y mantienen los bloqueos de seguridad.

## Build y distribución

Build Linux completa:

```bash
./build.sh --non-interactive --fast
```

La build Linux ejecuta rustfmt, Clippy, tests Rust, sintaxis Bash, contratos,
compilación release, tarball, AppImage, smoke, E2E de migración/rollback y E2E
de menús y funciones. También valida AppStream, FUSE, idiomas, gestores de
paquetes, duplicados y las rutas efectivas del ecosistema Wine. No ejecuta
binarios Windows mediante Wine.

La salida Linux ofrece tres entregables de uso:

- `ltools-VERSION-linux-ARQUITECTURA.AppImage`: AppImage autónomo; abre su
  propia ventana de terminal al ejecutarse sin argumentos.
- `ltools-VERSION-linux-ARQUITECTURA-cli.AppImage`: perfil CLI; usa siempre la
  terminal desde la que se invoca y nunca abre otra ventana.
- `ltools-VERSION-linux-ARQUITECTURA.tar.gz`: paquete runtime para integrarlo
  manualmente. En Windows, `ltools.exe` es el ejecutable nativo dentro del ZIP
  portable. El builder Windows deja también el `.exe` suelto para lanzarlo
  desde el explorador y abrir su consola propia.

Cada paquete incluye `ltools-capabilities.json`. Además, `ltools-terminal.json`
es un tercer entregable lógico, exclusivo para integraciones como LTerminal:
contiene el protocolo, los argumentos de apertura y la capacidad que debe
anunciar la terminal. No es necesario para ejecutar el AppImage ni el `.exe`.

Opciones frecuentes:

```bash
./build.sh --clean --output /tmp/ltools-dist
./build.sh --appimage --no-package
./build.sh --non-interactive --no-smoke --no-e2e
./build.sh --appimage --require-fuse
```

El builder Windows está en `windows/build.ps1` y usa MSVC por defecto:

### Descarga desde GitHub y manifiesto de release

El descriptor declarativo [`distribution/ltools-project.json`](distribution/ltools-project.json)
es la entrada recomendada para una sección de Proyectos de una terminal. Apunta
a la página de releases de GitHub y describe los artefactos disponibles para
Linux y Windows, sin instalar nada ni convertir LTools en un plugin obligatorio.

Cada build que genere artefactos produce también `ltools-release.json`. Este
manifiesto contiene únicamente archivos encontrados en las carpetas indicadas,
con su plataforma, arquitectura, tipo, tamaño, URL directa de GitHub y hash
SHA-256. El generador es Rust y no depende de scripts Bash para calcular ni
verificar los datos.

La URL estable para la terminal será:

```text
https://raw.githubusercontent.com/Darkeiser003/Tools/main/distribution/ltools-project.json
```

Después de construir Linux y Windows, el manifiesto unificado se puede regenerar
desde cualquier binario release de LTools:

```bash
VERSION="$(sed -n 's/^version = "\([^" ]*\)"/\1/p' rust/Cargo.toml | head -n1)"
./ltools.sh release-manifest \
  --output dist/ltools-release.json \
  --repository Darkeiser003/Tools \
  --tag "v$VERSION" \
  --artifacts-dir dist \
  --artifacts-dir dist/windows
```

Sube a la release de GitHub los artefactos y `dist/ltools-release.json` con el
nombre exacto `ltools-release.json`. La terminal puede leer primero el descriptor
del proyecto, seleccionar el artefacto apropiado para el sistema y verificar el
SHA-256 antes de ofrecer la instalación. El descriptor de integración de
terminal sigue siendo opcional y separado.

Para cambiar de repositorio o de etiqueta sin editar archivos, usa
`LTOOLS_GITHUB_REPOSITORY` y `LTOOLS_GITHUB_TAG` al ejecutar los builders.

```powershell
.\windows\build.ps1
.\windows\build.ps1 -Fast
.\windows\build.ps1 -Force -NoRun
.\windows\build.ps1 -Target x86_64-pc-windows-gnu
```

Mantiene su estado incremental en `dist/windows/.build-state.json`, separa el
target en `rust/target/windows` y genera un ZIP portable por arquitectura.

## Arquitectura

El proyecto mantiene dos capas compatibles:

```text
.
├── ltools.sh                 Lanzador compatible del backend Rust
├── rust/
│   ├── Cargo.toml            Backend nativo y metadata de la aplicación
│   └── src/                  Núcleo compartido y adaptadores por plataforma
│       └── platform/         Implementaciones Linux y Windows aisladas
├── platform/
│   └── linux/
│       └── build.sh          Builder Linux/AppImage
├── appimage/                 AppRun, desktop, icono y metadata AppStream
├── windows/                  Builder, lanzadores y tests nativos Windows
├── tests/                    Contratos y pruebas
│   └── linux/                Smoke y E2E Linux/AppImage
└── build.sh                  Lanzador compatible del builder Linux
```

Rust es el único backend de LTools para Linux, AppImage y Windows. Los scripts
`.sh` restantes son lanzadores, builders o harnesses de prueba; no se invocan
como implementación funcional ni se empaquetan en el tarball/AppImage.

## Idiomas

El idioma se obtiene de `LTOOLS_LANG`, `LC_ALL`, `LC_MESSAGES` o `LANG` y se
puede forzar en una ejecución:

```bash
LTOOLS_LANG=en ./ltools.sh --help
./ltools.sh --lang de menu
./ltools.sh --lang fr defaults
```

Los códigos soportados son `es`, `en`, `de`, `fr`, `pt`, `it`, `ca`, `nl` y
`pl`. `auto` usa el locale del entorno y, si no hay traducción, se usa español.
Los tests comprueban normalización como `en_US.UTF-8` y `pt-BR`, además de todos
los catálogos disponibles.

## Logs, planes y rollback

Cada build guarda una transcripción y una tabla de tiempos:

```text
dist/build-AAAAMMDD-HHMMSS-PID.log
dist/build-AAAAMMDD-HHMMSS-PID-timings.tsv
dist/appimage-smoke.log
```

El log registra configuración, comandos, códigos de salida y duración. Las
operaciones modificadoras aceptan `--plan FICHERO`; el plan describe acciones
ejecutadas y permite lanzar `rollback --plan FICHERO`. `--dry-run` no crea
destinos ni modifica datos.

Los logs, informes, planes, targets, staging y artefactos de distribución son
locales y están excluidos por `.gitignore`. Para inspeccionar qué residuos
ignorados se eliminarían antes de limpiar el árbol de trabajo:

```bash
git clean -ndX -- dist reports tmp rust/target
```

Cuando la lista sea correcta, se pueden retirar únicamente esas salidas
regenerables con:

```bash
git clean -fdX -- dist reports tmp rust/target
```

`./build.sh --clean` limpia solamente los targets Rust y conserva los demás
resultados locales; el builder vuelve a crear todo lo necesario en la próxima
ejecución.

## Pruebas

Para ejecutar la batería completa:

```bash
./build.sh --non-interactive --fast
```

Pruebas individuales:

```bash
cargo test --manifest-path rust/Cargo.toml --all-targets
cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
./tests/contracts.sh
./tests/linux/smoke.sh --binary rust/target/release/ltools
./tests/linux/e2e.sh --binary rust/target/release/ltools
./tests/linux/menu-e2e.sh --binary rust/target/release/ltools
```

En Windows nativo:

```powershell
.\windows\build.ps1 -Force
```

El smoke y la E2E Windows están en `windows/tests/` y prueban el ejecutable,
informes, copia verificada, planes, listado y papelera nativa. Deben ejecutarse
en Windows; el pipeline Linux solo compila/verifica estáticamente el target
Windows y no invoca Wine.

## Seguridad y límites

- Nunca se debe ejecutar una limpieza o migración real sin revisar primero el
  informe y probar `--dry-run`.
- `/`, `/home`, `/mnt`, `/opt`, `/usr`, `/var`, `/etc`, `/boot`, `/run`, puntos
  de montaje, `steamapps`, `compatdata` y runners compartidos tienen bloqueo
  adicional o requieren selección explícita.
- La detección de configuraciones SQLite y binarias es conservadora: muestra
  rutas y avisos cuando no puede modificar un formato con seguridad.
- Heroic, Lutris, UMU y Steam se actualizan solo en formatos conocidos y con
  copia de seguridad. Las referencias restantes se reportan para revisión
  manual.
- La build no instala paquetes ni modifica el sistema. `doctor` es de solo
  lectura; `doctor --install TOOL` y las acciones que necesitan una dependencia
  solo muestran una instalación concreta, piden confirmación y la registran en
  el plan. Nunca existe una instalación masiva.
- Los artefactos generados, caches, informes, logs, targets y dependencias de
  frontend están excluidos por `.gitignore`; `Cargo.lock` sí se versiona. Antes
  de un commit se recomienda revisar `git status --short --ignored`.
