# Núcleo Rust de LTools

Este directorio contiene la implementación Rust modular del proyecto. El
lanzador Bash sigue disponible como compatibilidad y `ltools.sh --rust`
permite seleccionar Rust por comando.

Compilación y prueba:

```text
cargo build --release
cargo run -- audit --root /ruta/de/juegos --out /tmp/informe-rust --dry-run
cargo run -- audit --root /ruta/de/juegos --out /tmp/informe-rust \
  --dry-run --plan /tmp/plan-rust.tsv
cargo run -- games --full --out /tmp/informe-juegos
cargo run -- packages --out /tmp/informe-paquetes
cargo run -- clean --dry-run --path "$HOME/.cache/paru" --plan /tmp/plan.tsv
cargo run -- clean --dry-run --orphans --package-caches --plan /tmp/clean-plan.tsv
cargo run -- rollback --plan /tmp/plan.tsv
cargo run -- prefix list --full
cargo run -- prefix migrate --source "$HOME/.wine" \
  --dest /mnt/JuegosLinux/prefixes/wine-main \
  --set-defaults --activate-shell --update-launchers --rewrite-configs
cargo run -- system status
```

El listado de gestores cubre pacman/AUR, dpkg/apt, rpm/dnf, zypper, apk,
xbps, pkg, snap, Flatpak, Homebrew, Nix, Guix, eopkg y emerge cuando están
instalados. Los artefactos `.deb`, `.rpm`, `.pkg`, `.apk`, `.txz` y formatos
Arch se separan por ámbito de sistema o usuario.

Las acciones mutantes conservan confirmación, papelera/backup y plan. La
migración verifica cada elemento con `rsync`, bloquea puntos de montaje,
plantillas `default_pfx` y compatdata de Steam salvo autorización explícita,
y puede actualizar Heroic y referencias de configuración con copia de
seguridad. Steam/Proton sigue siendo por AppID: no se inventa un default global.

El rollback restaura backups y puede retirar destinos creados cuando el
sistema de papelera lo permite; no puede reconstruir el estado anterior de
pacman, Flatpak, servicios o procesos, que se registran como no reversibles.
