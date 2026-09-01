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

pub fn host_tool_available(tool: &super::HostTool) -> bool {
    if tool.id == "trash" {
        return command_exists("powershell") || command_exists("pwsh");
    }
    if tool.id == "powershell" {
        return command_exists("powershell") || command_exists("pwsh");
    }
    if !tool.command.starts_with("Get-") {
        return command_exists(tool.command);
    }
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
            &format!("Get-Command {} -ErrorAction Stop | Out-Null", tool.command),
        ],
    )
    .is_some()
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

pub fn host_tools() -> &'static [super::HostTool] {
    HOST_TOOLS
}

static HOST_TOOLS: &[super::HostTool] = &[
    tool(
        "powershell",
        "system",
        "PowerShell para automatización Windows",
        false,
        false,
        "",
    ),
    tool("sc.exe", "system", "service-control", false, false, ""),
    tool("tasklist", "system", "process-inspection", false, false, ""),
    tool("taskkill", "system", "process-control", false, false, ""),
    tool("wevtutil", "system", "Windows-event-log", false, false, ""),
    tool(
        "Get-CimInstance",
        "system",
        "service-and-process-inventory",
        false,
        false,
        "",
    ),
    tool("trash", "cleanup", "Windows-recycle-bin", false, false, ""),
    tool(
        "jq",
        "configuration",
        "Heroic-json-rewrite-and-validation",
        false,
        true,
        "jqlang.jq",
    ),
];

const fn tool(
    command: &'static str,
    category: &'static str,
    feature: &'static str,
    required: bool,
    installable: bool,
    install_package: &'static str,
) -> super::HostTool {
    super::HostTool {
        id: command,
        command,
        category,
        feature,
        required,
        installable,
        install_package,
    }
}

pub fn install_tool(id: &str, dry_run: bool) -> Result<bool, String> {
    let tool = HOST_TOOLS
        .iter()
        .find(|tool| tool.id == id)
        .ok_or_else(|| format!("dependencia no gestionada por LTools: {id}"))?;
    if host_tool_available(tool) {
        println!("Ya disponible: {}", tool.command);
        return Ok(true);
    }
    if !tool.installable || tool.install_package.is_empty() {
        println!(
            "{} no está disponible. Es una herramienta integrada de Windows y LTools no instala componentes del sistema.",
            tool.command
        );
        return Ok(false);
    }
    if !command_exists("winget") {
        println!(
            "Falta {}. No se encontró winget; LTools no instalará un gestor de paquetes para resolverlo.",
            tool.command
        );
        return Ok(false);
    }
    let args = vec![
        "install".into(),
        "--id".into(),
        tool.install_package.into(),
        "-e".into(),
        "--accept-source-agreements".into(),
        "--accept-package-agreements".into(),
    ];
    println!(
        "Falta {} para {}. Se propone usar winget: winget {}",
        tool.command,
        tool.feature,
        args.iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    );
    if !crate::common::ask("¿Instalar esta dependencia ahora?") {
        println!("Instalación cancelada; no se modifica el sistema.");
        return Ok(false);
    }
    let ok = crate::platform::run_with_privilege("winget", &args, dry_run)
        .map_err(|error| error.to_string())?;
    Ok(ok && host_tool_available(tool))
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
