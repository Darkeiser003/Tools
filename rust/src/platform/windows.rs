use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn timestamp() -> String {
    command_output(
        "powershell",
        &["-NoProfile", "-Command", "Get-Date -Format yyyyMMdd-HHmmss"],
    )
    .or_else(|| {
        command_output(
            "pwsh",
            &["-NoProfile", "-Command", "Get-Date -Format yyyyMMdd-HHmmss"],
        )
    })
    .unwrap_or_else(|| std::process::id().to_string())
}

pub fn command_exists(name: &str) -> bool {
    let path = match std::env::var_os("PATH") {
        Some(path) => path,
        None => return false,
    };
    let extensions = std::env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
    std::env::split_paths(&path).any(|dir| {
        dir.join(name).is_file()
            || extensions
                .to_string_lossy()
                .split(';')
                .any(|ext| dir.join(format!("{name}{ext}")).is_file())
    })
}

pub fn run_with_privilege(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    println!("  > {} {}", program, args.join(" "));
    if dry_run {
        return Ok(true);
    }
    if is_elevated() {
        return Ok(Command::new(program).args(args).status()?.success());
    }
    let shell = if command_exists("powershell") {
        "powershell"
    } else if command_exists("pwsh") {
        "pwsh"
    } else {
        return Ok(false);
    };
    let arguments = args
        .iter()
        .map(|value| format!("'{}'", value.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");
    let script = format!(
        "$p=Start-Process -FilePath '{}' -ArgumentList @({}) -Verb RunAs -Wait -PassThru; exit $p.ExitCode",
        program.replace('\'', "''"),
        arguments
    );
    Ok(Command::new(shell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .status()?
        .success())
}

pub fn critical_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    let trimmed = normalized.trim_end_matches('/');
    trimmed.is_empty()
        || normalized.ends_with(":/")
        || [
            "/windows",
            "/program files",
            "/program files (x86)",
            "/programdata",
            "/users",
        ]
        .iter()
        .any(|suffix| trimmed.ends_with(suffix))
}

pub fn move_to_trash(path: &Path, dry_run: bool) -> io::Result<bool> {
    if !path.exists() {
        eprintln!("No existe: {}", path.display());
        return Ok(false);
    }
    if critical_path(path) {
        eprintln!("Bloqueado por seguridad: {}", path.display());
        return Ok(false);
    }
    if dry_run {
        println!("Simulación: se movería a la papelera: {}", path.display());
        return Ok(true);
    }
    let shell = if command_exists("powershell") {
        "powershell"
    } else if command_exists("pwsh") {
        "pwsh"
    } else {
        eprintln!("No se encontró PowerShell; el origen se conserva.");
        return Ok(false);
    };
    let escaped = path.to_string_lossy().replace('\'', "''");
    let script = format!(
        r#"Add-Type -AssemblyName Microsoft.VisualBasic; $p='{escaped}'; if ([IO.Directory]::Exists($p)) {{ [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory($p, [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs, [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin) }} elseif ([IO.File]::Exists($p)) {{ [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile($p, [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs, [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin) }} else {{ exit 2 }}; if (Test-Path -LiteralPath $p) {{ exit 1 }}"#
    );
    let status = Command::new(shell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
        ])
        .arg(script)
        .status()?;
    Ok(status.success() && !path.exists())
}

pub fn host_tools() -> &'static [&'static str] {
    &[
        "powershell",
        "where",
        "tasklist",
        "sc",
        "wevtutil",
        "winget",
    ]
}

pub fn fuse_available() -> bool {
    false
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()
    })
}

fn is_elevated() -> bool {
    let shell = if command_exists("powershell") {
        "powershell"
    } else if command_exists("pwsh") {
        "pwsh"
    } else {
        return false;
    };
    command_output(
        shell,
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)",
        ],
    )
    .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
}
