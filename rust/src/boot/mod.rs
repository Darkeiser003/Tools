//! Inspección segura del arranque y generación de planes por plataforma.
//!
//! El estado se consulta con herramientas nativas. Las operaciones que
//! escriben GRUB, EFI o BCD no se ejecutan desde el menú ni desde un alias:
//! primero se debe revisar un plan y usar una orden explícita de aplicación
//! que cada backend pueda validar y respaldar.

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
        "boot: status | plan; inspección nativa de BCD/UEFI sin modificar el arranque"
    }
    #[cfg(not(windows))]
    {
        "boot: status | plan; inspección nativa de GRUB/systemd-boot/EFI sin modificar el arranque"
    }
}
