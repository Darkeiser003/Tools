//! Alias cortos y estables para hosts de terminal.
//!
//! Los alias son una capa declarativa sobre comandos de LTools. No contienen
//! shell ni permiten ejecutar programas arbitrarios: cada uno se traduce a un
//! comando Rust conocido antes de llegar al despachador.

pub fn aliases_for(id: &str) -> &'static [&'static str] {
    match id {
        "audit.quick" | "audit" => &["taudit quick"],
        "packages.inventory" | "packages" => &["tpkg inventory"],
        "games.inventory" | "games" => &["tgame inventory"],
        "storage.overview" => &["tdisk status"],
        "storage.partitions" => &["tdisk partitions"],
        "storage.mounts" => &["tdisk mounts"],
        "storage.usage" => &["tdisk usage"],
        "storage.filesystems" => &["tdisk filesystems"],
        "storage.volume-stack" => &["tdisk stack"],
        "storage.block-device" => &["tdisk blockdev"],
        "storage.inspect" => &["tdisk inspect"],
        "storage.mount" => &["tdisk mount"],
        "storage.unmount" => &["tdisk unmount"],
        "system.health" => &["tsvc health"],
        "system.services" => &["tsvc list"],
        "system.service-status" => &["tsvc status"],
        "system.service-manage" => &["tsvc manage"],
        "system.processes" | "system.process-status" => &["tproc list"],
        "system.journal" | "system.events" => &["tjournal recent"],
        "system.dependencies" => &["tsvc dependencies"],
        "accounts.list" => &["tuser list"],
        "accounts.sessions" => &["tuser sessions"],
        "accounts.inspect" => &["tuser inspect"],
        "accounts.add" => &["tuser add"],
        "accounts.enable" => &["tuser enable"],
        "accounts.disable" => &["tuser disable"],
        "accounts.lock" => &["tuser lock"],
        "accounts.delete" => &["tuser delete"],
        "defaults.show" => &["tpaths show"],
        "clean.preview" => &["tclean preview"],
        "diagnostics.health" => &["tdiag health"],
        "automation.list" => &["tauto list"],
        "native.network-status" => &["tnet status"],
        "native.dns-flush" => &["tnet flush-dns"],
        "native.hardware-status" => &["thw status"],
        "native.power-status" => &["tpower status"],
        "native.security-status" => &["tsecurity status"],
        "boot.status" | "boot-status" => &["tboot status"],
        "boot.plan" | "boot-plan" => &["tboot plan"],
        _ => &[],
    }
}

/// Expande alias CLI en una invocación canónica de LTools.
///
/// Solo se expande cuando el alias aparece como primer argumento; las
/// opciones globales se mantienen explícitas para que no cambie su semántica.
pub fn expand(args: &[String]) -> Option<Vec<String>> {
    let alias = args.first()?.as_str();
    let mut expanded = match alias {
        "taudit" => vec!["audit".into()],
        "tpkg" => vec!["packages".into()],
        "tgame" => vec!["games".into()],
        "tdisk" => vec!["storage".into()],
        "tsvc" => vec!["system".into()],
        "tproc" => vec!["system".into(), "processes".into()],
        "tjournal" => vec!["system".into(), "journal".into()],
        "tuser" => vec!["accounts".into()],
        "tpaths" => vec!["defaults".into()],
        "tclean" => vec!["clean".into()],
        "tdiag" => vec!["diagnostics".into()],
        "tauto" => vec!["automation".into()],
        "tnet" => vec!["native".into(), "network".into()],
        "thw" => vec!["native".into(), "hardware".into()],
        "tpower" => vec!["native".into(), "power".into()],
        "tsecurity" => vec!["native".into(), "security".into()],
        "tboot" => vec!["boot".into()],
        _ => return None,
    };
    expanded.extend_from_slice(&args[1..]);
    Some(expanded)
}

pub fn json(id: &str) -> String {
    let values = aliases_for(id)
        .iter()
        .map(|value| format!("\"{}\"", escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{aliases_for, expand};

    #[test]
    fn aliases_are_stable_and_non_empty_for_core_actions() {
        for id in ["storage.overview", "system.services", "boot.status"] {
            assert!(!aliases_for(id).is_empty());
        }
    }

    #[test]
    fn expands_only_known_aliases_and_keeps_arguments() {
        let input = vec!["tdisk".into(), "partitions".into(), "--format=json".into()];
        assert_eq!(
            expand(&input).unwrap(),
            ["storage", "partitions", "--format=json"]
        );
        assert!(expand(&["unknown".into()]).is_none());
    }

    #[test]
    fn descriptor_ids_and_catalog_ids_share_boot_aliases() {
        assert_eq!(aliases_for("boot.status"), aliases_for("boot-status"));
        assert_eq!(aliases_for("boot.plan"), aliases_for("boot-plan"));
    }
}
