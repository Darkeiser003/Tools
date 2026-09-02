//! Gestión de servicios, procesos y registros del sistema operativo.
//!
//! La selección de plataforma ocurre en compilación: el módulo Linux solo
//! se compila con systemd y el módulo Windows solo con sus APIs nativas.

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
