//! Integración WTools/NSudo para lanzar procesos con un contexto explícito.
//! NSudo puede cambiar la identidad y los privilegios del proceso hijo; nunca
//! se usa como sustituto silencioso de UAC ni para modificar membresías.

use crate::common::Context;

#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Identity {
    Current,
    Elevated,
    System,
    TrustedInstaller,
    CurrentProcess,
    DropRights,
}

#[cfg(any(windows, test))]
impl Identity {
    fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "current" | "c" => Ok(Self::Current),
            "elevated" | "admin" | "e" => Ok(Self::Elevated),
            "system" | "s" => Ok(Self::System),
            "trustedinstaller" | "ti" | "t" => Ok(Self::TrustedInstaller),
            "process" | "p" => Ok(Self::CurrentProcess),
            "drop-rights" | "drop" | "d" => Ok(Self::DropRights),
            _ => Err("--identity debe ser current, elevated, system, trustedinstaller, process o drop-rights".into()),
        }
    }

    fn nsudo_code(self) -> &'static str {
        match self {
            Self::Current => "C",
            Self::Elevated => "E",
            Self::System => "S",
            Self::TrustedInstaller => "T",
            Self::CurrentProcess => "P",
            Self::DropRights => "D",
        }
    }

    #[cfg(windows)]
    fn label(self) -> &'static str {
        match self {
            Self::Current => "usuario actual",
            Self::Elevated => "usuario actual elevado",
            Self::System => "SYSTEM",
            Self::TrustedInstaller => "TrustedInstaller",
            Self::CurrentProcess => "token del proceso actual",
            Self::DropRights => "proceso actual con privilegios reducidos",
        }
    }
}

#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Integrity {
    System,
    High,
    Medium,
    Low,
}

#[cfg(any(windows, test))]
impl Integrity {
    fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "system" => Ok(Self::System),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            _ => Err("--integrity debe ser system, high, medium o low".into()),
        }
    }

    fn nsudo_code(self) -> &'static str {
        match self {
            Self::System => "S",
            Self::High => "H",
            Self::Medium => "M",
            Self::Low => "L",
        }
    }
}

#[cfg(any(windows, test))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchRequest {
    identity: Identity,
    program: String,
    arguments: Vec<String>,
    current_directory: Option<String>,
    integrity: Option<Integrity>,
    all_privileges: bool,
    wait: bool,
    current_console: bool,
    window_mode: Option<String>,
    confirmed: bool,
}

#[cfg(any(windows, test))]
fn usage() -> &'static str {
    "WTools / NSudo (Windows):\n  wtools status\n  wtools guide\n  wtools menu\n  wtools launch --identity current|elevated|system|trustedinstaller|process|drop-rights --program PROGRAMA [--arg ARG ...] [--cwd RUTA] [--integrity system|high|medium|low] [--all-privileges] [--wait] [--console] [--window show|hide|maximize|minimize] [--yes]\n\nLos argumentos se pasan separados, sin shell. `winslim` sigue siendo un alias técnico compatible. --yes es obligatorio para automatización no interactiva; --dry-run global muestra el plan sin lanzar nada."
}

#[cfg(any(windows, test))]
fn parse_launch_args(args: &[String]) -> Result<LaunchRequest, String> {
    let mut identity = None;
    let mut program = None;
    let mut arguments = Vec::new();
    let mut current_directory = None;
    let mut integrity = None;
    let mut all_privileges = false;
    let mut wait = false;
    let mut current_console = false;
    let mut window_mode = None;
    let mut confirmed = false;
    let mut index = if args.first().is_some_and(|value| value == "launch") {
        1
    } else {
        0
    };

    while index < args.len() {
        let option = args[index].as_str();
        index += 1;
        match option {
            "--identity" => {
                identity = Some(Identity::parse(next_value(args, &mut index, option)?)?);
            }
            "--program" => {
                program = Some(validate_token(
                    next_value(args, &mut index, option)?,
                    "--program",
                    4_096,
                )?);
            }
            "--arg" => {
                arguments.push(validate_token(
                    next_value(args, &mut index, option)?,
                    "--arg",
                    32_767,
                )?);
                if arguments.len() > 128 {
                    return Err("se permiten como máximo 128 argumentos para el proceso".into());
                }
            }
            "--cwd" => {
                current_directory = Some(validate_token(
                    next_value(args, &mut index, option)?,
                    "--cwd",
                    4_096,
                )?);
            }
            "--integrity" => {
                integrity = Some(Integrity::parse(next_value(args, &mut index, option)?)?);
            }
            "--all-privileges" => all_privileges = true,
            "--wait" => wait = true,
            "--console" => current_console = true,
            "--window" => {
                let mode = next_value(args, &mut index, option)?.to_ascii_lowercase();
                if !matches!(mode.as_str(), "show" | "hide" | "maximize" | "minimize") {
                    return Err("--window debe ser show, hide, maximize o minimize".into());
                }
                window_mode = Some(mode);
            }
            "--yes" => confirmed = true,
            "--" => {
                for value in &args[index..] {
                    arguments.push(validate_token(value, "argumento", 32_767)?);
                    if arguments.len() > 128 {
                        return Err("se permiten como máximo 128 argumentos para el proceso".into());
                    }
                }
                break;
            }
            "--help" | "-h" => return Err(usage().into()),
            _ => return Err(format!("opción NSudo desconocida: {option}\n\n{}", usage())),
        }
    }

    let identity = identity.ok_or_else(|| {
        "falta --identity; no se elige un contexto privilegiado por defecto".to_owned()
    })?;
    let program = program.ok_or_else(|| "falta --program PROGRAMA".to_owned())?;
    if program.starts_with('-') {
        return Err(
            "--program no puede empezar por guion; indica una ruta o nombre de ejecutable".into(),
        );
    }
    Ok(LaunchRequest {
        identity,
        program,
        arguments,
        current_directory,
        integrity,
        all_privileges,
        wait,
        current_console,
        window_mode,
        confirmed,
    })
}

#[cfg(any(windows, test))]
fn next_value<'a>(args: &'a [String], index: &mut usize, option: &str) -> Result<&'a str, String> {
    let value = args
        .get(*index)
        .map(String::as_str)
        .ok_or_else(|| format!("{option} requiere un valor"))?;
    *index += 1;
    Ok(value)
}

#[cfg(any(windows, test))]
fn validate_token(value: &str, label: &str, max_len: usize) -> Result<String, String> {
    if value.trim().is_empty() || value.len() > max_len || value.chars().any(char::is_control) {
        return Err(format!(
            "{label} está vacío, supera {max_len} bytes o contiene caracteres de control"
        ));
    }
    Ok(value.to_owned())
}

#[cfg(any(windows, test))]
fn nsudo_arguments(request: &LaunchRequest) -> Vec<String> {
    let mut arguments = vec![format!("-U:{}", request.identity.nsudo_code())];
    if request.all_privileges {
        arguments.push("-P:E".into());
    }
    if let Some(integrity) = request.integrity {
        arguments.push(format!("-M:{}", integrity.nsudo_code()));
    }
    if let Some(directory) = &request.current_directory {
        arguments.push(format!("-CurrentDirectory:{directory}"));
    }
    if request.current_console {
        arguments.push("-UseCurrentConsole".into());
    }
    if request.wait {
        arguments.push("-Wait".into());
    }
    if let Some(mode) = &request.window_mode {
        arguments.push(format!("-ShowWindowMode:{}", mode.to_ascii_title_case()));
    }
    arguments.push(request.program.clone());
    arguments.extend(request.arguments.iter().cloned());
    arguments
}

#[cfg(any(windows, test))]
trait TitleCase {
    fn to_ascii_title_case(&self) -> String;
}

#[cfg(any(windows, test))]
impl TitleCase for str {
    fn to_ascii_title_case(&self) -> String {
        let mut chars = self.chars();
        chars
            .next()
            .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
            .unwrap_or_default()
    }
}

#[cfg(windows)]
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    match args
        .iter()
        .find(|value| !value.starts_with('-'))
        .map(String::as_str)
    {
        None | Some("status") => status(),
        Some("guide") => {
            println!("{}", guide_text());
            Ok(())
        }
        Some("menu") => menu(ctx),
        Some("launch") => launch(ctx, args),
        Some("--help" | "help") => {
            println!("{}", usage());
            Ok(())
        }
        Some(action) => Err(format!(
            "acción WTools desconocida: {action}\n\n{}",
            usage()
        )),
    }
}

#[cfg(not(windows))]
pub fn run(_ctx: &Context, _args: &[String]) -> Result<(), String> {
    Err("WTools/NSudo solo se aplica al ejecutable Windows nativo; no se ejecutó nada.".into())
}

#[cfg(windows)]
fn status() -> Result<(), String> {
    match crate::platform::winslim_root() {
        Some(root) => println!(
            "{} {}",
            crate::i18n::automation_text("winslim_ready"),
            root.display()
        ),
        None => println!("{}", crate::i18n::automation_text("winslim_wscore_missing")),
    }
    match crate::platform::nsudo_path() {
        Some(path) => println!(
            "{} {}",
            crate::i18n::automation_text("winslim_nsudo_found"),
            path.display()
        ),
        None => println!("{}", crate::i18n::automation_text("winslim_nsudo_missing")),
    }
    println!("{}", crate::i18n::automation_text("winslim_default"));
    Ok(())
}

#[cfg(windows)]
fn launch(ctx: &Context, args: &[String]) -> Result<(), String> {
    let request = parse_launch_args(args)?;
    if let Some(directory) = &request.current_directory {
        let path = std::path::Path::new(directory);
        if !path.is_absolute() || (!ctx.dry_run && !path.is_dir()) {
            return Err("--cwd debe ser una carpeta absoluta existente (en --dry-run solo se valida que sea absoluta)".into());
        }
    }
    let launcher = crate::platform::nsudo_path();
    if launcher.is_none() && !ctx.dry_run {
        return Err("NSudo no está disponible. Se buscó en C:\\WSCore, sus carpetas de herramientas y PATH; no se descargó ni instaló nada.".into());
    }
    let launcher_args = nsudo_arguments(&request);
    println!(
        "NSudo: {}",
        launcher
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "no detectado (simulación)".into())
    );
    println!("Contexto solicitado: {}", request.identity.label());
    println!(
        "Opciones de contexto: -U:{}{}{}{}{}",
        request.identity.nsudo_code(),
        if request.all_privileges { " -P:E" } else { "" },
        request
            .integrity
            .map(|value| format!(" -M:{}", value.nsudo_code()))
            .unwrap_or_default(),
        if request.wait { " -Wait" } else { "" },
        if request.current_console {
            " -UseCurrentConsole"
        } else {
            ""
        }
    );
    if let Some(mode) = &request.window_mode {
        println!("Modo de ventana: {mode}");
    }
    println!(
        "Programa: {} ({} argumento(s), no se guardan en el registro)",
        request.program,
        request.arguments.len()
    );
    if ctx.dry_run {
        println!("Simulación: no se inició NSudo ni el proceso solicitado.");
        return Ok(());
    }
    if !request.confirmed
        && !crate::common::ask(&format!(
            "¿Lanzar {} como {}? Este contexto puede modificar archivos y configuración protegidos; revisa el programa y sus argumentos.",
            request.program,
            request.identity.label()
        ))
    {
        return Err("lanzamiento NSudo cancelado".into());
    }
    let launcher =
        launcher.ok_or_else(|| "NSudo no está disponible; no se ejecutó el proceso.".to_owned())?;
    let status = std::process::Command::new(&launcher)
        .args(&launcher_args)
        .status()
        .map_err(|error| format!("no se pudo iniciar {}: {error}", launcher.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("NSudo terminó con {status}; comprueba que esta versión admite el contexto y las opciones seleccionadas."))
    }
}

#[cfg(windows)]
fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        let has_nsudo = crate::platform::nsudo_path().is_some();
        println!("\nWTools / NSudo\n  1) Estado de WSCore y NSudo");
        if has_nsudo {
            println!("  2) Lanzar proceso con contexto elegido");
            println!("  3) Guía y compatibilidad");
        } else {
            println!("  2) Guía y compatibilidad (el lanzamiento requiere NSudo)");
        }
        println!("  q) Volver");
        match crate::menu_input("Elige una opción: ").as_deref() {
            Some("1") => status()?,
            Some("2") if has_nsudo => interactive_launch(ctx)?,
            Some("2") if !has_nsudo => println!("{}", guide_text()),
            Some("3") if has_nsudo => println!("{}", guide_text()),
            Some("") | Some("q") | Some("Q") | None => return Ok(()),
            Some(_) => println!("Opción no válida."),
        }
    }
}

#[cfg(windows)]
fn interactive_launch(ctx: &Context) -> Result<(), String> {
    println!("\nIdentidad: 1) usuario actual  2) usuario actual elevado  3) SYSTEM  4) TrustedInstaller  5) token del proceso  6) reducir privilegios  q) cancelar");
    let identity = match crate::menu_input("Perfil: ").as_deref() {
        Some("1") => "current",
        Some("2") => "elevated",
        Some("3") => "system",
        Some("4") => "trustedinstaller",
        Some("5") => "process",
        Some("6") => "drop-rights",
        _ => return Ok(()),
    };
    let Some(program) = crate::menu_input("Programa o ruta ejecutable (p. ej. powershell.exe): ")
    else {
        return Ok(());
    };
    if program.is_empty() {
        return Ok(());
    }
    let mut args = vec![
        "launch".to_owned(),
        "--identity".into(),
        identity.into(),
        "--program".into(),
        program,
    ];
    for index in 1..=32 {
        let Some(argument) =
            crate::menu_input(&format!("Argumento {index} (vacío para terminar): "))
        else {
            return Ok(());
        };
        if argument.is_empty() {
            break;
        }
        args.extend(["--arg".into(), argument]);
    }
    if crate::common::ask("¿Solicitar todos los privilegios disponibles para el proceso? (no recomendado salvo necesidad explícita)") {
        args.push("--all-privileges".into());
    }
    if crate::common::ask("¿Esperar a que termine el proceso elevado?") {
        args.push("--wait".into());
    }
    if crate::common::ask("¿Compartir la consola actual con el proceso?") {
        args.push("--console".into());
    }
    launch(ctx, &args)
}

#[cfg(windows)]
fn guide_text() -> &'static str {
    concat!(
        "Guía NSudo (solo Windows)\n\n",
        "NSudo puede lanzar procesos como usuario actual (C), usuario actual elevado (E), SYSTEM (S), TrustedInstaller (T), token del proceso actual (P) o con privilegios reducidos (D). Las versiones no ofrecen necesariamente todos los perfiles y opciones. LTools busca NSudo en WSCore y PATH; puedes indicar una ruta concreta con LTOOLS_NSUDO_PATH.\n\n",
        "Perfiles CLI: --identity current|elevated|system|trustedinstaller|process|drop-rights. Integridad: --integrity system|high|medium|low. También puedes indicar --cwd RUTA, --window show|hide|maximize|minimize, --all-privileges, --console y --wait.\n\n",
        "Ejemplo de consulta: ltools wtools launch --identity elevated --program powershell.exe --arg -NoProfile --arg -Command --arg Get-Date --wait\n",
        "Ejemplo de mantenimiento: ltools wtools launch --identity trustedinstaller --program notepad.exe --arg C:\\Windows\\System32\\drivers\\etc\\hosts\n\n",
        "Los argumentos se pasan separados, sin shell. Para scripts PowerShell, ejecuta powershell.exe/pwsh.exe y pasa -File y la ruta del script; NSudo no interpreta scripts por sí mismo. `--all-privileges` habilita explícitamente todos los privilegios de NSudo; `--integrity`, `--cwd`, `--window`, `--console` y `--wait` controlan el proceso hijo. `--dry-run` no lo inicia. En automatizaciones, `--yes` evita la confirmación interactiva y solo debe añadirse tras revisar el efecto; LTools no almacena los argumentos en los informes.\n\n",
        "NSudo cambia el token de un proceso: no añade usuarios a Administradores ni inicia sesión como otra cuenta arbitraria. La concesión de Administrador se gestiona en Cuentas; TrustedInstaller es una identidad de servicio, no un grupo de usuarios. `LTOOLS_USE_NSUDO=1` selecciona el token elevado del usuario actual para suboperaciones compatibles que ya solicitan privilegios; no selecciona SYSTEM/TrustedInstaller ni cambia la preferencia persistente. Si falta NSudo, las acciones normales conservan UAC. Usa solo una copia de confianza."
    )
}

#[cfg(test)]
mod tests {
    use super::{nsudo_arguments, parse_launch_args, Identity, Integrity};

    fn argv(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn launch_args_build_native_identity_integrity_and_child_argv_separately() {
        let request = parse_launch_args(&argv(&[
            "launch",
            "--identity",
            "trustedinstaller",
            "--program",
            "C:\\Windows\\System32\\notepad.exe",
            "--arg",
            "C:\\Program Files\\hosts.txt",
            "--integrity",
            "high",
            "--all-privileges",
            "--wait",
            "--console",
            "--window",
            "maximize",
        ]))
        .unwrap();
        assert_eq!(request.identity, Identity::TrustedInstaller);
        assert_eq!(request.integrity, Some(Integrity::High));
        assert_eq!(
            nsudo_arguments(&request),
            [
                "-U:T",
                "-P:E",
                "-M:H",
                "-UseCurrentConsole",
                "-Wait",
                "-ShowWindowMode:Maximize",
                "C:\\Windows\\System32\\notepad.exe",
                "C:\\Program Files\\hosts.txt"
            ]
        );
    }

    #[test]
    fn launch_requires_an_explicit_identity_and_rejects_unknown_or_controlled_inputs() {
        assert!(parse_launch_args(&argv(&["launch", "--program", "cmd.exe"])).is_err());
        assert!(parse_launch_args(&argv(&[
            "launch",
            "--identity",
            "other-user",
            "--program",
            "cmd.exe"
        ]))
        .is_err());
        assert!(parse_launch_args(&argv(&[
            "launch",
            "--identity",
            "system",
            "--program",
            "cmd.exe",
            "--arg",
            "bad\narg"
        ]))
        .is_err());
        assert!(parse_launch_args(&argv(&[
            "launch",
            "--identity",
            "system",
            "--program",
            "cmd.exe",
            "--window",
            "hidden"
        ]))
        .is_err());
    }

    #[test]
    fn nsudo_identity_codes_cover_current_admin_system_ti_process_and_drop_rights() {
        assert_eq!(Identity::Current.nsudo_code(), "C");
        assert_eq!(Identity::Elevated.nsudo_code(), "E");
        assert_eq!(Identity::System.nsudo_code(), "S");
        assert_eq!(Identity::TrustedInstaller.nsudo_code(), "T");
        assert_eq!(Identity::CurrentProcess.nsudo_code(), "P");
        assert_eq!(Identity::DropRights.nsudo_code(), "D");
    }

    #[test]
    fn child_arguments_can_start_with_option_prefixes_without_being_reparsed() {
        let request = parse_launch_args(&argv(&[
            "launch",
            "--identity",
            "system",
            "--program",
            "powershell.exe",
            "--arg",
            "--NoProfile",
            "--",
            "--ExecutionPolicy",
            "Bypass",
        ]))
        .unwrap();
        assert_eq!(
            request.arguments,
            ["--NoProfile", "--ExecutionPolicy", "Bypass"]
        );
    }
}
