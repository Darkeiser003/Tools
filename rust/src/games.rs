use crate::audit;
use crate::common::{clean, command_exists, human_bytes, Context};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    audit::run(ctx, args, true)?;
    let out = value(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("rust-games-{}", crate::common::timestamp())));
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

fn value(args: &[String], wanted: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == wanted)
        .map(|pair| pair[1].clone())
}

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

fn validate_heroic(ctx: &Context, out: &mut File) {
    for config in [
        ctx.home.join(".config/heroic/config.json"),
        ctx.home.join(".config/heroic/store/config.json"),
    ] {
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
                let status = if path.starts_with('/') {
                    if Path::new(&path).exists() {
                        "path-exists"
                    } else {
                        "path-missing"
                    }
                } else {
                    "literal"
                };
                row(
                    out,
                    "Heroic",
                    &config,
                    status,
                    line.split('"').nth(1).unwrap_or("field"),
                    &path,
                    "campo detectado",
                );
            }
        }
    }
}

fn validate_lutris(ctx: &Context, out: &mut File) {
    for config in [
        ctx.home.join(".local/share/lutris/system.yml"),
        ctx.home.join(".config/lutris/system.yml"),
    ] {
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
                let status = if Path::new(value).is_dir() {
                    "path-exists"
                } else {
                    "path-missing"
                };
                row(
                    out,
                    "Lutris",
                    &config,
                    status,
                    line.trim().split(':').next().unwrap_or("path"),
                    value,
                    "configuración YAML",
                );
            }
        }
    }
    let games_dir = ctx.home.join(".local/share/lutris/games");
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
                    if Path::new(value).is_dir() {
                        "path-exists"
                    } else {
                        "path-missing"
                    },
                    "prefix",
                    value,
                    "juego Lutris",
                );
            }
        }
    }
}

fn validate_umu(ctx: &Context, out: &mut File) {
    let root = ctx.home.join(".local/share/umu");
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

fn validate_steam(ctx: &Context, out: &mut File) {
    for config in [
        ctx.home
            .join(".local/share/Steam/steamapps/libraryfolders.vdf"),
        ctx.home.join(".steam/steam/steamapps/libraryfolders.vdf"),
    ] {
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

fn collect_config_files(ctx: &Context, out: &Path) -> Result<(), String> {
    let mut db =
        File::create(out.join("configuration-databases.tsv")).map_err(|e| e.to_string())?;
    let mut bin =
        File::create(out.join("configuration-binaries.tsv")).map_err(|e| e.to_string())?;
    writeln!(db, "bytes\thuman\tformat\tapp\tsqlite_header\tpath").map_err(|e| e.to_string())?;
    writeln!(bin, "bytes\thuman\tformat\tapp\tpath").map_err(|e| e.to_string())?;
    let roots = [
        ctx.home.join(".config/heroic"),
        ctx.home.join(".config/lutris"),
        ctx.home.join(".local/share/lutris/games"),
        ctx.home.join(".local/share/Steam/config"),
        ctx.home.join(".local/share/Steam/steamapps"),
        ctx.home.join("Games/Heroic/GamesConfig"),
    ];
    for root in roots {
        collect_known(&root, &mut db, &mut bin, 0);
    }
    Ok(())
}

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

fn starts_sqlite(path: &Path) -> bool {
    let mut buffer = [0u8; 16];
    File::open(path)
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buffer))
        .is_ok()
        && &buffer[..15] == b"SQLite format 3"
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn quoted_value(line: &str) -> Option<String> {
    line.split('"').nth(3).map(str::to_string)
}
fn quoted_value_after_path(line: &str) -> Option<String> {
    line.split('"').nth(3).map(str::to_string)
}

use std::process::Command;
