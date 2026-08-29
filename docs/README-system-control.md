# system-control

Módulo de consulta y administración prudente del sistema para CachyOS y otras distribuciones con systemd.

## Consultas

```bash
./ltools.sh system status
./ltools.sh system services
./ltools.sh system --user services
./ltools.sh system processes
./ltools.sh system journal --unit NetworkManager.service --lines 100
```

Incluye estado de systemd, unidades fallidas, servicios activos/habilitados, servicios de usuario, procesos por CPU/memoria y avisos del journal.

## Acciones protegidas

```bash
./ltools.sh system service status NetworkManager.service
./ltools.sh system service restart NetworkManager.service
./ltools.sh system service enable mi-servicio.service
./ltools.sh system process status 1234
./ltools.sh system process stop 1234
```

Antes de una acción se puede preparar un plan sin ejecutar el cambio:

```bash
./ltools.sh system --dry-run service restart NetworkManager.service \
  --plan /tmp/plan-servicio.tsv
./ltools.sh rollback --plan /tmp/plan-servicio.tsv
```

Los cambios de servicio y las señales de procesos se registran como no
reversibles automáticamente: el programa no puede adivinar de forma segura
el estado previo.

`start`, `stop`, `restart`, `enable`, `disable`, `mask`, `unmask` y las señales a procesos siempre muestran el objetivo y piden confirmación. Las unidades del sistema usan `sudo` únicamente cuando la operación lo necesita; las unidades de usuario se consultan con `systemctl --user`.

La herramienta no mata procesos ni cambia servicios durante una auditoría. Para acciones críticas conviene usar primero `status` y revisar el journal.
