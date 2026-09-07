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
        "native: network status|flush-dns; hardware status; power status|plans; security status; tools menu|status|install|ssh|adb|containers|kubernetes; utilities status/install (curl,wget,file,tree,htop,btop,lsof,strace,tcpdump,dig,nmap,openssl,gpg,7z,unzip,zip,zstd,tmux,python3,make,cmake,gcc,gdb); container inspect|stats|top|port|diff|pause|unpause|kill|rename|cp|prune; image inspect|history|build|tag|remove|prune; volume and network list|inspect|create|remove|prune; system info|df|prune; compose up|down|start|stop|restart|ps|logs|pull|build|config|images|top|run|exec|rm|pause|unpause"
    }
    #[cfg(not(windows))]
    {
        "native: network menu|status|flush-dns; hardware status; power menu|status; security status; tools menu|status|install|ssh|adb|containers|kubernetes; utilities status/install (curl,wget,file,tree,htop,btop,lsof,strace,tcpdump,dig,nmap,openssl,gpg,7z,unzip,zip,zstd,tmux,python3,make,cmake,gcc,gdb); container inspect|stats|top|port|diff|pause|unpause|kill|rename|cp|prune; image inspect|history|build|tag|remove|prune; volume and network list|inspect|create|remove|prune; compose up|down|start|stop|restart|ps|logs|pull|build|config|images|top|run|exec|rm|pause|unpause"
    }
}
