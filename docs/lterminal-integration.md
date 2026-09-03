# Integración de LTools con LTerminal y WinSlim Terminal

Este documento define el acuerdo entre ambos proyectos. LTools es autónomo y
puede ejecutarse sin LTerminal; la integración solo añade descubrimiento y
botones de acciones rápidas.

## Fuente canónica

La entrada pública del proyecto es:

```text
https://raw.githubusercontent.com/Darkeiser003/Tools/main/distribution/ltools-project.json
```

`distribution/ltools-project.json` es el descriptor mantenido a mano y
`distribution/ltools-project.schema.json` es su esquema. El builder copia ambos
a `release/`. `ltools-release.json` se genera en cada publicación y contiene
los archivos reales, tamaños y SHA-256; no debe editarse manualmente.

La publicación también incluye `SHA256SUMS.txt` y la firma separada
`SHA256SUMS.txt.sig`. Es una firma Ed25519 en Base64 del contenido exacto del
checksum, compatible con el firmador/verificador de LTerminal. LTools busca el
material local en `~/.config/lterminal/release-signing-private.pem` y
`~/.config/lterminal/release-signing-public.hex`, o en las rutas indicadas por
las variables de entorno del builder. La clave privada nunca se distribuye.

La terminal debe seleccionar únicamente la release estable, el artefacto de la
plataforma actual y la arquitectura compatible:

| Plataforma | Aplicación | Uso normal | CLI | Instalación recomendada |
|---|---|---|---|---|
| Linux | `LTools` | `.AppImage` | `-cli.AppImage` | `~/.local/bin` |
| Windows | `WinSlim-Tools` | `.exe` o ZIP portable | `-cli.exe` | `%LOCALAPPDATA%\\WinSlim-Tools` |

Antes de promover una descarga a la instalación activa hay que comprobar la
URL permitida, el tamaño anunciado, el SHA-256 y la firma Ed25519 de
`SHA256SUMS.txt`. Una descarga
parcial nunca debe sustituir al ejecutable actual.

## Flujo de instalación y actualización

El flujo común es `stage → verify → promote`:

1. Consultar `release_manifest` y comprobar que pertenece al repositorio
   esperado y que la etiqueta es estable.
2. Descargar a un archivo o carpeta temporal del mismo volumen.
3. Comprobar tamaño, SHA-256, firma Ed25519, formato del artefacto y permisos
   de ejecución.
4. Promover mediante sustitución segura, conservando la versión anterior para
   rollback. No sustituir un binario que esté en ejecución.
5. Revalidar `--version`, `--help` y `capabilities --format json`.
6. Informar de la ruta activa, la versión instalada y el resultado de cada
   comprobación.

El manifiesto no contiene instrucciones ejecutables ni comandos de shell. Las
URLs de descarga deben limitarse a los hosts de GitHub declarados en el
descriptor. El JSON de integración tampoco instala dependencias ni abre una
terminal por sí mismo.

## Acciones rápidas

`ltools-terminal.json` se usa exclusivamente para convertir `actions` en
botones. La terminal debe preferir siempre:

```text
executable + args[]
```

`command` es solo una representación legible para hosts antiguos. No se debe
dividir ni reinterpretar como una cadena de shell. Cada acción declara si es
segura, si es interactiva, si requiere privilegios y qué comandos anfitriones
necesita.

El catálogo también declara acciones de software y Git. `package-search` abre
la búsqueda contextual y puede pedir un nombre; `package-install` vuelve a
buscar, presenta los candidatos por gestor/versión y exige confirmación.
`git-status` es de solo lectura, mientras que `git-clone`, `git-fetch`,
`git-pull` y `git-login` son interactivos y están marcados como operaciones que
requieren confirmación. La salida estructurada de búsqueda usa
`ltools-package-search-v1`; el descriptor de terminal sigue siendo opcional y
no sustituye al contrato JSON de capacidades.

La integración generada identifica automáticamente:

- Linux: host `LTerminal`, ejecutable `ltools`.
- Windows: host `WinSlim Terminal`, ejecutable `ltools.exe`.

Si el protocolo `lterminal-startup-v1` no está disponible, LTools permanece
autónomo y no cambia silenciosamente a otra terminal. El descriptor es opcional
para AppImage, tarball, ZIP y `.exe`.

## Incorporación al panel Proyectos

La sección de Proyectos de LTerminal trabaja con repositorios GitHub. Para
mostrar LTools allí, la futura modificación de LTerminal debe añadir
`Darkeiser003/Tools` a su catálogo de repositorios. Esta tarea no se realiza en
este proyecto ni modifica LTerminal.

Cuando el panel admita entradas de proyecto declarativas, debe aceptar el
descriptor anterior como fuente de metadatos de release, sin duplicar a mano
las URLs, nombres de archivo o versiones.

## Compatibilidad y límites

- LTools no depende de LTerminal para arrancar.
- LTerminal no debe depender de Wine, Proton, Bash ni PowerShell para descargar
  o verificar una release.
- Linux y Windows reciben capacidades distintas: el binario Windows no anuncia
  prefijos Wine, Lutris, Heroic o UMU; el binario Linux no anuncia DiskPart ni
  `sc.exe`.
- La actualización de LTools no debe confundirse con actualizar LTerminal.
  Son productos, ejecutables, datos y ciclos de release independientes.

## Notas pendientes para LTerminal — no aplicadas

Estas observaciones salen de la auditoría cruzada y quedan deliberadamente
fuera de este cambio:

1. Añadir una entrada `Darkeiser003/Tools` al catálogo de proyectos cuando se
   quiera mostrar LTools oficialmente.
2. Añadir un lector opcional de `ltools-project.json` y
   `ltools-release.json`, con selección por plataforma/arquitectura y
   verificación SHA-256 antes de instalar.
3. Separar visualmente `clonar proyecto` de `descargar release`, porque son
   operaciones distintas.
4. Mostrar estado de instalación (`no instalado`, `versión activa`, `nueva
   release`, `fallo de hash`) y permitir rollback.
5. Mantener la descarga fuera de la shell visible, pero ejecutar las acciones
   interactivas de LTools en una pestaña con `executable` y `args[]` separados.
6. Añadir tests de contrato que comprueben que una entrada de proyecto no puede
   introducir hosts de descarga arbitrarios ni comandos concatenados.

## Validación desde LTools

El builder y las pruebas de LTools validan la presencia de los descriptores, la
estructura JSON, los artefactos, los hashes, los nombres de plataforma y la
independencia del host. La E2E de publicación verifica la carpeta `release/`
antes de subirla a GitHub.
