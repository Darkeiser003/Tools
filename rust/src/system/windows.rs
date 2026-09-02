use crate::common::{ask, ensure_tool, Context};
#[cfg(not(windows))]
use crate::common::{command_exists, run_with_sudo};
#[cfg(not(windows))]
use crate::i18n;
#[cfg(not(windows))]
use std::fs;
#[cfg(windows)]
use std::io::Write;
#[cfg(not(windows))]
use std::io::{self, Write};
#[cfg(not(windows))]
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("status");
    let required = match action {
        "status" | "services" | "user-services" | "export" | "report" => {
            &["powershell", "Get-CimInstance"][..]
        }
        "processes" => &["powershell"][..],
        "journal" => &["wevtutil"][..],
        "service" => &["sc.exe"][..],
        "process" => {
            if args.iter().any(|arg| arg == "status") {
                &["tasklist"][..]
            } else {
                &["taskkill"][..]
            }
        }
        _ => &[][..],
    };
    for tool in required {
        if !ensure_tool(ctx, tool)? {
            return Err(format!("no se puede ejecutar la acción sin {tool}"));
        }
    }
    match action {
        "status" => windows_status(),
        "services" | "user-services" => windows_services(args),
        "processes" => windows_processes(args),
        "journal" => windows_events(args),
        "service" => windows_service(ctx, args),
        "process" => windows_process(ctx, args),
        "export" | "report" => windows_export(ctx, args),
        "menu" => windows_menu(ctx),
        _ => Err(format!("acción de sistema desconocida: {action}")),
    }
}

#[cfg(windows)]
fn windows_status() -> Result<(), String> {
    let rows = windows_service_rows()?;
    let running = rows.iter().filter(|row| row.state == "Running").count();
    let stopped = rows.iter().filter(|row| row.state == "Stopped").count();
    println!("=== Salud de servicios Windows ===");
    println!("Gestor de servicios: disponible");
    println!("Servicios detectados: {}", rows.len());
    println!("Activos: {running}; detenidos: {stopped}");
    println!(
        "Windows no expone un estado systemd 'failed'; los detalles están en services y journal."
    );
    Ok(())
}

#[cfg(windows)]
#[derive(Debug, Clone)]
struct WindowsServiceRow {
    name: String,
    state: String,
    start_mode: String,
    display: String,
    category: String,
}

#[cfg(windows)]
fn windows_service_rows() -> Result<Vec<WindowsServiceRow>, String> {
    if !crate::common::command_exists("powershell") && !crate::common::command_exists("pwsh") {
        return Err("PowerShell no está disponible para consultar los servicios".into());
    }
    let script = "Get-CimInstance Win32_Service | ForEach-Object { '{0}`t{1}`t{2}`t{3}' -f $_.Name,$_.State,$_.StartMode,$_.DisplayName }";
    let output = windows_powershell(script)?;
    let rows = output
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(4, '\t');
            let name = fields.next()?.trim().to_string();
            let state = fields.next()?.trim().to_string();
            let start_mode = fields.next()?.trim().to_string();
            let display = fields.next().unwrap_or_default().trim().to_string();
            if name.is_empty() {
                return None;
            }
            let category = windows_service_category(&name, &display);
            Some(WindowsServiceRow {
                name,
                state,
                start_mode,
                display,
                category,
            })
        })
        .collect();
    Ok(rows)
}

#[cfg(windows)]
fn windows_services(args: &[String]) -> Result<(), String> {
    let filter = windows_option(args, "--filter").unwrap_or_else(|| "all".into());
    if !matches!(
        filter.as_str(),
        "all" | "active" | "enabled" | "disabled" | "stopped"
    ) {
        return Err("--filter debe ser all, active, enabled, disabled o stopped".into());
    }
    let search = windows_option(args, "--search")
        .unwrap_or_default()
        .to_lowercase();
    let limit = windows_option(args, "--limit").and_then(|value| value.parse::<usize>().ok());
    let mut rows: Vec<_> = windows_service_rows()?
        .into_iter()
        .filter(|row| match filter.as_str() {
            "active" => row.state == "Running",
            "enabled" => matches!(row.start_mode.as_str(), "Auto" | "Manual"),
            "disabled" => row.start_mode == "Disabled",
            "stopped" => row.state == "Stopped",
            _ => true,
        })
        .filter(|row| {
            search.is_empty()
                || format!("{} {} {}", row.name, row.display, row.category)
                    .to_lowercase()
                    .contains(&search)
        })
        .collect();
    rows.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then(left.name.cmp(&right.name))
    });
    if let Some(limit) = limit {
        rows.truncate(limit);
    }
    println!("=== Servicios Windows ===");
    println!("Filtro: {filter}; resultados: {}", rows.len());
    println!("Columnas: nombre | estado | arranque | categoría | descripción");
    for row in rows {
        println!(
            "{} | {} | {} | {} | {}",
            row.name, row.state, row.start_mode, row.category, row.display
        );
    }
    Ok(())
}

#[cfg(windows)]
fn windows_service(ctx: &Context, args: &[String]) -> Result<(), String> {
    let position = args
        .iter()
        .position(|arg| arg == "service")
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
        return Err("nombre de servicio no válido".into());
    }
    if operation == "status" {
        let output = windows_capture("sc.exe", &["query", unit])?;
        print!("{output}");
        return Ok(());
    }
    let service_args: Vec<String> = match operation.as_str() {
        "start" | "stop" => vec![operation.clone(), unit.clone()],
        "restart" => Vec::new(),
        "enable" => vec![
            "config".into(),
            unit.clone(),
            "start=".into(),
            "auto".into(),
        ],
        "disable" => vec![
            "config".into(),
            unit.clone(),
            "start=".into(),
            "disabled".into(),
        ],
        "mask" | "unmask" | "daemon-reload" => {
            return Err("acción solo disponible en systemd Linux".into())
        }
        _ => return Err("acción de servicio no válida".into()),
    };
    if !ask(&format!("¿Ejecutar sc.exe {} {}?", operation, unit)) {
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: no se cambiaría el servicio Windows.");
        return Ok(());
    }
    let ok = if operation == "restart" {
        let stopped = crate::common::run_with_sudo("sc.exe", &["stop".into(), unit.clone()], false)
            .map_err(|e| e.to_string())?;
        stopped
            && crate::common::run_with_sudo("sc.exe", &["start".into(), unit.clone()], false)
                .map_err(|e| e.to_string())?
    } else {
        crate::common::run_with_sudo("sc.exe", &service_args, false).map_err(|e| e.to_string())?
    };
    if ok {
        if let Some(plan) = &ctx.plan {
            plan.record(
                "windows-service",
                std::path::Path::new(unit),
                "executed",
                false,
                operation,
                "sc.exe",
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("sc.exe devolvió un error".into())
    }
}

#[cfg(windows)]
fn windows_process(ctx: &Context, args: &[String]) -> Result<(), String> {
    let position = args
        .iter()
        .position(|arg| arg == "process")
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
        let output = windows_capture("tasklist", &["/FI", &format!("PID eq {pid}")])?;
        print!("{output}");
        return Ok(());
    }
    let force = match operation.as_str() {
        "stop" => false,
        "kill" => true,
        _ => return Err("acción de proceso no válida".into()),
    };
    if !ask(&format!("¿Finalizar el PID {pid} en Windows?")) {
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: no se finalizaría el proceso.");
        return Ok(());
    }
    let mut process_args = vec!["/PID".into(), pid.clone(), "/T".into()];
    if force {
        process_args.push("/F".into());
    }
    let ok = crate::common::run_with_sudo("taskkill", &process_args, false)
        .map_err(|e| e.to_string())?;
    if ok {
        if let Some(plan) = &ctx.plan {
            plan.record(
                "windows-process",
                std::path::Path::new(&format!("PID:{pid}")),
                "executed",
                false,
                operation,
                pid,
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("taskkill devolvió un error".into())
    }
}

#[cfg(windows)]
fn windows_processes(args: &[String]) -> Result<(), String> {
    let sort = windows_option(args, "--sort").unwrap_or_else(|| "memory".into());
    if !matches!(sort.as_str(), "memory" | "cpu") {
        return Err("--sort debe ser memory o cpu".into());
    }
    let limit = windows_option(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(20);
    let search = windows_option(args, "--search")
        .unwrap_or_default()
        .to_lowercase();
    let script = format!(
        "$items=Get-Process | Sort-Object {} -Descending | Select-Object -First {}; $items | ForEach-Object {{ '{{0}}`t{{1}}`t{{2}}`t{{3}}' -f $_.Id,$_.ProcessName,$_.WorkingSet64,$_.CPU }}",
        if sort == "cpu" { "CPU" } else { "WorkingSet64" },
        limit.max(1)
    );
    let mut rows = windows_powershell(&script)?
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(4, '\t');
            Some((
                fields.next()?.trim().to_string(),
                fields.next()?.trim().to_string(),
                fields.next()?.trim().to_string(),
                fields.next().unwrap_or_default().trim().to_string(),
            ))
        })
        .filter(|(_, name, _, _)| search.is_empty() || name.to_lowercase().contains(&search))
        .collect::<Vec<_>>();
    rows.truncate(limit);
    println!("=== Procesos Windows ===");
    println!("Orden: {sort}; resultados: {}", rows.len());
    println!("Columnas: PID | proceso | memoria_bytes | CPU_segundos");
    for (pid, name, memory, cpu) in rows {
        println!("{pid} | {name} | {memory} | {cpu}");
    }
    Ok(())
}

#[cfg(windows)]
fn windows_events(args: &[String]) -> Result<(), String> {
    if !crate::common::command_exists("wevtutil") {
        return Err("wevtutil no está disponible".into());
    }
    let channel = windows_option(args, "--channel").unwrap_or_else(|| "System".into());
    if !channel
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || "./-_".contains(value))
    {
        return Err("canal Windows no válido".into());
    }
    let limit = windows_option(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(100)
        .clamp(1, 10_000);
    let output = windows_capture(
        "wevtutil",
        &[
            "qe",
            &channel,
            &format!("/c:{limit}"),
            "/rd:true",
            "/f:text",
        ],
    )?;
    println!("=== Eventos Windows: {channel} ===");
    if output.trim().is_empty() {
        println!("No hay eventos.");
    } else {
        print!("{output}");
    }
    Ok(())
}

#[cfg(windows)]
fn windows_export(ctx: &Context, args: &[String]) -> Result<(), String> {
    let format = windows_option(args, "--format").unwrap_or_else(|| "tsv".into());
    if !matches!(format.as_str(), "tsv" | "json") {
        return Err("--format debe ser tsv o json".into());
    }
    let path = windows_option(args, "--out")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            ctx.home.join("Documents/LTools/reports").join(format!(
                "windows-services-{}.{}",
                crate::common::timestamp(),
                format
            ))
        });
    let rows = windows_service_rows()?;
    let contents = if format == "tsv" {
        let mut value = String::from("name\tstate\tstart_mode\tcategory\tdescription\n");
        for row in &rows {
            value.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\n",
                row.name, row.state, row.start_mode, row.category, row.display
            ));
        }
        value
    } else {
        let values = rows
            .iter()
            .map(|row| {
                format!(
                    "{{\"name\":\"{}\",\"state\":\"{}\",\"start_mode\":\"{}\",\"category\":\"{}\",\"description\":\"{}\"}}",
                    windows_json_escape(&row.name),
                    windows_json_escape(&row.state),
                    windows_json_escape(&row.start_mode),
                    windows_json_escape(&row.category),
                    windows_json_escape(&row.display)
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");
        format!("[\n{values}\n]\n")
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, contents).map_err(|e| e.to_string())?;
    println!("Informe exportado: {}", path.display());
    Ok(())
}

#[cfg(windows)]
fn windows_menu(ctx: &Context) -> Result<(), String> {
    let mut first = true;
    loop {
        if !first {
            crate::clear_screen();
        }
        first = false;
        println!("\n=== Servicios y procesos Windows ===");
        println!("  1) Salud de servicios");
        println!("  2) Servicios activos");
        println!("  3) Servicios detenidos");
        println!("  4) Procesos por memoria");
        println!("  5) Eventos del canal System");
        println!("  6) Exportar servicios a JSON");
        println!("  q) Volver");
        print!("Elige una opción: ");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() || input.trim().is_empty() {
            return Ok(());
        }
        let result = match input.trim().to_lowercase().as_str() {
            "1" => windows_status(),
            "2" => windows_services(&["services".into(), "--filter".into(), "active".into()]),
            "3" => windows_services(&["services".into(), "--filter".into(), "stopped".into()]),
            "4" => windows_processes(&["processes".into(), "--sort".into(), "memory".into()]),
            "5" => windows_events(&["journal".into(), "--channel".into(), "System".into()]),
            "6" => windows_export(ctx, &["export".into(), "--format".into(), "json".into()]),
            "q" | "quit" | "salir" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
    }
}

#[cfg(windows)]
fn windows_service_category(name: &str, display: &str) -> String {
    let text = format!("{name} {display}").to_lowercase();
    for (category, words) in [
        ("steam", &["steam", "proton"][..]),
        ("docker", &["docker", "containerd"][..]),
        ("vmware", &["vmware", "vmnet", "virtualbox"][..]),
        ("gaming", &["xbox", "gaming", "game"][..]),
        ("network", &["network", "dhcp", "bluetooth", "wireless"][..]),
        ("security", &["defender", "security", "firewall"][..]),
    ] {
        if words.iter().any(|word| text.contains(word)) {
            return category.into();
        }
    }
    "system".into()
}

#[cfg(windows)]
fn windows_powershell(script: &str) -> Result<String, String> {
    let program = if crate::common::command_exists("powershell") {
        "powershell"
    } else {
        "pwsh"
    };
    windows_capture(
        program,
        &["-NoProfile", "-NonInteractive", "-Command", script],
    )
}

#[cfg(windows)]
fn windows_capture(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("no se pudo ejecutar {program}: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if output.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("{program} terminó con código {}", output.status)
        } else {
            stderr
        })
    }
}

#[cfg(windows)]
fn windows_option(args: &[String], key: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].clone())
}

#[cfg(windows)]
fn windows_json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}
