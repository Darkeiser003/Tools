//! Acciones nativas generales separadas por sistema operativo.
//!
//! La implementación concreta vive en cada adaptador para que nunca se
//! intente usar comandos Linux en Windows ni PowerShell en Linux.

use crate::common::Context;

#[cfg(not(windows))]
mod linux;
#[cfg(windows)]
mod windows;

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

pub fn help() -> &'static str {
    #[cfg(windows)]
    {
        "native: network status|flush-dns; hardware status; power status|plans; security status"
    }
    #[cfg(not(windows))]
    {
        "native: network status|flush-dns; hardware status; power status; security status"
    }
}
