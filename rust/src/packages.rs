use crate::common::{
    ask, command_exists, command_output, command_output_owned, ensure_tool, human_bytes,
    move_to_trash, run_command, run_with_sudo, Context,
};
use crate::i18n;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn query(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .env("LC_ALL", "C")
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_string()
        })
        .unwrap_or_default()
}

fn normalize_manager(value: &str) -> &str {
    match value {
        "apt" => "apt-get",
        "xbps" => "xbps-remove",
        other => other,
    }
}

fn manager_requires_process_elevation(manager: &str) -> bool {
    // Brew is per-user; Flatpak and Pamac use their own authorization paths;
    // AUR helpers must build as the user and elevate only their package step.
    // Do not switch HOME or the desktop session.
    !matches!(manager, "brew" | "flatpak" | "pamac" | "paru" | "yay")
}

fn run_package_manager(
    manager: &str,
    program: &str,
    args: &[String],
    dry_run: bool,
) -> std::io::Result<bool> {
    if manager_requires_process_elevation(manager) {
        run_with_sudo(program, args, dry_run)
    } else {
        run_command(program, args, dry_run)
    }
}

fn removal_command(manager: &str) -> Option<(&'static str, Vec<String>)> {
    Some(match manager {
        "pacman" => ("pacman", vec!["-Rns".into()]),
        "apt-get" => ("apt-get", vec!["remove".into()]),
        "dnf" => ("dnf", vec!["remove".into()]),
        "yum" => ("yum", vec!["remove".into()]),
        "zypper" => ("zypper", vec!["remove".into()]),
        "apk" => ("apk", vec!["del".into()]),
        "xbps-remove" => ("xbps-remove", vec!["-R".into()]),
        "brew" => ("brew", vec!["uninstall".into()]),
        "snap" => ("snap", vec!["remove".into()]),
        "flatpak" => ("flatpak", vec!["uninstall".into(), "--delete-data".into()]),
        "pamac" => ("pamac", vec!["remove".into()]),
        _ => return None,
    })
}

const REMOVAL_MANAGERS: &[&str] = &[
    "pacman",
    "apt-get",
    "dnf",
    "yum",
    "zypper",
    "apk",
    "xbps-remove",
    "pamac",
    "brew",
    "snap",
    "flatpak",
];

fn available_removal_managers() -> Vec<&'static str> {
    REMOVAL_MANAGERS
        .iter()
        .copied()
        .filter(|manager| command_exists(manager))
        .collect()
}

fn resolve_removal_manager(requested: Option<&str>, available: &[&str]) -> Result<String, String> {
    if let Some(requested) = requested {
        let normalized = normalize_manager(requested);
        return available
            .contains(&normalized)
            .then(|| normalized.to_owned())
            .ok_or_else(|| format!("el gestor no está disponible: {requested}"));
    }
    match available {
        [] => Err("no se encontró un gestor de paquetes compatible".into()),
        [manager] => Ok((*manager).to_owned()),
        managers => Err(format!(
            "hay varios gestores disponibles ({}); especifica --manager para evitar usar el incorrecto",
            managers.join(", ")
        )),
    }
}

fn parse_flatpak_installations(contents: &str) -> Vec<String> {
    let mut installations = Vec::new();
    for line in contents.lines().map(str::trim) {
        let Some(name) = line
            .strip_prefix("[Installation \"")
            .and_then(|value| value.strip_suffix("\"]"))
        else {
            continue;
        };
        if !name.is_empty() && name != "default" && !installations.iter().any(|found| found == name)
        {
            installations.push(name.to_owned());
        }
    }
    installations
}

fn custom_flatpak_installations() -> Vec<String> {
    custom_flatpak_installations_from(Path::new("/etc/flatpak/installations.d"))
}

fn custom_flatpak_installations_from(directory: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut installations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "conf")
        {
            if let Ok(contents) = fs::read_to_string(path) {
                installations.extend(parse_flatpak_installations(&contents));
            }
        }
    }
    installations.sort();
    installations.dedup();
    installations
}

fn flatpak_query_args(scope: &str) -> Vec<String> {
    match scope {
        "user" => vec![
            "list".into(),
            "--user".into(),
            "--columns=application".into(),
        ],
        "system" => vec![
            "list".into(),
            "--system".into(),
            "--columns=application".into(),
        ],
        installation => vec![
            format!("--installation={installation}"),
            "list".into(),
            "--columns=application".into(),
        ],
    }
}

fn flatpak_unused_args(scope: &str) -> Vec<String> {
    match scope {
        "user" | "system" => vec!["uninstall".into(), "--unused".into(), format!("--{scope}")],
        installation => vec![
            format!("--installation={installation}"),
            "uninstall".into(),
            "--unused".into(),
        ],
    }
}

fn flatpak_scoped_removal_args(scope: &str, mut args: Vec<String>) -> Vec<String> {
    if matches!(scope, "user" | "system") {
        args.push(format!("--{scope}"));
    } else {
        args.insert(0, format!("--installation={scope}"));
    }
    args
}

fn resolve_flatpak_scope(requested: Option<&str>, installed: &[String]) -> Result<String, String> {
    if let Some(requested) = requested {
        return installed
            .iter()
            .find(|scope| scope.as_str() == requested)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "el paquete no aparece en el ámbito Flatpak «{requested}»; opciones: {}",
                    installed.join(", ")
                )
            });
    }
    match installed {
        [] => Err("el paquete no aparece en ninguna instalación Flatpak consultada".into()),
        [scope] => Ok(scope.clone()),
        scopes => Err(format!(
            "el paquete está instalado en varios ámbitos Flatpak ({}); especifica --scope",
            scopes.join(", ")
        )),
    }
}

fn validate_cascade_manager(manager: &str, cascade: bool) -> Result<(), String> {
    if cascade && manager != "pacman" {
        return Err("--cascade solo se admite con --manager pacman".into());
    }
    Ok(())
}

fn flatpak_scope_for_package(package: &str, requested: Option<&str>) -> Result<String, String> {
    let custom_installations = custom_flatpak_installations();
    if requested.is_some_and(|scope| {
        !matches!(scope, "user" | "system")
            && !custom_installations.iter().any(|name| name == scope)
    }) {
        return Err(format!(
            "--scope Flatpak debe ser user, system o una instalación personalizada disponible; opciones: user, system{}",
            if custom_installations.is_empty() { String::new() } else { format!(", {}", custom_installations.join(", ")) }
        ));
    }
    let mut installed_scopes = Vec::new();
    for scope in ["user", "system"] {
        if requested.is_some_and(|selected| selected != scope) {
            continue;
        }
        let args = flatpak_query_args(scope);
        let output = command_output_owned("flatpak", &args)
            .ok_or_else(|| format!("no se pudo consultar Flatpak {scope}"))?;
        if output.lines().any(|line| line.trim() == package) {
            installed_scopes.push(scope.to_owned());
        }
    }
    for installation in custom_installations {
        if requested.is_some_and(|selected| selected != installation) {
            continue;
        }
        let args = flatpak_query_args(&installation);
        let output = command_output_owned("flatpak", &args)
            .ok_or_else(|| format!("no se pudo consultar Flatpak {installation}"))?;
        if output.lines().any(|line| line.trim() == package) {
            installed_scopes.push(installation);
        }
    }
    resolve_flatpak_scope(requested, &installed_scopes)
}

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let mut out = None;
    let mut package_only = false;
    let mut full = false;
    let mut view_report = false;
    for i in 0..args.len() {
        match args[i].as_str() {
            "--out" => out = args.get(i + 1).map(PathBuf::from),
            "--packages-only" => package_only = true,
            "--full" => full = true,
            "--view-report" => view_report = true,
            "--dry-run" | "--plan" => {}
            other if other.starts_with('-') => return Err(format!("opción desconocida: {other}")),
            _ => {}
        }
    }
    let managed_output = out.is_none();
    let out = out.unwrap_or_else(|| crate::common::default_report_dir(&ctx.home, "packages"));
    if managed_output {
        crate::common::reset_default_report_dir(&out)
            .map_err(|e| format!("no se pudo preparar el informe: {e}"))?;
    }
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut inventory = File::create(out.join("inventory.tsv")).map_err(|e| e.to_string())?;
    writeln!(inventory, "kind\tscope\tmanager\tdata\tpath\tbytes").map_err(|e| e.to_string())?;
    let mut managers = if full {
        let mut file = File::create(out.join("package-managers.tsv")).map_err(|e| e.to_string())?;
        writeln!(file, "manager\tpath\tstatus").map_err(|e| e.to_string())?;
        Some(file)
    } else {
        None
    };
    let known = [
        "pacman",
        "paru",
        "yay",
        "pikaur",
        "apt",
        "dpkg-query",
        "rpm",
        "dnf",
        "yum",
        "zypper",
        "apk",
        "xbps-query",
        "xbps-remove",
        "pkg",
        "snap",
        "flatpak",
        "brew",
        "pamac",
        "nix-env",
        "guix",
        "eopkg",
        "emerge",
    ];
    for manager in known {
        if command_exists(manager) {
            let path = command_output("sh", &["-c", &format!("command -v -- {manager}")])
                .unwrap_or_else(|| manager.into());
            writeln!(inventory, "manager\tsystem\t{manager}\tinstalled\t{path}\t")
                .map_err(|e| e.to_string())?;
            if let Some(file) = managers.as_mut() {
                writeln!(file, "{manager}\t{path}\tinstalled").map_err(|e| e.to_string())?;
            }
        }
    }
    let queries = [
        ("packages-pacman.tsv", "pacman", vec!["-Q"], "system"),
        (
            "packages-pacman-foreign.tsv",
            "pacman",
            vec!["-Qm"],
            "user/AUR",
        ),
        (
            "packages-pacman-orphans.tsv",
            "pacman",
            vec!["-Qdtq"],
            "orphan",
        ),
        (
            "packages-pacman-explicit.tsv",
            "pacman",
            vec!["-Qqe"],
            "explicit",
        ),
        (
            "packages-dpkg.tsv",
            "dpkg-query",
            vec![
                "-W",
                "-f=${binary:Package}\t${Version}\t${Installed-Size}\\n",
            ],
            "system",
        ),
        (
            "packages-rpm.tsv",
            "rpm",
            vec!["-qa", "--qf", "%{NAME}\t%{VERSION}-%{RELEASE}\t%{SIZE}\\n"],
            "system",
        ),
        (
            "packages-flatpak.tsv",
            "flatpak",
            vec![
                "list",
                "--app",
                "--columns=application,version,installation",
            ],
            "user/system",
        ),
        ("packages-snap.tsv", "snap", vec!["list"], "system"),
        (
            "packages-brew.tsv",
            "brew",
            vec!["list", "--formula"],
            "user",
        ),
        ("packages-nix.tsv", "nix-env", vec!["-q"], "user"),
    ];
    for (filename, program, query_args, scope) in &queries {
        collect_query_rows(&mut inventory, program, query_args, scope);
        if full {
            collect_query(&out, filename, program, query_args, scope);
        }
    }
    collect_artifact_rows(&mut inventory, &ctx.home);
    if full {
        collect_artifacts(&out, &ctx.home);
    }
    let mut summary = File::create(out.join("summary.txt")).map_err(|e| e.to_string())?;
    writeln!(summary, "ltools-rs package inventory").map_err(|e| e.to_string())?;
    writeln!(
        summary,
        "Modo: {}",
        if package_only { "packages" } else { "full" }
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        summary,
        "Formato: {}",
        if full {
            "compacto + detallado"
        } else {
            "compacto"
        }
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        summary,
        "Inventario principal: {}",
        out.join("inventory.tsv").display()
    )
    .map_err(|e| e.to_string())?;
    writeln!(summary, "Informe: {}", out.display()).map_err(|e| e.to_string())?;
    println!("Informe de paquetes: {}", out.display());
    println!(
        "Inventario principal: {}",
        out.join("inventory.tsv").display()
    );
    if ctx.dry_run
        || ctx
            .plan
            .as_ref()
            .is_some_and(crate::common::Plan::is_explicit)
    {
        if let Some(plan) = &ctx.plan {
            plan.record(
                "package-audit",
                &out,
                if ctx.dry_run { "planned" } else { "observed" },
                false,
                "solo lectura",
                "inventory",
            )
            .map_err(|e| e.to_string())?;
        }
    }
    if view_report {
        crate::report::interactive(&out)?;
    }
    Ok(())
}

fn collect_query_rows(inventory: &mut File, program: &str, args: &[&str], scope: &str) {
    if !command_exists(program) {
        return;
    }
    for line in query(program, args).lines() {
        let data = line.replace(['\t', '\r', '\n'], " ");
        let _ = writeln!(inventory, "package\t{scope}\t{program}\t{data}\t\t");
    }
}

fn collect_query(out: &Path, name: &str, program: &str, args: &[&str], scope: &str) {
    let mut file = match File::create(out.join(name)) {
        Ok(v) => v,
        Err(_) => return,
    };
    let _ = writeln!(file, "scope\tmanager\tdata");
    if !command_exists(program) {
        return;
    }
    for line in query(program, args).lines() {
        let _ = writeln!(file, "{}\t{}\t{}", scope, program, line.replace('\t', " "));
    }
}

fn collect_artifacts(out: &Path, home: &Path) {
    let mut file = match File::create(out.join("package-artifacts.tsv")) {
        Ok(v) => v,
        Err(_) => return,
    };
    let _ = writeln!(file, "scope\tformat\tbytes\thuman\tpath");
    let roots = [
        PathBuf::from("/var/cache/pacman/pkg"),
        PathBuf::from("/var/cache/apt/archives"),
        PathBuf::from("/var/cache/dnf"),
        home.join(".cache/paru"),
        home.join(".cache/yay"),
        home.join(".cache/pikaur"),
    ];
    for root in roots {
        collect_artifacts_dir(&root, &mut file, 0);
    }
}

fn collect_artifact_rows(inventory: &mut File, home: &Path) {
    let roots = [
        PathBuf::from("/var/cache/pacman/pkg"),
        PathBuf::from("/var/cache/apt/archives"),
        PathBuf::from("/var/cache/dnf"),
        home.join(".cache/paru"),
        home.join(".cache/yay"),
        home.join(".cache/pikaur"),
    ];
    for root in roots {
        collect_artifact_rows_dir(&root, inventory, 0);
    }
}

fn collect_artifact_rows_dir(path: &Path, inventory: &mut File, depth: usize) {
    if depth > 6 {
        return;
    }
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
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_artifact_rows_dir(&child, inventory, depth + 1);
            continue;
        }
        let ext = child
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_lowercase();
        let format = match ext.as_str() {
            "pkg" | "zst" | "xz" | "gz" => "arch",
            "deb" => "deb",
            "rpm" => "rpm",
            "apk" => "apk",
            "txz" => "pkg",
            _ => continue,
        };
        let scope = if child.starts_with("/var") {
            "system"
        } else {
            "user"
        };
        let path_text = crate::common::clean(&child.display().to_string());
        let _ = writeln!(
            inventory,
            "artifact\t{scope}\t{format}\t{format}\t{path_text}\t{}",
            meta.len()
        );
    }
}

fn collect_artifacts_dir(path: &Path, file: &mut File, depth: usize) {
    if depth > 6 {
        return;
    }
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
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_artifacts_dir(&child, file, depth + 1);
            continue;
        }
        let ext = child
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_lowercase();
        let format = match ext.as_str() {
            "pkg" | "zst" | "xz" | "gz" => "arch",
            "deb" => "deb",
            "rpm" => "rpm",
            "apk" => "apk",
            "txz" => "pkg",
            _ => continue,
        };
        let scope = if child.starts_with("/var") {
            "system"
        } else {
            "user"
        };
        let _ = writeln!(
            file,
            "{scope}\t{format}\t{}\t{}\t{}",
            meta.len(),
            human_bytes(meta.len()),
            child.display()
        );
    }
}

pub fn clean(ctx: &Context, args: &[String]) -> Result<(), String> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--automatic" | "--smart"))
    {
        return crate::cleaner::run(ctx, args);
    }
    if args.iter().any(|arg| arg == "menu") {
        return menu(ctx);
    }
    let mut packages = Vec::new();
    let mut paths = Vec::new();
    let mut orphans = false;
    let mut caches = false;
    let mut flatpak_unused = false;
    let mut preview = false;
    let mut force = false;
    let mut cascade = false;
    let mut manager = None;
    let mut scope = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--package" => {
                i += 1;
                packages.push(args.get(i).ok_or("--package requiere un nombre")?.clone());
            }
            "--path" => {
                i += 1;
                paths.push(PathBuf::from(
                    args.get(i).ok_or("--path requiere una ruta")?,
                ));
            }
            "--orphans" => orphans = true,
            "--package-caches" | "--pacman-cache" => caches = true,
            "--flatpak-unused" => flatpak_unused = true,
            "--preview" => preview = true,
            "--force" => force = true,
            "--cascade" => cascade = true,
            "--manager" => {
                i += 1;
                manager = Some(args.get(i).ok_or("--manager requiere un gestor")?.clone());
            }
            "--scope" => {
                i += 1;
                scope = Some(
                    args.get(i)
                        .ok_or("--scope requiere user, system o una instalación personalizada")?
                        .clone(),
                );
            }
            "--dry-run" | "--plan" => {
                if args[i] == "--plan" {
                    i += 1;
                }
            }
            "--menu" => {}
            other => return Err(format!("opción desconocida: {other}")),
        }
        i += 1;
    }
    if preview {
        return preview_clean(ctx);
    }
    if orphans
        && manager
            .as_deref()
            .is_some_and(|selected| normalize_manager(selected) != "pacman")
    {
        return Err(
            "--orphans corresponde a pacman; no combines esa opción con otro --manager".into(),
        );
    }
    if scope.is_some() && packages.is_empty() {
        return Err("--scope solo se puede usar con --package".into());
    }
    if manager.is_some() && packages.is_empty() && !orphans {
        return Err("--manager solo se puede usar con --package o --orphans".into());
    }
    if cascade && packages.is_empty() && !orphans {
        return Err("--cascade solo se puede usar con --package o --orphans".into());
    }
    for package in packages {
        remove_package(ctx, &package, cascade, manager.as_deref(), scope.as_deref())?;
    }
    if orphans {
        if command_exists("pacman") {
            let orphan_packages = query("pacman", &["-Qdtq"]);
            for package in orphan_packages.lines() {
                remove_package(ctx, package, cascade, Some("pacman"), None)?;
            }
        } else {
            eprintln!("--orphans solo está disponible cuando pacman está instalado.");
        }
    }
    for path in paths {
        if !force && referenced(&path, &ctx.home) {
            eprintln!(
                "Bloqueado: hay referencias a {}. Usa --force tras revisarlas.",
                path.display()
            );
            continue;
        }
        if !force && !ensure_tool(ctx, "rg")? {
            eprintln!(
                "No se elimina {} sin poder comprobar referencias. Usa --force solo tras revisarlo manualmente.",
                path.display()
            );
            continue;
        }
        if !ensure_tool(ctx, "trash")? {
            eprintln!(
                "No se elimina {} sin una papelera compatible.",
                path.display()
            );
            continue;
        }
        if ask(&format!("¿Mover {} a la papelera?", path.display()))
            && move_to_trash(&path, ctx.dry_run).map_err(|e| e.to_string())?
        {
            if let Some(plan) = &ctx.plan {
                plan.record(
                    "trash-move",
                    &path,
                    if ctx.dry_run { "planned" } else { "executed" },
                    false,
                    "papelera",
                    "",
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    if caches {
        clean_caches(ctx)?;
    }
    if flatpak_unused {
        run_flatpak_unused(ctx)?;
    }
    Ok(())
}

/// Presenta las posibles fuentes de limpieza sin ejecutar ninguna operación.
/// Es el contrato que usa la GUI: no abre un menú que dependa de stdin ni
/// inicia gestores de paquetes con una acción implícita.
fn preview_clean(ctx: &Context) -> Result<(), String> {
    println!("=== Revisión segura de limpieza ===");
    let managers = [
        ("pacman", "/var/cache/pacman/pkg"),
        ("apt-get", "/var/cache/apt/archives"),
        ("dnf", "/var/cache/dnf"),
        ("zypper", "/var/cache/zypp"),
        ("apk", "/var/cache/apk"),
        ("xbps-remove", "/var/cache/xbps"),
        ("brew", "brew-cache"),
        ("pamac", "caché Pamac (conserva las tres últimas versiones)"),
        ("flatpak", "runtimes sin uso"),
    ];
    let mut found = false;
    for (manager, target) in managers {
        if command_exists(manager) {
            found = true;
            println!("  Detectado: {manager} · revisar {target}");
        }
    }
    for name in ["paru", "yay", "pikaur", "trizen", "aura"] {
        let path = ctx.home.join(format!(".cache/{name}"));
        if path.is_dir() {
            found = true;
            println!("  Caché AUR: {} · {}", name, path.display());
        }
    }
    if command_exists("pacman") {
        let orphans = query("pacman", &["-Qdtq"]);
        if !orphans.trim().is_empty() {
            found = true;
            println!(
                "  Paquetes huérfanos detectados: {}",
                orphans.lines().count()
            );
        }
    }
    if !found {
        println!("No se detectaron fuentes de limpieza gestionables.");
    }
    println!("No se ha modificado ningún paquete, caché ni ruta.");
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        println!("\n{}", i18n::text("menu.clean.title"));
        println!("  1) {}", i18n::text("menu.clean.orphans"));
        println!("  2) {}", i18n::text("menu.clean.caches"));
        println!("  3) {}", i18n::text("menu.clean.flatpak"));
        println!("  4) {}", i18n::text("menu.clean.path"));
        println!("  5) {}", i18n::text("menu.clean.package"));
        println!("  6) Limpiador automático guiado (cachés, temporales y datos opcionales)");
        println!("  q) {}", i18n::text("menu.back"));
        print!("{}", i18n::text("menu.prompt"));
        let _ = std::io::stdout().flush();
        let mut answer = String::new();
        if std::io::stdin()
            .read_line(&mut answer)
            .map_err(|e| e.to_string())?
            == 0
        {
            return Ok(());
        }
        match answer.trim().to_lowercase().as_str() {
            "1" => clean(ctx, &["--orphans".into()])?,
            "2" => clean(ctx, &["--package-caches".into()])?,
            "3" => clean(ctx, &["--flatpak-unused".into()])?,
            "4" => {
                if let Some(path) = crate::common::prompt_path("Ruta: ") {
                    clean(ctx, &["--path".into(), path.display().to_string()])?;
                }
            }
            "5" => {
                if let Some(package) = crate::common::prompt_path("Paquete: ") {
                    let managers = available_removal_managers();
                    let manager = match managers.as_slice() {
                        [manager] => Some((*manager).to_owned()),
                        [] => None,
                        _ => {
                            println!("Gestores disponibles: {}", managers.join(", "));
                            print!("Gestor exacto (Enter para cancelar): ");
                            let _ = std::io::stdout().flush();
                            let mut selected = String::new();
                            std::io::stdin()
                                .read_line(&mut selected)
                                .map_err(|error| error.to_string())?;
                            let selected = selected.trim();
                            if selected.is_empty() {
                                println!("Operación cancelada.");
                                continue;
                            }
                            Some(selected.to_owned())
                        }
                    };
                    let mut remove_args = vec!["--package".into(), package.display().to_string()];
                    if manager.as_deref() == Some("flatpak") {
                        let installations = custom_flatpak_installations();
                        print!(
                            "Ámbito Flatpak [auto/user/system{}] (Enter para cancelar): ",
                            if installations.is_empty() {
                                String::new()
                            } else {
                                format!("/{}", installations.join("/"))
                            }
                        );
                        let _ = std::io::stdout().flush();
                        let mut selected_scope = String::new();
                        std::io::stdin()
                            .read_line(&mut selected_scope)
                            .map_err(|error| error.to_string())?;
                        let selected_scope = selected_scope.trim();
                        if selected_scope.is_empty() {
                            println!("Operación cancelada.");
                            continue;
                        }
                        if selected_scope != "auto" {
                            remove_args.extend(["--scope".into(), selected_scope.to_owned()]);
                        }
                    }
                    if let Some(manager) = manager {
                        remove_args.extend(["--manager".into(), manager]);
                    }
                    clean(ctx, &remove_args)?;
                }
            }
            "6" => clean(ctx, &["--automatic".into()])?,
            "" | "q" | "b" | "back" | "volver" | "retour" | "zurück" | "voltar" | "indietro"
            | "torna" | "terug" | "wstecz" => return Ok(()),
            _ => println!("{}", i18n::text("menu.invalid")),
        }
    }
}

fn remove_package(
    ctx: &Context,
    package: &str,
    cascade: bool,
    requested_manager: Option<&str>,
    requested_scope: Option<&str>,
) -> Result<(), String> {
    let available = available_removal_managers();
    let manager = resolve_removal_manager(requested_manager, &available)?;
    validate_cascade_manager(&manager, cascade)?;
    let mut has_dependents = false;
    let dependency_note: String;
    if manager == "pacman" {
        let info = query("pacman", &["-Qi", package]);
        if info.is_empty() {
            eprintln!("No está instalado: {package}");
            return Ok(());
        }
        let required = info
            .lines()
            .find(|l| l.starts_with("Required By"))
            .unwrap_or("");
        has_dependents = !required.contains("None")
            && required
                .split(':')
                .nth(1)
                .is_some_and(|v| !v.trim().is_empty());
        dependency_note = required.to_string();
        if has_dependents && !cascade {
            eprintln!("No se elimina {package}: tiene dependientes ({required}). Usa --cascade tras revisarlos.");
            return Ok(());
        }
    } else {
        dependency_note = "el gestor resolverá dependencias; revisar su resumen".into();
    }
    let (program, mut args) = removal_command(&manager)
        .ok_or_else(|| format!("gestor no soportado para eliminar: {manager}"))?;
    let flatpak_scope = if manager == "flatpak" {
        Some(flatpak_scope_for_package(package, requested_scope)?)
    } else {
        if requested_scope.is_some() {
            return Err("--scope solo se admite al eliminar un paquete Flatpak".into());
        }
        None
    };
    if let Some(scope) = flatpak_scope.as_deref() {
        args = flatpak_scoped_removal_args(scope, args);
    }
    let display = format!("{program} {}", args.join(" "));
    if !ask(&format!(
        "¿Eliminar {package} con {display}? Dependencias: {dependency_note}"
    )) {
        return Ok(());
    }
    if manager == "pacman" && has_dependents {
        args.push("-c".into());
    }
    args.push("--".into());
    args.push(package.into());
    let result = run_package_manager(&manager, program, &args, ctx.dry_run);
    let ok = result.as_ref().is_ok_and(|executed| *executed);
    let plan_detail = if let Some(scope) = flatpak_scope.as_deref() {
        format!("{dependency_note}; Flatpak {scope}")
    } else if has_dependents {
        "cascade".to_owned()
    } else {
        dependency_note.clone()
    };
    if let Some(plan) = &ctx.plan {
        plan.record(
            "package-remove",
            Path::new(package),
            if ctx.dry_run {
                "planned"
            } else if ok {
                "executed"
            } else {
                "failed"
            },
            false,
            &manager,
            &plan_detail,
        )
        .map_err(|e| e.to_string())?;
    }
    if !ctx.dry_run {
        match result {
            Err(error) => {
                return Err(format!(
                    "no se pudo iniciar la eliminación de {package} con {manager}: {error}"
                ));
            }
            Ok(false) => {
                return Err(format!(
                    "no se completó la eliminación de {package} con {manager}; revisa los permisos y la salida del gestor"
                ));
            }
            Ok(true) => {}
        }
    }
    Ok(())
}

fn clean_caches(ctx: &Context) -> Result<(), String> {
    let mut failures = Vec::new();
    if command_exists("pacman") && !command_exists("paccache") && !ensure_tool(ctx, "paccache")? {
        eprintln!("No se podrá limpiar la caché de pacman sin paccache.");
    }
    if command_exists("paccache")
        && ask("¿Limpiar la caché de pacman conservando las dos últimas versiones?")
    {
        let args = vec!["-rk2".into()];
        let result = run_with_sudo("paccache", &args, ctx.dry_run);
        let executed = result.as_ref().is_ok_and(|success| *success);
        if !executed && !ctx.dry_run {
            failures.push("pacman (paccache)".to_owned());
            match &result {
                Err(error) => eprintln!("No se pudo iniciar paccache: {error}"),
                Ok(false) => eprintln!(
                    "No se completó la limpieza de pacman: paccache terminó con error o la elevación fue rechazada."
                ),
                Ok(true) => {}
            }
        }
        if let Some(p) = &ctx.plan {
            p.record(
                "package-cache-clean",
                Path::new("/var/cache/pacman/pkg"),
                if ctx.dry_run {
                    "planned"
                } else if executed {
                    "executed"
                } else {
                    "failed"
                },
                false,
                "paccache",
                "",
            )
            .map_err(|e| e.to_string())?;
        }
    }
    for (manager, command, args, path) in [
        (
            "apt",
            "apt-get",
            vec!["clean".into()],
            "/var/cache/apt/archives",
        ),
        (
            "dnf",
            "dnf",
            vec!["clean".into(), "all".into()],
            "/var/cache/dnf",
        ),
        (
            "zypper",
            "zypper",
            vec!["clean".into(), "--all".into()],
            "/var/cache/zypp",
        ),
        (
            "apk",
            "apk",
            vec!["cache".into(), "clean".into()],
            "/var/cache/apk",
        ),
        ("xbps", "xbps-remove", vec!["-O".into()], "/var/cache/xbps"),
        ("brew", "brew", vec!["cleanup".into()], "brew-cache"),
        (
            "pamac",
            "pamac",
            vec!["clean".into(), "--keep".into(), "3".into()],
            "pamac-cache",
        ),
    ] {
        // Pamac and paccache manage the same pacman cache; avoid offering a
        // second cleaner that could remove the same packages twice.
        if manager == "pamac" && command_exists("paccache") {
            continue;
        }
        if command_exists(command) && ask(&format!("¿Ejecutar limpieza de {manager}?")) {
            let result = run_package_manager(manager, command, &args, ctx.dry_run);
            let executed = result.as_ref().is_ok_and(|success| *success);
            if !executed && !ctx.dry_run {
                failures.push(manager.to_owned());
                match &result {
                    Err(error) => eprintln!("No se pudo iniciar la limpieza de {manager}: {error}"),
                    Ok(false) => eprintln!(
                        "No se completó la limpieza de {manager}: el comando terminó con error o la elevación fue rechazada."
                    ),
                    Ok(true) => {}
                }
            }
            if let Some(p) = &ctx.plan {
                p.record(
                    "package-cache-clean",
                    Path::new(path),
                    if ctx.dry_run {
                        "planned"
                    } else if executed {
                        "executed"
                    } else {
                        "failed"
                    },
                    false,
                    command,
                    "",
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    for name in ["paru", "yay", "pikaur", "trizen", "aura"] {
        let path = ctx.home.join(format!(".cache/{name}"));
        if path.is_dir()
            && ask(&format!("¿Mover la caché de {name} a la papelera?"))
            && move_to_trash(&path, ctx.dry_run).map_err(|e| e.to_string())?
        {
            if let Some(p) = &ctx.plan {
                p.record(
                    "package-cache-trash",
                    &path,
                    if ctx.dry_run { "planned" } else { "executed" },
                    false,
                    name,
                    "AUR/build cache",
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    if !failures.is_empty() {
        return Err(format!(
            "no se completaron las limpiezas de caché: {}",
            failures.join(", ")
        ));
    }
    Ok(())
}

fn run_flatpak_unused(ctx: &Context) -> Result<(), String> {
    if !command_exists("flatpak") {
        if cfg!(windows) {
            eprintln!("Flatpak no aplica a Windows; no se modifica nada.");
            return Ok(());
        }
        if !ensure_tool(ctx, "flatpak")? {
            eprintln!("Flatpak no está disponible; no se modifica nada.");
            return Ok(());
        }
    }
    let mut scopes = vec!["user".to_owned(), "system".to_owned()];
    scopes.extend(custom_flatpak_installations());
    if ask(&format!(
        "¿Eliminar runtimes Flatpak sin uso en las instalaciones {}?",
        scopes.join(", ")
    )) {
        let mut failures = Vec::new();
        for scope in scopes {
            let args = flatpak_unused_args(&scope);
            let result = run_command("flatpak", &args, ctx.dry_run);
            let executed = result.as_ref().is_ok_and(|success| *success);
            if !executed && !ctx.dry_run {
                failures.push(scope.clone());
                match &result {
                    Err(error) => eprintln!("No se pudo iniciar Flatpak para {scope}: {error}"),
                    Ok(false) => {
                        eprintln!("Flatpak no completó la limpieza de runtimes sin uso en {scope}.")
                    }
                    Ok(true) => {}
                }
            }
            if let Some(p) = &ctx.plan {
                p.record(
                    "flatpak-unused",
                    Path::new(&format!("flatpak:{scope}")),
                    if ctx.dry_run {
                        "planned"
                    } else if executed {
                        "executed"
                    } else {
                        "failed"
                    },
                    false,
                    "flatpak uninstall --unused",
                    &scope,
                )
                .map_err(|e| e.to_string())?;
            }
        }
        if !failures.is_empty() {
            return Err(format!(
                "Flatpak no completó la limpieza de runtimes sin uso en: {}",
                failures.join(", ")
            ));
        }
    }
    Ok(())
}

fn referenced(path: &Path, home: &Path) -> bool {
    if !command_exists("rg") {
        return false;
    }
    let roots = [
        home.join(".config"),
        home.join(".local/share/lutris"),
        home.join(".local/share/umu"),
        home.join(".var/app"),
    ];
    Command::new("rg")
        .args(["-F", "-l", "--hidden", "--no-messages", "--"])
        .arg(path)
        .args(roots)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{
        custom_flatpak_installations_from, flatpak_query_args, flatpak_scoped_removal_args,
        flatpak_unused_args, manager_requires_process_elevation, normalize_manager,
        parse_flatpak_installations, removal_command, resolve_flatpak_scope,
        resolve_removal_manager, validate_cascade_manager,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn package_removal_never_guesses_between_multiple_managers() {
        assert_eq!(resolve_removal_manager(None, &["brew"]).unwrap(), "brew");
        assert!(resolve_removal_manager(None, &["brew", "flatpak"]).is_err());
        assert_eq!(
            resolve_removal_manager(Some("flatpak"), &["brew", "flatpak"]).unwrap(),
            "flatpak"
        );
        assert_eq!(
            resolve_removal_manager(Some("xbps"), &["xbps-remove"]).unwrap(),
            "xbps-remove"
        );
        assert!(resolve_removal_manager(None, &[]).is_err());
    }

    #[test]
    fn flatpak_removal_scope_is_explicit_when_ref_is_ambiguous() {
        let user = vec!["user".to_owned()];
        let system = vec!["system".to_owned()];
        let both = vec!["user".to_owned(), "system".to_owned()];
        let custom = vec!["user".to_owned(), "extra".to_owned()];
        assert_eq!(resolve_flatpak_scope(None, &user).unwrap(), "user");
        assert_eq!(resolve_flatpak_scope(None, &system).unwrap(), "system");
        assert!(resolve_flatpak_scope(None, &both).is_err());
        assert_eq!(
            resolve_flatpak_scope(Some("system"), &both).unwrap(),
            "system"
        );
        assert!(resolve_flatpak_scope(Some("user"), &system).is_err());
        assert_eq!(
            resolve_flatpak_scope(Some("extra"), &custom).unwrap(),
            "extra"
        );
        assert!(resolve_flatpak_scope(Some("invalid"), &user).is_err());
    }

    #[test]
    fn flatpak_custom_installation_config_names_are_parsed() {
        let parsed = parse_flatpak_installations(
            "[Installation \"extra\"]\nPath=/mnt/flatpak\n\n[Installation \"default\"]\nPath=/var/lib/flatpak\n[Installation \"extra\"]\n",
        );
        assert_eq!(parsed, vec!["extra"]);
    }

    #[test]
    fn flatpak_custom_installations_are_discovered_only_from_conf_files() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "ltools-flatpak-installations-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("create isolated test directory");
        fs::write(
            directory.join("extra.conf"),
            "[Installation \"extra\"]\nPath=/mnt/flatpak\n[Installation \"default\"]\n",
        )
        .expect("write valid Flatpak installation config");
        fs::write(
            directory.join("ignored.txt"),
            "[Installation \"must-not-appear\"]\n",
        )
        .expect("write ignored non-config file");
        fs::write(
            directory.join("duplicate.conf"),
            "[Installation \"extra\"]\nPath=/other/flatpak\n",
        )
        .expect("write duplicate Flatpak installation config");

        assert_eq!(custom_flatpak_installations_from(&directory), vec!["extra"]);
        fs::remove_dir_all(&directory).expect("remove isolated test directory");
    }

    #[test]
    fn flatpak_scope_is_passed_as_native_arguments_for_each_operation() {
        assert_eq!(
            flatpak_query_args("user"),
            vec!["list", "--user", "--columns=application"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            flatpak_query_args("extra"),
            vec!["--installation=extra", "list", "--columns=application"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            flatpak_unused_args("system"),
            vec!["uninstall", "--unused", "--system"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            flatpak_unused_args("extra"),
            vec!["--installation=extra", "uninstall", "--unused"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            flatpak_scoped_removal_args("user", vec!["uninstall".into()]),
            vec!["uninstall", "--user"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            flatpak_scoped_removal_args("extra", vec!["uninstall".into()]),
            vec!["--installation=extra", "uninstall"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn user_scoped_package_managers_keep_the_callers_identity() {
        assert!(!manager_requires_process_elevation("brew"));
        assert!(!manager_requires_process_elevation("flatpak"));
        assert!(!manager_requires_process_elevation("pamac"));
        assert!(!manager_requires_process_elevation("paru"));
        assert!(!manager_requires_process_elevation("yay"));
        assert!(manager_requires_process_elevation("apt-get"));
        assert!(manager_requires_process_elevation("pacman"));
    }

    #[test]
    fn cascade_removal_is_pacman_only_instead_of_being_silently_ignored() {
        assert!(validate_cascade_manager("pacman", true).is_ok());
        assert!(validate_cascade_manager("brew", true).is_err());
        assert!(validate_cascade_manager("flatpak", true).is_err());
        assert!(validate_cascade_manager("brew", false).is_ok());
    }

    #[test]
    fn normalizes_manager_aliases_without_changing_real_commands() {
        assert_eq!(normalize_manager("apt"), "apt-get");
        assert_eq!(normalize_manager("xbps"), "xbps-remove");
        assert_eq!(normalize_manager("dnf"), "dnf");
        assert_eq!(normalize_manager("pamac"), "pamac");
    }

    #[test]
    fn removal_commands_target_the_selected_manager() {
        assert_eq!(removal_command("dnf").unwrap().0, "dnf");
        assert_eq!(removal_command("yum").unwrap().0, "yum");
        assert_eq!(removal_command("pamac").unwrap().0, "pamac");
        assert_eq!(removal_command("apt-get").unwrap().0, "apt-get");
        assert_eq!(removal_command("pacman").unwrap().0, "pacman");
        assert!(removal_command("unsupported").is_none());
    }
}
