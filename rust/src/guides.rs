//! Guías operativas en texto para que cada acción tenga contexto, requisitos y
//! un flujo seguro tanto en la CLI como en las interfaces gráficas.

#![allow(dead_code)]

use crate::common::Context;

pub fn help() -> &'static str {
    "guide [cli|gui] [all|audit|packages|software|git|gh|automation|clean|storage|system|services|accounts|native|network|boot|registry|diagnostics|wine|defaults|containers|kubernetes|ssh|adb|utilities|actions]"
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
  git / gh    Git avanzado y GitHub CLI
  automation  automatizaciones y acciones declarativas
  clean       limpieza con selección segura
  storage     discos, particiones, formatos y volúmenes
  system      estado del sistema y herramientas nativas
  services    servicios, scopes, origen y gestión
  accounts    usuarios, grupos, sesiones y permisos
  network     interfaces, NetworkManager, rutas, DNS y puertos
  boot        EFI, GRUB, systemd-boot y Secure Boot
  registry    configuración y registros por plataforma
  diagnostics salud, dependencias y capacidades
  wine        prefijos Wine/Proton y migraciones
  containers  Docker/Podman, imágenes, volúmenes y Compose
  kubernetes  recursos, despliegues, rollouts y port-forward
  ssh / adb   acceso remoto y dispositivos Android
  utilities   utilidades detectadas e instalables
  defaults    rutas, configuración y valores efectivos
  actions     catálogo de acciones y contratos para automatizar

En la GUI, el botón «Guía de uso» de cada familia abre el mismo contenido
que este comando, junto con sus botones de gestión.
"#;

const GIT: &str = r#"GUÍA: GIT Y GITHUB (git + gh)

Requisitos:
  git para repositorios locales y remotos. gh (GitHub CLI) solo para login,
  repositorios, pull requests y releases de GitHub. Comprueba disponibilidad
  con `ltools doctor` o `ltools native tools status`.

Flujo recomendado:
  1. `ltools git status --repo RUTA` revisa rama y cambios.
  2. `ltools git clone URL --destination RUTA --dry-run` valida el destino.
  3. `ltools git fetch --repo RUTA` actualiza referencias sin mezclar cambios.
  4. `ltools git pull --repo RUTA` integra cambios tras revisar el estado.
  5. `ltools git add --repo RUTA --path FICHERO` prepara rutas explícitas.
  6. `ltools git commit --repo RUTA --message "Mensaje"` crea el commit.
  7. `ltools git push --repo RUTA --remote origin --branch main` publica.

Opciones cubiertas por la GUI y la CLI:
  status, log, clone, fetch, pull, add, commit, push, branch, tag y release.
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
  El login requiere navegador/credenciales y red; LTools no guarda ni
  imprime secretos. Si `gh` no está instalado, se informa y no se sustituye
  silenciosamente por una descarga.

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

Acciones: login, auth-status, repo, prs y releases. El login es interactivo;
repo/prs/releases son consultas. Proporciona `OWNER/REPO` cuando no estés
dentro del repositorio. Las operaciones remotas dependen de red, permisos y
el token configurado por `gh`; LTools nunca solicita el token como argumento.

Ejemplo seguro:
  ltools git gh auth-status
  ltools git gh repo --repo OWNER/REPO
  ltools git gh prs --repo OWNER/REPO
  ltools git gh releases --repo OWNER/REPO
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
parted (GPT/MBR, crear/borrar/redimensionar), mkfs/fsck/mount/swap, LUKS,
LVM, Btrfs, ZFS y RAID mdadm.

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
crear/eliminar grupos, membresías y grupo principal.

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
wine, defaults, containers, kubernetes, ssh, adb, utilities y actions.
"#;

fn cli_guide(topic: &str) -> String {
    if topic == "all" {
        return CLI_INDEX.to_owned();
    }
    let (title, queries, management, arguments, result) = match topic {
        "git" | "gh" => (
            "GIT Y GITHUB",
            "status, log, clone, fetch, pull, branch, tag, gh auth-status, repo, prs y releases",
            "add, commit, push y release, siempre con repositorio, remoto y rama explícitos",
            "--repo, --url, --destination, --remote, --branch, --path, --message, --limit y notas",
            "estado, referencias, historial, cambios preparados o información remota",
        ),
        "network" => (
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
        "storage" => (
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
            "create, edit, password, lock, unlock, expire, membership y primary-group",
            "--user, --group, --member, --scope y objetivo explícito",
            "identidad, sesiones, membresías y cambios protegidos",
        ),
        "containers" => (
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
            "--automatic, --preview, --include-personal, --all-known, --ask-each y las opciones clásicas de paquetes",
            "candidatos, rutas protegidas, plan y elementos retirados",
        ),
        "automation" | "actions" => (
            "AUTOMATIZACIÓN Y ACCIONES",
            "list del registro y del catálogo declarativo",
            "add, modify, run, remove y ejecución por identificador estable",
            "--name, --program, --args, --cwd, --id, --target, --format y --dry-run",
            "acción, compatibilidad, requisitos, plan y salida registrada",
        ),
        "aliases" | "alias" => (
            "GESTOR DE ALIAS",
            "list para ver alias, path para localizar el registro y doctor para revisar la instalación",
            "ensure crea el registro base y el lanzador; add, enable, disable y remove gestionan alias personalizados",
            "add NOMBRE COMANDO [ARG...]; los argumentos se conservan separados y solo se permiten comandos LTools conocidos",
            "registro persistente, alias activos, lanzador gestionado y estado de PATH",
        ),
        "diagnostics" | "utilities" | "defaults" | "audit" | "packages" | "software" | "system"
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

Cada pantalla conserva «Volver» y las acciones mutables abren confirmación.
Para una pantalla concreta, abre su botón «Guía»; allí deben aparecer todos
los botones de esa pantalla, sus campos y el flujo de verificación.
La CLI tiene una guía separada con sus propias opciones: guide cli <familia>.
"#;

#[cfg(not(windows))]
fn gui_storage_guide() -> String {
    r#"GUÍA GRÁFICA: ALMACENAMIENTO Y PARTICIONES (LINUX)

Esta es la referencia completa del menú. La guía no sustituye botones por
comandos: indica dónde entrar, qué botón pulsar, qué campos aparecen y cómo
verificar el resultado. Las consultas no modifican el equipo. Las acciones
que pueden borrar, formatear, desmontar o cambiar volúmenes muestran una
confirmación antes de ejecutarse.

1. PANTALLA PRINCIPAL: «Herramientas nativas»

  • «Resumen de espacio y montajes»: consulta espacio, dispositivos,
    sistemas de archivos y montajes activos.
  • «Discos y particiones»: lista discos y particiones detectados.
  • «Montajes activos»: muestra qué está montado y dónde.
  • «Mapa desplegable de discos y rutas»: abre el árbol de carpetas y
    archivos con tamaño acumulado, permisos observados, errores de acceso y
    explicación de rutas estándar. La profundidad se puede ampliar desde la
    CLI con `--depth` y el inventario se puede exportar a JSON/TSV.
  • «Particionado y tablas»: abre todas las operaciones de tabla y
    partición descritas en la sección 2.
  • «Sistemas de archivos»: abre formato, etiquetas, comprobación, montaje
    y swap descritos en la sección 3.
  • «Cifrado y volúmenes»: abre LUKS, LVM, Btrfs, ZFS y RAID mdadm.
  • «Herramientas de almacenamiento detectadas»: comprueba qué capacidades
    están realmente disponibles en este equipo.
  • «Abrir el gestor nativo de particiones»: abre el gestor gráfico externo
    disponible para una revisión visual adicional; sus cambios siguen siendo
    cambios reales y deben verificarse después en LTools.
  • «Revisar limpieza»: solo prepara una revisión de candidatos; no borra
    discos ni datos por sí solo.
  • «Guía de particionado y protecciones»: vuelve a esta explicación.
  • «Volver»: regresa al menú anterior; aparece también en cada submenú.

2. «Particionado y tablas»

Consultas y diagnóstico:
  • «Consultar tabla de un disco»: campo «Disco completo /dev/...».
    Muestra tabla, particiones, tamaños, tipos y flags.
  • «Consultar espacio libre»: mismo campo; localiza huecos sin asignar antes
    de crear o ampliar una partición.
  • «Inspeccionar dispositivo (probe)»: mismo campo; verifica que el objetivo
    sea el disco esperado antes de tocarlo.
  • «Comprobar alineación»: «Disco completo /dev/...», «Número de partición»
    y «Alineación: minimal u optimal». Es una comprobación, no una reparación.

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
  • «Borrar firmas (wipefs)»: campo «Dispositivo /dev/...». Retira firmas que
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

LVM, Btrfs, ZFS y RAID:
  • «Operación LVM»: «Operación: pvcreate, pvremove, vgcreate, vgremove,
    lvcreate, lvremove, lvextend, lvreduce»; opcionalmente PV/dispositivo,
    grupo de volúmenes, nombre VG/LV y tamaño (ej. 20G). Revisa dependencias
    antes de eliminar o reducir.
  • «Operación Btrfs»: operación, «Montaje o subvolumen absoluto», destino de
    snapshot, dispositivo secundario y nuevo tamaño cuando corresponda.
    Úsalo para subvolúmenes, snapshots, balanceo o ampliación solo con el
    montaje y destino identificados.
  • «Operación ZFS»: operación, «Pool, dataset o snapshot» y, al crear un
    pool, «Dispositivos separados por coma». Un pool incorrecto puede destruir
    metadatos de los dispositivos seleccionados.
  • «Operación RAID mdadm»: operación create/add/remove/fail/stop/grow,
    dispositivo md, nivel RAID, miembros separados por coma, miembro
    individual o número y cantidad para grow. Comprueba nivel, miembros y
    estado de sincronización antes de confirmar.

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
  • «Mapa desplegable de discos y rutas» muestra el árbol de cada volumen,
    tamaños acumulados, atributos de solo lectura, errores de acceso y la
    explicación de Windows, Program Files, ProgramData, Users y AppData.
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

fn gui_catalog_guide(topic: &str) -> String {
    let (title, menu, options, fields, process) = match topic {
        "accounts" => (
            "CUENTAS Y PERMISOS", "Usuarios, grupos y sesiones",
            &["Listar cuentas locales", "Listar grupos y miembros", "Ver mi identidad y grupos", "Ver sesiones abiertas", "Inspeccionar una cuenta", "Crear cuenta", "Editar cuenta y grupos", "Cambiar contraseña", "Bloquear cuenta", "Desbloquear cuenta", "Eliminar cuenta", "Configurar caducidad", "Crear grupo", "Eliminar grupo", "Añadir usuario a grupo", "Retirar usuario de grupo", "Cambiar grupo principal", "Guía de cuentas y permisos", "Volver"][..],
            "Inspeccionar: usuario. Crear: usuario, descripción, shell, grupos y opciones. Editar: usuario, descripción, shell y grupos. Contraseña: usuario y confirmación. Bloquear, desbloquear y eliminar: usuario. Caducidad: usuario y fechas. Grupos: nombre. Membresías: usuario y grupo.",
            "Consulta identidad, cuentas, grupos y sesiones. Para gestionar, identifica usuario/grupo, revisa el alcance, confirma y vuelve a listar.",
        ),
        "system" => (
            "SISTEMA, RED Y SEGURIDAD", "Sistema",
            &["Estado del sistema", "Usuarios, grupos y sesiones", "Red, rutas, DNS y puertos escuchando", "Arranque, EFI y cargador del sistema", "Registro", "Diagnóstico", "Servicios del sistema", "Guía del sistema", "Volver"][..],
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
            "DEPENDENCIAS Y DIAGNÓSTICO", "Dependencias",
            &["Doctor", "Estado de herramientas detectadas", "Elegir e instalar un paquete", "Instalar herramienta nativa", "Diagnóstico", "Guía de dependencias", "Volver"][..],
            "«Instalar herramienta nativa» pide el identificador de herramienta.",
            "Ejecuta Doctor y Estado antes de instalar. Selecciona una herramienta, revisa disponibilidad y confirma solo la instalación explícita.",
        ),
        "ssh" => (
            "CONECTIVIDAD Y SSH", "Conectividad / SSH, SCP y SFTP",
            &["SSH, SCP, SFTP y Android", "SSH, SCP y SFTP", "Conectar por SSH", "Copiar con SCP", "Abrir SFTP", "Guía de conectividad", "Guía de SSH, SCP y SFTP", "Volver"][..],
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
            &["Estado de utilidades", "Instalar utilidad", "Guía de utilidades", "Volver"][..],
            "«Instalar utilidad» pide el identificador de la utilidad.",
            "Consulta disponibilidad, selecciona el identificador exacto, revisa el gestor y confirma la instalación.",
        ),
        "containers" => (
            "DOCKER Y PODMAN", "Docker/Podman y sus pantallas",
            &["Contenedores", "Imágenes", "Volúmenes y redes", "Compose y diagnósticos", "Guía de Docker y Podman", "Descargar imagen", "Crear y ejecutar contenedor", "Iniciar contenedor", "Detener contenedor", "Reiniciar contenedor", "Eliminar contenedor", "Ver logs del contenedor", "Ejecutar comando en contenedor", "Inspeccionar contenedor", "Estadísticas de contenedor", "Procesos del contenedor", "Puertos publicados", "Cambios del contenedor", "Pausar contenedor", "Reanudar contenedor", "Terminar contenedor", "Renombrar contenedor", "Copiar archivos", "Limpiar contenedores detenidos", "Inspeccionar imagen", "Historial de imagen", "Construir imagen", "Etiquetar imagen", "Eliminar imagen", "Limpiar imágenes no usadas", "Listar volúmenes", "Inspeccionar volumen", "Crear volumen", "Eliminar volumen", "Limpiar volúmenes no usados", "Listar redes", "Inspeccionar red", "Crear red", "Eliminar red", "Limpiar redes no usadas", "Operación Compose guiada", "Información del motor", "Uso de espacio del motor", "Limpieza global del motor", "Volver"][..],
            "Contenedor: motor y nombre; ejecutar: imagen, nombre, puertos, volúmenes y comando; copiar: contenedor, origen y destino. Imagen: objetivo, ruta y etiqueta. Volumen/red: nombre y motor. Compose: operación, fichero, servicio y comando.",
            "Consulta motor, imagen, contenedor, volumen o red antes de modificar. Revisa nombre, imagen, rutas y puertos; confirma y repite la inspección.",
        ),
        "kubernetes" => (
            "KUBERNETES", "Kubernetes",
            &["Aplicar manifiesto", "Eliminar recurso", "Escalar deployment", "Reiniciar rollout", "Abrir port-forward", "Guía de Kubernetes", "Volver"][..],
            "Manifiesto: fichero y namespace. Recurso: tipo/nombre y namespace. Escalar: deployment y réplicas. Rollout: deployment. Port-forward: pod/servicio y puertos.",
            "Comprueba contexto, namespace y recurso. Usa inspección/diff antes de aplicar, eliminar, escalar o reiniciar y verifica el estado después.",
        ),
        "defaults" => (
            "VALORES PREDETERMINADOS Y AJUSTES", "Valores predeterminados / Ajustes",
            &["Valores predeterminados", "Registro", "Tema visual", "Idioma", "Visibilidad de categorías", "Guía de ajustes y visibilidad", "Guía de valores predeterminados", "Volver"][..],
            "Ajustes contiene todos los temas, todos los idiomas y un interruptor de visibilidad por categoría; cada preferencia se aplica al pulsarla.",
            "Cambia una preferencia, comprueba la vista y revisa la nota de reinicio cuando la visibilidad lo requiera.",
        ),
        "packages" | "software" => (
            "PAQUETES Y SOFTWARE", "Herramientas instalables / Paquetes y software",
            &["Git / GitHub", "Paquetes y software", "SSH, SCP, SFTP y Android", "Docker y Podman", "Kubernetes", "Utilidades instalables", "Buscar paquete", "Instalar paquete", "Tiendas y gestores", "Guía de herramientas instalables", "Guía de paquetes y software", "Volver"][..],
            "Buscar paquete pide nombre; Instalar paquete pide el número del candidato mostrado. Cada familia tiene sus propios campos.",
            "Busca primero, selecciona un candidato concreto, revisa gestor y versión, confirma y verifica la instalación.",
        ),
        "automation" => (
            "AUTOMATIZACIÓN", "Automatización, scripts registrados y registro",
            &["Scripts registrados", "Registrar nuevo script", "Recargar listado", "Ejecutar script", "Editar script", "Retirar script", "Registrar script", "Guía de automatización", "Guía para registrar scripts", "Volver"][..],
            "Registrar script pide nombre, programa, directorio de trabajo y argumentos. Cada script registrado ofrece ejecutar, editar y retirar.",
            "Registra ejecutables y argumentos separados, recarga y revisa el listado; ejecuta por identificador y retira solo después de confirmar.",
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
        return GUI_INDEX.to_owned();
    }
    match topic {
        "storage" => gui_storage_guide(),
        "network" => r#"GUÍA GRÁFICA: RED

Menú completo «Red, rutas, DNS y puertos escuchando»:
  1. «Estado general de red»: resumen de conectividad e interfaz activa.
  2. «Interfaces y direcciones»: interfaces, estados y direcciones.
  3. «Tabla de rutas»: rutas y puerta de enlace.
  4. «DNS y resolutores»: resolutores configurados y respuesta disponible.
  5. «Puertos escuchando»: servicios que escuchan y sus procesos.
  6. «Conexiones NetworkManager»: perfiles detectados y su estado.
  7. «Vaciar caché DNS»: acción protegida para renovar la resolución.
  8. «Activar / desactivar interfaz»: campos «Interfaz de red» y «Estado:
     up o down».
  9. «Conectar NetworkManager»: campo «Nombre exacto de la conexión
     NetworkManager».
 10. «Desconectar NetworkManager»: el mismo campo; puede cortar la conexión.
 11. «Guía de red»: vuelve a esta guía.
 12. «Volver»: regresa al menú anterior.

Proceso simple: ejecuta 1–6 y compara interfaz, ruta predeterminada, DNS,
puertos y perfil antes de modificar nada.

Proceso complejo: usa 8, 9 o 10, rellena el objetivo exacto, revisa la
confirmación y repite 1–3. No desconectes la interfaz de una sesión remota.
"#
        .to_owned(),
        "boot" => r#"GUÍA GRÁFICA: ARRANQUE, EFI Y GRUB

Menú completo «Arranque, EFI y cargador del sistema»:
  1. «Estado general del arranque»: resumen de cargador y modo de arranque.
  2. «Entradas EFI / NVRAM»: entradas UEFI disponibles y orden.
  3. «Entradas GRUB»: entradas reconocidas por GRUB.
  4. «Estado de systemd-boot»: loader y entradas systemd-boot.
  5. «Estado de Secure Boot»: estado de Secure Boot si el equipo lo expone.
  6. «Generar plan seguro»: prepara un plan de lectura/revisión.
  7. «Programar siguiente entrada GRUB»: campo «Título exacto de la entrada
     GRUB (ej. Ubuntu)»; afecta solo al siguiente arranque.
  8. «Cancelar siguiente entrada GRUB»: elimina la selección programada.
  9. «Guía de arranque, EFI y GRUB»: vuelve a esta guía.
 10. «Volver»: regresa al menú anterior.

Proceso simple: pulsa 1–6 y anota el título exacto de la entrada antes de
usar cualquier acción de cambio.

Proceso complejo: pulsa 7, escribe el título exacto, revisa la confirmación y
verifica con 1–3. Para deshacerlo usa 8 y vuelve a consultar el estado.
"#
        .to_owned(),
        "services" => r#"GUÍA GRÁFICA: SERVICIOS

Menú completo «Servicios del sistema»:
  1. «Automáticos y estáticos (system)»: unidades del sistema configuradas
     para iniciar automáticamente o estáticas.
  2. «Manuales / desactivados (system)»: unidades del sistema no automáticas.
  3. «Servicios del usuario»: unidades del ámbito de usuario.
  4. «Todos: sistema y usuario»: inventario combinado.
  5. «Servicios fallidos y journal»: fallos y eventos relacionados.
  6. «Gestionar servicio»: campos «Ámbito: system o user», «Unidad (ej.
     sshd.service)» y «Acción: status, start, stop, restart, enable, disable,
     mask o unmask».
  7. «Exportar informe completo»: guarda la vista combinada en TSV.
  8. «Guía de servicios»: vuelve a esta guía.
  9. «Volver»: regresa al menú anterior.

Proceso simple: revisa 1–5 y compara estado, unidad, ámbito, origen,
dependencias y eventos.

Proceso complejo: en 6 elige ámbito, unidad y acción exactos; revisa la
confirmación, ejecuta y vuelve a consultar 1–5. Usa 7 para conservar evidencia.
"#
        .to_owned(),
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
 18. «Volver»: regresa al menú anterior.

Proceso simple: empieza por Estado, Historial, identidad o autenticación.
Para Clonar completa URL y destino. Para GitHub usa repositorio, pull request
o release y revisa el remoto antes de confirmar.

Proceso complejo: Estado → Preparar cambios → Commit → rama/tag → Push o
Release. Revisa el diff y remoto; nunca pegues tokens en los campos.
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

#[cfg(windows)]
fn cli_platform_summary(topic: &str) -> &'static str {
    match topic {
        "network" => "PERFIL WINDOWS: los objetivos son adaptadores, perfiles, rutas, DNS y conexiones Windows.",
        "boot" => "PERFIL WINDOWS: se ofrece inspección BCD/UEFI y Secure Boot; las opciones EFI/GRUB de Linux no aplican.",
        "services" => "PERFIL WINDOWS: se gestionan servicios y eventos Windows; no existen scopes systemd.",
        "storage" => "PERFIL WINDOWS: los objetivos son discos, particiones, volúmenes y letras de unidad Windows.",
        "wine" => "PERFIL WINDOWS: WINE/PROTON NO APLICA al ejecutable Windows nativo.",
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

GitHub CLI es `gh.exe`: `gh auth status`, `gh repo view ORG/REPO`,
`gh pr list --repo ORG/REPO` y `gh release list --repo ORG/REPO`. No se usa
`sudo`; la red y las credenciales las gestiona Git for Windows/Windows.
"#
        }
        "gh" => {
            r#"WINDOWS: GH.EXE

La dependencia es `gh.exe` en PATH. Usa `ltools git gh auth-status`,
`ltools git gh repo --repo ORG/REPO`, `prs` y `releases`; el login abre el
flujo oficial de `gh.exe`. Los paths de trabajo son Windows y la autenticación
queda en el almacén/configuración de gh, nunca en argumentos de LTools.
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
        "git" => Ok(GIT),
        "gh" => Ok(GH),
        "automation" => Ok(AUTOMATION),
        "clean" => Ok(CLEAN),
        "storage" => Ok(STORAGE),
        "system" => Ok(SYSTEM),
        "services" => Ok(SERVICES),
        "accounts" => Ok(ACCOUNTS),
        "network" => Ok(NETWORK),
        "boot" => Ok(BOOT),
        "registry" => Ok(REGISTRY),
        "diagnostics" => Ok(DIAGNOSTICS),
        "wine" => Ok(WINE),
        "defaults" => Ok(DEFAULTS),
        "containers" => Ok(CONTAINERS),
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
        "git" => "git",
        "gh" | "github" => "gh",
        "automation" | "automations" => "automation",
        "clean" | "cleanup" => "clean",
        "storage" | "disks" | "partitions" => "storage",
        "system" | "native" => "system",
        "services" | "service" => "services",
        "accounts" | "users" => "accounts",
        "network" | "red" => "network",
        "boot" | "efi" | "grub" => "boot",
        "registry" | "records" => "registry",
        "diagnostics" | "doctor" => "diagnostics",
        "wine" | "proton" | "prefix" => "wine",
        "defaults" | "settings" => "defaults",
        "containers" | "docker" | "podman" => "containers",
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
        for topic in ["git / gh", "network", "boot", "services", "wine", "actions"] {
            assert!(INDEX.contains(topic), "falta {topic}");
        }
    }

    #[test]
    fn git_guide_covers_gh_and_safe_flow() {
        let guide = cli_guide("git");
        assert!(guide.contains("GIT Y GITHUB") && guide.contains("--dry-run"));
        assert!(guide.contains("clone") && guide.contains("push") && guide.contains("release"));
        assert!(!guide.contains("git status") && !guide.contains("gh auth status"));
    }

    #[test]
    fn gui_guides_explain_visual_simple_and_complex_flows() {
        let guide = gui_guide("storage");
        assert!(guide.contains("GUÍA GRÁFICA"));
        assert!(guide.contains("Proceso simple") && guide.contains("Proceso complejo"));
        assert!(guide.contains("Crear tabla GPT"));
        assert!(guide.contains("Operación LVM") && guide.contains("Operación RAID mdadm"));
        assert!(!guide.contains("lsblk"));
    }

    #[test]
    fn every_contextual_gui_guide_lists_its_menu_options() {
        for topic in [
            "accounts",
            "system",
            "native",
            "diagnostics",
            "ssh",
            "adb",
            "utilities",
            "containers",
            "kubernetes",
            "defaults",
            "packages",
            "automation",
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
        ] {
            let guide = gui_guide(topic);
            for option in options {
                assert!(guide.contains(option), "falta {option} en {topic}");
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_guides_use_native_options_and_reject_linux_assumptions() {
        assert!(cli_platform_summary("network").contains("adaptadores"));
        assert!(cli_platform_summary("boot").contains("BCD/UEFI"));
        assert!(cli_platform_summary("wine").contains("NO APLICA"));
    }

    #[cfg(not(windows))]
    #[test]
    fn linux_guides_use_native_options_and_scopes() {
        assert!(cli_platform_summary("network").contains("interfaces"));
        assert!(cli_platform_summary("services").contains("scopes"));
        assert!(cli_platform_summary("boot").contains("EFI"));
    }
}
