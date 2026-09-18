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
const SNAPSHOTS_STATUS: &[&str] = &["status"];
const SNAPSHOTS_LIST: &[&str] = &["list"];
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
        id: "storage.partition-table",
        category: "storage",
        command: "storage",
        args: &["partition-table"],
        target: "device",
        mutating: false,
        confirmation: "none",
        profile: "advanced",
    },
    ActionSpec {
        id: "storage.partition-guide",
        category: "storage",
        command: "storage",
        args: &["guide"],
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
        id: "snapshots.status",
        category: "snapshots",
        command: "snapshots",
        args: SNAPSHOTS_STATUS,
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "snapshots.list",
        category: "snapshots",
        command: "snapshots",
        args: SNAPSHOTS_LIST,
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
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
        id: "native.security-scanners",
        category: "native",
        command: "native",
        args: &["security", "scanners"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.tooling-status",
        category: "native",
        command: "native",
        args: &["tools"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.containers-status",
        category: "native",
        command: "native",
        args: &["containers", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.kubernetes-status",
        category: "native",
        command: "native",
        args: &["kubernetes", "status"],
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
        id: "native.network-interface",
        category: "native",
        command: "native",
        args: &["network", "set-interface"],
        target: "interface-and-state",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "native.networkmanager-connection",
        category: "native",
        command: "native",
        args: &["network", "connection-up"],
        target: "connection",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "native.networkmanager-disconnect",
        category: "native",
        command: "native",
        args: &["network", "connection-down"],
        target: "connection",
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
        id: "boot.clear-next",
        category: "boot",
        command: "boot",
        args: &["clear-next"],
        target: "boot-environment",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "wine.create",
        category: "wine",
        command: "wine",
        args: &["create"],
        target: "prefix-destination",
        mutating: true,
        confirmation: "backend",
        profile: "advanced",
    },
    ActionSpec {
        id: "wine.migrate",
        category: "wine",
        command: "wine",
        args: &["migrate"],
        target: "prefix-source-and-destination",
        mutating: true,
        confirmation: "backend",
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
        id: "storage.partition-guide",
        category: "storage",
        command: "storage",
        args: &["guide"],
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
        id: "snapshots.status",
        category: "snapshots",
        command: "snapshots",
        args: SNAPSHOTS_STATUS,
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "snapshots.list",
        category: "snapshots",
        command: "snapshots",
        args: SNAPSHOTS_LIST,
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
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
        id: "native.security-scanners",
        category: "native",
        command: "native",
        args: &["security", "scanners"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.tooling-status",
        category: "native",
        command: "native",
        args: &["tools"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.containers-status",
        category: "native",
        command: "native",
        args: &["containers", "status"],
        target: "none",
        mutating: false,
        confirmation: "none",
        profile: "safe-default",
    },
    ActionSpec {
        id: "native.kubernetes-status",
        category: "native",
        command: "native",
        args: &["kubernetes", "status"],
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

/// Indicates whether a named catalog action changes state and therefore needs
/// the transaction/plan boundary used by the CLI. Listing or running a
/// read-only action must not create a plan merely because its verb is `run`.
pub(crate) fn needs_plan(args: &[String]) -> bool {
    let Some(position) = args.iter().position(|value| value == "run") else {
        return false;
    };
    let Some(id) = args.get(position + 1) else {
        return false;
    };
    ACTIONS
        .iter()
        .find(|action| action_selector_matches(action, id))
        .is_some_and(|action| action.mutating)
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
    println!(
        "actionId | legacyId | actionKey | nombre | objetivo | perfil | modifica | confirmación"
    );
    println!("Valores seguros: sin objetivo por defecto; los objetivos sensibles siempre son explícitos.");
    for spec in ACTIONS {
        let metadata =
            action_metadata(spec.id).expect("cada acción debe tener metadatos descriptivos");
        println!(
            "{} | {} | {} | {} | {} | {} | {} | {} | {}",
            qualified_action_key(metadata.0),
            spec.id,
            metadata.0,
            metadata.1,
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
        .find(|candidate| action_selector_matches(candidate, id))
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
            match spec.target {
                "boot-environment" => {
                    if target != "grubenv" {
                        return Err("el objetivo debe ser exactamente: grubenv".into());
                    }
                }
                "interface-and-state" => {
                    let mut parts = target.split_whitespace();
                    let interface = parts.next().ok_or("falta la interfaz")?;
                    let state = parts.next().ok_or("falta el estado up|down")?;
                    if parts.next().is_some() {
                        return Err("el objetivo debe ser: INTERFAZ ESTADO".into());
                    }
                    delegated.extend([
                        "--interface".to_owned(),
                        interface.to_owned(),
                        "--state".to_owned(),
                        state.to_owned(),
                    ]);
                }
                "connection" => {
                    delegated.extend(["--connection".to_owned(), target.to_owned()]);
                }
                "prefix-destination" => {
                    delegated.extend(["--dest".to_owned(), target.to_owned()]);
                }
                "prefix-source-and-destination" => {
                    let (source, destination) = target
                        .split_once(" -> ")
                        .ok_or("el objetivo debe ser: ORIGEN -> DESTINO")?;
                    if source.is_empty() || destination.is_empty() {
                        return Err("origen y destino son obligatorios".into());
                    }
                    delegated.extend([
                        "--source".to_owned(),
                        source.to_owned(),
                        "--dest".to_owned(),
                        destination.to_owned(),
                    ]);
                }
                _ => delegated.push(target.to_string()),
            }
        }
    }
    let metadata = action_metadata(spec.id).expect("cada acción debe tener metadatos descriptivos");
    println!(
        "Acción: {} — {} (id: {}; perfil: {})",
        metadata.0, metadata.1, spec.id, spec.profile
    );
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
    let metadata = action_metadata(spec.id).expect("cada acción debe tener metadatos descriptivos");
    let action_key = metadata.0;
    let platform = if cfg!(windows) { "windows" } else { "linux" };
    let qualified_action_key = format!("{platform}.{action_key}");
    let scope = action_key.split('.').next().unwrap_or("action");
    let group = action_group(spec.category);
    let platform_label = if cfg!(windows) { "Windows" } else { "Linux" };
    let display_name = format!("{platform_label} · {group} · {}", metadata.1);
    // El catálogo también se muestra sin `scope`; la operación cualificada
    // evita colisiones como `status` entre dominios distintos.
    let operation = action_key.replace('.', "-");
    let args = spec
        .args
        .iter()
        .map(|arg| format!("\"{}\"", escape(arg)))
        .collect::<Vec<_>>()
        .join(",");
    let safe = !spec.mutating && spec.confirmation == "none";
    let executable = if cfg!(windows) {
        "ltools.exe"
    } else {
        "ltools"
    };
    // La invocación publicable usa la identidad cualificada. Los selectores
    // cortos siguen aceptándose en `actions run` por compatibilidad, pero un
    // descriptor copiado entre plataformas no debe perder su contexto.
    let invocation_args = format!(
        "[\"actions\",\"run\",\"{}\"]",
        escape(&qualified_action_key)
    );
    format!("{{\"id\":\"{}\",\"legacyId\":\"{}\",\"actionId\":\"{}\",\"actionKey\":\"{}\",\"qualifiedActionKey\":\"{}\",\"canonicalKey\":\"{}\",\"scope\":\"{}\",\"operation\":\"{}\",\"label\":\"{}\",\"shortLabel\":\"{}\",\"displayName\":\"{}\",\"menuPath\":[\"{}\",\"{}\"],\"group\":\"{}\",\"description\":\"{}\",\"category\":\"{}\",\"command\":\"{}\",\"args\":[{}],\"target\":\"{}\",\"targetPolicy\":\"{}\",\"invocation\":{{\"executable\":\"{}\",\"args\":{},\"target\":\"{}\"}},\"mutating\":{},\"confirmation\":\"{}\",\"safe\":{},\"profile\":\"{}\",\"aliases\":{},\"supports\":[\"dry-run\",\"plan\"]}}", escape(&qualified_action_key), escape(spec.id), escape(&qualified_action_key), escape(action_key), escape(&qualified_action_key), escape(&qualified_action_key), escape(scope), escape(&operation), escape(metadata.1), escape(metadata.2), escape(&display_name), escape(group), escape(metadata.1), escape(group), escape(metadata.3), escape(spec.category), escape(spec.command), args, escape(spec.target), if spec.target == "none" { "none" } else { "explicit-only" }, executable, invocation_args, escape(spec.target), spec.mutating, escape(spec.confirmation), safe, escape(spec.profile), crate::shortcuts::json(spec.id))
}

fn action_group(category: &str) -> &'static str {
    match category {
        "audit" => "Auditoría",
        "storage" => "Almacenamiento",
        "snapshots" => "Instantáneas y recuperación",
        "native" => "Herramientas nativas",
        "system" => "Sistema",
        "accounts" => "Usuarios y permisos",
        "defaults" => "Valores seguros",
        "clean" => "Limpieza",
        "diagnostics" => "Diagnóstico",
        "boot" => "Arranque",
        "wine" => "Wine y compatibilidad",
        "automation" => "Automatización",
        _ => "Acciones",
    }
}

fn qualified_action_key(action_key: &str) -> String {
    format!(
        "{}.{}",
        if cfg!(windows) { "windows" } else { "linux" },
        action_key
    )
}

fn action_selector_matches(spec: &ActionSpec, selector: &str) -> bool {
    spec.id == selector
        || action_metadata(spec.id).is_some_and(|metadata| {
            metadata.0 == selector || qualified_action_key(metadata.0) == selector
        })
}

/// Stable machine identity and human-readable metadata for the executable
/// action catalog. JSON `id`/`actionId`/`canonicalKey` are the qualified
/// platform identity; `legacyId` remains the backwards-compatible selector.
/// `displayName`/`menuPath` are what integrations should display.
/// Keeping this table explicit prevents a translated label or a terse command
/// name such as `status` from becoming an accidental action identity.
fn action_metadata(id: &str) -> Option<(&'static str, &'static str, &'static str, &'static str)> {
    Some(match id {
        "audit.quick" => (
            "audit.quick",
            "Auditar el sistema",
            "Auditoría rápida",
            "Revisa la salud básica del sistema sin modificarlo.",
        ),
        "packages.inventory" => (
            "packages.inventory",
            "Inventariar paquetes",
            "Paquetes detectados",
            "Enumera paquetes y gestores disponibles para conocer el software instalado.",
        ),
        "games.inventory" => (
            "games.inventory",
            "Inventariar juegos",
            "Juegos detectados",
            "Localiza juegos y lanzadores sin modificar instalaciones ni prefijos.",
        ),
        "storage.overview" => (
            "storage.overview",
            "Resumir almacenamiento",
            "Resumen de discos",
            "Muestra discos, particiones, espacio ocupado y espacio libre.",
        ),
        "storage.partitions" => (
            "storage.partitions",
            "Listar particiones",
            "Particiones",
            "Lista particiones y sus dispositivos para preparar una operación segura.",
        ),
        "storage.partition-table" => (
            "storage.partition-table",
            "Leer tabla de particiones",
            "Tabla de particiones",
            "Inspecciona la tabla GPT o MBR de un dispositivo elegido explícitamente.",
        ),
        "storage.partition-guide" => (
            "storage.partition-guide",
            "Abrir guía de particionado",
            "Guía de particiones",
            "Explica operaciones de particionado, riesgos, copias y comprobaciones.",
        ),
        "storage.mounts" => (
            "storage.mounts",
            "Listar montajes",
            "Montajes activos",
            "Muestra volúmenes montados, rutas y sistemas de archivos.",
        ),
        "storage.usage" => (
            "storage.usage",
            "Medir uso de almacenamiento",
            "Uso de disco",
            "Calcula el uso de rutas y volúmenes sin borrar ni mover datos.",
        ),
        "storage.filesystems" => (
            "storage.filesystems",
            "Detectar sistemas de archivos",
            "Sistemas de archivos",
            "Identifica sistemas de archivos y herramientas de comprobación disponibles.",
        ),
        "storage.volume-stack" => (
            "storage.volume-stack",
            "Inspeccionar capas de volumen",
            "Capas de volumen",
            "Consulta LUKS, LVM, RAID, Btrfs, ZFS u otras capas detectadas.",
        ),
        "storage.block-device" => (
            "storage.block-device",
            "Inspeccionar dispositivo de bloque",
            "Dispositivo de bloque",
            "Consulta atributos y metadatos de un dispositivo seleccionado explícitamente.",
        ),
        "storage.inspect" => (
            "storage.inspect",
            "Inspeccionar disco o volumen",
            "Inspección de volumen",
            "Reúne información detallada de un disco, partición o volumen concreto.",
        ),
        "snapshots.status" => (
            "snapshots.status",
            "Consultar instantáneas",
            "Estado de snapshots",
            "Detecta backends de instantáneas y su disponibilidad sin crear cambios.",
        ),
        "snapshots.list" => (
            "snapshots.list",
            "Listar instantáneas",
            "Lista de snapshots",
            "Enumera instantáneas existentes por backend y muestra sus objetivos.",
        ),
        "storage.mount" => (
            "storage.mount",
            "Montar dispositivo",
            "Montar volumen",
            "Monta un dispositivo indicado explícitamente después de revisar el plan.",
        ),
        "storage.unmount" => (
            "storage.unmount",
            "Desmontar dispositivo",
            "Desmontar volumen",
            "Desmonta un dispositivo o punto de montaje indicado explícitamente.",
        ),
        "storage.manager" => (
            "storage.manager",
            "Abrir gestor de particiones",
            "Gestor de discos",
            "Abre el gestor nativo para revisar operaciones avanzadas de discos.",
        ),
        "storage.volumes" => (
            "storage.volumes",
            "Administrar volúmenes Windows",
            "Volúmenes Windows",
            "Consulta y gestiona volúmenes mediante las herramientas nativas de Windows.",
        ),
        "storage.pools" => (
            "storage.pools",
            "Administrar espacios de almacenamiento",
            "Espacios Windows",
            "Consulta pools y espacios de almacenamiento de Windows antes de modificarlos.",
        ),
        "storage.bitlocker" => (
            "storage.bitlocker",
            "Administrar BitLocker",
            "BitLocker",
            "Consulta o gestiona el cifrado BitLocker con objetivos explícitos.",
        ),
        "storage.diskpart" => (
            "storage.diskpart",
            "Abrir DiskPart",
            "DiskPart",
            "Abre la herramienta nativa de particionado de Windows para operaciones revisables.",
        ),
        "native.network-status" => (
            "native.network.status",
            "Consultar red nativa",
            "Estado de red",
            "Muestra interfaces, rutas, DNS y conectividad con herramientas de la plataforma.",
        ),
        "native.hardware-status" => (
            "native.hardware.status",
            "Consultar hardware nativo",
            "Estado de hardware",
            "Muestra hardware y controladores detectados con utilidades nativas.",
        ),
        "native.power-status" => (
            "native.power.status",
            "Consultar energía nativa",
            "Estado de energía",
            "Muestra batería, suspensión y configuración de energía disponible.",
        ),
        "native.security-status" => (
            "native.security.status",
            "Consultar seguridad nativa",
            "Estado de seguridad",
            "Consulta firewall, antivirus, permisos y protecciones de la plataforma.",
        ),
        "native.security-scanners" => (
            "native.security.scanners",
            "Consultar escáneres de seguridad",
            "Escáneres de seguridad",
            "Comprueba analizadores de código y seguridad instalados o disponibles.",
        ),
        "native.tooling-status" => (
            "native.tooling.status",
            "Consultar herramientas nativas",
            "Herramientas nativas",
            "Enumera herramientas nativas, versiones y posibilidades de instalación.",
        ),
        "native.containers-status" => (
            "native.containers.status",
            "Consultar contenedores",
            "Estado de contenedores",
            "Consulta Docker, Podman y sus componentes sin iniciar ni eliminar recursos.",
        ),
        "native.kubernetes-status" => (
            "native.kubernetes.status",
            "Consultar Kubernetes",
            "Estado de Kubernetes",
            "Consulta kubectl, clústeres y herramientas Kubernetes disponibles.",
        ),
        "native.dns-flush" => (
            "native.network.flush-dns",
            "Vaciar caché DNS",
            "Vaciar DNS",
            "Vacía la caché DNS usando el mecanismo nativo y confirma antes de ejecutarlo.",
        ),
        "native.network-interface" => (
            "native.network.set-interface",
            "Cambiar estado de interfaz",
            "Activar interfaz",
            "Activa o desactiva una interfaz de red indicada explícitamente.",
        ),
        "native.networkmanager-connection" => (
            "native.network.connection-up",
            "Activar conexión de red",
            "Conectar red",
            "Activa una conexión de NetworkManager indicada explícitamente.",
        ),
        "native.networkmanager-disconnect" => (
            "native.network.connection-down",
            "Desactivar conexión de red",
            "Desconectar red",
            "Desactiva una conexión de NetworkManager indicada explícitamente.",
        ),
        "system.health" => (
            "system.health",
            "Consultar salud del sistema",
            "Salud del sistema",
            "Resume salud, versión, recursos y capacidades del sistema operativo.",
        ),
        "system.services" => (
            "system.services",
            "Explorar servicios",
            "Servicios del sistema",
            "Lista servicios automáticos, manuales, propietarios y estados relevantes.",
        ),
        "system.processes" => (
            "system.processes",
            "Explorar procesos",
            "Procesos activos",
            "Lista procesos y consumo de recursos para diagnóstico.",
        ),
        "system.journal" => (
            "system.journal",
            "Consultar journal",
            "Registros del sistema",
            "Consulta avisos recientes del registro del sistema con límites seguros.",
        ),
        "system.events" => (
            "system.events",
            "Consultar eventos Windows",
            "Eventos Windows",
            "Consulta eventos recientes del sistema y sus niveles de gravedad.",
        ),
        "system.service-status" => (
            "system.service.status",
            "Consultar un servicio",
            "Estado de servicio",
            "Consulta un servicio concreto y sus dependencias inmediatas.",
        ),
        "system.service-manage" => (
            "system.service.manage",
            "Gestionar un servicio",
            "Gestionar servicio",
            "Inicia, detiene o cambia un servicio tras validar acción y objetivo.",
        ),
        "system.process-status" => (
            "system.process.status",
            "Consultar un proceso",
            "Estado de proceso",
            "Inspecciona un proceso concreto por PID antes de actuar.",
        ),
        "system.dependencies" => (
            "system.dependencies",
            "Consultar dependencias",
            "Dependencias",
            "Muestra dependencias de un servicio para evitar cambios incompletos.",
        ),
        "accounts.list" => (
            "accounts.list",
            "Listar usuarios",
            "Usuarios del sistema",
            "Lista cuentas locales y su estado sin modificar permisos.",
        ),
        "accounts.sessions" => (
            "accounts.sessions",
            "Listar sesiones",
            "Sesiones activas",
            "Muestra sesiones y usuarios conectados en el sistema.",
        ),
        "accounts.inspect" => (
            "accounts.inspect",
            "Inspeccionar usuario",
            "Inspeccionar usuario",
            "Consulta grupos, permisos y estado de una cuenta elegida.",
        ),
        "accounts.add" => (
            "accounts.add",
            "Crear usuario",
            "Crear cuenta",
            "Crea una cuenta con datos explícitos y confirmación administrativa.",
        ),
        "accounts.lock" => (
            "accounts.lock",
            "Bloquear usuario",
            "Bloquear cuenta",
            "Bloquea una cuenta elegida después de confirmar el impacto.",
        ),
        "accounts.delete" => (
            "accounts.delete",
            "Eliminar usuario",
            "Eliminar cuenta",
            "Elimina una cuenta solo con objetivo y confirmación explícitos.",
        ),
        "accounts.enable" => (
            "accounts.enable",
            "Activar usuario",
            "Activar cuenta",
            "Activa una cuenta local de Windows con confirmación administrativa.",
        ),
        "accounts.disable" => (
            "accounts.disable",
            "Desactivar usuario",
            "Desactivar cuenta",
            "Desactiva una cuenta local de Windows con confirmación administrativa.",
        ),
        "defaults.show" => (
            "defaults.show",
            "Consultar valores predeterminados",
            "Valores predeterminados",
            "Muestra políticas y valores seguros detectados para la plataforma.",
        ),
        "clean.preview" => (
            "clean.preview",
            "Previsualizar limpieza",
            "Vista previa de limpieza",
            "Calcula cachés y temporales limpiables sin borrar ningún dato.",
        ),
        "diagnostics.health" => (
            "diagnostics.health",
            "Ejecutar diagnóstico",
            "Diagnóstico",
            "Ejecuta comprobaciones de salud y explica dependencias o permisos ausentes.",
        ),
        "boot.status" => (
            "boot.status",
            "Consultar arranque",
            "Estado de arranque",
            "Muestra cargadores, entradas y configuración de arranque detectada.",
        ),
        "boot.plan" => (
            "boot.plan",
            "Planificar arranque",
            "Plan de arranque",
            "Prepara un plan revisable para una operación de arranque protegida.",
        ),
        "boot.clear-next" => (
            "boot.clear-next",
            "Limpiar siguiente arranque",
            "Limpiar arranque",
            "Elimina una selección de arranque pendiente solo con objetivo explícito.",
        ),
        "wine.create" => (
            "wine.create",
            "Crear prefijo Wine",
            "Crear prefijo",
            "Crea un prefijo Wine en un destino indicado tras revisar el plan.",
        ),
        "wine.migrate" => (
            "wine.migrate",
            "Migrar prefijo Wine",
            "Migrar prefijo",
            "Migra un prefijo Wine entre destinos explícitos con comprobaciones.",
        ),
        "automation.list" => (
            "automation.list",
            "Listar automatizaciones",
            "Automatizaciones",
            "Lista tareas registradas, frecuencia, último resultado y acciones asociadas.",
        ),
        _ => return None,
    })
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
        assert!(value.starts_with("{\"id\":\"linux.audit.quick\""));
        assert!(value.contains("\"legacyId\":\"audit.quick\""));
        assert!(value.contains("\"actionId\":\"linux.audit.quick\""));
        assert!(value.contains("\"actionKey\":\"audit.quick\""));
        assert!(value.contains("\"qualifiedActionKey\":\"linux.audit.quick\""));
        assert!(value.contains("\"canonicalKey\":\"linux.audit.quick\""));
        assert!(value.contains("\"scope\":\"audit\""));
        assert!(value.contains("\"group\":\"Auditoría\""));
        assert!(value.contains("\"invocation\":{\"executable\":\"ltools\""));
        assert!(value.contains("\"label\":\"Auditar el sistema\""));
        assert!(value.contains("\"shortLabel\":\"Auditoría rápida\""));
        assert!(value.contains("\"displayName\":\"Linux · Auditoría · Auditar el sistema\""));
        assert!(value.contains("\"menuPath\":[\"Auditoría\",\"Auditar el sistema\"]"));
        assert!(value.contains("\"description\":"));
        assert!(value.contains("\"supports\":[\"dry-run\",\"plan\"]"));
        assert!(value.contains("\"operation\":\"audit-quick\""));
    }

    #[test]
    fn operations_are_unique_and_qualified() {
        let operations = ACTIONS
            .iter()
            .map(|action| action_metadata(action.id).unwrap().0.replace('.', "-"))
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(operations.len(), ACTIONS.len());
    }

    #[test]
    fn catalog_metadata_is_complete_unique_and_machine_safe() {
        let mut keys = std::collections::HashSet::new();
        let mut operations = std::collections::HashSet::new();
        let mut labels = std::collections::HashSet::new();
        let mut short_labels = std::collections::HashSet::new();
        let mut qualified_keys = std::collections::HashSet::new();
        let mut canonical_keys = std::collections::HashSet::new();
        let mut display_names = std::collections::HashSet::new();
        for action in ACTIONS {
            let value: serde_json::Value =
                serde_json::from_str(&action_json(action)).expect("acción JSON válida");
            let metadata = action_metadata(action.id)
                .unwrap_or_else(|| panic!("falta metadata para {}", action.id));
            let key = value["actionKey"].as_str().unwrap().to_owned();
            let operation = value["operation"].as_str().unwrap().to_owned();
            let label = value["label"].as_str().unwrap().to_owned();
            let short_label = value["shortLabel"].as_str().unwrap().to_owned();
            let legacy_id = value["legacyId"].as_str().unwrap().to_owned();
            let action_id = value["actionId"].as_str().unwrap().to_owned();
            let qualified_key = value["qualifiedActionKey"].as_str().unwrap().to_owned();
            let canonical_key = value["canonicalKey"].as_str().unwrap().to_owned();
            let display_name = value["displayName"].as_str().unwrap().to_owned();
            assert_eq!(key, metadata.0);
            assert_eq!(value["id"].as_str(), Some(qualified_key.as_str()));
            assert_eq!(legacy_id, action.id);
            assert_eq!(action_id, qualified_key);
            assert_eq!(qualified_key, qualified_action_key(&key));
            assert_eq!(canonical_key, qualified_key);
            assert_eq!(
                value["scope"].as_str(),
                Some(key.split('.').next().unwrap())
            );
            assert_eq!(operation, key.replace('.', "-"));
            assert!(!value["description"].as_str().unwrap().trim().is_empty());
            assert!(keys.insert(key.clone()), "actionKey duplicado: {key}");
            assert!(
                operations.insert(operation.clone()),
                "operation duplicada: {operation}"
            );
            assert!(labels.insert(label.clone()), "label duplicado: {label}");
            assert!(
                short_labels.insert(short_label.clone()),
                "shortLabel duplicado: {short_label}"
            );
            assert!(
                qualified_keys.insert(qualified_key.clone()),
                "qualifiedActionKey duplicado: {qualified_key}"
            );
            assert!(
                canonical_keys.insert(canonical_key.clone()),
                "canonicalKey duplicado: {canonical_key}"
            );
            assert!(
                display_names.insert(display_name.clone()),
                "displayName duplicado: {display_name}"
            );
            assert_eq!(
                value["menuPath"],
                serde_json::json!([action_group(action.category), metadata.1])
            );
            assert_eq!(value["group"].as_str(), Some(action_group(action.category)));
            let invocation = &value["invocation"];
            assert_eq!(
                invocation["executable"].as_str(),
                Some(if cfg!(windows) {
                    "ltools.exe"
                } else {
                    "ltools"
                })
            );
            assert_eq!(
                invocation["args"],
                serde_json::json!(["actions", "run", qualified_key])
            );
            assert_eq!(invocation["target"].as_str(), Some(action.target));
            assert_eq!(
                value["safe"].as_bool(),
                Some(!action.mutating && action.confirmation == "none")
            );
        }
    }

    #[test]
    fn read_only_catalog_actions_do_not_need_a_plan() {
        assert!(!needs_plan(&["run".into(), "audit.quick".into()]));
        assert!(!needs_plan(&["run".into(), "storage.overview".into()]));
        assert!(!needs_plan(&[
            "run".into(),
            "native.security-scanners".into(),
        ]));
        assert!(!needs_plan(
            &["run".into(), "native.network.status".into(),]
        ));
        assert!(!needs_plan(&[
            "run".into(),
            "linux.native.network.status".into(),
        ]));
        assert!(!needs_plan(&["list".into()]));
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

    #[test]
    fn every_catalog_action_has_a_privilege_decision() {
        for spec in ACTIONS {
            let args = vec!["run".to_owned(), spec.id.to_owned()];
            let class = crate::privilege::classify("actions", &args);
            if !spec.mutating {
                assert_eq!(
                    class,
                    crate::privilege::ActionPrivilege::ReadOnly,
                    "{} no debe elevarse",
                    spec.id
                );
            } else if matches!(spec.command, "git" | "wine" | "automation" | "aliases") {
                assert_eq!(
                    class,
                    crate::privilege::ActionPrivilege::Never,
                    "{} debe conservar la identidad del usuario",
                    spec.id
                );
            } else {
                assert_ne!(
                    class,
                    crate::privilege::ActionPrivilege::ReadOnly,
                    "{} no tiene protección de elevación",
                    spec.id
                );
            }
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
