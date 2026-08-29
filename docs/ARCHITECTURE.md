# Arquitectura prevista de LTools

## Objetivo

El proyecto se está construyendo como una herramienta modular de mantenimiento local para CachyOS/Arch y, cuando sea posible, otras distribuciones. Cada módulo debe poder ejecutarse desde `ltools.sh` o de forma independiente, producir informes reproducibles y separar claramente lectura, planificación y modificación.

## Capas

```text
CLI / menú
    └── parser y presentación
        └── operaciones (auditoría, limpieza, prefijos, sistema)
            └── adaptadores (pacman, Flatpak, Wine, systemd, Steam, Lutris...)
                └── sistema de archivos y comandos externos
```

### Contrato de cada módulo Bash

- `--help` y `--version`.
- modo de consulta sin cambios por defecto.
- confirmación explícita para cada modificación.
- copias de seguridad o papelera cuando sea posible.
- códigos de salida: `0` completado, `1` cancelado/fallo operativo, `2` argumentos o precondiciones inválidas.
- informes TSV legibles por herramientas externas y resumen humano.
- no asumir nombres fijos de discos, usuario, launcher ni punto de montaje.

## Planes y rollback

Los módulos que modifican el sistema aceptan `--dry-run` y `--plan FICHERO`.
El plan TSV común registra operación, objetivo, estado, reversibilidad y datos
de recuperación. `rollback.sh --plan FICHERO` solo intenta restaurar copias o
recuperar movimientos a papelera; desinstalaciones, señales a procesos y
cambios de servicios se marcan como no reversibles porque no se puede inferir
su estado anterior con seguridad.

## Estado actual de módulos

- `disk-audit.sh`: discos, paquetes instalados, gestores, artefactos y tamaños.
- `disk-clean.sh`: paquetes huérfanos, cachés y artefactos seleccionados con comprobaciones.
- `game-wine-audit.sh`: Steam, Lutris, Heroic, UMU, Bottles, runners y contenido de prefijos.
- `wine-prefix-manager.sh`: creación, copia verificable, migración, referencias y defaults.
- `system-control.sh`: estado de systemd, servicios, procesos y journal.
- `ltools.sh`: fachada única, menú y aliases compatibles.
- `rollback.sh` y `lib/ltools-plan.sh`: plan universal y recuperación segura.

## Migración a Rust

La migración se hará por componentes, no como una reescritura ciega:

1. `core`: rutas seguras, permisos, montajes, tamaños, papelera, copias y resultados estructurados.
2. `inventory`: paquetes, juegos, prefijos, runners y configuraciones.
3. `plan`: operaciones previstas, dependencias, advertencias, espacio requerido y reversibilidad.
4. `executor`: ejecución confirmada con registro, rollback cuando exista y límites de seguridad.
5. `adapters`: `pacman`, `paru/yay`, `apt/dpkg`, `rpm/dnf`, Flatpak, Snap, Homebrew, systemd, Wine/Proton, Steam, Lutris y Heroic.
6. `ui`: CLI estable, salida humana, JSON/TSV y posteriormente TUI.

El formato de informes TSV actual sirve como compatibilidad temporal. El
backend Rust ya conserva ese formato; la salida JSON versionada queda como
siguiente ampliación.

## Reglas de seguridad

- Nunca tratar una raíz de montaje como un prefijo normal.
- Nunca migrar plantillas `default_pfx` compartidas.
- No borrar directamente: papelera, copia o acción nativa del gestor.
- No modificar configuraciones de un launcher sin backup y sin listar los archivos afectados.
- Steam/Proton se modela por biblioteca y AppID; no se inventa un “default global”.
- Servicios y procesos requieren mostrar el objetivo exacto antes de actuar.
- Los módulos deben tolerar herramientas opcionales ausentes y explicar qué capacidad se omite.

## Backend Rust actual

`rust/` contiene un binario sin dependencias externas que conserva el contrato
TSV y detecta prefijos sin seguir enlaces simbólicos ni cruzar sistemas de
archivos. Ya ejecuta auditorías, inventario multi-gestor, limpieza protegida,
migración verificable de prefijos, actualización respaldada de configuraciones,
defaults de Wine, systemd/procesos y rollback de operaciones reversibles.
El lanzador Bash conserva la interfaz histórica y permite seleccionar Rust con
`--rust`.

## Próximas ampliaciones

- modelos estructurados y salida JSON versionada;
- paridad fina de informes especializados de juegos y aplicaciones;
- inventario ampliado de timers, sockets, mounts, scopes y servicios fallidos;
- detección y reparación guiada de enlaces simbólicos y rutas rotas;
- TUI opcional manteniendo el CLI estable.
