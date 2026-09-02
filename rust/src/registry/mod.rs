//! Registros y configuración nativa de cada sistema.

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
