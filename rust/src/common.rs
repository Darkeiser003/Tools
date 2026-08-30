use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Context {
    pub home: PathBuf,
    pub dry_run: bool,
    pub plan_path: Option<PathBuf>,
    pub plan: Option<Plan>,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub path: PathBuf,
}

impl Plan {
    pub fn create(path: Option<PathBuf>, module: &str) -> io::Result<Self> {
        let path = path.unwrap_or_else(|| {
            let base = std::env::var_os("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".local/state"))
                .join("ltools/plans");
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            base.join(format!(
                "plan-{}-{}-{}.tsv",
                timestamp(),
                unique,
                std::process::id()
            ))
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&path)?;
        writeln!(file, "# ltools-plan-v1")?;
        writeln!(file, "# module={module}")?;
        writeln!(file, "# created={}", timestamp())?;
        writeln!(file, "operation\ttarget\tstatus\treversible\tdata1\tdata2")?;
        Ok(Self { path })
    }

    pub fn record(
        &self,
        operation: &str,
        target: &Path,
        status: &str,
        reversible: bool,
        data1: &str,
        data2: &str,
    ) -> io::Result<()> {
        let mut file = fs::OpenOptions::new().append(true).open(&self.path)?;
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}",
            clean(operation),
            clean(&target.display().to_string()),
            clean(status),
            if reversible { "yes" } else { "no" },
            clean(data1),
            clean(data2)
        )
    }
}

pub fn home_dir() -> PathBuf {
    crate::platform::home_dir()
}

pub fn timestamp() -> String {
    crate::platform::timestamp()
}

pub fn clean(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

pub fn command_exists(name: &str) -> bool {
    crate::platform::command_exists(name)
}

pub fn platform_tools() -> &'static [&'static str] {
    crate::platform::host_tools()
}

pub fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
    )
}

pub fn command_output_owned(program: &str, args: &[String]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
    )
}

pub fn run_command(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    println!(
        "  $ {} {}",
        program,
        args.iter()
            .map(|a| shell_display(a))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if dry_run {
        return Ok(true);
    }
    Ok(Command::new(program).args(args).status()?.success())
}

pub fn run_with_sudo(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    crate::platform::run_with_privilege(program, args, dry_run)
}

pub fn shell_display(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ". /_:@%+-".contains(c))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

pub fn human_bytes(mut bytes: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut unit = 0;
    let mut value = bytes as f64;
    while bytes >= 1024 && unit < units.len() - 1 {
        bytes /= 1024;
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{}{}", bytes, units[unit])
    } else {
        format!("{value:.1}{}", units[unit])
    }
}

pub fn canonical(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok()
}

pub fn device(path: &Path) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        fs::symlink_metadata(path).ok().map(|m| m.dev())
    }
    #[cfg(windows)]
    {
        fs::symlink_metadata(path).ok().map(|_| 0)
    }
}

pub fn same_device(path: &Path, expected: u64) -> bool {
    #[cfg(unix)]
    {
        device(path) == Some(expected)
    }
    #[cfg(windows)]
    {
        let _ = expected;
        path.exists()
    }
}

pub fn directory_size(path: &Path, dev: Option<u64>) -> u64 {
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if metadata.file_type().is_symlink() || dev.is_some_and(|d| !same_device(path, d)) {
        return 0;
    }
    if metadata.is_file() {
        return metadata.len();
    }
    if !metadata.is_dir() {
        return 0;
    }
    fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| directory_size(&e.path(), dev))
                .sum()
        })
        .unwrap_or(0)
}

pub fn critical_path(path: &Path) -> bool {
    crate::platform::critical_path(path)
}

pub fn ask(question: &str) -> bool {
    print!("{} [y/N] ", question);
    let _ = io::stdout().flush();
    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() {
        return false;
    }
    matches!(
        answer.trim().to_lowercase().as_str(),
        "y" | "yes" | "s" | "si" | "sí"
    )
}

pub fn prompt_path(question: &str) -> Option<PathBuf> {
    print!("{}", question);
    let _ = io::stdout().flush();
    let mut value = String::new();
    io::stdin().read_line(&mut value).ok()?;
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

pub fn move_to_trash(path: &Path, dry_run: bool) -> io::Result<bool> {
    crate::platform::move_to_trash(path, dry_run)
}

pub fn read_lines(path: &Path) -> Vec<String> {
    File::open(path)
        .map(|file| BufReader::new(file).lines().map_while(Result::ok).collect())
        .unwrap_or_default()
}

pub fn file_contains(path: &Path, needle: &str) -> bool {
    let mut data = String::new();
    File::open(path)
        .and_then(|mut f| f.read_to_string(&mut data))
        .map(|_| data.contains(needle))
        .unwrap_or(false)
}

pub fn backup(path: &Path) -> io::Result<PathBuf> {
    let stamp = timestamp();
    let backup = PathBuf::from(format!("{}.bak-{}", path.display(), stamp));
    fs::copy(path, &backup)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o600);
        let _ = fs::set_permissions(&backup, permissions);
    }
    Ok(backup)
}

pub fn restore_plan(path: &Path) -> io::Result<()> {
    if !path.is_file() {
        eprintln!("No existe el plan: {}", path.display());
        return Ok(());
    }
    println!("Plan de rollback: {}", path.display());
    println!("Solo se restaurarán copias o movimientos reversibles ejecutados.");
    if !ask("¿Continuar con el rollback?") {
        println!("Rollback cancelado.");
        return Ok(());
    }
    let file = File::open(path)?;
    let mut restored = 0;
    let mut skipped = 0;
    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(4)
        .collect();
    for line in lines.into_iter().rev() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() < 6 || fields[2] != "executed" || fields[3] != "yes" {
            continue;
        }
        let operation = fields[0];
        let target = PathBuf::from(fields[1]);
        let data1 = PathBuf::from(fields[4]);
        match operation {
            "restore-file" => {
                if data1.is_file() {
                    if target.exists() && !move_to_trash(&target, false)? {
                        skipped += 1;
                        continue;
                    }
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&data1, &target)?;
                    println!("Restaurado: {}", target.display());
                    restored += 1;
                } else {
                    skipped += 1;
                }
            }
            "trash-move" => {
                if data1.exists() && !target.exists() {
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::rename(&data1, &target)?;
                    println!("Recuperado: {}", target.display());
                    restored += 1;
                } else {
                    skipped += 1;
                }
            }
            "remove-created" if target.exists() && move_to_trash(&target, false)? => {
                println!("Destino retirado a papelera: {}", target.display());
                restored += 1;
            }
            "remove-created" => skipped += 1,
            _ => {
                skipped += 1;
            }
        }
    }
    println!("Rollback terminado: {restored} restauradas, {skipped} omitidas/no reversibles.");
    Ok(())
}
