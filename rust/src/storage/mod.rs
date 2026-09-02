//! Inventario de almacenamiento y particiones.
//!
//! Solo este módulo conoce las herramientas de discos. Cada plataforma tiene
//! su propio archivo para evitar compilar comandos Linux en Windows o
//! comandos PowerShell en Linux.

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
