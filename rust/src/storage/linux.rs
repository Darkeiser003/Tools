use crate::common::{
    command_exists, command_output, ensure_tool, run_command, run_with_sudo, Context,
};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Gestión Linux de almacenamiento. Las operaciones destructivas de tabla de
/// particiones se delegan al gestor nativo tras una confirmación explícita.
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = first_action(args).unwrap_or("menu");
    match action {
        "status" | "disks" | "overview" => status(ctx),
        "partitions" | "partition" => partitions(ctx),
        "mounts" | "mountpoints" => mounts(),
        "inspect" | "details" => inspect(ctx, target_after(args, action)?),
        "mount" => mount(ctx, target_after(args, action)?),
        "unmount" | "umount" => unmount(ctx, target_after(args, action)?),
        "health" | "smart" => health(ctx, target_after(args, action)?),
        "check" | "filesystem-check" => filesystem_check(ctx, target_after(args, action)?),
        "usage" | "space" | "inodes" => usage(),
        "filesystems" | "filesystem-info" | "uuid" | "labels" => filesystems(),
        "volume-stack" | "lvm" | "btrfs" | "zfs" | "raid" => volume_stack(),
        "blockdev" | "block-device" => block_device_info(ctx, target_after(args, action)?),
        "open-gparted" | "gparted" | "partition-manager" => open_gparted(ctx, args),
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
        .ok_or_else(|| format!("{action} necesita un dispositivo o ruta"))
}

fn status(ctx: &Context) -> Result<(), String> {
    require("df")?;
    println!("=== Resumen de almacenamiento Linux ===");
    println!("Home: {}", ctx.home.display());
    run_capture("df", &["-h"])?;
    mounts()
}

fn partitions(ctx: &Context) -> Result<(), String> {
    println!("=== Discos y particiones Linux ===");
    if command_exists("lsblk") {
        run_capture(
            "lsblk",
            &[
                "-e7",
                "-o",
                "NAME,PATH,SIZE,FSTYPE,LABEL,UUID,MOUNTPOINTS,TYPE,RO,RM",
            ],
        )?;
    } else if !ensure_tool(ctx, "lsblk")? {
        return Err("lsblk es necesario para listar discos y particiones".into());
    }
    if command_exists("parted") {
        println!("\n=== Tablas de particiones (solo lectura) ===");
        run_capture("parted", &["-l"])?;
    } else {
        println!("parted: no instalado; lsblk sigue disponible como alternativa segura.");
    }
    println!(
        "\nGestor completo: {}",
        if command_exists("gparted") {
            "gparted disponible (abrir desde el menú)"
        } else {
            "gparted no instalado; puede instalarse bajo demanda"
        }
    );
    Ok(())
}

fn mounts() -> Result<(), String> {
    println!("\n=== Montajes activos ===");
    if command_exists("findmnt") {
        run_capture("findmnt", &["-r", "-o", "SOURCE,TARGET,FSTYPE,OPTIONS"])
    } else if command_exists("df") {
        run_capture("df", &["-hT"])
    } else {
        Err("no se encontró findmnt ni df para consultar montajes".into())
    }
}

fn inspect(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    println!("=== Detalles de {} ===", target.display());
    if command_exists("lsblk") {
        run_capture_owned(
            "lsblk",
            &["-f".into(), "-p".into(), target.display().to_string()],
        )?;
    }
    if command_exists("blkid") {
        run_capture_owned("blkid", &[target.display().to_string()])?;
    } else {
        println!("blkid: no disponible; instala util-linux si necesitas UUID y tipo de sistema de archivos.");
    }
    Ok(())
}

fn mount(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    if !confirm_or_simulate(ctx, &format!("¿Montar {}?", target.display())) {
        println!("Montaje cancelado.");
        return Ok(());
    }
    let (program, args): (&str, Vec<String>) = if command_exists("udisksctl") {
        (
            "udisksctl",
            vec!["mount".into(), "-b".into(), target.display().to_string()],
        )
    } else {
        ("mount", vec![target.display().to_string()])
    };
    let ok = if program == "mount" {
        run_with_sudo(program, &args, ctx.dry_run)
    } else {
        run_command(program, &args, ctx.dry_run)
    }
    .map_err(|e| e.to_string())?;
    record(ctx, "storage-mount", &target, ok);
    if !ok {
        return Err(format!("no se pudo montar {}", target.display()));
    }
    Ok(())
}

fn unmount(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_unmount_target(raw, ctx.dry_run)?;
    if is_protected_mount(&target) {
        return Err(format!(
            "desmontaje bloqueado para una ruta crítica: {}",
            target.display()
        ));
    }
    if !confirm_or_simulate(ctx, &format!("¿Desmontar {}?", target.display())) {
        println!("Desmontaje cancelado.");
        return Ok(());
    }
    let (program, args): (&str, Vec<String>) =
        if command_exists("udisksctl") && target.starts_with("/dev/") {
            (
                "udisksctl",
                vec!["unmount".into(), "-b".into(), target.display().to_string()],
            )
        } else {
            ("umount", vec![target.display().to_string()])
        };
    let ok = run_with_sudo(program, &args, ctx.dry_run).map_err(|e| e.to_string())?;
    record(ctx, "storage-unmount", &target, ok);
    if !ok {
        return Err(format!("no se pudo desmontar {}", target.display()));
    }
    Ok(())
}

fn health(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    if !command_exists("smartctl") {
        println!("smartctl no está instalado. Puedes instalarlo bajo demanda con doctor --install smartctl.");
        return Ok(());
    }
    println!("=== Salud SMART (solo lectura) ===");
    let ok = run_capture_owned(
        "smartctl",
        &["-H".into(), "-A".into(), target.display().to_string()],
    )
    .is_ok();
    if !ok {
        return Err("smartctl no pudo consultar el dispositivo".into());
    }
    Ok(())
}

fn filesystem_check(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    println!("=== Comprobación del sistema de archivos (no repara) ===");
    let ok = run_command(
        "fsck",
        &["-N".into(), target.display().to_string()],
        ctx.dry_run,
    )
    .map_err(|e| e.to_string())?;
    record(ctx, "storage-filesystem-check", &target, ok);
    if !ok {
        return Err("fsck -N no pudo analizar el dispositivo".into());
    }
    Ok(())
}

fn open_gparted(ctx: &Context, args: &[String]) -> Result<(), String> {
    if !command_exists("gparted") {
        return Err("gparted no está instalado; usa doctor --install gparted si quieres instalar el gestor gráfico".into());
    }
    if !args.iter().any(|arg| arg == "--yes")
        && !confirm_or_simulate(ctx, "¿Abrir GParted? Puede modificar particiones y datos.")
    {
        println!("Apertura cancelada.");
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se abriría gparted.");
        return Ok(());
    }
    Command::new("gparted")
        .spawn()
        .map_err(|e| format!("no se pudo abrir gparted: {e}"))?;
    println!("GParted se ha iniciado. Las operaciones de particionado se realizan allí.");
    record(ctx, "storage-open-manager", Path::new("gparted"), true);
    Ok(())
}

fn open_path(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_path(raw)?;
    if !command_exists("xdg-open") {
        return Err("xdg-open no está disponible".into());
    }
    if !confirm_or_simulate(
        ctx,
        &format!("¿Abrir {} en el explorador?", target.display()),
    ) {
        return Ok(());
    }
    let ok = run_command("xdg-open", &[target.display().to_string()], ctx.dry_run)
        .map_err(|e| e.to_string())?;
    record(ctx, "storage-open-path", &target, ok);
    if !ok {
        return Err("no se pudo abrir la ruta".into());
    }
    Ok(())
}

fn usage() -> Result<(), String> {
    println!("=== Uso de espacio ===");
    require("df")?;
    run_capture("df", &["-hT", "-x", "tmpfs", "-x", "devtmpfs"])?;
    println!("\n=== Uso de inodos ===");
    run_capture("df", &["-hi", "-x", "tmpfs", "-x", "devtmpfs"])
}

fn filesystems() -> Result<(), String> {
    println!("=== Sistemas de archivos, UUID y etiquetas ===");
    if command_exists("lsblk") {
        run_capture(
            "lsblk",
            &[
                "-e7",
                "-fp",
                "-o",
                "NAME,FSTYPE,FSVER,LABEL,UUID,FSAVAIL,FSUSE%,MOUNTPOINTS",
            ],
        )?;
    }
    if command_exists("blkid") {
        println!("\n=== blkid ===");
        run_capture("blkid", &[])?;
    }
    Ok(())
}

fn volume_stack() -> Result<(), String> {
    println!("=== Capas de almacenamiento detectadas ===");
    let tools = [
        (
            "pvs",
            &["pvs", "--options", "pv_name,vg_name,pv_size,pv_free"][..],
        ),
        ("vgs", &["vgs", "--options", "vg_name,vg_size,vg_free"][..]),
        (
            "lvs",
            &["lvs", "--options", "lv_name,vg_name,lv_size,lv_attr"][..],
        ),
        ("cryptsetup", &["cryptsetup", "status"][..]),
        ("btrfs", &["btrfs", "filesystem", "show"][..]),
        ("zpool", &["zpool", "list"][..]),
        ("zfs", &["zfs", "list"][..]),
        ("mdadm", &["mdadm", "--detail", "--scan"][..]),
    ];
    let mut found = false;
    for (name, args) in tools {
        if command_exists(name) {
            found = true;
            println!("\n--- {name} ---");
            if let Err(error) = run_capture(name, &args[1..]) {
                println!("No se pudo consultar {name}: {error}");
            }
        }
    }
    if !found {
        println!("No se detectaron capas LVM, cifrado, Btrfs, ZFS o RAID adicionales.");
    }
    Ok(())
}

fn block_device_info(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    println!("=== Información de bloque: {} ===", target.display());
    if command_exists("blockdev") {
        for query in ["--getsize64", "--getss", "--getro"] {
            let output = Command::new("blockdev")
                .args([query, target.to_string_lossy().as_ref()])
                .output()
                .map_err(|error| format!("no se pudo ejecutar blockdev: {error}"))?;
            println!(
                "{query}: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            );
        }
    } else {
        println!("blockdev no está instalado; lsblk sigue disponible como alternativa.");
    }
    inspect(ctx, raw)
}

fn open_external_manager(ctx: &Context, program: &str, label: &str) -> Result<(), String> {
    if !command_exists(program) {
        println!("{program} no está instalado; puedes instalarlo bajo demanda desde doctor.");
        return Ok(());
    }
    if !confirm_or_simulate(
        ctx,
        &format!("¿Abrir {label}? Revisa cada cambio antes de aplicarlo."),
    ) {
        println!("Apertura cancelada.");
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se abriría {program}.");
        return Ok(());
    }
    Command::new(program)
        .spawn()
        .map_err(|error| format!("no se pudo abrir {label}: {error}"))?;
    println!("{label} se ha iniciado; LTools no aplica cambios destructivos automáticamente.");
    Ok(())
}

fn managers_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Gestores nativos de almacenamiento ===");
        println!("  1) GParted");
        println!("  2) GNOME Disks");
        println!("  3) KDE Partition Manager");
        println!("  4) Abrir una ruta en el explorador");
        println!("  5) Herramientas detectadas");
        println!("  q) Volver");
        let choice =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        match choice.trim() {
            "1" => open_gparted(ctx, &[])?,
            "2" => open_external_manager(ctx, "gnome-disks", "GNOME Disks")?,
            "3" => open_external_manager(ctx, "partitionmanager", "KDE Partition Manager")?,
            "4" => prompt_then(ctx, "Ruta: ", open_path)?,
            "5" => tools()?,
            "" | "q" | "Q" => return Ok(()),
            _ => println!("Opción no válida."),
        }
        if choice.trim() != "" {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn advanced_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Consultas avanzadas de almacenamiento ===");
        println!("  1) Uso de espacio e inodos");
        println!("  2) Sistemas de archivos, UUID y etiquetas");
        println!("  3) LVM, cifrado, Btrfs, ZFS y RAID");
        println!("  4) Información detallada de un dispositivo");
        println!("  5) Gestores nativos");
        println!("  q) Volver");
        let choice =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        match choice.trim() {
            "1" => usage()?,
            "2" => filesystems()?,
            "3" => volume_stack()?,
            "4" => prompt_then(ctx, "Dispositivo (ej. /dev/sdb1): ", block_device_info)?,
            "5" => managers_menu(ctx)?,
            "" | "q" | "Q" => return Ok(()),
            _ => println!("Opción no válida."),
        }
        if choice.trim() != "" {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn tools() -> Result<(), String> {
    println!("=== Herramientas de almacenamiento Linux ===");
    for name in [
        "lsblk",
        "findmnt",
        "df",
        "blkid",
        "mount",
        "umount",
        "udisksctl",
        "parted",
        "fdisk",
        "sfdisk",
        "gdisk",
        "blockdev",
        "fsck",
        "smartctl",
        "nvme",
        "cryptsetup",
        "pvs",
        "vgs",
        "lvs",
        "btrfs",
        "zpool",
        "zfs",
        "mdadm",
        "gparted",
        "gnome-disks",
        "partitionmanager",
        "duf",
        "ncdu",
        "xdg-open",
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
    println!("\nConsultas: status, partitions, mounts, usage, filesystems, volume-stack, inspect, blockdev, health, check.");
    println!("Acciones con confirmación: mount, unmount, open-gparted, open.");
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("\n=== Gestión de discos y almacenamiento Linux ===");
        println!("  1) Resumen de espacio y montajes");
        println!("  2) Discos, particiones y tablas");
        println!("  3) Montajes activos");
        println!("  4) Inspeccionar un dispositivo");
        println!("  5) Montar un dispositivo");
        println!("  6) Desmontar un dispositivo o ruta");
        println!("  7) Salud SMART");
        println!("  8) Comprobar sistema de archivos (no repara)");
        println!("  9) Abrir GParted");
        println!(" 10) Abrir una ruta");
        println!(" 11) Herramientas detectadas");
        println!(" 12) Seleccionar un objetivo y elegir una acción");
        println!("\n Consultas avanzadas de almacenamiento");
        println!(" 13) Uso, inodos, UUID y etiquetas");
        println!(" 14) LVM, cifrado, Btrfs, ZFS y RAID");
        println!(" 15) Gestores nativos alternativos");
        println!("  q) Volver");
        let answer =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        match answer.trim() {
            "1" => status(ctx)?,
            "2" => partitions(ctx)?,
            "3" => mounts()?,
            "4" => prompt_then(ctx, "Dispositivo (ej. /dev/sdb1): ", inspect)?,
            "5" => prompt_then(ctx, "Dispositivo a montar: ", mount)?,
            "6" => prompt_then(ctx, "Dispositivo o ruta a desmontar: ", unmount)?,
            "7" => prompt_then(ctx, "Disco (ej. /dev/sda): ", health)?,
            "8" => prompt_then(ctx, "Dispositivo a comprobar: ", filesystem_check)?,
            "9" => open_gparted(ctx, &[])?,
            "10" => prompt_then(ctx, "Ruta: ", open_path)?,
            "11" => tools()?,
            "12" => guided_target(ctx)?,
            "13" => advanced_menu(ctx)?,
            "14" => volume_stack()?,
            "15" => managers_menu(ctx)?,
            "q" | "Q" | "" => return Ok(()),
            _ => println!("Opción no válida."),
        }
        if answer.trim() != "" {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

/// Flujo orientado a personas: primero limita la elección a dispositivos
/// detectados y después muestra solo acciones que aceptan ese tipo de
/// objetivo. No incluye borrado ni particionado automatizado: esas acciones
/// se delegan al gestor nativo, donde el usuario puede revisar cada cambio.
fn guided_target(ctx: &Context) -> Result<(), String> {
    let mut targets = Vec::new();
    if command_exists("lsblk") {
        if let Ok(output) = Command::new("lsblk")
            .args(["-nrpo", "NAME,TYPE,SIZE,FSTYPE,MOUNTPOINTS"])
            .output()
        {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                if fields.len() >= 2 && matches!(fields[1], "disk" | "part" | "crypt") {
                    targets.push(fields.join(" | "));
                }
            }
        }
    }
    println!("=== Selección segura de almacenamiento ===");
    if targets.is_empty() {
        println!("No se detectaron dispositivos; puedes introducir una ruta /dev/... manualmente.");
    } else {
        for (index, target) in targets.iter().enumerate() {
            println!("  {}) {}", index + 1, target);
        }
    }
    let Some(answer) = crate::menu_input("Número, ruta /dev/... o Enter para volver: ") else {
        return Ok(());
    };
    if answer.is_empty() {
        return Ok(());
    }
    let target = if let Ok(index) = answer.parse::<usize>() {
        targets
            .get(index.saturating_sub(1))
            .and_then(|value| value.split('|').next())
            .map(str::trim)
            .unwrap_or_default()
            .to_string()
    } else {
        answer
    };
    validate_device(&target, ctx.dry_run)?;
    loop {
        crate::clear_screen();
        println!("=== Acciones para {} ===", target);
        println!("  1) Inspeccionar detalles");
        println!("  2) Consultar salud SMART");
        println!("  3) Comprobar sistema de archivos (no repara)");
        println!("  4) Montar (requiere confirmación)");
        println!("  5) Desmontar (requiere confirmación y bloquea rutas críticas)");
        println!("  6) Abrir GParted para particionar manualmente");
        println!("  q) Volver");
        let choice =
            crate::menu_input("Elige una acción (Enter para volver): ").unwrap_or_default();
        let result = match choice.as_str() {
            "1" => inspect(ctx, &target),
            "2" => health(ctx, &target),
            "3" => filesystem_check(ctx, &target),
            "4" => mount(ctx, &target),
            "5" => unmount(ctx, &target),
            "6" => open_gparted(ctx, &[]),
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

fn validate_device(raw: &str, dry_run: bool) -> Result<PathBuf, String> {
    if raw.chars().any(|c| c.is_control()) {
        return Err("el dispositivo contiene caracteres de control".into());
    }
    let value = raw.trim();
    if value.is_empty()
        || value.chars().any(|c| {
            c.is_control()
                || c.is_whitespace()
                || !c.is_ascii_alphanumeric() && !"/._-:".contains(c)
        })
        || !value.starts_with("/dev/")
    {
        return Err("el dispositivo debe ser una ruta /dev/... con caracteres válidos".into());
    }
    let path = PathBuf::from(value);
    if !dry_run && !path.exists() {
        return Err(format!("no existe el dispositivo: {value}"));
    }
    Ok(path)
}

fn validate_unmount_target(raw: &str, dry_run: bool) -> Result<PathBuf, String> {
    if raw.chars().any(|c| c.is_control()) {
        return Err("la ruta contiene caracteres de control".into());
    }
    let value = raw.trim();
    if value.is_empty()
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
        || (!value.starts_with("/dev/") && !value.starts_with('/'))
    {
        return Err("indica un dispositivo /dev/... o una ruta absoluta de montaje".into());
    }
    let path = PathBuf::from(value);
    if !dry_run && !path.exists() {
        return Err(format!("no existe la ruta: {value}"));
    }
    Ok(path)
}

fn validate_path(raw: &str) -> Result<PathBuf, String> {
    let value = raw.trim();
    if value.is_empty() || value.chars().any(|c| c.is_control()) {
        return Err("ruta vacía o no válida".into());
    }
    Ok(PathBuf::from(value))
}
fn is_protected_mount(path: &Path) -> bool {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    matches!(
        canonical.to_string_lossy().as_ref(),
        "/" | "/home" | "/boot" | "/usr" | "/var" | "/etc" | "/run"
    )
}
fn confirm_or_simulate(ctx: &Context, question: &str) -> bool {
    ctx.dry_run || crate::common::ask(question)
}
fn record(ctx: &Context, operation: &str, target: &Path, ok: bool) {
    if let Some(plan) = &ctx.plan {
        let _ = plan.record(
            operation,
            target,
            if ok { "executed" } else { "failed" },
            false,
            "",
            "",
        );
    }
}
fn require(name: &str) -> Result<(), String> {
    if command_exists(name) {
        Ok(())
    } else {
        Err(format!(
            "{name} no está instalado; usa doctor --install {name} si tu distribución lo ofrece"
        ))
    }
}
fn run_capture(program: &str, args: &[&str]) -> Result<(), String> {
    run_capture_owned(
        program,
        &args
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>(),
    )
}
fn run_capture_owned(program: &str, args: &[String]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("no se pudo ejecutar {program}: {e}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if error.is_empty() {
            format!("{program} terminó con código {}", output.status)
        } else {
            error
        })
    }
}
#[allow(dead_code)]
fn _command_path(name: &str) -> Option<String> {
    command_output("sh", &["-c", &format!("command -v -- {name}")])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn solo_acepta_dispositivos_linux() {
        assert!(validate_device("/dev/sda", true).is_ok());
        assert!(validate_device("/etc/passwd", true).is_err());
        assert!(validate_device("/dev/sda;rm", true).is_err());
    }
    #[test]
    fn rechaza_espacios_y_control_en_destinos() {
        assert!(validate_device("/dev/sda 1", true).is_err());
        assert!(validate_unmount_target("/dev/sda\n", true).is_err());
    }
    #[test]
    fn protege_montajes_criticos() {
        assert!(is_protected_mount(Path::new("/")));
        assert!(is_protected_mount(Path::new("/home")));
        assert!(!is_protected_mount(Path::new("/mnt/data")));
    }
}
