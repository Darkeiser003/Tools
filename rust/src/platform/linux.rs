use std::fs;
use std::io;
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
        .map(|path| std::env::split_paths(&path).any(|dir| dir.join(name).is_file()))
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
            .map(|line| line.trim().chars().take(240).collect())
    })
}

pub fn run_with_privilege(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    if dry_run {
        return run_command(program, args, true);
    }
    if geteuid() == 0 {
        return run_command(program, args, false);
    }
    if command_exists("sudo") {
        let mut sudo_args = vec![program.to_string()];
        sudo_args.extend_from_slice(args);
        return run_command("sudo", &sudo_args, false);
    }
    eprintln!("Se necesita sudo para esta operación.");
    Ok(false)
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
    let text = path.to_string_lossy();
    matches!(
        text.as_ref(),
        "/" | "/home" | "/mnt" | "/media" | "/opt" | "/usr" | "/var" | "/etc" | "/boot" | "/run"
    ) || is_mount_root(path)
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
        false,
        "",
    ),
    tool(
        "gparted",
        "storage",
        "optional-graphical-disk-tool",
        false,
        false,
        "",
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
        "blkid",
        "storage",
        "filesystem-identification",
        false,
        false,
        "",
    ),
    tool(
        "udisksctl",
        "storage",
        "desktop-disk-control",
        false,
        false,
        "",
    ),
    tool("btrfs", "storage", "btrfs-management", false, false, ""),
    tool(
        "cryptsetup",
        "storage",
        "encrypted-volume-management",
        false,
        false,
        "",
    ),
    tool("lvs", "storage", "lvm-volume-inventory", false, false, ""),
    tool("zpool", "storage", "zfs-pool-inventory", false, false, ""),
    tool(
        "docker",
        "containers",
        "docker-engine-detected",
        false,
        false,
        "",
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
        false,
        "",
    ),
    tool(
        "podman-compose",
        "containers",
        "alternative-compose",
        false,
        false,
        "",
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
        false,
        "",
    ),
    tool(
        "crictl",
        "containers",
        "kubernetes-container-runtime-client",
        false,
        false,
        "",
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
        false,
        "",
    ),
    tool(
        "kubelet",
        "kubernetes",
        "kubernetes-node-agent",
        false,
        false,
        "",
    ),
    tool(
        "helm",
        "kubernetes",
        "kubernetes-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "kind",
        "kubernetes",
        "kubernetes-local-clusters",
        false,
        false,
        "",
    ),
    tool(
        "minikube",
        "kubernetes",
        "kubernetes-local-clusters",
        false,
        false,
        "",
    ),
    tool(
        "k3d",
        "kubernetes",
        "kubernetes-local-clusters",
        false,
        false,
        "",
    ),
    tool(
        "k9s",
        "kubernetes",
        "kubernetes-terminal-client",
        false,
        false,
        "",
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
        "flatpak",
        "package-manager",
        "flatpak-package-manager",
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
    args.push(tool.install_package.into());
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
    if !crate::common::ask("¿Instalar esta dependencia ahora?") {
        println!("Instalación cancelada; no se modifica el sistema.");
        return Ok(false);
    }
    let ok = crate::platform::run_with_privilege(manager, &args, dry_run)
        .map_err(|error| error.to_string())?;
    // En una simulación no se instala nada, así que la herramienta seguirá
    // ausente. El resultado correcto es que el plan se pudo ejecutar.
    Ok(ok && (dry_run || host_tool_available(tool)))
}

pub fn fuse_available() -> bool {
    let device = fs::metadata("/dev/fuse")
        .map(|metadata| {
            use std::os::unix::fs::FileTypeExt;
            metadata.file_type().is_char_device()
        })
        .unwrap_or(false);
    device && (command_exists("fusermount3") || command_exists("fusermount"))
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
