# disk-audit

`disk-audit.sh` es una auditoría de solo lectura para CachyOS, Arch y derivados. Está pensada para responder preguntas como:

- qué discos y montajes existen y cuánto espacio queda;
- qué ocupa cada ruta principal;
- qué paquetes huérfanos o externos/AUR hay;
- qué gestores de paquetes están realmente instalados y qué almacenes/cachés conocen;
- qué paquetes instalados pertenecen al sistema o al usuario, aunque no sean huérfanos;
- qué archivos descargados son `.deb`, `.rpm`, `.pkg.tar.*`, `.apk`, `.xbps`, `.pisi`, `.ipk`, `.pkg`, `.flatpak`, `.snap`, paquetes Slackware/Gentoo y formatos relacionados;
- qué aplicaciones aparecen en pacman, Flatpak, AppImage y lanzadores `.desktop`;
- cuántos prefijos Wine/Proton existen, dónde están y cuánto ocupan;
- dónde están Steam, Lutris, Heroic, Bottles y sus datos;
- qué máquinas virtuales y discos virtuales existen;
- qué archivos grandes hay;
- qué archivos son duplicados exactos mediante SHA-256.

El programa **no elimina, mueve ni modifica archivos**. La búsqueda de duplicados es opcional porque puede tardar y leer muchos datos.

## Uso

```bash
chmod +x disk-audit.sh
./disk-audit.sh
```

El informe se crea en un directorio como:

```text
./disk-audit-equipo-20260828-235900/
```

Para una auditoría más amplia y duplicados:

```bash
./disk-audit.sh --full --duplicates --min-size-mb 100
```

Para incluir explícitamente un disco o ruta concreta:

```bash
./disk-audit.sh --root /mnt/JuegosLinux --root /mnt/JuegosWindows
```

Para revisar gestores y almacenes sin recorrer todos los discos:

```bash
./disk-audit.sh --packages-only --out "$HOME/Informes/package-audit-$(date +%Y%m%d-%H%M)"
```

Este modo consulta solo los gestores disponibles —por ejemplo `pacman`, `paru`, `yay`, `pikaur`, `trizen`, `aura`, `pamac`, `apt/dpkg`, `rpm/dnf/yum`, `zypper`, `apk`, `xbps`, `pkg`, `snap`, `flatpak`, `brew` y `nix`— y sus cachés conocidas. Las carpetas de usuario habituales de descargas se pueden añadir con `--root`.

Para auditar únicamente rutas concretas, sin recorrer el directorio personal:

```bash
./disk-audit.sh --no-home --no-mounts --root /mnt/JuegosLinux
```

También se puede evitar el descubrimiento automático de montajes:

```bash
./disk-audit.sh --no-mounts --root "$HOME" --root /mnt/JuegosLinux
```

Se recomienda ejecutarlo como usuario normal, no con `sudo`, para que `$HOME` y los datos de aplicaciones correspondan a tu usuario. Algunas rutas del sistema pueden aparecer incompletas si no hay permisos de lectura.

## Ficheros de salida

- `summary.txt`: resumen y lista de informes.
- `system.txt`: `df`, `lsblk`, montajes, kernel y distribución.
- `directory-usage-*.txt`: tamaño de los directorios principales de cada raíz.
- `packages.txt`, `packages-orphans.txt`, `packages-foreign.txt`: inventario pacman.
- `packages-inventory.tsv`: inventario unificado de las bases de datos detectadas.
- `packages-system.tsv` y `packages-user.tsv`: separación por ámbito; incluye paquetes no huérfanos.
- `packages-by-scope.tsv`: recuento por gestor, ámbito, origen y motivo.
- `package-managers.tsv` y `package-managers.txt`: gestores disponibles, ejecutables, versiones y almacenes conocidos.
- `package-stores.tsv` y `package-stores.txt`: cachés, bases de datos, revisiones y tiendas realmente existentes, con tamaño y ámbito.
- `package-summary.txt`: explicación legible de la cobertura, los recuentos, el significado de cada categoría y las limitaciones de seguridad.
- `scan-scope.tsv` y `scan-scope.txt`: cada ruta inspeccionada, su función y el motivo por el que se incluye.
- `package-artifacts.tsv`: archivos de paquetes encontrados y clasificados por formato y ubicación.
- `packages-by-installed-size.txt`: tamaños instalados según `pacman -Qi`.
- `flatpak.txt` y `flatpak-size.txt`: aplicaciones, runtimes y tamaños Flatpak.
- `desktop-applications.tsv`: lanzadores `.desktop` detectados.
- `desktop-duplicate-names.tsv`: nombres de aplicaciones repetidos entre lanzadores.
- `appimages.tsv`: AppImages con tamaño y ruta.
- `wine-prefixes.tsv`: prefijos detectados y clasificación aproximada.
- `game-related-paths.tsv` y `game-locations.txt`: Steam/Lutris/Heroic/Bottles.
- `virtual-machines.tsv`: discos y definiciones de máquinas virtuales.
- `installers-and-archives.tsv`: ISO, imágenes, instaladores y archivos comprimidos.
- `build-artifacts.tsv`: `target`, `node_modules`, `build`, `dist` y otras carpetas regenerables.
- `large-files.tsv`: archivos de más de 500 MiB.
- `caches-and-trash.tsv`: cachés y papeleras conocidas.
- `duplicates.tsv`: duplicados exactos si se usó `--duplicates`.

## Configuración adaptable

Se puede crear `~/.config/disk-audit.conf` para mantener rutas que cambien entre equipos:

```bash
# Añade rutas adicionales al inventario.
EXTRA_SCAN_ROOTS=(
    "/mnt/JuegosLinux"
    "/mnt/JuegosWindows"
    "/media/romen/DiscoExterno"
)

# Valores por defecto para futuras ejecuciones.
AUTO_MOUNT_ROOTS=1
INCLUDE_HOME=1
DUPLICATE_MIN_MB=100
TOP_N=150

# Solo para --packages-only: almacenes o carpetas adicionales de paquetes.
PACKAGE_SCAN_ROOTS=(
    "/mnt/RepositorioPaquetes"
    "$HOME/Descargas"
)
```

También puede pasarse otra configuración con `--config /ruta/mi-config.conf`.

## Interpretación importante

Los prefijos detectados dentro de `files/share/default_pfx` son plantillas internas de Proton/Wine y no deben eliminarse como si fueran prefijos de juegos. Los prefijos bajo `steamapps/compatdata`, Lutris, Heroic, Bottles y `~/.wine` se muestran separados para facilitar la revisión, pero el script tampoco los elimina automáticamente.
