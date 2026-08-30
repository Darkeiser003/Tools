//! Adaptadores aislados por sistema operativo.
//!
//! El resto del backend solo usa esta interfaz. Así Linux puede conocer
//! systemd, montajes y la papelera XDG sin contaminar el binario Windows, y
//! Windows puede usar UAC, `sc.exe`, PowerShell y su papelera nativa.

use std::io;
use std::path::{Path, PathBuf};

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

pub fn run_with_privilege(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    current::run_with_privilege(program, args, dry_run)
}

pub fn critical_path(path: &Path) -> bool {
    current::critical_path(path)
}

pub fn move_to_trash(path: &Path, dry_run: bool) -> io::Result<bool> {
    current::move_to_trash(path, dry_run)
}

pub fn host_tools() -> &'static [&'static str] {
    current::host_tools()
}

pub fn fuse_available() -> bool {
    current::fuse_available()
}
