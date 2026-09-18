//! Clasificación conservadora de privilegios para acciones de LTools.
//!
//! Una consulta nunca necesita elevarse. Las operaciones sobre el sistema o
//! sobre ficheros protegidos pueden ofrecer elevación. Git, Wine/Proton,
//! automatizaciones y la papelera del usuario deben conservar la identidad
//! del usuario y no se relanzan como root/Administrador.

use crate::cli_args::{option_value, positionals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActionPrivilege {
    ReadOnly,
    Optional,
    Required,
    Never,
}

fn default_action(command: &str) -> &'static str {
    match command {
        "boot" | "bootloader" | "efi" | "accounts" | "users" | "user-management" | "storage"
        | "disks" | "partitions" | "native" | "native-tools" => "menu",
        "system" | "services" | "systemctl" | "registry" | "records" => "status",
        "snapshots" | "snapshot" | "restore-points" => "status",
        "software" => "search",
        _ => "",
    }
}

fn next_action<'a>(positionals: &[&'a str], parent: &str) -> Option<&'a str> {
    positionals
        .iter()
        .position(|value| *value == parent)
        .and_then(|index| positionals.get(index + 1).copied())
}

fn native_is_read_only(args: &[String]) -> bool {
    let positionals = positionals(args);
    let Some(area) = positionals.first().copied() else {
        // `native` with no arguments opens the interactive menu.
        return true;
    };
    let action = next_action(&positionals, area);
    match area {
        "menu" => true,
        "network" | "net" => {
            action.is_none()
                || matches!(
                    action,
                    Some(
                        "menu"
                            | "status"
                            | "overview"
                            | "interfaces"
                            | "ifaces"
                            | "routes"
                            | "route"
                            | "dns"
                            | "listening"
                            | "ports"
                            | "listeners"
                            | "connections"
                            | "nmcli"
                    )
                )
        }
        "hardware" | "hw" => action.is_none() || matches!(action, Some("status" | "overview")),
        "security" | "firewall" => {
            action.is_none()
                || matches!(
                    action,
                    Some("menu" | "status" | "overview" | "scanners" | "code-scanners")
                )
        }
        "power" | "energy" => {
            action.is_none() || matches!(action, Some("menu" | "status" | "overview" | "plans"))
        }
        "containers" | "container" | "kubernetes" | "k8s" | "cluster" => {
            action.is_none() || matches!(action, Some("menu" | "status" | "overview"))
        }
        "utilities" | "utility" => {
            action.is_none() || matches!(action, Some("menu" | "status" | "list" | "overview"))
        }
        "tools" | "devtools" | "tooling" => match action {
            None
            | Some(
                "status"
                | "list"
                | "overview"
                | "menu"
                | "ssh"
                | "remote"
                | "adb"
                | "android"
                | "containers"
                | "container"
                | "images"
                | "image"
                | "volumes"
                | "volume"
                | "networks"
                | "network"
                | "compose"
                | "kubernetes"
                | "k8s"
                | "utilities"
                | "utility"
                | "adb-status"
                | "container-list"
                | "container-inspect"
                | "container-logs"
                | "container-stats"
                | "container-top"
                | "container-port"
                | "container-diff"
                | "image-inspect"
                | "image-history"
                | "volume-list"
                | "volume-inspect"
                | "network-list"
                | "network-inspect"
                | "system-info"
                | "system-df"
                | "kubernetes-contexts",
            ) => true,
            Some("container-compose") => {
                option_value(args, "--operation").is_some_and(|operation| {
                    matches!(operation, "ps" | "logs" | "config" | "images" | "top")
                })
            }
            _ => false,
        },
        _ => false,
    }
}

fn native_action_keeps_user_context(positionals: &[&str]) -> bool {
    let Some(area) = positionals.first().copied() else {
        return false;
    };
    let action = next_action(positionals, area).unwrap_or_default();
    match area {
        // Estos subcomandos usan el contexto elegido por el usuario (engine,
        // kubeconfig, SSH keys, adb authorization); relanzar LTools como root
        // puede cambiar de daemon, credenciales o dispositivo autorizado.
        "containers" | "container" | "kubernetes" | "k8s" | "cluster" => true,
        "tools" | "devtools" | "tooling" => {
            matches!(action, "ssh-connect" | "scp-copy" | "sftp")
                || action.starts_with("adb-")
                || action.starts_with("container-")
                || action.starts_with("image-")
                || action.starts_with("volume-")
                || action.starts_with("network-")
                || action.starts_with("kubernetes-")
                || action == "system-prune"
        }
        _ => false,
    }
}

pub(crate) fn classify(command: &str, args: &[String]) -> ActionPrivilege {
    let positionals = positionals(args);
    let action = positionals
        .first()
        .copied()
        .unwrap_or_else(|| default_action(command));
    // Abrir un menú nunca debe relanzar toda la interfaz como root. La
    // elevación se decide de nuevo en la acción concreta que se elija dentro
    // del menú.
    if positionals.first() == Some(&"menu") {
        return ActionPrivilege::ReadOnly;
    }
    // Mostrar ayuda nunca debe abrir sudo/UAC, aunque la orden pertenezca a
    // una familia que también contiene operaciones modificadoras.
    if args
        .iter()
        .any(|value| matches!(value.as_str(), "--help" | "-h"))
        || positionals.first() == Some(&"help")
    {
        return ActionPrivilege::ReadOnly;
    }
    // Native menus, inventories, and status commands must remain in the
    // caller's context even when the global preference requests elevation.
    if matches!(command, "native" | "native-tools") && native_is_read_only(args) {
        return ActionPrivilege::ReadOnly;
    }
    if matches!(command, "native" | "native-tools")
        && native_action_keeps_user_context(&positionals)
    {
        return ActionPrivilege::Never;
    }
    if command == "software" && matches!(action, "install" | "add") {
        // Discovery, candidate selection, and confirmation belong to the
        // user's session. software::install_command elevates only the chosen
        // native package-manager process after confirmation.
        return ActionPrivilege::Never;
    }
    if matches!(command, "actions" | "action-catalog") {
        let id = next_action(&positionals, "run");
        let Some(id) = id else {
            return ActionPrivilege::ReadOnly;
        };
        if !crate::actions::needs_plan(args) {
            return ActionPrivilege::ReadOnly;
        }
        // El catálogo puede delegar en familias que deliberadamente
        // conservan la identidad del usuario, aunque estén marcadas como
        // mutadoras por su efecto en el prefijo o el repositorio.
        if id.starts_with("wine.")
            || id.starts_with("git.")
            || id.starts_with("automation.")
            || id.starts_with("aliases.")
        {
            return ActionPrivilege::Never;
        }
        return ActionPrivilege::Required;
    }
    let mutating = matches!(
        command,
        "clean"
            | "cleanup"
            | "storage"
            | "disks"
            | "partitions"
            | "system"
            | "services"
            | "systemctl"
            | "boot"
            | "bootloader"
            | "efi"
            | "accounts"
            | "users"
            | "user-management"
            | "native"
            | "native-tools"
            | "software"
            | "package-install"
            | "install-package"
            | "packages"
            | "registry"
            | "records"
            | "snapshots"
            | "snapshot"
            | "restore-points"
    );
    if !mutating {
        return if matches!(
            command,
            "git"
                | "git-tools"
                | "prefix"
                | "wine"
                | "automation"
                | "automations"
                | "import"
                | "winslim"
                | "wtools"
                | "update"
                | "self-update"
        ) {
            ActionPrivilege::Never
        } else {
            ActionPrivilege::ReadOnly
        };
    }

    match command {
        "git" | "git-tools" | "prefix" | "wine" | "automation" | "automations" | "import"
        | "aliases" | "alias" | "winslim" | "wtools" => ActionPrivilege::Never,
        "storage" | "disks" | "partitions" => match action {
            "menu" | "map" | "tree" | "paths" | "status" | "disks" | "overview" | "partitions"
            | "partition" | "mounts" | "mountpoints" | "usage" | "space" | "inodes"
            | "filesystems" | "filesystem-info" | "uuid" | "labels" | "tools" | "explain"
            | "inspect" | "details" | "health" | "smart" | "check" | "filesystem-check"
            | "volume-stack" | "pools" | "storage-pools" | "bitlocker" | "encryption" | "lvm"
            | "btrfs" | "zfs" | "raid" | "blockdev" | "block-device" | "partition-table"
            | "partition-inspect" | "guide" | "partition-guide" | "diskpart-guide" => {
                if args.iter().any(|value| value == "--elevated") {
                    ActionPrivilege::Optional
                } else {
                    ActionPrivilege::ReadOnly
                }
            }
            "manage" | "files" | "file-manager" => {
                let operation = next_action(&positionals, action).unwrap_or_default();
                match operation {
                    // Trash and external file-manager operations must retain
                    // the caller's desktop session and personal trash.
                    "delete" | "open" => ActionPrivilege::Never,
                    // This only inspects metadata/explanations; elevating it
                    // would needlessly relaunch the CLI as root/Administrator.
                    "permissions" => ActionPrivilege::ReadOnly,
                    "copy" | "move" | "zip" | "tar" => ActionPrivilege::Optional,
                    _ => ActionPrivilege::ReadOnly,
                }
            }
            "open"
            | "browse"
            | "open-gparted"
            | "gparted"
            | "partition-manager"
            | "open-disk-management"
            | "disk-management"
            | "open-diskpart"
            | "diskpart" => ActionPrivilege::Never,
            "mount" | "unmount" | "format" | "resize" | "operate" | "operation" | "mklabel"
            | "mkpart" | "mkfs" | "wipefs" | "discard" | "luks" | "mdadm" => {
                ActionPrivilege::Required
            }
            _ => ActionPrivilege::Optional,
        },
        "software" | "package-install" | "install-package" => match action {
            "search" | "find" | "list" | "query" | "stores" | "managers" => {
                ActionPrivilege::ReadOnly
            }
            "install" | "add" => {
                match crate::software::install_manager_needs_process_elevation(option_value(
                    args,
                    "--manager",
                )) {
                    Some(false) => ActionPrivilege::Never,
                    Some(true) if cfg!(windows) => ActionPrivilege::Optional,
                    Some(true) => ActionPrivilege::Required,
                    None => ActionPrivilege::Required,
                }
            }
            _ => ActionPrivilege::ReadOnly,
        },
        "packages" => {
            if matches!(action, "clean" | "remove" | "uninstall") {
                ActionPrivilege::Required
            } else {
                ActionPrivilege::ReadOnly
            }
        }
        "system" | "services" | "systemctl" => {
            if action == "service" {
                match next_action(&positionals, "service") {
                    Some("status") => return ActionPrivilege::ReadOnly,
                    Some(
                        "start" | "stop" | "restart" | "enable" | "disable" | "mask" | "unmask"
                        | "kill" | "daemon-reload",
                    ) if args.iter().any(|value| value == "--user") => {
                        // A user unit must use the caller's systemd user bus;
                        // sudo would target root's bus and often fail silently.
                        return ActionPrivilege::Never;
                    }
                    _ => {}
                }
            }
            let modifies = positionals.iter().any(|value| {
                matches!(
                    *value,
                    "start"
                        | "stop"
                        | "restart"
                        | "enable"
                        | "disable"
                        | "mask"
                        | "unmask"
                        | "kill"
                        | "daemon-reload"
                )
            });
            if !modifies
                && (matches!(
                    action,
                    "status"
                        | "failed"
                        | "failed-services"
                        | "services"
                        | "user-services"
                        | "processes"
                        | "journal"
                        | "dependencies"
                        | "tree"
                        | "export"
                        | "report"
                        | "health"
                        | "menu"
                        | "doctor"
                ) || positionals.contains(&"status"))
            {
                ActionPrivilege::ReadOnly
            } else {
                ActionPrivilege::Required
            }
        }
        "boot" | "bootloader" | "efi" => {
            if matches!(
                action,
                "status"
                    | "inspect"
                    | "efi-entries"
                    | "efi"
                    | "grub-entries"
                    | "grub"
                    | "systemd-boot"
                    | "bootctl"
                    | "secure-boot"
                    | "secureboot"
                    | "plan"
                    | "preview"
                    | "menu"
            ) {
                ActionPrivilege::ReadOnly
            } else {
                ActionPrivilege::Required
            }
        }
        "accounts" | "users" | "user-management" => match action {
            "list" | "users" | "identity" | "current" | "sessions" | "groups" | "group-list"
            | "inspect" | "details" | "menu" => ActionPrivilege::ReadOnly,
            // Opens the user's MMC session; never start it from an elevated parent process.
            "open-lusrmgr" => ActionPrivilege::Never,
            _ => ActionPrivilege::Required,
        },
        "native" | "native-tools" => {
            let modifies = positionals.iter().any(|value| {
                matches!(
                    *value,
                    "set-interface"
                        | "connection-up"
                        | "connection-down"
                        | "flush-dns"
                        | "dns-flush"
                        | "suspend"
                        | "hibernate"
                        | "reboot"
                        | "shutdown"
                        | "firewall-enable"
                        | "firewall-disable"
                        | "firewall-reload"
                        | "install"
                        | "create"
                        | "remove"
                        | "prune"
                        | "build"
                        | "tag"
                        | "up"
                        | "down"
                        | "start"
                        | "stop"
                        | "restart"
                        | "kill"
                        | "rename"
                        | "cp"
                )
            });
            let reads = positionals.iter().any(|value| {
                matches!(
                    *value,
                    "status"
                        | "interfaces"
                        | "routes"
                        | "dns"
                        | "listening"
                        | "connections"
                        | "inspect"
                        | "stats"
                        | "top"
                        | "port"
                        | "diff"
                        | "ps"
                        | "logs"
                        | "config"
                        | "images"
                        | "list"
                        | "version"
                        | "info"
                        | "usage"
                        | "menu"
                )
            });
            if !modifies && reads {
                ActionPrivilege::ReadOnly
            } else {
                ActionPrivilege::Required
            }
        }
        // La limpieza mezcla papelera, cachés del usuario y operaciones que
        // elevan internamente al gestor correcto. Elevar todo el proceso
        // cambiaría HOME y podría enviar datos personales a la papelera de root.
        "clean" | "cleanup" => ActionPrivilege::Never,
        "snapshots" | "snapshot" | "restore-points" => match action {
            "status" | "list" | "inventory" | "menu" => ActionPrivilege::ReadOnly,
            "create" | "delete" | "restore" | "rollback" => ActionPrivilege::Required,
            _ => ActionPrivilege::Required,
        },
        "registry" | "records" => {
            if matches!(action, "status" | "paths" | "query" | "inspect" | "list") {
                ActionPrivilege::ReadOnly
            } else if matches!(action, "write" | "apply" | "import") {
                ActionPrivilege::Optional
            } else if action == "export" {
                // La exportación crea un respaldo en el perfil del usuario;
                // elevarla produciría un fichero propiedad de root/Admin.
                ActionPrivilege::Never
            } else {
                ActionPrivilege::ReadOnly
            }
        }
        _ => ActionPrivilege::Optional,
    }
}

pub(crate) fn description(class: ActionPrivilege) -> &'static str {
    match class {
        ActionPrivilege::ReadOnly => "consulta; no necesita elevación",
        ActionPrivilege::Optional => "puede ejecutarse con elevación si lo eliges",
        ActionPrivilege::Required => "requiere elevación para modificar el sistema",
        ActionPrivilege::Never => "no se relanza globalmente; conserva la identidad del usuario",
    }
}

pub(crate) fn run() -> Result<(), String> {
    println!("POLÍTICA DE PRIVILEGIOS DE LTOOLS");
    println!(
        "Preferencia actual: {}",
        if crate::gui_preferences::load().elevate_by_default {
            "elevar acciones modificadoras por defecto"
        } else {
            "pedir elevación solo cuando la acción lo exige"
        }
    );
    println!();
    println!(
        "  Consultas, auditorías y mapas normales          no se elevan; el mapa permite reintento"
    );
    println!("  Copiar/mover/archivar y escrituras del sistema   elevación opcional");
    println!(
        "  Discos, montajes, servicios del sistema y boot  elevación requerida cuando hace falta"
    );
    println!("  Software: el gestor decide; Scoop y gestores de usuario conservan tu sesión");
    println!(
        "  Servicios --user, Git/Wine, automatización      conservan la identidad del usuario"
    );
    println!(
        "  Limpieza, papelera y exportaciones personales    conservan la identidad del usuario"
    );
    println!();
    println!("Usa --elevate para una acción concreta o cambia la casilla de Ajustes.");
    println!("Usa --no-elevate para impedir la elevación opcional en esa ejecución.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{classify, ActionPrivilege};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn privilege_catalog_distinguishes_queries_and_mutations() {
        assert_eq!(
            classify("storage", &args(&["map"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("storage", &args(&["help"])),
            ActionPrivilege::ReadOnly
        );
        for operation in ["lvm", "btrfs", "zfs", "raid"] {
            assert_eq!(
                classify("storage", &args(&[operation])),
                ActionPrivilege::ReadOnly,
                "storage {operation} opens an inventory, not a mutating operation"
            );
        }
        assert_eq!(
            classify(
                "storage",
                &args(&["operate", "raid", "--operation", "create"])
            ),
            ActionPrivilege::Required
        );
        assert_eq!(
            classify("winslim", &args(&["launch", "--identity", "system"])),
            ActionPrivilege::Never,
            "the selected NSudo profile is handled explicitly by winslim, not global elevation"
        );
        assert_eq!(
            classify("update", &args(&["download"])),
            ActionPrivilege::Never,
            "downloading a user-scoped update must never elevate the updater process"
        );
        assert_eq!(
            classify("storage", &args(&["manage", "delete"])),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify(
                "storage",
                &args(&["manage", "open", "--path", "/home/alice"])
            ),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify(
                "storage",
                &args(&["manage", "permissions", "--path", "/home/alice"])
            ),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify(
                "storage",
                &args(&[
                    "manage",
                    "copy",
                    "--source",
                    "/tmp/a",
                    "--destination",
                    "/tmp/b"
                ])
            ),
            ActionPrivilege::Optional
        );
        for operation in ["efi", "grub", "bootctl", "secureboot"] {
            assert_eq!(
                classify("boot", &args(&[operation])),
                ActionPrivilege::ReadOnly,
                "boot {operation} is a read-only alias"
            );
        }
        for operation in ["group-list", "current", "details"] {
            assert_eq!(
                classify("accounts", &args(&[operation])),
                ActionPrivilege::ReadOnly,
                "accounts {operation} is a read-only alias"
            );
        }
        assert_eq!(
            classify("accounts", &args(&["open-lusrmgr"])),
            ActionPrivilege::Never
        );
        for operation in ["find", "stores", "managers"] {
            assert_eq!(
                classify("software", &args(&[operation])),
                ActionPrivilege::ReadOnly,
                "software {operation} is read-only"
            );
        }
        assert_eq!(
            classify("software", &args(&["install", "example-package"])),
            ActionPrivilege::Never,
            "keep package discovery and selection in the user's session"
        );
        assert_eq!(
            classify(
                "software",
                &args(&["install", "example-package", "--manager", "not-a-manager"])
            ),
            ActionPrivilege::Never,
            "reject an unsupported manager before any elevation prompt"
        );
        for manager in [
            "apt", "pacman", "flatpak", "paru", "winget", "choco", "scoop",
        ] {
            assert_eq!(
                classify(
                    "software",
                    &args(&["install", "example-package", "--manager", manager])
                ),
                ActionPrivilege::Never,
                "the launcher stays unprivileged for {manager}; the manager is elevated only if needed"
            );
        }
        for operation in [
            "health",
            "failed-services",
            "user-services",
            "tree",
            "report",
        ] {
            assert_eq!(
                classify("system", &args(&[operation])),
                ActionPrivilege::ReadOnly,
                "system {operation} is read-only"
            );
        }
        for operation in [
            "disks",
            "partition",
            "partition-inspect",
            "partition-guide",
            "mountpoints",
            "details",
            "smart",
            "filesystem-check",
            "space",
            "inodes",
            "filesystem-info",
            "uuid",
            "labels",
            "block-device",
            "pools",
            "storage-pools",
            "bitlocker",
            "encryption",
        ] {
            assert_eq!(
                classify("storage", &args(&[operation])),
                ActionPrivilege::ReadOnly,
                "storage {operation} is read-only"
            );
        }
        for operation in [
            "open",
            "browse",
            "gparted",
            "partition-manager",
            "disk-management",
            "diskpart",
        ] {
            assert_eq!(
                classify("storage", &args(&[operation, "--path", "/home/alice"])),
                ActionPrivilege::Never,
                "storage {operation} must stay in the user's desktop session"
            );
        }
        assert_eq!(
            classify(
                "storage",
                &args(&["manage", "--path", "/tmp/target", "delete"])
            ),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify("storage", &args(&["manage", "--path", "menu", "delete"])),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify("storage", &args(&["operate", "mkfs"])),
            ActionPrivilege::Required
        );
        assert_eq!(classify("git", &args(&["push"])), ActionPrivilege::Never);
        assert_eq!(
            classify("clean", &args(&["--path", "/home/alice/Downloads"])),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify("clean", &args(&["--automatic", "--include-personal"])),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify("clean", &args(&["--package-caches"])),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify("registry", &args(&["status"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("registry", &args(&["export"])),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify("native", &args(&["network", "status"])),
            ActionPrivilege::ReadOnly
        );
        for operation in ["ifaces", "route", "ports", "listeners", "nmcli"] {
            assert_eq!(
                classify("native", &args(&["network", operation])),
                ActionPrivilege::ReadOnly,
                "native network {operation} is read-only"
            );
        }
        assert_eq!(
            classify("native", &args(&["network", "dns-flush"])),
            ActionPrivilege::Required
        );
        assert_eq!(
            classify("native", &args(&["network", "menu"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("native", &args(&["security", "menu"])),
            ActionPrivilege::ReadOnly
        );
        for action in ["status", "scanners", "code-scanners"] {
            assert_eq!(
                classify("native", &args(&["security", action])),
                ActionPrivilege::ReadOnly,
                "native security {action} must never elevate"
            );
        }
        for area in [
            "network",
            "hardware",
            "power",
            "security",
            "tools",
            "utilities",
            "containers",
            "kubernetes",
        ] {
            assert_eq!(
                classify("native", &args(&[area])),
                ActionPrivilege::ReadOnly,
                "native {area} defaults to a query or menu"
            );
        }
        assert_eq!(
            classify("native", &args(&["tools"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("native", &args(&["tools", "status"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("native", &args(&["tools", "install"])),
            ActionPrivilege::Required
        );
        assert_eq!(
            classify("native", &args(&["tools", "ssh"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("native", &args(&["tools", "container-list"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("native", &args(&["tools", "container-run"])),
            ActionPrivilege::Never
        );
        for command in [
            "boot", "accounts", "storage", "native", "system", "software",
        ] {
            assert_eq!(
                classify(command, &[]),
                ActionPrivilege::ReadOnly,
                "default action for {command} must not elevate"
            );
        }
        assert_eq!(
            classify("storage", &args(&["manage", "explain", "--path", "/tmp"])),
            ActionPrivilege::ReadOnly,
            "explanations inspect metadata and must not relaunch as administrator"
        );
        assert_eq!(
            classify("storage", &args(&["menu"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("actions", &args(&["run", "native.dns-flush"])),
            ActionPrivilege::Required
        );
        // Option values are data, not actions that may weaken elevation.
        assert_eq!(
            classify(
                "native",
                &args(&["tools", "container-remove", "--name", "menu"])
            ),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify(
                "native",
                &args(&["tools", "container-run", "--image", "help"])
            ),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify(
                "native",
                &args(&["tools", "container-compose", "--operation", "ps"])
            ),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify(
                "native",
                &args(&["tools", "container-compose", "--operation", "down"])
            ),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify(
                "native",
                &args(&["tools", "container-compose", "--operation=logs"])
            ),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("system", &args(&["stop", "--unit", "status"])),
            ActionPrivilege::Required
        );
        assert_eq!(
            classify("system", &args(&["stop", "--unit", "menu"])),
            ActionPrivilege::Required
        );
        assert_eq!(
            classify("system", &args(&["services", "--filter", "restart"])),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("boot", &args(&["set-default", "menu"])),
            ActionPrivilege::Required
        );
        assert_eq!(
            classify(
                "system",
                &args(&["service", "restart", "--user", "example.service"])
            ),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify(
                "system",
                &args(&["--user", "service", "restart", "example.service"])
            ),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify(
                "system",
                &args(&["service", "status", "--user", "example.service"])
            ),
            ActionPrivilege::ReadOnly
        );
        assert_eq!(
            classify("registry", &args(&["write", "--value", "menu"])),
            ActionPrivilege::Optional
        );
        assert_eq!(
            classify("storage", &args(&["map", "--path", "/mnt/help"])),
            ActionPrivilege::ReadOnly
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn action_catalog_preserves_user_identity_for_wine() {
        assert_eq!(
            classify("actions", &args(&["run", "wine.create", "/tmp/prefix"])),
            ActionPrivilege::Never
        );
    }

    #[test]
    fn remote_device_engine_and_cluster_actions_keep_the_callers_context() {
        for action in [
            "ssh-connect",
            "scp-copy",
            "sftp",
            "adb-install",
            "adb-reboot",
            "container-run",
            "container-remove",
            "container-compose",
            "image-prune",
            "volume-remove",
            "network-remove",
            "kubernetes-apply",
            "kubernetes-delete",
            "system-prune",
        ] {
            assert_eq!(
                classify("native", &args(&["tools", action])),
                ActionPrivilege::Never,
                "native tools {action} must retain the selected user context"
            );
        }
        assert_eq!(
            classify("native", &args(&["containers", "remove"])),
            ActionPrivilege::Never
        );
        assert_eq!(
            classify("native", &args(&["kubernetes", "apply"])),
            ActionPrivilege::Never
        );
    }
}
