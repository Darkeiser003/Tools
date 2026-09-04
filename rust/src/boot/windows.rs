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
    println!("=== Arranque Windows ===");
    println!(
        "Administrador de arranque: {}",
        if command_exists("bcdedit") {
            "bcdedit disponible"
        } else {
            "bcdedit no disponible"
        }
    );
    probe("BCD boot manager", "bcdedit", &["/enum", "{bootmgr}"]);
    probe("BCD cargadores", "bcdedit", &["/enum", "all"]);
    probe("volúmenes EFI", "mountvol", &["/list"]);
    probe("recuperación", "reagentc", &["/info"]);
    Ok(())
}

fn plan(ctx: &Context) -> Result<(), String> {
    println!("=== Plan de arranque Windows (solo lectura) ===");
    println!("No se modificarán BCD, particiones EFI ni WinRE.");
    if ctx.dry_run {
        println!("Modo dry-run activo: solo se registrará el plan.");
    }
    println!("Flujo protegido: exportar BCD, revisar cambios, aplicar con elevación y verificar la entrada resultante.");
    println!("Herramientas nativas previstas: bcdedit, bcdboot, mountvol y reagentc.");
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Arranque, BCD, EFI y recuperación Windows ===");
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
