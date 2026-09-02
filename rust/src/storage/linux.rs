use crate::common::{command_exists, command_output, ensure_tool, Context};
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = first_action(args).unwrap_or("status");
    match action {
        "status" | "disks" => status(ctx),
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

fn status(ctx: &Context) -> Result<(), String> {
    require("df")?;
    println!("=== Almacenamiento Linux ===");
    println!("Home: {}", ctx.home.display());
    run_capture("df", &["-h"])?;
    if command_exists("findmnt") {
        println!("\nMontajes:");
        run_capture("findmnt", &["-D"])?;
    } else {
        println!("\nMontajes: findmnt no está instalado; usa storage tools para ver alternativas.");
    }
    Ok(())
}

fn partitions(ctx: &Context) -> Result<(), String> {
    println!("=== Discos y particiones Linux ===");
    if command_exists("lsblk") {
        run_capture(
            "lsblk",
            &[
                "-e7",
                "-o",
                "NAME,PATH,SIZE,FSTYPE,LABEL,UUID,MOUNTPOINTS,TYPE",
            ],
        )?;
    } else if !ensure_tool(ctx, "lsblk")? {
        return Err("lsblk es necesario para listar discos y particiones".into());
    }
    if command_exists("parted") {
        println!("\nTabla de particiones detectada mediante parted:");
        run_capture("parted", &["-l"])?;
    } else {
        println!("parted no está instalado; se ha usado lsblk como alternativa segura.");
    }
    if command_exists("gparted") {
        println!(
            "gparted: disponible como herramienta gráfica; LTools no la abre automáticamente."
        );
    } else {
        println!("gparted: no instalado (opcional; no es necesario para este inventario).");
    }
    Ok(())
}

fn tools() -> Result<(), String> {
    println!("=== Herramientas de almacenamiento Linux ===");
    for name in ["lsblk", "findmnt", "df", "parted", "gparted", "fdisk"] {
        println!(
            "{name}: {}",
            if command_exists(name) {
                "disponible"
            } else {
                "no instalado"
            }
        );
    }
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    let mut first = true;
    loop {
        if !first {
            crate::clear_screen();
        }
        first = false;
        println!("\n=== Discos y almacenamiento Linux ===");
        println!("  1) Estado de discos y montajes");
        println!("  2) Particiones y tablas");
        println!("  3) Herramientas detectadas");
        println!("  q) Volver");
        let answer = crate::menu_input("Elige una opción: ").unwrap_or_default();
        match answer.trim().to_lowercase().as_str() {
            "1" => status(ctx)?,
            "2" => partitions(ctx)?,
            "3" => tools()?,
            "q" | "quit" | "salir" | "" => return Ok(()),
            _ => println!("Opción no válida."),
        }
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
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("no se pudo ejecutar {program}: {error}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if error.is_empty() {
            format!("{program} terminó con código {}", output.status)
        } else {
            error
        });
    }
    Ok(())
}

#[allow(dead_code)]
fn _command_path(name: &str) -> Option<String> {
    command_output("sh", &["-c", &format!("command -v -- {name}")])
}
