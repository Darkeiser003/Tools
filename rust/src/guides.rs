//! Guías operativas en texto para que cada acción tenga contexto, requisitos y
//! un flujo seguro tanto en la CLI como en las interfaces gráficas.

#![allow(dead_code)]

use crate::common::Context;

#[cfg(windows)]
pub fn help() -> &'static str {
    "guide [cli|gui] [all|audit|packages|software|installable|git|gh|aliases|automation|automation-register|clean|storage|storage-partitions|storage-filesystems|storage-volumes|system|services|accounts|native|network|connectivity|boot|registry|diagnostics|wine|defaults|settings|updates|containers|containers-lifecycle|containers-images|containers-volumes|containers-compose|kubernetes|ssh|adb|utilities|actions|privileges|winslim]"
}

#[cfg(not(windows))]
pub fn help() -> &'static str {
    "guide [cli|gui] [all|audit|packages|software|installable|git|gh|aliases|automation|automation-register|clean|storage|storage-partitions|storage-filesystems|storage-volumes|system|services|accounts|native|network|connectivity|boot|registry|diagnostics|wine|defaults|settings|updates|containers|containers-lifecycle|containers-images|containers-volumes|containers-compose|kubernetes|ssh|adb|utilities|actions|privileges]"
}

const INDEX: &str = r#"GUÍAS DE USO DE LTOOLS

Consulta una guía concreta con:
  ltools guide git
  ltools guide network
  ltools guide all

Todas las guías explican qué hace cada opción, requisitos, parámetros,
ejemplos, permisos y riesgos. Las acciones que cambian el sistema admiten
--dry-run y, cuando corresponde, --plan FICHERO. La simulación muestra el
comando y el objetivo sin aplicarlo; el plan permite revisar y hacer rollback.

Familias disponibles:
  audit       auditoría e inventarios
  packages    paquetes, almacenes y limpieza
  software    búsqueda e instalación contextual
  installable herramientas instalables y sus submenús
  git / gh    Git avanzado y GitHub CLI
  aliases     gestión CLI de alias por usuario; no tiene pantalla gráfica
  automation  automatizaciones y acciones declarativas
  automation-register  registro de scripts con campos separados
  clean       limpieza con selección segura
  storage     discos, particiones, formatos y volúmenes
  storage-partitions / storage-filesystems / storage-volumes
               menús especializados de almacenamiento
  system      estado del sistema y herramientas nativas
  services    servicios, scopes, origen y gestión
  accounts    usuarios, grupos, sesiones y permisos
  network     interfaces, NetworkManager, rutas, DNS y puertos
  connectivity SSH, SCP, SFTP y Android/ADB
  boot        EFI, GRUB, systemd-boot y Secure Boot
  registry    configuración y registros por plataforma
  diagnostics salud, dependencias y capacidades
  wine        prefijos Wine/Proton y migraciones
  containers  Docker/Podman, imágenes, volúmenes y Compose
  containers-lifecycle / containers-images / containers-volumes /
  containers-compose  menús especializados de contenedores
  kubernetes  recursos, despliegues, rollouts y port-forward
  ssh / adb   acceso remoto y dispositivos Android
  utilities   utilidades detectadas e instalables
  defaults    rutas, configuración y valores efectivos
  settings    preferencias visuales, visibilidad y elevación por defecto
  updates     comprobar releases y descargar el paquete verificado para esta plataforma
  actions     catálogo de acciones y contratos para automatizar
  privileges  política de elevación, sudo/UAC y acciones que nunca se elevan

En la GUI, las categorías que tienen pantalla propia ofrecen una guía
contextual con las opciones de esa pantalla. El gestor de alias es una función
CLI y no se presenta como un menú gráfico inexistente.
"#;

const UPDATES: &str = r#"GUÍA: ACTUALIZACIONES DE LTOOLS

Consulta:
  `ltools update check` consulta la release estable del repositorio configurado
  (por defecto `Darkeiser003/Tools`), muestra la versión actual y la disponible,
  selecciona el paquete compatible con este binario y no modifica archivos.
  `--repository OWNER/REPO` permite usar otro repositorio.

Descarga:
  `ltools update download` solo descarga una versión posterior. Antes comprueba
  el manifiesto, la firma Ed25519 de `SHA256SUMS.txt`, el hash del manifiesto,
  el hash del artefacto y su tamaño. Guarda el paquete en Descargas con un
  nombre que no sobrescribe archivos existentes. No ejecuta ni reemplaza el
  binario; cierra LTools, conserva una copia anterior y usa el procedimiento
  de instalación de tu paquete. `update install` no instala automáticamente.

Confianza y compatibilidad:
  Los binarios producidos por los builders oficiales incluyen la clave pública
  de release para verificar actualizaciones sin configuración adicional. Las
  builds locales sin esa clave solo informan; para permitir una descarga se
  debe configurar `LTOOLS_UPDATE_PUBLIC_KEY_FILE` o `LTOOLS_UPDATE_PUBLIC_KEY`.
  No se usa `gh`, no se solicita elevación y el actualizador nunca cambia de
  usuario: solo escribe en la carpeta Descargas del usuario actual.

Linux elige el AppImage si se ejecuta desde uno; en caso contrario ofrece el
tarball portable. Windows ofrece el ejecutable correspondiente al perfil GUI
o CLI. Tras la descarga, sustituye la instalación siguiendo las instrucciones
de su ubicación; no ejecutes un instalador o EXE sin revisar el origen.
"#;

const GIT: &str = r#"GUÍA: GIT Y GITHUB (git + gh)

Requisitos:
  git para repositorios locales y remotos. `gh` (GitHub CLI) es opcional y su
  catálogo cambia según la versión; consulta la ayuda del ejecutable instalado
  y usa el passthrough seguro para cualquier comando/extensión no integrada.
  Comprueba disponibilidad con `ltools doctor` o `ltools native tools status`.

Flujo recomendado:
  1. `ltools git status --repo RUTA` revisa rama y cambios.
  2. `ltools git clone URL --destination RUTA --dry-run` valida el destino.
  3. `ltools git fetch --repo RUTA` actualiza referencias sin mezclar cambios.
  4. `ltools git pull --repo RUTA` integra cambios tras revisar el estado.
  5. `ltools git add --repo RUTA --path FICHERO` prepara rutas explícitas.
  6. `ltools git commit --repo RUTA --message "Mensaje"` crea el commit.
  7. `ltools git push --repo RUTA --remote origin --branch main` publica.

Opciones cubiertas por la GUI y la CLI:
  status, log, clone, fetch, pull, add, commit, push, branch, tag, release,
  diagnose y repair del índice. `diagnose` informa del estado y `repair` solo
  reconstruye un índice ausente/ilegible desde un HEAD íntegro.
  `branch` permite listar/crear/cambiar/eliminar; `tag` permite crear y
  publicar etiquetas; `release` prepara una release con tag, título y notas.
  Las acciones destructivas o remotas piden confirmación y admiten `--yes`
  únicamente cuando una interfaz ya confirmó la operación.

GitHub CLI (`gh`):
  `ltools git gh auth-status` comprueba la sesión sin mostrar tokens.
  `ltools git gh login` inicia el flujo oficial de autenticación interactiva.
  `ltools git gh repo --repo OWNER/REPO` consulta el repositorio.
  `ltools git gh prs --repo OWNER/REPO` lista pull requests.
  `ltools git gh releases --repo OWNER/REPO` lista releases.
  `ltools git gh version` y `ltools git gh help` muestran la versión y ayuda
  de la instalación detectada. Para cubrir subcomandos y extensiones que
  cambian según la versión, usa `ltools git gh native <comando> [argumentos]`;
  por ejemplo, `native issue list`, `native pr view 123`, `native run list`,
  `native workflow list` o `native extension list`. La ayuda exacta instalada
  se obtiene con `ltools git gh native help`. El passthrough no fija una lista
  de comandos: recibe las opciones que acepte la versión actual de `gh`, así
  que también cubre `project`, `ruleset`, `attestation`, `codespace`, `api` y
  futuras extensiones/comandos. En GUI, «Comando nativo de gh…» pide un
  comando raíz y argumentos en una sola línea; admite comillas simples/dobles, no
  expande variables ni ejecuta una shell. Usa el campo Repositorio o incluye
  `--repo OWNER/REPO`/`-R OWNER/REPO` en los argumentos, no ambos. El campo no
  se aplica a `auth`, `help`, `version` ni `api`; para API indica el endpoint
  completo y deja vacío el campo.
  Las consultas conocidas no piden confirmación; los comandos nativos no
  clasificados como consulta la piden siempre. `--dry-run` muestra el grupo
  del subcomando sin ejecutarlo. `--ltools-confirmed` confirma el efecto de
  LTools en automatizaciones revisadas; `--yes` se conserva como argumento
  nativo para la versión instalada de `gh`.
  La salida y argumentos completos no se guardan en el plan. Se bloquean
  `gh auth token` y `--insecure-storage` para proteger credenciales. `gh api`
  puede consultar o modificar GitHub: revisa método, endpoint, scopes y datos
  antes de confirmarlo. Las extensiones ejecutan código de terceros; instala
  solo las que confíes. El login requiere navegador/credenciales y red; LTools
  no guarda ni imprime secretos.

Reparación de `.git`:
  `ltools git diagnose --repo RUTA` comprueba raíz, HEAD, índice y objetos.
  `ltools git repair --repo RUTA --dry-run` previsualiza la única reparación
  automática: respaldar y reconstruir un índice ilegible desde un HEAD íntegro.
  La reconstrucción no toca los archivos de trabajo, pero los cambios staged
  dejan de estar activos; la copia previa queda junto al índice para rescate.
  Si falta `.git`, puedes indicar el remoto correcto con
  `ltools git repair --repo RUTA --remote URL [--branch RAMA] --dry-run`.
  Solo tras revisar el plan, repite sin `--dry-run`: LTools clona sin checkout
  a un temporal, valida HEAD/objetos e instala únicamente la metadata; no
  sobrescribe los archivos locales. Compara después el estado con la rama
  remota. Si `.git` existe como archivo/enlace dañado, o faltan objetos, no
  ejecutes `git init` ni lo reemplaces: usa una copia de seguridad o recupera
  en otra ruta; no hay una reparación genérica segura para esos casos.

Seguridad y automatización:
  Nunca se concatena la entrada en una shell. URL, repositorio, rama y rutas
  se pasan como argumentos separados. Usa `--dry-run` antes de clone/pull/
  push/release y `--plan FICHERO` en flujos modificadores. En CI usa el
  catálogo: `ltools actions list --format json` y después
  `ltools actions run native.git-status RUTA` (consulta el ID exacto).
  Revisa el remoto, la rama y el diff antes de publicar; `push --force` no es
  una opción predeterminada y un release afecta al repositorio remoto.
"#;

const GH: &str = r#"GUÍA: GITHUB CLI (gh)

`gh` es opcional y solo se usa para operaciones de GitHub. Instala o habilita
la herramienta desde Dependencias si tu plataforma ofrece un paquete fiable.
Después comprueba `ltools git gh auth-status`.

Acciones integradas: login, auth-status, repo, prs y releases; `version` y
`help` identifican la versión instalada. El login es interactivo;
repo/prs/releases son consultas. Proporciona `OWNER/REPO` cuando no estés
dentro del repositorio.

Para usar el catálogo completo que ofrezca tu versión, incluidos issues,
projects, Actions, API y extensiones, ejecuta `ltools git gh native help` o
`ltools git gh native issue list`. El passthrough llama a `gh` con argumentos
separados, nunca a una shell. Acepta comandos de versiones nuevas sin cambiar
LTools, por ejemplo `gh project`, `gh ruleset`, `gh attestation`, `gh codespace`,
`gh api` y extensiones. Consulta siempre `gh help` para conocer la sintaxis
exacta y `gh extension list` para revisar complementos instalados. Los comandos que LTools no reconoce como
consulta requieren confirmación; `--dry-run` permite previsualizarlos y
`--ltools-confirmed` sirve para una automatización revisada; `--yes` sigue
siendo un argumento nativo de gh. `gh api` admite lectura y
escritura: examina método y endpoint. Las extensiones son código de terceros.

Las operaciones remotas dependen de red, permisos y la sesión de `gh`; LTools
nunca solicita ni registra tokens. No uses `gh auth token` desde LTools y no
actives el almacenamiento de credenciales en texto plano.

Ejemplo seguro:
  ltools git gh auth-status
  ltools git gh repo --repo OWNER/REPO
  ltools git gh prs --repo OWNER/REPO
  ltools git gh releases --repo OWNER/REPO
  ltools git gh native issue list --repo OWNER/REPO
  ltools git gh native run list --repo OWNER/REPO
  ltools git gh native project list --owner OWNER
  ltools git gh native api repos/OWNER/REPO
"#;

const NETWORK: &str = r#"GUÍA: RED

Consultas: status, interfaces, routes, dns, listening y connections.
Gestión: activar/desactivar una interfaz, conectar/desconectar un perfil de
NetworkManager y vaciar caché DNS. Elige siempre la interfaz o conexión
exacta; no se inventa un objetivo.

Flujo: consulta status/interfaces, comprueba rutas y DNS, aplica una acción
concreta y vuelve a consultar. Las mutaciones requieren permisos y
confirmación; `--dry-run` muestra el comando. Desactivar la interfaz activa o
desconectar el perfil puede cortar la sesión remota.

Ejemplos:
  ltools native network status
  ltools native network set-interface --interface eth0 --state up --dry-run
  ltools native network connection-up --connection "Mi WiFi" --dry-run
  ltools native network flush-dns --dry-run
"#;

const BOOT: &str = r#"GUÍA: ARRANQUE, EFI Y GRUB

Consultas: status, efi-entries, grub-entries, systemd-boot y secure-boot.
Gestión disponible: programar la siguiente entrada GRUB y cancelarla. No se
reinicia el equipo automáticamente.

Flujo: identifica el cargador y la entrada exacta, ejecuta `boot plan`, usa
`set-next --entry "Título exacto"` con `--dry-run`, revisa el plan y confirma.
`clear-next --yes` cancela la selección pendiente. EFI/NVRAM y GRUB requieren
elevación en muchos equipos; un nombre incorrecto puede dejar otro sistema
como siguiente arranque, por lo que la GUI exige el valor explícito.

`grboot` puede seguir usándose en tu máquina para el flujo que ya conoces;
LTools ofrece la misma inspección integrada y deja cada cambio registrado.
"#;

const SERVICES: &str = r#"GUÍA: SERVICIOS DEL SISTEMA

La vista separa servicios automáticos/estáticos, manuales/desactivados,
servicios de usuario, ambos scopes, fallidos y journal. También muestra estado,
unidad y origen cuando systemd lo proporciona, y permite exportar TSV.

Gestión: status, start, stop, restart, enable, disable, mask, unmask y
daemon-reload mediante un nombre de unidad exacto. Elige `--scope system` o
`--scope user`; no mezcles un servicio de usuario con uno del sistema.

Flujo: inspecciona, revisa dependencias/journal, usa `--dry-run` o un plan,
aplica el cambio y verifica status. Detener, deshabilitar o enmascarar un
servicio puede afectar red, escritorio o acceso remoto y normalmente exige
sudo. En Windows se informa de la compatibilidad nativa equivalente.
"#;

const STORAGE: &str = r#"GUÍA: ALMACENAMIENTO

Consultas: status, partitions, mounts y tools. Las pantallas avanzadas cubren
parted (GPT/MBR, probe, espacio libre, crear/borrar/redimensionar, nombres,
flags, alineación y rescate), mkfs/fsck/mount/swap, LUKS, LVM, Btrfs, ZFS y
RAID mdadm. En las guías gráficas, las operaciones se enumeran por botón y
campo; la CLI conserva los nombres de operaciones y argumentos nativos.

Flujo seguro: identifica el dispositivo con `lsblk`, consulta la tabla, haz
copia de seguridad, ejecuta `--dry-run` y revisa el plan. Formatear, borrar,
wipefs, discard, LUKS y cambios de tabla son destructivos; requieren objetivo
explícito y confirmación. Nunca uses `/`, `/home` o `/boot` como objetivo por
defecto. Los gestores nativos se abren para operaciones que necesitan una
revisión visual adicional.
"#;

const SYSTEM: &str = r#"GUÍA: SISTEMA Y HERRAMIENTAS NATIVAS

Empieza por `native tools status` y `doctor`: muestran qué existe, versión,
alternativas y limitaciones. Después usa la familia adecuada: red, arranque,
servicios, usuarios, almacenamiento, SSH/ADB o contenedores.

Las consultas no mutan el sistema. Suspender, apagar, firewall, conexiones,
instalaciones, procesos y contenedores sí requieren objetivo/confirmación.
Usa `--dry-run`, planes y registros en automatizaciones; LTools no ejecuta
comandos concatenados ni instala dependencias sin confirmación.
"#;

const WINE: &str = r#"GUÍA: WINE Y PROTON

`wine list` localiza prefijos en Home, Steam, Lutris, Heroic, UMU y Bottles.
`inspect` muestra arquitectura, versión Windows, runner, ejecutables y
bloqueos. `create` crea un prefijo nuevo y `migrate` copia/mueve uno con
actualización de rutas, lanzadores y configuración cuando procede.

Indica origen y destino exactos. Primero lista e inspecciona, luego prueba con
`--dry-run`; migrar puede requerir mucho espacio, permisos y cerrar procesos.
No se borra el origen salvo que lo pidas expresamente. Wine/Proton no se
confunden con los juegos ni con el sistema anfitrión.
"#;

const ACCOUNTS: &str = r#"GUÍA: CUENTAS, GRUPOS Y SESIONES

Consulta cuentas, grupos, identidad, sesiones e inspección de una cuenta.
Gestiona crear/editar/eliminar, contraseña, bloqueo/desbloqueo, caducidad,
crear/eliminar grupos, membresías, grupo principal y el grupo administrativo
local. Linux detecta `sudo`, `wheel` o `admin` solo si existen; comprueba las
reglas sudoers antes de añadir a alguien. Windows usa el grupo integrado
Administradores identificado por su SID estable, aunque el nombre visible
cambie según el idioma. `TrustedInstaller` es una identidad de servicio y no
un grupo de usuarios al que se deba añadir una cuenta.

Usa nombres explícitos y revisa la identidad antes de cambiarla. Eliminar,
cambiar contraseña o permisos puede cortar acceso; requiere confirmación y
normalmente elevación. En Windows se usan las capacidades nativas disponibles
y se informa de las operaciones no soportadas.
"#;

const AUTOMATION: &str = r#"GUÍA: AUTOMATIZACIÓN Y ACCIONES

`automation list` muestra automatizaciones registradas; add/modify/run/remove
las gestionan con campos explícitos. `actions list` publica el catálogo
estable con categoría, objetivo, perfil, mutación, confirmación y compatibilidad.

Para integrar LTerminal, usa executable y args[] separados, nunca una cadena
de shell. Ejemplo: `ltools actions list --format json`; valida el ID y ejecuta
`ltools actions run ID OBJETIVO --dry-run`. Los IDs mutadores crean plan y
requieren confirmación. Registra salida y revisa el resultado antes de repetir.
"#;

const PACKAGES: &str = r#"GUÍA: PAQUETES, ALMACENES Y SOFTWARE

`packages` audita gestores y formatos instalados. `software search` consulta
un nombre y `software install` solo propone el gestor nativo disponible.
LTools no es una tienda ni descarga catálogos de terceros.

La limpieza separa preview de ejecución: usa `clean --preview` para el
inventario clásico y `clean --automatic --preview` para el limpiador guiado.
Este último calcula tamaños de cachés, temporales, cachés de paquetes,
papelera, aplicaciones y, solo si lo pides, Descargas/Documentos y otros
datos personales. Por defecto no incluye datos personales ni aplicaciones.

Ejecuta `clean --automatic` para revisar cada categoría. Después de aceptar
una categoría puedes borrar todos sus elementos o responder uno a uno; una
respuesta vacía/negativa conserva la ruta. Los cachés y temporales conocidos
se eliminan solo tras confirmación; los datos personales se envían a la
papelera; las aplicaciones nunca se borran como carpetas, sino que se deben
desinstalar con el gestor nativo (`packages`/`software`).

`--include-personal` añade Descargas, Documentos, Escritorio, Imágenes,
Vídeos y Música al análisis; `--all-known` es un alias explícito para incluir
esas rutas; `--ask-each` fuerza la pregunta elemento por elemento.
Instalar, actualizar o desinstalar paquetes requiere objetivo explícito,
confirmación y, cuando aplica, plan.
"#;

const CONTAINERS: &str = r#"GUÍA: DOCKER Y PODMAN

Selecciona Docker o Podman según el motor detectado. Gestiona contenedores
(pull, run, start, stop, restart, logs, inspect, exec, pause, kill, rename,
cp, prune), imágenes (build, tag, remove, prune), volúmenes, redes y Compose.

Primero consulta info/df/inspect. Revisa imagen, puertos, volúmenes y comando
antes de ejecutar. remove/prune, bind mounts y `exec` pueden borrar datos o
alterar el host; requieren objetivo explícito, confirmación y `--dry-run` si
el motor permite previsualización. No se mezclan nombres de contenedor con
rutas del host.
"#;

const KUBERNETES: &str = r#"GUÍA: KUBERNETES

Usa el contexto y namespace correctos antes de aplicar. La GUI ofrece apply,
delete, scale, rollout restart y port-forward. Consulta el contexto y revisa
el manifiesto; usa `kubectl diff` cuando esté disponible.

apply/delete/scale alteran el clúster y requieren confirmación, permisos RBAC
y contexto explícito. `port-forward` mantiene una sesión activa y debe
cerrarse al terminar. En CI usa acciones declarativas y conserva el log.
"#;

const SSH: &str = r#"GUÍA: SSH, SCP Y SFTP

SSH conecta a `usuario@host` con puerto y clave explícitos; SCP copia origen y
destino; SFTP abre una sesión. Comprueba primero conectividad y fingerprint.
Las transferencias pueden sobrescribir archivos: confirma las rutas y usa
`--dry-run` cuando la herramienta lo soporte. No introduzcas contraseñas en
argumentos ni logs; usa el agente/llavero del sistema.
"#;

const ADB: &str = r#"GUÍA: ANDROID / ADB

Comprueba dispositivos y autorización USB antes de abrir shell, instalar APK,
push/pull o reiniciar. Selecciona el serial exacto si hay varios dispositivos.
Instalar, escribir archivos y reiniciar son mutaciones; revisa origen/destino,
confirma y conserva los logs. Un shell ADB tiene el alcance del dispositivo y
no debe recibir secretos desde la línea de comandos.
"#;

const REGISTRY: &str = r#"GUÍA: CONFIGURACIÓN Y REGISTRO

`registry status` inspecciona la configuración apropiada a la plataforma:
rutas/journal en Linux y consultas/exportaciones reg.exe en Windows. Las
operaciones de escritura/importación exigen fichero explícito, copia previa,
confirmación y plan. No se intenta usar Registro Windows en Linux.
"#;

const PRIVILEGES: &str = r#"GUÍA: ELEVACIÓN Y PERMISOS

La política se aplica por acción, no como un sudo ciego para toda la
aplicación. `ltools privileges` muestra el catálogo resumido; `--elevate`
solicita elevación para una acción compatible y `--no-elevate` la desactiva
para esa ejecución. En Ajustes puedes activar «Elevar acciones modificadoras
por defecto».

Solo lectura: mapas, inventarios, estado, diagnósticos y guías no necesitan
elevación. Opcional: copiar, mover, comprimir, exportar y limpiar pueden
elevarse cuando el objetivo está protegido; se conserva el plan y se informa
del diálogo de sudo/pkexec o UAC/NSudo según la plataforma.

Obligatoria: particionar, formatear, montar, LUKS/LVM/RAID, gestionar
servicios, cuentas, firewall, red modificadora, EFI/NVRAM y cambios del
sistema. LTools solicita la contraseña o autorización y aborta si no se
concede; nunca la guarda ni la imprime.

La instalación de software es una excepción deliberada al relanzamiento global:
la búsqueda, selección del paquete y confirmación permanecen en tu sesión; tras
confirmar, solo se eleva el gestor nativo si el ámbito de instalación lo
requiere. En Linux los gestores del sistema solicitan sudo/pkexec; AUR, Pamac,
Flatpak de usuario, Brew, Nix y Guix mantienen su contexto y su mecanismo de
autorización nativo. En Windows, `--elevate` o
la preferencia de Ajustes permite elevar winget/Chocolatey; Scoop permanece siempre en el perfil
del usuario.

Nunca se eleva globalmente: Git/GitHub, Wine/Proton, automatizaciones,
aliases, SSH/SCP/SFTP, ADB, Kubernetes y operaciones de Docker/Podman que
deben conservar el contexto elegido. Esto evita cambiar de daemon, dispositivo,
configuración o credenciales, y crear archivos root en el perfil personal.
La papelera del usuario tampoco se eleva; las eliminaciones se envían al
contenedor de papelera con la identidad del usuario.
La elevación no convierte una ruta inválida en válida ni salta las
confirmaciones, objetivos explícitos, planes o bloqueos de seguridad.

La GUI muestra la política en Ajustes y la aplica al pulsar una acción; la CLI
acepta las mismas opciones globales. Las opciones, nombres de rutas y
autorizadores son nativos de cada sistema: sudo/pkexec y systemd en Linux,
UAC/NSudo, PowerShell y servicios Windows en Windows.
"#;

const DIAGNOSTICS: &str = r#"GUÍA: DIAGNÓSTICO Y DEPENDENCIAS

`doctor` verifica herramientas, versiones, permisos y alternativas. `diagnostics
health` reúne salud del anfitrión; `capabilities --format json` publica el
contrato para frontends. Son consultas y no instalan nada por sí solas.

Si una acción necesita una dependencia, LTools muestra cuál, qué gestor y qué
comando propone. La instalación es independiente, explícita y confirmada.
"#;

const DEFAULTS: &str = r#"GUÍA: VALORES PREDETERMINADOS Y AJUSTES

`defaults` muestra rutas y configuración efectiva sin modificarla. Ajusta
idioma, tema, visibilidad y opciones desde Ajustes o variables documentadas.
Los valores por defecto evitan rutas críticas; objetivos de almacenamiento,
usuarios, servicios, red, EFI y prefijos siempre deben introducirse de forma
explícita.
"#;

const AUDIT: &str = r#"GUÍA: AUDITORÍA E INVENTARIO

`audit` recopila inventarios, espacio, archivos grandes, duplicados y caches;
`games` localiza juegos y prefijos; `packages` revisa gestores. Son consultas:
no limpian, no instalan y no modifican prefijos.

Usa filtros, exporta informes y después abre la acción de gestión concreta.
Antes de limpiar o migrar, revisa la ruta, el tamaño y el plan de la familia
correspondiente.
"#;

const CLEAN: &str = r#"GUÍA: LIMPIEZA

Empieza por `clean --preview`. Selecciona paquete, ruta, huérfanos o caché de
un gestor; el preview enumera candidatos y excluye rutas críticas. Ejecuta
solo con un objetivo revisado, confirmación y plan. La limpieza no elimina
datos de usuario por defecto y una ruta ambigua se rechaza.

Para liberar espacio de forma guiada usa `clean --automatic --preview`. Muestra
el tamaño de cachés regenerables, temporales conocidos, cachés de paquetes,
papelera, aplicaciones y datos personales detectables. Ejecuta después
`clean --automatic`; por cada categoría puedes aceptar o conservar, y luego
elegir borrar todo o revisar elemento por elemento. Descargas, Documentos,
Escritorio, Imágenes, Vídeos y Música solo aparecen con
`--include-personal`/`--all-known`, y `--ask-each` fuerza la revisión individual.
Las aplicaciones se inventarían, pero no se borran como carpetas: se
desinstalan desde el gestor nativo. El resumen separa potencial, selección y
espacio liberado; lo enviado a la papelera sigue ocupando espacio hasta
vaciarla.
"#;

const UTILITIES: &str = r#"GUÍA: UTILIDADES

Consulta el estado de utilidades detectadas y usa la instalación contextual
solo cuando una acción lo necesita. La GUI muestra el paquete y gestor antes
de instalar; no hay instalaciones silenciosas. Para una automatización,
comprueba disponibilidad y versión con `doctor` y registra el resultado.
"#;

const ACTIONS: &str = r#"GUÍA: CATÁLOGO DE ACCIONES

`actions list` es el contrato para botones, CLI y frontends. Cada entrada
indica ID, categoría, argumentos, perfil, mutación, confirmación, alias y
compatibilidad. `actions run ID [objetivo] --dry-run` valida un flujo antes de
aplicarlo; los IDs mutadores conservan un plan y piden confirmación.

No construyas shell a partir de campos del usuario: pasa executable y args[]
por separado. Si un ID no existe en la plataforma actual, el catálogo lo
marca y la acción se rechaza de forma segura.
"#;

const CLI_INDEX: &str = r#"GUÍAS CLI DE LTOOLS

La CLI describe las opciones nativas de LTools, sus argumentos y sus
resultados. No sustituye la guía por comandos internos del sistema: indica
qué operación elegir, qué objetivo necesita y qué protección aplica.

Modos:
  guide cli <familia>  opciones, argumentos y compatibilidad de la CLI
  guide gui <familia>  pasos dentro de la interfaz gráfica
  guide all            índice de todas las familias

Familias: audit, packages, software, git, gh, aliases, automation, clean,
storage, system, services, accounts, network, boot, registry, diagnostics,
wine, defaults, updates, containers, kubernetes, ssh, adb, utilities, actions
y privileges.
"#;

fn cli_guide(topic: &str) -> String {
    if topic == "all" {
        return CLI_INDEX.to_owned();
    }
    if topic == "updates" {
        return UPDATES.to_owned();
    }
    let (title, queries, management, arguments, result) = match topic {
        "git" | "gh" => (
            "GIT Y GITHUB",
            "status, diagnose, log, clone, fetch, pull, branch, tag; gh version/help, auth-status, repo, prs, releases y native help",
            "add, commit, push, repair de índice y comandos GitHub CLI nativos clasificados por versión",
            "--repo, --url, --destination, --remote, --branch, --path, --message, --limit, --yes y argumentos nativos separados",
            "estado, diagnóstico, copia recuperable del índice, historial, cambios preparados o respuesta de gh instalado",
        ),
        "network" | "connectivity" => (
            "RED",
            "status, interfaces, routes, dns, listening y connections",
            "set-interface, connection-up, connection-down y flush-dns",
            "--interface, --state y --connection; el objetivo debe ser exacto",
            "interfaces, rutas, DNS, puertos y perfiles de conexión",
        ),
        "boot" => (
            "ARRANQUE, EFI Y GRUB",
            "status, efi-entries, grub-entries, systemd-boot y secure-boot",
            "plan, set-next y clear-next; no hay reinicio automático",
            "--entry para seleccionar la entrada exacta y --yes solo después de revisar",
            "cargador detectado, entradas disponibles y siguiente arranque",
        ),
        "services" => (
            "SERVICIOS",
            "status, services, failed, journal, processes y export",
            "service con status, start, stop, restart, enable, disable, mask o unmask",
            "--scope system|user|both, --filter, --unit, --operation, --limit y formato",
            "estado, origen, unidad, dependencias, eventos e informe exportable",
        ),
        "storage" | "storage-partitions" | "storage-filesystems" | "storage-volumes" => (
            "ALMACENAMIENTO",
            "status, partitions, mounts, map/tree, usage, filesystems y tools",
            "mapa desplegable de discos y rutas; copiar, mover, borrar a papelera, zip, tar y abrir con el gestor nativo; además de particiones, sistemas de archivos, montajes, swap, cifrado, LVM, Btrfs, ZFS y RAID",
            "--path, --depth, --max-children, --format text|json|tsv, --out, --source, --destination, --device, --mountpoint, --filesystem, --label y --yes",
            "árbol con tamaños acumulados, permisos observados, rutas protegidas, errores de acceso, dispositivos, volúmenes, planes y resultado de la operación",
        ),
        "wine" => (
            "WINE Y PROTON",
            "list e inspect para localizar y revisar prefijos",
            "create y migrate con comprobación de origen y destino",
            "--path, --source, --dest y --arch; no se elimina el origen por defecto",
            "runner, arquitectura, ejecutables, rutas y validación del prefijo",
        ),
        "accounts" => (
            "CUENTAS Y GRUPOS",
            "list, identity, sessions, groups e inspect",
            "create, edit, password, lock, unlock, expire, membership, primary-group, admin-groups y admin-add",
            "--user, --group y objetivo explícito; Linux detecta sudo/wheel/admin existentes y Windows usa el SID Administradores",
            "identidad, sesiones, membresías, grupo administrativo detectado y cambios protegidos",
        ),
        "containers" | "containers-lifecycle" | "containers-images" | "containers-volumes"
        | "containers-compose" => (
            "CONTENEDORES",
            "status, inspect, stats, top, logs, images, volumes, networks y compose",
            "run, start, stop, restart, exec, pause, unpause, kill, rename, cp, prune y remove",
            "--engine, --container, --image, --volume, --network, --ports y --command",
            "motor, recursos, estado, logs y cambios confirmados",
        ),
        "kubernetes" => (
            "KUBERNETES",
            "context, namespace, get, describe, diff y logs",
            "apply, delete, scale, rollout y port-forward",
            "--context, --namespace, --manifest, --resource, --replicas y --local-port",
            "contexto, recursos, rollout, sesión activa y resultado",
        ),
        "clean" => (
            "LIMPIEZA",
            "preview de paquetes, huérfanos, cachés, Flatpak y rutas",
            "remove solo después de revisar candidatos y exclusiones",
            "--automatic, --preview, --include-personal, --all-known, --ask-each; limpieza clásica: --package, --manager, --scope user|system|INSTALACIÓN (Flatpak), --path y --cascade (solo Pacman)",
            "candidatos, rutas protegidas, plan y elementos retirados",
        ),
        "automation" | "automation-register" | "actions" => (
            "AUTOMATIZACIÓN Y ACCIONES",
            "list del registro y del catálogo declarativo",
            "add, modify, run, remove y ejecución por identificador estable",
            "--name, --program, --args, --cwd, --id, --target, --format y --dry-run",
            "acción, compatibilidad, requisitos, plan y salida registrada",
        ),
        "privileges" => (
            "ELEVACIÓN Y PERMISOS",
            "privileges para consultar el catálogo y saber si una acción es solo lectura",
            "--elevate, --no-elevate y la preferencia de Ajustes para acciones compatibles",
            "opciones globales antes o después de la acción; no cambian objetivos ni confirmaciones",
            "clasificación ReadOnly, Optional, Required o Never y el autorizador nativo",
        ),
        "aliases" | "alias" => (
            "GESTOR DE ALIAS",
            "list para ver alias, path para localizar el registro y doctor para revisar la instalación",
            "ensure crea el registro base y el lanzador; add, enable, disable y remove gestionan alias personalizados",
            "add NOMBRE COMANDO [ARG...]; los argumentos se conservan separados y solo se permiten comandos LTools conocidos",
            "registro persistente, alias activos, lanzador gestionado y estado de PATH",
        ),
        "diagnostics" | "utilities" | "defaults" | "settings" | "audit" | "packages" | "software" | "installable" | "system"
        | "registry" | "ssh" | "adb" => (
            "SISTEMA Y HERRAMIENTAS",
            "status, list, inspect, health, report, capabilities y export",
            "consulta, selección, instalación explícita, exportación y gestión contextual",
            "--format, --limit, --scope, --path, --target, --manager y --dry-run",
            "capacidades, versiones, límites, informe y compatibilidad de plataforma",
        ),
        _ => (
            "FAMILIA",
            "consulta, estado, listado e inspección",
            "gestión disponible en la categoría seleccionada",
            "--target, --format, --limit, --dry-run y --plan",
            "resultado estructurado y estado de la operación",
        ),
    };
    format!(
        "GUÍA CLI: {title}\n\n\
         Opciones de consulta:\n  {queries}\n\n\
         Opciones de gestión:\n  {management}\n\n\
         Argumentos nativos de LTools:\n  {arguments}\n\n\
         Resultado esperado:\n  {result}\n\n\
         Flujo seguro: consulta primero, revisa el objetivo y usa --dry-run o\n\
         --plan antes de una mutación. Las confirmaciones, permisos y límites\n\
         dependen de la plataforma; no se ejecutan cadenas de shell ni se\n\
         inventan equivalencias entre Linux y Windows."
    )
}

#[cfg(not(windows))]
const GUI_INDEX: &str = r#"GUÍA GRÁFICA COMPLETA DE LTOOLS

Esta guía es el mapa de la ventana. Cada categoría abre un menú propio y cada
menú puede contener consultas, acciones, submenús y una guía contextual.
Las guías contextuales deben enumerar todas las opciones de la pantalla donde
se abren; no son listas parciales ni una sustitución por comandos.

Panel principal:
  • Auditoría e inventario: auditoría, juegos, paquetes y prefijos.
  • Herramientas nativas: almacenamiento y particiones; sistema, red,
    seguridad, usuarios, arranque, Registro, diagnóstico y servicios.
  • Dependencias: doctor, herramientas detectadas, instalación y diagnóstico.
  • Valores predeterminados: rutas y Registro.
  • Herramientas instalables: Git/GitHub, paquetes, conectividad, Docker,
    Kubernetes y utilidades.
  • Automatización: scripts registrados y registrar nuevo script.
  • Ajustes: incluye la casilla «Elevar acciones modificadoras por defecto».
    Las operaciones que requieren permisos informan y piden autenticación;
    consultas, Git/Wine, automatizaciones y papelera conservan el usuario.

Gestor de alias (solo CLI): `ltools aliases` permite list, ensure, doctor,
add, enable, disable, remove, path y shell-init. No hay un menú gráfico para
estas operaciones.

Menú completo «Auditar / Inventariar»:
  • «Volver».
  • «Auditar discos y aplicaciones».
  • «Inventariar juegos y lanzadores».
  • «Inventario de paquetes».
  • «Prefijos Wine/Proton».
  • «Gestionar paquetes y almacenes».
  • «Guía general de uso».

Cada pantalla conserva «Volver» y las acciones mutables abren confirmación.
Para una pantalla concreta, abre su botón «Guía»; allí deben aparecer todos
los botones de esa pantalla, sus campos y el flujo de verificación.
La CLI tiene una guía separada con sus propias opciones: guide cli <familia>.
"#;

#[cfg(not(windows))]
fn gui_storage_guide() -> String {
    r#"GUÍA GRÁFICA: ALMACENAMIENTO Y PARTICIONES (LINUX)

Esta es la referencia completa del menú. La guía indica dónde entrar, qué
botón pulsar, qué campos aparecen y cómo verificar el resultado. Las consultas
no modifican el equipo. Las acciones
que pueden borrar, formatear, desmontar o cambiar volúmenes muestran una
confirmación antes de ejecutarse.

1. Menú completo «Almacenamiento y particiones» (dentro de «Herramientas nativas»)

  • «Resumen de espacio y montajes»: consulta espacio, dispositivos,
    sistemas de archivos y montajes activos.
  • «Discos y particiones»: lista discos y particiones detectados.
  • «Montajes activos»: muestra qué está montado y dónde.
  • «Mapa desplegable de discos y rutas»: abre el árbol de carpetas y
    archivos con tamaño acumulado, permisos observados, errores de acceso y
    explicación de rutas estándar. En cada raíz muestra capacidad total,
    espacio ocupado, espacio libre total y espacio disponible para la cuenta;
    el tamaño calculado del árbol es una medida separada. Mientras se calcula
    muestra el contador de rutas y una barra de actividad. Al terminar,
    «Expandir todo» y «Colapsar todo» controlan el árbol; si quedan rutas
    bloqueadas, «Reintentar como administrador» solicita autorización al
    sistema y repite el análisis con los permisos concedidos. «Cerrar» finaliza
    la vista. Al seleccionar una fila aparecen «Copiar seleccionada», «Mover
    seleccionada» y «Enviar seleccionada a la papelera»: copiar y mover piden
    el destino, y las tres acciones reutilizan el gestor seguro de archivos,
    su confirmación, su plan y sus bloqueos de rutas críticas.
  • «Particionado y tablas»: abre todas las operaciones de tabla y
    partición descritas en la sección 2.
  • «Sistemas de archivos»: abre formato, etiquetas, comprobación, montaje
    y swap descritos en la sección 3.
  • «Cifrado y volúmenes»: abre las funciones de cifrado, volúmenes y RAID.
  • «Herramientas detectadas»: comprueba qué capacidades
    están realmente disponibles en este equipo.
  • «Explicar ruta y permisos»: explica el propósito, propietario, permisos
    y protección de la ruta seleccionada antes de modificarla.
  • «Borrar a la papelera»: mueve un archivo o carpeta a la papelera del
    usuario; no elimina silenciosamente ni eleva la acción.
  • «Copiar archivo o carpeta» y «Mover archivo o carpeta»: piden origen y
    destino explícitos, muestran el plan y conservan la protección de rutas.
  • «Crear archivo ZIP» y «Crear archivo TAR»: empaquetan la selección en un
    destino indicado y verifican el archivo resultante.
  • «Abrir con el gestor nativo»: abre el explorador/gestor de archivos del
    sistema para revisar la ruta con sus propias garantías.
  • «Abrir gestor nativo de particiones»: abre el gestor gráfico externo
    disponible para una revisión visual adicional; sus cambios siguen siendo
    cambios reales y deben verificarse después en LTools.
  • «Revisar limpieza»: solo prepara una revisión de candidatos; no borra
    discos ni datos por sí solo.
  • «Limpiador automático guiado»: calcula cachés, temporales y rutas
    conocidas, permite seleccionar categorías y pregunta antes de borrar.
  • «Guía de particionado y protecciones»: vuelve a esta explicación.
  • «Volver»: regresa al menú anterior; aparece también en cada submenú.

2. «Particionado y tablas»

Consultas y diagnóstico:
  • «Consultar tabla de un disco»: campo «Disco completo /dev/...».
    Muestra tabla, particiones, tamaños, tipos y flags.
  • «Consultar espacio libre»: mismo campo; localiza huecos sin asignar antes
    de crear o ampliar una partición.
  • «Inspeccionar dispositivo»: mismo campo; verifica que el objetivo
    sea el disco esperado antes de tocarlo.
  • «Comprobar alineación»: «Disco completo /dev/...», «Número de partición»
    y «Alineación: minimal u optimal». Es solo lectura y no requiere
    confirmación destructiva.

Crear y modificar la tabla:
  • «Crear tabla GPT» o «Crear tabla MBR / msdos»: campo «Dispositivo
    /dev/...». Sustituir una tabla puede hacer inaccesibles todas sus
    particiones; úsalo solo con una copia y un disco confirmado.
  • «Crear partición»: «Disco completo /dev/...», «Tipo: primary, logical o
    extended», «Sistema de archivos», «Inicio (ej. 1MiB)», «Fin (ej. 100%)»
    y «Nombre GPT». El tipo, inicio y fin determinan el espacio ocupado.
  • «Borrar partición»: «Disco completo /dev/...» y «Número de partición».
    El botón elimina la entrada de la tabla; no lo uses para liberar una
    partición sin comprobar antes sus montajes y copias.
  • «Redimensionar partición»: disco, número y «Nuevo fin (ej. 100%)».
    Primero comprueba el sistema de archivos y deja espacio para el objetivo.
  • «Nombrar partición GPT»: disco, número y «Nombre GPT». Solo cambia el
    nombre GPT, no el contenido del sistema de archivos.
  • «Activar / desactivar flag»: disco, número, «Flag (boot, esp, lvm...)» y
    «Estado: on u off». Usa «esp» para la partición EFI solo cuando el diseño
    del equipo lo requiera.
  • «Cambiar flag del disco» y «Alternar flag del disco»: disco, «Flag de
    disco» y, para la primera, «Estado: on u off». Revisa el plan porque el
    flag pertenece al disco y no a una partición individual.
  • «Buscar y rescatar partición»: disco, «Inicio de búsqueda» y «Fin de
    búsqueda». Es una operación de recuperación: nunca elijas rangos a ciegas.
  • «Retirar firmas de almacenamiento»: campo «Dispositivo /dev/...». Retira firmas que
    identifican sistemas de archivos o volúmenes y puede destruir metadatos.
  • «Descartar bloques»: campo «Dispositivo /dev/...». Solicita al medio que
    descarte bloques; no lo uses como limpieza genérica ni en un objetivo
    dudoso.

Flujo seguro para crear una partición:
  1. Pulsa «Consultar tabla de un disco», «Consultar espacio libre» e
     «Inspeccionar dispositivo».
  2. Comprueba que el disco no contiene «/», «/home», «/boot», la partición
     EFI ni un volumen que estés usando.
  3. Ejecuta «Comprobar alineación» si vas a reutilizar el espacio.
  4. Pulsa «Crear partición», completa todos los campos y revisa el plan.
  5. Confirma solo si el disco, inicio, fin y tipo son exactamente los tuyos.
  6. Vuelve a «Consultar tabla» y continúa en «Sistemas de archivos».

3. «Sistemas de archivos»

  • «Crear / formatear sistema de archivos»: «Partición /dev/...», «Tipo
    (ext4, btrfs, xfs, ntfs, vfat...)» y «Etiqueta (opcional)». Formatear
    destruye el contenido de la partición.
  • «Cambiar etiqueta»: partición, «Tipo de sistema de archivos» y «Nueva
    etiqueta». Verifica el tipo para no aplicar una herramienta incorrecta.
  • «Comprobar sin reparar»: partición. Úsalo primero para conocer errores sin
    modificar el sistema de archivos.
  • «Comprobar y reparar automáticamente»: partición. Desmonta antes cuando
    proceda y conserva una copia: la reparación puede cambiar metadatos.
  • «Redimensionar sistema de archivos»: «Dispositivo /dev/... o montaje
    absoluto», «Tipo ext4, xfs, btrfs o ntfs» y «Nuevo tamaño (opcional)».
    El tamaño del sistema de archivos debe ser compatible con el de la
    partición; redimensionar solo una de las dos capas puede dejar datos
    inaccesibles.
  • «Montar partición»: «Partición /dev/...» y «Punto de montaje absoluto».
    Comprueba que el directorio destino sea el correcto y que no oculte datos
    ya existentes.
  • «Desmontar dispositivo o ruta»: «Dispositivo o ruta absoluta». Cierra
    archivos y comprueba usuarios antes de desmontar.
  • «Activar swap» y «Desactivar swap»: campo «Dispositivo /dev/...». Antes de
    desactivar, confirma que no sea el único espacio de intercambio necesario.

Flujo para preparar y montar una partición:
  1. Crea la partición y verifica la tabla.
  2. Pulsa «Crear / formatear sistema de archivos», revisa dispositivo, tipo
     y etiqueta, y confirma solo con copia de seguridad.
  3. Pulsa «Comprobar sin reparar» y corrige cualquier problema antes de
     montar.
  4. Crea o elige un punto de montaje absoluto y pulsa «Montar partición».
  5. Vuelve a «Resumen de espacio y montajes» y «Montajes activos».
  6. Si el resultado es correcto, documenta la etiqueta y el punto de
     montaje; si no, desmonta y corrige el objetivo antes de repetir.

Flujo avanzado — recuperación de particiones:
  1. Identifica primero el disco completo y su tabla; conserva una copia
     externa. No confundas el disco completo con una de sus particiones.
  2. Para rescatar, usa «Buscar y rescatar partición» con límites medidos;
     no escribas una tabla nueva ni formatees antes de verificar resultados.
  3. Antes de cambiar un flag de partición/disco, comprueba el firmware y el
     propósito del flag: uno incorrecto puede impedir el arranque.
  4. Revisa la confirmación completa y vuelve a consultar la tabla. La búsqueda
     no convierte el hallazgo en restauración automática.

4. «Cifrado y volúmenes»

LUKS:
  • «Crear contenedor LUKS»: «Dispositivo /dev/...». Inicializa cifrado y
    requiere contraseña; verifica el dispositivo porque el contenido anterior
    deja de estar disponible.
  • «Abrir contenedor LUKS»: «Dispositivo /dev/...» y «Nombre del mapeo».
    Después comprueba el mapeo y, si procede, el volumen que contiene.
  • «Cerrar contenedor LUKS»: «Nombre del mapeo». Desmonta primero los sistemas
    de archivos dependientes.
  • «Copiar cabecera LUKS» / «Restaurar cabecera LUKS»: dispositivo y «Archivo
    de cabecera». Guarda la copia en un medio seguro; restaurar una cabecera
    equivocada puede inutilizar el volumen.

Capas de volumen:
  • «Gestionar volúmenes LVM». «Acción» ofrece: «Preparar dispositivo para
    volúmenes», «Retirar dispositivo de volúmenes», «Crear grupo de volúmenes»,
    «Eliminar grupo de volúmenes», «Crear volumen lógico», «Eliminar volumen
    lógico», «Ampliar volumen lógico» y «Reducir volumen lógico». Los demás
    campos piden dispositivo, grupo, nombre y tamaño según la acción.
  • «Gestionar volúmenes Btrfs». «Acción» ofrece: «Crear subvolumen», «Eliminar
    subvolumen», «Crear instantánea», «Cambiar tamaño del sistema de archivos»,
    «Reequilibrar datos», «Agregar dispositivo», «Retirar dispositivo»,
    «Reemplazar dispositivo» y «Comprobar sin modificar». Los campos restantes
    describen el montaje, destino, dispositivo secundario y tamaño.
  • «Gestionar volúmenes ZFS». «Acción» ofrece: «Crear conjunto de
    almacenamiento», «Eliminar conjunto de almacenamiento», «Exportar
    conjunto», «Importar conjunto», «Crear conjunto de datos», «Eliminar
    conjunto de datos», «Crear instantánea», «Comprobar conjunto», «Cambiar
    propiedad», «Renombrar conjunto de datos» y «Volver a una instantánea».
    El formulario incluye nombre, dispositivos, propiedad, valor y nuevo nombre.
  • «Gestionar conjuntos RAID». «Acción» ofrece: «Consultar estado del
    conjunto», «Inspeccionar dispositivo miembro», «Reunir un conjunto
    existente», «Crear conjunto nuevo», «Agregar dispositivo miembro»,
    «Reemplazar dispositivo miembro», «Retirar dispositivo miembro», «Marcar
    miembro como fallido», «Volver a agregar miembro», «Detener conjunto»,
    «Comprobar consistencia», «Reparar conjunto» y «Cambiar cantidad de
    miembros». Completa solo los campos pertinentes: conjunto, miembros, nivel,
    sustituto o cantidad. Crear un conjunto puede sobrescribir los datos de los
    dispositivos seleccionados.

Proceso simple — revisar discos:
  1. Pulsa «Resumen de espacio y montajes», «Discos y particiones» y
     «Montajes activos».
  2. Comprueba dispositivo, tamaño, tipo, etiqueta, espacio libre y destino.
  3. No uses una acción de escritura hasta identificar el objetivo exacto.

Proceso complejo — cifrado + volumen + montaje:
  1. Inspecciona disco, particiones y herramientas disponibles.
  2. Haz copia de la cabecera si ya existe un LUKS y confirma el objetivo.
  3. Crea o abre LUKS; no introduzcas contraseñas en capturas o informes.
  4. Crea el volumen LVM/Btrfs/ZFS/RAID que corresponda y verifica su estado.
  5. Crea o comprueba el sistema de archivos, móntalo y vuelve a consultar
     espacio y montajes.
  6. Si una capa falla, detente: no repitas una operación destructiva para
     «probar». Desmonta y revierte solo con una copia y un plan confirmado.

5. Reglas de verificación

  • Para cualquier acción: revisa el resumen de confirmación, cancela si el
    objetivo no coincide y conserva la salida de la acción.
  • Tras una mutación: vuelve a la consulta equivalente y comprueba dispositivo,
    tamaño, tipo, etiqueta, estado y montaje.
  • No formatees, borres firmas, cambies tablas, reduzcas volúmenes ni
    desmontes la raíz/EFI mientras el sistema dependa de ellos.
  • El «gestor nativo» y los cambios hechos fuera de LTools deben verificarse
    volviendo a estas consultas; abrir la herramienta no equivale a completar
    ni validar una operación.
"#
    .to_owned()
}

#[cfg(windows)]
fn gui_storage_guide() -> String {
    r#"GUÍA GRÁFICA: ALMACENAMIENTO Y PARTICIONES (WINDOWS)

La GUI de Windows muestra únicamente operaciones compatibles con discos,
particiones, volúmenes y letras de unidad Windows. No ofrece los flujos Linux
de LUKS, LVM, Btrfs, ZFS, mdadm, /dev/... o montajes POSIX.

En «Herramientas nativas»:
  • «Resumen de espacio y montajes» revisa espacio y volúmenes.
  • «Discos y particiones» lista discos, particiones, volúmenes y letras.
  • «Montajes activos» revisa las rutas y letras actualmente disponibles.
  • El mapa desplegable de archivos y rutas todavía no está integrado en la
    GUI Windows. No se presenta como un botón disponible; esta pantalla ofrece
    las consultas de volúmenes y particiones enumeradas arriba.
  • «Abrir el gestor nativo de particiones» abre Administración de discos; si
    se ofrece «DiskPart», úsalo solo con una selección y confirmación exactas.
  • «Guía de particionado y protecciones» vuelve a esta explicación.
  • «Volver»: regresa al menú anterior.

Flujo simple — inspeccionar:
  1. Abre «Discos y particiones» y localiza el número de disco, partición,
     volumen, tamaño, formato y letra.
  2. Comprueba que no sea el volumen de arranque, recuperación o sistema.
  3. Abre «Montajes activos» y verifica qué letra o ruta usa cada volumen.

Flujo complejo — preparar un volumen:
  1. Abre el gestor nativo y selecciona el disco exacto por número y tamaño.
  2. Revisa el espacio no asignado y crea, elimina, formatea o asigna una
     letra únicamente después de confirmar el objetivo.
  3. No conviertas el disco ni cambies la tabla si no tienes copia y una
     recuperación preparada.
  4. Cierra el gestor, vuelve a LTools y repite «Discos y particiones» y
     «Montajes activos» para comprobar el resultado.

Windows usa volúmenes, letras y permisos UAC; no copies a esta guía
procedimientos Linux ni rutas /dev. Las acciones externas también deben
verificarse desde LTools.
"#
    .to_owned()
}

fn gui_settings_guide() -> String {
    let mut options = Vec::new();
    let fields: String;

    #[cfg(windows)]
    {
        fields = format!(
            "Campo «Tema»: valor admitido entre {}. Campo «Idioma»: auto o uno de {}. La página usa casillas para la visibilidad y la elevación; «Aplicar ajustes» guarda los cambios.",
            crate::theme::SUPPORTED.join(", "),
            std::iter::once("auto")
                .chain(crate::i18n::SUPPORTED.iter().copied())
                .collect::<Vec<_>>()
                .join(", "),
        );
        options.push("Campo «Tema»".to_owned());
        options.push("Campo «Idioma»".to_owned());
        options.push(crate::i18n::gui_text("elevation_default").to_owned());
        for category in crate::i18n::SETTINGS_CATEGORY_KEYS {
            options.push(crate::i18n::category_text(category).to_owned());
        }
        options.push(crate::i18n::gui_text("settings_apply").to_owned());
        options.push(crate::i18n::gui_text("update_check").to_owned());
        options.push(crate::i18n::gui_text("update_download").to_owned());
        options.push(crate::i18n::text("menu.back").to_owned());
    }

    #[cfg(not(windows))]
    {
        fields = format!(
            "No hay campos de texto. Temas: {}. Idiomas: {}. Las categorías y la elevación son casillas; «{}» abre esta guía.",
            crate::theme::SUPPORTED
                .iter()
                .map(|id| crate::theme::label(id))
                .collect::<Vec<_>>()
                .join(", "),
            std::iter::once("auto")
                .chain(crate::i18n::SUPPORTED.iter().copied())
                .map(crate::i18n::language_label)
                .collect::<Vec<_>>()
                .join(", "),
            crate::i18n::gui_text("settings_guide"),
        );
        options.push(crate::i18n::text("menu.back").to_owned());
        options.extend(
            crate::theme::SUPPORTED
                .iter()
                .map(|id| crate::theme::label(id).to_owned()),
        );
        options.extend(
            std::iter::once("auto")
                .chain(crate::i18n::SUPPORTED.iter().copied())
                .map(crate::i18n::language_label)
                .map(str::to_owned),
        );
        options.extend(
            crate::i18n::SETTINGS_CATEGORY_KEYS
                .iter()
                .map(|category| crate::i18n::category_text(category).to_owned()),
        );
        options.push(crate::i18n::gui_text("elevation_default").to_owned());
        options.push(crate::i18n::gui_text("settings_guide").to_owned());
        options.push(crate::i18n::gui_text("update_check").to_owned());
        options.push(crate::i18n::gui_text("update_download").to_owned());
    }

    let listed = options
        .iter()
        .enumerate()
        .map(|(index, option)| format!("  {}. «{}»", index + 1, option))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
            "GUÍA GRÁFICA: AJUSTES\n\nMenú completo «{}»:\n{}\n\nCampos y argumentos de la GUI:\n  {}\n\nProceso simple:\n  Cambia una preferencia cada vez, aplica o guarda los ajustes y comprueba que el valor elegido siga disponible. «{}» consulta la release estable; «{}» descarga en Descargas solo si la firma/hash son válidos.\n\nProceso complejo:\n  Configura tema e idioma, decide qué categorías mostrar y revisa la política de elevación antes de guardar. Al instalar software, la búsqueda, selección del paquete y confirmación permanecen en tu sesión; solo se eleva el gestor nativo cuando el ámbito lo requiere. Para actualizar, comprueba primero la versión y plataforma, descarga el paquete verificado, cierra LTools, conserva una copia anterior y sigue el procedimiento portable de Linux o Windows. No se eleva, ejecuta ni reemplaza automáticamente el binario.\n\nLas opciones enumeradas corresponden al formulario de esta plataforma; «{}» regresa a la pantalla anterior.",
        crate::i18n::gui_text("settings_title"), listed, fields,
        crate::i18n::gui_text("update_check"), crate::i18n::gui_text("update_download"),
        crate::i18n::text("menu.back")
    )
}

fn gui_updates_guide() -> String {
    let listed = [
        crate::i18n::gui_text("update_check"),
        crate::i18n::gui_text("update_download"),
        crate::i18n::text("menu.back"),
    ]
    .iter()
    .enumerate()
    .map(|(index, label)| format!("  {}. «{}»", index + 1, label))
    .collect::<Vec<_>>()
    .join("\n");
    #[cfg(windows)]
    let platform_flow = "Windows: el perfil GUI descarga el EXE GUI; el perfil CLI descarga el EXE CLI. Revisa la ruta portable y cierra LTools antes de reemplazarlo.";
    #[cfg(not(windows))]
    let platform_flow = "Linux: desde AppImage se descarga otro AppImage; desde instalación tarball/binario se descarga el tarball portable. Conserva el lanzador y los permisos adecuados al sustituirlo.";
    format!(
        "GUÍA GRÁFICA: ACTUALIZACIONES\n\nMenú completo «{}»:\n{}\n\nCampos y argumentos de la GUI:\n  No hay campos. Comprobar usa la release estable predeterminada; descargar selecciona plataforma y perfil del ejecutable actual.\n\nProceso simple:\n  Pulsa «{}» y revisa versión instalada, versión publicada, compatibilidad e integridad. Esta consulta necesita Internet, pero no cambia archivos.\n\nProceso complejo:\n  Pulsa «{}» solo si hay una versión posterior. Se valida la firma Ed25519, el checksum del manifiesto, el hash y tamaño del paquete y se guarda en Descargas sin sobrescribir. Después cierra LTools, conserva una copia y reemplaza el artefacto manualmente. {} Nunca se eleva, ejecuta ni sustituye por sí sola la instalación.\n\n«{}» vuelve a Ajustes.",
        crate::i18n::category_text("settings"), listed,
        crate::i18n::gui_text("update_check"), crate::i18n::gui_text("update_download"),
        platform_flow, crate::i18n::text("menu.back")
    )
}

#[cfg(windows)]
fn windows_gui_topic_page(topic: &str) -> Option<(usize, &'static str, &'static str)> {
    match topic {
        "audit" | "packages" => Some((0, "audit_inventory", "AUDITORÍA E INVENTARIO")),
        "native" | "system" | "network" | "boot" | "registry" => {
            Some((1, "native_tools", "HERRAMIENTAS NATIVAS"))
        }
        "diagnostics" => Some((2, "dependencies", "DEPENDENCIAS")),
        "defaults" => Some((3, "defaults", "RUTAS PREDETERMINADAS")),
        "installable" | "software" | "git" | "containers" | "kubernetes" | "adb" => {
            Some((4, "installable_tools", "HERRAMIENTAS INSTALABLES"))
        }
        "automation" | "automation-register" => Some((5, "automation", "AUTOMATIZACIÓN")),
        "accounts" => Some((8, "accounts", "USUARIOS, GRUPOS Y SESIONES")),
        "winslim" => Some((7, "winslim", "WINSLIM")),
        "updates" => Some((6, "settings", "ACTUALIZACIONES")),
        _ => None,
    }
}

#[cfg(windows)]
fn windows_gui_unavailable(topic: &str) -> String {
    let detail = match topic {
        "clean" => "La limpieza guiada no tiene un panel propio en esta GUI Windows.",
        "services" => "La gestión detallada de servicios no tiene botones en esta GUI Windows.",
        "containers-lifecycle" | "containers-images" | "containers-volumes"
        | "containers-compose" => "La GUI Windows solo ofrece una consulta del estado Docker/Podman. El ciclo de vida, imágenes, volúmenes, redes y Compose no tienen acciones gráficas en este ejecutable.",
        "ssh" | "connectivity" => "SSH/SCP/SFTP no tienen acciones gráficas en este ejecutable. El menú de Herramientas instalables solo ofrece el estado de ADB; no anuncia transferencias SSH inexistentes.",
        "utilities" => "El catálogo de utilidades instalables no tiene una pantalla propia en esta GUI Windows.",
        "aliases" => "El gestor de alias se administra desde la CLI y no tiene una pantalla gráfica Windows.",
        "winslim" => "Este anfitrión no tiene disponible la pantalla WinSlim/NSudo; solo aparece en Windows cuando existe C:\\WSCore o se detecta un lanzador NSudo compatible.",
        "wine" | "prefix" => "Wine y Proton no aplican al ejecutable Windows nativo y no tienen pantalla gráfica aquí.",
        "storage-partitions" | "storage-filesystems" | "storage-volumes" => "Las operaciones Linux de parted, sistemas de archivos POSIX, LUKS, LVM, Btrfs, ZFS y RAID no se implementan en la GUI Windows.",
        _ => "Este tema no tiene una pantalla propia en la GUI Windows.",
    };
    format!(
        "GUÍA GRÁFICA WINDOWS: {topic}\n\n{detail}\n\nNo se inventan botones ni se muestran pasos de la GUI Linux. Para opciones gráficas reales, abre «Herramientas nativas», «Dependencias», «Rutas predeterminadas», «Herramientas instalables», «Automatización» o «Ajustes» según el objetivo; el índice `guide gui all` enumera sus botones reales."
    )
}

#[cfg(windows)]
fn windows_gui_page_guide(topic: &str) -> Option<String> {
    if topic == "settings" {
        return Some(gui_settings_guide());
    }
    if topic == "winslim" && !crate::platform::winslim_available() {
        return None;
    }
    let (page, category, title) = windows_gui_topic_page(topic)?;
    let title = if topic == "accounts" {
        crate::i18n::accounts_label()
    } else {
        title
    };
    let menu = crate::gui::windows_menu_title(page)
        .unwrap_or_else(|| crate::i18n::category_text(category).to_owned());
    let mut options = crate::gui::windows_menu_labels(page);
    options.push(crate::i18n::text("menu.back").to_owned());
    let listed = options
        .iter()
        .enumerate()
        .map(|(index, option)| format!("  {}. «{}»", index + 1, option))
        .collect::<Vec<_>>()
        .join("\n");
    let (fields, simple, complex) = match page {
        1 => (
            "Las consultas no piden campos. «Abrir gestor nativo de particiones» solicita confirmación; «Usuarios, grupos y sesiones» abre el formulario especializado.",
            "Consulta espacio, volúmenes y estado antes de elegir una acción; vuelve a la consulta correspondiente para comprobar el resultado.",
            "Para cuentas, abre su submenú, identifica exactamente el usuario o grupo, revisa la confirmación y vuelve a listar. Las acciones de Disco/Registro solo se realizan en su gestor nativo tras revisar el objetivo.",
        ),
        5 => (
            "Registrar/editar solicita nombre, ejecutable, directorio de trabajo y argumentos separados; no se evalúa una cadena de shell.",
            "Abre «Scripts registrados» y revisa el nombre, el programa y el estado antes de ejecutar.",
            "Registra un ejecutable Windows con cada argumento separado, inspecciona el registro, prueba el flujo y retira únicamente la entrada seleccionada.",
        ),
        6 => (
            "No hay campos. «Comprobar actualizaciones» consulta GitHub; «Descargar actualización verificada» descarga únicamente una versión más reciente al perfil del usuario.",
            "Comprueba versión actual, última versión y el paquete que coincide con este ejecutable.",
            "Descarga solo tras revisar que el paquete corresponde a Windows y a GUI/CLI. La firma Ed25519 y los hashes deben validarse; cierra el programa antes de reemplazar el ejecutable y conserva una copia. No se eleva ni se instala automáticamente.",
        ),
        8 => (
            crate::i18n::gui_account_text("guide_fields"),
            crate::i18n::gui_account_text("guide_simple"),
            crate::i18n::gui_account_text("guide_complex"),
        ),
        7 => (
            if crate::platform::nsudo_path().is_some() {
                "No hay campos para las consultas. «Abrir el asistente de lanzamiento NSudo» abre una consola guiada; allí eliges identidad, ejecutable y argumentos por separado."
            } else {
                "No hay campos. NSudo no está detectado, así que el botón de lanzamiento no se muestra; la elevación normal sigue usando UAC."
            },
            "Consulta la detección de WSCore y NSudo antes de abrir el asistente.",
            "Si NSudo está disponible, abre el asistente, selecciona usuario actual/elevado, SYSTEM, TrustedInstaller, token actual o reducción de privilegios; introduce el programa y cada argumento, revisa la confirmación y repite la consulta de estado. Para automatizarlo, registra el comando `winslim launch` con argumentos separados y `--yes` únicamente tras validar el efecto.",
        ),
        _ => (
            "Este menú no pide campos; sus consultas se ejecutan con valores nativos de Windows.",
            "Abre una consulta y comprueba que la salida corresponde al anfitrión Windows.",
            "Revisa la familia, confirma la operación si es modificadora y repite una consulta equivalente para verificar el resultado.",
        ),
    };
    let account_sid = if page == 8 {
        "\n\nBuilt-in Administrators group SID: S-1-5-32-544."
    } else {
        ""
    };
    Some(format!(
        "GUÍA GRÁFICA: {title}\n\nMenú completo «{menu}» (opciones obtenidas del catálogo real de botones Windows):\n{listed}\n\nCampos y argumentos de la GUI:\n  {fields}{account_sid}\n\nProceso simple:\n  {simple}\n\nProceso complejo:\n  {complex}\n\nLas opciones enumeradas corresponden a esta ventana Windows; «{}» vuelve a la pantalla anterior. No se muestran acciones Linux ni se gestionan prefijos Wine/Proton desde el ejecutable Windows.",
        crate::i18n::text("menu.back")
    ))
}

#[cfg(windows)]
fn windows_gui_index() -> String {
    let mut pages = vec![
        ("audit", 0),
        ("native", 1),
        ("diagnostics", 2),
        ("defaults", 3),
        ("installable", 4),
        ("automation", 5),
        ("accounts", 8),
    ];
    if crate::platform::winslim_available() {
        pages.push(("winslim", 7));
    }
    let mut result = String::from(
        "GUÍA GRÁFICA COMPLETA DE WINSLIM-TOOLS\n\nCada categoría abre un menú propio. Las guías contextuales enumeran las opciones de la interfaz Windows y no sustituyen sus botones por comandos.\n\nPanel principal:\n",
    );
    for (topic, page) in pages {
        let Some((_, category, _)) = windows_gui_topic_page(topic) else {
            continue;
        };
        let title = crate::gui::windows_menu_title(page)
            .unwrap_or_else(|| crate::i18n::category_text(category).to_owned());
        let options = crate::gui::windows_menu_labels(page)
            .into_iter()
            .chain(std::iter::once(crate::i18n::text("menu.back").to_owned()))
            .collect::<Vec<_>>();
        result.push_str(&format!("\n{}:\n", title));
        for option in options {
            result.push_str(&format!("  • «{}»\n", option));
        }
        if page == 1 {
            result.push_str(
                "  • «Usuarios, grupos y sesiones» abre el submenú de gestión de cuentas.\n",
            );
        }
    }
    result.push_str(&format!(
        "\n{}:\n  Tema, idioma, elevación por defecto, visibilidad de estas categorías, aplicar ajustes y volver.\n  WinSlim solo aparece cuando el anfitrión ofrece WSCore.\n\nEl gestor de alias está disponible únicamente por CLI; no se anuncia como botón gráfico. El ejecutable Windows nativo tampoco ofrece el gestor de prefijos Wine/Proton ni opciones Linux. La guía de cada categoría vuelve a enumerar sus botones, argumentos y flujos.",
        crate::i18n::gui_text("settings_button")
    ));
    result.push_str("\n\n");
    result.push_str(&gui_settings_guide());
    result
}

fn gui_catalog_guide(topic: &str) -> String {
    if topic == "settings" {
        return gui_settings_guide();
    }
    if topic == "accounts" {
        let options = [
            "list",
            "groups",
            "identity",
            "sessions",
            "inspect",
            "create",
            "modify",
            "password",
            "lock",
            "unlock",
            "delete",
            "expire",
            "group_create",
            "group_delete",
            "group_add",
            "group_remove",
            "group_primary",
            "admin_add",
            "admin_groups",
            "guide",
        ]
        .into_iter()
        .map(crate::i18n::gui_account_text)
        .chain(std::iter::once(crate::i18n::text("menu.back")))
        .collect::<Vec<_>>();
        let listed = options
            .iter()
            .enumerate()
            .map(|(index, option)| format!("  {}. «{}»", index + 1, option))
            .collect::<Vec<_>>()
            .join("\n");
        return format!(
            "GUÍA GRÁFICA: {}\n\nMenú completo «{}» (opciones del menú Linux):\n{}\n\nCampos y argumentos de la GUI:\n  {}\n\nProceso simple:\n  {}\n\nProceso complejo:\n  {}\n\nLinux administra cuentas locales; los permisos de administrador se conceden solo mediante grupos sudo, wheel o admin que ya existan. No se muestran acciones Windows ni identidades de servicio.",
            crate::i18n::accounts_label(),
            crate::i18n::accounts_label(),
            listed,
            crate::i18n::gui_account_text("guide_fields_linux"),
            crate::i18n::gui_account_text("guide_simple"),
            crate::i18n::gui_account_text("guide_complex_linux"),
        );
    }
    let (title, menu, options, fields, process) = match topic {
        "storage-partitions" => (
            "PARTICIONADO Y TABLAS", "Particionado y tablas",
            &["Consultar tabla de un disco", "Consultar espacio libre", "Inspeccionar dispositivo", "Crear tabla GPT", "Crear tabla MBR / msdos", "Crear partición", "Borrar partición", "Redimensionar partición", "Nombrar partición GPT", "Activar / desactivar flag", "Buscar y rescatar partición", "Comprobar alineación", "Cambiar flag del disco", "Alternar flag del disco", "Retirar firmas de almacenamiento", "Descartar bloques", "Guía de particionado", "Volver"][..],
            "Disco o dispositivo; las operaciones avanzadas añaden número de partición, tipo, inicio/fin, etiqueta, flag o rango de rescate.",
            "Consulta tabla, espacio libre e inspección del dispositivo; confirma el objetivo y conserva una copia antes de crear, borrar, redimensionar, cambiar marcas o retirar firmas.",
        ),
        "storage-filesystems" => (
            "SISTEMAS DE ARCHIVOS", "Sistemas de archivos",
            &["Crear / formatear sistema de archivos", "Cambiar etiqueta", "Comprobar sin reparar", "Comprobar y reparar automáticamente", "Redimensionar sistema de archivos", "Montar partición", "Desmontar dispositivo o ruta", "Activar swap", "Desactivar swap", "Guía de sistemas de archivos", "Volver"][..],
            "Partición/dispositivo, tipo de sistema de archivos, etiqueta, montaje, tamaño y ruta según la operación.",
            "Comprueba el dispositivo, decide si la operación es destructiva, confirma el tipo y verifica después el montaje, tamaño o estado.",
        ),
        "storage-volumes" => (
            "CIFRADO Y VOLÚMENES", "Cifrado y volúmenes",
            &["Crear contenedor LUKS", "Abrir contenedor LUKS", "Cerrar contenedor LUKS", "Copiar cabecera LUKS", "Restaurar cabecera LUKS", "Gestionar volúmenes LVM", "Gestionar volúmenes Btrfs", "Gestionar volúmenes ZFS", "Gestionar conjuntos RAID", "Guía de cifrado y volúmenes", "Volver"][..],
            concat!(
                "Campos: dispositivo, nombre del mapeo, archivo de cabecera, conjunto/grupo, volumen, tamaño, miembros, sustituto, propiedad, valor y nombre nuevo. Selector LVM: ",
                "«Preparar dispositivo para volúmenes», «Retirar dispositivo de volúmenes», «Crear grupo de volúmenes», «Eliminar grupo de volúmenes», «Crear volumen lógico», «Eliminar volumen lógico», «Ampliar volumen lógico» y «Reducir volumen lógico». Selector Btrfs: ",
                "«Crear subvolumen», «Eliminar subvolumen», «Crear instantánea», «Cambiar tamaño del sistema de archivos», «Reequilibrar datos», «Agregar dispositivo», «Retirar dispositivo», «Reemplazar dispositivo» y «Comprobar sin modificar». Selector ZFS: ",
                "«Crear conjunto de almacenamiento», «Eliminar conjunto de almacenamiento», «Exportar conjunto», «Importar conjunto», «Crear conjunto de datos», «Eliminar conjunto de datos», «Crear instantánea», «Comprobar conjunto», «Cambiar propiedad», «Renombrar conjunto de datos» y «Volver a una instantánea». Selector RAID: ",
                "«Consultar estado del conjunto», «Inspeccionar dispositivo miembro», «Reunir un conjunto existente», «Crear conjunto nuevo», «Agregar dispositivo miembro», «Reemplazar dispositivo miembro», «Retirar dispositivo miembro», «Marcar miembro como fallido», «Volver a agregar miembro», «Detener conjunto», «Comprobar consistencia», «Reparar conjunto» y «Cambiar cantidad de miembros»."
            ),
            "Identifica la capa y sus dependencias, conserva una copia de cabecera, revisa dispositivos y confirma solo con un plan de recuperación.",
        ),
        "accounts" => (
            "CUENTAS Y PERMISOS", "Usuarios, grupos y sesiones",
            &["Listar cuentas locales", "Listar grupos y miembros", "Ver mi identidad y grupos", "Ver sesiones abiertas", "Inspeccionar una cuenta", "Crear cuenta", "Editar cuenta y grupos", "Cambiar contraseña", "Bloquear cuenta", "Desbloquear cuenta", "Eliminar cuenta", "Configurar caducidad", "Crear grupo", "Eliminar grupo", "Añadir usuario a grupo", "Retirar usuario de grupo", "Cambiar grupo principal", "Conceder permisos de administrador", "Ver grupo y miembros administradores", "Guía de cuentas y permisos", "Volver"][..],
            "Inspeccionar: usuario. Crear: usuario, descripción, shell, grupos y opciones. Editar: usuario, descripción, shell y grupos. Contraseña: usuario y confirmación. Bloquear, desbloquear y eliminar: usuario. Caducidad: usuario y fechas. Grupos: nombre. Membresías: usuario y grupo. Administración: cuenta opcional (vacío significa cuenta actual).",
            "Consulta identidad, cuentas, grupos y sesiones. Para gestionar, identifica usuario/grupo, revisa el alcance, confirma y vuelve a listar. Linux solo añade la cuenta a un grupo sudo/wheel/admin existente; Windows usa el grupo integrado Administradores por SID. TrustedInstaller es una identidad de servicio, no un grupo normal para añadir usuarios.",
        ),
        "system" => (
            "SISTEMA, RED Y SEGURIDAD", "Sistema",
            &["Estado del sistema", "Usuarios, grupos y sesiones", "Red, rutas, DNS y puertos escuchando", "Arranque, EFI y cargador del sistema", crate::i18n::registry_label(), crate::i18n::diagnostics_label(), "Servicios del sistema", "Guía del sistema", "Volver"][..],
            "Los submenús tienen sus campos: servicios (ámbito, unidad, operación), red (interfaz/estado o conexión), arranque (entrada GRUB), cuentas (usuario/grupo) y Registro (clave/acción).",
            "Abre Estado del sistema y Diagnóstico; entra en el submenú de gestión, completa el objetivo exacto y verifica con una consulta posterior.",
        ),
        "native" => (
            "HERRAMIENTAS NATIVAS", "Herramientas nativas",
            &["Almacenamiento y particiones", "Sistema, red y seguridad", "Guía de herramientas nativas", "Volver"][..],
            "Esta pantalla no tiene campos; cada submenú documenta sus propios campos y opciones.",
            "Elige un objetivo y continúa al submenú correspondiente; no mezcles almacenamiento, red, servicios y arranque.",
        ),
        "diagnostics" => (
            "DEPENDENCIAS Y DIAGNÓSTICO", "Dependencias y diagnóstico",
            &[crate::i18n::gui_text("doctor"), crate::i18n::native_action_text("tools_status"), crate::i18n::tools_text("install"), crate::i18n::native_action_text("tools_install"), crate::i18n::diagnostics_label(), "Guía de dependencias", "Volver"][..],
            "«Instalar herramienta nativa» pide el identificador de herramienta.",
            "Ejecuta Doctor y Estado antes de instalar. Selecciona una herramienta, revisa disponibilidad y confirma solo la instalación explícita.",
        ),
        "connectivity" => (
            "CONECTIVIDAD", "SSH, SCP, SFTP y Android",
            &["SSH / SCP / SFTP", "Android (ADB)", "Guía de conectividad", "Volver"][..],
            "Esta pantalla no pide campos; cada submenú documenta sus propios campos.",
            "Elige SSH/SCP/SFTP o Android/ADB y continúa en el menú especializado; no mezcles sus destinos ni sus dispositivos.",
        ),
        "ssh" => (
            "SSH Y TRANSFERENCIAS", "SSH, SCP y SFTP",
            &["Conectar por SSH", "Copiar con SCP", "Abrir SFTP", "Guía de SSH, SCP y SFTP", "Volver"][..],
            "SSH: host, puerto, usuario, ruta/clave y destino según la acción. SCP: origen, destino y dirección. SFTP: host, puerto, usuario y ruta.",
            "Comprueba host e identidad. Ejecuta una conexión mínima, confirma transferencias y verifica el fichero de destino.",
        ),
        "adb" => (
            "ANDROID Y ADB", "Android y ADB",
            &["Abrir shell ADB", "Instalar APK", "Enviar archivo ADB", "Extraer archivo ADB", "Reiniciar dispositivo ADB", "Guía de Android y ADB", "Volver"][..],
            "Shell: dispositivo y comando. APK: dispositivo y APK. Transferencias: dispositivo, origen y destino. Reinicio: dispositivo y modo.",
            "Selecciona el dispositivo exacto, prueba identidad/estado, revisa origen y destino y confirma instalación, transferencia o reinicio.",
        ),
        "utilities" => (
            "UTILIDADES", "Utilidades instalables",
            &[crate::i18n::native_action_text("tools_status"), crate::i18n::native_action_text("tools_install"), "Guía de utilidades", "Volver"][..],
            "«Instalar utilidad» pide el identificador de la utilidad.",
            "Consulta disponibilidad, selecciona el identificador exacto, revisa el gestor y confirma la instalación.",
        ),
        "containers" => (
            "DOCKER Y PODMAN", "Docker/Podman",
            &["Contenedores", "Imágenes", "Volúmenes y redes", "Compose y diagnóstico", "Guía de Docker y Podman", "Volver"][..],
            "Esta pantalla no pide campos; cada submenú documenta sus propios campos.",
            "Abre el submenú adecuado y consulta primero el motor; las operaciones de ciclo de vida, imágenes, recursos y Compose tienen guías propias.",
        ),
        "containers-lifecycle" => (
            "CICLO DE VIDA DE CONTENEDORES", "Contenedores",
            &["Descargar imagen", "Crear y ejecutar contenedor", "Iniciar contenedor", "Detener contenedor", "Reiniciar contenedor", "Eliminar contenedor", "Ver logs del contenedor", "Ejecutar comando en contenedor", "Inspeccionar contenedor", "Estadísticas de contenedor", "Procesos del contenedor", "Puertos publicados", "Cambios del contenedor", "Pausar contenedor", "Reanudar contenedor", "Terminar contenedor", "Renombrar contenedor", "Copiar archivos", "Limpiar contenedores detenidos", "Guía del ciclo de vida de contenedores", "Volver"][..],
            "Motor y nombre; crear: imagen, nombre, puertos, volúmenes y comando; ejecutar: comando; copiar: origen y destino.",
            "Consulta el motor y el contenedor, revisa imagen, puertos y rutas, confirma la acción y vuelve a inspeccionar el estado.",
        ),
        "containers-images" => (
            "IMÁGENES", "Imágenes",
            &["Inspeccionar imagen", "Historial de imagen", "Construir imagen", "Etiquetar imagen", "Eliminar imagen", "Limpiar imágenes no usadas", "Guía de imágenes", "Volver"][..],
            "Inspeccionar/historial: imagen; construir: ruta y etiqueta; etiquetar: imagen y etiqueta; limpiar: motor.",
            "Inspecciona antes de construir, etiquetar o borrar; confirma la imagen exacta y conserva las etiquetas necesarias.",
        ),
        "containers-volumes" => (
            "VOLÚMENES Y REDES", "Volúmenes y redes",
            &["Listar volúmenes", "Inspeccionar volumen", "Crear volumen", "Eliminar volumen", "Limpiar volúmenes no usados", "Listar redes", "Inspeccionar red", "Crear red", "Eliminar red", "Limpiar redes no usadas", "Guía de volúmenes y redes", "Volver"][..],
            "Volumen/red: nombre y motor; las limpiezas usan solo el motor seleccionado.",
            "Lista e inspecciona recursos, confirma nombres y dependencias, y solo después crea, elimina o limpia.",
        ),
        "containers-compose" => (
            "COMPOSE Y DIAGNÓSTICOS", "Compose y diagnósticos",
            &["Operación Compose guiada", "Información del motor", "Uso de espacio del motor", "Limpieza global del motor", "Guía de Compose y diagnósticos", "Volver"][..],
            "Compose: operación, fichero, servicio y comando; diagnósticos: motor.",
            "Consulta información y uso, revisa el fichero Compose y el servicio, confirma la operación y verifica el motor.",
        ),
        "kubernetes" => (
            "KUBERNETES", "Kubernetes",
            &["Aplicar manifiesto", "Eliminar recurso", "Escalar deployment", "Reiniciar rollout", "Abrir port-forward", "Guía de Kubernetes", "Volver"][..],
            "Manifiesto: fichero y namespace. Recurso: tipo/nombre y namespace. Escalar: deployment y réplicas. Rollout: deployment. Port-forward: pod/servicio y puertos.",
            "Comprueba contexto, namespace y recurso. Usa inspección/diff antes de aplicar, eliminar, escalar o reiniciar y verifica el estado después.",
        ),
        "defaults" => (
            "VALORES PREDETERMINADOS", "Rutas predeterminadas",
            &[crate::i18n::gui_text("defaults"), crate::i18n::registry_label(), "Guía de valores predeterminados", "Volver"][..],
            "Esta pantalla no pide argumentos; abre valores o Registro y conserva la navegación.",
            "Consulta rutas y configuración efectiva, revisa la plataforma y vuelve a la guía antes de cambiar preferencias.",
        ),
        "installable" => (
            "HERRAMIENTAS INSTALABLES", "Herramientas instalables",
            &["Git / GitHub", "Software, paquetes y almacenes", "SSH, SCP, SFTP y Android", "Docker / Podman / Compose", "Kubernetes", "Utilidades del sistema", "Guía de herramientas instalables", "Volver"][..],
            "Esta pantalla solo contiene submenús; cada familia documenta sus argumentos y operaciones.",
            "Elige una familia, comprueba herramientas disponibles y abre su guía contextual antes de ejecutar una acción.",
        ),
        "packages" | "software" => (
            "SOFTWARE, PAQUETES Y ALMACENES", "Software, paquetes y almacenes",
            &["Buscar un paquete en las stores disponibles", "Elegir e instalar un paquete", "Almacenes de paquetes", "Guía de paquetes y software", "Volver"][..],
            "Buscar e instalar usan el nombre o candidato del campo de paquete; Tiendas y gestores no pide una ruta.",
            "Busca primero, selecciona un candidato concreto, revisa gestor y versión, confirma y verifica la instalación.",
        ),
        "automation" => (
            "AUTOMATIZACIÓN", "Automatización, scripts registrados y registro",
            &[
                "Scripts registrados",
                "Registrar nuevo script",
                "Guía de automatización",
                "Guía de scripts y automatización",
                "Volver",
            ][..],
            "Esta pantalla no pide campos; sus submenús contienen el registro y los scripts existentes.",
            "Abre Scripts registrados para ejecutar/editar/retirar o Registrar nuevo script para completar sus campos.",
        ),
        "automation-register" => (
            "REGISTRAR AUTOMATIZACIÓN", "Registrar nuevo script",
            &["Registrar script", "Guía para registrar scripts", "Volver"][..],
            "Nombre, programa, directorio de trabajo y argumentos separados.",
            "Rellena nombre y ejecutable, añade argumentos separados, registra, recarga el listado y prueba la acción sin ocultar errores.",
        ),
        _ => return format!("GUÍA GRÁFICA: {topic}\n\nEsta categoría no tiene un catálogo contextual registrado."),
    };
    let listed = options
        .iter()
        .enumerate()
        .map(|(index, option)| format!("  {}. «{}»", index + 1, option))
        .collect::<Vec<_>>()
        .join("\n");
    format!("GUÍA GRÁFICA: {title}\n\nMenú completo «{menu}»:\n{listed}\n\nCampos y argumentos de la GUI:\n  {fields}\n\nProceso simple:\n  Consulta primero el estado, inventario o diagnóstico y revisa el objetivo.\n\nProceso complejo:\n  {process}\n\nTodas las pantallas conservan «Volver». Las acciones mutables muestran confirmación y deben verificarse repitiendo la consulta correspondiente.")
}

fn gui_guide(topic: &str) -> String {
    if topic == "all" {
        #[cfg(windows)]
        return windows_gui_index();
        #[cfg(not(windows))]
        return GUI_INDEX.to_owned();
    }
    if topic == "updates" {
        #[cfg(windows)]
        if let Some(guide) = windows_gui_page_guide(topic) {
            return guide;
        }
        #[cfg(not(windows))]
        return gui_updates_guide();
    }
    #[cfg(not(windows))]
    if matches!(topic, "git" | "gh") {
        return linux_git_gui_guide();
    }
    #[cfg(windows)]
    {
        if matches!(
            topic,
            "storage-partitions" | "storage-filesystems" | "storage-volumes"
        ) {
            return format!(
                "GUÍA GRÁFICA WINDOWS: {topic}\n\n{}\n\nEstas operaciones específicas de Linux no existen en esta GUI; la guía anterior solo describe almacenamiento nativo Windows.",
                gui_storage_guide()
            );
        }
        if matches!(topic, "settings" | "privileges") {
            return gui_settings_guide();
        }
        if let Some(guide) = windows_gui_page_guide(topic) {
            return guide;
        }
        if topic == "storage" {
            return gui_storage_guide();
        }
        windows_gui_unavailable(topic)
    }
    #[cfg(not(windows))]
    match topic {
        "storage" => gui_storage_guide(),
        "aliases" => r#"GUÍA GRÁFICA: GESTOR DE ALIAS

Esta categoría no tiene un catálogo contextual registrado: el gestor de alias
no dispone de menú en la GUI. Se administra desde la CLI con `ltools aliases
ensure`, `list`, `doctor`, `add NOMBRE COMANDO [ARG...]`, `enable NOMBRE`,
`disable NOMBRE`, `remove NOMBRE`, `path` y `shell-init`. El registro se limita
a comandos conocidos de LTools y conserva sus argumentos por separado. Usa
`ltools guide aliases` para el flujo, las rutas y la compatibilidad por sistema.
"#
        .to_owned(),
        "network" => format!(
            r#"GUÍA GRÁFICA: {network_title}

Menú completo «{network_title}»:
  1. «{network_status}»: resumen de conectividad e interfaz activa.
  2. «{network_interfaces}»: interfaces, estados y direcciones.
  3. «{network_routes}»: rutas y puerta de enlace.
  4. «{network_dns}»: resolutores configurados y respuesta disponible.
  5. «{network_listening}»: servicios que escuchan y sus procesos.
  6. «{network_connections}»: perfiles detectados y su estado.
  7. «{network_flush_dns}»: acción protegida para renovar la resolución.
  8. «{network_interface_manage}»: campos «{interface_prompt}» y «{state_prompt}».
  9. «{network_connect}»: campo «{connection_prompt}».
 10. «{network_disconnect}»: el mismo campo; puede cortar la conexión.
 11. «{network_guide}»: vuelve a esta guía.
 12. «{back}»: regresa al menú anterior.

Proceso simple: ejecuta 1–6 y compara interfaz, ruta predeterminada, DNS,
puertos y perfil antes de modificar nada.

Proceso complejo: usa 8, 9 o 10, rellena el objetivo exacto, revisa la
confirmación y repite 1–3. No desconectes la interfaz de una sesión remota.
"#,
            network_title = crate::i18n::system_page_text("network_title"),
            network_status = crate::i18n::system_page_text("network_status"),
            network_interfaces = crate::i18n::system_page_text("network_interfaces"),
            network_routes = crate::i18n::system_page_text("network_routes"),
            network_dns = crate::i18n::system_page_text("network_dns"),
            network_listening = crate::i18n::system_page_text("network_listening"),
            network_connections = crate::i18n::system_page_text("network_connections"),
            network_flush_dns = crate::i18n::system_page_text("network_flush_dns"),
            network_interface_manage = crate::i18n::system_page_text("network_interface_manage"),
            network_connect = crate::i18n::system_page_text("network_connect"),
            network_disconnect = crate::i18n::system_page_text("network_disconnect"),
            network_guide = crate::i18n::system_page_text("network_guide"),
            interface_prompt = crate::i18n::gui_native_prompt("Interfaz de red (ej. eth0)"),
            state_prompt = crate::i18n::gui_native_prompt("Estado: up o down"),
            connection_prompt =
                crate::i18n::gui_native_prompt("Nombre exacto de la conexión NetworkManager"),
            back = crate::i18n::text("menu.back"),
        ),
        "boot" => format!(
            r#"GUÍA GRÁFICA: {boot_title}

Menú completo «{boot_title}»:
  1. «{boot_status}»: resumen de cargador y modo de arranque.
  2. «{efi_entries}»: entradas UEFI disponibles y orden.
  3. «{grub_entries}»: entradas reconocidas por GRUB.
  4. «{systemd_boot}»: loader y entradas systemd-boot.
  5. «{secure_boot}»: estado de Secure Boot si el equipo lo expone.
  6. «{boot_plan}»: prepara un plan de lectura/revisión.
  7. «{grub_schedule}»: campo «{entry_prompt}»; afecta solo al siguiente arranque.
  8. «{grub_cancel}»: elimina la selección programada.
  9. «{boot_guide}»: vuelve a esta guía.
 10. «{back}»: regresa al menú anterior.

Proceso simple: pulsa 1–6 y anota el título exacto de la entrada antes de
usar cualquier acción de cambio.

Proceso complejo: pulsa 7, escribe el título exacto, revisa la confirmación y
verifica con 1–3. Para deshacerlo usa 8 y vuelve a consultar el estado.
"#,
            boot_title = crate::i18n::system_page_text("boot_title"),
            boot_status = crate::i18n::system_page_text("boot_status"),
            efi_entries = crate::i18n::system_page_text("efi_entries"),
            grub_entries = crate::i18n::system_page_text("grub_entries"),
            systemd_boot = crate::i18n::system_page_text("systemd_boot"),
            secure_boot = crate::i18n::system_page_text("secure_boot"),
            boot_plan = crate::i18n::system_page_text("boot_plan"),
            grub_schedule = crate::i18n::system_page_text("grub_schedule"),
            grub_cancel = crate::i18n::system_page_text("grub_cancel"),
            boot_guide = crate::i18n::system_page_text("boot_guide"),
            entry_prompt =
                crate::i18n::gui_native_prompt("Título exacto de la entrada GRUB (ej. Ubuntu)"),
            back = crate::i18n::text("menu.back"),
        ),
        "privileges" => r#"GUÍA GRÁFICA: ELEVACIÓN Y PERMISOS

Menú completo «Ajustes»:
  1. «Tema visual»: cambia la apariencia.
  2. «Idioma»: cambia el idioma disponible.
  3. «Visibilidad de categorías»: muestra u oculta familias de la ventana.
  4. «Elevar acciones modificadoras por defecto»: activa o desactiva la
     solicitud automática de sudo/pkexec en Linux o UAC/NSudo en Windows.
  5. «Guía de ajustes y visibilidad»: abre la explicación de esta pantalla.
  6. «Volver»: regresa al menú anterior.

Proceso simple: deja la casilla desactivada para que las consultas y las
acciones de usuario se ejecuten con tu identidad; las acciones obligatorias
seguirán pidiendo autorización cuando el sistema la exija.

Al instalar software, búsqueda, selección y confirmación permanecen en esta
sesión. Después solo el gestor nativo recibe elevación si su operación la
necesita; los gestores de usuario no se fuerzan como administrador.

Proceso complejo: activa la casilla, abre almacenamiento, servicios, red,
cuentas o arranque, revisa el objetivo exacto y confirma. La política eleva
solo acciones Optional/Required; Git/GitHub, Wine/Proton, automatizaciones,
aliases, limpieza guiada, exportaciones al perfil y papelera se mantienen sin
elevar el proceso completo para conservar la identidad, los permisos y el
destino de los datos. Los gestores de paquetes autorizan solo la operación
concreta que requiere permisos del sistema.

La GUI informa qué clasificación aplica antes de ejecutar. «Cancelar» o
rechazar la contraseña deja el sistema sin cambios; `--no-elevate` permite
desactivar la elevación opcional en una ejecución concreta.
"#
        .to_owned(),
        "services" => format!(
            r#"GUÍA GRÁFICA: {services_title}

Menú completo «{services_title}»:
  1. «{services_automatic}»: unidades del sistema configuradas para iniciar
     automáticamente o estáticas.
  2. «{services_manual}»: unidades del sistema no automáticas.
  3. «{services_user}»: unidades del ámbito de usuario.
  4. «{services_both}»: inventario combinado.
  5. «{services_failed}»: fallos y eventos relacionados.
  6. «{services_manage}»: campos «{scope_prompt}», «{unit_prompt}» y
     «{action_prompt}».
  7. «{services_export}»: guarda la vista combinada en TSV.
  8. «{services_guide}»: vuelve a esta guía.
  9. «{back}»: regresa al menú anterior.

Proceso simple: revisa 1–5 y compara estado, unidad, ámbito, origen,
dependencias y eventos.

Proceso complejo: en 6 elige ámbito, unidad y acción exactos; revisa la
confirmación, ejecuta y vuelve a consultar 1–5. Usa 7 para conservar evidencia.
"#,
            services_title = crate::i18n::system_page_text("services_title"),
            services_automatic = crate::i18n::system_page_text("services_automatic"),
            services_manual = crate::i18n::system_page_text("services_manual"),
            services_user = crate::i18n::system_page_text("services_user"),
            services_both = crate::i18n::system_page_text("services_both"),
            services_failed = crate::i18n::system_page_text("services_failed"),
            services_manage = crate::i18n::system_page_text("services_manage"),
            services_export = crate::i18n::system_page_text("services_export"),
            services_guide = crate::i18n::system_page_text("services_guide"),
            scope_prompt = crate::i18n::gui_native_prompt("Ámbito: system o user"),
            unit_prompt = crate::i18n::gui_native_prompt("Unidad (ej. sshd.service)"),
            action_prompt = crate::i18n::gui_native_prompt(
                "Acción: status, start, stop, restart, enable, disable, mask o unmask"
            ),
            back = crate::i18n::text("menu.back"),
        ),
        "git" | "gh" => r#"GUÍA GRÁFICA: GIT Y GITHUB

Campos compartidos del menú: repositorio/ruta, URL, destino de clonación,
remoto, rama, mensaje, notas y límite. Déjalos vacíos solo cuando la pantalla
indique que se usa el repositorio actual o el valor predeterminado.

Menú completo «Git / GitHub»:
  1. «Guía completa de Git y GitHub (gh)».
  2. «Estado del repositorio Git».
  3. «Clonar un repositorio Git».
  4. «Descargar referencias Git remotas».
  5. «Descargar e integrar cambios Git».
  6. «Ver historial de commits».
  7. «Preparar cambios».
  8. «Crear commit».
  9. «Subir cambios».
 10. «Listar o cambiar de rama».
 11. «Listar o crear tag».
 12. «Crear release de GitHub».
 13. «Comprobar identidad Git e inicio de sesión GitHub opcional».
 14. «Repositorio de GitHub».
 15. «Pull requests de GitHub».
 16. «Releases de GitHub».
 17. «Estado de autenticación GitHub».
 18. «Versión de GitHub CLI».
 19. «Ayuda nativa de GitHub CLI».
 20. «Comando nativo de gh…»: solicita comando, subcomando/identificador,
     elemento opcional y repositorio; ejecuta argumentos separados con la
     versión de gh instalada. Para flags avanzados usa la CLI guiada por
     `ltools guide gh`.
 21. «Diagnosticar repositorio»: comprueba raíz, índice, HEAD y objetos.
 22. «Reconstruir índice .git…»: conserva una copia del índice y solo lo
     reconstruye si HEAD está íntegro; no recupera objetos perdidos.
 23. «Recuperar .git desde remoto…»: solicita URL HTTPS/SSH y rama opcional;
     clona sin checkout y restaura solo la metadata, sin sobrescribir archivos
     locales. Después revisa los cambios frente al remoto y conserva el estado.
 24. «Volver»: regresa al menú anterior.

Proceso simple: empieza por Estado, Diagnosticar, Historial, identidad o
autenticación. Para Clonar completa URL y destino. «Ayuda nativa» muestra las
opciones que ofrece tu versión real de gh; no todas las versiones incluyen los
mismos comandos ni extensiones.

Proceso complejo: Estado → Preparar cambios → Commit → rama/tag → Push o
Release. Revisa el diff y remoto; nunca pegues tokens en los campos. Antes de
reparar, diagnostica; conserva la copia del índice hasta comprobar el estado.
Si falta .git, aporta el remoto correcto, valida la comparación del árbol local
y no repitas el proceso sobre una ruta distinta sin revisar la primera copia.
"#
        .to_owned(),
        "wine" => r#"GUÍA GRÁFICA: WINE Y PROTON

Menú completo «Gestión de prefijos Wine y Proton»:
  1. «Listar prefijos detectados»: no pide campos; muestra ruta, runner y
     arquitectura.
  2. «Inspeccionar prefijo»: campo «Ruta del prefijo Wine/Proton».
  3. «Crear prefijo»: «Ruta del nuevo prefijo» y «Arquitectura: win64 o
     win32 (opcional)».
  4. «Migrar y automatizar prefijo»: origen, destino y las opciones Steam/
     Proton, valores predeterminados, configuraciones, lanzadores, retirar
     origen y forzar cuando corresponda.
  5. «Guía de Wine y Proton»: vuelve a esta guía.
  6. «Volver»: regresa al menú anterior.

Proceso simple: usa 1 y 2 para revisar antes de tocar un prefijo.
Proceso complejo: en 3 o 4 confirma espacio, origen, destino y arquitectura;
el origen se conserva por defecto y la migración se verifica antes de retirar
cualquier fuente.
"#
        .to_owned(),
        _ => gui_catalog_guide(topic),
    }
}

#[cfg(not(windows))]
fn linux_git_gui_guide() -> String {
    let labels = [
        crate::i18n::git_action_text("guide"),
        crate::i18n::tools_text("git_status"),
        crate::i18n::tools_text("git_clone"),
        crate::i18n::tools_text("git_fetch"),
        crate::i18n::tools_text("git_pull"),
        crate::i18n::tools_text("git_log"),
        crate::i18n::tools_text("git_add"),
        crate::i18n::tools_text("git_commit"),
        crate::i18n::tools_text("git_push"),
        crate::i18n::tools_text("git_branch"),
        crate::i18n::tools_text("git_tag"),
        crate::i18n::tools_text("git_release"),
        crate::i18n::tools_text("git_login"),
        crate::i18n::tools_text("gh_repo"),
        crate::i18n::tools_text("gh_prs"),
        crate::i18n::tools_text("gh_releases"),
        crate::i18n::tools_text("gh_auth_status"),
        crate::i18n::git_action_text("version"),
        crate::i18n::git_action_text("help"),
        crate::i18n::git_action_text("native"),
        crate::i18n::git_action_text("diagnose"),
        crate::i18n::git_action_text("repair_index"),
        crate::i18n::git_action_text("repair_remote"),
        crate::i18n::text("menu.back"),
    ];
    let options = labels
        .iter()
        .enumerate()
        .map(|(index, label)| format!("{:>2}. «{label}»", index + 1))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "GUÍA GRÁFICA: GIT Y GITHUB\n\nMenú completo «{}»:\n{}\n\nCampos del menú: para Git, Repositorio es una ruta local; para acciones GitHub pide OWNER/REPO (o HOST/OWNER/REPO). También hay campos URL, destino de clonación, remoto, rama, mensaje, notas y límite. La acción «{}» abre un formulario con dos campos: comando raíz de gh (una palabra) y argumentos en una sola línea. Las comillas simples/dobles agrupan espacios; no se expande ni ejecuta una shell. El campo Repositorio añade --repo al final de los argumentos, salvo para auth/help/version/api, donde no se aplica. Si ya incluyes --repo o -R en Argumentos, deja vacío el campo Repositorio; si los duplicas, la acción se detiene y explica el conflicto. Para gh api indica el endpoint completo en Argumentos.\n\nProceso simple: consulta estado, historial, identidad o autenticación. Usa «{}» para consultar la ayuda exacta de la versión instalada; la lista de comandos y sus argumentos puede variar según versiones y extensiones.\n\nProceso complejo: revisa el repositorio, prepara cambios, crea el commit y confirma la rama antes de publicar o crear una release. Para el índice, diagnostica primero: la reconstrucción solo está disponible si HEAD y los objetos están íntegros. Si falta .git, aporta un remoto exacto; la recuperación valida el remoto, clona sin checkout y nunca sobrescribe los archivos locales. Tras recuperar, revisa cada diferencia antes de continuar.\n\n«{}» vuelve a la pantalla anterior.",
        crate::i18n::tools_text("git_menu"),
        options,
        crate::i18n::git_action_text("native"),
        crate::i18n::git_action_text("help"),
        crate::i18n::text("menu.back"),
    )
}

#[cfg(windows)]
fn cli_platform_summary(topic: &str) -> &'static str {
    match topic {
        "network" => "PERFIL WINDOWS: los objetivos son adaptadores, perfiles, rutas, DNS y conexiones Windows.",
        "boot" => "PERFIL WINDOWS: se ofrece inspección BCD/UEFI y Secure Boot; las opciones EFI/GRUB de Linux no aplican.",
        "services" => "PERFIL WINDOWS: se gestionan servicios y eventos Windows; no existen scopes systemd.",
        "storage" => "PERFIL WINDOWS: los objetivos son discos, particiones, volúmenes y letras de unidad Windows.",
        "wine" => "PERFIL WINDOWS: WINE/PROTON NO APLICA al ejecutable Windows nativo.",
        "updates" => "PERFIL WINDOWS: se descarga el EXE Windows del perfil GUI/CLI y no se eleva ni reemplaza automáticamente.",
        _ => "PERFIL WINDOWS: LTools adapta objetivos, permisos y capacidades a Windows nativo.",
    }
}

#[cfg(not(windows))]
fn cli_platform_summary(topic: &str) -> &'static str {
    match topic {
        "network" => "PERFIL LINUX: los objetivos son interfaces, perfiles, rutas, DNS y puertos Linux.",
        "boot" => "PERFIL LINUX: EFI, GRUB, systemd-boot y Secure Boot se muestran solo si la plataforma los ofrece.",
        "services" => "PERFIL LINUX: los scopes system, user y both distinguen servicios del sistema y del usuario.",
        "storage" => "PERFIL LINUX: los objetivos son dispositivos, volúmenes, montajes y rutas POSIX.",
        "wine" => "PERFIL LINUX: se pueden localizar y gestionar prefijos Wine/Proton compatibles.",
        "updates" => "PERFIL LINUX: AppImage o tarball según el ejecutable actual; descarga al perfil del usuario sin sudo.",
        _ => "PERFIL LINUX: LTools adapta objetivos, permisos y capacidades al anfitrión Linux.",
    }
}

#[cfg(windows)]
const WINDOWS_PLATFORM_OVERVIEW: &str = r#"PERFIL WINDOWS: DIFERENCIAS IMPORTANTES

Esta guía corresponde al ejecutable Windows nativo. No se deben copiar aquí
comandos Linux como `systemctl`, `nmcli`, `ip`, `ss`, `efibootmgr`, `grub-*`,
`parted`, `lsblk`, `mount`, `sudo` o rutas `/etc` y `/dev`.

Equivalencias que usa LTools:
  Red: PowerShell `Get-NetAdapter`, `Get-NetIPConfiguration`, `Get-NetRoute`,
  `Get-DnsClientServerAddress`, `Get-NetTCPConnection` y `netsh` cuando se
  necesita una acción compatible. Los argumentos son nombres de interfaz,
  perfil y dirección Windows, no nombres Linux como eth0.
  Servicios: `Get-Service`, `Start-Service`, `Stop-Service`, `Restart-Service`,
  `Set-Service`, `sc.exe` y `Get-WinEvent`; no hay scopes systemd user/system.
  Arranque: BCD/UEFI mediante `bcdedit`, `mountvol` y
  `Confirm-SecureBootUEFI`; `boot set-next`, GRUB, systemd-boot y efibootmgr
  no son operaciones Windows disponibles.
  Discos: `Get-Disk`, `Get-Partition`, `Get-Volume` y `diskpart`; no se usan
  parted, mkfs, LUKS, LVM, Btrfs, ZFS ni mdadm.
  Configuración: `reg.exe` y cmdlets PowerShell; no se inspeccionan rutas de
  configuración Linux.
  Cuentas: `Get-LocalUser`, `New-LocalUser`, `Set-LocalUser`,
  `Disable-LocalUser`, `Enable-LocalUser`, `Get-LocalGroup` y
  `Add-LocalGroupMember`; no useradd/usermod/groupadd.

PowerShell y CMD tienen reglas de comillas y rutas distintas de Bash. LTools
mantiene executable y args separados, no convierte una cadena PowerShell en
shell arbitraria y usa UAC solo en las acciones Windows que lo necesitan.
"#;

#[cfg(not(windows))]
const WINDOWS_PLATFORM_OVERVIEW: &str = r#"PERFIL LINUX

Esta guía corresponde al ejecutable Linux nativo. Usa las herramientas
disponibles en el anfitrión, normalmente `ip`, `nmcli`, `resolvectl`, `ss`,
`systemctl`, `journalctl`, `efibootmgr`, `grub-editenv`, `bootctl`, `lsblk`,
`findmnt`, `parted`, `mount`, `cryptsetup` y el motor Docker/Podman detectado.

Los nombres de interfaz, unidades systemd, dispositivos `/dev/...` y rutas
son objetivos Linux explícitos. No se sustituyen por cmdlets PowerShell ni se
asume que exista una herramienta opcional: `doctor` muestra disponibilidad y
la guía indica la alternativa compatible.
"#;

#[cfg(windows)]
const WINDOWS_GUIDE_RULES: &str = r#"REGLAS COMUNES DEL PERFIL WINDOWS

Las consultas no modifican el equipo. Las acciones mutables requieren
confirmación, `--dry-run` cuando esté disponible y `--plan FICHERO` cuando el
módulo lo admita. UAC se solicita solo si la acción Windows lo necesita.
Los valores se pasan como argumentos separados: no se ejecutan cadenas Bash,
PowerShell o CMD construidas con entrada del usuario y no se introducen
contraseñas/tokens en argumentos ni logs.
"#;

#[cfg(windows)]
fn platform_details(topic: &str) -> &'static str {
    match topic {
        "git" => {
            r#"WINDOWS: GIT Y GITHUB CLI

Git for Windows usa los mismos subcomandos Git, pero las rutas son `C:\\...`
o rutas relativas de PowerShell. Ejemplos:
  `ltools git status --repo C:\\src\\proyecto`
  `ltools git clone https://github.com/ORG/REPO.git C:\\src\\REPO --dry-run`
  `ltools git push --repo C:\\src\\REPO --remote origin --branch main --dry-run`

GitHub CLI es `gh.exe`. `ltools git gh native help` muestra la ayuda de la
versión instalada y `ltools git gh native issue list --repo ORG/REPO` pasa
argumentos por separado. `gh native` admite comandos nuevos y extensiones que
ofrezca esa versión; no convierte rutas Linux a Windows. El diagnóstico y la
reparación del índice usan rutas Windows; si falta `.git`, la recuperación
requiere URL HTTPS/SSH explícita, clona sin checkout y no sobrescribe archivos.
No se usa `sudo`; la red y las credenciales las gestiona Git for Windows/
Windows.
"#
        }
        "gh" => {
            r#"WINDOWS: GH.EXE

La dependencia es `gh.exe` en PATH. Usa `ltools git gh auth-status`,
`ltools git gh repo --repo ORG/REPO`, `prs` y `releases`; `ltools git gh
native help` consulta la ayuda y `native <comando> <argumentos...>` aprovecha
la versión instalada, incluidas funciones nuevas y extensiones compatibles.
El login abre el flujo oficial de `gh.exe`. Los paths de trabajo son Windows y
la autenticación queda en el almacén/configuración de gh, nunca en argumentos
de LTools.
"#
        }
        "network" => {
            r#"WINDOWS: RED Y ARGUMENTOS

Los objetivos son nombres Windows (`Ethernet`, `Wi-Fi`) y perfiles de
Network Connections, no `eth0` ni NetworkManager. La inspección corresponde a
`Get-NetAdapter`, `Get-NetIPConfiguration`, `Get-NetRoute`,
`Get-DnsClientServerAddress` y `Get-NetTCPConnection`. La GUI solicita el
adaptador o perfil exacto; `--dry-run` muestra la acción PowerShell/netsh.
"#
        }
        "boot" => {
            r#"WINDOWS: ARRANQUE

La inspección se hace sobre BCD/UEFI con `bcdedit`, `mountvol` y
`Confirm-SecureBootUEFI`. No aparecen ni se ejecutan `efibootmgr`, `grub-*`,
`systemd-boot` o `bootctl`. LTools solo ofrece el estado y el plan compatible;
no traduce una entrada GRUB a BCD ni reinicia Windows automáticamente.
"#
        }
        "services" => {
            r#"WINDOWS: SERVICIOS Y EVENTOS

La gestión usa `Get-Service`/`sc.exe` y acciones como Start, Stop, Restart,
Set-StartType y consulta de `Get-WinEvent`. No hay filtro `--scope user/system`
de systemd ni journal; la GUI muestra el origen como servicio Windows y exige
el nombre exacto de la unidad.
"#
        }
        "storage" => {
            r#"WINDOWS: DISCOS Y VOLUMENES

La consulta usa `Get-Disk`, `Get-Partition`, `Get-Volume` y `mountvol`; el
flujo avanzado de particionado se delega a `diskpart` con revisión explícita.
No se ofrecen `parted`, `mkfs`, LUKS, LVM, Btrfs, ZFS, RAID mdadm ni targets
`/dev/sdX`. Los objetivos son discos/volúmenes Windows y las letras de unidad.
"#
        }
        "accounts" => {
            r#"WINDOWS: CUENTAS Y GRUPOS

Los argumentos son nombres de cuenta/grupo Windows. Las acciones se apoyan en
cmdlets `Get/New/Set/Enable/Disable-LocalUser` y `Add/Remove-LocalGroupMember`.
No se usan `useradd`, `usermod`, `groupadd`, `/etc/passwd` ni shells Linux.
"#
        }
        "registry" => {
            r#"WINDOWS: REGISTRO

La acción usa `reg.exe` y, cuando corresponde, `Get-ItemProperty`/
`Export-Clixml` o exportación nativa. El objetivo es una colmena/clave como
`HKLM\\Software\\...`; no se acepta una ruta Linux. Exportar es consulta;
importar/escribir exige fichero, copia y confirmación.
"#
        }
        "wine" => {
            r#"WINDOWS: WINE/PROTON NO APLICA

El ejecutable Windows nativo no escanea ni crea prefijos Wine/Proton. La guía
de esa opción solo informa de incompatibilidad; para gestionar prefijos hay
que usar el ejecutable Linux o una máquina Linux. No se interpretan rutas
`WINEPREFIX`, Steam compatdata o Lutris en Windows nativo.
"#
        }
        "system" | "diagnostics" | "utilities" => {
            r#"WINDOWS: SISTEMA Y DIAGNOSTICO

La disponibilidad se comprueba con PowerShell, `Get-Command`, `Get-CimInstance`
y `Get-WindowsOptionalFeature` cuando procede. No se presupone `systemd`,
procfs ni utilidades GNU; las capacidades no disponibles se marcan como tales
y no se sustituyen silenciosamente por una orden Linux.
"#
        }
        "containers" => {
            r#"WINDOWS: CONTENEDORES

Docker Desktop, Docker Engine o Podman deben estar instalados y accesibles
desde PowerShell. Los nombres de imagen/contenedor son los mismos, pero las
rutas de bind mount son `C:\\datos:/datos` o la sintaxis aceptada por el motor
Windows. La GUI comprueba el motor antes de ofrecer acciones.
"#
        }
        "kubernetes" => {
            r#"WINDOWS: KUBERNETES

La acción usa `kubectl.exe`/Helm desde PowerShell y respeta el contexto
seleccionado. Las rutas de manifiesto son Windows (`C:\\...`); apply, delete,
scale y rollout siguen requiriendo contexto, RBAC y confirmación.
"#
        }
        "ssh" | "adb" => {
            r#"WINDOWS: SSH Y ADB

Se usan `ssh.exe`, `scp.exe`, `sftp.exe` y `adb.exe` nativos o instalados. Los
paths locales son Windows y no se antepone `sudo`. Para ADB se selecciona el
serial exacto y se mantienen las mismas confirmaciones de instalación,
transferencia y reinicio.
"#
        }
        "packages" | "software" | "clean" => {
            r#"WINDOWS: PAQUETES Y LIMPIEZA

Se priorizan winget, Chocolatey, Scoop o Microsoft Store solo si están
disponibles y el usuario confirma. Los argumentos de instalación y rutas de
caché son Windows; no se ejecutan apt, pacman, dnf, Flatpak ni rutas Linux.
"#
        }
        "automation" | "actions" => {
            r#"WINDOWS: AUTOMATIZACION

Los scripts registrados deben apuntar a ejecutables Windows (`.exe`, `.cmd`,
`.ps1` mediante PowerShell) y sus argumentos separados. No se genera una
cadena Bash; el catálogo marca requisitos y compatibilidad Windows antes de
ejecutar.
"#
        }
        "defaults" | "audit" => {
            r#"WINDOWS: RUTAS Y AUDITORIA

Las rutas efectivas salen de `%USERPROFILE%`, `%LOCALAPPDATA%`, Steam Windows,
Documentos y volúmenes montados. La auditoría no presupone Home Linux, `/mnt`
ni prefijos Wine/Proton; el sistema anfitrión Windows decide qué ubicaciones
están disponibles.
"#
        }
        _ => WINDOWS_PLATFORM_OVERVIEW,
    }
}

#[cfg(not(windows))]
fn platform_details(topic: &str) -> &'static str {
    match topic {
        "network" => {
            r#"LINUX: COMANDOS NATIVOS

La GUI/CLI usa `ip`, `nmcli`, `resolvectl`/`systemd-resolve` y `ss` según
disponibilidad. `--interface` espera una interfaz Linux real (por ejemplo
`enp3s0`), `--connection` un perfil NetworkManager y `--state up|down` el
estado solicitado.
"#
        }
        "boot" => {
            r#"LINUX: COMANDOS NATIVOS

EFI usa `efibootmgr`; GRUB usa `grub-editenv`/`grub-reboot`; systemd-boot usa
`bootctl`; Secure Boot se consulta con `mokutil` cuando existe. Los nombres de
entrada y dispositivos son Linux y pueden requerir root/polkit.
"#
        }
        "services" => {
            r#"LINUX: SYSTEMD

Los scopes `system`, `user` y `both` corresponden a `systemctl` del sistema y
del usuario. Las acciones usan `systemctl` y `journalctl`; `--filter
automatic|manual|all` es una opción de LTools para clasificar unidades.
"#
        }
        "storage" => {
            r#"LINUX: COMANDOS NATIVOS

Se usan `lsblk`, `findmnt`, `df`, `parted`, `mount`, `fsck`, `cryptsetup`, LVM,
Btrfs, ZFS y mdadm solo si están disponibles. Los objetivos son dispositivos
`/dev/...`; nunca se convierten automáticamente en letras de unidad Windows.
"#
        }
        "registry" => {
            r#"LINUX: CONFIGURACION

No existe Registro Windows. `registry` inspecciona rutas de configuración,
journal y ubicaciones efectivas Linux; no ejecuta `reg.exe` ni acepta colmenas
HKLM/HKCU.
"#
        }
        "wine" => {
            r#"LINUX: PREFIJOS

Wine/Proton solo se ofrece en el perfil Linux. Las rutas pueden estar en
`WINEPREFIX`, Steam compatdata, Lutris, Heroic, UMU o Bottles; se inspeccionan
antes de crear o migrar y no se asumen en Windows nativo.
"#
        }
        "git" | "gh" => {
            r#"LINUX: GIT Y GH

Se invocan `git` y `gh` nativos desde una terminal Linux. Las rutas siguen la
sintaxis POSIX y no se antepone `sudo`; URL, repo, rama y destino se pasan como
argumentos separados. El flujo Git/GitHub es el mismo conceptualmente, pero
los paths, permisos y gestores de credenciales son los de Linux.
"#
        }
        "automation" | "actions" => {
            r#"LINUX: AUTOMATIZACION

Los scripts apuntan a ejecutables Linux y sus argumentos separados; no se
evalúa Bash arbitrario. Una acción declara si requiere `sudo`, un comando
opcional o una herramienta del anfitrión antes de habilitarse.
"#
        }
        _ => WINDOWS_PLATFORM_OVERVIEW,
    }
}

fn common_guide(topic: &str) -> Result<&'static str, String> {
    match topic {
        "all" => Ok(INDEX),
        "audit" => Ok(AUDIT),
        "packages" => Ok(PACKAGES),
        "installable" => Ok(PACKAGES),
        "git" => Ok(GIT),
        "gh" => Ok(GH),
        "automation" => Ok(AUTOMATION),
        "automation-register" => Ok(AUTOMATION),
        "clean" => Ok(CLEAN),
        "storage" | "storage-partitions" | "storage-filesystems" | "storage-volumes" => Ok(STORAGE),
        "updates" => Ok(UPDATES),
        "system" => Ok(SYSTEM),
        "services" => Ok(SERVICES),
        "accounts" => Ok(ACCOUNTS),
        "network" | "connectivity" => Ok(NETWORK),
        "boot" => Ok(BOOT),
        "registry" => Ok(REGISTRY),
        "privileges" => Ok(PRIVILEGES),
        "diagnostics" => Ok(DIAGNOSTICS),
        "wine" => Ok(WINE),
        "defaults" | "settings" => Ok(DEFAULTS),
        "containers"
        | "containers-lifecycle"
        | "containers-images"
        | "containers-volumes"
        | "containers-compose" => Ok(CONTAINERS),
        "kubernetes" => Ok(KUBERNETES),
        "ssh" => Ok(SSH),
        "adb" => Ok(ADB),
        "utilities" => Ok(UTILITIES),
        "actions" => Ok(ACTIONS),
        _ => Err(format!(
            "guía desconocida: {topic}. Usa `ltools guide all`."
        )),
    }
}

pub fn run(_ctx: &Context, args: &[String]) -> Result<(), String> {
    let (mode, topic_value) = match args.first().map(String::as_str) {
        Some("gui") => ("gui", args.get(1)),
        Some("cli") => ("cli", args.get(1)),
        _ => ("cli", args.first()),
    };
    let requested = topic_value
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "all".to_owned());
    let topic = match requested.as_str() {
        "all" | "list" | "help" => "all",
        "audit" | "inventory" | "games" => "audit",
        "packages" | "software" | "package" => "packages",
        "installable" | "installable-tools" => "installable",
        "git" => "git",
        "gh" | "github" => "gh",
        #[cfg(windows)]
        "winslim" if mode == "gui" => "winslim",
        "aliases" | "alias" => "aliases",
        "automation" | "automations" => "automation",
        "automation-register" | "register-script" => "automation-register",
        "clean" | "cleanup" => "clean",
        "storage" | "disks" | "partitions" => "storage",
        "storage-partitions" | "partition-guide" => "storage-partitions",
        "storage-filesystems" | "filesystem-guide" => "storage-filesystems",
        "storage-volumes" | "volume-guide" => "storage-volumes",
        // «native» es el índice contextual de la página «Herramientas
        // nativas» en la GUI. En CLI conserva la guía general de sistema
        // para no romper el alias histórico `guide native`.
        "system" => "system",
        "native" if mode == "gui" => "native",
        "native" => "system",
        "services" | "service" => "services",
        "accounts" | "users" => "accounts",
        "network" | "red" => "network",
        "connectivity" | "connect" => "connectivity",
        "boot" | "efi" | "grub" => "boot",
        "registry" | "records" => "registry",
        "privileges" | "elevation" | "permissions" => "privileges",
        "diagnostics" | "doctor" => "diagnostics",
        "wine" | "proton" | "prefix" => "wine",
        "defaults" => "defaults",
        "settings" => "settings",
        "updates" | "update" => "updates",
        "containers" | "docker" | "podman" => "containers",
        "containers-lifecycle" | "container-lifecycle" => "containers-lifecycle",
        "containers-images" | "container-images" => "containers-images",
        "containers-volumes" | "container-volumes" => "containers-volumes",
        "containers-compose" | "container-compose" => "containers-compose",
        "kubernetes" | "k8s" => "kubernetes",
        "ssh" | "scp" | "sftp" => "ssh",
        "adb" | "android" => "adb",
        "utilities" | "tools" => "utilities",
        "actions" | "catalog" => "actions",
        _ => {
            return Err(format!(
                "guía desconocida: {requested}. Usa `ltools guide all`."
            ))
        }
    };
    if mode == "gui" {
        print!("{}", gui_guide(topic));
    } else {
        print!("{}\n\n{}", cli_guide(topic), cli_platform_summary(topic));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_mentions_all_major_families() {
        for topic in [
            "git / gh", "network", "boot", "services", "wine", "updates", "actions",
        ] {
            assert!(INDEX.contains(topic), "falta {topic}");
        }
        assert!(
            CLI_INDEX.contains("updates"),
            "falta updates en el índice de CLI"
        );
    }

    #[test]
    fn updater_guides_cover_integrity_and_manual_platform_specific_installation() {
        let _language_guard = crate::i18n::language_test_guard();
        crate::i18n::set("es");
        let guide = common_guide("updates").unwrap();
        assert!(guide.contains("Ed25519") && guide.contains("SHA256SUMS.txt"));
        assert!(guide.contains("no sobrescribe") && guide.contains("no instala automáticamente"));
        let graphical = gui_guide("updates");
        assert!(graphical.contains("Comprobar actualizaciones"));
        assert!(graphical.contains("Descargar actualización verificada"));
        assert!(graphical.contains("Proceso simple") && graphical.contains("Proceso complejo"));
    }

    #[test]
    fn git_guide_covers_gh_and_safe_flow() {
        let _language_guard = crate::i18n::language_test_guard();
        crate::i18n::set("es");
        let guide = common_guide("git").unwrap();
        assert!(guide.contains("GIT Y GITHUB") && guide.contains("--dry-run"));
        assert!(guide.contains("clone") && guide.contains("push") && guide.contains("release"));
        assert!(guide.contains("gh native <comando> [argumentos]"));
        assert!(guide.contains("project") && guide.contains("codespace"));
        assert!(guide.contains("--ltools-confirmed") && guide.contains("--yes` se conserva"));
        assert!(guide.contains("--remote URL") && guide.contains("clona sin checkout"));
        assert!(guide.contains("ltools git status --repo"));
        assert!(guide.contains("ltools git gh auth-status"));
    }

    #[test]
    fn privilege_guide_documents_manager_scoped_software_elevation() {
        let _language_guard = crate::i18n::language_test_guard();
        crate::i18n::set("es");
        let guide = common_guide("privileges").unwrap();
        assert!(guide.contains("búsqueda, selección del paquete y confirmación"));
        assert!(guide.contains("solo se eleva el gestor nativo"));
        assert!(guide.contains("Scoop permanece siempre en el perfil"));
        assert!(gui_settings_guide().contains("solo se eleva el gestor nativo"));
    }

    #[test]
    fn gui_guides_explain_visual_simple_and_complex_flows() {
        let guide = gui_guide("storage");
        assert!(guide.contains("GUÍA GRÁFICA"));
        assert!(guide.contains("Proceso simple") && guide.contains("Proceso complejo"));
        assert!(guide.contains("Crear tabla GPT"));
        assert!(
            guide.contains("Gestionar volúmenes LVM") && guide.contains("Gestionar conjuntos RAID")
        );
        assert!(!guide.contains("lsblk"));
        for internal in ["parted", "wipefs", "mdadm", "--depth", "pkexec"] {
            assert!(
                !guide.contains(internal),
                "la guía gráfica expone {internal}"
            );
        }
        for option in [
            "Consultar tabla de un disco",
            "Inspeccionar dispositivo",
            "Retirar firmas de almacenamiento",
            "Gestionar volúmenes Btrfs",
            "Gestionar volúmenes ZFS",
            "Gestionar conjuntos RAID",
            "Restaurar cabecera LUKS",
        ] {
            assert!(guide.contains(option), "falta la acción GUI {option}");
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn linux_account_gui_guide_matches_all_localized_actions_and_platform() {
        let _language_guard = crate::i18n::language_test_guard();
        let action_keys = [
            "list",
            "groups",
            "identity",
            "sessions",
            "inspect",
            "create",
            "modify",
            "password",
            "lock",
            "unlock",
            "delete",
            "expire",
            "group_create",
            "group_delete",
            "group_add",
            "group_remove",
            "group_primary",
            "admin_add",
            "admin_groups",
            "guide",
        ];
        for language in crate::i18n::SUPPORTED {
            crate::i18n::set(language);
            let guide = gui_guide("accounts");
            for key in action_keys {
                assert!(
                    guide.contains(crate::i18n::gui_account_text(key)),
                    "Linux account GUI guide ({language}) omits {key}"
                );
            }
            assert!(guide.contains(crate::i18n::gui_account_text("guide_fields_linux")));
            assert!(guide.contains(crate::i18n::gui_account_text("guide_complex_linux")));
            assert!(!guide.contains("TrustedInstaller"));
            assert!(!guide.contains("S-1-5-32-544"));
        }
        crate::i18n::set("es");
    }

    #[test]
    fn every_contextual_gui_guide_lists_its_menu_options() {
        let _language_guard = crate::i18n::language_test_guard();
        crate::i18n::set("es");
        for topic in [
            "accounts",
            "system",
            "native",
            "diagnostics",
            "ssh",
            "adb",
            "utilities",
            "containers",
            "containers-lifecycle",
            "containers-images",
            "containers-volumes",
            "containers-compose",
            "kubernetes",
            "defaults",
            "settings",
            "updates",
            "packages",
            "automation",
            "automation-register",
            "connectivity",
            "storage-partitions",
            "storage-filesystems",
            "storage-volumes",
        ] {
            let guide = gui_guide(topic);
            assert!(
                guide.contains("Menú completo"),
                "falta catálogo para {topic}"
            );
            assert!(
                guide.contains("Campos y argumentos"),
                "falta campos para {topic}"
            );
            assert!(guide.contains("Volver"), "falta navegación para {topic}");
        }
        for (topic, options) in [
            (
                "network",
                [
                    "Estado general de red",
                    "Vaciar caché DNS",
                    "Desconectar NetworkManager",
                ],
            ),
            (
                "boot",
                [
                    "Entradas EFI / NVRAM",
                    "Generar plan seguro",
                    "Cancelar siguiente entrada GRUB",
                ],
            ),
            (
                "services",
                [
                    "Automáticos y estáticos (system)",
                    "Gestionar servicio",
                    "Exportar informe completo",
                ],
            ),
            (
                "git",
                [
                    "Clonar un repositorio Git",
                    "Crear commit",
                    "Estado de autenticación GitHub",
                ],
            ),
            (
                "wine",
                [
                    "Listar prefijos detectados",
                    "Migrar y automatizar prefijo",
                    "Guía de Wine y Proton",
                ],
            ),
            (
                "storage-partitions",
                [
                    "Consultar tabla de un disco",
                    "Retirar firmas de almacenamiento",
                    "Guía de particionado",
                ],
            ),
            (
                "storage-filesystems",
                [
                    "Crear / formatear sistema de archivos",
                    "Montar partición",
                    "Guía de sistemas de archivos",
                ],
            ),
            (
                "storage-volumes",
                [
                    "Crear contenedor LUKS",
                    "Gestionar volúmenes Btrfs",
                    "Guía de cifrado y volúmenes",
                ],
            ),
            (
                "connectivity",
                ["SSH / SCP / SFTP", "Android (ADB)", "Guía de conectividad"],
            ),
        ] {
            let guide = gui_guide(topic);
            for option in options {
                assert!(guide.contains(option), "falta {option} en {topic}");
            }
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn network_boot_and_services_guides_match_each_localized_menu() {
        let _language_guard = crate::i18n::language_test_guard();
        let page_keys = [
            ("network", "network_title"),
            ("network", "network_status"),
            ("network", "network_interfaces"),
            ("network", "network_routes"),
            ("network", "network_dns"),
            ("network", "network_listening"),
            ("network", "network_connections"),
            ("network", "network_flush_dns"),
            ("network", "network_interface_manage"),
            ("network", "network_connect"),
            ("network", "network_disconnect"),
            ("network", "network_guide"),
            ("boot", "boot_title"),
            ("boot", "boot_status"),
            ("boot", "efi_entries"),
            ("boot", "grub_entries"),
            ("boot", "systemd_boot"),
            ("boot", "secure_boot"),
            ("boot", "boot_plan"),
            ("boot", "grub_schedule"),
            ("boot", "grub_cancel"),
            ("boot", "boot_guide"),
            ("services", "services_title"),
            ("services", "services_automatic"),
            ("services", "services_manual"),
            ("services", "services_user"),
            ("services", "services_both"),
            ("services", "services_failed"),
            ("services", "services_manage"),
            ("services", "services_export"),
            ("services", "services_guide"),
        ];
        let prompts = [
            "Interfaz de red (ej. eth0)",
            "Estado: up o down",
            "Nombre exacto de la conexión NetworkManager",
            "Título exacto de la entrada GRUB (ej. Ubuntu)",
            "Ámbito: system o user",
            "Unidad (ej. sshd.service)",
            "Acción: status, start, stop, restart, enable, disable, mask o unmask",
        ];

        for language in crate::i18n::SUPPORTED {
            crate::i18n::set(language);
            for topic in ["network", "boot", "services"] {
                let guide = gui_guide(topic);
                assert!(
                    guide.contains(crate::i18n::text("menu.back")),
                    "{language}: {topic} omits the localized Back action"
                );
                for (page, key) in page_keys.iter().filter(|(page, _)| *page == topic) {
                    let label = crate::i18n::system_page_text(key);
                    assert!(
                        guide.contains(label),
                        "{language}: {topic} guide omits menu label {key} ({label})"
                    );
                    if key.ends_with("_title") {
                        assert!(
                            guide.starts_with(&format!("GUÍA GRÁFICA: {label}")),
                            "{language}: {page} guide title diverges from its menu"
                        );
                    }
                }
                for prompt in prompts.iter().filter(|prompt| match topic {
                    "network" => {
                        prompt.starts_with("Interfaz")
                            || prompt.starts_with("Estado:")
                            || prompt.starts_with("Nombre exacto")
                    }
                    "boot" => prompt.starts_with("Título exacto"),
                    "services" => {
                        prompt.starts_with("Ámbito:")
                            || prompt.starts_with("Unidad")
                            || prompt.starts_with("Acción:")
                    }
                    _ => false,
                }) {
                    let label = crate::i18n::gui_native_prompt(prompt);
                    assert!(
                        guide.contains(label),
                        "{language}: {topic} guide omits GUI field prompt {prompt} ({label})"
                    );
                }
            }
        }
        crate::i18n::set("es");
    }

    #[cfg(not(windows))]
    #[test]
    fn git_gui_guide_matches_every_menu_label_in_all_supported_languages() {
        let _language_guard = crate::i18n::language_test_guard();
        for language in crate::i18n::SUPPORTED {
            crate::i18n::set(language);
            let guide = gui_guide("git");
            let expected = [
                crate::i18n::git_action_text("guide"),
                crate::i18n::tools_text("git_status"),
                crate::i18n::tools_text("git_clone"),
                crate::i18n::tools_text("git_fetch"),
                crate::i18n::tools_text("git_pull"),
                crate::i18n::tools_text("git_log"),
                crate::i18n::tools_text("git_add"),
                crate::i18n::tools_text("git_commit"),
                crate::i18n::tools_text("git_push"),
                crate::i18n::tools_text("git_branch"),
                crate::i18n::tools_text("git_tag"),
                crate::i18n::tools_text("git_release"),
                crate::i18n::tools_text("git_login"),
                crate::i18n::tools_text("gh_repo"),
                crate::i18n::tools_text("gh_prs"),
                crate::i18n::tools_text("gh_releases"),
                crate::i18n::tools_text("gh_auth_status"),
                crate::i18n::git_action_text("version"),
                crate::i18n::git_action_text("help"),
                crate::i18n::git_action_text("native"),
                crate::i18n::git_action_text("diagnose"),
                crate::i18n::git_action_text("repair_index"),
                crate::i18n::git_action_text("repair_remote"),
                crate::i18n::text("menu.back"),
            ];
            for (index, label) in expected.iter().enumerate() {
                assert!(
                    guide.contains(&format!("{:>2}. «{label}»", index + 1)),
                    "guide git lacks menu entry {} for {language}: {label}",
                    index + 1
                );
            }
            assert!(guide.contains("argumentos en una sola línea"));
            assert!(guide.contains("no se expande ni ejecuta una shell"));
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn linux_settings_guide_is_generated_from_the_preferences_catalog() {
        let _language_guard = crate::i18n::language_test_guard();
        crate::i18n::set("es");
        let guide = gui_guide("settings");
        assert!(guide.contains("Menú completo"));
        assert!(guide.contains("Campos y argumentos de la GUI"));
        assert!(guide.contains("No hay campos de texto"));
        for theme in crate::theme::SUPPORTED {
            assert!(
                guide.contains(crate::theme::label(theme)),
                "la guía Ajustes no enumera el tema {theme}"
            );
        }
        for language in std::iter::once("auto").chain(crate::i18n::SUPPORTED.iter().copied()) {
            assert!(
                guide.contains(crate::i18n::language_label(language)),
                "la guía Ajustes no enumera el idioma {language}"
            );
        }
        for category in crate::i18n::SETTINGS_CATEGORY_KEYS {
            assert!(
                guide.contains(crate::i18n::category_text(category)),
                "la guía Ajustes no enumera la categoría {category}"
            );
        }
        assert!(guide.contains(crate::i18n::gui_text("elevation_default")));
        assert!(guide.contains(crate::i18n::gui_text("settings_guide")));
        assert!(!guide.contains("Aplicar ajustes»"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_settings_guide_documents_the_native_form_not_linux_buttons() {
        let guide = gui_guide("settings");
        assert!(guide.contains("Campo «Tema»"));
        assert!(guide.contains("Campo «Idioma»"));
        assert!(guide.contains("«Aplicar ajustes»"));
        assert!(guide.contains("auto"));
        assert!(!guide.contains("Guía de ajustes y visibilidad»"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_guides_use_native_options_and_reject_linux_assumptions() {
        assert!(cli_platform_summary("network").contains("adaptadores"));
        assert!(cli_platform_summary("boot").contains("BCD/UEFI"));
        assert!(cli_platform_summary("wine").contains("NO APLICA"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_gui_guides_never_fall_back_to_linux_menus() {
        let ssh = gui_guide("ssh");
        assert!(ssh.contains("GUÍA GRÁFICA WINDOWS"));
        assert!(ssh.contains("no tienen acciones gráficas"));
        assert!(!ssh.contains("Copiar con SCP"));

        let storage = gui_guide("storage-partitions");
        assert!(storage.contains("no se implementan en la GUI Windows"));
        assert!(!storage.contains("Crear tabla GPT"));

        let automation = gui_guide("automation");
        for option in [
            "Listar scripts registrados",
            "Registrar un script",
            "Ejecutar un script registrado",
            "Editar un script registrado",
            "Eliminar un registro",
        ] {
            assert!(automation.contains(option), "falta la acción GUI {option}");
        }
        assert!(!automation.contains("«»"));

        let containers = gui_guide("containers-lifecycle");
        assert!(containers.contains("no tienen acciones gráficas"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_accounts_guide_documents_admin_sid_and_trustedinstaller_boundary() {
        let _language_guard = crate::i18n::language_test_guard();
        crate::i18n::set("es");
        let guide = gui_guide("accounts");
        assert!(guide.contains("Conceder permisos de administrador"));
        assert!(guide.contains("Ver grupo y miembros administradores"));
        assert!(guide.contains("S-1-5-32-544"));
        assert!(guide.contains("TrustedInstaller es una identidad de servicio"));
        assert!(!guide.contains("sudo/wheel"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_gui_guides_match_every_visible_menu_label_in_all_languages() {
        let _language_guard = crate::i18n::language_test_guard();
        let pages = [
            ("audit", 0),
            ("native", 1),
            ("diagnostics", 2),
            ("defaults", 3),
            ("installable", 4),
            ("automation", 5),
            ("accounts", 8),
        ];
        for language in crate::i18n::SUPPORTED {
            crate::i18n::set(language);
            for (topic, page) in pages {
                let guide = gui_guide(topic);
                let title = crate::gui::windows_menu_title(page)
                    .unwrap_or_else(|| panic!("falta el título de la página Windows {page}"));
                assert!(
                    guide.contains(&format!("Menú completo «{title}»")),
                    "guía Windows/{topic} ({language}) no coincide con el título de la GUI «{title}»"
                );
                let options = crate::gui::windows_menu_labels(page)
                    .into_iter()
                    .chain(std::iter::once(crate::i18n::text("menu.back").to_owned()));
                for option in options {
                    assert!(
                        guide.contains(&option),
                        "guía Windows/{topic} ({language}) no enumera el botón «{option}»"
                    );
                }
                if topic == "accounts" {
                    for key in ["guide_fields", "guide_simple", "guide_complex"] {
                        assert!(
                            guide.contains(crate::i18n::gui_account_text(key)),
                            "Windows account guide ({language}) is missing localized {key}"
                        );
                    }
                    assert!(guide.contains("S-1-5-32-544"));
                }
            }
            if crate::platform::winslim_available() {
                let guide = gui_guide("winslim");
                for option in crate::gui::windows_menu_labels(7)
                    .into_iter()
                    .chain(std::iter::once(crate::i18n::text("menu.back").to_owned()))
                {
                    assert!(
                        !option.trim().is_empty(),
                        "guía Windows/winslim ({language}) contiene una etiqueta vacía"
                    );
                    assert!(
                        guide.contains(&option),
                        "guía Windows/winslim ({language}) no enumera el botón «{option}»"
                    );
                }
            }
        }
        crate::i18n::set("es");
    }

    #[cfg(windows)]
    #[test]
    fn winslim_guide_reflects_conditional_host_availability() {
        let guide = gui_guide("winslim");
        if crate::platform::winslim_available() {
            assert!(guide.contains("Estado de WSCore y NSudo"));
            assert!(guide.contains("Guía de uso y seguridad NSudo"));
            if crate::platform::nsudo_path().is_some() {
                assert!(guide.contains("Abrir el asistente de lanzamiento NSudo"));
            } else {
                assert!(!guide.contains("Abrir el asistente de lanzamiento NSudo"));
            }
        } else {
            assert!(guide.contains("C:\\WSCore o se detecta un lanzador NSudo"));
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn linux_guides_use_native_options_and_scopes() {
        assert!(cli_platform_summary("network").contains("interfaces"));
        assert!(cli_platform_summary("services").contains("scopes"));
        assert!(cli_platform_summary("boot").contains("EFI"));
    }
}
