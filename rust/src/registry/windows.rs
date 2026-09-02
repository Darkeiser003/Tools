use crate::common::{command_exists, Context};
use std::path::PathBuf;
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .map(String::as_str)
        .find(|arg| !arg.starts_with('-'))
        .unwrap_or("status");
    match action {
        "status" => status(),
        "query" => query(args),
        "export" => export(ctx, args),
        "menu" => menu(ctx),
        _ => Err(format!("acción del registro Windows desconocida: {action}")),
    }
}

fn status() -> Result<(), String> {
    require_reg()?;
    println!("=== Registro Windows ===");
    println!("reg.exe: disponible");
    for hive in ["HKCU", "HKLM", "HKCR", "HKU"] {
        println!("{hive}: disponible para consulta");
    }
    println!("Las consultas son de solo lectura; exportar solo crea un archivo de respaldo.");
    Ok(())
}

fn query(args: &[String]) -> Result<(), String> {
    require_reg()?;
    let key = option(args, "--key").ok_or("registry query requiere --key HIVE\\RUTA")?;
    validate_key(&key)?;
    run_reg(&["query", &key])
}

fn export(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_reg()?;
    let key = option(args, "--key").ok_or("registry export requiere --key HIVE\\RUTA")?;
    validate_key(&key)?;
    let output = option(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| ctx.home.join("Documents/LTools/registry-backup.reg"));
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let output_text = output.to_string_lossy().to_string();
    run_reg(&["export", &key, &output_text, "/y"])?;
    println!("Respaldo exportado: {}", output.display());
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    let mut first = true;
    loop {
        if !first {
            crate::clear_screen();
        }
        first = false;
        println!("\n=== Registro Windows ===");
        println!("  1) Estado de hives");
        println!("  2) Consultar una clave");
        println!("  3) Exportar una clave");
        println!("  q) Volver");
        let answer = crate::menu_input("Elige una opción: ").unwrap_or_default();
        match answer.trim().to_lowercase().as_str() {
            "1" => status()?,
            "2" => {
                let key = crate::menu_input("Clave (Enter para volver): ").unwrap_or_default();
                if !key.is_empty() {
                    query(&["query".into(), "--key".into(), key])?;
                }
            }
            "3" => {
                let key = crate::menu_input("Clave (Enter para volver): ").unwrap_or_default();
                if !key.is_empty() {
                    let output = crate::menu_input("Archivo .reg (Enter para el predeterminado): ")
                        .unwrap_or_default();
                    let mut args = vec!["export".into(), "--key".into(), key];
                    if !output.is_empty() {
                        args.extend(["--out".into(), output]);
                    }
                    export(ctx, &args)?;
                }
            }
            "q" | "quit" | "salir" | "" => return Ok(()),
            _ => println!("Opción no válida."),
        }
    }
}

fn require_reg() -> Result<(), String> {
    if command_exists("reg.exe") || command_exists("reg") {
        Ok(())
    } else {
        Err("reg.exe no está disponible en este Windows".into())
    }
}

fn run_reg(args: &[&str]) -> Result<(), String> {
    let program = if command_exists("reg.exe") {
        "reg.exe"
    } else {
        "reg"
    };
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("no se pudo ejecutar {program}: {error}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].clone())
}

fn validate_key(key: &str) -> Result<(), String> {
    let uppercase = key.to_ascii_uppercase();
    let valid_hive = [
        "HKCU\\",
        "HKLM\\",
        "HKCR\\",
        "HKU\\",
        "HKEY_CURRENT_USER\\",
        "HKEY_LOCAL_MACHINE\\",
    ]
    .iter()
    .any(|prefix| uppercase.starts_with(prefix));
    if valid_hive
        && !key
            .chars()
            .any(|value| value == '"' || value == '\n' || value == '\r')
    {
        Ok(())
    } else {
        Err("clave de registro no válida; usa HKCU\\..., HKLM\\..., HKCR\\... o HKU\\...".into())
    }
}
