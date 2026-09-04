//! Diagnóstico nativo y de solo lectura del anfitrión.
//!
//! La selección de plataforma se hace al compilar. Cada backend solo conoce
//! comandos propios de su sistema y devuelve el mismo modelo para el CLI, la
//! GUI y el contrato JSON.

#[cfg(not(windows))]
mod linux;
#[cfg(windows)]
mod windows;

use crate::common::Context;

#[derive(Debug, Clone)]
pub(crate) struct Probe {
    pub key: &'static str,
    pub command: &'static str,
    pub available: bool,
    pub output: String,
}

pub fn run(_ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("health");
    let format = format_of(args)?;
    let probes = match action {
        "health" | "status" | "summary" => current::collect("health"),
        "network" | "net" | "red" => current::collect("network"),
        "hardware" | "hw" | "hardware-info" => current::collect("hardware"),
        "users" | "sessions" | "user-sessions" => current::collect("users"),
        "help" => {
            println!("{}", help());
            return Ok(());
        }
        _ => return Err(format!("diagnostics: acción desconocida: {action}")),
    };
    if probes.is_empty() {
        return Err("el backend de diagnóstico no devolvió ninguna comprobación".into());
    }
    match format.as_str() {
        "human" => print_human(action, &probes),
        "tsv" => print_tsv(&probes),
        "json" => print_json(action, &probes),
        _ => unreachable!(),
    }
    Ok(())
}

pub fn help() -> &'static str {
    crate::i18n::diagnostics_help()
}

fn format_of(args: &[String]) -> Result<String, String> {
    if args.iter().any(|arg| arg == "--json") {
        return Ok("json".into());
    }
    if let Some(index) = args.iter().position(|arg| arg == "--format") {
        let Some(value) = args.get(index + 1) else {
            return Err("diagnostics: --format requiere human, tsv o json".into());
        };
        if value.starts_with('-') {
            return Err("diagnostics: --format requiere human, tsv o json".into());
        }
    }
    let value = args
        .windows(2)
        .find(|pair| pair[0] == "--format")
        .map(|pair| pair[1].to_lowercase())
        .or_else(|| {
            args.iter()
                .find_map(|arg| arg.strip_prefix("--format=").map(str::to_lowercase))
        })
        .unwrap_or_else(|| "human".into());
    if matches!(value.as_str(), "human" | "text" | "tsv" | "json") {
        Ok(if value == "text" {
            "human".into()
        } else {
            value
        })
    } else {
        Err("diagnostics admite --format human|tsv|json".into())
    }
}

fn print_human(action: &str, probes: &[Probe]) {
    println!("=== {}: {} ===", crate::i18n::diagnostics_label(), action);
    for probe in probes {
        let state = if probe.available {
            crate::i18n::diagnostics_available()
        } else {
            crate::i18n::diagnostics_unavailable()
        };
        println!("\n[{}] {} — {}", state, probe.key, probe.command);
        if probe.output.trim().is_empty() {
            println!("{}", crate::i18n::diagnostics_no_output());
        } else {
            println!("{}", probe.output);
        }
    }
}

fn print_tsv(probes: &[Probe]) {
    println!("key\tcommand\tavailable\toutput");
    for probe in probes {
        println!(
            "{}\t{}\t{}\t{}",
            probe.key,
            probe.command,
            if probe.available { "yes" } else { "no" },
            crate::common::clean(&probe.output)
        );
    }
}

fn print_json(action: &str, probes: &[Probe]) {
    println!(
        "{{\"schema\":\"ltools-diagnostics-v1\",\"action\":\"{}\",\"platform\":\"{}\",\"probes\":[{}]}}",
        escape(action),
        if cfg!(windows) { "windows" } else { "linux" },
        probes
            .iter()
            .map(|probe| {
                format!(
                    "{{\"key\":\"{}\",\"command\":\"{}\",\"available\":{},\"output\":\"{}\"}}",
                    escape(probe.key),
                    escape(probe.command),
                    probe.available,
                    escape(&probe.output)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    );
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(not(windows))]
use linux as current;
#[cfg(windows)]
use windows as current;

#[cfg(test)]
mod tests {
    use super::format_of;

    #[test]
    fn acepta_formatos_de_diagnostico() {
        assert_eq!(format_of(&[]).unwrap(), "human");
        assert_eq!(format_of(&["--json".into()]).unwrap(), "json");
        assert_eq!(format_of(&["--format=json".into()]).unwrap(), "json");
        assert_eq!(
            format_of(&["--format".into(), "tsv".into()]).unwrap(),
            "tsv"
        );
    }

    #[test]
    fn rechaza_formatos_no_declarados() {
        assert!(format_of(&["--format".into(), "xml".into()]).is_err());
        assert!(format_of(&["--format".into()]).is_err());
    }
}
