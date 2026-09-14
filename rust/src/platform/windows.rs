use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn timestamp() -> String {
    command_output(
        "powershell",
        &["-NoProfile", "-Command", "Get-Date -Format yyyyMMdd-HHmmss"],
    )
    .filter(|value| !value.trim().is_empty())
    .or_else(|| {
        command_output(
            "pwsh",
            &["-NoProfile", "-Command", "Get-Date -Format yyyyMMdd-HHmmss"],
        )
        .filter(|value| !value.trim().is_empty())
    })
    .unwrap_or_else(|| std::process::id().to_string())
}

pub fn command_exists(name: &str) -> bool {
    let path = match std::env::var_os("PATH") {
        Some(path) => path,
        None => return false,
    };
    let extensions = std::env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
    std::env::split_paths(&path).any(|dir| {
        dir.join(name).is_file()
            || extensions
                .to_string_lossy()
                .split(';')
                .any(|ext| dir.join(format!("{name}{ext}")).is_file())
    })
}

pub fn host_tool_available(tool: &super::HostTool) -> bool {
    // Wine puede exponer nombres de Windows integrados aunque no tenga
    // PowerShell real. Intentar cada cmdlet con `Get-Command` en ese entorno
    // puede quedar esperando indefinidamente; el ejecutable nativo debe
    // informar esos cmdlets como no disponibles y conservar la validación
    // completa en Windows real.
    if running_under_wine() && (tool.command.starts_with("Get-") || tool.id == "powershell") {
        return false;
    }
    if tool.id == "nsudo" {
        return nsudo_path().is_some();
    }
    if tool.id == "docker-compose" {
        return command_exists("docker-compose")
            || (command_exists("docker")
                && command_output("docker", &["compose", "version"]).is_some());
    }
    if tool.id == "trash" {
        return command_exists("powershell") || command_exists("pwsh");
    }
    if tool.id == "powershell" {
        return ["powershell", "pwsh"].iter().any(|shell| {
            command_exists(shell)
                && command_output(
                    shell,
                    &[
                        "-NoProfile",
                        "-NonInteractive",
                        "-Command",
                        "$PSVersionTable.PSVersion.ToString()",
                    ],
                )
                .is_some_and(|version| !version.trim().is_empty())
        });
    }
    if !tool.command.starts_with("Get-") {
        return command_exists(tool.command);
    }
    let shell = if command_exists("powershell") {
        "powershell"
    } else if command_exists("pwsh") {
        "pwsh"
    } else {
        return false;
    };
    command_output(
        shell,
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "Get-Command {} -ErrorAction Stop | Select-Object -ExpandProperty Name",
                tool.command
            ),
        ],
    )
    .is_some_and(|name| !name.trim().is_empty())
}

pub fn host_tool_version(tool: &super::HostTool) -> Option<String> {
    // Los ejecutables integrados de Wine (por ejemplo sc.exe) pueden no
    // terminar al recibir una opción de versión. El contrato bajo Wine se
    // valida por disponibilidad y esquema; las versiones se consultan en
    // Windows nativo, donde las herramientas sí tienen su comportamiento
    // documentado.
    if running_under_wine() {
        return None;
    }
    if !host_tool_available(tool) || tool.command.starts_with("Get-") {
        return None;
    }
    if matches!(
        tool.id,
        "diskmgmt.msc"
            | "lusrmgr.msc"
            | "services.msc"
            | "eventvwr.msc"
            | "taskmgr.exe"
            | "perfmon.exe"
            | "msinfo32.exe"
    ) {
        return None;
    }
    let default_args: &[&str] = match tool.id {
        "docker-compose" | "podman-compose" => &["version"],
        "kubectl" => &["version", "--client"],
        "helm" => &["version", "--short"],
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
    println!(
        "  > {} ({} argumento(s); valores no mostrados)",
        program,
        args.len()
    );
    if dry_run {
        return Ok(true);
    }
    if is_elevated() {
        return Ok(Command::new(program).args(args).status()?.success());
    }
    // NSudo solo se usa tras una selección explícita de backend para esta
    // sesión; las acciones normales siguen solicitando la elevación UAC.
    if std::env::var_os("LTOOLS_USE_NSUDO").is_some_and(|value| value == "1") {
        if let Some(nsudo) = nsudo_path() {
            let mut nsudo_args = vec![
                "-U:E".into(),
                "-Wait".into(),
                "-UseCurrentConsole".into(),
                program.to_string(),
            ];
            nsudo_args.extend_from_slice(args);
            println!(
                "  > {} -U:E -Wait -UseCurrentConsole {} ({} argumento(s) omitidos)",
                nsudo.display(),
                program,
                args.len()
            );
            return Ok(Command::new(nsudo).args(nsudo_args).status()?.success());
        }
        eprintln!("Se solicitó NSudo para esta sesión, pero no se encontró en WinSlim ni en PATH.");
        return Ok(false);
    }
    let shell = if command_exists("powershell") {
        "powershell"
    } else if command_exists("pwsh") {
        "pwsh"
    } else {
        return Ok(false);
    };
    // Start-Process combina los elementos de -ArgumentList en una sola línea
    // de comandos. Cada argumento debe ir citado según las reglas de Windows;
    // citarlo solo como literal PowerShell pierde los límites cuando hay
    // espacios y puede cambiar el destino o las opciones de la acción.
    let arguments = args
        .iter()
        .map(|value| crate::common::windows_command_line_argument(value))
        .collect::<Vec<_>>()
        .join(" ");
    let script = format!(
        "$p=Start-Process -FilePath {} -ArgumentList {} -Verb RunAs -Wait -PassThru; exit $p.ExitCode",
        crate::common::powershell_single_quoted(program),
        crate::common::powershell_single_quoted(&arguments)
    );
    Ok(Command::new(shell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .status()?
        .success())
}

pub fn critical_path(path: &Path) -> bool {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let normalized = canonical
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();
    let trimmed = normalized.trim_end_matches('/');
    trimmed.is_empty()
        || normalized.ends_with(":/")
        || trimmed.ends_with(':')
        || [
            "/windows",
            "/program files",
            "/program files (x86)",
            "/programdata",
            "/users",
        ]
        .iter()
        .any(|suffix| trimmed.ends_with(suffix))
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
    if dry_run {
        println!("Simulación: se movería a la papelera: {}", path.display());
        return Ok(true);
    }
    let shell = if command_exists("powershell") {
        "powershell"
    } else if command_exists("pwsh") {
        "pwsh"
    } else {
        eprintln!("No se encontró PowerShell; el origen se conserva.");
        return Ok(false);
    };
    let escaped = path.to_string_lossy().replace('\'', "''");
    let script = format!(
        r#"Add-Type -AssemblyName Microsoft.VisualBasic; $p='{escaped}'; if ([IO.Directory]::Exists($p)) {{ [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory($p, [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs, [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin) }} elseif ([IO.File]::Exists($p)) {{ [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile($p, [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs, [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin) }} else {{ exit 2 }}; if (Test-Path -LiteralPath $p) {{ exit 1 }}"#
    );
    let status = Command::new(shell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
        ])
        .arg(script)
        .status()?;
    Ok(status.success() && !path.exists())
}

pub fn host_tools() -> &'static [super::HostTool] {
    HOST_TOOLS
}

static HOST_TOOLS: &[super::HostTool] = &[
    tool(
        "powershell",
        "system",
        "PowerShell para automatización Windows",
        false,
        false,
        "",
    ),
    tool("sc.exe", "system", "service-control", false, false, ""),
    tool("tasklist", "system", "process-inspection", false, false, ""),
    tool("taskkill", "system", "process-control", false, false, ""),
    tool("wevtutil", "system", "Windows-event-log", false, false, ""),
    tool(
        "ipconfig",
        "network",
        "network-configuration-fallback",
        false,
        false,
        "",
    ),
    tool(
        "route",
        "network",
        "network-route-fallback",
        false,
        false,
        "",
    ),
    tool(
        "netstat",
        "network",
        "listening-sockets-fallback",
        false,
        false,
        "",
    ),
    tool(
        "ssh",
        "remote",
        "OpenSSH-remote-shell-client",
        false,
        true,
        "OpenSSH.Client",
    ),
    tool(
        "scp",
        "remote",
        "OpenSSH-remote-copy-client",
        false,
        true,
        "OpenSSH.Client",
    ),
    tool(
        "sftp",
        "remote",
        "OpenSSH-file-transfer-client",
        false,
        true,
        "OpenSSH.Client",
    ),
    tool("git", "git", "Git-version-control", false, true, "Git.Git"),
    tool("gh", "git", "GitHub-CLI", false, true, "GitHub.cli"),
    tool(
        "adb",
        "mobile",
        "Android-device-bridge",
        false,
        true,
        "Android.PlatformTools",
    ),
    tool(
        "ssh-keygen",
        "remote",
        "OpenSSH-key-generation",
        false,
        true,
        "OpenSSH.Client",
    ),
    tool(
        "ssh-keyscan",
        "remote",
        "OpenSSH-host-key-discovery",
        false,
        true,
        "OpenSSH.Client",
    ),
    tool(
        "curl.exe",
        "utilities",
        "HTTP-and-API-client",
        false,
        false,
        "",
    ),
    tool(
        "tar.exe",
        "utilities",
        "archive-management",
        false,
        false,
        "",
    ),
    tool(
        "where.exe",
        "utilities",
        "executable-discovery",
        false,
        false,
        "",
    ),
    tool(
        "certutil.exe",
        "security",
        "certificate-and-hash-tools",
        false,
        false,
        "",
    ),
    tool(
        "nslookup.exe",
        "network",
        "DNS-query-client",
        false,
        false,
        "",
    ),
    tool(
        "tracert.exe",
        "network",
        "route-diagnostics",
        false,
        false,
        "",
    ),
    tool(
        "robocopy.exe",
        "utilities",
        "resilient-file-copy",
        false,
        false,
        "",
    ),
    tool(
        "wsl.exe",
        "compatibility",
        "Windows-Subsystem-for-Linux",
        false,
        false,
        "",
    ),
    tool(
        "dism.exe",
        "system",
        "Windows-image-and-feature-management",
        false,
        false,
        "",
    ),
    tool(
        "bcdedit.exe",
        "boot",
        "Windows-boot-configuration",
        false,
        false,
        "",
    ),
    tool(
        "msiexec.exe",
        "system",
        "Windows-installer",
        false,
        false,
        "",
    ),
    tool(
        "7z.exe",
        "utilities",
        "archive-management",
        false,
        true,
        "7zip.7zip",
    ),
    tool(
        "powercfg",
        "power",
        "power-plan-management",
        false,
        false,
        "",
    ),
    tool(
        "query",
        "users",
        "logged-in-user-sessions",
        false,
        false,
        "",
    ),
    tool(
        "lusrmgr.msc",
        "users",
        "local-account-manager",
        false,
        false,
        "",
    ),
    tool(
        "Get-LocalUser",
        "users",
        "local-account-inventory",
        false,
        false,
        "",
    ),
    tool(
        "Get-LocalGroup",
        "users",
        "local-group-inventory",
        false,
        false,
        "",
    ),
    tool(
        "Get-NetIPConfiguration",
        "network",
        "network-addresses-and-routes",
        false,
        false,
        "",
    ),
    tool(
        "Get-NetRoute",
        "network",
        "network-routes",
        false,
        false,
        "",
    ),
    tool(
        "Get-DnsClientServerAddress",
        "network",
        "DNS-server-inventory",
        false,
        false,
        "",
    ),
    tool(
        "Get-NetTCPConnection",
        "network",
        "listening-sockets",
        false,
        false,
        "",
    ),
    tool(
        "Get-NetFirewallProfile",
        "security",
        "firewall-profile-status",
        false,
        false,
        "",
    ),
    tool(
        "Get-MpComputerStatus",
        "security",
        "Microsoft-Defender-status",
        false,
        false,
        "",
    ),
    tool(
        "reg.exe",
        "registry",
        "Windows-registry-query-and-export",
        false,
        false,
        "",
    ),
    tool(
        "diskpart",
        "storage",
        "disk-and-partition-control",
        false,
        false,
        "",
    ),
    tool(
        "mountvol",
        "storage",
        "volume-mount-inventory",
        false,
        false,
        "",
    ),
    tool(
        "diskmgmt.msc",
        "storage",
        "native-graphical-disk-manager",
        false,
        false,
        "",
    ),
    tool("Get-Disk", "storage", "disk-inventory", false, false, ""),
    tool(
        "Get-Partition",
        "storage",
        "partition-inventory",
        false,
        false,
        "",
    ),
    tool(
        "Get-Volume",
        "storage",
        "volume-inventory",
        false,
        false,
        "",
    ),
    tool(
        "Get-StoragePool",
        "storage",
        "storage-pool-inventory",
        false,
        false,
        "",
    ),
    tool(
        "Get-VirtualDisk",
        "storage",
        "virtual-disk-inventory",
        false,
        false,
        "",
    ),
    tool(
        "Get-BitLockerVolume",
        "storage",
        "bitlocker-status",
        false,
        false,
        "",
    ),
    tool(
        "Get-CimInstance",
        "system",
        "service-and-process-inventory",
        false,
        false,
        "",
    ),
    tool("trash", "cleanup", "Windows-recycle-bin", false, false, ""),
    tool(
        "winget",
        "package-manager",
        "Windows-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "choco",
        "package-manager",
        "Chocolatey-package-manager",
        false,
        false,
        "",
    ),
    tool(
        "scoop",
        "package-manager",
        "Scoop-package-manager",
        false,
        false,
        "",
    ),
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
        "Docker.DockerCompose",
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
        "kubectl",
        "kubernetes",
        "kubernetes-primary-client-installer",
        false,
        true,
        "Kubernetes.kubectl",
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
        "nsudo",
        "winslim",
        "elevated-user-script-launcher",
        false,
        false,
        "",
    ),
    // Diagnóstico, reparación y recuperación nativos de Windows. Las
    // herramientas integradas se detectan sin instalar nada; las acciones
    // mutables se mantienen en flujos explícitos y con elevación.
    tool(
        "chkdsk.exe",
        "storage",
        "filesystem-check-and-repair",
        false,
        false,
        "",
    ),
    tool(
        "fsutil.exe",
        "storage",
        "filesystem-and-volume-administration",
        false,
        false,
        "",
    ),
    tool(
        "diskshadow.exe",
        "storage",
        "volume-shadow-copy-management",
        false,
        false,
        "",
    ),
    tool(
        "manage-bde.exe",
        "storage",
        "BitLocker-management",
        false,
        false,
        "",
    ),
    tool(
        "repair-bde.exe",
        "recovery",
        "BitLocker-data-recovery",
        false,
        false,
        "",
    ),
    tool(
        "cipher.exe",
        "security",
        "NTFS-encryption-management",
        false,
        false,
        "",
    ),
    tool(
        "defrag.exe",
        "storage",
        "volume-optimization",
        false,
        false,
        "",
    ),
    tool(
        "sfc.exe",
        "system",
        "Windows-system-file-checker",
        false,
        false,
        "",
    ),
    tool(
        "reagentc.exe",
        "recovery",
        "Windows-recovery-environment",
        false,
        false,
        "",
    ),
    tool(
        "pnputil.exe",
        "hardware",
        "driver-package-management",
        false,
        false,
        "",
    ),
    tool(
        "driverquery.exe",
        "hardware",
        "installed-driver-inventory",
        false,
        false,
        "",
    ),
    tool(
        "systeminfo.exe",
        "system",
        "Windows-system-inventory",
        false,
        false,
        "",
    ),
    tool(
        "msinfo32.exe",
        "system",
        "graphical-system-information",
        false,
        false,
        "",
    ),
    tool(
        "perfmon.exe",
        "system",
        "performance-monitor",
        false,
        false,
        "",
    ),
    tool(
        "schtasks.exe",
        "system",
        "scheduled-task-management",
        false,
        false,
        "",
    ),
    tool(
        "gpupdate.exe",
        "system",
        "Group-Policy-refresh",
        false,
        false,
        "",
    ),
    tool(
        "gpresult.exe",
        "system",
        "Group-Policy-report",
        false,
        false,
        "",
    ),
    tool(
        "services.msc",
        "system",
        "graphical-service-manager",
        false,
        false,
        "",
    ),
    tool(
        "eventvwr.msc",
        "system",
        "graphical-event-viewer",
        false,
        false,
        "",
    ),
    tool(
        "taskmgr.exe",
        "system",
        "graphical-process-manager",
        false,
        false,
        "",
    ),
    tool(
        "netsh.exe",
        "network",
        "Windows-network-configuration",
        false,
        false,
        "",
    ),
    tool(
        "arp.exe",
        "network",
        "ARP-cache-management",
        false,
        false,
        "",
    ),
    tool(
        "pathping.exe",
        "network",
        "path-latency-and-loss-diagnostics",
        false,
        false,
        "",
    ),
    tool(
        "getmac.exe",
        "network",
        "network-adapter-MAC-inventory",
        false,
        false,
        "",
    ),
    tool(
        "hostname.exe",
        "network",
        "host-name-query",
        false,
        false,
        "",
    ),
    tool(
        "telnet.exe",
        "network",
        "TCP-connectivity-diagnostic",
        false,
        true,
        "Telnet.Client",
    ),
    tool(
        "icacls.exe",
        "security",
        "NTFS-permission-management",
        false,
        false,
        "",
    ),
    tool(
        "takeown.exe",
        "security",
        "file-ownership-management",
        false,
        false,
        "",
    ),
    tool(
        "auditpol.exe",
        "security",
        "Windows-audit-policy-management",
        false,
        false,
        "",
    ),
    tool(
        "secedit.exe",
        "security",
        "Windows-security-policy-management",
        false,
        false,
        "",
    ),
    tool(
        "vssadmin.exe",
        "backup",
        "volume-shadow-copy-administration",
        false,
        false,
        "",
    ),
    tool(
        "wbadmin.exe",
        "backup",
        "Windows-backup-administration",
        false,
        false,
        "",
    ),
    tool(
        "runas.exe",
        "security",
        "alternate-user-process-launch",
        false,
        false,
        "",
    ),
    tool(
        "python.exe",
        "development",
        "Python-runtime",
        false,
        true,
        "Python.Python.3.12",
    ),
    tool(
        "node.exe",
        "development",
        "JavaScript-runtime",
        false,
        true,
        "OpenJS.NodeJS.LTS",
    ),
    tool(
        "npm.cmd",
        "development",
        "JavaScript-package-manager",
        false,
        true,
        "OpenJS.NodeJS",
    ),
    tool(
        "java.exe",
        "development",
        "Java-runtime",
        false,
        true,
        "EclipseAdoptium.Temurin.21.JDK",
    ),
    tool(
        "dotnet.exe",
        "development",
        ".NET-runtime-and-SDK",
        false,
        true,
        "Microsoft.DotNet.SDK.8",
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
            "{} no está disponible. Es una herramienta integrada de Windows y LTools no instala componentes del sistema.",
            tool.command
        );
        return Ok(false);
    }
    if matches!(id, "ssh" | "scp" | "sftp" | "ssh-keygen" | "ssh-keyscan") {
        let Some(shell) = ["powershell", "pwsh"]
            .into_iter()
            .find(|candidate| command_exists(candidate))
        else {
            println!("No se encontró PowerShell para habilitar OpenSSH Client.");
            return Ok(false);
        };
        let args = [
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0",
        ];
        println!(
            "Falta la dependencia OpenSSH Client para SSH/SCP/SFTP. Se propone habilitarla mediante {}.",
            shell
        );
        let question = crate::common::dependency_confirmation(
            "OpenSSH Client",
            "SSH, SCP y SFTP",
            "OpenSSH.Client~~~~0.0.1.0",
            "Windows",
            &format!(
                "{} -Command Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0",
                shell
            ),
        );
        if !crate::common::ask(&question) {
            println!(
                "Instalación cancelada para la dependencia «OpenSSH Client»; no se modifica el sistema."
            );
            return Ok(false);
        }
        let ok = crate::platform::run_with_privilege(shell, &args.map(str::to_owned), dry_run)
            .map_err(|error| error.to_string())?;
        return Ok(ok && (dry_run || host_tool_available(tool)));
    }
    let manager = ["winget", "choco", "scoop"]
        .into_iter()
        .find(|manager| command_exists(manager));
    let Some(manager) = manager else {
        println!(
            "Falta {}. No se encontró winget, Chocolatey ni Scoop; LTools no instalará un gestor de paquetes para resolverlo.",
            tool.command
        );
        return Ok(false);
    };
    let package = package_for(tool, manager);
    let args = match manager {
        "winget" => vec![
            "install".into(),
            "--id".into(),
            package.into(),
            "-e".into(),
            "--accept-source-agreements".into(),
            "--accept-package-agreements".into(),
        ],
        "choco" => vec![
            "install".into(),
            package.into(),
            "-y".into(),
            "--no-progress".into(),
        ],
        "scoop" => vec!["install".into(), package.into()],
        _ => unreachable!(),
    };
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
    let ok = crate::platform::run_with_privilege(manager, &args, dry_run)
        .map_err(|error| error.to_string())?;
    Ok(ok && (dry_run || host_tool_available(tool)))
}

fn package_for(tool: &super::HostTool, manager: &str) -> &'static str {
    match (tool.id, manager) {
        ("git", "winget") => "Git.Git",
        ("git", "choco") => "git",
        ("git", "scoop") => "git",
        ("gh", "winget") => "GitHub.cli",
        ("gh", "choco") => "gh",
        ("gh", "scoop") => "gh",
        ("docker-compose", "winget") => "Docker.DockerCompose",
        ("docker-compose", "choco") => "docker-compose",
        ("docker-compose", "scoop") => "docker-compose",
        ("kubectl", "winget") => "Kubernetes.kubectl",
        ("kubectl", "choco") => "kubernetes-cli",
        ("kubectl", "scoop") => "kubectl",
        ("7z.exe", "winget") => "7zip.7zip",
        ("7z.exe", "choco") => "7zip",
        ("7z.exe", "scoop") => "7zip",
        _ => tool.install_package,
    }
}

fn running_under_wine() -> bool {
    std::env::var_os("WINEPREFIX").is_some() || std::env::var_os("WINELOADERNOEXEC").is_some()
}

pub fn fuse_prerequisites_detected() -> bool {
    false
}

pub fn winslim_root() -> Option<PathBuf> {
    let root = PathBuf::from(r"C:\WSCore");
    root.is_dir().then_some(root)
}

pub fn nsudo_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LTOOLS_NSUDO_PATH").map(PathBuf::from) {
        if path.is_file() && is_nsudo_launcher(&path) {
            return Some(path);
        }
    }
    let mut candidates = Vec::new();
    if let Some(root) = winslim_root() {
        for directory in [
            root.clone(),
            root.join("NSudo"),
            root.join("Tools").join("NSudo"),
            root.join("tools").join("NSudo"),
            root.join("bin"),
        ] {
            for name in ["NSudoLC.exe", "NSudoLG.exe", "NSudo.exe"] {
                candidates.push(directory.join(name));
            }
        }
        if let Some(found) = find_nsudo(&root, 0) {
            candidates.push(found);
        }
    }
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .or_else(|| {
            ["NSudoLC.exe", "NSudoLG.exe", "NSudo.exe"]
                .into_iter()
                .find_map(|name| command_path(name))
        })
}

fn find_nsudo(directory: &Path, depth: usize) -> Option<PathBuf> {
    if depth > 4 {
        return None;
    }
    let entries = std::fs::read_dir(directory)
        .ok()?
        .flatten()
        .collect::<Vec<_>>();
    let mut launchers = entries
        .iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| is_nsudo_launcher(path))
        .collect::<Vec<_>>();
    launchers.sort_by_key(|path| {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match name.as_str() {
            "nsudolc.exe" => 0,
            "nsudolg.exe" => 1,
            "nsudo.exe" => 2,
            _ => 3,
        }
    });
    if let Some(launcher) = launchers.into_iter().next() {
        return Some(launcher);
    }
    let mut directories = entries
        .into_iter()
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();
    directories.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    directories
        .into_iter()
        .find_map(|entry| find_nsudo(&entry.path(), depth + 1))
}

fn is_nsudo_launcher(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "nsudolc.exe" | "nsudolg.exe" | "nsudo.exe"
            )
        })
}

#[cfg(test)]
mod nsudo_detection_tests {
    use super::is_nsudo_launcher;
    use std::path::Path;

    #[test]
    fn detector_accepts_supported_launchers_but_not_sibling_tools() {
        for name in ["NSudoLC.exe", "NSudoLG.exe", "NSudo.exe", "nsudolg.EXE"] {
            assert!(is_nsudo_launcher(Path::new(name)), "missed {name}");
        }
        for name in [
            "NSudoDM.exe",
            "NSudoHelper.exe",
            "NSudoGUI.exe",
            "NSudo.txt",
        ] {
            assert!(!is_nsudo_launcher(Path::new(name)), "misdetected {name}");
        }
    }
}

fn command_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|directory| directory.join(name))
            .find(|candidate| candidate.is_file())
    })
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()
    })
}

pub fn is_elevated() -> bool {
    let shell = if command_exists("powershell") {
        "powershell"
    } else if command_exists("pwsh") {
        "pwsh"
    } else {
        return false;
    };
    command_output(
        shell,
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)",
        ],
    )
    .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
}
