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

pub fn host_tools() -> &'static [&'static str] {
    &[
        "findmnt",
        "sha256sum",
        "rsync",
        "jq",
        "perl",
        "wine",
        "wineboot",
        "winetricks",
        "paccache",
        "systemctl",
        "journalctl",
        "ps",
        "gio",
    ]
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
