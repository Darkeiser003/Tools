# Hoja de ruta de LTools

## Ya implementado

- [x] Punto de entrada único `ltools.sh` con menú, aliases y ayuda.
- [x] Auditoría de discos, directorios, archivos grandes, basura, AppImages, máquinas virtuales y duplicados.
- [x] Inventario de pacman, AUR helpers, apt/dpkg, rpm/dnf, apk, xbps, pkg, Snap, Flatpak, Homebrew, Nix y archivos de paquetes.
- [x] Limpieza interactiva con cachés nativas, papelera, comprobación de dependencias y revisión de paquetes explícitos.
- [x] Detección de Wine, Proton, Steam, Lutris, Heroic, UMU, Bottles, runners y prefijos anidados.
- [x] Clasificación de raíces de montaje con restos Wine para impedir migraciones accidentales del disco completo.
- [x] Informe de contenido de prefijos: programas registrados, carpetas, EXE/MSI, bloqueos, arquitectura, Windows y runner.
- [x] Migración individual y por lotes con destinos independientes, copia verificable y retirada reversible.
- [x] Actualización segura de referencias de Lutris/Heroic/UMU con backup.
- [x] Default persistente de Wine/winetricks y activación para shell y sesión de usuario.
- [x] Actualización optativa de defaults globales compatibles de Heroic.
- [x] Módulo de systemd: estado, servicios, daemons, procesos y journal; las acciones mutantes piden confirmación.

## Próxima iteración Bash

- [x] Un plan de operación común (`--plan`, `--dry-run`, resumen de cambios y rollback) para limpieza, migración, configuración y servicios.
- [ ] Detectar y reparar enlaces simbólicos rotos, rutas inexistentes y configuraciones que apunten a la papelera.
- [ ] Añadir timers, sockets, mounts, scopes y servicios fallidos a la vista de systemd.
- [ ] Añadir adaptadores opcionales para OpenRC/SysV sin asumir que existen.
- [x] Inventariar configuraciones binarias/SQLite, cabeceras SQLite y formatos VDF/ACF, mostrando cuándo solo es posible la reconfiguración manual.
- [x] Validar rutas reales de Heroic, Lutris, UMU y Steam, incluyendo JSON, YAML, `config_info` y bibliotecas Steam.
- [ ] Detectar AppImages instaladas mediante desktop files y separar copias de desarrollo, papelera y ejecutables activos.
- [ ] Añadir pruebas automatizadas con árboles sintéticos de prefijos, caches, configuraciones y montajes simulados.
- [ ] Reducir el coste de las auditorías profundas mediante caché de tamaños e índices reutilizables.

## Migración a Rust

- [x] Mantener los TSV actuales como formato de compatibilidad.
- [x] Crear el backend Rust modular compilable para auditoría, paquetes, limpieza, prefijos, configuraciones y systemd.
- [x] Añadir un CLI Rust ejecutable con menú, defaults y aliases desde el lanzador único.
- [x] Añadir plan común Rust, `--dry-run`, copias verificadas y rollback de operaciones reversibles.
- [x] Añadir adaptadores Rust para gestores de paquetes y cachés comunes, Wine/Proton, Heroic, Lutris, UMU, Steam y systemd.
- [ ] Introducir modelos estructurados versionados y salida JSON.
- [x] Portar rutas seguras, detección, planificador y validaciones básicas.
- [ ] Endurecer y ampliar los adaptadores Rust hasta paridad completa con todos los informes Bash.
- [ ] Añadir ejecución transaccional completa y logs estructurados.
- [ ] Sustituir progresivamente la UI Bash por CLI/TUI, conservando los comandos existentes durante la transición.
