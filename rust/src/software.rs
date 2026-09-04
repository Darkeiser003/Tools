//! Búsqueda e instalación explícita usando únicamente gestores presentes.
//!
//! Este módulo no es una tienda: descubre los gestores nativos disponibles,
//! consulta sus índices sin actualizarlos y deja que el usuario elija un
//! candidato concreto. Los comandos se ejecutan como argumentos, nunca a
//! través de una shell.

use crate::common::{self, Context};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct Manager {
    id: &'static str,
    query_command: &'static str,
    install_command: &'static str,
    privileged: bool,
}

#[derive(Debug, Clone)]
struct Candidate {
    manager: String,
    package_id: String,
    version: String,
    source: String,
    description: String,
}

#[derive(Debug, Default)]
struct SearchReport {
    candidates: Vec<Candidate>,
    errors: Vec<String>,
}

#[derive(Debug)]
struct InstallRequest {
    query: String,
    manager_filter: Option<String>,
    candidate_number: Option<usize>,
    yes: bool,
    format: String,
    limit: usize,
}

#[derive(Debug)]
struct ToolOutput {
    status: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

#[cfg(not(windows))]
const MANAGERS: &[Manager] = &[
    Manager {
        id: "pacman",
        query_command: "pacman",
        install_command: "pacman",
        privileged: true,
    },
    Manager {
        id: "paru",
        query_command: "paru",
        install_command: "paru",
        privileged: false,
    },
    Manager {
        id: "yay",
        query_command: "yay",
        install_command: "yay",
        privileged: false,
    },
    Manager {
        id: "pikaur",
        query_command: "pikaur",
        install_command: "pikaur",
        privileged: false,
    },
    Manager {
        id: "apt",
        query_command: "apt-cache",
        install_command: "apt",
        privileged: true,
    },
    Manager {
        id: "dnf",
        query_command: "dnf",
        install_command: "dnf",
        privileged: true,
    },
    Manager {
        id: "yum",
        query_command: "yum",
        install_command: "yum",
        privileged: true,
    },
    Manager {
        id: "zypper",
        query_command: "zypper",
        install_command: "zypper",
        privileged: true,
    },
    Manager {
        id: "apk",
        query_command: "apk",
        install_command: "apk",
        privileged: true,
    },
    Manager {
        id: "xbps",
        query_command: "xbps-query",
        install_command: "xbps-install",
        privileged: true,
    },
    Manager {
        id: "pkg",
        query_command: "pkg",
        install_command: "pkg",
        privileged: true,
    },
    Manager {
        id: "flatpak",
        query_command: "flatpak",
        install_command: "flatpak",
        privileged: false,
    },
    Manager {
        id: "snap",
        query_command: "snap",
        install_command: "snap",
        privileged: true,
    },
    Manager {
        id: "brew",
        query_command: "brew",
        install_command: "brew",
        privileged: false,
    },
    Manager {
        id: "nix",
        query_command: "nix-env",
        install_command: "nix-env",
        privileged: false,
    },
    Manager {
        id: "guix",
        query_command: "guix",
        install_command: "guix",
        privileged: false,
    },
    Manager {
        id: "eopkg",
        query_command: "eopkg",
        install_command: "eopkg",
        privileged: true,
    },
];

#[cfg(windows)]
const MANAGERS: &[Manager] = &[
    Manager {
        id: "winget",
        query_command: "winget",
        install_command: "winget",
        privileged: false,
    },
    Manager {
        id: "choco",
        query_command: "choco",
        install_command: "choco",
        privileged: false,
    },
    Manager {
        id: "scoop",
        query_command: "scoop",
        install_command: "scoop",
        privileged: false,
    },
];

pub fn help() -> &'static str {
    #[cfg(windows)]
    {
        "software search NAME [--manager M] [--format text|json] [--limit N] | software install NAME [--candidate N] [--yes] [--limit N] [--dry-run]"
    }
    #[cfg(not(windows))]
    {
        "software search NAME [--manager M] [--format text|json] [--limit N] | software install NAME [--candidate N] [--yes] [--limit N] [--dry-run]"
    }
}

pub fn menu_label() -> &'static str {
    crate::i18n::tools_text("menu")
}

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let operation = args.first().map(String::as_str).unwrap_or("search");
    match operation {
        "search" | "find" => search_command(args.get(1..).unwrap_or_default()),
        "install" | "add" => install_command(ctx, args.get(1..).unwrap_or_default()),
        "stores" | "managers" => list_managers(),
        _ => Err(format!("operación software desconocida: {operation}")),
    }
}

fn list_managers() -> Result<(), String> {
    println!("{}", crate::i18n::tools_text("stores_title"));
    for manager in MANAGERS {
        println!(
            "  {:<10} {}",
            manager.id,
            if common::command_exists(manager.query_command) {
                "disponible"
            } else {
                "no instalado"
            }
        );
    }
    Ok(())
}

fn search_command(args: &[String]) -> Result<(), String> {
    let (query, manager_filter, format, limit) = parse_search_args(args)?;
    let report = search(&query, manager_filter.as_deref(), limit);
    if format == "json" {
        println!("{}", report_json(&query, &report));
    } else {
        print_report(&query, &report);
    }
    if report.candidates.is_empty() {
        return Err(crate::i18n::tools_text("no_results").to_string());
    }
    Ok(())
}

fn install_command(ctx: &Context, args: &[String]) -> Result<(), String> {
    let request = parse_install_args(args)?;
    let report = search(
        &request.query,
        request.manager_filter.as_deref(),
        request.limit,
    );
    if request.format == "json" {
        println!("{}", report_json(&request.query, &report));
    } else {
        print_report(&request.query, &report);
    }
    if report.candidates.is_empty() {
        return Err(crate::i18n::tools_text("no_results").to_string());
    }
    let candidate = if let Some(number) = request.candidate_number {
        report
            .candidates
            .get(number.saturating_sub(1))
            .cloned()
            .ok_or_else(|| format!("candidato fuera de rango: {number}"))?
    } else if report.candidates.len() == 1 {
        report.candidates[0].clone()
    } else if request.yes {
        return Err("hay varios candidatos; usa --candidate N para seleccionar uno".into());
    } else {
        let answer = prompt(crate::i18n::tools_text("select_candidate"))
            .map_err(|error| error.to_string())?;
        let number = answer
            .trim()
            .parse::<usize>()
            .map_err(|_| "debes indicar el número del candidato".to_string())?;
        report
            .candidates
            .get(number.saturating_sub(1))
            .cloned()
            .ok_or_else(|| format!("candidato fuera de rango: {number}"))?
    };
    let manager = MANAGERS
        .iter()
        .find(|item| item.id == candidate.manager)
        .ok_or_else(|| "gestor no disponible".to_string())?;
    let command_args = install_args(manager, &candidate.package_id);
    println!(
        "{} {} {}",
        crate::i18n::tools_text("selected"),
        candidate.manager,
        candidate.package_id
    );
    let display_program = if manager.privileged && !cfg!(windows) {
        format!("sudo {}", manager.install_command)
    } else {
        manager.install_command.to_string()
    };
    println!(
        "  $ {} {}",
        display_program,
        command_args
            .iter()
            .map(|arg| common::shell_display(arg))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if ctx.dry_run {
        record(ctx, &candidate, "planned", &command_args);
        println!("{}", crate::i18n::tools_text("dry_run"));
        return Ok(());
    }
    if !request.yes && !common::ask(crate::i18n::tools_text("confirm_install")) {
        record(ctx, &candidate, "cancelled", &command_args);
        return Err(crate::i18n::tools_text("cancelled").to_string());
    }
    let success = if manager.privileged && !cfg!(windows) {
        common::run_with_sudo(manager.install_command, &command_args, false)
            .map_err(|e| e.to_string())?
    } else {
        common::run_command(manager.install_command, &command_args, false)
            .map_err(|e| e.to_string())?
    };
    record(
        ctx,
        &candidate,
        if success { "executed" } else { "failed" },
        &command_args,
    );
    if success {
        Ok(())
    } else {
        Err(format!("falló la instalación con {}", candidate.manager))
    }
}

fn record(ctx: &Context, candidate: &Candidate, status: &str, args: &[String]) {
    if let Some(plan) = &ctx.plan {
        let _ = plan.record(
            "package-install",
            std::path::Path::new(&candidate.package_id),
            status,
            false,
            &candidate.manager,
            &args.join(" "),
        );
    }
}

fn search(query: &str, manager_filter: Option<&str>, limit: usize) -> SearchReport {
    let mut report = SearchReport::default();
    for manager in MANAGERS {
        if manager_filter.is_some_and(|filter| filter != manager.id)
            || !common::command_exists(manager.query_command)
        {
            continue;
        }
        let args = search_args(manager.id, query);
        let result = run_capture(manager.query_command, &args, Duration::from_secs(20));
        if result.timed_out {
            report
                .errors
                .push(format!("{}: tiempo de espera agotado", manager.id));
            continue;
        }
        if result.status != Some(0) {
            let detail = result.stderr.lines().next().unwrap_or("comando rechazado");
            report
                .errors
                .push(format!("{}: {}", manager.id, detail.trim()));
            continue;
        }
        report
            .candidates
            .extend(parse_candidates(manager.id, &result.stdout));
    }
    report.candidates.sort_by(|left, right| {
        left.manager
            .cmp(&right.manager)
            .then(left.package_id.cmp(&right.package_id))
            .then(left.version.cmp(&right.version))
    });
    report.candidates.dedup_by(|left, right| {
        left.manager == right.manager
            && left.package_id == right.package_id
            && left.version == right.version
    });
    report.candidates.truncate(limit);
    report
}

fn search_args(manager: &str, query: &str) -> Vec<String> {
    match manager {
        "pacman" | "paru" | "yay" | "pikaur" => vec!["-Ss".into(), query.into()],
        "apt" => vec!["search".into(), query.into()],
        "dnf" | "yum" => vec!["search".into(), query.into()],
        "zypper" => vec!["search".into(), query.into()],
        "apk" => vec!["search".into(), query.into()],
        "xbps" => vec!["-Rs".into(), query.into()],
        "pkg" => vec!["search".into(), query.into()],
        "flatpak" => vec!["search".into(), query.into()],
        "snap" => vec!["find".into(), query.into()],
        "brew" => vec!["search".into(), query.into()],
        "nix" => vec!["-qaP".into(), query.into()],
        "guix" => vec!["search".into(), query.into()],
        "eopkg" => vec!["search".into(), query.into()],
        "winget" => vec![
            "search".into(),
            "--name".into(),
            query.into(),
            "--accept-source-agreements".into(),
        ],
        "choco" => vec!["search".into(), query.into(), "--limit-output".into()],
        "scoop" => vec!["search".into(), query.into()],
        _ => vec![query.into()],
    }
}

fn install_args(manager: &Manager, package_id: &str) -> Vec<String> {
    match manager.id {
        "pacman" => vec![
            "-S".into(),
            "--needed".into(),
            "--".into(),
            package_id.into(),
        ],
        "paru" | "yay" | "pikaur" => vec!["-S".into(), "--needed".into(), package_id.into()],
        "apt" => vec!["install".into(), package_id.into()],
        "dnf" | "yum" | "zypper" => vec!["install".into(), package_id.into()],
        "apk" => vec!["add".into(), package_id.into()],
        "xbps" => vec!["-S".into(), package_id.into()],
        "pkg" => vec!["install".into(), package_id.into()],
        "flatpak" => vec![
            "install".into(),
            "--user".into(),
            "flathub".into(),
            package_id.into(),
        ],
        "snap" => vec!["install".into(), package_id.into()],
        "brew" => vec!["install".into(), package_id.into()],
        "nix" => vec!["-iA".into(), package_id.into()],
        "guix" => vec!["install".into(), package_id.into()],
        "eopkg" => vec!["install".into(), package_id.into()],
        "winget" => vec![
            "install".into(),
            "--id".into(),
            package_id.into(),
            "--exact".into(),
            "--accept-package-agreements".into(),
            "--accept-source-agreements".into(),
        ],
        "choco" => vec![
            "install".into(),
            package_id.into(),
            "-y".into(),
            "--no-progress".into(),
        ],
        "scoop" => vec!["install".into(), package_id.into()],
        _ => vec![package_id.into()],
    }
}

fn parse_candidates(manager: &str, output: &str) -> Vec<Candidate> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty()
                || line.starts_with("---")
                || line.starts_with("Name ")
                || line.starts_with("Name\t")
            {
                return None;
            }
            let (package_id, version, description): (String, String, String) = match manager {
                "pacman" | "paru" | "yay" | "pikaur" => {
                    let mut parts = line.splitn(3, char::is_whitespace);
                    let id = parts.next()?.trim().to_string();
                    let version = parts.next().unwrap_or("unknown").trim().to_string();
                    (id, version, parts.next().unwrap_or("").trim().to_string())
                }
                "apt" | "dnf" | "yum" | "zypper" | "apk" | "xbps" | "pkg" | "eopkg" => {
                    let id = line.split_whitespace().next()?.to_string();
                    let description = line
                        .strip_prefix(&id)
                        .unwrap_or("")
                        .trim_start_matches(" - ")
                        .trim()
                        .to_string();
                    (id, "unknown".into(), description)
                }
                "flatpak" => {
                    let mut parts = line.split_whitespace();
                    let id = parts.next()?.to_string();
                    let version = parts.next().unwrap_or("unknown").to_string();
                    let description = parts.collect::<Vec<_>>().join(" ");
                    (id, version, description)
                }
                "nix" => {
                    let mut parts = line.split_whitespace();
                    (
                        parts.next()?.to_string(),
                        parts.next().unwrap_or("unknown").to_string(),
                        String::new(),
                    )
                }
                "brew" | "snap" | "guix" | "scoop" => (
                    line.split_whitespace().next()?.to_string(),
                    "unknown".into(),
                    line.to_string(),
                ),
                "winget" => {
                    let columns: Vec<_> = line.split_whitespace().collect();
                    if columns.len() < 2 || line.contains("Successfully installed") {
                        return None;
                    }
                    (
                        columns.get(1)?.to_string(),
                        columns.get(2).copied().unwrap_or("unknown").to_string(),
                        line.to_string(),
                    )
                }
                "choco" => (
                    line.split('|').next()?.trim().to_string(),
                    "unknown".into(),
                    line.to_string(),
                ),
                _ => return None,
            };
            if package_id == "No" || package_id == "The" || package_id.len() > 240 {
                return None;
            }
            Some(Candidate {
                manager: manager.to_string(),
                package_id,
                version,
                source: manager.to_string(),
                description,
            })
        })
        .collect()
}

fn print_report(query: &str, report: &SearchReport) {
    println!("{}: {query}", crate::i18n::tools_text("search_title"));
    if report.candidates.is_empty() {
        println!("{}", crate::i18n::tools_text("no_candidates"));
    } else {
        for (index, candidate) in report.candidates.iter().enumerate() {
            println!(
                " {:>3}) {:<10} {:<42} {:<18} {}",
                index + 1,
                candidate.manager,
                candidate.package_id,
                candidate.version,
                candidate.description
            );
        }
    }
    for error in &report.errors {
        eprintln!("  ! {error}");
    }
}

fn report_json(query: &str, report: &SearchReport) -> String {
    let candidates = report.candidates.iter().enumerate().map(|(index, candidate)| format!(
        "{{\"index\":{},\"manager\":\"{}\",\"id\":\"{}\",\"version\":\"{}\",\"source\":\"{}\",\"description\":\"{}\"}}",
        index + 1, json_escape(&candidate.manager), json_escape(&candidate.package_id), json_escape(&candidate.version), json_escape(&candidate.source), json_escape(&candidate.description)
    )).collect::<Vec<_>>().join(",");
    let errors = report
        .errors
        .iter()
        .map(|error| format!("\"{}\"", json_escape(error)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"schema\":\"ltools-package-search-v1\",\"query\":\"{}\",\"platform\":\"{}\",\"candidates\":[{}],\"errors\":[{}]}}", json_escape(query), if cfg!(windows) { "windows" } else { "linux" }, candidates, errors)
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| match character {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            c if c.is_control() => "?".chars().collect(),
            c => vec![c],
        })
        .collect()
}

fn parse_search_args(args: &[String]) -> Result<(String, Option<String>, String, usize), String> {
    let mut query = None;
    let mut manager = None;
    let mut format = "text".to_string();
    let mut limit = 100_usize;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--manager" => {
                manager = Some(normalize_manager(
                    args.get(index + 1).ok_or("--manager requiere un valor")?,
                )?);
                index += 2;
            }
            option if option.starts_with("--manager=") => {
                manager = Some(normalize_manager(option.trim_start_matches("--manager="))?);
                index += 1;
            }
            "--format" => {
                format =
                    normalize_format(args.get(index + 1).ok_or("--format requiere un valor")?)?;
                index += 2;
            }
            option if option.starts_with("--format=") => {
                format = normalize_format(option.trim_start_matches("--format="))?;
                index += 1;
            }
            "--limit" => {
                limit = parse_limit(args.get(index + 1).ok_or("--limit requiere un número")?)?;
                index += 2;
            }
            option if option.starts_with("--limit=") => {
                limit = parse_limit(option.trim_start_matches("--limit="))?;
                index += 1;
            }
            value if !value.starts_with('-') && query.is_none() => {
                query = Some(value.to_string());
                index += 1;
            }
            value => return Err(format!("opción software desconocida: {value}")),
        }
    }
    let query = query
        .or_else(|| prompt(crate::i18n::tools_text("query_prompt")).ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or("falta el nombre del paquete")?;
    validate_query(&query)?;
    Ok((query, manager, format, limit))
}

fn parse_limit(value: &str) -> Result<usize, String> {
    let limit = value
        .parse::<usize>()
        .map_err(|_| "--limit no es válido".to_string())?;
    if !(1..=1000).contains(&limit) {
        return Err("--limit debe estar entre 1 y 1000".into());
    }
    Ok(limit)
}

fn parse_install_args(args: &[String]) -> Result<InstallRequest, String> {
    let mut search_args = Vec::new();
    let mut candidate = None;
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--candidate" => {
                candidate = Some(
                    args.get(index + 1)
                        .ok_or("--candidate requiere un número")?
                        .parse()
                        .map_err(|_| "--candidate no es válido")?,
                );
                index += 2;
            }
            option if option.starts_with("--candidate=") => {
                candidate = Some(
                    option
                        .trim_start_matches("--candidate=")
                        .parse()
                        .map_err(|_| "--candidate no es válido")?,
                );
                index += 1;
            }
            "--yes" | "-y" => {
                yes = true;
                index += 1;
            }
            value => {
                search_args.push(value.to_string());
                index += 1;
            }
        }
    }
    let (query, manager, format, limit) = parse_search_args(&search_args)?;
    Ok(InstallRequest {
        query,
        manager_filter: manager,
        candidate_number: candidate,
        yes,
        format,
        limit,
    })
}

fn normalize_manager(value: &str) -> Result<String, String> {
    let lowered = value.to_lowercase();
    let normalized = match lowered.as_str() {
        "apt-get" => "apt",
        "pacman/aur" => "pacman",
        "flat-pack" => "flatpak",
        other => other,
    };
    if MANAGERS.iter().any(|manager| manager.id == normalized) {
        Ok(normalized.to_string())
    } else {
        Err(format!("gestor no soportado en esta plataforma: {value}"))
    }
}

fn normalize_format(value: &str) -> Result<String, String> {
    match value.to_lowercase().as_str() {
        "text" | "json" => Ok(value.to_lowercase()),
        _ => Err("--format debe ser text o json".into()),
    }
}

fn validate_query(query: &str) -> Result<(), String> {
    if query.len() > 200 || query.chars().any(char::is_control) {
        return Err("el nombre de paquete contiene caracteres no permitidos".into());
    }
    Ok(())
}

fn prompt(message: &str) -> io::Result<String> {
    print!("{message}");
    let _ = io::stdout().flush();
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(value.trim().to_string())
}

fn run_capture(program: &str, args: &[String], timeout: Duration) -> ToolOutput {
    let child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(error) => {
            return ToolOutput {
                status: None,
                stdout: String::new(),
                stderr: error.to_string(),
                timed_out: false,
            }
        }
    };
    let stdout = child.stdout.take().map(|mut stream| {
        thread::spawn(move || {
            let mut data = Vec::new();
            let _ = stream.read_to_end(&mut data);
            String::from_utf8_lossy(&data).into_owned()
        })
    });
    let stderr = child.stderr.take().map(|mut stream| {
        thread::spawn(move || {
            let mut data = Vec::new();
            let _ = stream.read_to_end(&mut data);
            String::from_utf8_lossy(&data).into_owned()
        })
    });
    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.code(),
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                timed_out = true;
                break None;
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(_) => break None,
        }
    };
    let stdout = stdout
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    let stderr = stderr
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    ToolOutput {
        status,
        stdout,
        stderr,
        timed_out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_args_are_native_and_not_shell_fragments() {
        assert_eq!(search_args("pacman", "foo bar"), vec!["-Ss", "foo bar"]);
        assert_eq!(
            search_args("winget", "foo"),
            vec!["search", "--name", "foo", "--accept-source-agreements"]
        );
    }

    #[test]
    fn install_args_keep_package_as_one_argument() {
        let manager = Manager {
            id: "pacman",
            query_command: "pacman",
            install_command: "pacman",
            privileged: true,
        };
        assert_eq!(
            install_args(&manager, "foo bar"),
            vec!["-S", "--needed", "--", "foo bar"]
        );
    }

    #[test]
    fn query_rejects_control_chars_but_allows_spaces() {
        assert!(validate_query("foo bar").is_ok());
        assert!(validate_query("foo\nbar").is_err());
    }

    #[test]
    fn json_escape_handles_quotes_and_newlines() {
        assert_eq!(json_escape("a\"b\nc"), "a\\\"b\\nc");
    }

    #[test]
    fn search_limit_is_bounded_and_parsed() {
        let args = vec!["tool".into(), "--limit=25".into()];
        assert_eq!(parse_search_args(&args).unwrap().3, 25);
        assert!(parse_limit("0").is_err());
        assert!(parse_limit("1001").is_err());
    }
}
