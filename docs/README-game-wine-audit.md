# game-wine-audit

Auditor especializado para entender instalaciones de juegos y Wine/Proton en CachyOS, Arch y derivados. Complementa a `disk-audit.sh`: aquí la prioridad no es el sistema general, sino explicar qué prefijos y juegos existen, dónde están y cuánto ocupan.

## Uso

```bash
chmod +x game-wine-audit.sh
./game-wine-audit.sh --full \
  --out "$HOME/Informes/game-wine-$(date +%Y%m%d-%H%M)"
```

Descubre automáticamente el directorio personal y los montajes bajo `/mnt`, `/media` y `/run/media`. Para revisar solo un disco:

```bash
./game-wine-audit.sh --no-home --no-mounts \
  --root /mnt/JuegosLinux
```

También acepta varias rutas:

```bash
./game-wine-audit.sh --root /mnt/JuegosLinux --root /mnt/JuegosWindows
```

## Qué distingue

Los prefijos se clasifican como:

- `default-wine`: normalmente `~/.wine`;
- `steam-proton`: `steamapps/compatdata/<appid>/pfx`;
- `lutris-prefix`, `heroic-prefix`, `bottles-prefix` y `umu-prefix`;
- `runner-default`: plantillas `files/share/default_pfx` de Proton/Wine;
- `game-prefix` y `wine-prefix-unknown` para los demás.
- `candidate-drive_c` para una carpeta `drive_c` sin `system.reg`; se conserva como pista, pero no se trata como prefijo válido sin revisión.

Las plantillas `default_pfx` se muestran aparte para que no se confundan con instalaciones independientes. No se eliminan ni se modifican.

## Informes más importantes

- `wine-prefixes.tsv`: todos los prefijos con tamaño del root, tamaño de `drive_c`, tipo, AppID de Steam y ambas rutas. Si un prefijo está en la raíz de un disco montado, evita contar todo el disco como tamaño del prefijo.
- `wine-prefix-details.tsv` y `wine-prefix-details.txt`: arquitectura, versión de Windows registrada, runner, bloqueos, ejecutables, instaladores, carpetas de programas y marcadores Wine por prefijo.
- `wine-prefix-programs.tsv`: nombres de programas encontrados en las entradas `Uninstall` del registro de Wine.
- `wine-prefix-content.tsv`: tamaños de `Program Files`, `Program Files (x86)`, perfiles de usuario, `Windows`, `Games` y directorios no estándar junto al prefijo. Sirve para detectar prefijos incrustados accidentalmente.
- `wine-prefix-binaries.tsv`: todos los `.exe` y `.msi` encontrados, con tipo, tamaño y ruta completa.
- `wine-prefix-summary.tsv`: cantidad y tamaño total por tipo de prefijo.
- `wine-drive-c-candidates.tsv`: carpetas `drive_c` que no tienen `system.reg` en su padre inmediato.
- `wine-mount-root-candidates.tsv`: raíces de discos montados que contienen restos de un prefijo. Se informan aparte y nunca se consideran migrables automáticamente.
- `wine-trash-prefixes.tsv`: prefijos que siguen dentro de la papelera; se excluyen del inventario operativo para no duplicar tamaños ni ofrecerlos para migración.
- `wine-prefix-overlaps.tsv`: prefijos anidados o superpuestos.
- `steam-games.tsv`: manifiestos Steam, AppID, nombre, biblioteca, directorio y tamaño.
- `steam-unmanaged-directories.tsv`: directorios bajo `steamapps/common` que requieren revisión.
- `steam-duplicate-appids.tsv`: mismo AppID encontrado en varias bibliotecas.
- `lutris-games.tsv`: configuraciones Lutris con nombre, AppID, runner, prefijo, estado del prefijo, ejecutable y ruta resuelta; también marca rutas inexistentes o alias/symlinks.
- `lutris-duplicate-slugs.tsv`: juegos o servicios repetidos en varias configuraciones Lutris, con sus prefijos y ficheros.
- `heroic-configs.tsv`: configuraciones JSON de Heroic; con `jq` puede extraer nombre, instalación y prefijo.
- `bottles.tsv`: botellas y tamaños.
- `runners.tsv`: runners Proton/Wine compartidos, separados de los prefijos de juegos.
- `game-roots.tsv`: directorios principales de juegos detectados.
- `configuration-validation.tsv`: validación de rutas reales de Heroic, Lutris,
  UMU y Steam; marca JSON inválido, prefijos inexistentes, runners ausentes y
  bibliotecas Steam no disponibles.
- `configuration-files.tsv`: configuraciones conocidas por aplicación y tipo.
- `configuration-databases.tsv`: ficheros SQLite/DB, tamaño, descripción y
  comprobación de la cabecera SQLite.
- `configuration-binaries.tsv`: bases de datos y formatos binarios o
  semibinarios como VDF y ACF, sin intentar modificar su contenido.

La detección de bases de datos es descriptiva: no altera SQLite ni reescribe
formatos binarios. Si una ruta interna está codificada en uno de ellos, el
informe la marca para reconfiguración desde la aplicación.

Las rutas se deduplican antes de catalogarlas. Una carpeta que tenga a la vez `system.reg` y `drive_c` solo aparece una vez. En modo `--full` también se revisan ubicaciones habituales del sistema como `/opt`, `/usr/local/share`, `/var/lib/flatpak`, `/var/lib/steam` y `/srv`.

## Configuración por equipo

Usa el mismo fichero opcional que el auditor general, `~/.config/disk-audit.conf`:

```bash
EXTRA_SCAN_ROOTS=(
    "/mnt/JuegosLinux"
    "/mnt/JuegosWindows"
    "/media/romen/DiscoExterno"
)
AUTO_MOUNT_ROOTS=1
INCLUDE_HOME=1
```

El script solo genera informes. Para borrar algo se debe seleccionar después con `disk-clean.sh`, revisando previamente sus referencias y dependencias.
