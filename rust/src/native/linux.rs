use crate::common::{ask, command_exists, run_command, Context};
use std::io::{self, Write};
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let area = first(args).unwrap_or("menu");
    match area {
        "network" | "net" => network(ctx, sub(args, area)),
        "hardware" | "hw" => hardware(ctx, sub(args, area)),
        "power" | "energy" => power(ctx, sub(args, area)),
        "security" | "firewall" => security(ctx, sub(args, area)),
        "menu" => menu(ctx),
        _ => Err("native admite network, hardware, power, security o menu".into()),
    }
}

fn first(args: &[String]) -> Option<&str> {
    args.iter()
        .map(String::as_str)
        .find(|v| !v.starts_with('-'))
}
fn sub<'a>(args: &'a [String], area: &str) -> &'a str {
    args.iter()
        .position(|v| v == area)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .filter(|v| !v.starts_with('-'))
        .unwrap_or("status")
}

fn network(ctx: &Context, action: &str) -> Result<(), String> {
    match action {
        "status" | "overview" => {
            println!("=== Red Linux ===");
            if offer(ctx, "ip") {
                best_effort("ip", &["-brief", "address"]);
            }
            println!("\n=== Rutas ===");
            if offer(ctx, "ip") {
                best_effort("ip", &["route"]);
            }
            println!("\n=== DNS ===");
            if offer(ctx, "resolvectl") {
                best_effort("resolvectl", &["status"]);
            } else {
                println!("resolvectl no está disponible.");
            }
            println!("\n=== Puertos escuchando ===");
            if offer(ctx, "ss") {
                best_effort("ss", &["-tuln"]);
            } else {
                println!("ss no está disponible.");
            }
            Ok(())
        }
        "flush-dns" | "dns-flush" => {
            if !offer(ctx, "resolvectl") {
                return Err("resolvectl no está disponible en este sistema".into());
            }
            if !ctx.dry_run && !ask("¿Vaciar la caché DNS de resolvectl?") {
                println!("Operación cancelada.");
                return Ok(());
            }
            let ok = run_command("resolvectl", &["flush-caches".into()], ctx.dry_run)
                .map_err(|e| e.to_string())?;
            if ok {
                println!("Caché DNS vaciada.");
                Ok(())
            } else {
                Err("resolvectl no pudo vaciar la caché DNS".into())
            }
        }
        _ => Err("network admite status u flush-dns".into()),
    }
}

fn hardware(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("hardware admite status".into());
    }
    println!("=== Hardware Linux ===");
    if offer(ctx, "uname") {
        best_effort("uname", &["-srmo"]);
    }
    println!("\n=== CPU ===");
    if offer(ctx, "lscpu") {
        best_effort("lscpu", &[]);
    }
    println!("\n=== Memoria ===");
    if offer(ctx, "free") {
        best_effort("free", &["-h"]);
    }
    println!("\n=== PCI ===");
    if offer(ctx, "lspci") {
        best_effort("lspci", &[]);
    }
    println!("\n=== USB ===");
    if offer(ctx, "lsusb") {
        best_effort("lsusb", &[]);
    }
    Ok(())
}

fn power(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("power admite status".into());
    }
    println!("=== Energía Linux ===");
    if offer(ctx, "powerprofilesctl") {
        if best_effort("powerprofilesctl", &["get"]) {
            best_effort("powerprofilesctl", &["list"]);
        }
    } else {
        println!("powerprofilesctl no está disponible.");
    }
    if offer(ctx, "upower") {
        best_effort("upower", &["-e"]);
    } else {
        println!("upower no está disponible; no se instala automáticamente.");
    }
    if offer(ctx, "systemd-inhibit") {
        println!("\n=== Bloqueos e inhibidores ===");
        best_effort("systemd-inhibit", &["--list"]);
    }
    Ok(())
}

fn security(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("security admite status".into());
    }
    println!("=== Firewall Linux ===");
    let mut found = false;
    if offer(ctx, "firewall-cmd") {
        found = true;
        println!("firewalld:");
        best_effort("firewall-cmd", &["--state"]);
    }
    if offer(ctx, "ufw") {
        found = true;
        println!("ufw:");
        best_effort("ufw", &["status"]);
    }
    if offer(ctx, "nft") {
        found = true;
        println!("nftables (consulta):");
        best_effort("nft", &["list", "ruleset"]);
    }
    if !found {
        println!("No se detectó firewalld, ufw ni nftables.");
    }
    println!("LTools solo consulta el firewall; los cambios se realizan en su gestor nativo.");
    Ok(())
}

fn offer(ctx: &Context, id: &str) -> bool {
    if command_exists(id) {
        return true;
    }
    match crate::common::ensure_tool(ctx, id) {
        Ok(available) => available,
        Err(error) => {
            println!("No se pudo preparar {id}: {error}");
            false
        }
    }
}

fn best_effort(program: &str, args: &[&str]) -> bool {
    if let Err(error) = capture(program, args) {
        println!("Aviso: no se pudo consultar {program}: {error}");
        false
    } else {
        true
    }
}
fn capture(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("no se pudo ejecutar {program}: {e}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if error.is_empty() {
            Ok(())
        } else {
            Err(error)
        }
    }
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Red, hardware, energía y seguridad Linux ===");
        println!("  1) Estado de red, rutas, DNS y puertos\n  2) Vaciar caché DNS\n  3) Hardware\n  4) Energía y perfiles\n  5) Estado del firewall\n  q) Volver");
        print!("Elige una opción (Enter para volver): ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Ok(());
        }
        let result = match input.trim() {
            "1" => network(ctx, "status"),
            "2" => network(ctx, "flush-dns"),
            "3" => hardware(ctx, "status"),
            "4" => power(ctx, "status"),
            "5" => security(ctx, "status"),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(e) = result {
            println!("Error: {e}");
        }
        if !input.trim().is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}
