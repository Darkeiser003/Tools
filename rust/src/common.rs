use std::fs::{self, File};
#[cfg(not(windows))]
use std::io::Read;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
    explicit: bool,
}

impl Plan {
    pub fn create(path: Option<PathBuf>, module: &str) -> io::Result<Self> {
        let explicit = path.is_some();
        let path = path.unwrap_or_else(|| {
            let base = std::env::var_os("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".local/state"))
                .join("ltools/plans");
            base.join(format!("{}.tsv", stable_plan_name(module)))
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let reusable_explicit_plan = explicit
            && path.is_file()
            && File::open(&path)
                .ok()
                .and_then(|file| BufReader::new(file).lines().next())
                .and_then(Result::ok)
                .is_some_and(|line| line == "# ltools-plan-v1");
        if !reusable_explicit_plan {
            let mut file = File::create(&path)?;
            writeln!(file, "# ltools-plan-v1")?;
            writeln!(file, "# module={module}")?;
            writeln!(file, "# created={}", timestamp())?;
            writeln!(file, "operation\ttarget\tstatus\treversible\tdata1\tdata2")?;
        }
        Ok(Self { path, explicit })
    }

    /// Indicates that the user explicitly requested a plan file. Automatic
    /// plans are an internal transaction boundary for menus and are removed
    /// when the action did not record a real step.
    pub fn is_explicit(&self) -> bool {
        self.explicit
    }

    /// Explicit plans remain available for rollback and can be reused by an
    /// interactive session without truncating earlier operations. Automatic
    /// plans are retained only when an action actually recorded a reversible
    /// step, so a read-only menu does not leave an empty state file behind.
    pub fn finalize(&self) -> io::Result<bool> {
        if self.explicit || self.has_records()? {
            return Ok(true);
        }
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(false),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn has_records(&self) -> io::Result<bool> {
        let file = File::open(&self.path)?;
        Ok(BufReader::new(file).lines().nth(4).transpose()?.is_some())
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

fn stable_plan_name(module: &str) -> String {
    let mut name = String::from("plan");
    for character in module.chars() {
        if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
            name.push(character.to_ascii_lowercase());
        } else {
            name.push('-');
        }
    }
    name.trim_end_matches('-').to_string()
}

pub fn home_dir() -> PathBuf {
    crate::platform::home_dir()
}

pub fn timestamp() -> String {
    crate::platform::timestamp()
}

/// Directory used for generated reports when the user does not provide
/// `--out`. Reports are reusable state, not project files: keep one current
/// directory per module under the platform's state directory.
pub fn default_report_dir(home: &Path, module: &str) -> PathBuf {
    let state = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("AppData/Local"))
    } else {
        std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/state"))
    };
    state
        .join(if cfg!(windows) {
            "LTools/reports"
        } else {
            "ltools/reports"
        })
        .join(module)
}

/// Remove only the files directly generated in LTools' managed report folder.
/// Explicit `--out` paths never pass through this helper.
pub fn reset_default_report_dir(path: &Path) -> io::Result<()> {
    let module = path.file_name().and_then(|value| value.to_str());
    let reports = path.parent();
    let app_dir = reports.and_then(Path::parent);
    let managed = matches!(module, Some("audit" | "games" | "packages" | "system"))
        && reports
            .and_then(Path::file_name)
            .and_then(|value| value.to_str())
            == Some("reports")
        && matches!(
            app_dir
                .and_then(Path::file_name)
                .and_then(|value| value.to_str()),
            Some("ltools" | "LTools")
        );
    if !managed {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ruta de informe predeterminada no gestionada",
        ));
    }
    fs::create_dir_all(path)?;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_file() || file_type.is_symlink() {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

pub fn clean(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

pub fn command_exists(name: &str) -> bool {
    crate::platform::command_exists(name)
}

pub fn platform_tools() -> &'static [crate::platform::HostTool] {
    crate::platform::host_tools()
}

pub fn dependency_confirmation(
    command: &str,
    feature: &str,
    package: &str,
    manager: &str,
    invocation: &str,
) -> String {
    format!(
        "Falta la dependencia «{}».\n\nNecesaria para: {}\nPaquete que se instalará: {}\nGestor seleccionado: {}\nComando: {}\n\n¿Instalarla ahora?",
        command, feature, package, manager, invocation
    )
}

pub fn ensure_tool(ctx: &Context, id: &str) -> Result<bool, String> {
    let tool = platform_tools()
        .iter()
        .find(|tool| tool.id == id)
        .ok_or_else(|| format!("dependencia no gestionada por LTools: {id}"))?;
    if crate::platform::host_tool_available(tool) {
        return Ok(true);
    }
    if ctx.dry_run {
        println!(
            "Simulación: se comprobaría o instalaría {} para {}.",
            tool.command, tool.feature
        );
        return Ok(true);
    }
    let installed = crate::platform::install_tool(id, false)?;
    if installed {
        let record = |plan: &Plan| {
            plan.record(
                "dependency-install",
                Path::new(id),
                "executed",
                false,
                tool.install_package,
                tool.feature,
            )
            .map_err(|error| error.to_string())
        };
        if let Some(plan) = &ctx.plan {
            record(plan)?;
        } else {
            // A read-only query may offer an installation after the user
            // explicitly accepts it. Keep that real mutation auditable
            // without creating a plan for the preceding query itself.
            let plan = Plan::create(None, "rust-dependencies")
                .map_err(|error| format!("no se pudo crear el plan de dependencia: {error}"))?;
            if let Err(error) = record(&plan) {
                let _ = plan.finalize();
                return Err(error);
            }
            plan.finalize()
                .map_err(|error| format!("no se pudo cerrar el plan de dependencia: {error}"))?;
            println!("Plan: {}", plan.path.display());
        }
        Ok(true)
    } else {
        Ok(false)
    }
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

#[cfg(not(windows))]
pub fn critical_path(path: &Path) -> bool {
    crate::platform::critical_path(path)
}

pub fn ask(question: &str) -> bool {
    if std::env::var_os("LTOOLS_FRONTEND").is_some_and(|value| value == "gui") {
        #[cfg(any(target_os = "linux", windows))]
        {
            return crate::gui::confirm(question);
        }
    }
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

#[cfg(not(windows))]
pub fn read_lines(path: &Path) -> Vec<String> {
    File::open(path)
        .map(|file| BufReader::new(file).lines().map_while(Result::ok).collect())
        .unwrap_or_default()
}

#[cfg(not(windows))]
pub fn file_contains(path: &Path, needle: &str) -> bool {
    let mut data = String::new();
    File::open(path)
        .and_then(|mut f| f.read_to_string(&mut data))
        .map(|_| data.contains(needle))
        .unwrap_or(false)
}

#[cfg(not(windows))]
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

pub fn restore_plan(path: &Path, dry_run: bool) -> io::Result<()> {
    if !path.is_file() {
        eprintln!("No existe el plan: {}", path.display());
        return Ok(());
    }
    println!("Plan de rollback: {}", path.display());
    println!("Solo se restaurarán copias o movimientos reversibles ejecutados.");
    if dry_run {
        println!(
            "Modo dry-run: solo se mostrarán las restauraciones; no se modificará ningún archivo."
        );
    } else if !ask("¿Continuar con el rollback?") {
        println!("Rollback cancelado.");
        return Ok(());
    }
    let file = File::open(path)?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .transpose()?
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "plan vacío"))?;
    if header != "# ltools-plan-v1" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "formato de plan no reconocido",
        ));
    }
    let module = lines.next().transpose()?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "plan incompleto: falta el módulo",
        )
    })?;
    if !module.starts_with("# module=") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "plan incompleto: módulo inválido",
        ));
    }
    let created = lines.next().transpose()?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "plan incompleto: falta la fecha",
        )
    })?;
    if !created.starts_with("# created=") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "plan incompleto: fecha inválida",
        ));
    }
    let columns = lines.next().transpose()?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "plan incompleto: faltan columnas",
        )
    })?;
    if columns != "operation\ttarget\tstatus\treversible\tdata1\tdata2" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "plan incompleto: columnas no reconocidas",
        ));
    }
    let mut restored = 0;
    let mut skipped = 0;
    let lines: Vec<String> = lines.collect::<io::Result<Vec<_>>>()?;
    for line in lines.into_iter().rev() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 6 || fields[2] != "executed" || fields[3] != "yes" {
            if fields.len() < 6 {
                skipped += 1;
            }
            continue;
        }
        let operation = fields[0];
        let target = PathBuf::from(fields[1]);
        let data1 = PathBuf::from(fields[4]);
        match operation {
            "restore-file" => {
                if data1.is_file() {
                    if dry_run {
                        println!(
                            "Simulación: restauraría {} desde {}{}.",
                            target.display(),
                            data1.display(),
                            if target.exists() {
                                " (retirando antes el destino actual)"
                            } else {
                                ""
                            }
                        );
                        restored += 1;
                        continue;
                    }
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
                    if dry_run {
                        println!(
                            "Simulación: recuperaría {} desde la papelera ({}).",
                            target.display(),
                            data1.display()
                        );
                        restored += 1;
                        continue;
                    }
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
            "remove-created" if dry_run && target.exists() => {
                println!(
                    "Simulación: retiraría el destino creado a la papelera: {}",
                    target.display()
                );
                restored += 1;
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
    if dry_run {
        println!("Rollback simulado: {restored} operaciones previstas, {skipped} omitidas/no reversibles.");
    } else {
        println!("Rollback terminado: {restored} restauradas, {skipped} omitidas/no reversibles.");
    }
    Ok(())
}
