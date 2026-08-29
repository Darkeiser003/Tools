# LTools

`ltools.sh` es el punto de entrada único de LTools para mantenimiento de CachyOS:

- auditoría general de discos, paquetes y aplicaciones;
- auditoría de juegos, Wine, Proton, Lutris, Heroic y Steam;
- limpieza protegida de paquetes, cachés y rutas;
- creación, clonado y migración verificable de prefijos Wine;
- listado de prefijos Wine y Proton existentes.
- comprobación de rutas efectivas y defaults de Wine, winetricks, Proton y Steam.
- inventario de gestores, paquetes instalados por ámbito y archivos de paquetes descargados.
- consulta y control protegido de servicios, daemons, procesos y journal mediante systemd.

La marca del producto es **LTools**. CachyOS, Arch y otras distribuciones se
refieren únicamente al sistema compatible, no al nombre de la aplicación.

## Uso rápido

Sin argumentos se abre el menú:

```bash
./ltools.sh
```

También se puede usar por comandos:

```bash
./ltools.sh audit --full --duplicates --min-size-mb 100
./ltools.sh games --full --root /mnt/JuegosLinux
./ltools.sh packages
./ltools.sh doctor
./ltools.sh --lang en menu
./ltools.sh clean --menu --report "$HOME/Informes/disk-audit-..."
./ltools.sh prefix migrate \
  --source "$HOME/.wine" \
  --dest /mnt/JuegosLinux/prefixes/wine-main \
  --select --rewrite-configs --set-defaults --remove-source
```

Cada módulo conserva su ayuda completa:

```bash
./ltools.sh prefix list
./ltools.sh prefix inspect
./ltools.sh defaults
./ltools.sh system status
./ltools.sh system services
./ltools.sh system --user services
./ltools.sh clean --dry-run --path "$HOME/.cache/paru" --plan /tmp/limpieza.tsv
./ltools.sh rollback --plan /tmp/limpieza.tsv
./ltools.sh --rust games --full --dry-run
./ltools.sh packages --help
./ltools.sh audit --help
./ltools.sh games --help
./ltools.sh clean --help
./ltools.sh prefix --help
```

El lanzador y los módulos deben permanecer en el mismo directorio. Los módulos siguen disponibles para compatibilidad y recuperación; el uso normal recomendado es siempre `ltools.sh`.

La limpieza y la migración mantienen sus confirmaciones y bloqueos de seguridad. La migración no retira el origen hasta verificar la copia y recibir las frases de confirmación correspondientes.

La migración por lotes es uno-a-uno: selecciona varios orígenes y el programa crea un destino independiente para cada uno dentro de la carpeta central elegida. El contenido de cada origen se copia directamente en su destino; no se fusionan varios prefijos en uno porque sus registros y `drive_c` entrarían en conflicto.

`--set-defaults` genera el entorno persistente para Wine/winetricks. Con
`--activate-shell` también ofrece activarlo en `.bashrc`, `.zshrc` y
`environment.d`, siempre con backup. `--update-launchers` actualiza los
defaults globales compatibles de Heroic y revisa referencias de Lutris, UMU y
Steam con copia de seguridad. Steam/Proton usa un `compatdata` por AppID; no
se inventa un default global que pueda romper juegos.

Durante la migración por lotes, las confirmaciones por prefijo son preguntas sencillas `[y/N]`. Se conserva la verificación de la copia, la comprobación de bloqueos y la retirada reversible a la papelera.

El modo por lotes no establece un `WINEPREFIX` global, ya que solo puede haber un default activo. Para fijar uno concreto usa después `--set-defaults` en una migración individual.

La arquitectura modular y la hoja de ruta de la implementación Rust están en [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
La interfaz prevista con LTerminal está descrita en
[`docs/LTERMINAL-INTEGRATION.md`](docs/LTERMINAL-INTEGRATION.md).

El backend Rust ya cubre auditoría, juegos/configuraciones, paquetes,
limpieza, prefijos, defaults, systemd/procesos y rollback. Se selecciona con
`--rust` o mediante `rust-tools.sh`; conserva los TSV y planes compatibles.
Compílalo con `cargo build --release --manifest-path rust/Cargo.toml`. La
paridad exacta de todos los informes Bash y la salida JSON versionada siguen
siendo ampliaciones pendientes.

## Build distribuible

Para validar y empaquetar todo el proyecto en un único archivo:

```bash
./build.sh
./build.sh --non-interactive --fast --no-tests
./build.sh --clean --output /tmp/ltools-dist
./build.sh --appimage --no-package
./build.sh --non-interactive --no-smoke --no-e2e
./build.sh --appimage --require-fuse
```

El build comprueba Rust, Clippy, rustfmt y la sintaxis Bash, compila el backend
release y genera `dist/ltools-VERSION-linux-ARQUITECTURA.tar.gz` y
`dist/ltools-VERSION-linux-ARQUITECTURA.AppImage` (o el directorio
indicado con `--output`). El AppImage incluye un preflight que detecta las
herramientas del sistema y ofrece instalarlas según el gestor disponible. El
build ejecuta además smoke tests y una prueba E2E aislada de migración y
rollback. También comprueba las dependencias de empaquetado y la capacidad
FUSE del equipo. El build no instala paquetes ni modifica el sistema.

`--require-fuse` convierte la comprobación en obligatoria y detiene la build
si falta `/dev/fuse` o `fusermount3`/`fusermount`. Sin esa opción se genera el
AppImage igualmente y se valida el modo de extracción, que es compatible con
equipos sin FUSE.

Desde el AppImage puedes revisar las dependencias del equipo con:

```bash
./dist/ltools-*.AppImage --doctor
./dist/ltools-*.AppImage --fuse-check
```

Si el sistema no tiene FUSE disponible, un AppImage directo puede fallar antes
de iniciar. La build genera `dist/run-ltools.sh`, que detecta FUSE y
activa automáticamente `APPIMAGE_EXTRACT_AND_RUN=1` como fallback seguro:

```bash
./dist/run-ltools.sh --doctor
./dist/run-ltools.sh games --full
```

`--doctor --install-missing` ofrece instalar las dependencias detectadas,
incluido FUSE. En CachyOS/Arch propone `fuse2`; en Debian/Ubuntu y Fedora
propone `fuse3`. También puedes ejecutar `sudo modprobe fuse` si el paquete ya
está instalado.

Cada build deja el registro de la apertura directa en
`dist/appimage-smoke.log`. Si la ejecución falla, el build se detiene y muestra
el motivo real del runtime AppImage, en lugar de entregar silenciosamente un
archivo que no abre.

Las pruebas se pueden ejecutar sin empaquetar:

```bash
./tests/smoke.sh --binary rust/target/release/ltools
./tests/e2e.sh --binary rust/target/release/ltools
```

El backend Rust es el predeterminado del AppImage. `--bash` permite usar la
fachada Bash de compatibilidad.
