mod audit;
mod common;
mod compat;
mod games;
mod i18n;
mod packages;
mod platform;
mod registry;
mod release;
mod storage;
mod system;
#[cfg(not(windows))]
mod wine;

use common::{home_dir, Context, Plan};
use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};

/// Version centralizada en `rust/Cargo.toml` y expuesta por Cargo al binario.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(unix)]
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn handle_interrupt(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

#[cfg(unix)]
fn install_interrupt_handler() {
    // The handler only flips an atomic flag; user-facing work stays in
    // normal Rust code after the interrupted read returns.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = handle_interrupt as *const () as libc::sighandler_t;
        libc::sigemptyset(&mut action.sa_mask);
        action.sa_flags = 0;
        libc::sigaction(libc::SIGINT, &action, std::ptr::null_mut());
    }
}

#[cfg(not(unix))]
fn install_interrupt_handler() {}

#[cfg(unix)]
fn interrupt_requested() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

#[cfg(not(unix))]
fn interrupt_requested() -> bool {
    false
}

fn finish_after_interrupt() -> bool {
    if interrupt_requested() {
        println!("\nInterrupción recibida. Saliendo de LTools.");
        true
    } else {
        false
    }
}

#[cfg(unix)]
fn read_menu_line() -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        let count = unsafe { libc::read(libc::STDIN_FILENO, byte.as_mut_ptr().cast(), 1) };
        if count == 0 {
            return Ok(None);
        }
        if count < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            return Err(error);
        }
        bytes.push(byte[0]);
        if byte[0] == b'\n' {
            return Ok(Some(String::from_utf8_lossy(&bytes).into_owned()));
        }
    }
}

#[cfg(not(unix))]
fn read_menu_line() -> io::Result<Option<String>> {
    let mut answer = String::new();
    let count = io::stdin().read_line(&mut answer)?;
    if count == 0 {
        Ok(None)
    } else {
        Ok(Some(answer))
    }
}

fn usage() {
    println!("{} Rust {VERSION}", i18n::text("app.title"));
    println!("{}", i18n::text("usage"));
    println!();
    println!("{}", i18n::text("commands"));
    println!("  audit       {}", i18n::text("help.audit"));
    println!("  games       {}", i18n::games_help());
    println!("  packages    {}", i18n::text("help.packages"));
    println!("  clean       {}", i18n::text("help.clean"));
    println!("  prefix      {}", i18n::prefix_help());
    println!("  defaults    {}", i18n::defaults_help());
    println!("  system      {}", i18n::system_help());
    println!("              {}", i18n::system_options());
    println!("  doctor      {}", i18n::text("help.doctor"));
    println!("              doctor --install TOOL");
    println!("  rollback    {}", i18n::text("help.rollback"));
    println!("  storage     {}", i18n::storage_help());
    println!("  registry    {}", i18n::registry_help());
    println!("  capabilities  {}", i18n::text("help.capabilities"));
    println!("  release-manifest  Genera el manifiesto verificable de una release de GitHub");
    println!();
    println!("{}", i18n::text("help.common"));
    println!("{}", i18n::text("help.clean.options"));
    println!("{}", i18n::prefix_options());
    let prefix_flags = i18n::prefix_flags();
    if !prefix_flags.is_empty() {
        println!("{prefix_flags}");
    }
    println!("{}", i18n::text("help.compat"));
}

enum MenuSelection {
    Command(String, Vec<String>),
    Continue,
    Quit,
}

fn execute_action(command: &str, ctx: &Context, args: &[String]) -> Result<(), String> {
    match command {
        "audit" | "disk-audit" => audit::run(ctx, args, false),
        "games" | "game-audit" => games::run(ctx, args),
        #[cfg(not(windows))]
        "wine-audit" => games::run(ctx, args),
        "packages" | "pkg-audit" | "package-audit" => packages::run(ctx, args),
        "clean" | "cleanup" => packages::clean(ctx, args),
        #[cfg(not(windows))]
        "prefix" | "wine" => wine::run(ctx, args),
        #[cfg(windows)]
        "prefix" | "wine" => {
            let _ = (ctx, args);
            println!("Los prefijos Wine/Proton no aplican al ejecutable Windows nativo; no se ha escaneado ninguna ruta.");
            Ok(())
        }
        "system" | "services" | "systemctl" => system::run(ctx, args),
        "storage" | "disks" | "partitions" => storage::run(ctx, args),
        "registry" | "records" => registry::run(ctx, args),
        "doctor" | "diagnose" => doctor_action(ctx, args),
        "defaults" | "paths" => show_defaults(ctx),
        "capabilities" | "compat" => compat::run(args),
        "release-manifest" | "release" => release::run(args),
        _ => {
            usage();
            Err(format!("comando desconocido: {command}"))
        }
    }
}

pub(crate) fn clear_screen() {
    if env::var_os("LTOOLS_NO_CLEAR").is_some() {
        return;
    }
    // ANSI funciona en las terminales Linux habituales y en las consolas
    // modernas de Windows. Se mantiene dentro de Rust para que el mismo
    // comportamiento llegue al binario, AppImage y ejecutable Windows.
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

fn run_interactive_menu(base_args: &[String], dry_run: bool, plan_path: Option<PathBuf>) {
    let mut requested_plan_path = plan_path;
    let mut first_menu = true;
    loop {
        if !first_menu {
            clear_screen();
        }
        first_menu = false;
        let (command, mut args) = match menu_choice() {
            MenuSelection::Command(command, selected_args) => (command, selected_args),
            MenuSelection::Continue => continue,
            MenuSelection::Quit => return,
        };
        let mut action_args = base_args.to_vec();
        action_args.append(&mut args);
        let is_submenu = matches!(
            command.as_str(),
            "clean" | "system" | "storage" | "registry"
        ) && action_args.iter().any(|arg| arg == "menu");
        let plan = match Plan::create(requested_plan_path.take(), &format!("rust-{command}")) {
            Ok(plan) => plan,
            Err(error) => {
                eprintln!("No se pudo crear el plan: {error}");
                continue;
            }
        };
        let ctx = Context {
            home: home_dir(),
            dry_run,
            plan_path: Some(plan.path.clone()),
            plan: Some(plan),
        };
        let result = execute_action(&command, &ctx, &action_args);
        if is_submenu && result.is_ok() {
            // El submenú ya gestiona su navegación. Al salir con Enter/q,
            // volver directamente al menú que lo abrió, sin una pausa extra.
            continue;
        }
        match result {
            Ok(()) => println!("Operación terminada correctamente."),
            Err(error) => eprintln!("Error: {error}"),
        }
        println!("Plan: {}", ctx.plan_path.as_ref().unwrap().display());
        print!("Pulsa Enter para volver al menú, o q para salir: ");
        let _ = io::stdout().flush();
        match read_menu_line() {
            Ok(Some(answer)) if answer.trim().eq_ignore_ascii_case("q") => return,
            Ok(Some(_)) => println!(),
            Ok(None) => return,
            Err(error) if error.kind() == io::ErrorKind::Interrupted || interrupt_requested() => {
                finish_after_interrupt();
                return;
            }
            Err(_) => return,
        }
    }
}

fn main() {
    let raw: Vec<String> = env::args().skip(1).collect();
    apply_language(&raw);
    install_interrupt_handler();
    if raw.iter().any(|a| a == "--version") {
        println!("ltools-rs {VERSION}");
        return;
    }
    if raw.iter().any(|a| a == "--capabilities") {
        if let Err(error) = compat::run(&raw) {
            eprintln!("Error: {error}");
            std::process::exit(2);
        }
        return;
    }
    if raw.iter().any(|a| a == "--help" || a == "-h") {
        usage();
        return;
    }
    // El ejecutable distribuido es autónomo: al abrirlo sin argumentos entra
    // directamente en su menú interactivo. Los lanzadores de cada plataforma
    // solo se ocupan de proporcionar una ventana de terminal cuando hace
    // falta; el backend normal sigue siendo exclusivamente Rust.
    if raw.is_empty() {
        if cli_profile() {
            usage();
        } else {
            run_interactive_menu(&[], false, None);
        }
        return;
    }
    // Acepta tanto `comando --opciones` como `--opciones comando ...`.
    // El lanzador Bash usa la segunda forma para las opciones globales.
    let mut command_index = 0;
    while command_index < raw.len() {
        match raw[command_index].as_str() {
            "--dry-run" => command_index += 1,
            "--lang" | "--language" => {
                if command_index + 1 >= raw.len() {
                    eprintln!("--lang requiere un idioma");
                    std::process::exit(2);
                }
                command_index += 2;
            }
            option if option.starts_with("--lang=") => command_index += 1,
            "--plan" => {
                if command_index + 1 >= raw.len() {
                    eprintln!("--plan requiere un fichero");
                    std::process::exit(2);
                }
                command_index += 2;
            }
            option if option.starts_with('-') => break,
            _ => break,
        }
    }
    let (command, args) = if command_index < raw.len() && !raw[command_index].starts_with('-') {
        let mut args = raw[..command_index].to_vec();
        args.extend_from_slice(&raw[command_index + 1..]);
        (raw[command_index].clone(), args)
    } else {
        ("audit".into(), raw.clone())
    };
    let mut dry_run = false;
    let mut plan_path = None;
    let mut filtered = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => dry_run = true,
            "--lang" | "--language" => {
                if let Some(value) = args.get(i + 1) {
                    i += 1;
                    i18n::set(value);
                } else {
                    eprintln!("--lang requiere un idioma");
                    std::process::exit(2);
                }
            }
            option if option.starts_with("--lang=") => {
                i18n::set(option.trim_start_matches("--lang="))
            }
            "--plan" => {
                if let Some(path) = args.get(i + 1) {
                    plan_path = Some(PathBuf::from(path));
                    i += 1;
                } else {
                    eprintln!("--plan requiere un fichero");
                    std::process::exit(2);
                }
            }
            _ => filtered.push(args[i].clone()),
        }
        i += 1;
    }
    if command == "rollback" || command == "undo" {
        let plan = plan_path.or_else(|| value(&filtered, "--plan").map(PathBuf::from));
        if let Some(path) = plan {
            if let Err(e) = common::restore_plan(&path) {
                eprintln!("Rollback fallido: {e}");
                std::process::exit(1);
            }
        } else {
            eprintln!("rollback requiere --plan FICHERO");
            std::process::exit(2);
        }
        return;
    }
    if matches!(command.as_str(), "capabilities" | "compat") {
        if let Err(error) = compat::run(&filtered) {
            eprintln!("Error: {error}");
            std::process::exit(2);
        }
        return;
    }
    if matches!(command.as_str(), "release-manifest" | "release") {
        if let Err(error) = release::run(&filtered) {
            eprintln!("Error: {error}");
            std::process::exit(2);
        }
        return;
    }
    if command == "menu" || command == "m" {
        run_interactive_menu(&filtered, dry_run, plan_path);
        return;
    }
    if matches!(
        command.as_str(),
        "doctor" | "diagnose" | "fuse" | "fuse-check"
    ) {
        let plan = match Plan::create(plan_path, "rust-doctor") {
            Ok(plan) => plan,
            Err(error) => {
                eprintln!("No se pudo crear el plan: {error}");
                std::process::exit(1);
            }
        };
        let ctx = Context {
            home: home_dir(),
            dry_run,
            plan_path: Some(plan.path.clone()),
            plan: Some(plan),
        };
        if let Err(error) = doctor_action(&ctx, &filtered) {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
        println!("Plan: {}", ctx.plan_path.unwrap().display());
        return;
    }
    let plan = match Plan::create(plan_path, &format!("rust-{command}")) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("No se pudo crear el plan: {e}");
            std::process::exit(1);
        }
    };
    let ctx = Context {
        home: home_dir(),
        dry_run,
        plan_path: Some(plan.path.clone()),
        plan: Some(plan.clone()),
    };
    let result = execute_action(&command, &ctx, &filtered);
    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
    println!("Plan: {}", ctx.plan_path.unwrap().display());
}

fn menu_choice() -> MenuSelection {
    #[cfg(windows)]
    {
        return menu_choice_windows();
    }
    #[cfg(not(windows))]
    {
        menu_choice_linux()
    }
}

#[cfg(not(windows))]
fn menu_choice_linux() -> MenuSelection {
    println!("{} Rust {VERSION}", i18n::text("menu.title"));
    println!("  1) {}", i18n::text("menu.audit"));
    println!("  2) {}", i18n::games_label());
    println!("  3) {}", i18n::text("menu.clean"));
    println!("  4) {}", i18n::prefix_label());
    println!("  5) {}", i18n::text("menu.defaults"));
    println!("  6) {}", i18n::text("menu.system"));
    println!("  7) {}", i18n::text("menu.doctor"));
    println!("  8) {}", i18n::text("menu.packages"));
    println!("  9) {}", i18n::storage_label());
    println!(" 10) {}", i18n::registry_label());
    println!("  h) {}", i18n::text("menu.help"));
    println!("  q) {}", i18n::text("menu.quit"));
    print!("{}", i18n::text("menu.prompt"));
    let _ = io::stdout().flush();
    let answer = match read_menu_line() {
        Ok(Some(answer)) if !finish_after_interrupt() => answer,
        Ok(Some(_)) | Ok(None) => return MenuSelection::Quit,
        Err(error) if error.kind() == io::ErrorKind::Interrupted || interrupt_requested() => {
            finish_after_interrupt();
            return MenuSelection::Quit;
        }
        Err(_) => return MenuSelection::Quit,
    };
    match answer.trim().to_lowercase().as_str() {
        "" => MenuSelection::Quit,
        "1" => menu_audit()
            .map(|args| MenuSelection::Command("audit".into(), args))
            .unwrap_or(MenuSelection::Quit),
        "2" => menu_games()
            .map(|args| MenuSelection::Command("games".into(), args))
            .unwrap_or(MenuSelection::Quit),
        "3" => MenuSelection::Command("clean".into(), vec!["menu".into()]),
        "4" => MenuSelection::Command("prefix".into(), vec!["list".into()]),
        "5" => MenuSelection::Command("defaults".into(), Vec::new()),
        "6" => MenuSelection::Command("system".into(), vec!["menu".into()]),
        "7" => MenuSelection::Command("doctor".into(), Vec::new()),
        "8" => menu_packages()
            .map(|args| MenuSelection::Command("packages".into(), args))
            .unwrap_or(MenuSelection::Quit),
        "9" => MenuSelection::Command("storage".into(), vec!["menu".into()]),
        "10" => MenuSelection::Command("registry".into(), vec!["menu".into()]),
        "q" | "quit" | "salir" => MenuSelection::Quit,
        "h" => {
            usage();
            MenuSelection::Continue
        }
        _ => {
            println!("Opción no válida.");
            MenuSelection::Continue
        }
    }
}

#[cfg(windows)]
fn menu_choice_windows() -> MenuSelection {
    println!("{} Rust {VERSION}", i18n::text("menu.title"));
    println!("  1) {}", i18n::text("menu.audit"));
    println!("  2) {}", i18n::games_label());
    println!("  3) {}", i18n::text("menu.clean"));
    println!("  4) {}", i18n::storage_label());
    println!("  5) {}", i18n::registry_label());
    println!("  6) {}", i18n::text("menu.system"));
    println!("  7) {}", i18n::text("menu.doctor"));
    println!("  8) {}", i18n::text("menu.packages"));
    println!("  9) {}", i18n::defaults_help());
    println!("  h) {}", i18n::text("menu.help"));
    println!("  q) {}", i18n::text("menu.quit"));
    print!("{}", i18n::text("menu.prompt"));
    let _ = io::stdout().flush();
    let answer = match read_menu_line() {
        Ok(Some(answer)) if !finish_after_interrupt() => answer,
        Ok(Some(_)) | Ok(None) => return MenuSelection::Quit,
        Err(error) if error.kind() == io::ErrorKind::Interrupted || interrupt_requested() => {
            finish_after_interrupt();
            return MenuSelection::Quit;
        }
        Err(_) => return MenuSelection::Quit,
    };
    match answer.trim().to_lowercase().as_str() {
        "" => MenuSelection::Quit,
        "1" => menu_audit()
            .map(|args| MenuSelection::Command("audit".into(), args))
            .unwrap_or(MenuSelection::Quit),
        "2" => menu_games()
            .map(|args| MenuSelection::Command("games".into(), args))
            .unwrap_or(MenuSelection::Quit),
        "3" => MenuSelection::Command("clean".into(), vec!["menu".into()]),
        "4" => MenuSelection::Command("storage".into(), vec!["menu".into()]),
        "5" => MenuSelection::Command("registry".into(), vec!["menu".into()]),
        "6" => MenuSelection::Command("system".into(), vec!["menu".into()]),
        "7" => MenuSelection::Command("doctor".into(), vec!["menu".into()]),
        "8" => menu_packages()
            .map(|args| MenuSelection::Command("packages".into(), args))
            .unwrap_or(MenuSelection::Quit),
        "9" => MenuSelection::Command("defaults".into(), Vec::new()),
        "q" | "quit" | "salir" => MenuSelection::Quit,
        "h" => {
            usage();
            MenuSelection::Continue
        }
        _ => {
            println!("Opción no válida.");
            MenuSelection::Continue
        }
    }
}

fn menu_input(prompt: &str) -> Option<String> {
    print!("{prompt}");
    let _ = io::stdout().flush();
    match read_menu_line() {
        Ok(None) => None,
        Ok(_) if finish_after_interrupt() => None,
        Ok(Some(answer)) => Some(answer.trim().to_string()),
        Err(error) if error.kind() == io::ErrorKind::Interrupted || interrupt_requested() => {
            finish_after_interrupt();
            None
        }
        Err(_) => None,
    }
}

fn menu_yes_no(prompt: &str, default: bool) -> Option<bool> {
    let answer = menu_input(prompt)?;
    if answer.is_empty() {
        return Some(default);
    }
    Some(matches!(
        answer.to_lowercase().as_str(),
        "y" | "yes" | "s" | "si" | "sí"
    ))
}

fn menu_audit() -> Option<Vec<String>> {
    println!("\nAuditoría general");
    #[cfg(windows)]
    println!("El escaneo Windows puede incluir las unidades disponibles.");
    #[cfg(not(windows))]
    println!("El escaneo rápido no añade automáticamente discos montados.");
    #[cfg(windows)]
    let full = menu_yes_no("¿Incluir todas las unidades disponibles? [y/N] ", false)?;
    #[cfg(not(windows))]
    let full = menu_yes_no("¿Escaneo completo, incluyendo montajes? [y/N] ", false)?;
    let duplicates = menu_yes_no("¿Buscar duplicados por SHA-256? [y/N] ", false)?;
    let root = menu_input("Ruta adicional (vacío para ninguna): ")?;
    let out = menu_input("Directorio de informe (vacío para el predeterminado): ")?;
    let mut args = Vec::new();
    if full {
        args.push("--full".into());
    } else {
        args.push("--no-mounts".into());
    }
    if duplicates {
        args.push("--duplicates".into());
    }
    if !root.is_empty() {
        args.extend(["--root".into(), root]);
    }
    if !out.is_empty() {
        args.extend(["--out".into(), out]);
    }
    Some(args)
}

fn menu_games() -> Option<Vec<String>> {
    #[cfg(windows)]
    {
        println!("\nInventario nativo de juegos y lanzadores Windows");
    }
    #[cfg(not(windows))]
    {
        println!("\nAuditoría de juegos, Wine y Proton");
    }
    #[cfg(windows)]
    let full = false;
    #[cfg(not(windows))]
    let full = menu_yes_no("¿Escaneo completo, incluyendo montajes? [y/N] ", false)?;
    let root = menu_input("Ruta adicional (vacío para ninguna): ")?;
    let out = menu_input("Directorio de informe (vacío para el predeterminado): ")?;
    let mut args = Vec::new();
    if full {
        args.push("--full".into());
    } else {
        args.push("--no-mounts".into());
    }
    if !root.is_empty() {
        args.extend(["--root".into(), root]);
    }
    if !out.is_empty() {
        args.extend(["--out".into(), out]);
    }
    Some(args)
}

fn menu_packages() -> Option<Vec<String>> {
    println!("\nInventario de paquetes y almacenes");
    let out = menu_input("Directorio de informe (vacío para el predeterminado): ")?;
    if out.is_empty() {
        Some(Vec::new())
    } else {
        Some(vec!["--out".into(), out])
    }
}

fn apply_language(raw: &[String]) {
    for (index, value) in raw.iter().enumerate() {
        if (value == "--lang" || value == "--language") && raw.get(index + 1).is_some() {
            i18n::set(&raw[index + 1]);
        } else if let Some(language) = value.strip_prefix("--lang=") {
            i18n::set(language);
        }
    }
}

fn cli_profile() -> bool {
    if matches!(
        env::var("LTOOLS_CLI").ok().as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("si") | Some("sí")
    ) {
        return true;
    }
    env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().to_lowercase())
        })
        .is_some_and(|stem| stem.ends_with("-cli") || stem.ends_with("_cli"))
}

fn value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn show_defaults(ctx: &Context) -> Result<(), String> {
    #[cfg(windows)]
    {
        return games::windows_defaults(ctx);
    }
    #[cfg(not(windows))]
    {
        println!("=== Defaults efectivos ===");
        println!("Wine: {}", command_path("wine"));
        println!("wineboot: {}", command_path("wineboot"));
        println!("winetricks: {}", command_path("winetricks"));
        println!(
            "WINEPREFIX activo: {}",
            env::var("WINEPREFIX").unwrap_or_else(|_| "no definido".into())
        );
        println!("Wine fallback: {}", ctx.home.join(".wine").display());
        println!("Proton: {}", command_path("proton"));
        println!("Steam: {}", command_path("steam"));
        println!("Steam compatdata: por AppID; no existe un default global seguro.");
        let heroic = ctx.home.join(".config/heroic/config.json");
        if heroic.is_file() {
            println!("Heroic config: {}", heroic.display());
            print_heroic_paths(&heroic);
        } else {
            println!("Heroic config: no encontrada");
        }
        let lutris = ctx.home.join(".local/share/lutris/system.yml");
        if lutris.is_file() {
            println!("Lutris system.yml: {}", lutris.display());
        }
        let umu = ctx.home.join(".local/share/umu");
        println!(
            "UMU: {} ({})",
            umu.display(),
            if umu.exists() {
                "existe"
            } else {
                "no encontrado"
            }
        );
        Ok(())
    }
}

#[cfg(not(windows))]
fn command_path(name: &str) -> String {
    if common::command_exists(name) {
        if cfg!(windows) {
            common::command_output("where", &[name]).unwrap_or_else(|| "instalado".into())
        } else {
            common::command_output("sh", &["-c", &format!("command -v {name}")])
                .unwrap_or_else(|| "instalado".into())
        }
    } else {
        "no instalado".into()
    }
}

fn host_doctor() -> Result<(), String> {
    println!("=== LTools host diagnostics ===");
    let mut available = 0;
    let mut missing_optional = Vec::new();
    let mut categories: BTreeMap<&str, Vec<&platform::HostTool>> = BTreeMap::new();
    for tool in common::platform_tools() {
        categories.entry(tool.category).or_default().push(tool);
    }
    for (category, tools) in categories {
        println!("\n[{category}]");
        for tool in tools {
            if platform::host_tool_available(tool) {
                available += 1;
                println!("  OK       {:<22} {}", tool.command, tool.feature);
            } else if tool.required {
                println!("  REQUIRED {:<22} {}", tool.command, tool.feature);
            } else {
                missing_optional.push(tool.command);
                println!(
                    "  MISSING  {:<22} {}{}",
                    tool.command,
                    tool.feature,
                    if tool.installable {
                        " (doctor --install puede ofrecerlo)"
                    } else {
                        " (integrada o dependiente de la plataforma)"
                    }
                );
            }
        }
    }
    println!(
        "\nHerramientas disponibles: {available}/{}",
        common::platform_tools().len()
    );
    if !missing_optional.is_empty() {
        println!("Opcionales ausentes: {}", missing_optional.join(", "));
    }
    println!(
        "  FUSE    {}",
        if platform::fuse_available() {
            "available"
        } else {
            "missing (AppImage extraction fallback is available)"
        }
    );
    #[cfg(windows)]
    println!("  Sistema Windows: systemctl/journalctl/FUSE no aplican en esta plataforma.");
    Ok(())
}

fn doctor_action(ctx: &Context, args: &[String]) -> Result<(), String> {
    if let Some(tool) = value(args, "--install") {
        if common::ensure_tool(ctx, &tool)? {
            println!("Dependencia disponible: {tool}");
            return Ok(());
        }
        return Err(format!("no se pudo disponer de la dependencia: {tool}"));
    }
    host_doctor()?;
    if args.iter().any(|arg| arg == "menu") {
        let tool =
            menu_input("Dependencia ausente a instalar (Enter para volver; nombre exacto): ")
                .unwrap_or_default();
        if !tool.is_empty() {
            if common::ensure_tool(ctx, &tool)? {
                println!("Dependencia disponible: {tool}");
            } else {
                println!("No se instaló {tool}; no se modifica nada más.");
            }
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn print_heroic_paths(file: &std::path::Path) {
    for line in common::read_lines(file).into_iter().filter(|l| {
        l.to_lowercase().contains("wineprefix")
            || l.to_lowercase().contains("defaultinstallpath")
            || l.to_lowercase().contains("defaultsteampath")
    }) {
        println!("  {}", line.trim());
    }
}
