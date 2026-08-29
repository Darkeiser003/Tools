# Integración futura con LTerminal

LTools está pensado para funcionar como complemento de LTerminal sin depender
de su implementación interna. La integración debe tratarlo como una
herramienta CLI autónoma que puede vivir en el panel de scripts, en una acción
rápida o en una pestaña dedicada.

## Contrato estable

```text
ltools [--lang IDIOMA] [--dry-run] [--plan FICHERO] COMANDO [OPCIONES]
```

- `--help` y `--version` no modifican el sistema.
- `doctor` comprueba dependencias, FUSE y herramientas disponibles.
- `audit`, `games`, `packages` y `defaults` son consultas o inventarios.
- `clean`, `prefix migrate` y `system service` requieren confirmación y
  pueden registrar un plan reversible.
- `rollback --plan FICHERO` recupera las operaciones reversibles registradas.
- Las salidas de informes TSV son aptas para mostrar en una pestaña o para
  consumirlas desde una futura API de LTerminal.

## Descubrimiento desde LTerminal

El complemento puede localizar, en este orden:

1. `ltools` en `PATH`.
2. Un AppImage `ltools-*.AppImage` junto al lanzador instalado.
3. La ruta indicada por `LTOOLS_APPIMAGE`.
4. El binario Rust `rust/target/release/ltools` en un checkout de desarrollo.

Antes de ofrecer acciones mutables, LTerminal debería ejecutar `ltools doctor`
y mostrar el estado de cada dependencia. Una acción que requiera permisos debe
indicar el comando y dejar la confirmación al usuario.

## Idiomas y salida de máquina

El idioma se controla con `--lang` o `LTOOLS_LANG`; si no se indica, se usa el
locale del sistema y se cae a español. La UI de LTools no debe interpretar
frases traducidas para decidir estados: debe usar códigos de salida, planes TSV
y, en una siguiente iteración, `--format json` con un esquema versionado.

## Evolución prevista

La primera integración puede ser un panel que ejecute comandos y muestre su
salida. Más adelante se puede añadir un protocolo local de eventos para
progreso, advertencias y solicitudes de confirmación, manteniendo el CLI como
fallback universal y evitando que LTools necesite una instancia de LTerminal
para funcionar.
