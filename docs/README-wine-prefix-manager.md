# wine-prefix-manager

Herramienta protegida para crear, clonar y migrar prefijos Wine. Complementa a `game-wine-audit.sh` y no sustituye a Steam, Lutris, Heroic o Bottles para sus propias bases de datos.

## Crear un prefijo vacío

```bash
./wine-prefix-manager.sh create \
  --dest "$HOME/Games/prefixes/test" \
  --arch win64
```

## Clonar/migrar un prefijo completo

```bash
./wine-prefix-manager.sh migrate \
  --source "$HOME/.wine" \
  --dest /mnt/JuegosLinux/prefixes/wine-main \
  --rewrite-configs
```

La operación copia el prefijo entero, incluidos `system.reg`, `user.reg`, `drive_c` y enlaces. Comprueba el espacio disponible, detecta posibles bloqueos, anuncia las acciones posteriores antes de empezar, verifica la copia con `rsync` y conserva el origen. Tras la verificación no repite una confirmación genérica; solo solicita las confirmaciones específicas de cada acción posterior.

## Elegir qué copiar

Para seleccionar elementos de primer nivel de forma interactiva:

```bash
./wine-prefix-manager.sh migrate \
  --source "$HOME/.wine" \
  --dest /mnt/JuegosLinux/prefixes/wine-main \
  --select
```

También se puede automatizar la selección:

```bash
./wine-prefix-manager.sh migrate \
  --source "$HOME/.wine" \
  --dest /mnt/JuegosLinux/prefixes/wine-main \
  --include drive_c \
  --include system.reg \
  --include user.reg \
  --exclude dosdevices
```

`--include` y `--exclude` solo aceptan nombres directamente dentro del prefijo; no borran ni mueven los elementos omitidos. Si la selección no incluye `drive_c` y `system.reg`, el programa exige `CONTINUAR-PARCIAL` y avisa de que el resultado puede no arrancar como prefijo Wine.

Para retirar el origen después de verificarlo:

```bash
./wine-prefix-manager.sh migrate \
  --source "$HOME/Games/ea-app" \
  --dest /mnt/JuegosLinux/Lutrs/ea-app \
  --rewrite-configs \
  --remove-source
```

El origen se mueve a la papelera y requiere la frase `RETIRAR-ORIGEN`; no se elimina permanentemente.
Si la migración fue selectiva, antes exige además `RETIRAR-OMITIDOS`, porque los elementos no seleccionados se perderían al retirar el prefijo antiguo.

## Migración por lotes desde LTools

`ltools.sh prefix batch` usa `--batch-mode` internamente. En este modo las confirmaciones repetitivas se muestran como preguntas normales `[y/N]` para cada prefijo: cierre de procesos, inicio de copia, actualización de referencias y retirada del origen. Las comprobaciones de seguridad, la verificación con `rsync` y la papelera se mantienen; solo cambia la forma de confirmar.

## Hacer que Wine y winetricks usen la nueva ruta

Después de validar la copia, se puede generar una configuración de usuario:

```bash
./wine-prefix-manager.sh migrate \
  --source "$HOME/.wine" \
  --dest /mnt/JuegosLinux/prefixes/wine-main \
  --set-defaults
```

Esto crea `~/.config/wine-prefix-manager/default-prefix.sh`, con `WINEPREFIX`, `wine-prefix()` y `winetricks-prefix()`. No fuerza `WINEARCH`, para conservar la arquitectura del prefijo migrado. Se activa en la terminal actual con:

```bash
source "$HOME/.config/wine-prefix-manager/default-prefix.sh"
```

Si se quiere añadirlo permanentemente a `.bashrc` y `.zshrc`, y a las aplicaciones iniciadas en la sesión de usuario, se puede usar `--activate-shell`. Requiere escribir `ACTIVAR-DEFAULT`, crea copias de seguridad y genera también `~/.config/environment.d/90-ltools-wine.conf`. Hay que abrir una nueva terminal o reiniciar la sesión para que el proceso padre recoja el nuevo entorno.

Para actualizar el default global compatible de Heroic, añade `--update-launchers`. Modifica `config.json` y `store/config.json` solo si existen, guarda un backup y actualiza la carpeta de prefijos y el prefijo compartido. Las configuraciones individuales de cada juego se revisan con `--rewrite-configs`; no se fuerza un único prefijo para todos los juegos.

Proton se trata aparte: Steam necesita un `compatdata` independiente por AppID, por lo que el script nunca exporta `STEAM_COMPAT_DATA_PATH` globalmente para un prefijo Wine normal. Si el origen es realmente un prefijo Proton de Steam y se usa `--allow-steam --set-defaults`, genera `proton-prefix()` apuntando al nuevo directorio `compatdata`.

## Referencias de configuración

`--rewrite-configs` busca referencias literales al origen en configuraciones de Lutris, Heroic, UMU y lanzadores `.desktop`. Antes de modificar cada archivo crea una copia dentro de `migration-config-backup-YYYYMMDD-HHMMSS` en el destino.

Después de cada migración, aunque no se use `--rewrite-configs`, el gestor muestra la aplicación probable, el archivo afectado, la ruta antigua y la nueva. Así puedes reconfigurar manualmente cada juego o reiniciar Lutris, Heroic y UMU después de la actualización.

Si existen referencias y no se usa `--rewrite-configs`, el prefijo se copia igualmente, pero las aplicaciones seguirán apuntando a la ruta antigua hasta que se actualicen desde sus propias interfaces o mediante esta opción. Las bases de datos SQLite, Steam y configuraciones binarias requieren revisión manual.

## Steam/Proton

Las plantillas `default_pfx` compartidas están bloqueadas. Los prefijos `steamapps/compatdata/<appid>/pfx` también están bloqueados por defecto porque Steam administra su relación con el AppID. Para una operación deliberada:

```bash
./wine-prefix-manager.sh clone \
  --source /ruta/a/steamapps/compatdata/123456/pfx \
  --dest /otra/ruta/pfx \
  --allow-steam
```

Para mover juegos y sus prefijos Steam, es preferible usar el administrador de almacenamiento de Steam.

## Prefijo accidental en la raíz de un disco

Si se ejecutó Wine con `WINEPREFIX=/mnt/JuegosLinux`, la raíz del disco puede contener `drive_c` y los archivos `.reg` junto a los juegos. El gestor bloquea esa ruta como origen completo para no copiar el disco entero.

Para rescatar únicamente los componentes conocidos del prefijo:

```bash
./wine-prefix-manager.sh migrate \
  --source /mnt/JuegosLinux \
  --dest /mnt/JuegosLinux/prefixes/wine-root-migrated \
  --allow-mount-root \
  --include drive_c \
  --include system.reg \
  --include user.reg \
  --include dosdevices
```

Exige `RAIZ-MONTAJE`, no permite retirar la raíz ni reescribir referencias globales del disco, y no toca los juegos ni las demás carpetas. El destino debe ser nuevo o estar vacío; no uses un prefijo ya inicializado como destino.

## Limitaciones deliberadas

- Los juegos instalados fuera del prefijo no se copian automáticamente.
- La selección trabaja con elementos de primer nivel; no intenta interpretar instaladores o archivos internos del registro.
- No se reescriben bases de datos SQLite ni configuraciones binarias.
- No se borra el origen automáticamente.
- Se recomienda usar ext4, btrfs u otro sistema de archivos Linux para prefijos; NTFS, exFAT y FAT pueden causar problemas con enlaces, permisos y nombres.
