# LTools

Centro de acciones rápidas multiplataforma para LTerminal y WinSlim Terminal.
LTools usa un backend Rust nativo y ofrece una aplicación autónoma con menú,
un perfil CLI para automatización y paquetes portables para Linux y Windows.
Los scripts Bash y PowerShell se limitan a lanzadores, builders y pruebas; no
son el backend funcional.

La variante Linux se presenta como `LTools`; la variante Windows se presenta
como `WinSlim-Tools`. Los nombres técnicos `ltools`, `ltools.exe`, los IDs de
los contratos y las rutas de compatibilidad se conservan para no romper los
lanzadores ni las terminales anfitrionas.

| | |
|---|---|
| Versión | La definida en `rust/Cargo.toml` |
| Plataformas | Linux x86_64 · Windows x86_64 portable |
| Runtime | Rust 2021 · Bash/PowerShell solo para lanzadores, build y tests |
| Distribución | AppImage terminal, AppImage CLI, tarball Linux y ZIP Windows |
| Licencia | MIT |
| Idiomas | Los 15 identificadores de LTerminal, más selección automática desde el entorno del host |
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
- [Búsqueda e instalación contextual](#búsqueda-e-instalación-contextual)
- [Operaciones Git](#operaciones-git)
- [Salud y gestión del sistema](#salud-y-gestión-del-sistema)
- [Diagnóstico nativo](#diagnóstico-nativo)
- [Discos, particiones y configuración nativa](#discos-particiones-y-configuración-nativa)
- [Build y distribución](#build-y-distribución)
- [Descarga desde GitHub y manifiesto de release](#descarga-desde-github-y-manifiesto-de-release)
- [Integración JSON y terminal](#integración-json-y-terminal)
- [Contrato de integración con LTerminal](docs/lterminal-integration.md)
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
- Inspecciona discos, volúmenes y particiones con herramientas nativas: en
  Linux `lsblk`, `findmnt`, `df`, `parted` y, si existe, `gparted`; en Windows
  PowerShell (`Get-Disk`, `Get-Partition`, `Get-Volume`), `diskpart` y
  `mountvol`. El inventario es seguro y no ejecuta cambios destructivos de
  particiones.
- Inspecciona la configuración adecuada a cada sistema: rutas de configuración
  y journal en Linux, y consultas/exportaciones `.reg` mediante `reg.exe` en
  Windows. No mezcla Registro Windows con la configuración Linux.
- Detecta un catálogo amplio de herramientas nativas del anfitrión, agrupadas
  por almacenamiento, hardware, red, paquetes, servicios, procesos, archivos y
  escritorio, utilidades y desarrollo. Wine, juegos y virtualización siguen
  siendo módulos separados para no mezclar sus dependencias con el anfitrión.
  El catálogo se publica también en `capabilities --format json` para
  frontends.
- Genera AppImage con fallback de extracción si FUSE no está disponible y un
  ZIP portable nativo para Windows.

## Acciones guiadas y valores seguros

Las funciones operativas no se limitan a mostrar inventarios. Cada módulo
ofrece acciones concretas y reutilizables: consultar salud, inspeccionar,
comprobar, montar o desmontar con confirmación, abrir el gestor nativo,
gestionar servicios y procesos, revisar el journal, exportar informes,
limpiar mediante planes y ejecutar automatizaciones registradas.

La interfaz de botones y la CLI comparten un registro de acciones estable:

```text
ltools actions list
ltools actions list --format json
ltools --dry-run actions run storage.mount /dev/sdb1
ltools actions run system.service-status sshd.service
```

Cada acción publica su categoría, objetivo, perfil (`safe-default` o
`advanced`), si modifica el sistema, confirmación requerida y compatibilidad
con `--dry-run` y planes. Los botones de consulta usan valores seguros; las
acciones que requieren un objetivo nunca lo inventan. En Windows, `C:` está
excluido de la selección guiada y solo puede introducirse manualmente en una
acción avanzada. En Linux, `/`, `/boot` y `/home` no se usan como objetivos
predeterminados. El particionado destructivo no se ejecuta desde una cadena
oculta: se abre el gestor nativo para que el usuario revise cada paso.

Este contrato (`ltools-actions-v1`) permite que LTerminal, WinSlim Terminal,
la GUI y futuros scripts de WinSlim compartan botones y parámetros sin
duplicar comandos de shell. Las acciones se compilan por plataforma: Linux no
publica acciones Windows y Windows no publica acciones Linux.

En la GUI, las operaciones largas se ejecutan fuera del hilo visual. Mientras
una acción está activa se muestra un indicador animado y su estado en la salida,
se deshabilitan temporalmente los submenús para impedir acciones concurrentes y
se restauran al terminar, también cuando la herramienta devuelve un error. El
panel de salida sigue siendo desplazable y el separador entre controles y
resultados se puede redimensionar.

Los alias cortos se publican junto a cada acción para que el host pueda ofrecer
comandos cómodos como tdisk status, tsvc list, tnet status, tboot status o
tclean preview. Son metadatos declarativos, no shell: LTerminal debe usar
siempre executable y args[] separados.

### Arranque, firmware y recuperación

boot status inspecciona únicamente el arranque nativo de la plataforma: GRUB,
systemd-boot, EFI y Secure Boot en Linux; BCD, EFI y WinRE en Windows. boot
plan explica el flujo protegido para una futura modificación, pero no escribe
GRUB, BCD, NVRAM ni particiones. Cualquier aplicación futura deberá exigir
destino explícito, copia exportada, diff, elevación, verificación y rollback.
Wine no se considera una prueba válida de firmware o arranque.

## Requisitos

### Para utilizarlo

En Linux, el AppImage lleva el backend de LTools. Las funciones dependen de
las herramientas que existan en el equipo; `doctor` las enumera y explica las
limitaciones. LTools no instala paquetes durante una auditoría ni actúa como
tienda. Si una operación concreta necesita una herramienta básica que falta,
el módulo puede mostrar una propuesta explícita tras comprobar alternativas;
la instalación siempre requiere confirmación del usuario.

El diagnóstico enumera las dependencias que LTools puede utilizar en una
acción automatizada y las alternativas nativas relevantes: auditoría,
almacenamiento, servicios, procesos, configuración, contenedores, Kubernetes,
limpieza y gestores de paquetes. Cada entrada indica si está disponible, su
versión cuando se puede consultar y si LTools puede proponer instalarla. No es
un inventario indiscriminado de aplicaciones ni una tienda. En Windows solo
se muestran comandos, cmdlets y gestores propios de Windows; usa UAC
únicamente para las acciones que lo necesitan.

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
instala paquetes opcionales por iniciativa propia. Wine, juegos y
virtualización no forman parte del catálogo general; Wine solo se ofrece como
dependencia contextual de la creación de un prefijo. Las utilidades de
diagnóstico, automatización y desarrollo sí aparecen como opcionales cuando
son herramientas del sistema razonables y su instalación puede resolverse con
el gestor nativo.

El catálogo cubre herramientas básicas y avanzadas que las acciones de
mantenimiento pueden aprovechar. En Linux detecta almacenamiento y
recuperación (`lsblk`, `findmnt`, `parted`, `fdisk`, `sfdisk`, `sgdisk`,
`wipefs`, `blkdiscard`, `testdisk`, `photorec`, `ddrescue`, Btrfs, LVM, ZFS,
LUKS, XFS y NTFS), copias (`restic`, `borg`, `rclone`, `timeshift`, `snapper`),
Docker/Compose, Podman, Buildah, Skopeo, containerd, nerdctl, SSH/SCP/SFTP,
ADB y Kubernetes (`kubectl`, Helm, Kind, Minikube, k3d, k9s, Kustomize,
Helmfile y Argo CD). También ofrece red avanzada (`ethtool`, `iw`, `mtr`,
`iperf3`, `socat`, WireGuard, OpenVPN y Tailscale), hardware y rendimiento
(`inxi`, `lshw`, `hwinfo`, `dmidecode`, `sensors`, `powertop`, `iotop`, `nvtop`,
`nvidia-smi` y `radeontop`), seguridad de contenedores (`trivy`, `cosign`) y
desarrollo (`jq`, `yq`, `git-lfs`, `git-filter-repo`, LazyGit, Delta, GLab,
Python, Node, Go, Rustup, Java, Maven, Gradle, Make, CMake, GCC, GDB, Valgrind,
Perf y BPFTrace).

En Windows detecta PowerShell, DiskPart, Administración de discos, cmdlets de
discos/red/Defender, `reg.exe`, `sc.exe`, `tasklist`, `taskkill`, `wevtutil`,
Docker/Compose, Podman, OpenSSH, ADB y Kubernetes. También cataloga reparación
y recuperación (`chkdsk`, `sfc`, DISM, `fsutil`, `diskshadow`, BitLocker,
`cipher`, `vssadmin`, `wbadmin`, WinRE), drivers (`pnputil`, `driverquery`),
red (`netsh`, `arp`, `pathping`, `getmac`), tareas (`schtasks`, políticas de
grupo), permisos (`icacls`, `takeown`, `auditpol`) y runtimes de desarrollo
como Python, Node, Java y .NET.

Las herramientas instalables se ofrecen bajo demanda cuando la plataforma
conoce un paquete fiable. Las herramientas integradas de Windows y las que no
tienen un paquete universal se identifican como nativas o de instalación
manual. El JSON publica `available`, `installable`, `install_package` y
`version` para que una terminal pueda mostrar el estado sin ejecutar acciones
inesperadas.
El JSON publica `available`, `installable`, `install_package` y `version` para
que una terminal pueda mostrar el estado sin ejecutar acciones inesperadas.

### Codificación de archivos

El repositorio aplica una política por compatibilidad, no una conversión
uniforme:

- Rust, Bash, JSON, XML, SVG, Markdown, TOML y archivos `.desktop`: UTF-8 sin
  BOM. El BOM rompería especialmente los shebangs de Bash y algunos lectores
  JSON.
- PowerShell (`.ps1`): UTF-8 con BOM (`UTF-8-BOM`), necesario para que
  Windows PowerShell 5.1 reconozca correctamente textos como `á`, `ñ` y `¿`.
- CMD (`.cmd`): ASCII/ANSI seguro, sin BOM. Al contener únicamente caracteres
  ASCII, funciona con las páginas de código Windows habituales y no depende
  de que `cmd.exe` interprete UTF-8.
- UTF-16-LE no se usa para fuentes ni configuración de LTools: ningún
  consumidor actual lo requiere y aplicarlo globalmente dañaría Bash, Rust y
  JSON. Solo debe introducirse para un archivo externo que documente
  explícitamente esa exigencia.

La comprobación reproducible está en `tests/encoding.sh` y se ejecuta también
durante `./build.sh`. Valida UTF-8, presencia o ausencia de BOM, y que los CMD
no contengan bytes dependientes de una página de código.

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

### Interfaz gráfica y perfil CLI

El AppImage Linux normal y el `ltools.exe` Windows normal abren su propia
ventana gráfica nativa escrita en Rust. La interfaz se organiza como una
aplicación de terminal: barra superior compacta, rail lateral persistente de
secciones, área central para el submenú activo y panel inferior de salida
redimensionable. Así las acciones no se amontonan en una lista única y cada
módulo conserva su contexto. La ventana ofrece secciones generales y sus
submenús: auditoría/inventario, dependencias, herramientas nativas, herramientas
instalables y automatización. En Linux usa GTK3
del sistema y en Windows usa Win32.
Si el entorno Linux no tiene sesión gráfica o GTK, el AppImage conserva el
fallback controlado a una terminal externa.

Los artefactos con sufijo `-cli` no abren ninguna ventana: sin argumentos
muestran la ayuda y con argumentos ejecutan exclusivamente la acción solicitada.
El JSON de integración sigue siendo opcional y solo sirve para que LTerminal o
WinSlim Terminal lancen acciones declarativas; no es necesario para la GUI ni
para el funcionamiento autónomo de las releases.

La CLI y la GUI usan la misma navegación jerárquica para evitar un menú principal
saturado. Cada entrada abre la siguiente pantalla y no conserva un panel duplicado:

1. **Auditar / Inventariar**: discos y aplicaciones, juegos y lanzadores,
   paquetes y, en Linux, prefijos Wine/Proton.
2. **Dependencias**: detectar, consultar almacenes, buscar e instalar paquetes,
   revisar versiones, instalar herramientas faltantes y consultar dependencias de
   servicios. Esta es la única sección que ofrece instalación.
3. **Herramientas nativas**: discos, particiones, montajes, servicios, procesos,
   usuarios, red, hardware, energía, seguridad, arranque, rutas y configuración.
4. **Herramientas instalables**: Git/GitHub, SSH/SCP/SFTP, ADB, Docker/Podman/Compose
   y Kubernetes, cada uno con su propio flujo operativo.
5. **Automatización**: registrar, listar, ejecutar y retirar automatizaciones,
   además de acciones guiadas.
6. **Ajustes**: idioma, tema, modo de color y visibilidad de secciones.

Los comandos antiguos (`menu-storage`, `menu-services`, `menu-import`, etc.) siguen
aceptándose como compatibilidad de terminal, pero ya no aparecen como botones ni
categorías duplicadas en la portada.

En Windows aparece una categoría **WinSlim** después de Ajustes únicamente si existe
`C:\WSCore`. La sección informa también si encuentra `NSudoLC.exe`, `NSudoLG.exe`
o `NSudo.exe` dentro de esa raíz (o en el `PATH`). La elevación normal continúa
usando UAC; NSudo no se activa automáticamente. Solo una integración WinSlim
explícita puede habilitarlo para la sesión, evitando que una acción accidental
se ejecute con privilegios de TrustedInstaller/SYSTEM.

NSudo se busca primero en `C:\WSCore`, incluyendo sus subdirectorios de
herramientas, y después en el `PATH`. LTools no descarga ejecutables de
privilegios desde URLs no verificadas ni crea usuarios administrativos. Si no
está disponible, la instalación debe hacerse mediante una fuente de Windows
que el usuario haya elegido y confirmado.

Para una automatización WinSlim que requiera explícitamente ese backend, la
sesión puede habilitarlo con `LTOOLS_USE_NSUDO=1`. Sin esa variable, incluso
cuando NSudo esté presente, las acciones de LTools continúan usando la
elevación UAC nativa de Windows.

La CLI comparte la misma gramática de navegación: cabecera persistente,
sección activa como ruta, grupos de acciones, `Enter`/`b` para volver, `h`/`?`
para ayuda y `q` para salir. Un cambio de tema o idioma se aplica a la sesión
actual y los lanzamientos con argumentos siguen siendo no interactivos y
adecuados para alias de LTerminal.

Las automatizaciones registradas conservan el programa, el directorio de
trabajo y cada argumento por separado. LTools invoca el programa sin shell;
para scripts `.sh`, `.py`, `.cmd`, `.bat` o `.ps1` selecciona el intérprete
nativo correspondiente. El registro por línea de comandos permite integrar
botones de una terminal sin depender de un script Bash:

```text
ltools automation add --name informe --program /ruta/informe.sh --cwd /ruta/proyecto --arg --json
ltools automation list
ltools automation list --format json
ltools automation run informe
ltools automation remove informe
```

La opción Importar del menú ofrece el mismo flujo de forma interactiva.

En cualquier nivel, `Enter` o `q` vuelven al nivel anterior; desde el menú
principal `q` sale de la aplicación y `h` muestra la ayuda. Después de una
acción, una entrada vacía conserva el contexto actual; la pantalla se limpia
al cambiar de nivel.

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

Para automatización o integración desde otra terminal usa el perfil CLI:

```powershell
.\ltools-cli.exe --help
.\ltools-cli.ps1 storage tools
```

Si ejecutas windows\ltools.ps1 desde un checkout del proyecto, el lanzador
busca automáticamente el ejecutable en el paquete Windows de dist y en el
target Rust. Si no existe todavía, ejecuta windows\build.cmd; al fallar,
el lanzador conserva el mensaje visible para poder diagnosticarlo.

La release Windows no incluye scripts Bash, FUSE, Wine, Proton ni comandos
Linux. Las capacidades no aplicables se muestran como tales.

## Comandos

Sin argumentos se abre el menú interactivo:

```bash
./ltools.sh
```

El ejecutable normal (`ltools`, el AppImage principal o `ltools.exe`) sin
argumentos abre el menú de su plataforma. El perfil CLI (`ltools-cli`,
`ltools-cli.sh`, el AppImage CLI o `ltools-cli.exe`) sin argumentos muestra
la ayuda y espera un comando explícito; nunca abre otra ventana.

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
./ltools.sh storage status
./ltools.sh storage partitions
./ltools.sh registry status
./ltools.sh rollback --plan /tmp/ltools-plan.tsv
./ltools.sh capabilities --format json
```

Los informes generados sin `--out` no se guardan en el checkout ni crean un
directorio nuevo por ejecución. LTools mantiene un único informe actualizado
por módulo en `$XDG_STATE_HOME/ltools/reports/` (o en
`~/.local/state/ltools/reports/` si no está definido; en Windows usa
`%LOCALAPPDATA%\\LTools\\reports`). Si se necesita conservar una copia
independiente, se puede indicar explícitamente `--out`.

| Comando | Función |
|---|---|
| `audit` | Discos, aplicaciones, archivos grandes y duplicados |
| `games` | Juegos, runners, Wine, Proton, Steam, Lutris, Heroic y UMU |
| `packages` | Gestores, paquetes instalados, AUR y archivos descargados |
| `report` | Leer informes desde la propia CLI con salida, paginador o editor |
| `clean` | Limpieza protegida de paquetes, caches y rutas |
| `prefix` | Listar, inspeccionar, crear y migrar prefijos |
| `defaults` | Rutas efectivas y defaults de las herramientas |
| `system` | Servicios, procesos, daemons y journal |
| `storage` | Discos, volúmenes, montajes y particiones nativas |
| `registry` | Registro Windows o rutas de configuración Linux |
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
./ltools.sh storage tools
./ltools.sh registry status
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

El descriptor específico incluye además `actions`: un catálogo directamente
convertible en botones de acciones rápidas. Cada acción ofrece `id`, `label`,
`shortLabel`, `group`, `description`, `executable`, `args`, `command`,
`workingDirectory`, `interactive`, `requiresAdmin`, `confirmation`, `safe`,
`supports` y `requiresCommands`. La terminal debe preferir `executable` +
`args` (argv separado, sin interpretar una cadena de shell); `command` queda
como representación legible y compatibilidad con hosts antiguos. Por ejemplo,
un botón de auditoría puede usar `executable: "ltools"`, `args: ["audit"]`,
`workingDirectory: "current"` y `terminal: true`. Así LTerminal solo tiene que
resolver el ejecutable de la release instalada, comprobar los requisitos y
abrir una pestaña con esos argumentos.

Las acciones marcadas como `safe: true` son consultas o previsualizaciones. Las
acciones que puedan cambiar el sistema deben declarar confirmación y, cuando
proceda, ofrecer `--dry-run`; la terminal no debe ocultar ni elevar comandos
por su cuenta. `requiresCommands` permite ocultar o marcar un botón cuando la
dependencia concreta no está disponible, sin convertir LTools en una tienda.

El flujo recomendado para LTerminal/WinSlim Terminal es: leer
`distribution/ltools-project.json` desde el catálogo de proyectos, descargar
la release estable indicada para el sistema, seleccionar el descriptor de
`integration.descriptors` para la plataforma (`ltools-terminal.json` en Linux,
`ltools-terminal-windows.json` en Windows) y convertir `actions` en botones.
El botón debe ejecutar
`executable` con `args`, conservar `workingDirectory: "current"` y mostrar la
salida en una pestaña. El descriptor se genera en cada build, por lo que en
Windows ya contiene `ltools.exe` y solo acciones nativas Windows; en Linux
contiene `ltools` y las acciones Linux disponibles. No hace falta mantener un
segundo catálogo manual ni modificar el JSON cuando cambie una ruta local.

Para una integración directa basta con distribuir también los descriptores
generados por plataforma; `ltools-terminal.json` es la versión reducida del
contrato Linux y `ltools-terminal-windows.json` su variante Windows. Están
destinados única y exclusivamente a la integración con la familia LTerminal.
Son opcionales: no se
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

## Diagnóstico nativo

La acción `diagnostics` es de solo lectura: consulta la salud del sistema, la
red, el hardware y los usuarios/sesiones usando únicamente las herramientas
nativas de la plataforma. Si alguna herramienta opcional no existe, aparece
como no disponible; no instala nada automáticamente ni cambia servicios,
procesos, discos o configuración.

```bash
./ltools.sh diagnostics health
./ltools.sh diagnostics network
./ltools.sh diagnostics hardware
./ltools.sh diagnostics users
./ltools.sh diagnostics health --format json
./ltools.sh diagnostics network --format tsv
```

En Linux puede utilizar `ip`, `ss`, `resolvectl`, `lsblk`, `lscpu`, `free`,
`lspci`, `lsusb`, `who` y `loginctl`. En Windows utiliza PowerShell, CIM/WMI,
`query` y cmdlets nativos de red, discos, memoria y vídeo. La salida JSON usa
el esquema `ltools-diagnostics-v1`, por lo que LTerminal, WinSlim Terminal u
otro frontend puede mostrar cada comprobación y su disponibilidad sin
interpretar texto humano.

## Herramientas nativas generales

Además del diagnóstico, `native` reúne acciones concretas de red, hardware,
energía y seguridad. Linux usa `ip`, `resolvectl`, `lscpu`, `free`,
`powerprofilesctl`, `upower`, `firewalld`, `ufw` o `nftables` cuando están
disponibles; Windows usa PowerShell, CIM, `ipconfig`, `powercfg`, Firewall y
Defender. Las consultas no cambian el estado.

```bash
./ltools.sh native menu
./ltools.sh native network status
./ltools.sh native hardware status
./ltools.sh native power status
./ltools.sh native security status
./ltools.sh --dry-run native network flush-dns
```

`network flush-dns` es la única acción mutable de este bloque por ahora:
requiere confirmación y admite `--dry-run`. En Windows se usa
`ltools.exe native network flush-dns`. Si falta una herramienta opcional, se
informa y, cuando el catálogo conoce un paquete seguro, se ofrece instalar
solo esa dependencia desde el gestor existente; LTools no instala Wine,
juegos, virtualización ni una colección de terceros.

Las operaciones se publican como `native.network-status`,
`native.hardware-status`, `native.power-status`, `native.security-status` y
`native.dns-flush` en `ltools actions list --format json`, para que una GUI o
una terminal cree botones sin duplicar la lógica.

## Discos, particiones y configuración nativa

El menú muestra acciones adaptadas a la plataforma actual. En Linux, el
submenú permite consultar y gestionar el flujo habitual:

```bash
./ltools.sh storage status
./ltools.sh storage partitions
./ltools.sh storage mounts
./ltools.sh storage inspect /dev/sda1
./ltools.sh storage mount /dev/sdb1
./ltools.sh storage unmount /mnt/datos
./ltools.sh storage health /dev/sda
./ltools.sh storage check /dev/sda1
./ltools.sh storage usage
./ltools.sh storage filesystems
./ltools.sh storage volume-stack
./ltools.sh storage blockdev /dev/sda
./ltools.sh storage partition-table /dev/sda
./ltools.sh storage guide
./ltools.sh storage tools
./ltools.sh registry status
./ltools.sh registry paths
```

`status`, `partitions`, `partition-table`, `mounts`, `inspect`, `health` y `check` son consultas;
la comprobación usa `fsck -N` y nunca repara. `mount` y `unmount` piden
confirmación, validan el objetivo y se anotan en el plan. `open-gparted` abre
el gestor gráfico instalado, pero LTools no genera órdenes destructivas de
particionado. Si falta una herramienta opcional, se ofrece su instalación
puntual mediante `doctor --install`.

`storage guide` documenta el flujo seguro de particionado. En Linux enumera
`lsblk`, `parted print`, `fdisk -l` y `sfdisk --dump`; en Windows documenta
`list`, `select`, `detail`, `create`, `extend`, `shrink`, `format`, `convert`,
`clean` y `delete`, marcando las operaciones destructivas y sin generarlas
automáticamente. El selector guiado excluye `/`, raíces de montaje y `C:` de
cualquier preselección.

La acción native tools status consulta dependencias y muestra qué componentes
puede instalar LTools. `native tools install --tool ID` abre la instalación
guiada de una dependencia concreta; en la GUI existe el mismo botón con un
modal selector. La acción native tools menu abre los
flujos operativos. No son informes pasivos: permiten conectar por SSH, copiar
con SCP, abrir SFTP, ejecutar shell/transferencias/reinicio con ADB,
descargar/crear/iniciar/detener/reiniciar/eliminar contenedores, ver logs,
ejecutar comandos, inspeccionar contenedores, consultar estadísticas, procesos,
puertos y diferencias, pausar/reanudar/terminar, renombrar y copiar archivos;
también permite construir, inspeccionar, etiquetar, eliminar y limpiar imágenes,
gestionar volúmenes y redes, consultar el uso del motor, ejecutar `prune` y
gestionar Compose/Podman Compose con ciclo de vida, servicios, logs, build,
config, run, exec y limpieza, además de aplicar/eliminar manifiestos,
escalar deployments, reiniciar rollouts y abrir port-forward en Kubernetes.
Cada objetivo se solicita explícitamente, se muestra el comando completo, pide
confirmación cuando puede modificar estado y admite --dry-run.

Los mismos flujos pueden invocarse sin stdin —útil para la GUI y para
LTerminal— con argumentos separados, por ejemplo:

    ./ltools.sh native tools ssh-connect --target usuario@servidor
    ./ltools.sh native tools adb-install --apk ./app.apk
    ./ltools.sh native tools container-run --image alpine:latest --name prueba
    ./ltools.sh native tools container-inspect --name prueba
    ./ltools.sh native tools image-build --path ./servicio --tag servicio:dev
    ./ltools.sh native tools container-compose --operation ps --file compose.yml
    ./ltools.sh native tools system-df
    ./ltools.sh native tools kubernetes-apply --file ./deployment.yaml

Si falta ssh, adb, Docker/Podman, containerd, crictl o cualquier cliente
Kubernetes catalogado, LTools ofrece la instalación del componente cuando el
catálogo de la plataforma conoce un instalador seguro;
nunca inventa un gestor ni ejecuta una orden de instalación sin aceptación
explícita. Las operaciones remotas y de clúster requieren objetivos explícitos.

En Windows, el mismo comando usa PowerShell y las herramientas nativas:

```powershell
.\ltools.exe storage status
.\ltools.exe storage partitions
.\ltools.exe storage mounts
.\ltools.exe storage inspect C:
.\ltools.exe storage health C:
.\ltools.exe storage open-disk-management
.\ltools.exe storage open-diskpart
.\ltools.exe storage tools
.\ltools.exe registry status
.\ltools.exe registry query --key HKCU\Software
.\ltools.exe registry export --key HKCU\Software --out C:\Temp\ltools.reg
```

En Windows, `health` y `check` ejecutan `Repair-Volume -Scan` sin reparar.
`open-disk-management` y `open-diskpart` delegan las operaciones de
particionado al Administrador de discos o DiskPart nativos, después de pedir
confirmación; LTools no ejecuta scripts destructivos por su cuenta. Todas las
acciones admiten `--dry-run` para revisar el plan sin cambiar el sistema.

Las consultas del Registro son de solo lectura. `export` crea un respaldo
`.reg`; no importa claves ni cambia el Registro. LTools no usa `systemctl`,
`parted` ni rutas Wine en el ejecutable Windows, y tampoco usa `sc.exe`,
`diskpart` ni `reg.exe` en el ejecutable Linux.

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

Por defecto genera únicamente `summary.txt` e `inventory.tsv`, que reúnen la
información esencial en un informe compacto. Si se necesita compatibilidad con
flujos que esperan un fichero por gestor, `--full` conserva además los TSV
detallados y `package-artifacts.tsv`.

Desde el menú de inventario se ofrece leer el informe inmediatamente. También
se puede abrir de forma explícita:

```bash
./ltools.sh report view --path "$HOME/Informes/ltools-packages/summary.txt"
./ltools.sh report menu --path "$HOME/Informes/ltools-packages"
./ltools.sh packages --out "$HOME/Informes/ltools-packages" --view-report
```

El lector usa salida directa por defecto, `PAGER`/`less`/`more` como paginador
y `VISUAL`/`EDITOR` o `nano`/`vi` para editar cuando el usuario lo solicita.
El editor nunca se abre automáticamente.

Ejemplos seguros:

```bash
./ltools.sh packages --out "$HOME/Informes/ltools-packages"
./ltools.sh clean --dry-run --package-caches --plan /tmp/ltools-clean.tsv
./ltools.sh clean --dry-run --path "$HOME/.cache/paru" --plan /tmp/ltools-cache.tsv
./ltools.sh --dry-run rollback --plan /tmp/ltools-cache.tsv
./ltools.sh rollback --plan /tmp/ltools-cache.tsv
```

El modo de limpieza no incluye automáticamente bibliotecas de juegos, máquinas
virtuales, prefijos ni puntos de montaje. Esas rutas requieren selección
explícita y mantienen los bloqueos de seguridad.

### Búsqueda e instalación contextual

`software search` consulta únicamente las stores que existan en el sistema
actual. En Linux reconoce pacman/AUR (`pacman`, `paru`, `yay`, `pikaur`), apt,
dnf/yum, zypper, apk, XBPS, pkg, Flatpak, Snap, Homebrew, Nix, Guix y eopkg;
en Windows reconoce solo `winget`, Chocolatey y Scoop. No ejecuta `update`, no
mantiene un catálogo propio y continúa si una store no responde.

```bash
./ltools.sh software stores
./ltools.sh software search firefox --format json --limit 50
./ltools.sh --dry-run software install firefox --candidate 2 --yes
```

Si hay varios resultados, muestra gestor, identificador, versión y origen, y
exige elegir uno. La instalación real siempre muestra el comando nativo y
solicita confirmación; `--yes` solo evita esa pregunta cuando el usuario ya
ha seleccionado un candidato exacto con `--candidate`. Los gestores que
requieren privilegios usan UAC/sudo según la plataforma. LTools nunca instala
varios paquetes en lote ni resuelve dependencias a ciegas.

El formato JSON usa el esquema `ltools-package-search-v1`, pensado para que
una acción rápida pueda presentar candidatos sin interpretar texto humano.
`--limit` admite entre 1 y 1000 resultados (por defecto, 100), evitando que
una store ruidosa bloquee la interfaz.

En la GUI, **Instalar** abre este flujo en una pantalla propia: la búsqueda no
ocupa espacio permanentemente en el menú principal y los resultados aparecen
en el panel inferior redimensionable.

### Operaciones Git

El módulo `git` no es un cliente de credenciales ni modifica una shell. Usa
`git` mediante argumentos separados y valida repositorios, URLs y destinos.

```bash
./ltools.sh git status --repo ./proyecto
./ltools.sh git log --repo ./proyecto --limit 30
./ltools.sh --dry-run git clone https://github.com/usuario/proyecto.git ./proyecto --yes
./ltools.sh --dry-run git fetch --repo ./proyecto --prune --yes
./ltools.sh --dry-run git pull --repo ./proyecto --rebase --yes
./ltools.sh --dry-run git add --repo ./proyecto --all --yes
./ltools.sh --dry-run git commit --repo ./proyecto --message "mensaje" --all --yes
./ltools.sh --dry-run git push --repo ./proyecto --remote origin --branch main --yes
./ltools.sh --dry-run git branch --repo ./proyecto --switch main --yes
./ltools.sh --dry-run git tag --repo ./proyecto --name v1.0.0 --message "release" --yes
./ltools.sh --dry-run git release --repo usuario/proyecto --tag v1.0.0 --title "LTools 1.0.0" --notes "Notas" --yes
./ltools.sh --dry-run git gh repo --repo usuario/proyecto
./ltools.sh git login
```

`pull` se bloquea si hay cambios sin confirmar salvo que se indique
`--allow-dirty` de forma explícita. `clone`, `fetch`, `pull`, `add`, `commit`,
`push`, ramas, tags y releases piden confirmación o requieren `--yes` explícito
en automatización, y quedan registrados en el plan. No se ofrece rollback
automático de cambios Git porque un pull puede implicar hooks, merges o trabajo
remoto irreversible. La GUI Linux expone estas mismas operaciones desde un
submenú Git/GitHub: el repositorio, URL, destino, rama, remoto, mensajes, notas
y límite se introducen en campos separados; la salida completa queda en el
panel de resultados. Las consultas de GitHub (`repo`, `prs`, `releases` y
`auth-status`) solo se habilitan si está instalado `gh`. `git login` muestra la
identidad configurada y ofrece el flujo oficial; LTools nunca lee, guarda ni
imprime tokens o contraseñas.

## Build y distribución

Build Linux completa:

```bash
./build.sh --non-interactive --fast
```

La build Linux ejecuta rustfmt, Clippy, tests Rust, sintaxis Bash, contratos,
compilación release, tarball, AppImage, smoke, E2E de migración/rollback, E2E
de menús y funciones, y una E2E aislada de stores simuladas y Git. También valida AppStream, FUSE, idiomas, gestores de
paquetes, duplicados y las rutas efectivas del ecosistema Wine. En una build
interactiva, Wine/Proton viene activado por defecto (`S/n`); se puede desactivar
con `n` o con `--no-windows-wine`.

`dist/` es staging local: contiene logs, tiempos, informes y salidas de trabajo.
`release/` es la carpeta canónica de publicación: el builder copia allí los
artefactos finales sin mezclar código Linux y Windows. Tras ejecutar ambos
builders, contiene los dos AppImage Linux, los dos `.exe` y ZIP Windows, los
perfiles CLI, los descriptores JSON, sus esquemas y `ltools-release.json`.

Validación Windows opcional desde la misma build Linux:

```bash
./build.sh --windows-wine
./build.sh --windows-wine --windows-wine-runner "$HOME/.local/share/umu/compatibilitytools/UMU-Latest/files/bin/wine"
```

Esta etapa compila `x86_64-pc-windows-gnu`, comprueba que el runner puede abrir
una consola Windows y ejecuta la misma superficie verificable del ejecutable:
versión, ayuda, capacidades JSON, rutas por defecto y menú. Usa un prefijo
temporal aislado, registra tiempos y salida en el log principal, y genera en
`dist/windows-wine/` los dos perfiles (`.exe` normal y `-cli.exe`). Cuando se
ejecuta desde el builder raíz, ambos perfiles se copian también a `release/` y
la E2E exige que estén presentes y cubiertos por el manifiesto y sus hashes.
El target de Cargo de esta etapa está aislado en
`rust/target/windows-wine`; por ello `--clean --windows-wine` no puede borrar
el binario Linux que todavía necesita el empaquetado. Se puede cambiar de
forma explícita con `LTOOLS_WINDOWS_CARGO_TARGET_DIR`, aunque no es necesario
para el uso normal.
Estos `.exe` GNU están validados bajo Wine; la release oficial Windows para
distribuir a usuarios Windows sigue siendo la producida por `windows/build.ps1`
con MSVC.

En una ejecución interactiva sin argumentos, el builder pregunta si se mantiene
esta etapa activada por defecto. `--windows-wine-prefix` permite usar un prefijo
concreto, y `--windows-wine-install-mono` permite preparar Wine Mono cuando el
runner no lo incluye. LTools no necesita Mono: se ofrece únicamente para
validar el entorno de otras aplicaciones Windows.

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
./build.sh --release-dir /tmp/ltools-release
./build.sh --appimage --no-package
./build.sh --non-interactive --no-smoke --no-e2e
./build.sh --appimage --require-fuse
```

El builder Windows está en `windows/build.ps1` y usa MSVC por defecto:

### Validación Windows desde Linux con Wine/Proton

Para compilar el target GNU y ejecutar el ejecutable Windows en un prefijo
aislado, usa:

```bash
./tests/linux/windows-wine.sh
```

El comprobador prioriza UMU-Wine, prueba la consola, el ejecutable, el JSON,
defaults y el menú, y elimina el prefijo temporal al terminar. También acepta
`--runner RUTA`, `--prefix RUTA`, `--output DIR`, `--keep-prefix`,
`--no-tests`, `--fast`, `--offline`, `--jobs N` y `--install-mono`. Wine Mono no
se instala por defecto porque LTools no usa .NET; esa opción solo prepara el
prefijo para probar software Windows que sí lo necesite. El ejecutable se
construye en `rust/target/windows-wine` por defecto, para no interferir con
otros targets locales. Este flujo es una validación desde Linux; la release
oficial continúa compilándose con el builder nativo Windows.

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

Las releases publicables generan además `SHA256SUMS.txt` y su firma separada
`SHA256SUMS.txt.sig`. La firma usa Ed25519 y contiene Base64 de la firma del
contenido exacto de `SHA256SUMS.txt`, el mismo formato que usa LTerminal. El
builder busca automáticamente estas claves, sin incluirlas en el paquete:

```text
~/.config/lterminal/release-signing-private.pem
~/.config/lterminal/release-signing-public.hex
```

También se pueden indicar otras rutas con `LTOOLS_SIGNING_PRIVATE_KEY_FILE` y
`LTOOLS_UPDATE_PUBLIC_KEY_FILE` (o sus equivalentes `LTERMINAL_*`). La clave
privada nunca debe entrar en GitHub ni en el repositorio. Los builders exigen
la firma por defecto, tanto en Linux como en Windows; si faltan las claves la
release se detiene antes de publicar artefactos:

```bash
./build.sh
```

Para una build local deliberadamente no publicable sin firma hay que declarar
la excepción explícita `./build.sh --allow-unsigned` o
`.\windows\build.ps1 -AllowUnsigned`. `--require-signing` y las variables
`LTOOLS_REQUIRE_SIGNING`/`LTERMINAL_REQUIRE_SIGNING` se mantienen como
compatibilidad para pipelines que quieran expresar la exigencia de forma
explícita. El backend también permite verificar manualmente una release:

```bash
./ltools.sh release-signature \
  --manifest release/SHA256SUMS.txt \
  --signature release/SHA256SUMS.txt.sig \
  --public-key-file ~/.config/lterminal/release-signing-public.hex \
  --verify
```

La URL estable para la terminal será:

```text
https://raw.githubusercontent.com/Darkeiser003/Tools/main/distribution/ltools-project.json
```

Después de construir Linux y Windows, el manifiesto unificado se puede regenerar
desde cualquier binario release de LTools:

```bash
VERSION="$(sed -n 's/^version = "\([^" ]*\)"/\1/p' rust/Cargo.toml | head -n1)"
./ltools.sh release-manifest \
  --output release/ltools-release.json \
  --repository Darkeiser003/Tools \
  --tag "v$VERSION" \
  --artifacts-dir release
```

Sube a la release de GitHub todos los archivos publicables de `release/`,
incluidos `ltools-release.json`, `SHA256SUMS.txt` y `SHA256SUMS.txt.sig` con
esos nombres exactos. La terminal puede leer primero el descriptor
del proyecto, seleccionar el artefacto apropiado para el sistema y verificar el
SHA-256 y la firma Ed25519 antes de ofrecer la instalación. El descriptor de
integración de terminal sigue siendo opcional y separado.

Para cambiar de repositorio o de etiqueta sin editar archivos, usa
`LTOOLS_GITHUB_REPOSITORY` y `LTOOLS_GITHUB_TAG` al ejecutar los builders.

```powershell
.\windows\build.ps1
.\windows\build.ps1 -Fast
.\windows\build.ps1 -Force -NoRun
.\windows\build.ps1 -ReleaseOutput .\release
.\windows\build.ps1 -Target x86_64-pc-windows-gnu
```

Mantiene su estado incremental en `dist/windows/.build-state.json`, separa el
target en `rust/target/windows`, genera un ZIP portable por arquitectura y
publica el `.exe`, el perfil CLI, el ZIP y los JSON en `release/`. `-Output`
controla el staging Windows y `-ReleaseOutput` la carpeta que se puede subir a
GitHub.

## Arquitectura

El proyecto mantiene dos capas compatibles:

```text
.
├── ltools.sh                 Lanzador compatible del backend Rust
├── rust/
│   ├── Cargo.toml            Backend nativo y metadata de la aplicación
│   └── src/                  Núcleo compartido y módulos funcionales
│       ├── platform/         Capacidades, privilegios e instalación por plataforma
│       ├── storage/          Discos y particiones: Linux/Windows separados
│       ├── registry/         Configuración: Linux/Windows separados
│       └── automation.rs     Registro seguro de scripts y automatizaciones
├── platform/
│   └── linux/
│       └── build.sh          Builder Linux/AppImage
├── appimage/                 AppRun, desktop, icono y metadata AppStream
├── windows/                  Builder, lanzadores y tests nativos Windows
├── release/                  Artefactos publicables regenerables (ignorado)
├── dist/                     Staging, logs y salidas locales (ignorado)
├── clean-repository.sh       Limpieza segura de artefactos regenerables
├── tests/                    Contratos y pruebas
│   └── linux/                Smoke y E2E Linux/AppImage
└── build.sh                  Lanzador compatible del builder Linux
```

Rust es el único backend de LTools para Linux, AppImage y Windows. Los scripts
`.sh` restantes son lanzadores, builders o harnesses de prueba; no se invocan
como implementación funcional ni se empaquetan en el tarball/AppImage.

## Idiomas y temas

El idioma se obtiene, por orden, de `LTOOLS_LANG`, de las variables que puede
proporcionar una terminal anfitriona (`LTERMINAL_LANGUAGE`, `LTERMINAL_LANG`,
`WINSLIM_TERMINAL_LANGUAGE`, `WINSLIM_TERMINAL_LANG`) y de
`LC_ALL`, `LC_MESSAGES` o `LANG`. También se puede forzar en una ejecución:

```bash
LTOOLS_LANG=en ./ltools.sh --help
./ltools.sh --lang de menu
./ltools.sh --lang fr defaults
```

LTools acepta los 15 IDs de locale de LTerminal: `ar`, `de`, `en`, `es`, `fr`,
`hi`, `it`, `ja`, `ko`, `pl`, `pt`, `ro`, `ru`, `uk` y `zh`. `auto` usa el
locale del entorno y, si una cadena concreta aún no tiene traducción, se usa
español como fallback.
Los tests comprueban normalización como `en_US.UTF-8` y `pt-BR`, además de todos
los catálogos disponibles.

La CLI comparte el contexto visual con la terminal sin depender de ella. La
precedencia es: argumentos explícitos, variables `LTOOLS_*`, variables del
host y valores autónomos. Ejemplos:

```bash
LTERMINAL_LANGUAGE=en LTERMINAL_THEME=matrix ./ltools-cli.sh menu
./ltools.sh --lang de --theme nordic --color always menu
./ltools.sh --theme contrast --no-color audit --format json
```

Los temas disponibles, en el mismo orden que LTerminal, son `silver`,
`winslim`, `ocean`, `forest`, `amber`, `violet`, `nordic`, `crimson`, `matrix`,
`contrast`, `slate`, `plum` y `teal`.
También se aceptan alias compatibles con LTerminal como `greenPhosphor`,
`highContrast` y `techCyan` (normalizado a `winslim`). `--color auto` solo usa ANSI cuando la salida es una terminal;
`always` y `never` permiten automatizar o depurar ese comportamiento. Las
salidas JSON y TSV nunca incluyen secuencias ANSI.

La GUI autónoma usa por defecto la paleta oscura `silver`, inspirada en la
superficie negra, los bordes finos y el texto plateado de LTerminal. Incluye
botones de **Tema**, **Idioma** y **Ajustes**. Tema se aplica inmediatamente;
idioma y visibilidad se guardan para el siguiente arranque en
`$XDG_CONFIG_HOME/ltools/gui-preferences.conf` (o `%APPDATA%\ltools` en
Windows). Una elección manual
se guarda en la configuración de LTools y tiene prioridad sobre el tema/idioma
recibido de la terminal; mientras no exista, se hereda el contexto de
LTerminal o WinSlim Terminal. La integración declarativa para hosts documenta
el mismo contrato en `ui_context`.

## Logs, planes y rollback

Cada build guarda una transcripción y una tabla de tiempos:

```text
dist/build-AAAAMMDD-HHMMSS-PID.log
dist/build-AAAAMMDD-HHMMSS-PID-timings.tsv
dist/appimage-smoke.log
```

El log registra configuración, comandos, códigos de salida y duración. Las
operaciones modificadoras aceptan `--plan FICHERO`; el plan describe acciones
ejecutadas y permite lanzar `rollback --plan FICHERO`. `rollback --dry-run`
valida el plan y muestra las restauraciones previstas sin mover, copiar ni
retirar nada. Las consultas puras no crean planes de estado salvo que se
solicite explícitamente `--plan` o `--dry-run`; así los inventarios no llenan
la carpeta de planes vacíos. Los planes automáticos usan un único fichero
estable por módulo (`plan-rust-storage.tsv`, `plan-rust-system.tsv`, etc.) y
se eliminan si la operación no registró ningún paso reversible. Una nueva
operación del mismo módulo reemplaza su plan automático anterior; para
conservar un plan concreto y poder recuperarlo después, usa `--plan FICHERO`.

Los logs, informes, planes, targets, staging y artefactos de distribución son
locales y están excluidos por `.gitignore`. Para inspeccionar qué residuos
regenerables existen antes de limpiar el árbol de trabajo:

```bash
./clean-repository.sh --dry-run
```

Los planes automáticos legacy se almacenan fuera del repositorio. Para
revisarlos sin borrar nada, usa `./clean-repository.sh --plans-only --dry-run`.
La retirada requiere `--plans-only --apply`; solo coincide con los nombres
fechados o asociados a un PID de la implementación antigua y conserva los
planes estables y los ficheros que el usuario haya nombrado explícitamente.

El limpiador solo conoce carpetas de salida explícitas (`dist/`, `release/`,
`rust/target/`, targets Windows y caches habituales de herramientas). Protege
carpetas que contengan archivos versionados, enlaces simbólicos y cualquier
archivo no ignorado por Git. No borra fuentes, documentación, tests ni
configuración. La simulación es el comportamiento predeterminado; para
retirar los candidatos confirmados:

```bash
./clean-repository.sh --apply
```

Para automatizaciones no interactivas, después de revisar previamente el plan:

```bash
./clean-repository.sh --apply --yes
```

La limpieza no afecta cachés fuera del repositorio ni paquetes del sistema.
Los artefactos de release se regeneran con `./build.sh`; los cambios de código
sin commit permanecen intactos.

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
./tests/linux/software-git-e2e.sh --binary rust/target/release/ltools
```

En Windows nativo:

```powershell
.\windows\build.ps1 -Force
```

El ejecutable Windows es nativo y no comparte los detectores Linux: `games`
inspecciona Steam, Epic Games, Ubisoft Connect, EA app, itch.io, Battle.net,
Rockstar, GOG y Xbox/Microsoft Store mediante rutas y manifiestos Windows.
`prefix` se conserva como comando reconocido para devolver una explicación,
pero no busca ni migra prefijos Wine/Proton. `defaults` muestra las ubicaciones
nativas de esos lanzadores.

El smoke y la E2E Windows están en `windows/tests/` y prueban el ejecutable,
el contrato de capacidades, el inventario nativo, informes, planes y acciones
del sistema. Usan `native-process.ps1`, un capturador .NET común con UTF-8,
timeouts, cierre de stdin y diagnóstico de stdout/stderr; no dependen del
pipeline frágil de PowerShell para procesos nativos. Deben ejecutarse en
Windows. Desde Linux, la compilación y prueba aislada bajo Wine/Proton viene
activa por defecto en `./build.sh`; se puede desactivar con
`--no-windows-wine`. Usa un prefijo temporal y no activa la lógica Linux de
prefijos.

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
