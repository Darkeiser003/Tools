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
    if uses_storage_map(args) {
        return crate::storage_map::run(ctx, args);
    }
    #[cfg(windows)]
    {
        windows::run(ctx, args)
    }
    #[cfg(not(windows))]
    {
        linux::run(ctx, args)
    }
}

fn uses_storage_map(args: &[String]) -> bool {
    crate::cli_args::positionals(args)
        .first()
        .is_some_and(|arg| {
            matches!(
                *arg,
                "map" | "tree" | "paths" | "explain" | "manage" | "files" | "file-manager"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::uses_storage_map;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn storage_dispatch_ignores_option_values_that_match_subcommands() {
        assert!(!uses_storage_map(&args(&["mount", "--device", "map"])));
        assert!(!uses_storage_map(&args(&[
            "operate", "mkfs", "--label", "manage"
        ])));
        assert!(uses_storage_map(&args(&[
            "--format=json",
            "map",
            "--path",
            "/tmp"
        ])));
        assert!(uses_storage_map(&args(&[
            "manage", "--path", "delete", "copy"
        ])));
    }
}
