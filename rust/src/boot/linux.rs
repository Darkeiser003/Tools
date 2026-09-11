use crate::common::{ask, command_exists, run_with_sudo, Context};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .find(|value| !value.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("menu");
    match action {
        "status" | "inspect" => status(),
        "efi-entries" | "efi" => efi_entries(),
        "grub-entries" | "grub" => grub_entries_report(),
        "systemd-boot" | "bootctl" => bootctl_status(),
        "secure-boot" | "secureboot" => secure_boot_status(),
        "set-next" | "grub-reboot" => set_next(ctx, args),
        "clear-next" => clear_next(ctx, args),
        "plan" | "preview" => plan(ctx),
        "menu" => menu(ctx),
        _ => Err("boot admite menu, status, efi-entries, grub-entries, systemd-boot, secure-boot, set-next, clear-next o plan".into()),
    }
}

fn status() -> Result<(), String> {
    println!("=== Arranque Linux ===");
    println!(
        "Modo firmware: {}",
        if Path::new("/sys/firmware/efi").exists() {
            "UEFI"
        } else {
            "BIOS/CSM o no detectable"
        }
    );
    println!("/boot: {}", mount_state("/boot"));
    println!("/boot/efi: {}", mount_state("/boot/efi"));
    bootctl_status()?;
    efi_entries()?;
    probe("GRUB", "grub-install", &["--version"]);
    probe("generador GRUB", "grub-mkconfig", &["--version"]);
    secure_boot_status()?;
    Ok(())
}

fn efi_entries() -> Result<(), String> {
    println!("=== Entradas EFI / NVRAM ===");
    if command_exists("efibootmgr") {
        probe("efibootmgr", "efibootmgr", &["-v"]);
    } else {
        println!("efibootmgr no está disponible o el sistema no expone EFI.");
    }
    Ok(())
}

fn bootctl_status() -> Result<(), String> {
    println!("=== systemd-boot ===");
    if command_exists("bootctl") {
        probe("bootctl", "bootctl", &["status"]);
    } else {
        println!("bootctl no está disponible.");
    }
    Ok(())
}

fn secure_boot_status() -> Result<(), String> {
    println!("=== Secure Boot ===");
    if command_exists("mokutil") {
        probe("mokutil", "mokutil", &["--sb-state"]);
    } else {
        println!("mokutil no está disponible.");
    }
    Ok(())
}

fn grub_config_path() -> Option<PathBuf> {
    ["/boot/grub/grub.cfg", "/boot/grub2/grub.cfg"]
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn grub_entries() -> Result<Vec<String>, String> {
    let path = grub_config_path()
        .ok_or_else(|| "no se encontró /boot/grub/grub.cfg ni /boot/grub2/grub.cfg".to_owned())?;
    let content = std::fs::read_to_string(&path)
        .map_err(|error| format!("no se pudo leer {}: {error}", path.display()))?;
    let mut entries = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("menuentry ") || trimmed.starts_with("submenu ")) {
            continue;
        }
        let rest = trimmed
            .split_once(' ')
            .map(|(_, value)| value)
            .unwrap_or_default();
        let title = if let Some(value) = rest.strip_prefix('\'') {
            value.split('\'').next().unwrap_or_default()
        } else if let Some(value) = rest.strip_prefix('"') {
            value.split('"').next().unwrap_or_default()
        } else {
            rest.split_whitespace().next().unwrap_or_default()
        };
        if !title.is_empty() && !entries.iter().any(|entry| entry == title) {
            entries.push(title.to_owned());
        }
    }
    Ok(entries)
}

fn grub_entries_report() -> Result<(), String> {
    println!("=== Entradas GRUB ===");
    match grub_entries() {
        Ok(entries) if entries.is_empty() => {
            println!("No se encontraron entradas menuentry/submenu.")
        }
        Ok(entries) => {
            for (index, entry) in entries.iter().enumerate() {
                println!("{:>3}) {}", index + 1, entry);
            }
        }
        Err(error) => println!("No disponible: {error}"),
    }
    Ok(())
}

fn argument_value(args: &[String], option: &str) -> Option<String> {
    args.iter()
        .position(|value| value == option)
        .and_then(|index| args.get(index + 1))
        .filter(|value| !value.starts_with('-'))
        .cloned()
}

fn set_next(ctx: &Context, args: &[String]) -> Result<(), String> {
    let entry = argument_value(args, "--entry")
        .or_else(|| argument_value(args, "--title"))
        .ok_or("set-next requiere --entry NOMBRE")?;
    if !command_exists("grub-reboot") {
        return Err("grub-reboot no está disponible".into());
    }
    println!("$ grub-reboot {}", crate::common::shell_display(&entry));
    if ctx.dry_run {
        println!("Simulación: se programaría esta entrada para el siguiente arranque.");
        return Ok(());
    }
    if !args.iter().any(|value| value == "--yes")
        && !ask(&format!(
            "¿Programar «{entry}» como siguiente entrada de GRUB?"
        ))
    {
        println!("Operación cancelada.");
        return Ok(());
    }
    let ok = run_with_sudo("grub-reboot", std::slice::from_ref(&entry), false)
        .map_err(|error| error.to_string())?;
    if ok {
        println!("Siguiente arranque programado: {entry}");
        Ok(())
    } else {
        Err("grub-reboot terminó con error".into())
    }
}

fn plan(ctx: &Context) -> Result<(), String> {
    println!("=== Plan de arranque Linux (solo lectura) ===");
    println!("No se modificarán GRUB, EFI, systemd-boot ni la NVRAM.");
    if ctx.dry_run {
        println!("Modo dry-run activo: solo se registrará el plan.");
    }
    if command_exists("grub-mkconfig") {
        println!(
            "Disponible: generar configuración GRUB en un fichero temporal para revisar el diff."
        );
    } else {
        println!(
            "No disponible: grub-mkconfig; no se ofrece instalación automática desde este plan."
        );
    }
    if command_exists("bootctl") {
        println!("Disponible: inspección y actualización controlada de systemd-boot.");
    }
    println!(
        "Para cambiar el siguiente arranque usa la opción programada y confirma el título exacto."
    );
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Arranque, GRUB, EFI y systemd-boot ===");
        println!("  1) Estado general\n  2) Entradas EFI / NVRAM\n  3) Entradas GRUB\n  4) systemd-boot\n  5) Secure Boot\n  6) Generar plan seguro\n  7) Programar siguiente entrada GRUB\n  8) Cancelar siguiente entrada GRUB\n q) Volver");
        print!("Elige una opción (Enter para volver): ");
        let _ = io::stdout().flush();
        let Some(input) = crate::menu_input("") else {
            return Ok(());
        };
        let result = match input.trim() {
            "1" => status(),
            "2" => efi_entries(),
            "3" => grub_entries_report(),
            "4" => bootctl_status(),
            "5" => secure_boot_status(),
            "6" => plan(ctx),
            "7" => select_next(ctx),
            "8" => clear_next(ctx, &["clear-next".into()]),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !input.trim().is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn clear_next(ctx: &Context, args: &[String]) -> Result<(), String> {
    if !command_exists("grub-editenv") {
        return Err("grub-editenv no está disponible".into());
    }
    if !args.iter().any(|value| value == "--yes") && !ask("¿Cancelar la entrada GRUB programada?")
    {
        return Ok(());
    }
    if let Some(plan) = &ctx.plan {
        plan.record(
            "boot-clear-next",
            Path::new("/boot/grub/grubenv"),
            if ctx.dry_run { "planned" } else { "executed" },
            true,
            "grub-editenv unset next_entry",
            "arranque",
        )
        .map_err(|error| error.to_string())?;
    }
    if ctx.dry_run {
        println!("Simulación: se cancelaría la entrada GRUB programada.");
        return Ok(());
    }
    let status = run_with_sudo(
        "grub-editenv",
        &[
            "/boot/grub/grubenv".into(),
            "unset".into(),
            "next_entry".into(),
        ],
        false,
    )
    .map_err(|error| error.to_string())?;
    if status {
        println!("Entrada GRUB programada cancelada.");
        Ok(())
    } else {
        Err("grub-editenv no pudo cancelar la entrada siguiente".into())
    }
}

fn select_next(ctx: &Context) -> Result<(), String> {
    let entries = grub_entries()?;
    if entries.is_empty() {
        return Err("no hay entradas GRUB seleccionables".into());
    }
    println!("=== Selecciona el siguiente arranque GRUB ===");
    for (index, entry) in entries.iter().enumerate() {
        println!("{:>3}) {}", index + 1, entry);
    }
    let answer = crate::menu_input("Número de entrada (Enter para cancelar): ").unwrap_or_default();
    if answer.trim().is_empty() {
        return Ok(());
    }
    let index = answer
        .trim()
        .parse::<usize>()
        .map_err(|_| "selección no válida".to_owned())?;
    let entry = entries
        .get(index.saturating_sub(1))
        .ok_or("selección fuera de rango")?;
    set_next(ctx, &["set-next".into(), "--entry".into(), entry.clone()])
}

fn probe(label: &str, program: &str, args: &[&str]) {
    if !command_exists(program) {
        println!("{label}: no disponible ({program})");
        return;
    }
    match Command::new(program).args(args).output() {
        Ok(output) => {
            let text = String::from_utf8_lossy(if output.stdout.is_empty() {
                &output.stderr
            } else {
                &output.stdout
            });
            let text = text.trim();
            println!(
                "{label}: {}",
                if text.is_empty() { "disponible" } else { text }
            );
        }
        Err(error) => println!("{label}: no se pudo consultar: {error}"),
    }
}

fn mount_state(path: &str) -> &'static str {
    if Path::new(path).is_dir() {
        "directorio presente; montaje no confirmado"
    } else {
        "no presente"
    }
}
