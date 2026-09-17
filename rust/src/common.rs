use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(not(test))]
const NATIVE_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(test)]
const NATIVE_COMMAND_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        !self.timed_out && self.status_code == Some(0)
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub home: PathBuf,
    pub dry_run: bool,
    pub elevate_by_default: bool,
    pub privileged_child: bool,
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
            #[cfg(unix)]
            if !explicit {
                ensure_private_plan_directory(parent)?;
            }
        }
        let existing = match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "la ruta del plan debe ser un archivo regular, no un enlace",
                    ));
                }
                true
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };
        let existing_plan = existing && is_ltools_plan(&path)?;
        if existing && !existing_plan {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "el archivo de destino ya existe y no es un plan LTools; no se sobrescribió",
            ));
        }
        #[cfg(unix)]
        if existing_plan {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }
        if explicit && existing_plan {
            return Ok(Self { path, explicit });
        }

        let mut options = OpenOptions::new();
        options.write(true);
        if existing_plan {
            options.truncate(true);
        } else {
            options.create_new(true);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let mut file = options.open(&path)?;
        writeln!(file, "# ltools-plan-v1")?;
        writeln!(file, "# module={module}")?;
        writeln!(file, "# created={}", timestamp())?;
        writeln!(file, "operation\ttarget\tstatus\treversible\tdata1\tdata2")?;
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

fn is_ltools_plan(path: &Path) -> io::Result<bool> {
    let file = File::open(path)?;
    let mut lines = BufReader::new(file).lines();
    let Some(header) = lines.next().transpose()? else {
        return Ok(false);
    };
    let Some(module) = lines.next().transpose()? else {
        return Ok(false);
    };
    let Some(created) = lines.next().transpose()? else {
        return Ok(false);
    };
    let Some(columns) = lines.next().transpose()? else {
        return Ok(false);
    };
    let valid_header = header == "# ltools-plan-v1"
        && module
            .strip_prefix("# module=")
            .is_some_and(|value| !value.is_empty())
        && created
            .strip_prefix("# created=")
            .is_some_and(|value| !value.is_empty())
        && columns == "operation\ttarget\tstatus\treversible\tdata1\tdata2";
    if !valid_header {
        return Ok(false);
    }
    for row in lines {
        let row = row?;
        let fields = row.split('\t').collect::<Vec<_>>();
        if fields.len() != 6
            || fields[0].is_empty()
            || fields[2].is_empty()
            || !matches!(fields[3], "yes" | "no")
        {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(unix)]
fn ensure_private_plan_directory(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "el directorio automático de planes debe ser un directorio real",
        ));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(test)]
mod plan_tests {
    use super::{copy_file_without_replace, stable_plan_name, Plan};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEMP_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn automatic_plan_name_matches_the_documented_module_format() {
        assert_eq!(stable_plan_name("rust-clean"), "plan-rust-clean");
        assert_eq!(stable_plan_name("rust-storage"), "plan-rust-storage");
    }

    fn temporary_directory(label: &str) -> PathBuf {
        for _ in 0..1024 {
            let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "ltools-plan-{label}-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&directory) {
                Ok(()) => return directory,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("no se pudo crear el directorio de prueba: {error}"),
            }
        }
        panic!("no se encontró un nombre temporal libre para la prueba")
    }

    #[test]
    fn plan_creation_never_truncates_an_unrecognized_existing_file() {
        let directory = temporary_directory("preserve");
        let path = directory.join("notes.tsv");
        fs::write(&path, "user data that must survive\n").unwrap();

        let result = Plan::create(Some(path.clone()), "test");
        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "user data that must survive\n"
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn plan_creation_preserves_a_file_with_only_the_plan_marker() {
        let directory = temporary_directory("partial-header");
        let path = directory.join("plan.tsv");
        let contents = "# ltools-plan-v1\nthis is not a complete plan\n";
        fs::write(&path, contents).unwrap();

        assert!(Plan::create(Some(path.clone()), "test").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), contents);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn plan_creation_preserves_a_plan_with_a_malformed_record() {
        let directory = temporary_directory("partial-record");
        let path = directory.join("plan.tsv");
        let contents = concat!(
            "# ltools-plan-v1\n",
            "# module=rust-test\n",
            "# created=2026-09-13T00:00:00Z\n",
            "operation\ttarget\tstatus\treversible\tdata1\tdata2\n",
            "truncated\trow\n",
        );
        fs::write(&path, contents).unwrap();

        assert!(Plan::create(Some(path.clone()), "test").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), contents);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn explicit_valid_plan_is_reused_without_losing_its_records() {
        let directory = temporary_directory("reuse");
        let path = directory.join("plan.tsv");
        let plan = Plan::create(Some(path.clone()), "test").unwrap();
        plan.record(
            "copy",
            std::path::Path::new("/source"),
            "planned",
            false,
            "/destination",
            "",
        )
        .unwrap();
        let before = fs::read_to_string(&path).unwrap();
        drop(plan);

        let _reused = Plan::create(Some(path.clone()), "test").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn plan_files_and_automatic_plan_directories_are_private() {
        use std::os::unix::fs::PermissionsExt;

        let directory = temporary_directory("permissions");
        let path = directory.join("plan.tsv");
        {
            let _plan = Plan::create(Some(path.clone()), "test").unwrap();
        }
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        {
            let _plan = Plan::create(Some(path.clone()), "test").unwrap();
        }
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let automatic = directory.join("automatic-plans");
        fs::create_dir(&automatic).unwrap();
        fs::set_permissions(&automatic, fs::Permissions::from_mode(0o755)).unwrap();
        super::ensure_private_plan_directory(&automatic).unwrap();
        assert_eq!(
            fs::metadata(&automatic).unwrap().permissions().mode() & 0o777,
            0o700
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn plan_creation_rejects_symbolic_links_without_touching_the_target() {
        let directory = temporary_directory("symlink");
        let target = directory.join("important.txt");
        let link = directory.join("plan.tsv");
        fs::write(&target, "keep this file\n").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(Plan::create(Some(link), "test").is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "keep this file\n");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn restoring_a_file_never_overwrites_a_destination_that_appears() {
        let directory = temporary_directory("restore-no-replace");
        let backup = directory.join("backup.txt");
        let destination = directory.join("destination.txt");
        fs::write(&backup, "backup data").unwrap();
        fs::write(&destination, "new user data").unwrap();

        assert!(copy_file_without_replace(&backup, &destination).is_err());
        assert_eq!(fs::read_to_string(&backup).unwrap(), "backup data");
        assert_eq!(fs::read_to_string(&destination).unwrap(), "new user data");

        fs::remove_dir_all(directory).unwrap();
    }
}

fn stable_plan_name(module: &str) -> String {
    let mut name = String::from("plan-");
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
    command_output_detailed(program, args)
        .ok()
        .filter(CommandOutput::success)
        .map(|output| output.stdout.trim_end().to_owned())
}

pub fn command_output_owned(program: &str, args: &[String]) -> Option<String> {
    command_output_detailed_owned(program, args)
        .ok()
        .filter(CommandOutput::success)
        .map(|output| output.stdout.trim_end().to_owned())
}

pub fn command_output_detailed(program: &str, args: &[&str]) -> io::Result<CommandOutput> {
    command_output_detailed_with_env(program, args, &[])
}

pub fn command_output_detailed_with_env(
    program: &str,
    args: &[&str],
    environment: &[(&str, &str)],
) -> io::Result<CommandOutput> {
    let args = args
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    command_output_detailed_owned_with_env(program, &args, environment)
}

pub fn command_output_detailed_owned(program: &str, args: &[String]) -> io::Result<CommandOutput> {
    command_output_detailed_owned_with_env(program, args, &[])
}

fn command_output_detailed_owned_with_env(
    program: &str,
    args: &[String],
    environment: &[(&str, &str)],
) -> io::Result<CommandOutput> {
    let mut child = Command::new(program)
        .args(args)
        .envs(environment.iter().copied())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().map(|mut reader| {
        thread::spawn(move || {
            let mut data = Vec::new();
            let _ = reader.read_to_end(&mut data);
            data
        })
    });
    let stderr = child.stderr.take().map(|mut reader| {
        thread::spawn(move || {
            let mut data = Vec::new();
            let _ = reader.read_to_end(&mut data);
            data
        })
    });
    let deadline = Instant::now() + NATIVE_COMMAND_TIMEOUT;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                timed_out = true;
                let _ = child.kill();
                break child.wait()?;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    };
    let stdout = stdout
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default();
    let stderr = stderr
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default();
    Ok(CommandOutput {
        status_code: status.code(),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
    })
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
    // Las acciones modificadoras deben conservar stdin/stdout/stderr de la
    // terminal: los gestores pueden pedir confirmaciones y las elevaciones
    // necesitan mostrar su prompt. Tampoco se les aplica el timeout de las
    // consultas de detección; una instalación o reparación legítima puede
    // tardar más de 30 segundos.
    Ok(Command::new(program).args(args).status()?.success())
}

pub fn run_with_sudo(program: &str, args: &[String], dry_run: bool) -> io::Result<bool> {
    crate::platform::run_with_privilege(program, args, dry_run)
}

/// Ejecuta una orden privilegiada alimentando su stdin sin convertir el
/// contenido en argumentos, logs ni planes. Se usa únicamente para flujos
/// guiados que ya obtuvieron el secreto en un diálogo propio.
#[cfg(not(windows))]
pub fn run_with_sudo_input(
    program: &str,
    args: &[String],
    input: &[u8],
    dry_run: bool,
) -> io::Result<bool> {
    crate::platform::run_with_privilege_input(program, args, input, dry_run)
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

#[cfg(any(windows, test))]
pub(crate) fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Cita un argumento individual para CommandLineToArgvW y las aplicaciones
/// Windows basadas en el runtime C. Se comparte con las pruebas Linux para
/// que los límites de argumentos de la elevación UAC sean verificables sin
/// depender de un diálogo interactivo de Windows.
#[cfg(any(windows, test))]
pub(crate) fn windows_command_line_argument(value: &str) -> String {
    let mut quoted = String::from("\"");
    let mut backslashes = 0_usize;
    for character in value.chars() {
        match character {
            '\\' => backslashes += 1,
            '"' => {
                quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            }
            _ => {
                quoted.extend(std::iter::repeat_n('\\', backslashes));
                quoted.push(character);
                backslashes = 0;
            }
        }
    }
    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
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
    directory_size_with_cancel(path, dev, None)
}

pub(crate) fn is_link_or_reparse_point(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

pub fn directory_size_with_cancel(
    path: &Path,
    dev: Option<u64>,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> u64 {
    if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
        return 0;
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if is_link_or_reparse_point(&metadata) || dev.is_some_and(|d| !same_device(path, d)) {
        return 0;
    }
    if metadata.is_file() {
        return metadata.len();
    }
    if !metadata.is_dir() {
        return 0;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0_u64;
    for entry in entries.flatten() {
        if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
            break;
        }
        total = total.saturating_add(directory_size_with_cancel(&entry.path(), dev, cancelled));
    }
    total
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
                            if path_entry_exists(&target) {
                                " (retirando antes el destino actual)"
                            } else {
                                ""
                            }
                        );
                        restored += 1;
                        continue;
                    }
                    if path_entry_exists(&target) && !move_to_trash(&target, false)? {
                        skipped += 1;
                        continue;
                    }
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    copy_file_without_replace(&data1, &target)?;
                    println!("Restaurado: {}", target.display());
                    restored += 1;
                } else {
                    skipped += 1;
                }
            }
            "trash-move" => {
                if path_entry_exists(&data1) && !path_entry_exists(&target) {
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
                    crate::storage_map::rename_without_replace(&data1, &target)?;
                    println!("Recuperado: {}", target.display());
                    restored += 1;
                } else {
                    skipped += 1;
                }
            }
            "path-move" if path_entry_exists(&target) && !path_entry_exists(&data1) => {
                if dry_run {
                    println!(
                        "Simulación: devolvería {} a su ubicación original {}.",
                        target.display(),
                        data1.display()
                    );
                    restored += 1;
                    continue;
                }
                if let Some(parent) = data1.parent() {
                    fs::create_dir_all(parent)?;
                }
                match crate::storage_map::rename_without_replace(&target, &data1) {
                    Ok(()) => {
                        println!("Movimiento restaurado: {}", data1.display());
                        restored += 1;
                    }
                    Err(error) => {
                        eprintln!(
                            "No se pudo restaurar el movimiento {} -> {}: {error}",
                            target.display(),
                            data1.display()
                        );
                        skipped += 1;
                    }
                }
            }
            "path-move" => skipped += 1,
            "remove-created" if dry_run && path_entry_exists(&target) => {
                println!(
                    "Simulación: retiraría el destino creado a la papelera: {}",
                    target.display()
                );
                restored += 1;
            }
            "remove-created" if path_entry_exists(&target) && move_to_trash(&target, false)? => {
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

fn path_entry_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn copy_file_without_replace(source: &Path, destination: &Path) -> io::Result<()> {
    crate::storage_map::copy_new_path(source, destination).map_err(io::Error::other)
}

#[cfg(test)]
mod process_tests {
    #[cfg(unix)]
    use super::{command_output_detailed, command_output_detailed_owned, run_command};

    #[cfg(unix)]
    #[test]
    fn conserva_stderr_y_codigo_de_salida() {
        let output = command_output_detailed("sh", &["-c", "printf error >&2; exit 7"]).unwrap();
        assert_eq!(output.status_code, Some(7));
        assert!(!output.success());
        assert_eq!(output.stderr, "error");
    }

    #[cfg(unix)]
    #[test]
    fn corta_comandos_que_no_terminan() {
        let output = command_output_detailed_owned("sleep", &["31".into()]).unwrap();
        assert!(output.timed_out);
        assert!(!output.success());
    }

    #[cfg(unix)]
    #[test]
    fn las_acciones_reales_no_heredan_el_timeout_de_las_consultas() {
        let started = std::time::Instant::now();
        assert!(run_command("sleep", &["0.4".into()], false).unwrap());
        assert!(started.elapsed() >= std::time::Duration::from_millis(350));
    }

    #[test]
    fn argumentos_uac_conservan_espacios_comillas_y_barras_finales() {
        assert_eq!(
            super::windows_command_line_argument(r"C:\Program Files\LTools\ltools.exe"),
            r#""C:\Program Files\LTools\ltools.exe""#
        );
        assert_eq!(super::windows_command_line_argument("a\"b"), r#""a\"b""#);
        assert_eq!(
            super::windows_command_line_argument("tail\\"),
            r#""tail\\""#
        );
        assert_eq!(super::powershell_single_quoted("O'Brien"), "'O''Brien'");
    }
}
