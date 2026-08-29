# disk-clean

`disk-clean.sh` es el complemento interactivo de `disk-audit.sh`. Su objetivo es que la auditoría y la limpieza estén separadas:

- paquetes: usa `pacman`, muestra información y dependientes antes de actuar;
- paquetes explícitos: permite revisar también paquetes que no son huérfanos;
- archivos de usuario: los mueve a la papelera mediante `gio` o `trash-put`;
- runtimes Flatpak: delega la decisión al propio Flatpak;
- caché de pacman: usa `paccache`;
- no contiene una limpieza automática global ni ejecuta `rm -rf`.

## Uso recomendado

Primero genera el informe:

```bash
./disk-audit.sh --full --duplicates --min-size-mb 100 \
  --out "$HOME/Informes/disk-audit-$(date +%Y%m%d-%H%M)"
```

Después abre el limpiador con ese informe:

```bash
./disk-clean.sh --menu --report "$HOME/Informes/disk-audit-20260828-2334"
```

El menú permite seleccionar entradas individuales o varias mediante números separados por espacios. `all` selecciona todas las entradas de una categoría, pero cada elemento mantiene su confirmación.

## Acciones directas

```bash
./disk-clean.sh --package electron41
./disk-clean.sh --orphans
./disk-clean.sh --foreign
./disk-clean.sh --explicit
./disk-clean.sh --path "$HOME/.cache/paru/clone"
./disk-clean.sh --pacman-cache
./disk-clean.sh --flatpak-unused
./disk-clean.sh --package-caches
./disk-clean.sh --package-artifacts --report "$HOME/Informes/package-audit-..."
```

Antes de una limpieza se puede generar un plan sin cambiar nada:

```bash
./disk-clean.sh --dry-run --package-caches --plan "$HOME/Informes/plan-limpieza.tsv"
./rollback.sh --plan "$HOME/Informes/plan-limpieza.tsv"
```

El rollback solo recupera operaciones con copia o papelera. Las operaciones
nativas de gestores de paquetes y Flatpak quedan registradas, pero no se
invierten automáticamente porque su estado anterior no se puede deducir de
forma fiable.

Los paquetes AUR/manuales (`pacman -Qm`) no se consideran basura automáticamente. El script los presenta para que el usuario los seleccione.

`--package-caches` detecta los gestores disponibles en el equipo, muestra el ámbito, tamaño y ruta de cada almacén, y ofrece cada limpieza por separado: caché de pacman, compilaciones de paru/yay, APT, DNF/YUM, Zypper, APK, XBPS, `pkg`, Homebrew y runtimes Flatpak sin uso. No ejecuta acciones para gestores que no estén instalados. Snap y Nix se muestran como detectados, pero no se alteran automáticamente porque sus revisiones y generaciones requieren una decisión específica.

La opción `Rutas del informe → Archivos de paquetes descargados` permite revisar y enviar a la papelera `.deb`, `.rpm`, paquetes de Arch y otros formatos detectados. Esto no desinstala el paquete instalado: solo retira el archivo descargado después de confirmarlo.

Los archivos que estén dentro de cachés protegidas del sistema (`/var/cache` o revisiones Snap) no se mueven directamente a la papelera; el limpiador los bloquea y remite a `--package-caches`, que usa la orden nativa correspondiente.

## Dependencias y referencias

Para cada paquete se consulta `Required By` y, si está disponible, se muestra `pactree -r`. Si hay dependientes, el limpiador no continúa salvo que el usuario escriba una confirmación explícita de cascada:

```text
CASCADE nombre-del-paquete
```

Una cascada de pacman puede retirar también paquetes explícitos que dependan del objetivo, por eso requiere esa frase exacta además de una confirmación posterior.

Para archivos, se buscan referencias literales en configuraciones de usuario de Steam/Lutris/Heroic/UMU y lanzadores `.desktop`. Si se encuentran, el movimiento se bloquea. `--force` permite continuar después de revisar esas referencias, pero no desactiva las confirmaciones ni los bloqueos de rutas críticas.

## Protecciones

El script bloquea:

- `/`, `/home`, `/mnt`, `/usr`, `/etc`, `/var`, `/opt` y otros directorios raíz;
- puntos de montaje;
- `default_pfx` compartidos de Proton/Wine;
- directorios generales `steamapps`, `compatdata` y `steamapps/common`;
- enlaces simbólicos;
- cualquier archivo si no existe o no puede resolverse con seguridad.

Para liberar espacio definitivamente después de revisar la papelera, utiliza el gestor de archivos. Mover a la papelera es deliberado: permite recuperar una selección equivocada y evita que una decisión irreversible ocurra durante el primer análisis.
