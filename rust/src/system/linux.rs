use crate::common::{ask, command_exists, ensure_tool, run_with_sudo, Context};
use crate::i18n;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

#[cfg(not(windows))]
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = args.iter().any(|a| a == "--user");
    let action = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("status");
    let required = match action {
        "status" | "failed" | "failed-services" | "services" | "user-services" | "export"
        | "report" | "dependencies" | "tree" | "service" => &["systemctl"][..],
        "processes" => &["ps"][..],
        "process" => {
            if args.iter().any(|arg| arg == "status") {
                &["ps"][..]
            } else {
                &["kill"][..]
            }
        }
        "journal" => &["journalctl"][..],
        _ => &[][..],
    };
    for tool in required {
        if !ensure_tool(ctx, tool)? {
            return Err(format!("no se puede ejecutar la acción sin {tool}"));
        }
    }
    match action {
        "status" | "health" => status(),
        "failed" | "failed-services" => failed_services(args),
        "services" | "user-services" => list_services(args, user || action == "user-services"),
        "processes" => processes(args),
        "journal" => journal(args),
        "service" => service(ctx, args, user),
        "process" => process(ctx, args),
        "dependencies" | "tree" => dependencies(args),
        "export" | "report" => export_report(ctx, args),
        "menu" => menu(ctx),
        _ => Err(format!("acción de sistema desconocida: {action}")),
    }
}

#[cfg(not(windows))]
fn status() -> Result<(), String> {
    require_systemctl()?;
    let state = systemctl_capture(false, &["is-system-running"])
        .1
        .trim()
        .to_string();
    let services = query_services(false)?;
    let failed: Vec<_> = services.iter().filter(|row| row.is_failed()).collect();
    let not_found = services.iter().filter(|row| row.is_not_found()).count();
    let normal_oneshot = services
        .iter()
        .filter(|row| row.is_normal_completion())
        .count();
    println!("=== Salud del sistema ===");
    println!(
        "Estado systemd: {}",
        if state.is_empty() {
            "desconocido"
        } else {
            &state
        }
    );
    if failed.is_empty() {
        println!("No hay servicios fallidos.");
    } else {
        println!("Servicios fallidos: {}", failed.len());
        print_service_rows(&failed);
    }
    println!(
        "Referencias not-found: {} (informativas, no necesariamente fallos)",
        not_found
    );
    println!(
        "oneshot inactive/dead o active/exited normales: {}",
        normal_oneshot
    );
    for row in services.iter().filter(|row| row.is_masked()) {
        println!(
            "Aviso: {} está masked; no se modifica automáticamente.",
            row.unit
        );
    }
    Ok(())
}

#[cfg(not(windows))]
fn list_services(args: &[String], forced_user: bool) -> Result<(), String> {
    require_systemctl()?;
    let requested_scope = option_value(args, "--scope").unwrap_or_else(|| {
        if forced_user || args.iter().any(|arg| arg == "--user") {
            "user".into()
        } else {
            "system".into()
        }
    });
    if !matches!(requested_scope.as_str(), "system" | "user" | "both") {
        return Err("--scope debe ser system, user o both".into());
    }
    let user = requested_scope == "user";
    let rows = if requested_scope == "both" && !forced_user {
        let mut rows = query_services(false)?;
        rows.extend(query_services(true)?);
        rows
    } else {
        query_services(user)?
    };
    let filter = option_value(args, "--filter").unwrap_or_else(|| "noteworthy".into());
    let category = option_value(args, "--category").unwrap_or_else(|| "all".into());
    let search = option_value(args, "--search")
        .unwrap_or_default()
        .to_lowercase();
    let limit = option_value(args, "--limit").and_then(|value| value.parse::<usize>().ok());
    let mut selected: Vec<_> = rows
        .iter()
        .filter(|row| row.matches_filter(&filter))
        .filter(|row| category == "all" || row.category == category)
        .filter(|row| search.is_empty() || row.search_text().contains(&search))
        .collect();
    if let Some(limit) = limit {
        selected.truncate(limit);
    }
    println!(
        "=== Servicios {} ===",
        match requested_scope.as_str() {
            "user" => "del usuario",
            "both" => "del sistema y del usuario",
            _ => "del sistema",
        }
    );
    println!(
        "Filtro: {filter}; categoría: {category}; resultados: {}",
        selected.len()
    );
    println!(
        "Columnas: unidad | estado | carga | subestado | tipo | arranque | categoría | descripción"
    );
    if selected.is_empty() {
        println!("No hay servicios que coincidan con estos filtros.");
    } else {
        print_grouped_service_rows(&selected);
    }
    Ok(())
}

#[cfg(not(windows))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceRow {
    unit: String,
    load: String,
    active: String,
    sub: String,
    description: String,
    kind: String,
    enabled: String,
    category: String,
    user: bool,
}

#[cfg(not(windows))]
impl ServiceRow {
    fn is_failed(&self) -> bool {
        self.active == "failed" || self.sub == "failed"
    }

    fn is_not_found(&self) -> bool {
        self.load == "not-found"
    }

    fn is_masked(&self) -> bool {
        self.enabled == "masked"
    }

    fn is_normal_completion(&self) -> bool {
        self.kind == "oneshot"
            && ((self.active == "inactive" && self.sub == "dead") || self.sub == "exited")
    }

    fn matches_filter(&self, filter: &str) -> bool {
        match filter.to_lowercase().as_str() {
            "all" => true,
            "active" => self.active == "active",
            "enabled" => matches!(self.enabled.as_str(), "enabled" | "static" | "indirect"),
            "failed" => self.is_failed(),
            "noteworthy" | "" => {
                !self.is_not_found()
                    && !(self.active == "inactive" && self.sub == "dead")
                    && !self.is_normal_completion()
            }
            _ => false,
        }
    }

    fn search_text(&self) -> String {
        format!(
            "{} {} {} {}",
            self.unit, self.category, self.description, self.kind
        )
        .to_lowercase()
    }
}

#[cfg(not(windows))]
fn query_services(user: bool) -> Result<Vec<ServiceRow>, String> {
    let (success, stdout, stderr) = systemctl_capture(
        user,
        &[
            "list-units",
            "--type=service",
            "--all",
            "--no-legend",
            "--no-pager",
            "--plain",
            "--full",
        ],
    );
    if !success && stdout.trim().is_empty() {
        println!(
            "Aviso: no se pudo consultar systemd: {}",
            systemctl_error(stderr)
        );
        return Ok(Vec::new());
    }
    Ok(stdout
        .lines()
        .filter_map(parse_service_line)
        .map(|mut row| {
            row.user = user;
            row.kind = service_property(user, &row.unit, "Type");
            row.enabled = service_enabled(user, &row.unit);
            row.category = service_category(&row.unit, &row.description);
            row
        })
        .collect())
}

#[cfg(not(windows))]
fn parse_service_line(line: &str) -> Option<ServiceRow> {
    let mut fields = line.split_whitespace();
    let unit = fields.next()?.to_string();
    let load = fields.next()?.to_string();
    let active = fields.next()?.to_string();
    let sub = fields.next()?.to_string();
    let description = fields.collect::<Vec<_>>().join(" ");
    Some(ServiceRow {
        unit,
        load,
        active,
        sub,
        description,
        kind: String::new(),
        enabled: String::new(),
        category: String::new(),
        user: false,
    })
}

#[cfg(not(windows))]
fn service_property(user: bool, unit: &str, property: &str) -> String {
    let args = [
        "show",
        unit,
        "--property",
        property,
        "--value",
        "--no-pager",
    ];
    let (_, stdout, _) = systemctl_capture(user, &args);
    stdout.trim().to_string()
}

#[cfg(not(windows))]
fn service_enabled(user: bool, unit: &str) -> String {
    let (_, stdout, stderr) = systemctl_capture(user, &["is-enabled", unit, "--no-pager"]);
    let value = stdout.trim();
    if value.is_empty() {
        stderr
            .lines()
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string()
    } else {
        value.to_string()
    }
}

#[cfg(not(windows))]
fn service_category(unit: &str, description: &str) -> String {
    let text = format!("{unit} {description}").to_lowercase();
    for (category, words) in [
        ("steam", &["steam", "proton"][..]),
        ("kde", &["kde", "plasma", "baloo"][..]),
        ("docker", &["docker", "containerd"][..]),
        ("vmware", &["vmware", "vmnet", "virtualbox"][..]),
        ("wine", &["wine", "lutris", "heroic", "umu"][..]),
        (
            "network",
            &["network", "iwd", "connman", "systemd-resolved"][..],
        ),
        ("audio", &["pipewire", "pulseaudio", "wireplumber"][..]),
    ] {
        if words.iter().any(|word| text.contains(word)) {
            return category.into();
        }
    }
    if unit.starts_with("user@") || unit.starts_with("session-") {
        "session".into()
    } else {
        "system".into()
    }
}

#[cfg(not(windows))]
fn print_service_rows(rows: &[&ServiceRow]) {
    for row in rows {
        println!(
            "{} | {} | {} | {} | {} | {} | {} | {}",
            row.unit,
            row.active,
            row.load,
            row.sub,
            if row.kind.is_empty() {
                "unknown"
            } else {
                &row.kind
            },
            if row.enabled.is_empty() {
                "unknown"
            } else {
                &row.enabled
            },
            row.category,
            row.description
        );
    }
}

#[cfg(not(windows))]
fn print_grouped_service_rows(rows: &[&ServiceRow]) {
    let categories = [
        "steam", "kde", "docker", "vmware", "wine", "network", "audio", "session", "system",
    ];
    for category in categories {
        let group: Vec<_> = rows
            .iter()
            .copied()
            .filter(|row| row.category == category)
            .collect();
        if group.is_empty() {
            continue;
        }
        println!("\n[{category}] {} servicio(s)", group.len());
        print_service_rows(&group);
    }
    let other: Vec<_> = rows
        .iter()
        .copied()
        .filter(|row| !categories.contains(&row.category.as_str()))
        .collect();
    if !other.is_empty() {
        println!("\n[other] {} servicio(s)", other.len());
        print_service_rows(&other);
    }
}

#[cfg(not(windows))]
fn failed_services(args: &[String]) -> Result<(), String> {
    require_systemctl()?;
    let user = args.iter().any(|arg| arg == "--user");
    let rows = query_services(user)?;
    let failed: Vec<_> = rows.iter().filter(|row| row.is_failed()).collect();
    println!("=== Servicios fallidos ===");
    if failed.is_empty() {
        println!("No hay servicios fallidos.");
        return Ok(());
    }
    print_service_rows(&failed);
    if args.iter().any(|arg| arg == "--journal") {
        for row in failed {
            println!("\n--- Journal de {} ---", row.unit);
            let (_, output, error) = journal_capture(user, &["--unit", &row.unit, "-n", "30"]);
            if output.is_empty() {
                println!("{}", error.trim());
            } else {
                print!("{output}");
            }
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn systemctl_capture(user: bool, args: &[&str]) -> (bool, String, String) {
    let mut command = Command::new("systemctl");
    if user {
        command.arg("--user");
    }
    let output = match command.args(args).output() {
        Ok(output) => output,
        Err(error) => return (false, String::new(), error.to_string()),
    };
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[cfg(not(windows))]
fn systemctl_error(stderr: String) -> String {
    if stderr.trim().is_empty() {
        "systemctl no pudo consultar los servicios".into()
    } else {
        stderr.trim().to_string()
    }
}

#[cfg(not(windows))]
fn option_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].clone())
}

#[cfg(not(windows))]
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
        } else {
            return Err(format!(
                "systemctl no pudo ejecutar {operation} sobre {unit}"
            ));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
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

#[cfg(not(windows))]
fn processes(args: &[String]) -> Result<(), String> {
    if !command_exists("ps") {
        return Err("ps no está instalado".into());
    }
    let sort = match option_value(args, "--sort").as_deref() {
        Some("cpu") | None => "-%cpu",
        Some("memory") | Some("mem") => "-%mem",
        Some(other) => return Err(format!("orden de procesos no válida: {other}")),
    };
    let limit = option_value(args, "--limit").and_then(|value| value.parse::<usize>().ok());
    let output = Command::new("ps")
        .args([
            "-eo",
            "pid=,ppid=,user=,%cpu=,%mem=,stat=,etime=,comm=,args=",
            &format!("--sort={sort}"),
        ])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    println!(
        "=== Procesos por {} ===",
        if sort == "-%cpu" { "CPU" } else { "memoria" }
    );
    println!("PID | PPID | USUARIO | CPU% | MEM% | STAT | TIEMPO | COMANDO | ARGUMENTOS");
    let mut count = 0;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(limit) = limit {
            if count >= limit {
                break;
            }
        }
        if line.trim().is_empty() {
            continue;
        }
        println!("{}", line.trim());
        count += 1;
    }
    Ok(())
}

#[cfg(not(windows))]
fn journal(args: &[String]) -> Result<(), String> {
    if !command_exists("journalctl") {
        return Err("journalctl no está instalado".into());
    }
    let level = option_value(args, "--level").unwrap_or_else(|| "warning".into());
    let priority = match level.to_lowercase().as_str() {
        "error" | "err" => "err..alert",
        "warning" | "warn" | "aviso" => "warning..alert",
        "info" => "info..alert",
        "all" => "debug..alert",
        other => return Err(format!("nivel de journal no válido: {other}")),
    };
    let mut owned = vec![
        "--no-pager".into(),
        "-b".into(),
        "-p".into(),
        priority.into(),
    ];
    if let Some(hours) = option_value(args, "--hours") {
        let hours = hours
            .parse::<u64>()
            .map_err(|_| "--hours debe ser un número".to_string())?;
        owned.extend(["--since".into(), format!("{hours} hours ago")]);
    } else {
        owned.extend(["--since".into(), "today".into()]);
    }
    if let Some(unit) = option_value(args, "--unit") {
        validate_unit(&unit)?;
        owned.extend(["--unit".into(), unit]);
    }
    if let Some(limit) = option_value(args, "--limit") {
        let limit = limit
            .parse::<u64>()
            .map_err(|_| "--limit debe ser un número".to_string())?;
        owned.extend(["-n".into(), limit.to_string()]);
    }
    let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
    let (_, output, error) = journal_capture(false, &refs);
    println!("=== Journal: nivel {level} ===");
    if output.trim().is_empty() {
        println!(
            "{}",
            if error.trim().is_empty() {
                "No hay entradas."
            } else {
                error.trim()
            }
        );
    } else {
        print!("{output}");
    }
    Ok(())
}

#[cfg(not(windows))]
fn journal_capture(user: bool, args: &[&str]) -> (bool, String, String) {
    let mut command = Command::new("journalctl");
    if user {
        command.arg("--user");
    }
    match command.args(args).output() {
        Ok(output) => (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ),
        Err(error) => (false, String::new(), error.to_string()),
    }
}
#[cfg(not(windows))]
fn require_systemctl() -> Result<(), String> {
    if command_exists("systemctl") {
        Ok(())
    } else {
        Err("systemctl no está instalado".into())
    }
}

#[cfg(not(windows))]
fn validate_unit(unit: &str) -> Result<(), String> {
    if unit.is_empty()
        || !unit
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".@_:-".contains(c))
    {
        Err("unidad no válida".into())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn dependencies(args: &[String]) -> Result<(), String> {
    require_systemctl()?;
    let unit = option_value(args, "--unit")
        .or_else(|| {
            args.windows(2)
                .find(|window| window[0] == "dependencies" || window[0] == "tree")
                .map(|window| window[1].clone())
        })
        .or_else(|| {
            args.iter()
                .skip(1)
                .find(|arg| !arg.starts_with('-'))
                .cloned()
        })
        .ok_or("dependencies requiere --unit UNIDAD")?;
    validate_unit(&unit)?;
    let user = args.iter().any(|arg| arg == "--user");
    let reverse = args.iter().any(|arg| arg == "--reverse");
    let mut command = Command::new("systemctl");
    if user {
        command.arg("--user");
    }
    command.args(["list-dependencies", &unit, "--no-pager", "--plain"]);
    if reverse {
        command.arg("--reverse");
    }
    let output = command.output().map_err(|error| error.to_string())?;
    println!(
        "=== Dependencias de {unit}{} ===",
        if reverse { " (inversas)" } else { "" }
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!(
            "{}",
            if stderr.trim().is_empty() {
                "No hay dependencias."
            } else {
                stderr.trim()
            }
        );
    } else {
        print!("{stdout}");
    }
    Ok(())
}

#[cfg(not(windows))]
fn export_report(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_systemctl()?;
    let requested_scope = option_value(args, "--scope").unwrap_or_else(|| {
        if args.iter().any(|arg| arg == "--user") {
            "user".into()
        } else {
            "system".into()
        }
    });
    if !matches!(requested_scope.as_str(), "system" | "user" | "both") {
        return Err("--scope debe ser system, user o both".into());
    }
    let user = requested_scope == "user";
    let format = option_value(args, "--format").unwrap_or_else(|| "tsv".into());
    let path = option_value(args, "--out")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            ctx.home.join(".local/state/ltools/reports").join(format!(
                "system-{}.{}",
                crate::common::timestamp(),
                format
            ))
        });
    let rows = if requested_scope == "both" {
        let mut rows = query_services(false)?;
        rows.extend(query_services(true)?);
        rows
    } else {
        query_services(user)?
    };
    let contents = match format.as_str() {
        "tsv" => {
            let mut text = String::from(
                "scope\tunit\tload\tactive\tsub\ttype\tenabled\tcategory\tdescription\n",
            );
            for row in &rows {
                text.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    if row.user { "user" } else { "system" },
                    crate::common::clean(&row.unit),
                    crate::common::clean(&row.load),
                    crate::common::clean(&row.active),
                    crate::common::clean(&row.sub),
                    crate::common::clean(&row.kind),
                    crate::common::clean(&row.enabled),
                    crate::common::clean(&row.category),
                    crate::common::clean(&row.description)
                ));
            }
            text
        }
        "json" => {
            let entries = rows
                .iter()
                .map(|row| {
                    format!(
                        "{{\"scope\":\"{}\",\"unit\":\"{}\",\"load\":\"{}\",\"active\":\"{}\",\"sub\":\"{}\",\"type\":\"{}\",\"enabled\":\"{}\",\"category\":\"{}\",\"description\":\"{}\"}}",
                        json_escape(if row.user { "user" } else { "system" }),
                        json_escape(&row.unit),
                        json_escape(&row.load),
                        json_escape(&row.active),
                        json_escape(&row.sub),
                        json_escape(&row.kind),
                        json_escape(&row.enabled),
                        json_escape(&row.category),
                        json_escape(&row.description)
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n");
            format!("[\n{entries}\n]\n")
        }
        _ => return Err("--format debe ser tsv o json".into()),
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&path, contents).map_err(|error| error.to_string())?;
    println!("Informe exportado: {}", path.display());
    Ok(())
}

#[cfg(not(windows))]
fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(not(windows))]
fn menu(ctx: &Context) -> Result<(), String> {
    let _ = ctx;
    let mut first = true;
    loop {
        if !first {
            crate::clear_screen();
        }
        first = false;
        println!("\n{}", i18n::text("menu.system.title"));
        println!("  1) {}", i18n::text("menu.system.status"));
        println!("  2) {}", i18n::text("menu.system.failed"));
        println!("  3) {}", i18n::text("menu.system.services"));
        println!("  4) {}", i18n::text("menu.system.user_services"));
        println!("  5) {}", i18n::text("menu.system.processes"));
        println!("  6) {}", i18n::text("menu.system.journal"));
        println!("  7) {}", i18n::text("menu.system.manage"));
        println!("  8) {}", i18n::text("menu.system.dependencies"));
        println!("  9) {}", i18n::text("menu.system.export"));
        println!("  q) {}", i18n::text("menu.back"));
        print!("{}", i18n::text("menu.prompt"));
        let _ = io::stdout().flush();
        let mut answer = String::new();
        match io::stdin().read_line(&mut answer) {
            Ok(0) => return Ok(()),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => return Ok(()),
            Err(error) => return Err(error.to_string()),
        }
        match answer.trim().to_lowercase().as_str() {
            "1" => status()?,
            "2" => failed_services(&["failed".into(), "--journal".into()])?,
            "3" => menu_service_list(false)?,
            "4" => menu_service_list(true)?,
            "5" => menu_processes()?,
            "6" => menu_journal()?,
            "7" => menu_manage_service(ctx)?,
            "8" => menu_dependencies()?,
            "9" => menu_export(ctx)?,
            "" | "q" | "b" | "back" | "volver" | "retour" | "zurück" | "voltar" | "indietro"
            | "torna" | "terug" | "wstecz" => return Ok(()),
            _ => println!("{}", i18n::text("menu.invalid")),
        }
    }
}

#[cfg(not(windows))]
fn menu_input(prompt: &str) -> Result<Option<String>, String> {
    print!("{prompt}");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut answer = String::new();
    match io::stdin().read_line(&mut answer) {
        Ok(0) => Ok(None),
        Ok(_) => Ok(Some(answer.trim().to_string())),
        Err(error) if error.kind() == io::ErrorKind::Interrupted => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(not(windows))]
fn menu_manage_service(ctx: &Context) -> Result<(), String> {
    println!("\n=== Gestionar un servicio ===");
    let scope = match menu_input("Ámbito [system/user] (Enter para volver): ")? {
        Some(value) if !value.is_empty() => value,
        _ => return Ok(()),
    };
    let user = match scope.to_lowercase().as_str() {
        "user" | "usuario" => true,
        "system" | "sistema" => false,
        _ => {
            println!("Ámbito no válido.");
            return Ok(());
        }
    };
    let unit = match menu_input("Unidad (Enter para volver): ")? {
        Some(value) if !value.is_empty() => value,
        _ => return Ok(()),
    };
    validate_unit(&unit)?;
    let action = match menu_input(
        "Acción [status/start/stop/restart/enable/disable/mask/unmask] (Enter para volver): ",
    )? {
        Some(value) if !value.is_empty() => value,
        _ => return Ok(()),
    };
    if unit == "power-profiles-daemon.service" && action == "mask" {
        println!(
            "Aviso: power-profiles-daemon está relacionado con perfiles de energía; \
no se modificará automáticamente. La acción solo continúa con tu confirmación."
        );
    }
    let mut args = vec!["service".into(), action, unit];
    if user {
        args.push("--user".into());
    }
    service(ctx, &args, user)
}

#[cfg(not(windows))]
fn menu_service_list(user: bool) -> Result<(), String> {
    let filter = menu_input("Filtro [noteworthy/active/enabled/failed/all] (Enter: noteworthy): ")?
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "noteworthy".into());
    let category =
        menu_input("Categoría [all/steam/kde/docker/vmware/wine/network/audio] (Enter: all): ")?
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "all".into());
    let search = menu_input("Buscar texto (Enter: sin búsqueda): ")?.unwrap_or_default();
    let mut args = vec![
        "services".into(),
        "--filter".into(),
        filter,
        "--category".into(),
        category,
    ];
    if !search.is_empty() {
        args.extend(["--search".into(), search]);
    }
    if user {
        args.push("--user".into());
    }
    list_services(&args, user)
}

#[cfg(not(windows))]
fn menu_processes() -> Result<(), String> {
    let sort = menu_input("Orden [cpu/memory] (Enter: cpu):")?
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "cpu".into());
    let limit = menu_input("Máximo de procesos (Enter: 30):")?
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "30".into());
    processes(&[
        "processes".into(),
        "--sort".into(),
        sort,
        "--limit".into(),
        limit,
    ])
}

#[cfg(not(windows))]
fn menu_journal() -> Result<(), String> {
    let level = menu_input("Nivel [error/warning/info/all] (Enter: warning):")?
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "warning".into());
    let hours = menu_input("Últimas horas (Enter: hoy):")?.unwrap_or_default();
    let unit = menu_input("Unidad (Enter: todas):")?.unwrap_or_default();
    let mut args = vec![
        "journal".into(),
        "--level".into(),
        level,
        "--limit".into(),
        "100".into(),
    ];
    if !hours.is_empty() {
        args.extend(["--hours".into(), hours]);
    } else {
        args.push("--today".into());
    }
    if !unit.is_empty() {
        args.extend(["--unit".into(), unit]);
    }
    journal(&args)
}

#[cfg(not(windows))]
fn menu_dependencies() -> Result<(), String> {
    let unit = match menu_input("Unidad (Enter para volver): ")? {
        Some(value) if !value.is_empty() => value,
        _ => return Ok(()),
    };
    validate_unit(&unit)?;
    let reverse = match menu_input("¿Mostrar dependencias inversas? [y/N] ")? {
        Some(value) => matches!(
            value.to_lowercase().as_str(),
            "y" | "yes" | "s" | "si" | "sí"
        ),
        None => return Ok(()),
    };
    let mut args = vec!["dependencies".into(), "--unit".into(), unit];
    if reverse {
        args.push("--reverse".into());
    }
    dependencies(&args)
}

#[cfg(not(windows))]
fn menu_export(ctx: &Context) -> Result<(), String> {
    let path = menu_input("Ruta del informe (Enter para la predeterminada): ")?.unwrap_or_default();
    let format = menu_input("Formato [tsv/json] (Enter para TSV): ")?.unwrap_or_default();
    let mut args = vec!["export".into()];
    if !path.is_empty() {
        args.extend(["--out".into(), path]);
    }
    if !format.is_empty() {
        args.extend(["--format".into(), format]);
    }
    export_report(ctx, &args)
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;

    #[test]
    fn el_parseo_conserva_descripciones_sin_truncarlas() {
        let mut row = parse_service_line(
            "demo.service loaded active running Una descripción muy larga con espacios",
        )
        .unwrap();
        row.kind = "simple".into();
        row.enabled = "enabled".into();
        row.category = service_category(&row.unit, &row.description);
        assert_eq!(row.unit, "demo.service");
        assert_eq!(row.description, "Una descripción muy larga con espacios");
        assert_eq!(row.category, "system");
    }

    #[test]
    fn not_found_y_dead_no_se_presentan_como_fallos_reales() {
        let mut stale =
            parse_service_line("old.service not-found inactive dead Referencia antigua").unwrap();
        stale.kind = "simple".into();
        stale.enabled = "not-found".into();
        let mut oneshot =
            parse_service_line("job.service loaded inactive dead Tarea terminada").unwrap();
        oneshot.kind = "oneshot".into();
        oneshot.enabled = "static".into();
        assert!(!stale.is_failed());
        assert!(stale.is_not_found());
        assert!(!stale.matches_filter("noteworthy"));
        assert!(oneshot.is_normal_completion());
        assert!(!oneshot.matches_filter("noteworthy"));
    }

    #[test]
    fn active_exited_se_identifica_como_finalizacion_normal() {
        let mut row =
            parse_service_line("task.service loaded active exited Tarea completada").unwrap();
        row.kind = "oneshot".into();
        row.enabled = "static".into();
        assert!(row.is_normal_completion());
        assert!(row.matches_filter("all"));
        assert!(!row.matches_filter("noteworthy"));
    }

    #[test]
    fn las_categorias_reconocen_los_grupos_de_juegos_y_virtualizacion() {
        assert_eq!(
            service_category("docker.service", "Docker Engine"),
            "docker"
        );
        assert_eq!(
            service_category("vmware.service", "VMware networking"),
            "vmware"
        );
        assert_eq!(service_category("pipewire.service", "PipeWire"), "audio");
        assert_eq!(service_category("steam-helper.service", "Steam"), "steam");
    }

    #[test]
    fn el_json_escapa_comillas_y_saltos() {
        assert_eq!(json_escape("a\"b\nc"), "a\\\"b\\nc");
    }
}
