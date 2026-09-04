//! Registro común de acciones guiadas.
//!
//! Cada entrada describe una operación real del backend: no contiene shell,
//! no concatena argumentos y no ejecuta nada al enumerarse. Los frontends
//! pueden usar el mismo identificador para crear un botón, un comando CLI o
//! una acción declarativa de terminal.

use crate::common::Context;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionSpec {
    pub id: &'static str,
    pub category: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub target: &'static str,
    pub mutating: bool,
    pub confirmation: &'static str,
    pub profile: &'static str,
}

const NONE: &[&str] = &[];
#[cfg(not(windows))]
const STORAGE_MOUNT: &[&str] = &["mount"];
#[cfg(not(windows))]
const STORAGE_UNMOUNT: &[&str] = &["unmount"];
const STORAGE_INSPECT: &[&str] = &["inspect"];
const SERVICE_STATUS: &[&str] = &["service", "status"];
const PROCESS_STATUS: &[&str] = &["process", "status"];
#[cfg(not(windows))]
const DEPENDENCIES: &[&str] = &["dependencies", "--unit"];

#[cfg(not(windows))]
static ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        id: "audit.quick",
        category: "audit",
        command: "audit",
        args: &["--no-mounts"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "packages.inventory",
        category: "audit",
        command: "packages",
        args: NONE,
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "games.inventory",
        category: "audit",
        command: "games",
        args: &["--no-mounts"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.overview",
        category: "storage",
        command: "storage",
        args: &["status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.partitions",
        category: "storage",
        command: "storage",
        args: &["partitions"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.mounts",
        category: "storage",
        command: "storage",
        args: &["mounts"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.usage",
        category: "storage",
        command: "storage",
        args: &["usage"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.filesystems",
        category: "storage",
        command: "storage",
        args: &["filesystems"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.volume-stack",
        category: "storage",
        command: "storage",
        args: &["volume-stack"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.block-device",
        category: "storage",
        command: "storage",
        args: &["blockdev"],
        target: "device",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.inspect",
        category: "storage",
        command: "storage",
        args: STORAGE_INSPECT,
        target: "device-or-volume",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.mount",
        category: "storage",
        command: "storage",
        args: STORAGE_MOUNT,
        target: "device",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.unmount",
        category: "storage",
        command: "storage",
        args: STORAGE_UNMOUNT,
        target: "device-or-mount",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.manager",
        category: "storage",
        command: "storage",
        args: &["open-gparted"],
        target: "none",
        mutating: false,
        confirmation: "backend",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.network-status",
        category: "native",
        command: "native",
        args: &["network", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.hardware-status",
        category: "native",
        command: "native",
        args: &["hardware", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.power-status",
        category: "native",
        command: "native",
        args: &["power", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.security-status",
        category: "native",
        command: "native",
        args: &["security", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.dns-flush",
        category: "native",
        command: "native",
        args: &["network", "flush-dns"],
        target: "none",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "system.health",
        category: "system",
        command: "system",
        args: &["status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "system.services",
        category: "system",
        command: "system",
        args: &[
            "services",
            "--filter",
            "noteworthy",
            "--scope",
            "both",
            "--limit",
            "50",
        ],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "system.processes",
        category: "system",
        command: "system",
        args: &["processes", "--sort", "cpu", "--limit", "20"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "system.journal",
        category: "system",
        command: "system",
        args: &[
            "journal", "--level", "warning", "--hours", "24", "--limit", "100",
        ],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "system.service-status",
        category: "system",
        command: "system",
        args: SERVICE_STATUS,
        target: "service",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "system.service-manage",
        category: "system",
        command: "system",
        args: &["service"],
        target: "action-and-service",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "system.process-status",
        category: "system",
        command: "system",
        args: PROCESS_STATUS,
        target: "pid",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "system.dependencies",
        category: "system",
        command: "system",
        args: DEPENDENCIES,
        target: "service",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "accounts.list",
        category: "accounts",
        command: "accounts",
        args: &["list"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "accounts.sessions",
        category: "accounts",
        command: "accounts",
        args: &["sessions"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "accounts.inspect",
        category: "accounts",
        command: "accounts",
        args: &["inspect"],
        target: "user",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "accounts.add",
        category: "accounts",
        command: "accounts",
        args: &["add"],
        target: "user",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "accounts.lock",
        category: "accounts",
        command: "accounts",
        args: &["lock"],
        target: "user",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "accounts.delete",
        category: "accounts",
        command: "accounts",
        args: &["delete"],
        target: "user",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "defaults.show",
        category: "configuration",
        command: "defaults",
        args: NONE,
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "clean.preview",
        category: "maintenance",
        command: "clean",
        args: &["--preview"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "diagnostics.health",
        category: "diagnostics",
        command: "diagnostics",
        args: &["health"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "boot.status",
        category: "boot",
        command: "boot",
        args: &["status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "boot.plan",
        category: "boot",
        command: "boot",
        args: &["plan"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "automation.list",
        category: "automation",
        command: "automation",
        args: &["list"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
];

#[cfg(windows)]
static ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        id: "boot.status",
        category: "boot",
        command: "boot",
        args: &["status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "boot.plan",
        category: "boot",
        command: "boot",
        args: &["plan"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "audit.quick",
        category: "audit",
        command: "audit",
        args: &["--no-mounts"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "packages.inventory",
        category: "audit",
        command: "packages",
        args: NONE,
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "games.inventory",
        category: "audit",
        command: "games",
        args: &["--no-mounts"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.overview",
        category: "storage",
        command: "storage",
        args: &["status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.partitions",
        category: "storage",
        command: "storage",
        args: &["partitions"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.mounts",
        category: "storage",
        command: "storage",
        args: &["mounts"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.volumes",
        category: "storage",
        command: "storage",
        args: &["status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.usage",
        category: "storage",
        command: "storage",
        args: &["usage"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.pools",
        category: "storage",
        command: "storage",
        args: &["pools"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.bitlocker",
        category: "storage",
        command: "storage",
        args: &["bitlocker"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.inspect",
        category: "storage",
        command: "storage",
        args: STORAGE_INSPECT,
        target: "volume-or-disk",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.manager",
        category: "storage",
        command: "storage",
        args: &["open-disk-management"],
        target: "none",
        mutating: false,
        confirmation: "backend",
        profile: "safe-default",
    },
    ActionSpec {
        id: "storage.diskpart",
        category: "storage",
        command: "storage",
        args: &["open-diskpart"],
        target: "none",
        mutating: false,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "native.network-status",
        category: "native",
        command: "native",
        args: &["network", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.hardware-status",
        category: "native",
        command: "native",
        args: &["hardware", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.power-status",
        category: "native",
        command: "native",
        args: &["power", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.security-status",
        category: "native",
        command: "native",
        args: &["security", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.dns-flush",
        category: "native",
        command: "native",
        args: &["network", "flush-dns"],
        target: "none",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "system.health",
        category: "system",
        command: "system",
        args: &["status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "system.services",
        category: "system",
        command: "system",
        args: &["services", "--filter", "active", "--limit", "50"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "system.processes",
        category: "system",
        command: "system",
        args: &["processes", "--sort", "memory", "--limit", "20"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "system.events",
        category: "system",
        command: "system",
        args: &["journal", "--channel", "System", "--limit", "100"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "system.service-status",
        category: "system",
        command: "system",
        args: SERVICE_STATUS,
        target: "service",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "system.service-manage",
        category: "system",
        command: "system",
        args: &["service"],
        target: "action-and-service",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "system.process-status",
        category: "system",
        command: "system",
        args: PROCESS_STATUS,
        target: "pid",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "accounts.list",
        category: "accounts",
        command: "accounts",
        args: &["list"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "accounts.sessions",
        category: "accounts",
        command: "accounts",
        args: &["sessions"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "accounts.inspect",
        category: "accounts",
        command: "accounts",
        args: &["inspect"],
        target: "user",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "accounts.enable",
        category: "accounts",
        command: "accounts",
        args: &["enable"],
        target: "user",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "accounts.disable",
        category: "accounts",
        command: "accounts",
        args: &["disable"],
        target: "user",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "accounts.delete",
        category: "accounts",
        command: "accounts",
        args: &["delete"],
        target: "user",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "defaults.show",
        category: "configuration",
        command: "defaults",
        args: NONE,
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "clean.preview",
        category: "maintenance",
        command: "clean",
        args: &["--preview"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "diagnostics.health",
        category: "diagnostics",
        command: "diagnostics",
        args: &["health"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "automation.list",
        category: "automation",
        command: "automation",
        args: &["list"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
];

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .find(|value| !value.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("menu");
    match action {
        "list" => list(args),
        "run" => run_named(ctx, args),
        "menu" => menu(ctx),
        _ => Err("actions admite: list, run ID [TARGET] o menu".into()),
    }
}

fn list(args: &[String]) -> Result<(), String> {
    let format = option_value(args, "--format").unwrap_or_else(|| "text".into());
    if format == "json" {
        let policy = if cfg!(windows) {
            r#"{"excluded_defaults":["C:"],"target_selection":"explicit-only"}"#
        } else {
            r#"{"excluded_defaults":["/","/boot","/home"],"target_selection":"explicit-only"}"#
        };
        println!("{{\"schema\":\"ltools-actions-v1\",\"platform\":\"{}\",\"safety\":{},\"actions\":[{}]}}", if cfg!(windows) { "windows" } else { "linux" }, policy, ACTIONS.iter().map(action_json).collect::<Vec<_>>().join(","));
        return Ok(());
    }
    if format != "text" {
        return Err("actions --format debe ser text o json".into());
    }
    println!("{}", crate::i18n::actions_text("title"));
    println!("ID | categoría | objetivo | perfil | modifica | confirmación");
    println!("Valores seguros: sin objetivo por defecto; los objetivos sensibles siempre son explícitos.");
    for spec in ACTIONS {
        println!(
            "{} | {} | {} | {} | {} | {}",
            spec.id,
            spec.category,
            spec.target,
            spec.profile,
            if spec.mutating { "sí" } else { "no" },
            spec.confirmation
        );
    }
    println!("\n{}", crate::i18n::actions_text("hint"));
    Ok(())
}

fn run_named(ctx: &Context, args: &[String]) -> Result<(), String> {
    let position = args
        .iter()
        .position(|value| value == "run")
        .ok_or("falta run")?;
    let id = args.get(position + 1).ok_or("actions run requiere un ID")?;
    let spec = ACTIONS
        .iter()
        .find(|candidate| candidate.id == id)
        .ok_or_else(|| format!("acción desconocida o no disponible en esta plataforma: {id}"))?;
    let target = args
        .get(position + 2)
        .filter(|value| !value.starts_with('-'))
        .map(String::as_str);
    if spec.target != "none" && target.is_none() {
        return Err(format!("{} requiere objetivo ({})", spec.id, spec.target));
    }
    let mut delegated = spec
        .args
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    if let Some(target) = target {
        if spec.target == "action-and-service" {
            let mut parts = target.split_whitespace();
            let operation = parts.next().ok_or("falta la acción del servicio")?;
            let unit = parts.next().ok_or("falta la unidad del servicio")?;
            if parts.next().is_some() {
                return Err("el objetivo de servicio debe ser: ACCIÓN UNIDAD".into());
            }
            delegated.push(operation.to_string());
            delegated.push(unit.to_string());
        } else {
            delegated.push(target.to_string());
        }
    }
    println!("Acción: {} ({})", spec.id, spec.profile);
    crate::execute_action(spec.command, ctx, &delegated)
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("{}\n", crate::i18n::actions_text("title"));
        for (index, spec) in ACTIONS.iter().enumerate() {
            println!("  {:>2}) {} [{}]", index + 1, spec.id, spec.category);
        }
        println!("  l) {}", crate::i18n::actions_text("list"));
        println!("  q) {}", crate::i18n::text("menu.back"));
        print!("{}", crate::i18n::text("menu.prompt"));
        let _ = io::stdout().flush();
        let Some(answer) = crate::menu_input("") else {
            return Ok(());
        };
        let answer = answer.trim();
        if answer.is_empty() || matches!(answer.to_lowercase().as_str(), "q" | "quit" | "salir") {
            return Ok(());
        }
        if answer.eq_ignore_ascii_case("l") {
            list(&[])?;
            let _ = crate::menu_input(crate::i18n::tools_text("pause"));
            continue;
        }
        let Ok(index) = answer.parse::<usize>() else {
            println!("{}", crate::i18n::text("menu.invalid"));
            let _ = crate::menu_input(crate::i18n::tools_text("pause"));
            continue;
        };
        let Some(spec) = ACTIONS.get(index.saturating_sub(1)) else {
            println!("{}", crate::i18n::text("menu.invalid"));
            let _ = crate::menu_input(crate::i18n::tools_text("pause"));
            continue;
        };
        let mut args = spec
            .args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        if spec.target != "none" {
            let Some(target) =
                crate::menu_input(&format!("Objetivo ({}; Enter para volver): ", spec.target))
            else {
                return Ok(());
            };
            if target.is_empty() {
                continue;
            }
            args.push(target);
        }
        let result = crate::execute_action(spec.command, ctx, &args);
        match result {
            Ok(()) => println!("Operación terminada correctamente."),
            Err(error) => println!("Error: {error}"),
        }
        let _ = crate::menu_input(crate::i18n::tools_text("pause"));
    }
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].clone())
        .or_else(|| {
            args.iter().find_map(|value| {
                value
                    .strip_prefix(&format!("{name}="))
                    .map(ToOwned::to_owned)
            })
        })
}

fn action_json(spec: &ActionSpec) -> String {
    let args = spec
        .args
        .iter()
        .map(|arg| format!("\"{}\"", escape(arg)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"id\":\"{}\",\"category\":\"{}\",\"command\":\"{}\",\"args\":[{}],\"target\":\"{}\",\"targetPolicy\":\"{}\",\"mutating\":{},\"confirmation\":\"{}\",\"profile\":\"{}\",\"aliases\":{},\"supports\":[\"dry-run\",\"plan\"]}}", escape(spec.id), escape(spec.category), escape(spec.command), args, escape(spec.target), if spec.target == "none" { "none" } else { "explicit-only" }, spec.mutating, escape(spec.confirmation), escape(spec.profile), crate::shortcuts::json(spec.id))
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_have_stable_shape() {
        let mut ids = ACTIONS.iter().map(|action| action.id).collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ACTIONS.len());
        assert!(ACTIONS.iter().all(|action| action.id.contains('.')));
    }

    #[test]
    fn action_json_is_valid_enough_for_machine_consumers() {
        let value = action_json(&ACTIONS[0]);
        assert!(value.starts_with("{\"id\":"));
        assert!(value.contains("\"supports\":[\"dry-run\",\"plan\"]"));
    }

    #[test]
    fn potentially_destructive_actions_have_an_explicit_guard() {
        for action in ACTIONS.iter().filter(|action| action.mutating) {
            assert_ne!(
                action.confirmation, "none",
                "{} lacks confirmation",
                action.id
            );
            if action.id != "native.dns-flush" {
                assert_ne!(
                    action.target, "none",
                    "{} lacks an explicit target",
                    action.id
                );
            }
        }
        for action in ACTIONS.iter().filter(|action| {
            action.id.contains("storage.manager") || action.id.contains("storage.diskpart")
        }) {
            assert_eq!(
                action.confirmation, "backend",
                "{} must confirm before opening a native manager",
                action.id
            );
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn linux_does_not_publish_windows_only_actions() {
        assert!(!ACTIONS.iter().any(|action| action.id.contains("diskpart")));
        assert!(ACTIONS.iter().any(|action| action.id == "storage.mount"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_does_not_publish_linux_only_actions() {
        assert!(!ACTIONS.iter().any(|action| action.id == "storage.mount"));
        assert!(ACTIONS.iter().any(|action| action.id == "storage.diskpart"));
    }
}
