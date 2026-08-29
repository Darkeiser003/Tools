use crate::common::{ask, command_exists, run_with_sudo, Context};
use std::path::Path;
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = args.iter().any(|a| a == "--user");
    let action = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("status");
    match action {
        "status" => status(),
        "services" | "user-services" => list_services(user || action == "user-services"),
        "processes" => processes(),
        "journal" => journal(args),
        "service" => service(ctx, args, user),
        "process" => process(ctx, args),
        "menu" => menu(ctx),
        _ => Err(format!("acción de sistema desconocida: {action}")),
    }
}

fn status() -> Result<(), String> {
    require_systemctl()?;
    println!("=== Estado de systemd ===");
    let _ = Command::new("systemctl")
        .args(["is-system-running"])
        .status();
    let _ = Command::new("systemctl")
        .args(["--failed", "--no-pager"])
        .status();
    Ok(())
}

fn list_services(user: bool) -> Result<(), String> {
    require_systemctl()?;
    let mut args: Vec<String> = Vec::new();
    if user {
        args.push("--user".into());
    }
    args.extend([
        "list-units".into(),
        "--type=service".into(),
        "--all".into(),
        "--no-pager".into(),
    ]);
    let _ = Command::new("systemctl").args(args).status();
    Ok(())
}

fn service(ctx: &Context, args: &[String], user: bool) -> Result<(), String> {
    require_systemctl()?;
    let position = args
        .iter()
        .position(|a| a == "service")
        .ok_or("falta service")?;
    let operation = args
        .get(position + 1)
        .ok_or("service requiere acción y unidad")?;
    let unit = args
        .get(position + 2)
        .ok_or("service requiere acción y unidad")?;
    if !unit
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".@_:-".contains(c))
    {
        return Err("unidad no válida".into());
    }
    if operation == "status" {
        let mut command = Command::new("systemctl");
        if user {
            command.arg("--user");
        }
        let _ = command.args(["status", unit, "--no-pager"]).status();
        return Ok(());
    }
    if !matches!(
        operation.as_str(),
        "start" | "stop" | "restart" | "enable" | "disable" | "mask" | "unmask" | "daemon-reload"
    ) {
        return Err("acción de servicio no válida".into());
    }
    if !ask(&format!("¿Ejecutar systemctl {} {}?", operation, unit)) {
        return Ok(());
    }
    let scope = if user { "user" } else { "system" };
    if ctx.dry_run {
        println!("Simulación: no se cambiaría el servicio.");
        if let Some(p) = &ctx.plan {
            p.record(
                "service-change",
                Path::new(unit),
                "planned",
                false,
                operation,
                scope,
            )
            .map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    let mut command_args = Vec::new();
    if user {
        command_args.push("--user".into());
    }
    command_args.push(operation.clone());
    if operation != "daemon-reload" {
        command_args.push(unit.clone());
    }
    let ok = if user {
        Command::new("systemctl")
            .args(&command_args)
            .status()
            .map_err(|e| e.to_string())?
            .success()
    } else {
        run_with_sudo("systemctl", &command_args, false).map_err(|e| e.to_string())?
    };
    if ok {
        if let Some(p) = &ctx.plan {
            p.record(
                "service-change",
                Path::new(unit),
                "executed",
                false,
                operation,
                scope,
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn process(ctx: &Context, args: &[String]) -> Result<(), String> {
    let position = args
        .iter()
        .position(|a| a == "process")
        .ok_or("falta process")?;
    let operation = args
        .get(position + 1)
        .ok_or("process requiere acción y PID")?;
    let pid = args
        .get(position + 2)
        .ok_or("process requiere acción y PID")?;
    if !pid.chars().all(|c| c.is_ascii_digit()) || pid == "0" {
        return Err("PID no válido".into());
    }
    if operation == "status" {
        let _ = Command::new("ps")
            .args([
                "-o",
                "pid,ppid,user,%cpu,%mem,stat,etime,comm,args",
                "-p",
                pid,
            ])
            .status();
        return Ok(());
    }
    let signal = match operation.as_str() {
        "stop" => "TERM",
        "kill" => "KILL",
        _ => return Err("acción de proceso no válida".into()),
    };
    if !Path::new(&format!("/proc/{pid}")).exists() {
        return Err("no existe el PID".into());
    }
    if !ask(&format!("¿Enviar SIG{signal} al PID {pid}?")) {
        return Ok(());
    }
    let target = Path::new("/proc").join(pid);
    if ctx.dry_run {
        println!("Simulación: no se enviaría SIG{signal}.");
        if let Some(p) = &ctx.plan {
            p.record("process-signal", &target, "planned", false, signal, pid)
                .map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    let args = vec![format!("-{signal}"), pid.clone()];
    if run_with_sudo("kill", &args, false).map_err(|e| e.to_string())? {
        if let Some(p) = &ctx.plan {
            p.record("process-signal", &target, "executed", false, signal, pid)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn processes() -> Result<(), String> {
    let _ = Command::new("ps")
        .args([
            "-eo",
            "pid,user,%cpu,%mem,stat,etime,comm,args",
            "--sort=-%mem",
        ])
        .status();
    Ok(())
}
fn journal(args: &[String]) -> Result<(), String> {
    if !command_exists("journalctl") {
        return Err("journalctl no está instalado".into());
    }
    let mut command = Command::new("journalctl");
    command.args([
        "--since",
        "today",
        "--no-pager",
        "-p",
        "warning..alert",
        "-b",
    ]);
    if let Some(unit) = args.windows(2).find(|w| w[0] == "--unit") {
        command.args(["--unit", &unit[1]]);
    }
    let _ = command.status();
    Ok(())
}
fn require_systemctl() -> Result<(), String> {
    if command_exists("systemctl") {
        Ok(())
    } else {
        Err("systemctl no está instalado".into())
    }
}
fn menu(ctx: &Context) -> Result<(), String> {
    println!("Usa status, services, processes, journal, service o process con argumentos.");
    let _ = ctx;
    Ok(())
}
