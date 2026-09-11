//! Adaptadores aislados por sistema operativo.
//!
//! El resto del backend solo usa esta interfaz. Así Linux puede conocer
//! systemd, montajes y la papelera XDG sin contaminar el binario Windows, y
//! Windows puede usar UAC, `sc.exe`, PowerShell y su papelera nativa.

use std::io;
use std::path::{Path, PathBuf};

/// Herramienta del anfitrión que LTools puede aprovechar si está instalada.
/// `required` solo identifica dependencias básicas de un módulo; nunca hace
/// que el builder o el AppImage instalen nada automáticamente.
#[derive(Debug, Clone, Copy)]
pub struct HostTool {
    pub id: &'static str,
    pub command: &'static str,
    pub category: &'static str,
    pub feature: &'static str,
    pub required: bool,
    pub installable: bool,
    pub install_package: &'static str,
}

/// Normaliza la primera línea de versión antes de publicarla en JSON. Algunas
/// utilidades detectan que stdout no es un TTY de forma incompleta y emiten
/// secuencias ANSI; esos bytes son válidos en una terminal, pero invalidan el
/// contrato JSON y ensucian la salida de la GUI.
pub(super) fn sanitize_version(value: &str) -> String {
    let mut clean = String::with_capacity(value.len());
    let mut escape = false;
    for character in value.chars() {
        if escape {
            if character == '\u{7}' || character.is_ascii_alphabetic() {
                escape = false;
            }
            continue;
        }
        if character == '\u{1b}' {
            escape = true;
            continue;
        }
        if character.is_control() {
            continue;
        }
        clean.push(character);
    }
    clean.trim().chars().take(240).collect()
}

#[cfg(unix)]
mod linux;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
use linux as current;
#[cfg(windows)]
use windows as current;

pub fn home_dir() -> PathBuf {
    current::home_dir()
}

pub fn timestamp() -> String {
    current::timestamp()
}

pub fn command_exists(name: &str) -> bool {
    current::command_exists(name)
}

pub fn host_tool_available(tool: &HostTool) -> bool {
    current::host_tool_available(tool)
}

/// Obtiene una versión corta solo cuando la herramienta está disponible. Los
/// adaptadores por plataforma conocen las excepciones (por ejemplo,
/// `docker-compose version` o los cmdlets de PowerShell).
pub fn host_tool_version(tool: &HostTool) -> Option<String> {
    current::host_tool_version(tool)
}

pub fn run_with_privilege(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    current::run_with_privilege(program, args, dry_run)
}

#[cfg(not(windows))]
pub fn run_with_privilege_input(
    program: &str,
    args: &[String],
    input: &[u8],
    dry_run: bool,
) -> io::Result<bool> {
    current::run_with_privilege_input(program, args, input, dry_run)
}

pub fn critical_path(path: &Path) -> bool {
    current::critical_path(path)
}

pub fn move_to_trash(path: &Path, dry_run: bool) -> io::Result<bool> {
    current::move_to_trash(path, dry_run)
}

pub fn host_tools() -> &'static [HostTool] {
    current::host_tools()
}

pub fn install_tool(id: &str, dry_run: bool) -> Result<bool, String> {
    current::install_tool(id, dry_run)
}

pub fn fuse_available() -> bool {
    current::fuse_available()
}

/// Optional native WinSlim integration root. The platform adapters own this
/// detection so Linux never compiles or evaluates Windows-specific paths.
#[allow(dead_code)]
pub fn winslim_root() -> Option<PathBuf> {
    current::winslim_root()
}

#[allow(dead_code)]
pub fn winslim_available() -> bool {
    winslim_root().is_some()
}

/// Localiza el lanzador NSudo únicamente en la superficie WinSlim o en el
/// PATH del propio Windows. No se busca en Linux ni se descarga nada aquí.
#[allow(dead_code)]
pub fn nsudo_path() -> Option<PathBuf> {
    current::nsudo_path()
}

#[cfg(test)]
mod tests {
    use super::sanitize_version;

    #[test]
    fn sanitize_version_removes_ansi_and_control_bytes() {
        assert_eq!(
            sanitize_version("btop version: \x1b[1m1.4.7\x1b[0m\n"),
            "btop version: 1.4.7"
        );
    }
}
