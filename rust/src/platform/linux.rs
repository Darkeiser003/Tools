use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

pub fn timestamp() -> String {
    command_output("date", &["+%Y%m%d-%H%M%S"]).unwrap_or_else(|| std::process::id().to_string())
}

pub fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|path| {
            std::env::split_paths(&path).any(|dir| {
                let candidate = dir.join(name);
                candidate.metadata().is_ok_and(|metadata| {
                    metadata.is_file() && {
                        use std::os::unix::fs::PermissionsExt;
                        metadata.permissions().mode() & 0o111 != 0
                    }
                })
            })
        })
        .unwrap_or(false)
}

pub fn host_tool_available(tool: &super::HostTool) -> bool {
    if tool.id == "docker-compose" {
        command_exists("docker-compose")
            || (command_exists("docker")
                && command_output("docker", &["compose", "version"]).is_some())
    } else if tool.id == "trash" {
        command_exists("gio") || command_exists("trash-put")
    } else {
        command_exists(tool.command)
    }
}

pub fn host_tool_version(tool: &super::HostTool) -> Option<String> {
    if !host_tool_available(tool) || tool.command.starts_with("Get-") {
        return None;
    }
    // La consulta de capacidades no debe ejecutar Wine ni acciones de
    // limpieza: algunos binarios pueden abrir ventanas, inicializar un
    // prefijo o esperar interacción. Su disponibilidad ya se informa.
    if matches!(
        tool.id,
        "wineboot" | "paccache" | "trash" | "kill" | "gparted"
    ) {
        return None;
    }
    let default_args: &[&str] = match tool.id {
        "docker-compose" | "podman-compose" => &["version"],
        "kubectl" => &["version", "--client"],
        "helm" => &["version", "--short"],
        "udisksctl" => &["--version"],
        _ => &["--version"],
    };
    let (program, args) = if tool.id == "docker-compose" && !command_exists("docker-compose") {
        ("docker", &["compose", "version"][..])
    } else {
        (tool.command, default_args)
    };
    command_output(program, args).and_then(|output| {
        output
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(super::sanitize_version)
    })
}

pub fn run_with_privilege(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    if dry_run {
        return run_command(program, args, true);
    }
    if geteuid() == 0 {
        return run_command(program, args, false);
    }
    // La GUI no tiene un TTY donde `sudo` pueda mostrar su prompt. En ese
    // caso se usa el agente gráfico de polkit; el CLI conserva sudo porque
    // sí dispone de un terminal interactivo para pedir la contraseña.
    let graphical_frontend = std::env::var_os("LTOOLS_FRONTEND")
        .is_some_and(|value| value == "gui")
        && (std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some());
    if graphical_frontend {
        // Algunas sesiones (por ejemplo Hyprland) tienen `sudo` autorizado
        // sin contraseña pero no exponen un agente Polkit compatible con
        // `pkexec`. Probar primero el modo no interactivo evita que una
        // instalación confirmada por la GUI quede bloqueada en una
        // autorización gráfica que nunca llega a mostrar un diálogo.
        if sudo_available_without_prompt() {
            eprintln!("LTools: usando sudo autorizado sin prompt para esta acción gráfica.");
            let result = run_command("sudo", &build_privileged_args(program, args), false);
            if matches!(result, Ok(false)) {
                eprintln!(
                    "sudo no pudo ejecutar la acción. Comprueba la política sudo y los permisos del usuario."
                );
            }
            return result;
        }
        if command_exists("pkexec") {
            let mut pkexec_args = vec![program.to_string()];
            pkexec_args.extend_from_slice(args);
            eprintln!("LTools: solicitando autorización gráfica mediante polkit.");
            let result = run_command("pkexec", &pkexec_args, false);
            if matches!(result, Ok(false)) {
                eprintln!(
                    "La autorización gráfica fue rechazada o no hay un agente polkit activo. "
                );
                eprintln!(
                    "Abre la acción desde una sesión gráfica con polkit/pkexec, o ejecútala desde la CLI con sudo."
                );
            }
            return result;
        }
        eprintln!(
            "No se encontró pkexec; la acción requiere autorización gráfica mediante polkit."
        );
        eprintln!("Puedes intentar instalarlo con: doctor --install pkexec");
        return Ok(false);
    }
    if command_exists("sudo") {
        let mut sudo_args = vec![program.to_string()];
        sudo_args.extend_from_slice(args);
        let result = run_command("sudo", &sudo_args, false);
        if matches!(result, Ok(false)) {
            eprintln!(
                "sudo no ejecutó la operación. Comprueba la contraseña, la política sudo y los permisos del usuario."
            );
        }
        return result;
    }
    eprintln!("Se necesita sudo para esta operación.");
    Ok(false)
}

pub fn is_elevated() -> bool {
    geteuid() == 0
}

pub fn gui_privilege_available() -> bool {
    geteuid() == 0 || sudo_available_without_prompt() || command_exists("pkexec")
}

pub fn run_with_privilege_input(
    program: &str,
    args: &[String],
    input: &[u8],
    dry_run: bool,
) -> io::Result<bool> {
    if dry_run {
        return run_command(program, args, true);
    }
    let (executable, mut command_args) = if geteuid() == 0 {
        (program.to_owned(), args.to_vec())
    } else if std::env::var_os("LTOOLS_FRONTEND").is_some_and(|value| value == "gui") {
        if sudo_available_without_prompt() {
            ("sudo".to_owned(), build_privileged_args(program, args))
        } else if command_exists("pkexec") {
            let mut values = vec![program.to_owned()];
            values.extend_from_slice(args);
            ("pkexec".to_owned(), values)
        } else {
            eprintln!("No hay un agente de autorización gráfica disponible para esta acción.");
            return Ok(false);
        }
    } else {
        // El CLI debe conservar el stdin de la terminal para que sudo/passwd
        // pueda pedir la contraseña del usuario de forma normal.
        return Err(io::Error::other(
            "esta acción requiere una terminal interactiva; usa passwd desde el CLI",
        ));
    };
    // Mantener el mismo formato visible que el resto de acciones privilegiadas.
    if executable == "sudo" || executable == "pkexec" {
        command_args.shrink_to_fit();
    }
    let mut child = Command::new(executable)
        .args(&command_args)
        .stdin(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input)?;
    }
    Ok(child.wait()?.success())
}

fn sudo_available_without_prompt() -> bool {
    command_exists("sudo")
        && Command::new("sudo")
            .args(["-n", "-v"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
}

fn build_privileged_args(program: &str, args: &[String]) -> Vec<String> {
    let mut privileged_args = vec![program.to_string()];
    privileged_args.extend_from_slice(args);
    privileged_args
}

pub fn is_mount_root(path: &Path) -> bool {
    command_output_owned(
        "findmnt",
        &[
            "-rn".into(),
            "-T".into(),
            path.display().to_string(),
            "-o".into(),
            "TARGET".into(),
        ],
    )
    .map(|value| value.trim() == path.display().to_string())
    .unwrap_or(false)
}

pub fn critical_path(path: &Path) -> bool {
    // Las operaciones de limpieza reciben rutas del usuario. Comprobar la
    // forma textual original permitiría que `..` o un enlace ocultasen una
    // raíz protegida; para rutas existentes siempre comparamos su destino
    // canónico.
    let normalized = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = normalized.to_string_lossy();
    matches!(
        text.as_ref(),
        "/" | "/home" | "/mnt" | "/media" | "/opt" | "/usr" | "/var" | "/etc" | "/boot" | "/run"
    ) || is_mount_root(&normalized)
        || text.ends_with("/steamapps")
        || text.ends_with("/compatdata")
        || text.ends_with("/steamapps/common")
        || text.ends_with("/files/share/default_pfx")
}

pub fn move_to_trash(path: &Path, dry_run: bool) -> io::Result<bool> {
    if !path.exists() {
        eprintln!("No existe: {}", path.display());
        return Ok(false);
    }
    if critical_path(path) {
        eprintln!("Bloqueado por seguridad: {}", path.display());
        return Ok(false);
    }
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if dry_run {
        println!("Simulación: se movería a la papelera: {}", path.display());
        return Ok(true);
    }
    if command_exists("gio") {
        return Ok(Command::new("gio")
            .args(["trash", "--"])
            .arg(&path)
            .status()?
            .success());
    }
    if command_exists("trash-put") {
        return Ok(Command::new("trash-put").arg(&path).status()?.success());
    }
    eprintln!("No se encontró gio ni trash-put.");
    Ok(false)
}

pub fn host_tools() -> &'static [super::HostTool] {
    HOST_TOOLS
}

static HOST_TOOLS: &[super::HostTool] = &[
    tool(
        "findmnt",
        "audit",
        "mount-safety-and-discovery",
        false,
        true,
        "util-linux",
    ),
    tool(
        "lsblk",
        "storage",
        "disk-and-partition-inventory",
        false,
        true,
        "util-linux",
    ),
    tool(
        "parted",
        "storage",
        "partition-table-inspection",
        false,
        true,
        "parted",
    ),
    tool(
        "gparted",
        "storage",
        "optional-graphical-disk-tool",
        false,
        true,
        "gparted",
    ),
    tool(
        "xhost",
        "storage",
        "graphical-display-authorization-for-gparted",
        false,
        true,
        "xorg-xhost",
    ),
    tool(
        "pkexec",
        "services",
        "graphical-polkit-authorization",
        false,
        true,
        "polkit",
    ),
    tool(
        "udisksctl",
        "storage",
        "safe-user-mount-and-unmount",
        false,
        true,
        "udisks2",
    ),
    tool(
        "smartctl",
        "storage",
        "SMART-drive-health",
        false,
        true,
        "smartmontools",
    ),
    tool(
        "fsck",
        "storage",
        "read-only-filesystem-check",
        false,
        true,
        "util-linux",
    ),
    tool(
        "sha256sum",
        "audit",
        "duplicate-file-hashes",
        false,
        true,
        "coreutils",
    ),
    tool(
        "rsync",
        "prefix",
        "verified-prefix-migration",
        false,
        true,
        "rsync",
    ),
    tool(
        "wineboot",
        "prefix",
        "create-wine-prefix",
        false,
        true,
        "wine",
    ),
    tool(
        "df",
        "prefix",
        "destination-space-check",
        false,
        true,
        "coreutils",
    ),
    tool(
        "jq",
        "configuration",
        "Heroic-json-rewrite-and-validation",
        false,
        true,
        "jq",
    ),
    tool(
        "perl",
        "configuration",
        "safe-json-rewrite-fallback",
        false,
        true,
        "perl",
    ),
    tool(
        "rg",
        "safety",
        "reference-detection-before-removal",
        false,
        true,
        "ripgrep",
    ),
    tool("trash", "cleanup", "safe-trash-move", false, true, "glib2"),
    tool(
        "paccache",
        "cleanup",
        "pacman-cache-cleanup",
        false,
        true,
        "pacman-contrib",
    ),
    tool(
        "flatpak",
        "cleanup",
        "unused-flatpak-cleanup",
        false,
        true,
        "flatpak",
    ),
    tool(
        "systemctl",
        "system",
        "service-and-daemon-control",
        false,
        false,
        "",
    ),
    tool(
        "journalctl",
        "system",
        "journal-filtering",
        false,
        false,
        "",
    ),
    tool(
        "ip",
        "network",
        "network-addresses-and-routes",
        false,
        true,
        "iproute2",
    ),
    tool(
        "ss",
        "network",
        "listening-sockets",
        false,
        true,
        "iproute2",
    ),
    tool(
        "resolvectl",
        "network",
        "DNS-resolution-status",
        false,
        false,
        "",
    ),
    tool(
        "nmcli",
        "network",
        "NetworkManager-connection-status",
        false,
        true,
        "networkmanager",
    ),
    tool(
        "ssh",
        "remote",
        "secure-remote-shell-client",
        false,
        true,
        "openssh",
    ),
    tool(
        "scp",
        "remote",
        "secure-remote-copy-client",
        false,
        true,
        "openssh",
    ),
    tool(
        "sftp",
        "remote",
        "secure-file-transfer-client",
        false,
        true,
        "openssh",
    ),
    tool(
        "ssh-keygen",
        "remote",
        "SSH-key-generation",
        false,
        true,
        "openssh",
    ),
    tool(
        "ssh-copy-id",
        "remote",
        "SSH-public-key-installation",
        false,
        true,
        "openssh",
    ),
    tool("git", "git", "Git-version-control", false, true, "git"),
    tool("gh", "git", "GitHub-CLI", false, true, "github-cli"),
    tool(
        "adb",
        "mobile",
        "Android-device-bridge",
        false,
        true,
        "android-tools",
    ),
    tool(
        "upower",
        "power",
        "battery-and-power-device-status",
        false,
        true,
        "upower",
    ),
    tool(
        "powerprofilesctl",
        "power",
        "power-profile-status",
        false,
        true,
        "power-profiles-daemon",
    ),
    tool(
        "firewall-cmd",
        "security",
        "firewalld-status",
        false,
        true,
        "firewalld",
    ),
    tool("ufw", "security", "ufw-firewall-status", false, true, "ufw"),
    tool(
        "nft",
        "security",
        "nftables-firewall-status",
        false,
        true,
        "nftables",
    ),
    tool(
        "uname",
        "hardware",
        "kernel-and-architecture",
        false,
        true,
        "coreutils",
    ),
    tool(
        "lscpu",
        "hardware",
        "CPU-inventory",
        false,
        true,
        "util-linux",
    ),
    tool(
        "free",
        "hardware",
        "memory-inventory",
        false,
        true,
        "procps-ng",
    ),
    tool(
        "lspci",
        "hardware",
        "PCI-device-inventory",
        false,
        true,
        "pciutils",
    ),
    tool(
        "lsusb",
        "hardware",
        "USB-device-inventory",
        false,
        true,
        "usbutils",
    ),
    tool("who", "users", "logged-in-users", false, true, "util-linux"),
    tool(
        "loginctl",
        "users",
        "user-session-inventory",
        false,
        false,
        "",
    ),
    tool(
        "systemd-inhibit",
        "power",
        "power-inhibitor-inventory",
        false,
        false,
        "",
    ),
    tool(
        "id",
        "users",
        "current-user-identity",
        false,
        true,
        "coreutils",
    ),
    tool(
        "getent",
        "users",
        "account-and-directory-service-lookup",
        false,
        true,
        "glibc",
    ),
    tool(
        "useradd",
        "users",
        "local-account-creation",
        false,
        false,
        "",
    ),
    tool(
        "usermod",
        "users",
        "local-account-modification",
        false,
        false,
        "",
    ),
    tool(
        "userdel",
        "users",
        "local-account-removal",
        false,
        false,
        "",
    ),
    tool(
        "ps",
        "system",
        "process-inspection",
        false,
        true,
        "procps-ng",
    ),
    tool(
        "kill",
        "system",
        "process-control",
        false,
        true,
        "procps-ng",
    ),
    tool(
        "fdisk",
        "storage",
        "alternative-partition-inspection",
        false,
        false,
        "",
    ),
    tool(
        "sfdisk",
        "storage",
        "scriptable-partition-management",
        false,
        false,
        "",
    ),
    tool(
        "gdisk",
        "storage",
        "GPT-partition-inspection",
        false,
        false,
        "",
    ),
    tool(
        "blockdev",
        "storage",
        "block-device-information",
        false,
        true,
        "util-linux",
    ),
    tool(
        "blkid",
        "storage",
        "filesystem-identification",
        false,
        false,
        "",
    ),
    tool(
        "btrfs",
        "storage",
        "btrfs-management",
        false,
        true,
        "btrfs-progs",
    ),
    tool(
        "cryptsetup",
        "storage",
        "encrypted-volume-management",
        false,
        true,
        "cryptsetup",
    ),
    tool(
        "pvs",
        "storage",
        "lvm-physical-volume-inventory",
        false,
        true,
        "lvm2",
    ),
    tool(
        "vgs",
        "storage",
        "lvm-volume-group-inventory",
        false,
        true,
        "lvm2",
    ),
    tool(
        "lvs",
        "storage",
        "lvm-volume-inventory",
        false,
        true,
        "lvm2",
    ),
    tool(
        "zpool",
        "storage",
        "zfs-pool-inventory",
        false,
        true,
        "zfs-utils",
    ),
    tool(
        "zfs",
        "storage",
        "zfs-dataset-inventory",
        false,
        true,
        "zfs-utils",
    ),
    tool(
        "mdadm",
        "storage",
        "software-raid-inventory",
        false,
        true,
        "mdadm",
    ),
    // Utilidades que LTools puede ejecutar desde el flujo operativo de
    // almacenamiento. Mantenerlas en el catálogo hace que una ausencia se
    // explique y ofrezca instalación contextual en vez de terminar como
    // "dependencia no gestionada".
    tool(
        "mkfs",
        "storage",
        "filesystem-creation",
        false,
        true,
        "util-linux",
    ),
    tool(
        "mkfs.ext4",
        "storage",
        "ext-filesystem-creation",
        false,
        true,
        "e2fsprogs",
    ),
    tool(
        "mkfs.btrfs",
        "storage",
        "btrfs-filesystem-creation",
        false,
        true,
        "btrfs-progs",
    ),
    tool(
        "mkfs.xfs",
        "storage",
        "xfs-filesystem-creation",
        false,
        true,
        "xfsprogs",
    ),
    tool(
        "mkfs.ntfs",
        "storage",
        "ntfs-filesystem-creation",
        false,
        true,
        "ntfs-3g",
    ),
    tool(
        "mkfs.vfat",
        "storage",
        "fat-filesystem-creation",
        false,
        true,
        "dosfstools",
    ),
    tool(
        "mkfs.exfat",
        "storage",
        "exfat-filesystem-creation",
        false,
        true,
        "exfatprogs",
    ),
    tool(
        "mkswap",
        "storage",
        "swap-formatting",
        false,
        true,
        "util-linux",
    ),
    tool(
        "resize2fs",
        "storage",
        "ext-filesystem-resize",
        false,
        true,
        "e2fsprogs",
    ),
    tool(
        "xfs_admin",
        "storage",
        "xfs-filesystem-label",
        false,
        true,
        "xfsprogs",
    ),
    tool(
        "fatlabel",
        "storage",
        "fat-filesystem-label",
        false,
        true,
        "dosfstools",
    ),
    tool(
        "ntfslabel",
        "storage",
        "ntfs-filesystem-label",
        false,
        true,
        "ntfs-3g",
    ),
    tool(
        "exfatlabel",
        "storage",
        "exfat-filesystem-label",
        false,
        true,
        "exfatprogs",
    ),
    tool(
        "pvcreate",
        "storage",
        "lvm-physical-volume-create",
        false,
        true,
        "lvm2",
    ),
    tool(
        "pvremove",
        "storage",
        "lvm-physical-volume-remove",
        false,
        true,
        "lvm2",
    ),
    tool(
        "vgcreate",
        "storage",
        "lvm-volume-group-create",
        false,
        true,
        "lvm2",
    ),
    tool(
        "vgremove",
        "storage",
        "lvm-volume-group-remove",
        false,
        true,
        "lvm2",
    ),
    tool(
        "lvcreate",
        "storage",
        "lvm-logical-volume-create",
        false,
        true,
        "lvm2",
    ),
    tool(
        "lvremove",
        "storage",
        "lvm-logical-volume-remove",
        false,
        true,
        "lvm2",
    ),
    tool(
        "lvextend",
        "storage",
        "lvm-logical-volume-extend",
        false,
        true,
        "lvm2",
    ),
    tool(
        "lvreduce",
        "storage",
        "lvm-logical-volume-reduce",
        false,
        true,
        "lvm2",
    ),
    tool(
        "nvme",
        "storage",
        "NVMe-device-inventory",
        false,
        true,
        "nvme-cli",
    ),
    tool(
        "gnome-disks",
        "storage",
        "GNOME-disk-manager",
        false,
        true,
        "gnome-disk-utility",
    ),
    tool(
        "partitionmanager",
        "storage",
        "KDE-disk-manager",
        false,
        true,
        "partitionmanager",
    ),
    tool(
        "duf",
        "storage",
        "readable-space-report",
        false,
        true,
        "duf",
    ),
    tool(
        "ncdu",
        "storage",
        "interactive-space-report",
        false,
        true,
        "ncdu",
    ),
    tool(
        "docker",
        "containers",
        "docker-engine-detected",
        false,
        true,
        "docker",
    ),
    tool(
        "docker-compose",
        "containers",
        "docker-compose-primary-installer",
        false,
        true,
        "docker-compose",
    ),
    tool(
        "podman",
        "containers",
        "alternative-container-engine",
        false,
        true,
        "podman",
    ),
    tool(
        "podman-compose",
        "containers",
        "alternative-compose",
        false,
        true,
        "podman-compose",
    ),
    tool(
        "nerdctl",
        "containers",
        "alternative-container-client",
        false,
        false,
        "",
    ),
    tool(
        "containerd",
        "containers",
        "container-runtime",
        false,
        true,
        "containerd",
    ),
    tool(
        "crictl",
        "containers",
        "kubernetes-container-runtime-client",
        false,
        true,
        "cri-tools",
    ),
    tool(
        "kubectl",
        "kubernetes",
        "kubernetes-primary-client-installer",
        false,
        true,
        "kubectl",
    ),
    tool(
        "kubeadm",
        "kubernetes",
        "kubernetes-cluster-bootstrap",
        false,
        true,
        "kubeadm",
    ),
    tool(
        "kubelet",
        "kubernetes",
        "kubernetes-node-agent",
        false,
        true,
        "kubelet",
    ),
    tool(
        "helm",
        "kubernetes",
        "kubernetes-package-manager",
        false,
        true,
        "helm",
    ),
    tool(
        "kind",
        "kubernetes",
        "kubernetes-local-clusters",
        false,
        true,
        "kind",
    ),
    tool(
        "minikube",
        "kubernetes",
        "kubernetes-local-clusters",
        false,
        true,
        "minikube",
    ),
    tool(
        "k3d",
        "kubernetes",
        "kubernetes-local-clusters",
        false,
        true,
        "k3d",
    ),
    tool(
        "k9s",
        "kubernetes",
        "kubernetes-terminal-client",
        false,
        true,
        "k9s",
    ),
    tool(
        "pacman",
        "package-manager",
        "arch-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "paru",
        "package-manager",
        "aur-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "yay",
        "package-manager",
        "aur-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "pamac",
        "package-manager",
        "graphical-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "apt-get",
        "package-manager",
        "debian-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "apt",
        "package-manager",
        "debian-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "dpkg",
        "package-manager",
        "debian-package-database",
        false,
        false,
        "",
    ),
    tool(
        "dnf",
        "package-manager",
        "fedora-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "yum",
        "package-manager",
        "legacy-rpm-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "zypper",
        "package-manager",
        "suse-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "apk",
        "package-manager",
        "alpine-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "xbps-install",
        "package-manager",
        "void-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "pkg",
        "package-manager",
        "bsd-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "brew",
        "package-manager",
        "homebrew-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "snap",
        "package-manager",
        "snap-package-manager",
        false,
        false,
        "",
    ),
    // Utilidades opcionales: se catalogan para detección e instalación
    // explícita, pero ninguna consulta las instala por sorpresa.
    tool(
        "curl",
        "utilities",
        "HTTP-and-API-client",
        false,
        true,
        "curl",
    ),
    tool(
        "wget",
        "utilities",
        "HTTP-download-client",
        false,
        true,
        "wget",
    ),
    tool(
        "file",
        "utilities",
        "file-type-detection",
        false,
        true,
        "file",
    ),
    tool(
        "tree",
        "utilities",
        "directory-tree-viewer",
        false,
        true,
        "tree",
    ),
    tool(
        "htop",
        "utilities",
        "interactive-process-viewer",
        false,
        true,
        "htop",
    ),
    tool("btop", "utilities", "resource-monitor", false, true, "btop"),
    tool(
        "lsof",
        "utilities",
        "open-files-and-ports",
        false,
        true,
        "lsof",
    ),
    tool(
        "strace",
        "utilities",
        "system-call-tracing",
        false,
        true,
        "strace",
    ),
    tool(
        "tcpdump",
        "utilities",
        "packet-capture",
        false,
        true,
        "tcpdump",
    ),
    tool("dig", "utilities", "DNS-query-client", false, true, "bind"),
    tool(
        "nslookup",
        "utilities",
        "DNS-query-client",
        false,
        true,
        "bind",
    ),
    tool(
        "nmap",
        "utilities",
        "network-discovery",
        false,
        true,
        "nmap",
    ),
    tool(
        "openssl",
        "utilities",
        "TLS-and-cryptography",
        false,
        true,
        "openssl",
    ),
    tool("gpg", "utilities", "OpenPGP-signing", false, true, "gnupg"),
    tool(
        "7z",
        "utilities",
        "archive-management",
        false,
        true,
        "p7zip",
    ),
    tool("unzip", "utilities", "ZIP-extraction", false, true, "unzip"),
    tool("zip", "utilities", "ZIP-creation", false, true, "zip"),
    tool(
        "zstd",
        "utilities",
        "Zstandard-compression",
        false,
        true,
        "zstd",
    ),
    tool(
        "tmux",
        "utilities",
        "terminal-multiplexer",
        false,
        true,
        "tmux",
    ),
    tool(
        "python3",
        "utilities",
        "Python-automation-runtime",
        false,
        true,
        "python",
    ),
    tool(
        "make",
        "development",
        "build-automation",
        false,
        true,
        "make",
    ),
    tool(
        "cmake",
        "development",
        "cross-platform-build-system",
        false,
        true,
        "cmake",
    ),
    tool("gcc", "development", "C-compiler", false, true, "gcc"),
    tool("gdb", "development", "native-debugger", false, true, "gdb"),
    // Almacenamiento avanzado y recuperación. Son herramientas opcionales:
    // se detectan y se pueden instalar bajo demanda, pero nunca se ejecutan
    // automáticamente porque varias admiten operaciones destructivas.
    tool(
        "wipefs",
        "storage",
        "filesystem-signature-management",
        false,
        true,
        "util-linux",
    ),
    tool(
        "blkdiscard",
        "storage",
        "block-device-discard",
        false,
        true,
        "util-linux",
    ),
    tool(
        "sgdisk",
        "storage",
        "GPT-scriptable-partition-management",
        false,
        true,
        "gptfdisk",
    ),
    tool(
        "cgdisk",
        "storage",
        "interactive-GPT-partition-management",
        false,
        true,
        "gptfdisk",
    ),
    tool(
        "partx",
        "storage",
        "kernel-partition-table-update",
        false,
        true,
        "util-linux",
    ),
    tool(
        "kpartx",
        "storage",
        "device-mapper-partition-mapping",
        false,
        true,
        "multipath-tools",
    ),
    tool("lvm", "storage", "LVM-administration", false, true, "lvm2"),
    tool(
        "tune2fs",
        "storage",
        "ext-filesystem-tuning",
        false,
        true,
        "e2fsprogs",
    ),
    tool(
        "e2fsck",
        "storage",
        "ext-filesystem-check",
        false,
        true,
        "e2fsprogs",
    ),
    tool(
        "xfs_info",
        "storage",
        "XFS-filesystem-inspection",
        false,
        true,
        "xfsprogs",
    ),
    tool(
        "xfs_repair",
        "storage",
        "XFS-filesystem-repair",
        false,
        true,
        "xfsprogs",
    ),
    tool(
        "xfs_growfs",
        "storage",
        "XFS-filesystem-growth",
        false,
        true,
        "xfsprogs",
    ),
    tool(
        "ntfsresize",
        "storage",
        "NTFS-filesystem-resize",
        false,
        true,
        "ntfs-3g",
    ),
    tool(
        "ntfsfix",
        "storage",
        "NTFS-filesystem-repair",
        false,
        true,
        "ntfs-3g",
    ),
    tool(
        "fstrim",
        "storage",
        "filesystem-discard-maintenance",
        false,
        true,
        "util-linux",
    ),
    tool(
        "fsfreeze",
        "storage",
        "filesystem-freeze-control",
        false,
        true,
        "util-linux",
    ),
    tool(
        "mount",
        "storage",
        "filesystem-mounting",
        false,
        true,
        "util-linux",
    ),
    tool(
        "umount",
        "storage",
        "filesystem-unmounting",
        false,
        true,
        "util-linux",
    ),
    tool(
        "swapon",
        "storage",
        "swap-activation",
        false,
        true,
        "util-linux",
    ),
    tool(
        "swapoff",
        "storage",
        "swap-deactivation",
        false,
        true,
        "util-linux",
    ),
    tool(
        "testdisk",
        "recovery",
        "partition-and-file-recovery",
        false,
        true,
        "testdisk",
    ),
    tool(
        "photorec",
        "recovery",
        "file-carving-recovery",
        false,
        true,
        "testdisk",
    ),
    tool(
        "ddrescue",
        "recovery",
        "failing-disk-rescue",
        false,
        true,
        "gddrescue",
    ),
    tool(
        "restic",
        "backup",
        "encrypted-deduplicated-backups",
        false,
        true,
        "restic",
    ),
    tool(
        "borg",
        "backup",
        "deduplicated-backups",
        false,
        true,
        "borg",
    ),
    tool(
        "rclone",
        "backup",
        "cloud-and-remote-file-sync",
        false,
        true,
        "rclone",
    ),
    tool(
        "timeshift",
        "backup",
        "system-snapshot-management",
        false,
        true,
        "timeshift",
    ),
    tool(
        "snapper",
        "backup",
        "filesystem-snapshot-management",
        false,
        true,
        "snapper",
    ),
    // Red avanzada y observabilidad de interfaces.
    tool(
        "ethtool",
        "network",
        "Ethernet-interface-diagnostics",
        false,
        true,
        "ethtool",
    ),
    tool(
        "iw",
        "network",
        "WiFi-interface-management",
        false,
        true,
        "iw",
    ),
    tool(
        "bridge",
        "network",
        "Linux-bridge-management",
        false,
        true,
        "iproute2",
    ),
    tool(
        "mtr",
        "network",
        "combined-ping-and-traceroute",
        false,
        true,
        "mtr",
    ),
    tool(
        "iperf3",
        "network",
        "network-throughput-testing",
        false,
        true,
        "iperf3",
    ),
    tool("socat", "network", "socket-relay", false, true, "socat"),
    tool(
        "ncat",
        "network",
        "network-connection-tool",
        false,
        true,
        "nmap",
    ),
    tool(
        "sshfs",
        "network",
        "SSH-filesystem-mounting",
        false,
        true,
        "sshfs",
    ),
    tool(
        "wg",
        "network",
        "WireGuard-management",
        false,
        true,
        "wireguard-tools",
    ),
    tool(
        "openvpn",
        "network",
        "OpenVPN-client",
        false,
        true,
        "openvpn",
    ),
    tool(
        "tailscale",
        "network",
        "mesh-VPN-management",
        false,
        true,
        "tailscale",
    ),
    // Hardware, energía y rendimiento.
    tool(
        "inxi",
        "hardware",
        "human-readable-hardware-report",
        false,
        true,
        "inxi",
    ),
    tool(
        "lshw",
        "hardware",
        "detailed-hardware-inventory",
        false,
        true,
        "lshw",
    ),
    tool(
        "hwinfo",
        "hardware",
        "hardware-detection-report",
        false,
        true,
        "hwinfo",
    ),
    tool(
        "dmidecode",
        "hardware",
        "firmware-DMI-inventory",
        false,
        true,
        "dmidecode",
    ),
    tool(
        "sensors",
        "hardware",
        "temperature-and-voltage-monitoring",
        false,
        true,
        "lm_sensors",
    ),
    tool(
        "powertop",
        "power",
        "power-consumption-analysis",
        false,
        true,
        "powertop",
    ),
    tool(
        "iotop",
        "system",
        "disk-I/O-process-monitoring",
        false,
        true,
        "iotop",
    ),
    tool(
        "iostat",
        "system",
        "CPU-and-I/O-statistics",
        false,
        true,
        "sysstat",
    ),
    tool(
        "pidstat",
        "system",
        "per-process-statistics",
        false,
        true,
        "sysstat",
    ),
    tool("nvtop", "hardware", "GPU-monitoring", false, true, "nvtop"),
    tool(
        "nvidia-smi",
        "hardware",
        "NVIDIA-GPU-management",
        false,
        true,
        "nvidia-utils",
    ),
    tool(
        "radeontop",
        "hardware",
        "AMD-GPU-monitoring",
        false,
        true,
        "radeontop",
    ),
    tool(
        "memtester",
        "hardware",
        "memory-stress-testing",
        false,
        true,
        "memtester",
    ),
    // Herramientas de desarrollo, Git y formatos estructurados.
    tool(
        "yq",
        "utilities",
        "YAML-query-and-editing",
        false,
        true,
        "yq",
    ),
    tool(
        "git-lfs",
        "development",
        "Git-large-file-storage",
        false,
        true,
        "git-lfs",
    ),
    tool(
        "git-filter-repo",
        "development",
        "Git-history-rewriting",
        false,
        true,
        "git-filter-repo",
    ),
    tool(
        "lazygit",
        "development",
        "interactive-Git-client",
        false,
        true,
        "lazygit",
    ),
    tool(
        "delta",
        "development",
        "Git-diff-pager",
        false,
        true,
        "git-delta",
    ),
    tool("glab", "development", "GitLab-CLI", false, true, "glab"),
    tool(
        "node",
        "development",
        "JavaScript-runtime",
        false,
        true,
        "nodejs",
    ),
    tool(
        "npm",
        "development",
        "JavaScript-package-manager",
        false,
        true,
        "npm",
    ),
    tool(
        "pnpm",
        "development",
        "fast-JavaScript-package-manager",
        false,
        true,
        "pnpm",
    ),
    tool(
        "go",
        "development",
        "Go-runtime-and-toolchain",
        false,
        true,
        "go",
    ),
    tool(
        "rustup",
        "development",
        "Rust-toolchain-manager",
        false,
        true,
        "rustup",
    ),
    tool(
        "java",
        "development",
        "Java-runtime",
        false,
        true,
        "jdk-openjdk",
    ),
    tool(
        "mvn",
        "development",
        "Maven-build-tool",
        false,
        true,
        "maven",
    ),
    tool(
        "gradle",
        "development",
        "Gradle-build-tool",
        false,
        true,
        "gradle",
    ),
    tool(
        "valgrind",
        "development",
        "memory-debugger",
        false,
        true,
        "valgrind",
    ),
    tool(
        "perf",
        "development",
        "Linux-performance-profiler",
        false,
        true,
        "perf",
    ),
    tool(
        "bpftrace",
        "development",
        "eBPF-tracing",
        false,
        true,
        "bpftrace",
    ),
    // Construcción, seguridad de imágenes y ecosistema OCI/Kubernetes.
    tool(
        "buildah",
        "containers",
        "OCI-image-building",
        false,
        true,
        "buildah",
    ),
    tool(
        "skopeo",
        "containers",
        "OCI-image-copy-and-inspection",
        false,
        true,
        "skopeo",
    ),
    tool(
        "crun",
        "containers",
        "OCI-container-runtime",
        false,
        true,
        "crun",
    ),
    tool(
        "runc",
        "containers",
        "OCI-reference-runtime",
        false,
        true,
        "runc",
    ),
    tool(
        "ctr",
        "containers",
        "containerd-client",
        false,
        true,
        "containerd",
    ),
    tool(
        "kustomize",
        "kubernetes",
        "Kubernetes-manifest-customization",
        false,
        true,
        "kustomize",
    ),
    tool(
        "helmfile",
        "kubernetes",
        "Helm-release-orchestration",
        false,
        true,
        "helmfile",
    ),
    tool(
        "argocd",
        "kubernetes",
        "GitOps-Kubernetes-client",
        false,
        true,
        "argocd",
    ),
    tool(
        "trivy",
        "security",
        "container-and-dependency-scanner",
        false,
        true,
        "trivy",
    ),
    tool(
        "cosign",
        "security",
        "container-signing-and-verification",
        false,
        true,
        "cosign",
    ),
];

const fn tool(
    command: &'static str,
    category: &'static str,
    feature: &'static str,
    required: bool,
    installable: bool,
    install_package: &'static str,
) -> super::HostTool {
    super::HostTool {
        id: command,
        command,
        category,
        feature,
        required,
        installable,
        install_package,
    }
}

pub fn install_tool(id: &str, dry_run: bool) -> Result<bool, String> {
    let tool = HOST_TOOLS
        .iter()
        .find(|tool| tool.id == id)
        .ok_or_else(|| format!("dependencia no gestionada por LTools: {id}"))?;
    if host_tool_available(tool) {
        println!("Ya disponible: {}", tool.command);
        return Ok(true);
    }
    if !tool.installable || tool.install_package.is_empty() {
        println!(
            "{} no está disponible. No se propone instalarlo automáticamente: depende de la distribución.",
            tool.command
        );
        return Ok(false);
    }
    let manager = [
        ("pacman", vec!["-S".into(), "--needed".into()]),
        ("pamac", vec!["install".into(), "--no-confirm".into()]),
        ("paru", vec!["-S".into(), "--needed".into()]),
        ("yay", vec!["-S".into(), "--needed".into()]),
        ("apt-get", vec!["install".into()]),
        ("apt", vec!["install".into()]),
        ("dnf", vec!["install".into()]),
        ("yum", vec!["install".into()]),
        ("zypper", vec!["install".into()]),
        ("apk", vec!["add".into()]),
        ("xbps-install", vec!["-S".into()]),
    ]
    .into_iter()
    .find(|(manager, _)| command_exists(manager));
    let Some((manager, mut args)) = manager else {
        println!(
            "Falta {} (paquete {}). No se encontró un gestor oficial compatible; no se instalará nada.",
            tool.command, tool.install_package
        );
        return Ok(false);
    };
    let package = install_package_name(id, manager);
    args.push(package.into());
    append_gui_install_flags(manager, &mut args);
    let command_line = format!(
        "{} {}",
        manager,
        args.iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!(
        "Falta {} para {}. Se propone usar {}: {} {}",
        tool.command,
        tool.feature,
        manager,
        manager,
        args.iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    );
    let question = crate::common::dependency_confirmation(
        tool.command,
        tool.feature,
        package,
        manager,
        &command_line,
    );
    if !crate::common::ask(&question) {
        println!(
            "Instalación cancelada para la dependencia «{}» ({}); no se modifica el sistema.",
            tool.command, package
        );
        return Ok(false);
    }
    let ok = if tool_install_uses_privilege_wrapper(manager) {
        crate::platform::run_with_privilege(manager, &args, dry_run)
    } else {
        // Pamac and AUR helpers perform their own narrowly-scoped
        // authorization. Running the frontend itself as root breaks user
        // session access and can make AUR build steps execute as root.
        run_command(manager, &args, dry_run)
    }
    .map_err(|error| error.to_string())?;
    // En una simulación no se instala nada, así que la herramienta seguirá
    // ausente. El resultado correcto es que el plan se pudo ejecutar.
    Ok(ok && (dry_run || host_tool_available(tool)))
}

fn tool_install_uses_privilege_wrapper(manager: &str) -> bool {
    !matches!(manager, "pamac" | "paru" | "yay")
}

fn append_gui_install_flags(manager: &str, args: &mut Vec<String>) {
    if std::env::var_os("LTOOLS_FRONTEND").is_none_or(|value| value != "gui") {
        return;
    }
    args.extend(gui_install_flags(manager).iter().map(|flag| (*flag).into()));
}

fn gui_install_flags(manager: &str) -> &'static [&'static str] {
    match manager {
        "pacman" | "paru" | "yay" => &["--noconfirm"],
        "apt" | "apt-get" | "dnf" | "yum" | "xbps-install" => &["-y"],
        "zypper" => &["--non-interactive"],
        // Pamac already receives its native --no-confirm flag at the call
        // site. --noconfirm is a pacman option and is not a Pamac CLI flag.
        "pamac" => &[],
        _ => &[],
    }
}

fn install_package_name(id: &str, manager: &str) -> &'static str {
    // El nombre del ejecutable no siempre coincide con el paquete y cambia
    // entre familias de distribución. Mantener este mapa aquí evita que una
    // instalación iniciada desde la GUI falle por reutilizar el nombre Arch
    // del catálogo en Debian, Fedora u openSUSE.
    match id {
        "python3" => {
            return match manager {
                "pacman" | "pamac" | "paru" | "yay" => "python",
                "apt" | "apt-get" | "dnf" | "yum" | "zypper" => "python3",
                _ => "python3",
            };
        }
        "dig" | "nslookup" => {
            return match manager {
                "apt" | "apt-get" => "dnsutils",
                "dnf" | "yum" | "zypper" => "bind-utils",
                _ => "bind",
            };
        }
        "7z" => {
            return match manager {
                "apt" | "apt-get" => "p7zip-full",
                "dnf" | "yum" | "zypper" => "p7zip",
                _ => "p7zip",
            };
        }
        "gpg" => {
            return match manager {
                "apt" | "apt-get" => "gnupg",
                "dnf" | "yum" => "gnupg2",
                _ => "gnupg",
            };
        }
        "sensors" => {
            return match manager {
                "apt" | "apt-get" => "lm-sensors",
                _ => "lm_sensors",
            };
        }
        "iostat" | "pidstat" => return "sysstat",
        "ddrescue" => {
            return match manager {
                "apt" | "apt-get" => "gddrescue",
                _ => "ddrescue",
            };
        }
        "sgdisk" | "cgdisk" => return "gptfdisk",
        "kpartx" => {
            return match manager {
                "apt" | "apt-get" => "multipath-tools",
                _ => "multipath-tools",
            };
        }
        "node" => {
            return match manager {
                "apt" | "apt-get" => "nodejs",
                _ => "nodejs",
            };
        }
        "npm" => return "npm",
        "java" => {
            return match manager {
                "apt" | "apt-get" => "default-jdk",
                "dnf" | "yum" => "java-21-openjdk",
                _ => "jdk-openjdk",
            };
        }
        "mvn" => return "maven",
        "nvidia-smi" => {
            return match manager {
                "apt" | "apt-get" => "nvidia-utils-535",
                _ => "nvidia-utils",
            };
        }
        _ => {}
    }
    if id == "xhost" {
        return match manager {
            "apt" | "apt-get" => "x11-xserver-utils",
            "dnf" | "yum" | "zypper" => "xorg-x11-server-utils",
            _ => "xorg-xhost",
        };
    }
    if id == "pkexec" {
        return match manager {
            "apt" | "apt-get" => "policykit-1",
            "dnf" | "yum" | "zypper" => "polkit",
            _ => "polkit",
        };
    }
    HOST_TOOLS
        .iter()
        .find(|tool| tool.id == id)
        .map(|tool| tool.install_package)
        .unwrap_or("")
}

pub fn fuse_prerequisites_detected() -> bool {
    let device = fs::metadata("/dev/fuse")
        .map(|metadata| {
            use std::os::unix::fs::FileTypeExt;
            metadata.file_type().is_char_device()
        })
        .unwrap_or(false);
    device && (command_exists("fusermount3") || command_exists("fusermount"))
}

#[allow(dead_code)]
pub fn winslim_root() -> Option<PathBuf> {
    None
}

pub fn nsudo_path() -> Option<PathBuf> {
    None
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()
    })
}

fn command_output_owned(program: &str, args: &[String]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()
    })
}

fn run_command(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    println!(
        "  $ {} {}",
        program,
        args.iter()
            .map(|arg| shell_display(arg))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if dry_run {
        return Ok(true);
    }
    Ok(Command::new(program).args(args).status()?.success())
}

fn shell_display(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ". /_:@%+-".contains(c))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn geteuid() -> u32 {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|value| {
            value
                .lines()
                .find(|line| line.starts_with("Uid:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|uid| uid.parse().ok())
        })
        .unwrap_or(1)
}

#[cfg(test)]
mod privilege_tests {
    use super::{gui_install_flags, tool_install_uses_privilege_wrapper};

    #[test]
    fn user_managers_keep_their_own_authentication_and_build_context() {
        for manager in ["pamac", "paru", "yay"] {
            assert!(!tool_install_uses_privilege_wrapper(manager));
        }
        for manager in ["pacman", "apt", "dnf", "zypper", "apk"] {
            assert!(tool_install_uses_privilege_wrapper(manager));
        }
    }

    #[test]
    fn gui_install_flags_match_each_package_managers_native_cli() {
        assert!(gui_install_flags("pamac").is_empty());
        assert_eq!(gui_install_flags("pacman"), &["--noconfirm"]);
        assert_eq!(gui_install_flags("apt-get"), &["-y"]);
        assert_eq!(gui_install_flags("zypper"), &["--non-interactive"]);
    }
}
