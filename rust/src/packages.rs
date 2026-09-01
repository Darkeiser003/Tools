use crate::common::{
    ask, command_exists, command_output, ensure_tool, human_bytes, move_to_trash, run_command,
    run_with_sudo, Context,
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

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let mut out = None;
    let mut package_only = false;
    for i in 0..args.len() {
        match args[i].as_str() {
            "--out" => out = args.get(i + 1).map(PathBuf::from),
            "--packages-only" => package_only = true,
            "--dry-run" | "--plan" => {}
            "--full" => {}
            other if other.starts_with('-') => return Err(format!("opción desconocida: {other}")),
            _ => {}
        }
    }
    let out = out.unwrap_or_else(|| {
        PathBuf::from(format!("rust-package-audit-{}", crate::common::timestamp()))
    });
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut managers = File::create(out.join("package-managers.tsv")).map_err(|e| e.to_string())?;
    writeln!(managers, "manager\tpath\tstatus").map_err(|e| e.to_string())?;
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
            writeln!(
                managers,
                "{manager}\t{}\tinstalled",
                command_output("sh", &["-c", &format!("command -v -- {manager}")])
                    .unwrap_or_else(|| manager.into())
            )
            .map_err(|e| e.to_string())?;
        }
    }
    collect_query(&out, "packages-pacman.tsv", "pacman", &["-Q"], "system");
    collect_query(
        &out,
        "packages-pacman-foreign.tsv",
        "pacman",
        &["-Qm"],
        "user/AUR",
    );
    collect_query(
        &out,
        "packages-pacman-orphans.tsv",
        "pacman",
        &["-Qdtq"],
        "orphan",
    );
    collect_query(
        &out,
        "packages-pacman-explicit.tsv",
        "pacman",
        &["-Qqe"],
        "explicit",
    );
    collect_query(
        &out,
        "packages-dpkg.tsv",
        "dpkg-query",
        &[
            "-W",
            "-f=${binary:Package}\t${Version}\t${Installed-Size}\\n",
        ],
        "system",
    );
    collect_query(
        &out,
        "packages-rpm.tsv",
        "rpm",
        &["-qa", "--qf", "%{NAME}\t%{VERSION}-%{RELEASE}\t%{SIZE}\\n"],
        "system",
    );
    collect_query(
        &out,
        "packages-flatpak.tsv",
        "flatpak",
        &[
            "list",
            "--app",
            "--columns=application,version,installation",
        ],
        "user/system",
    );
    collect_query(&out, "packages-snap.tsv", "snap", &["list"], "system");
    collect_query(
        &out,
        "packages-brew.tsv",
        "brew",
        &["list", "--formula"],
        "user",
    );
    collect_query(&out, "packages-nix.tsv", "nix-env", &["-q"], "user");
    collect_artifacts(&out, &ctx.home);
    let mut summary = File::create(out.join("summary.txt")).map_err(|e| e.to_string())?;
    writeln!(summary, "ltools-rs package inventory").map_err(|e| e.to_string())?;
    writeln!(
        summary,
        "Modo: {}",
        if package_only { "packages" } else { "full" }
    )
    .map_err(|e| e.to_string())?;
    writeln!(summary, "Informe: {}", out.display()).map_err(|e| e.to_string())?;
    println!("Informe de paquetes: {}", out.display());
    if let Some(plan) = &ctx.plan {
        plan.record(
            "package-audit",
            &out,
            "executed",
            true,
            "solo lectura",
            "inventory",
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
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
    if args.iter().any(|arg| arg == "menu") {
        return menu(ctx);
    }
    let mut packages = Vec::new();
    let mut paths = Vec::new();
    let mut orphans = false;
    let mut caches = false;
    let mut flatpak_unused = false;
    let mut force = false;
    let mut cascade = false;
    let mut manager = None;
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
            "--force" => force = true,
            "--cascade" => cascade = true,
            "--manager" => {
                i += 1;
                manager = Some(args.get(i).ok_or("--manager requiere un gestor")?.clone());
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
    if orphans && command_exists("pacman") {
        packages.extend(query("pacman", &["-Qdtq"]).lines().map(str::to_string));
    }
    for package in packages {
        remove_package(ctx, &package, cascade, manager.as_deref())?;
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

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        println!("\n{}", i18n::text("menu.clean.title"));
        println!("  1) {}", i18n::text("menu.clean.orphans"));
        println!("  2) {}", i18n::text("menu.clean.caches"));
        println!("  3) {}", i18n::text("menu.clean.flatpak"));
        println!("  4) {}", i18n::text("menu.clean.path"));
        println!("  5) {}", i18n::text("menu.clean.package"));
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
                    clean(ctx, &["--package".into(), package.display().to_string()])?;
                }
            }
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
) -> Result<(), String> {
    let manager = requested_manager
        .map(normalize_manager)
        .map(str::to_string)
        .or_else(|| {
            [
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
            ]
            .iter()
            .find(|name| command_exists(name))
            .map(|name| (*name).to_string())
        })
        .ok_or("no se encontró un gestor de paquetes compatible")?;
    if !command_exists(&manager) {
        return Err(format!("el gestor no está disponible: {manager}"));
    }
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
    let ok = run_with_sudo(program, &args, ctx.dry_run).map_err(|e| e.to_string())?;
    if ok {
        if let Some(plan) = &ctx.plan {
            plan.record(
                "package-remove",
                Path::new(package),
                if ctx.dry_run { "planned" } else { "executed" },
                false,
                &manager,
                if has_dependents {
                    "cascade"
                } else {
                    &dependency_note
                },
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn clean_caches(ctx: &Context) -> Result<(), String> {
    if command_exists("pacman") && !command_exists("paccache") && !ensure_tool(ctx, "paccache")? {
        eprintln!("No se podrá limpiar la caché de pacman sin paccache.");
    }
    if command_exists("paccache")
        && ask("¿Limpiar la caché de pacman conservando las dos últimas versiones?")
    {
        let args = vec!["-rk2".into()];
        if run_with_sudo("paccache", &args, ctx.dry_run).map_err(|e| e.to_string())? {
            if let Some(p) = &ctx.plan {
                p.record(
                    "package-cache-clean",
                    Path::new("/var/cache/pacman/pkg"),
                    if ctx.dry_run { "planned" } else { "executed" },
                    false,
                    "paccache",
                    "",
                )
                .map_err(|e| e.to_string())?;
            }
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
    ] {
        if command_exists(command) && ask(&format!("¿Ejecutar limpieza de {manager}?")) {
            let _ = run_with_sudo(command, &args, ctx.dry_run).map_err(|e| e.to_string())?;
            if let Some(p) = &ctx.plan {
                p.record(
                    "package-cache-clean",
                    Path::new(path),
                    if ctx.dry_run { "planned" } else { "executed" },
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
    if ask("¿Eliminar runtimes Flatpak sin uso?") {
        let args = vec!["uninstall".into(), "--unused".into()];
        let _ = run_command("flatpak", &args, ctx.dry_run).map_err(|e| e.to_string())?;
        if let Some(p) = &ctx.plan {
            p.record(
                "flatpak-unused",
                Path::new("flatpak"),
                if ctx.dry_run { "planned" } else { "executed" },
                false,
                "flatpak uninstall --unused",
                "",
            )
            .map_err(|e| e.to_string())?;
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
    use super::{normalize_manager, removal_command};

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
