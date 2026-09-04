//! Gestión nativa de cuentas, grupos y sesiones.
//!
//! Cada plataforma mantiene su propio backend para no cruzar comandos ni
//! supuestos entre Linux y Windows. Las operaciones mutantes exigen un
//! objetivo explícito y confirmación.

#[cfg(not(windows))]
mod linux;
#[cfg(windows)]
mod windows;

use crate::common::Context;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    #[cfg(windows)]
    {
        windows::run(ctx, args)
    }
    #[cfg(not(windows))]
    {
        linux::run(ctx, args)
    }
}
