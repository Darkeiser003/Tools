mod audit;
mod common;
mod games;
mod i18n;
mod packages;
mod system;
mod wine;

use common::{home_dir, Context, Plan};
use std::env;
use std::io::{self, Write};
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;

pub const VERSION: &str = "0.3.0";

fn usage() {
    println!("{} Rust {VERSION}", i18n::text("app.title"));
    println!("{}", i18n::text("usage"));
    println!();
    println!("{}", i18n::text("commands"));
    println!("  audit       {}", i18n::text("help.audit"));
    println!("  games       {}", i18n::text("help.games"));
    println!("  packages    {}", i18n::text("help.packages"));
    println!("  clean       {}", i18n::text("help.clean"));
    println!("  prefix      {}", i18n::text("help.prefix"));
    println!("  defaults    {}", i18n::text("help.defaults"));
    println!("  system      {}", i18n::text("help.system"));
    println!("  doctor      {}", i18n::text("help.doctor"));
    println!("  rollback    {}", i18n::text("help.rollback"));
    println!();
    println!("{}", i18n::text("help.common"));
    println!("{}", i18n::text("help.clean.options"));
    println!("{}", i18n::text("help.prefix.options"));
    println!("{}", i18n::text("help.prefix.flags"));
    println!("{}", i18n::text("help.compat"));
}

fn main() {
    let raw: Vec<String> = env::args().skip(1).collect();
    apply_language(&raw);
    if raw.iter().any(|a| a == "--version") {
        println!("ltools-rs {VERSION}");
        return;
    }
    if raw.is_empty() || raw.iter().any(|a| a == "--help" || a == "-h") {
        usage();
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
    let (mut command, args) = if command_index < raw.len() && !raw[command_index].starts_with('-') {
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
    if command == "menu" || command == "m" {
        command = match menu_choice() {
            Some(choice) => choice,
            None => return,
        };
    }
    if command == "doctor" || command == "diagnose" {
        if let Err(error) = host_doctor() {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
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
    let result = match command.as_str() {
        "audit" | "disk-audit" => audit::run(&ctx, &filtered, false),
        "games" | "game-audit" | "wine-audit" => games::run(&ctx, &filtered),
        "packages" | "pkg-audit" | "package-audit" => packages::run(&ctx, &filtered),
        "clean" | "cleanup" => packages::clean(&ctx, &filtered),
        "prefix" | "wine" => wine::run(&ctx, &filtered),
        "system" | "services" | "systemctl" => system::run(&ctx, &filtered),
        "doctor" | "diagnose" => host_doctor(),
        "defaults" | "paths" => show_defaults(&ctx),
        _ => {
            usage();
            Err(format!("comando desconocido: {command}"))
        }
    };
    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
    println!("Plan: {}", ctx.plan_path.unwrap().display());
}

fn menu_choice() -> Option<String> {
    println!("{} Rust {VERSION}", i18n::text("menu.title"));
    println!("  1) {}", i18n::text("menu.audit"));
    println!("  2) {}", i18n::text("menu.games"));
    println!("  3) {}", i18n::text("menu.clean"));
    println!("  4) {}", i18n::text("menu.prefix"));
    println!("  5) {}", i18n::text("menu.defaults"));
    println!("  6) {}", i18n::text("menu.system"));
    println!("  7) {}", i18n::text("menu.doctor"));
    println!("  h) {}", i18n::text("menu.help"));
    println!("  q) {}", i18n::text("menu.quit"));
    print!("{}", i18n::text("menu.prompt"));
    let _ = io::stdout().flush();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).ok()?;
    match answer.trim().to_lowercase().as_str() {
        "1" => Some("audit".into()),
        "2" => Some("games".into()),
        "3" => Some("clean".into()),
        "4" => Some("prefix".into()),
        "5" => Some("defaults".into()),
        "6" => Some("system".into()),
        "7" => Some("doctor".into()),
        "h" => {
            usage();
            None
        }
        _ => None,
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

fn value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn show_defaults(ctx: &Context) -> Result<(), String> {
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

fn command_path(name: &str) -> String {
    if common::command_exists(name) {
        common::command_output("sh", &["-c", &format!("command -v {name}")])
            .unwrap_or_else(|| "instalado".into())
    } else {
        "no instalado".into()
    }
}

fn host_doctor() -> Result<(), String> {
    println!("=== LTools host diagnostics ===");
    for tool in [
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
    ] {
        if common::command_exists(tool) {
            println!("  OK      {tool}");
        } else {
            println!("  MISSING {tool}");
        }
    }
    let fuse_device = std::fs::metadata("/dev/fuse")
        .map(|metadata| metadata.file_type().is_char_device())
        .unwrap_or(false);
    let fusermount = common::command_exists("fusermount3") || common::command_exists("fusermount");
    println!(
        "  FUSE    {}",
        if fuse_device && fusermount {
            "available"
        } else {
            "missing (AppImage extraction fallback is available)"
        }
    );
    Ok(())
}
fn print_heroic_paths(file: &std::path::Path) {
    for line in common::read_lines(file).into_iter().filter(|l| {
        l.to_lowercase().contains("wineprefix")
            || l.to_lowercase().contains("defaultinstallpath")
            || l.to_lowercase().contains("defaultsteampath")
    }) {
        println!("  {}", line.trim());
    }
}
