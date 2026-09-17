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
        "native: network status|interfaces|routes|dns|listening|connections|flush-dns; hardware status; power status|plans; security status|scanners; tools menu|status|install|ssh|adb|containers|kubernetes; utilities status/install (curl,wget,file,tree,htop,btop,lsof,strace,tcpdump,dig,nmap,openssl,gpg,7z,unzip,zip,zstd,tmux,python3,make,cmake,gcc,gdb); container inspect|stats|top|port|diff|pause|unpause|kill|rename|cp|prune; image inspect|history|build|tag|remove|prune; volume and network list|inspect|create|remove|prune; system info|df|prune; compose up|down|start|stop|restart|ps|logs|pull|build|config|images|top|run|exec|rm|pause|unpause"
    }
    #[cfg(not(windows))]
    {
        "native: network menu|status|interfaces|routes|dns|listening|connections|set-interface|connection-up|connection-down|flush-dns; hardware status; power menu|status; security status|scanners; tools menu|status|install|ssh|adb|containers|kubernetes; utilities status/install (curl,wget,file,tree,htop,btop,lsof,strace,tcpdump,dig,nmap,openssl,gpg,7z,unzip,zip,zstd,tmux,python3,make,cmake,gcc,gdb); container inspect|stats|top|port|diff|pause|unpause|kill|rename|cp|prune; image inspect|history|build|tag|remove|prune; volume and network list|inspect|create|remove|prune; compose up|down|start|stop|restart|ps|logs|pull|build|config|images|top|run|exec|rm|pause|unpause"
    }
}

/// Inventario de analizadores de código y CI disponibles en el anfitrión.
/// Solo consulta el catálogo; no instala ni ejecuta un escáner y por eso no
/// debe cambiar la identidad del proceso ni solicitar elevación.
pub(super) fn security_scanner_inventory() {
    const SCANNER_IDS: &[&str] = &[
        "shellcheck",
        "actionlint",
        "zizmor",
        "gitleaks",
        "osv-scanner",
        "codeql",
        "scorecard",
        "cargo-audit",
        "cargo-deny",
    ];
    println!(
        "=== {} ===",
        crate::i18n::system_page_text("native_security_scanners")
    );
    let mut found = false;
    for tool in crate::platform::host_tools().iter().filter(|tool| {
        SCANNER_IDS.contains(&tool.id)
            || tool
                .id
                .strip_suffix(".exe")
                .is_some_and(|id| SCANNER_IDS.contains(&id))
    }) {
        found = true;
        let status = if crate::platform::host_tool_available(tool) {
            crate::i18n::diagnostics_available()
        } else {
            crate::i18n::diagnostics_unavailable()
        };
        println!("{:<20} {:<16} {}", tool.command, status, tool.feature);
    }
    if !found {
        println!("No hay analizadores de código catalogados para esta plataforma.");
    }
    println!("{}", crate::i18n::help_extra("scanner_note"));
}
