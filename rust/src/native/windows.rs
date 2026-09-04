use crate::common::{ask, command_exists, run_with_sudo, Context};
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
            if !offer(ctx, "Get-NetIPConfiguration") {
                return Err("no se pudo preparar la consulta de red".into());
            }
            powershell("Get-NetIPConfiguration; Get-NetRoute -AddressFamily IPv4 | Format-Table -AutoSize; Get-DnsClientServerAddress | Format-Table -AutoSize; Get-NetTCPConnection -State Listen | Sort-Object LocalPort | Format-Table -AutoSize")
        }
        "flush-dns" | "dns-flush" => {
            if !offer(ctx, "ipconfig") {
                return Err("ipconfig no está disponible".into());
            }
            if !ctx.dry_run && !ask("¿Vaciar la caché DNS de Windows?") {
                println!("Operación cancelada.");
                return Ok(());
            }
            let ok = run_with_sudo("ipconfig", &["/flushdns".into()], ctx.dry_run)
                .map_err(|e| e.to_string())?;
            if ok {
                Ok(())
            } else {
                Err("ipconfig /flushdns devolvió un error".into())
            }
        }
        _ => Err("network admite status u flush-dns".into()),
    }
}
fn hardware(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("hardware admite status".into());
    }
    if !offer(ctx, "Get-CimInstance") {
        return Err("PowerShell/CIM no está disponible".into());
    }
    powershell("Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,OSArchitecture,LastBootUpTime | Format-List; Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors | Format-Table -AutoSize; Get-CimInstance Win32_ComputerSystem | Select-Object TotalPhysicalMemory | Format-List; Get-CimInstance Win32_VideoController | Select-Object Name,AdapterRAM,DriverVersion | Format-Table -AutoSize")
}
fn power(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" && action != "plans" {
        return Err("power admite status o plans".into());
    }
    if !offer(ctx, "powercfg") {
        return Err("powercfg no está disponible".into());
    }
    native(
        "powercfg",
        if action == "plans" {
            &["/list"]
        } else {
            &["/getactivescheme"]
        },
    )
}
fn security(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("security admite status".into());
    }
    if !offer(ctx, "Get-NetFirewallProfile") {
        return Err("Get-NetFirewallProfile no está disponible".into());
    }
    powershell("Get-NetFirewallProfile | Select-Object Name,Enabled,DefaultInboundAction,DefaultOutboundAction | Format-Table -AutoSize; Get-MpComputerStatus | Select-Object AMServiceEnabled,AntivirusEnabled,RealTimeProtectionEnabled | Format-List")
}
fn powershell(script: &str) -> Result<(), String> {
    let shell = if command_exists("powershell") {
        "powershell"
    } else {
        "pwsh"
    };
    native(
        shell,
        &["-NoProfile", "-NonInteractive", "-Command", script],
    )
}
fn native(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
fn offer(ctx: &Context, id: &str) -> bool {
    match crate::common::ensure_tool(ctx, id) {
        Ok(available) => available,
        Err(error) => {
            println!("No se pudo preparar {id}: {error}");
            false
        }
    }
}
fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Red, hardware, energía y seguridad Windows ===");
        println!("  1) Estado de red, rutas, DNS y puertos\n  2) Vaciar caché DNS\n  3) Hardware\n  4) Planes de energía\n  5) Firewall y Defender\n  q) Volver");
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
            "4" => power(ctx, "plans"),
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
