use crate::common::{command_exists, Context};
use std::fs;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .map(String::as_str)
        .find(|arg| !arg.starts_with('-'))
        .unwrap_or("status");
    match action {
        "status" => {
            println!("=== Registros y configuración Linux ===");
            println!("Linux no tiene un registro central equivalente al de Windows.");
            println!("Registros: journalctl (módulo system); configuración: /etc y ~/.config.");
            println!(
                "journalctl: {}",
                if command_exists("journalctl") {
                    "disponible"
                } else {
                    "no instalado"
                }
            );
            Ok(())
        }
        "paths" => {
            println!("=== Rutas de configuración Linux ===");
            for path in [
                ctx.home.join(".config"),
                ctx.home.join(".local/share"),
                "/etc".into(),
                "/var/log".into(),
            ] {
                let state = if fs::metadata(&path).is_ok() {
                    "existe"
                } else {
                    "no existe"
                };
                println!("{state}\t{}", path.display());
            }
            Ok(())
        }
        "menu" => menu(ctx),
        _ => Err(format!("acción de registros desconocida: {action}")),
    }
}

fn menu(ctx: &Context) -> Result<(), String> {
    let mut first = true;
    loop {
        if !first {
            crate::clear_screen();
        }
        first = false;
        println!("\n=== Registros y configuración Linux ===");
        println!("  1) Estado y alternativas nativas");
        println!("  2) Rutas de configuración");
        println!("  q) Volver");
        let answer = crate::menu_input("Elige una opción: ").unwrap_or_default();
        match answer.trim().to_lowercase().as_str() {
            "1" => run(ctx, &["status".into()])?,
            "2" => run(ctx, &["paths".into()])?,
            "q" | "quit" | "salir" | "" => return Ok(()),
            _ => println!("Opción no válida."),
        }
    }
}
