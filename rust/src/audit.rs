use crate::common::{
    canonical, clean, command_exists, command_output, command_output_owned, device, directory_size,
    human_bytes, same_device, Context,
};
use std::collections::HashMap;
#[cfg(not(windows))]
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PrefixInfo {
    pub path: PathBuf,
    pub kind: String,
    pub appid: String,
    pub bytes: u64,
    pub drive_c_bytes: u64,
    pub mount_root: bool,
}

pub fn discover_prefixes(roots: &[PathBuf]) -> Vec<PrefixInfo> {
    #[cfg(windows)]
    {
        let _ = roots;
        return Vec::new();
    }
    #[cfg(not(windows))]
    {
        let mut candidates = HashSet::new();
        let mut result = Vec::new();
        for root in roots {
            let root = match canonical(root) {
                Some(v) => v,
                None => continue,
            };
            let dev = match device(&root) {
                Some(v) => v,
                None => continue,
            };
            find_prefix_markers(&root, dev, &mut candidates);
            for path in candidates.iter().filter(|p| p.starts_with(&root)) {
                let bytes = directory_size(path, Some(dev));
                let drive_c_bytes = directory_size(&path.join("drive_c"), Some(dev));
                result.push(PrefixInfo {
                    kind: prefix_kind(path),
                    appid: prefix_appid(path),
                    mount_root: *path == root,
                    path: path.clone(),
                    bytes,
                    drive_c_bytes,
                });
            }
        }
        result.sort_by_key(|p| p.bytes);
        result.dedup_by(|a, b| a.path == b.path);
        result
    }
}

#[cfg(not(windows))]
fn find_prefix_markers(path: &Path, dev: u64, found: &mut HashSet<PathBuf>) {
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let child = entry.path();
        let metadata = match fs::symlink_metadata(&child) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if metadata.file_type().is_symlink() || !same_device(&child, dev) {
            continue;
        }
        if metadata.is_file() && child.file_name().is_some_and(|n| n == "system.reg") {
            if let Some(parent) = child.parent().and_then(canonical) {
                found.insert(parent);
            }
        } else if metadata.is_dir() {
            if child.file_name().is_some_and(|n| n == "drive_c") {
                if let Some(parent) = child.parent().and_then(canonical) {
                    found.insert(parent);
                }
            }
            find_prefix_markers(&child, dev, found);
        }
    }
}

#[cfg(not(windows))]
pub fn prefix_kind(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.contains("/.Trash-") || text.contains("/Trash/") {
        "trash-prefix".into()
    } else if text.ends_with("/.wine") {
        "default-wine".into()
    } else if text.contains("/steamapps/compatdata/") && text.ends_with("/pfx") {
        "steam-proton".into()
    } else if text.contains("/lutris/") || text.contains("/Lutrs/") {
        "lutris-prefix".into()
    } else if text.contains("/Heroic/") || text.contains("/heroic/") {
        "heroic-prefix".into()
    } else if text.contains("/umu/") {
        "umu-prefix".into()
    } else if text.contains("/bottles/") {
        "bottles-prefix".into()
    } else {
        "wine-prefix-unknown".into()
    }
}

#[cfg(not(windows))]
pub fn prefix_appid(path: &Path) -> String {
    let parts: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    parts
        .windows(2)
        .find(|p| p[0] == "compatdata" && p[1].parse::<u64>().is_ok())
        .map(|p| p[1].clone())
        .unwrap_or_else(|| "-".into())
}

pub fn default_roots(
    home: &Path,
    full: bool,
    explicit: &[PathBuf],
    include_home: bool,
    auto_mounts: bool,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if explicit.is_empty() && include_home {
        roots.push(home.to_path_buf());
    } else {
        roots.extend_from_slice(explicit);
    }
    if full && explicit.is_empty() && !cfg!(windows) {
        roots.extend([
            PathBuf::from("/opt"),
            PathBuf::from("/usr/local/share"),
            PathBuf::from("/srv"),
        ]);
    }
    // Discover mount points without assuming disk names. Only direct mount
    // targets are added; the recursive scanner still stays on one device.
    let auto_mounts = auto_mounts
        && !matches!(
            std::env::var("LTOOLS_NO_MOUNTS").ok().as_deref(),
            Some("1") | Some("true") | Some("yes") | Some("si") | Some("sí")
        );
    if auto_mounts && full && explicit.is_empty() && !cfg!(windows) && command_exists("findmnt") {
        if let Some(mounts) = command_output("findmnt", &["-rn", "-o", "TARGET"]) {
            for mount in mounts.lines().filter(|p| {
                p.starts_with("/mnt/") || p.starts_with("/media/") || p.starts_with("/run/media/")
            }) {
                let path = PathBuf::from(mount);
                if !roots.contains(&path) {
                    roots.push(path);
                }
            }
        }
    }
    #[cfg(windows)]
    if explicit.is_empty() && include_home {
        for folder in ["Documents", "Downloads", "Desktop", "Games", "AppData"] {
            let path = home.join(folder);
            if path.exists() && !roots.contains(&path) {
                roots.push(path);
            }
        }
        for drive in b'A'..=b'Z' {
            let path = PathBuf::from(format!("{}:\\", drive as char));
            if path.exists() && !roots.contains(&path) {
                roots.push(path);
            }
        }
    }
    roots
}

fn collect_large(path: &Path, dev: u64, min: u64, output: &mut Vec<(u64, PathBuf)>) {
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let child = entry.path();
        let meta = match fs::symlink_metadata(&child) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() || !same_device(&child, dev) {
            continue;
        }
        if meta.is_file() && meta.len() >= min {
            output.push((meta.len(), child));
        } else if meta.is_dir() {
            collect_large(&child, dev, min, output);
        }
    }
}

fn write_large_files(out: &Path, roots: &[PathBuf], min: u64) -> io::Result<()> {
    let mut files = Vec::new();
    for root in roots {
        let root = match canonical(root) {
            Some(v) => v,
            None => continue,
        };
        if let Some(dev) = device(&root) {
            collect_large(&root, dev, min, &mut files);
        }
    }
    files.sort_by_key(|item| std::cmp::Reverse(item.0));
    files.truncate(1000);
    let mut file = File::create(out.join("large-files.tsv"))?;
    writeln!(file, "bytes\thuman\tpath")?;
    for (bytes, path) in files {
        writeln!(
            file,
            "{}\t{}\t{}",
            bytes,
            human_bytes(bytes),
            clean(&path.display().to_string())
        )?;
    }
    Ok(())
}

fn collect_files(
    path: &Path,
    dev: u64,
    predicate: fn(&Path) -> bool,
    result: &mut Vec<(u64, PathBuf)>,
) {
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let child = entry.path();
        let meta = match fs::symlink_metadata(&child) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() || !same_device(&child, dev) {
            continue;
        }
        if meta.is_file() && predicate(&child) {
            result.push((meta.len(), child));
        } else if meta.is_dir() {
            collect_files(&child, dev, predicate, result);
        }
    }
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        extensions
            .iter()
            .any(|wanted| e.eq_ignore_ascii_case(wanted))
    })
}

fn write_appimages(out: &Path, roots: &[PathBuf]) -> io::Result<()> {
    let mut file = File::create(out.join("appimages.tsv"))?;
    writeln!(file, "bytes\thuman\tpath")?;
    let mut rows = Vec::new();
    for root in roots {
        if let (Some(path), Some(dev)) = (canonical(root), device(root)) {
            collect_files(
                &path,
                dev,
                |p| has_extension(p, &["AppImage", "appimage"]),
                &mut rows,
            );
        }
    }
    rows.sort_by_key(|r| r.0);
    for (bytes, path) in rows {
        writeln!(
            file,
            "{}\t{}\t{}",
            bytes,
            human_bytes(bytes),
            clean(&path.display().to_string())
        )?;
    }
    Ok(())
}

fn write_installers(out: &Path, roots: &[PathBuf]) -> io::Result<()> {
    let mut file = File::create(out.join("installers-and-archives.tsv"))?;
    writeln!(file, "bytes\thuman\tformat\tpath")?;
    let mut rows = Vec::new();
    for root in roots {
        if let (Some(path), Some(dev)) = (canonical(root), device(root)) {
            collect_files(
                &path,
                dev,
                |p| {
                    has_extension(
                        p,
                        &[
                            "iso", "zip", "7z", "rar", "tar", "gz", "xz", "bz2", "deb", "rpm",
                            "pkg", "msi", "exe",
                        ],
                    )
                },
                &mut rows,
            );
        }
    }
    rows.sort_by_key(|r| r.0);
    for (bytes, path) in rows {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("-");
        writeln!(
            file,
            "{}\t{}\t{}\t{}",
            bytes,
            human_bytes(bytes),
            ext,
            clean(&path.display().to_string())
        )?;
    }
    Ok(())
}

fn write_virtual_machines(out: &Path, roots: &[PathBuf]) -> io::Result<()> {
    let mut file = File::create(out.join("virtual-machines.tsv"))?;
    writeln!(file, "bytes\thuman\tformat\tpath")?;
    let mut rows = Vec::new();
    for root in roots {
        if let (Some(path), Some(dev)) = (canonical(root), device(root)) {
            collect_files(
                &path,
                dev,
                |p| has_extension(p, &["vmdk", "vdi", "qcow2", "ova", "ovf", "vmx", "vbox"]),
                &mut rows,
            );
        }
    }
    rows.sort_by_key(|r| r.0);
    for (bytes, path) in rows {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("-");
        writeln!(
            file,
            "{}\t{}\t{}\t{}",
            bytes,
            human_bytes(bytes),
            ext,
            clean(&path.display().to_string())
        )?;
    }
    Ok(())
}

fn write_desktops(out: &Path, home: &Path) -> io::Result<()> {
    let mut file = File::create(out.join("desktop-applications.tsv"))?;
    writeln!(file, "bytes\thuman\tpath")?;
    for root in [
        home.join(".local/share/applications"),
        PathBuf::from("/usr/share/applications"),
    ] {
        let mut rows = Vec::new();
        if let (Some(path), Some(dev)) = (canonical(&root), device(&root)) {
            collect_files(&path, dev, |p| has_extension(p, &["desktop"]), &mut rows);
            for (bytes, path) in rows {
                writeln!(
                    file,
                    "{}\t{}\t{}",
                    bytes,
                    human_bytes(bytes),
                    path.display()
                )?;
            }
        }
    }
    Ok(())
}

fn write_directory_usage(out: &Path, roots: &[PathBuf]) -> io::Result<()> {
    let mut file = File::create(out.join("directory-usage.tsv"))?;
    writeln!(file, "bytes\thuman\tpath")?;
    for root in roots {
        let root = match canonical(root) {
            Some(v) => v,
            None => continue,
        };
        let dev = device(&root);
        let entries = match fs::read_dir(&root) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let mut rows = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                rows.push((directory_size(&path, dev), path));
            }
        }
        rows.sort_by_key(|r| std::cmp::Reverse(r.0));
        for (bytes, path) in rows {
            writeln!(
                file,
                "{}\t{}\t{}",
                bytes,
                human_bytes(bytes),
                path.display()
            )?;
        }
    }
    Ok(())
}

fn write_duplicates(out: &Path, roots: &[PathBuf], min: u64) -> io::Result<()> {
    let mut file = File::create(out.join("duplicates.tsv"))?;
    writeln!(file, "hash\tbytes\thuman\tpath")?;
    if !command_exists("sha256sum") {
        writeln!(file, "-\t0\t-\tsha256sum no disponible")?;
        return Ok(());
    }
    let mut sized: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for root in roots {
        if let (Some(path), Some(dev)) = (canonical(root), device(root)) {
            let mut rows = Vec::new();
            collect_files(&path, dev, |_| true, &mut rows);
            for (bytes, path) in rows.into_iter().filter(|r| r.0 >= min) {
                sized.entry(bytes).or_default().push(path);
            }
        }
    }
    for (bytes, paths) in sized.into_iter().filter(|(_, paths)| paths.len() > 1) {
        let mut hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for path in paths {
            if let Some(hash) = command_output_owned("sha256sum", &[path.display().to_string()]) {
                if let Some(hash) = hash.split_whitespace().next() {
                    hashes.entry(hash.to_string()).or_default().push(path);
                }
            }
        }
        for (hash, paths) in hashes.into_iter().filter(|(_, paths)| paths.len() > 1) {
            for path in paths {
                writeln!(
                    file,
                    "{}\t{}\t{}\t{}",
                    hash,
                    bytes,
                    human_bytes(bytes),
                    clean(&path.display().to_string())
                )?;
            }
        }
    }
    Ok(())
}

pub fn write_prefix_report(out: &Path, prefixes: &[PrefixInfo]) -> io::Result<()> {
    let mut file = File::create(out.join("wine-prefixes.tsv"))?;
    writeln!(
        file,
        "bytes\thuman\tdrive_c_bytes\tdrive_c_human\ttype\tsteam_appid\tmount_root\tprefix"
    )?;
    for p in prefixes {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            p.bytes,
            human_bytes(p.bytes),
            p.drive_c_bytes,
            human_bytes(p.drive_c_bytes),
            p.kind,
            p.appid,
            p.mount_root,
            p.path.display()
        )?;
    }
    Ok(())
}

pub fn run(ctx: &Context, args: &[String], games: bool) -> Result<(), String> {
    let mut roots = Vec::new();
    let mut full = false;
    let mut include_home = true;
    let mut auto_mounts = true;
    let mut out = None;
    let mut min_size = if games { 0 } else { 100 * 1024 * 1024 };
    let mut duplicates = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--full" => full = true,
            "--duplicates" => duplicates = true,
            "--root" => {
                i += 1;
                roots.push(PathBuf::from(
                    args.get(i).ok_or("--root requiere una ruta")?,
                ));
            }
            "--out" => {
                i += 1;
                out = Some(PathBuf::from(args.get(i).ok_or("--out requiere una ruta")?));
            }
            "--min-size-mb" => {
                i += 1;
                min_size = args
                    .get(i)
                    .ok_or("--min-size-mb requiere un número")?
                    .parse::<u64>()
                    .map_err(|_| "--min-size-mb requiere un número")?
                    * 1024
                    * 1024;
            }
            "--no-home" => include_home = false,
            "--no-mounts" => auto_mounts = false,
            "--dry-run" | "--plan" => {
                if args[i] == "--plan" {
                    i += 1;
                }
            }
            other => return Err(format!("opción desconocida: {other}")),
        }
        i += 1;
    }
    if full
        && auto_mounts
        && roots.is_empty()
        && !cfg!(windows)
        && !command_exists("findmnt")
        && !crate::common::ensure_tool(ctx, "findmnt")?
    {
        println!("findmnt no está disponible; se omitirá la detección automática de montajes.");
        auto_mounts = false;
    }
    if duplicates && !command_exists("sha256sum") && !crate::common::ensure_tool(ctx, "sha256sum")?
    {
        return Err("sha256sum es necesario para buscar duplicados".into());
    }
    let roots = default_roots(&ctx.home, full, &roots, include_home, auto_mounts);
    let out = out.unwrap_or_else(|| {
        PathBuf::from(format!(
            "rust-{}-{}",
            if games { "games" } else { "audit" },
            crate::common::timestamp()
        ))
    });
    fs::create_dir_all(&out).map_err(|e| format!("no se pudo crear el informe: {e}"))?;
    let phase_total = if duplicates { 7 } else { 6 };
    println!(
        "Auditoría {}: {} ruta(s); las rutas grandes pueden tardar.",
        if full { "completa" } else { "rápida" },
        roots.len()
    );
    for root in &roots {
        println!("  Ruta: {}", root.display());
    }
    println!("Fase 1/{phase_total}: detectando prefijos...");
    let prefixes = discover_prefixes(&roots);
    write_prefix_report(&out, &prefixes).map_err(|e| e.to_string())?;
    println!("Fase 2/{phase_total}: archivos grandes...");
    write_large_files(&out, &roots, min_size).map_err(|e| e.to_string())?;
    println!("Fase 3/{phase_total}: AppImages...");
    write_appimages(&out, &roots).map_err(|e| e.to_string())?;
    println!("Fase 4/{phase_total}: instaladores y archivos...");
    write_installers(&out, &roots).map_err(|e| e.to_string())?;
    println!("Fase 5/{phase_total}: máquinas virtuales...");
    write_virtual_machines(&out, &roots).map_err(|e| e.to_string())?;
    println!("Fase 6/{phase_total}: aplicaciones y uso de directorios...");
    write_desktops(&out, &ctx.home).map_err(|e| e.to_string())?;
    write_directory_usage(&out, &roots).map_err(|e| e.to_string())?;
    if duplicates {
        println!("Fase 7/{phase_total}: duplicados por SHA-256...");
        write_duplicates(&out, &roots, min_size).map_err(|e| e.to_string())?;
    }
    let mut summary = File::create(out.join("summary.txt")).map_err(|e| e.to_string())?;
    writeln!(summary, "ltools-rs {}", crate::VERSION).map_err(|e| e.to_string())?;
    writeln!(summary, "Modo: {}", if full { "full" } else { "quick" })
        .map_err(|e| e.to_string())?;
    writeln!(summary, "Rutas escaneadas:").map_err(|e| e.to_string())?;
    for root in &roots {
        writeln!(summary, "  {}", root.display()).map_err(|e| e.to_string())?;
    }
    writeln!(summary, "Prefijos detectados: {}", prefixes.len()).map_err(|e| e.to_string())?;
    writeln!(summary, "Informe: {}", out.display()).map_err(|e| e.to_string())?;
    println!("Informe: {}", out.display());
    println!("Prefijos detectados: {}", prefixes.len());
    println!("Solo lectura: no se han modificado datos de origen.");
    if let Some(plan) = &ctx.plan {
        plan.record(
            "audit",
            &out,
            "executed",
            true,
            "solo lectura",
            if ctx.dry_run { "dry-run" } else { "audit" },
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
