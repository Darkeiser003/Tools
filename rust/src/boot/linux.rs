use crate::common::{command_exists, Context};
use std::io::{self, Write};
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .find(|value| !value.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("menu");
    match action {
        "status" | "inspect" => status(),
        "plan" | "preview" => plan(ctx),
        "menu" => menu(ctx),
        _ => Err("boot admite status, plan o menu".into()),
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
    probe("systemd-boot", "bootctl", &["status"]);
    probe("entradas EFI", "efibootmgr", &["-v"]);
    probe("GRUB", "grub-install", &["--version"]);
    probe("generador GRUB", "grub-mkconfig", &["--version"]);
    probe("Secure Boot", "mokutil", &["--sb-state"]);
    Ok(())
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
    println!("Siguiente paso seguro: exportar una copia, revisar diff y aplicar con --apply y destino explícito.");
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Arranque, GRUB, EFI y systemd-boot ===");
        println!("  1) Estado del arranque\n  2) Generar plan seguro\n  q) Volver");
        print!("Elige una opción (Enter para volver): ");
        let _ = io::stdout().flush();
        let Some(input) = crate::menu_input("") else {
            return Ok(());
        };
        let result = match input.trim() {
            "1" => status(),
            "2" => plan(ctx),
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
    if std::path::Path::new(path).is_dir() {
        "directorio presente; montaje no confirmado"
    } else {
        "no presente"
    }
}

use std::path::Path;
