# Política de seguridad

## Versiones con soporte

| Versión | Estado |
| --- | --- |
| 1.0.x | Recibe correcciones de seguridad mientras sea la versión estable vigente. |

Las versiones preliminares y las builds locales no tienen garantía de soporte.
Cuando se publique una versión estable posterior, esta tabla indicará qué
versiones siguen recibiendo correcciones.

## Reportar una vulnerabilidad

No publiques detalles de una vulnerabilidad sin corregir en una incidencia,
discusión o pull request público. Usa el canal privado «Report a
vulnerability» de GitHub en
[la página de seguridad del repositorio](https://github.com/Darkeiser003/Tools/security/advisories/new).
Incluye la versión afectada, los pasos mínimos para reproducirla y su posible
impacto; no adjuntes datos personales ni secretos.

Si el canal privado no está habilitado, contacta al mantenedor desde su
[perfil de GitHub](https://github.com/Darkeiser003) para coordinar un reporte
privado. No se garantiza un plazo de respuesta, pero se revisarán los reportes
que permitan reproducir el problema.

Las releases incluyen checksums y firmas separadas: la firma Ed25519 se usa
para la verificación de actualizaciones de LTerminal; la firma OpenSSH ofrece
una verificación adicional del manifiesto. Comprueba ambas junto a la clave
pública obtenida por un canal de confianza.
