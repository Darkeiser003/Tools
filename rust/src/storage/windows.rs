use crate::common::{command_exists, ensure_tool, Context};
use std::path::PathBuf;
use std::process::Command;

/// Gestión nativa de almacenamiento Windows. LTools consulta y abre las
/// herramientas del sistema; no genera scripts destructivos de DiskPart.
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = first_action(args).unwrap_or("menu");
    match action {
        "status" | "disks" | "overview" => status(),
        "partitions" | "partition" => partitions(ctx),
        "mounts" | "mountpoints" => mounts(),
        "usage" | "space" => usage(),
        "pools" | "storage-pools" => storage_pools(),
        "bitlocker" | "encryption" => bitlocker(),
        "inspect" | "details" => inspect(ctx, target_after(args, action)?),
        "health" | "smart" => health(ctx, target_after(args, action)?),
        "check" | "filesystem-check" => filesystem_check(ctx, target_after(args, action)?),
        "open-disk-management" | "disk-management" => {
            open_native(ctx, args, "diskmgmt.msc", "Administrador de discos")
        }
        "open-diskpart" | "diskpart" => open_native(ctx, args, "diskpart.exe", "DiskPart"),
        "open" | "browse" => open_path(ctx, target_after(args, action)?),
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
fn target_after<'a>(args: &'a [String], action: &str) -> Result<&'a str, String> {
    let index = args
        .iter()
        .position(|arg| arg == action)
        .ok_or_else(|| format!("no se encontró la acción {action}"))?;
    args.get(index + 1)
        .map(String::as_str)
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| format!("{action} necesita una letra de unidad o disco"))
}

fn status() -> Result<(), String> {
    require_powershell()?;
    println!("=== Resumen de almacenamiento Windows ===");
    powershell("Get-Volume | Sort-Object DriveLetter | Format-Table -AutoSize DriveLetter,FileSystemLabel,FileSystem,HealthStatus,SizeRemaining,Size")
}

fn partitions(ctx: &Context) -> Result<(), String> {
    require_powershell()?;
    println!("=== Discos y particiones Windows ===");
    powershell("Get-Disk | Get-Partition | ForEach-Object { $v=$_.DriveLetter; '{0}`t{1}`t{2}`t{3}`t{4}' -f $_.DiskNumber,$_.PartitionNumber,$_.Size,$_.Type,$v }")?;
    if command_exists("diskpart") {
        println!("\nDiskPart: disponible como gestor nativo (apertura controlada desde el menú).");
    } else if !ensure_tool(ctx, "diskpart")? {
        println!("DiskPart no detectado; las consultas PowerShell siguen disponibles.");
    }
    Ok(())
}

fn mounts() -> Result<(), String> {
    if command_exists("mountvol") {
        println!("=== Volúmenes y puntos de montaje ===");
        run_command("mountvol", &[])?;
        Ok(())
    } else {
        powershell("Get-Volume | Where-Object DriveLetter | Select-Object DriveLetter,Path,FileSystem,HealthStatus | Format-Table -AutoSize")
    }
}

fn usage() -> Result<(), String> {
    require_powershell()?;
    powershell("Get-Volume | Where-Object DriveLetter | Sort-Object DriveLetter | Format-Table DriveLetter,FileSystemLabel,FileSystem,HealthStatus,SizeRemaining,Size -AutoSize")
}

fn storage_pools() -> Result<(), String> {
    require_powershell()?;
    println!("=== Espacios de almacenamiento Windows ===");
    powershell("Get-StoragePool -ErrorAction SilentlyContinue | Format-Table FriendlyName,HealthStatus,OperationalStatus,Size,AllocatedSize -AutoSize")?;
    println!("\n=== Discos virtuales ===");
    powershell("Get-VirtualDisk -ErrorAction SilentlyContinue | Format-Table FriendlyName,HealthStatus,OperationalStatus,ResiliencySettingName,Size,FootprintOnPool -AutoSize")
}

fn bitlocker() -> Result<(), String> {
    require_powershell()?;
    powershell("Get-BitLockerVolume -ErrorAction SilentlyContinue | Format-Table MountPoint,VolumeStatus,ProtectionStatus,EncryptionMethod,EncryptionPercentage -AutoSize")
}

fn inspect(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_target(raw, ctx.dry_run)?;
    let value = target.to_string_lossy().replace('\\', "/");
    if let Some(number) = value.strip_prefix("disk:") {
        if !number.chars().all(|c| c.is_ascii_digit()) {
            return Err("el número de disco no es válido".into());
        }
        powershell(&format!("Get-Disk -Number {number} | Format-List *"))
    } else {
        let letter = value.trim_end_matches(':').trim_end_matches('/');
        powershell(&format!("Get-Volume -DriveLetter {letter} | Format-List *"))
    }
}

fn health(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_drive(raw, ctx.dry_run)?;
    println!("=== Salud del volumen (escaneo, no repara) ===");
    powershell(&format!("Repair-Volume -DriveLetter {target} -Scan"))
}

fn filesystem_check(ctx: &Context, raw: &str) -> Result<(), String> {
    health(ctx, raw)
}

fn open_native(ctx: &Context, args: &[String], program: &str, label: &str) -> Result<(), String> {
    if !args.iter().any(|arg| arg == "--yes")
        && !confirm_or_simulate(
            ctx,
            &format!("¿Abrir {label}? Las operaciones de particionado pueden destruir datos."),
        )
    {
        println!("Apertura cancelada.");
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se abriría {program}.");
        return Ok(());
    }
    Command::new(program)
        .spawn()
        .map_err(|e| format!("no se pudo abrir {label}: {e}"))?;
    println!("{label} se ha iniciado. LTools no ejecuta cambios destructivos automáticamente.");
    Ok(())
}

fn open_path(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_path(raw)?;
    if !confirm_or_simulate(
        ctx,
        &format!("¿Abrir {} en el Explorador?", target.display()),
    ) {
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se abriría explorer.exe {}.", target.display());
        return Ok(());
    }
    Command::new("explorer.exe")
        .arg(&target)
        .spawn()
        .map_err(|e| format!("no se pudo abrir el Explorador: {e}"))?;
    Ok(())
}

fn tools() -> Result<(), String> {
    println!("=== Herramientas de almacenamiento Windows ===");
    for name in [
        "powershell",
        "pwsh",
        "diskpart",
        "diskmgmt.msc",
        "mountvol",
        "explorer.exe",
    ] {
        println!(
            "{name}: {}",
            if command_exists(name) {
                "disponible"
            } else {
                "no instalado"
            }
        );
    }
    for name in [
        "Get-Disk",
        "Get-Partition",
        "Get-Volume",
        "Get-StoragePool",
        "Get-VirtualDisk",
        "Get-BitLockerVolume",
        "Repair-Volume",
    ] {
        println!(
            "{name}: {}",
            if powershell_check(name) {
                "disponible"
            } else {
                "no disponible"
            }
        );
    }
    println!(
        "\nConsultas: status, partitions, mounts, usage, pools, bitlocker, inspect, health, check."
    );
    println!("Gestión nativa: open-disk-management, open-diskpart y open.");
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("\n=== Gestión de discos y almacenamiento Windows ===");
        println!("  1) Resumen de espacio y volúmenes");
        println!("  2) Discos y particiones");
        println!("  3) Volúmenes y puntos de montaje");
        println!("  4) Inspeccionar disco o unidad");
        println!("  5) Escanear la salud de una unidad");
        println!("  6) Abrir Administrador de discos");
        println!("  7) Abrir DiskPart");
        println!("  8) Abrir una ruta");
        println!("  9) Herramientas detectadas");
        println!(" 10) Seleccionar un volumen y elegir una acción");
        println!(" 11) Uso de espacio por volumen");
        println!(" 12) Espacios de almacenamiento y discos virtuales");
        println!(" 13) Estado de BitLocker");
        println!("  q) Volver");
        let answer =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        match answer.trim() {
            "1" => status()?,
            "2" => partitions(ctx)?,
            "3" => mounts()?,
            "4" => prompt_then(ctx, "Unidad (C:) o disco (disk:N): ", inspect)?,
            "5" => prompt_then(ctx, "Letra de unidad (C): ", health)?,
            "6" => open_native(ctx, &[], "diskmgmt.msc", "Administrador de discos")?,
            "7" => open_native(ctx, &[], "diskpart.exe", "DiskPart")?,
            "8" => prompt_then(ctx, "Ruta: ", open_path)?,
            "9" => tools()?,
            "10" => guided_target(ctx)?,
            "11" => usage()?,
            "12" => storage_pools()?,
            "13" => bitlocker()?,
            "q" | "Q" | "" => return Ok(()),
            _ => println!("Opción no válida."),
        }
        if answer.trim() != "" {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

/// Selector guiado Windows. C: se muestra como protegido, pero nunca se
/// utiliza como valor por defecto. Las acciones de borrado/clear no se
/// automatizan: DiskPart se abre explícitamente para una revisión manual.
fn guided_target(ctx: &Context) -> Result<(), String> {
    require_powershell()?;
    let output = powershell_output("Get-Volume | Where-Object DriveLetter | ForEach-Object { '{0}`t{1}`t{2}`t{3}' -f $_.DriveLetter,$_.FileSystem,$_.HealthStatus,$_.SizeRemaining }")?;
    let mut targets = Vec::new();
    println!("=== Selección segura de almacenamiento Windows ===");
    for line in output.lines() {
        let mut fields = line.split('\t');
        let letter = fields.next().unwrap_or_default().trim().to_uppercase();
        if letter.is_empty() || !letter.chars().all(|value| value.is_ascii_alphabetic()) {
            continue;
        }
        let details = fields.collect::<Vec<_>>().join(" | ");
        if letter == "C" {
            println!(
                "  - {}: {} (protegida; no se preselecciona)",
                letter, details
            );
        } else {
            targets.push(letter.clone());
            println!("  {}) {}: {}", targets.len(), letter, details);
        }
    }
    if targets.is_empty() {
        println!("No hay volúmenes secundarios seleccionables. C: queda excluido por seguridad.");
        let _ = crate::menu_input("Pulsa Enter para volver...");
        return Ok(());
    }
    let Some(answer) = crate::menu_input("Número, letra de unidad o Enter para volver: ") else {
        return Ok(());
    };
    if answer.is_empty() {
        return Ok(());
    }
    let target = if let Ok(index) = answer.parse::<usize>() {
        targets
            .get(index.saturating_sub(1))
            .cloned()
            .unwrap_or_default()
    } else {
        answer.trim().trim_end_matches(':').to_uppercase()
    };
    let target = validate_target(&target, ctx.dry_run)?;
    let letter = target.to_string_lossy().to_string();
    if letter.trim_end_matches(':').eq_ignore_ascii_case("C") {
        return Err("C: está protegido en el selector seguro; introdúcelo manualmente en una acción avanzada".into());
    }
    loop {
        crate::clear_screen();
        println!("=== Acciones para {} ===", letter);
        println!("  1) Inspeccionar detalles");
        println!("  2) Escanear salud del volumen (no repara)");
        println!("  3) Abrir Administrador de discos");
        println!("  4) Abrir DiskPart para acciones avanzadas");
        println!("  5) Abrir en el Explorador");
        println!("  q) Volver");
        let choice =
            crate::menu_input("Elige una acción (Enter para volver): ").unwrap_or_default();
        let result = match choice.as_str() {
            "1" => inspect(ctx, &letter),
            "2" => health(ctx, &letter),
            "3" => open_native(ctx, &[], "diskmgmt.msc", "Administrador de discos"),
            "4" => open_native(ctx, &[], "diskpart.exe", "DiskPart"),
            "5" => open_path(ctx, &format!("{}\\", letter.trim_end_matches(':'))),
            "" | "q" | "quit" | "salir" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        let _ = crate::menu_input("Pulsa Enter para continuar...");
    }
}

fn prompt_then(
    ctx: &Context,
    question: &str,
    action: fn(&Context, &str) -> Result<(), String>,
) -> Result<(), String> {
    if let Some(value) = crate::common::prompt_path(question) {
        action(ctx, &value.to_string_lossy())?;
    }
    Ok(())
}
fn validate_target(raw: &str, dry_run: bool) -> Result<PathBuf, String> {
    let value = raw.trim();
    if value.is_empty() || value.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("la unidad o disco no es válido".into());
    }
    let normalized = value.replace('\\', "/");
    if !((normalized.len() == 2
        && normalized.as_bytes()[1] == b':'
        && normalized.as_bytes()[0].is_ascii_alphabetic())
        || (normalized.len() == 3
            && normalized.as_bytes()[1] == b':'
            && normalized.as_bytes()[2] == b'/'
            && normalized.as_bytes()[0].is_ascii_alphabetic()))
        && !normalized
            .strip_prefix("disk:")
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
    {
        return Err("usa una unidad como C: o un disco como disk:0".into());
    }
    if !dry_run && normalized.starts_with("disk:") { /* PowerShell validará la existencia. */ }
    Ok(PathBuf::from(value))
}
fn validate_drive(raw: &str, dry_run: bool) -> Result<String, String> {
    let target = validate_target(raw, dry_run)?;
    let value = target
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .trim_end_matches(':')
        .to_uppercase();
    if value.len() != 1 || !value.as_bytes()[0].is_ascii_alphabetic() {
        return Err("Repair-Volume necesita una letra de unidad como C:".into());
    }
    Ok(value)
}
fn validate_path(raw: &str) -> Result<PathBuf, String> {
    let value = raw.trim();
    if value.is_empty() || value.chars().any(|c| c.is_control()) {
        return Err("ruta vacía o no válida".into());
    }
    Ok(PathBuf::from(value))
}
fn confirm_or_simulate(ctx: &Context, question: &str) -> bool {
    ctx.dry_run || crate::common::ask(question)
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
        .map_err(|e| format!("no se pudo ejecutar PowerShell: {e}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn powershell_output(script: &str) -> Result<String, String> {
    let program = if command_exists("powershell") {
        "powershell"
    } else {
        "pwsh"
    };
    let output = Command::new(program)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| format!("no se pudo ejecutar PowerShell: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("no se pudo ejecutar {program}: {e}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
fn powershell_check(command: &str) -> bool {
    if !(command_exists("powershell") || command_exists("pwsh")) {
        return false;
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valida_unidades_y_discos() {
        assert!(validate_target("C:", true).is_ok());
        assert!(validate_target("disk:0", true).is_ok());
        assert!(validate_target("C: & del", true).is_err());
        assert!(validate_target("/dev/sda", true).is_err());
    }
    #[test]
    fn health_solo_acepta_letra() {
        assert!(validate_drive("C:", true).is_ok());
        assert!(validate_drive("disk:0", true).is_err());
    }
}
