#[cfg(not(windows))]
use crate::audit;
use crate::common::{clean, Context};
#[cfg(not(windows))]
use crate::common::{command_exists, ensure_tool, human_bytes};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    #[cfg(windows)]
    {
        run_windows(ctx, args)
    }
    #[cfg(not(windows))]
    {
        run_linux(ctx, args)
    }
}

#[cfg(not(windows))]
fn run_linux(ctx: &Context, args: &[String]) -> Result<(), String> {
    let out = value(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("rust-games-{}", crate::common::timestamp())));
    let mut audit_args = args.to_vec();
    if value(args, "--out").is_none() {
        audit_args.extend(["--out".into(), out.display().to_string()]);
    }
    audit::run(ctx, &audit_args, true)?;
    if heroic_configs(ctx).iter().any(|file| file.is_file())
        && !command_exists("jq")
        && !command_exists("python3")
    {
        let _ = ensure_tool(ctx, "jq")?;
    }
    let validation = out.join("configuration-validation.tsv");
    let mut file = File::create(&validation).map_err(|e| e.to_string())?;
    writeln!(file, "app\tconfig\tstatus\tfield\tvalue\tnote").map_err(|e| e.to_string())?;
    validate_heroic(ctx, &mut file);
    validate_lutris(ctx, &mut file);
    validate_umu(ctx, &mut file);
    validate_steam(ctx, &mut file);
    collect_config_files(ctx, &out)?;
    println!(
        "Validación de Heroic/Lutris/UMU/Steam: {}",
        validation.display()
    );
    Ok(())
}

#[cfg(windows)]
#[derive(Clone, Copy)]
struct WindowsLauncher {
    name: &'static str,
    paths: &'static [&'static str],
    configs: &'static [&'static str],
    manifest_dirs: &'static [&'static str],
}

#[cfg(windows)]
const WINDOWS_LAUNCHERS: &[WindowsLauncher] = &[
    WindowsLauncher {
        name: "Steam",
        paths: &["Steam", "SteamLibrary", "Games/Steam"],
        configs: &[
            "Steam/config/loginusers.vdf",
            "Steam/steamapps/libraryfolders.vdf",
        ],
        manifest_dirs: &[
            "Steam/steamapps",
            "SteamLibrary/steamapps",
            "Games/Steam/steamapps",
        ],
    },
    WindowsLauncher {
        name: "Epic Games",
        paths: &["Epic Games", "EpicGamesLauncher", "Games/Epic Games"],
        configs: &[
            "EpicGamesLauncher/Data/Manifests",
            "Epic/EpicGamesLauncher/Data/Manifests",
        ],
        manifest_dirs: &[
            "EpicGamesLauncher/Data/Manifests",
            "Epic/EpicGamesLauncher/Data/Manifests",
            "Epic Games/Manifests",
        ],
    },
    WindowsLauncher {
        name: "Ubisoft Connect",
        paths: &[
            "Ubisoft/Ubisoft Game Launcher",
            "Ubisoft Connect",
            "Games/Ubisoft",
        ],
        configs: &["Ubisoft Game Launcher/settings.yaml"],
        manifest_dirs: &[],
    },
    WindowsLauncher {
        name: "EA app",
        paths: &["EA Games", "Electronic Arts", "EA Desktop", "Games/EA"],
        configs: &["EA Desktop/Logs", "Electronic Arts/EA Desktop"],
        manifest_dirs: &[],
    },
    WindowsLauncher {
        name: "itch.io",
        paths: &["itch", "itch.io", "Games/itch"],
        configs: &["itch/config.ini", "itch/settings.json"],
        manifest_dirs: &[],
    },
    WindowsLauncher {
        name: "Battle.net",
        paths: &["Battle.net", "Blizzard Entertainment", "Games/Battle.net"],
        configs: &["Battle.net/Agent/Agent.7164/Agent.db"],
        manifest_dirs: &[],
    },
    WindowsLauncher {
        name: "Rockstar Games",
        paths: &["Rockstar Games", "Games/Rockstar Games"],
        configs: &["Rockstar Games/Launcher/launcher.log"],
        manifest_dirs: &[],
    },
    WindowsLauncher {
        name: "GOG Galaxy",
        paths: &["GOG Galaxy", "GOG Games", "Games/GOG"],
        configs: &["GOG Galaxy/galaxyClientService.log"],
        manifest_dirs: &[],
    },
    WindowsLauncher {
        name: "Xbox / Microsoft Store",
        paths: &["XboxGames", "ModifiableWindowsApps", "WindowsApps"],
        configs: &[],
        manifest_dirs: &[],
    },
];

#[cfg(windows)]
fn run_windows(ctx: &Context, args: &[String]) -> Result<(), String> {
    let out = value(args, "--out").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "ltools-windows-games-{}",
            crate::common::timestamp()
        ))
    });
    fs::create_dir_all(&out).map_err(|e| format!("no se pudo crear el informe: {e}"))?;
    let roots = windows_search_roots(ctx, args);
    let mut locations =
        File::create(out.join("windows-game-launchers.tsv")).map_err(|e| e.to_string())?;
    writeln!(locations, "launcher\tstatus\tpath\tsource").map_err(|e| e.to_string())?;
    let mut configs =
        File::create(out.join("configuration-validation.tsv")).map_err(|e| e.to_string())?;
    writeln!(configs, "app\tconfig\tstatus\tfield\tvalue\tnote").map_err(|e| e.to_string())?;
    let mut detected = 0usize;
    let mut seen = Vec::new();
    for launcher in WINDOWS_LAUNCHERS {
        for base in &roots {
            for relative in launcher.paths {
                let path = base.join(relative);
                if !path.exists() || seen.contains(&path) {
                    continue;
                }
                seen.push(path.clone());
                detected += 1;
                writeln!(
                    locations,
                    "{}\tinstalled\t{}\tcommon Windows path",
                    launcher.name,
                    clean(&path.display().to_string())
                )
                .map_err(|e| e.to_string())?;
            }
            for relative in launcher.configs {
                let path = base.join(relative);
                if !path.exists() {
                    continue;
                }
                writeln!(
                    configs,
                    "{}\t{}\texists\tpath\t{}\tconfiguración nativa Windows",
                    launcher.name,
                    clean(&path.display().to_string()),
                    clean(&path.display().to_string())
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    for launcher in WINDOWS_LAUNCHERS {
        for root in &roots {
            for relative in launcher.manifest_dirs {
                collect_windows_manifests(&root.join(relative), launcher.name, &mut locations)?;
            }
        }
    }
    let mut summary = File::create(out.join("summary.txt")).map_err(|e| e.to_string())?;
    writeln!(
        summary,
        "{} {}",
        crate::i18n::product_name(),
        crate::VERSION
    )
    .map_err(|e| e.to_string())?;
    writeln!(summary, "Modo: games-windows-native").map_err(|e| e.to_string())?;
    writeln!(summary, "Lanzadores Windows detectados: {detected}").map_err(|e| e.to_string())?;
    writeln!(
        summary,
        "No se buscan prefijos Wine, Lutris, Heroic ni UMU en Windows."
    )
    .map_err(|e| e.to_string())?;
    writeln!(summary, "Rutas inspeccionadas:").map_err(|e| e.to_string())?;
    for root in &roots {
        writeln!(summary, "  {}", root.display()).map_err(|e| e.to_string())?;
    }
    writeln!(summary, "Informe: {}", out.display()).map_err(|e| e.to_string())?;
    println!("Auditoría nativa de juegos Windows: {detected} ubicación(es).");
    println!("No se buscan prefijos Wine, Lutris, Heroic ni UMU.");
    println!("Informe: {}", out.display());
    let _ = ctx;
    Ok(())
}

#[cfg(windows)]
fn windows_search_roots(ctx: &Context, args: &[String]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for pair in args.windows(2) {
        if pair[0] == "--root" {
            roots.push(PathBuf::from(&pair[1]));
        }
    }
    if roots.is_empty() {
        roots.push(ctx.home.clone());
        for variable in [
            "PROGRAMFILES",
            "PROGRAMFILES(X86)",
            "PROGRAMDATA",
            "LOCALAPPDATA",
            "APPDATA",
            "PUBLIC",
        ] {
            if let Some(path) = std::env::var_os(variable).map(PathBuf::from) {
                roots.push(path);
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

#[cfg(windows)]
fn collect_windows_manifests(root: &Path, launcher: &str, out: &mut File) -> Result<(), String> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("item")
                    || ext.eq_ignore_ascii_case("acf")
                    || ext.eq_ignore_ascii_case("vdf")
            })
        {
            writeln!(
                out,
                "{}\tmanifest\t{}\tnative game manifest",
                launcher,
                clean(&path.display().to_string())
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(windows)]
pub fn windows_defaults(ctx: &Context) -> Result<(), String> {
    println!("=== Defaults nativos de juegos Windows ===");
    let roots = windows_search_roots(ctx, &[]);
    for launcher in WINDOWS_LAUNCHERS {
        let found = roots
            .iter()
            .flat_map(|root| launcher.paths.iter().map(move |path| root.join(path)))
            .find(|path| path.exists());
        println!(
            "{}: {}",
            launcher.name,
            found
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "no detectado".into())
        );
    }
    println!("Prefijos Wine/Lutris/Heroic/UMU: no aplican al ejecutable Windows nativo.");
    Ok(())
}

fn value(args: &[String], wanted: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == wanted)
        .map(|pair| pair[1].clone())
}

#[cfg(not(windows))]
fn heroic_configs(ctx: &Context) -> Vec<PathBuf> {
    if cfg!(windows) {
        vec![
            ctx.home.join("AppData/Roaming/heroic/config.json"),
            ctx.home.join("AppData/Roaming/heroic/store/config.json"),
        ]
    } else {
        vec![
            ctx.home.join(".config/heroic/config.json"),
            ctx.home.join(".config/heroic/store/config.json"),
        ]
    }
}

#[cfg(not(windows))]
fn lutris_configs(ctx: &Context) -> Vec<PathBuf> {
    if cfg!(windows) {
        vec![
            ctx.home.join("AppData/Roaming/lutris/system.yml"),
            ctx.home.join("AppData/Local/lutris/system.yml"),
        ]
    } else {
        vec![
            ctx.home.join(".local/share/lutris/system.yml"),
            ctx.home.join(".config/lutris/system.yml"),
        ]
    }
}

#[cfg(not(windows))]
fn lutris_games_dir(ctx: &Context) -> PathBuf {
    if cfg!(windows) {
        ctx.home.join("AppData/Roaming/lutris/games")
    } else {
        ctx.home.join(".local/share/lutris/games")
    }
}

#[cfg(not(windows))]
fn umu_root(ctx: &Context) -> PathBuf {
    if cfg!(windows) {
        ctx.home.join("AppData/Local/umu")
    } else {
        ctx.home.join(".local/share/umu")
    }
}

#[cfg(not(windows))]
fn steam_configs(ctx: &Context) -> Vec<PathBuf> {
    let mut configs = if cfg!(windows) {
        vec![
            ctx.home
                .join("AppData/Roaming/Steam/steamapps/libraryfolders.vdf"),
            ctx.home
                .join("AppData/Local/Steam/steamapps/libraryfolders.vdf"),
            PathBuf::from("C:/Program Files (x86)/Steam/steamapps/libraryfolders.vdf"),
            PathBuf::from("C:/Program Files/Steam/steamapps/libraryfolders.vdf"),
        ]
    } else {
        vec![
            ctx.home
                .join(".local/share/Steam/steamapps/libraryfolders.vdf"),
            ctx.home.join(".steam/steam/steamapps/libraryfolders.vdf"),
        ]
    };
    configs.sort();
    configs.dedup();
    configs
}

#[cfg(not(windows))]
fn config_roots(ctx: &Context) -> Vec<PathBuf> {
    if cfg!(windows) {
        vec![
            ctx.home.join("AppData/Roaming/heroic"),
            ctx.home.join("AppData/Roaming/lutris"),
            ctx.home.join("AppData/Local/umu"),
            ctx.home.join("AppData/Local/Steam/config"),
            ctx.home.join("AppData/Local/Steam/steamapps"),
            ctx.home.join("AppData/Roaming/Steam/config"),
            ctx.home.join("AppData/Roaming/Steam/steamapps"),
        ]
    } else {
        vec![
            ctx.home.join(".config/heroic"),
            ctx.home.join(".config/lutris"),
            ctx.home.join(".local/share/lutris/games"),
            ctx.home.join(".local/share/Steam/config"),
            ctx.home.join(".local/share/Steam/steamapps"),
            ctx.home.join("Games/Heroic/GamesConfig"),
        ]
    }
}

#[cfg(not(windows))]
fn configured_path_status(value: &str) -> &'static str {
    let path = Path::new(value);
    let windows_absolute = value.len() >= 3
        && value.as_bytes().get(1) == Some(&b':')
        && value
            .as_bytes()
            .get(2)
            .is_some_and(|byte| *byte == b'\\' || *byte == b'/');
    if path.is_absolute() || windows_absolute {
        if path.exists() {
            "path-exists"
        } else {
            "path-missing"
        }
    } else {
        "literal"
    }
}

#[cfg(not(windows))]
fn row(
    file: &mut File,
    app: &str,
    config: &Path,
    status: &str,
    field: &str,
    value: &str,
    note: &str,
) {
    let _ = writeln!(
        file,
        "{}\t{}\t{}\t{}\t{}\t{}",
        app,
        config.display(),
        status,
        field,
        clean(value),
        clean(note)
    );
}

#[cfg(not(windows))]
fn validate_heroic(ctx: &Context, out: &mut File) {
    for config in heroic_configs(ctx) {
        if !config.is_file() {
            continue;
        }
        let valid = if command_exists("jq") {
            Command::new("jq")
                .args(["empty"])
                .arg(&config)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else if command_exists("python3") {
            Command::new("python3")
                .args(["-c", "import json,sys; json.load(open(sys.argv[1]))"])
                .arg(&config)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            false
        };
        row(
            out,
            "Heroic",
            &config,
            if valid { "valid" } else { "unknown/invalid" },
            "syntax",
            "-",
            if valid {
                "JSON válido"
            } else {
                "no se pudo validar JSON"
            },
        );
        for line in read_text(&config).lines() {
            let lower = line.to_lowercase();
            if !(lower.contains("wineprefix")
                || lower.contains("defaultinstallpath")
                || lower.contains("defaultsteampath")
                || lower.contains("\"bin\""))
            {
                continue;
            }
            if let Some(path) = quoted_value(line) {
                row(
                    out,
                    "Heroic",
                    &config,
                    configured_path_status(&path),
                    line.split('"').nth(1).unwrap_or("field"),
                    &path,
                    "campo detectado",
                );
            }
        }
    }
}

#[cfg(not(windows))]
fn validate_lutris(ctx: &Context, out: &mut File) {
    for config in lutris_configs(ctx) {
        if !config.is_file() {
            continue;
        }
        for line in read_text(&config).lines() {
            if line.trim_start().starts_with("game_path:")
                || line.trim_start().starts_with("prefix:")
            {
                let value = line
                    .split_once(':')
                    .map(|(_, v)| v.trim().trim_matches(['\'', '"']))
                    .unwrap_or("");
                row(
                    out,
                    "Lutris",
                    &config,
                    configured_path_status(value),
                    line.trim().split(':').next().unwrap_or("path"),
                    value,
                    "configuración YAML",
                );
            }
        }
    }
    let games_dir = lutris_games_dir(ctx);
    if let Ok(entries) = fs::read_dir(games_dir) {
        for entry in entries
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "yml"))
        {
            let path = entry.path();
            for line in read_text(&path)
                .lines()
                .filter(|l| l.trim_start().starts_with("prefix:"))
            {
                let value = line
                    .split_once(':')
                    .map(|(_, v)| v.trim().trim_matches(['\'', '"']))
                    .unwrap_or("");
                row(
                    out,
                    "Lutris",
                    &path,
                    configured_path_status(value),
                    "prefix",
                    value,
                    "juego Lutris",
                );
            }
        }
    }
}

#[cfg(not(windows))]
fn validate_umu(ctx: &Context, out: &mut File) {
    let root = umu_root(ctx);
    if !root.is_dir() {
        return;
    }
    row(
        out,
        "UMU",
        &root,
        "exists",
        "root",
        &root.display().to_string(),
        "datos y runners",
    );
    for tool in [
        root.join("compatibilitytools/UMU-Latest"),
        root.join("steamrt3"),
        root.join("steamrt4"),
    ] {
        if tool.exists() {
            row(
                out,
                "UMU",
                &tool,
                if tool.join("toolmanifest.vdf").is_file() {
                    "tool-valid"
                } else {
                    "tool-incomplete"
                },
                "toolmanifest",
                &tool.join("toolmanifest.vdf").display().to_string(),
                "runner detectado",
            );
        }
    }
}

#[cfg(not(windows))]
fn validate_steam(ctx: &Context, out: &mut File) {
    for config in steam_configs(ctx) {
        if !config.is_file() {
            continue;
        }
        let mut seen = Vec::new();
        for line in read_text(&config)
            .lines()
            .filter(|l| l.contains("\"path\""))
        {
            if let Some(path) = quoted_value_after_path(line) {
                if seen.contains(&path) {
                    continue;
                }
                seen.push(path.clone());
                let library = PathBuf::from(&path);
                let status = if library.join("steamapps").is_dir() {
                    "library-exists"
                } else {
                    "library-missing"
                };
                row(
                    out,
                    "Steam",
                    &config,
                    status,
                    "library",
                    &path,
                    "libraryfolders.vdf",
                );
            }
        }
    }
}

#[cfg(not(windows))]
fn collect_config_files(ctx: &Context, out: &Path) -> Result<(), String> {
    let mut db =
        File::create(out.join("configuration-databases.tsv")).map_err(|e| e.to_string())?;
    let mut bin =
        File::create(out.join("configuration-binaries.tsv")).map_err(|e| e.to_string())?;
    writeln!(db, "bytes\thuman\tformat\tapp\tsqlite_header\tpath").map_err(|e| e.to_string())?;
    writeln!(bin, "bytes\thuman\tformat\tapp\tpath").map_err(|e| e.to_string())?;
    for root in config_roots(ctx) {
        collect_known(&root, &mut db, &mut bin, 0);
    }
    Ok(())
}

#[cfg(not(windows))]
fn collect_known(path: &Path, db: &mut File, bin: &mut File, depth: usize) {
    if depth > 5 {
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_known(&path, db, bin, depth + 1);
            continue;
        }
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();
        let format = match ext.as_str() {
            "db" | "sqlite" | "sqlite3" => "sqlite/db",
            "vdf" => "vdf",
            "acf" => "acf",
            "bin" | "dat" => "binary/dat",
            _ => continue,
        };
        let bytes = meta.len();
        let is_sqlite = matches!(ext.as_str(), "db" | "sqlite" | "sqlite3") && starts_sqlite(&path);
        if matches!(ext.as_str(), "db" | "sqlite" | "sqlite3") {
            let _ = writeln!(
                db,
                "{}\t{}\t{}\tuser\t{}\t{}",
                bytes,
                human_bytes(bytes),
                format,
                is_sqlite,
                path.display()
            );
        } else {
            let _ = writeln!(
                bin,
                "{}\t{}\t{}\tuser\t{}",
                bytes,
                human_bytes(bytes),
                format,
                path.display()
            );
        }
    }
}

#[cfg(not(windows))]
fn starts_sqlite(path: &Path) -> bool {
    let mut buffer = [0u8; 16];
    File::open(path)
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buffer))
        .is_ok()
        && &buffer[..15] == b"SQLite format 3"
}

#[cfg(not(windows))]
fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

#[cfg(not(windows))]
fn quoted_value(line: &str) -> Option<String> {
    line.split('"').nth(3).map(str::to_string)
}
#[cfg(not(windows))]
fn quoted_value_after_path(line: &str) -> Option<String> {
    line.split('"').nth(3).map(str::to_string)
}

#[cfg(not(windows))]
use std::process::Command;
