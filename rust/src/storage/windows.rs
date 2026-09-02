use crate::common::{command_exists, ensure_tool, Context};
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = first_action(args).unwrap_or("status");
    match action {
        "status" | "disks" => status(),
        "partitions" | "partition" => partitions(ctx),
        "tools" => tools(),
        "menu" => menu(ctx),
        _ => Err(format!("acción de almacenamiento desconocida: {action}")),
    }
}

fn first_action(args: &[String]) -> Option<&str> {
    args.iter()
        .map(String::as_str)
        .find(|arg| !arg.starts_with('-'))
}

fn status() -> Result<(), String> {
    require_powershell()?;
    println!("=== Almacenamiento Windows ===");
    powershell("Get-Volume | Sort-Object DriveLetter | Format-Table -AutoSize DriveLetter,FileSystemLabel,FileSystem,SizeRemaining,Size")?;
    Ok(())
}

fn partitions(ctx: &Context) -> Result<(), String> {
    require_powershell()?;
    println!("=== Discos y particiones Windows ===");
    powershell("Get-Disk | Get-Partition | ForEach-Object { $v=$_.DriveLetter; '{0}`t{1}`t{2}`t{3}`t{4}' -f $_.DiskNumber,$_.PartitionNumber,$_.Size,$_.Type,$v }")?;
    if command_exists("diskpart") {
        println!("diskpart: disponible; LTools usa Get-Disk/Get-Partition para consultas sin modificar tablas.");
    } else if !ensure_tool(ctx, "diskpart")? {
        println!("diskpart: no detectado; las consultas PowerShell siguen disponibles.");
    }
    Ok(())
}

fn tools() -> Result<(), String> {
    println!("=== Herramientas de almacenamiento Windows ===");
    for name in ["diskpart", "mountvol", "powershell", "pwsh"] {
        println!(
            "{name}: {}",
            if host_tool_available(name) {
                "disponible"
            } else {
                "no instalado"
            }
        );
    }
    for name in ["Get-Disk", "Get-Partition", "Get-Volume"] {
        let available = if command_exists("powershell") || command_exists("pwsh") {
            powershell_check(name)
        } else {
            false
        };
        println!(
            "{name}: {}",
            if available {
                "disponible"
            } else {
                "no disponible"
            }
        );
    }
    Ok(())
}

fn host_tool_available(id: &str) -> bool {
    crate::platform::host_tools()
        .iter()
        .find(|tool| tool.id == id)
        .is_some_and(crate::platform::host_tool_available)
}

fn menu(ctx: &Context) -> Result<(), String> {
    let mut first = true;
    loop {
        if !first {
            crate::clear_screen();
        }
        first = false;
        println!("\n=== Discos y almacenamiento Windows ===");
        println!("  1) Estado de discos y volúmenes");
        println!("  2) Particiones");
        println!("  3) Herramientas detectadas");
        println!("  q) Volver");
        let answer = crate::menu_input("Elige una opción: ").unwrap_or_default();
        match answer.trim().to_lowercase().as_str() {
            "1" => status()?,
            "2" => partitions(ctx)?,
            "3" => tools()?,
            "q" | "quit" | "salir" | "" => return Ok(()),
            _ => println!("Opción no válida."),
        }
    }
}

fn require_powershell() -> Result<(), String> {
    if command_exists("powershell") || command_exists("pwsh") {
        Ok(())
    } else {
        Err("PowerShell no está disponible para consultar discos Windows".into())
    }
}

fn powershell(script: &str) -> Result<(), String> {
    let program = if command_exists("powershell") {
        "powershell"
    } else {
        "pwsh"
    };
    let output = Command::new(program)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|error| format!("no se pudo ejecutar PowerShell: {error}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn powershell_check(command: &str) -> bool {
    let program = if command_exists("powershell") {
        "powershell"
    } else {
        "pwsh"
    };
    Command::new(program)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("Get-Command {command} -ErrorAction Stop | Out-Null"),
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
