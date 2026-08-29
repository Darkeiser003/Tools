use crate::audit::{default_roots, discover_prefixes, prefix_kind};
use crate::common::{
    ask, backup, canonical, critical_path, device, directory_size, file_contains, human_bytes,
    move_to_trash, run_command, Context,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str).unwrap_or("list") {
        "list" | "ls" => list(ctx, &args[1..]),
        "inspect" | "info" => inspect(ctx, &args[1..]),
        "create" => create(ctx, &args[1..]),
        "migrate" | "clone" => migrate(ctx, &args[1..]),
        action => Err(format!("acción de prefijo desconocida: {action}")),
    }
}

fn list(ctx: &Context, args: &[String]) -> Result<(), String> {
    let roots = parse_roots(ctx, args);
    let prefixes = discover_prefixes(&roots);
    println!("Prefijos detectados: {}", prefixes.len());
    for (n, p) in prefixes.iter().enumerate() {
        println!(
            "{:3}) {:18} {:>8}  {}",
            n + 1,
            p.kind,
            human_bytes(p.bytes),
            p.path.display()
        );
    }
    Ok(())
}

fn inspect(ctx: &Context, args: &[String]) -> Result<(), String> {
    let path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .or_else(|| crate::common::prompt_path("Ruta del prefijo: "))
        .ok_or("falta la ruta")?;
    let path = canonical(&path).ok_or("el prefijo no existe")?;
    if !looks_like_prefix(&path) {
        return Err("no parece un prefijo Wine".into());
    }
    println!(
        "Prefijo: {}\nTipo: {}\nTamaño: {}",
        path.display(),
        prefix_kind(&path),
        human_bytes(directory_size(&path, device(&path)))
    );
    for item in [
        "system.reg",
        "user.reg",
        "userdef.reg",
        "drive_c",
        "dosdevices",
        "config_info",
    ] {
        if path.join(item).exists() {
            println!("  contiene: {item}");
        }
    }
    println!(
        "Ejecutables/instaladores: {}",
        count_suffix(&path.join("drive_c"), &["exe", "msi"])
    );
    let _ = ctx;
    Ok(())
}

fn create(ctx: &Context, args: &[String]) -> Result<(), String> {
    let dest = value(args, "--dest")
        .map(PathBuf::from)
        .or_else(|| crate::common::prompt_path("Ruta del nuevo prefijo: "))
        .ok_or("falta --dest")?;
    validate_destination(&dest)?;
    let arch = value(args, "--arch").unwrap_or_else(|| "win64".into());
    println!("Se creará un prefijo {arch} en {}", dest.display());
    if !ask("¿Crear el prefijo?") {
        return Ok(());
    }
    if let Some(plan) = &ctx.plan {
        plan.record(
            "create-prefix",
            &dest,
            "planned",
            true,
            "wineboot -u",
            &arch,
        )
        .map_err(|e| e.to_string())?;
    }
    if ctx.dry_run {
        println!("Simulación: no se ejecutaría wineboot.");
        return Ok(());
    }
    if !crate::common::command_exists("wineboot") {
        return Err("wineboot no está instalado".into());
    }
    let ok = Command::new("wineboot")
        .env("WINEARCH", &arch)
        .env("WINEPREFIX", &dest)
        .arg("-u")
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if !ok || !looks_like_prefix(&dest) {
        return Err("wineboot no creó un prefijo válido".into());
    }
    println!("Prefijo creado y validado: {}", dest.display());
    Ok(())
}

fn migrate(ctx: &Context, args: &[String]) -> Result<(), String> {
    let source = value_any(args, &["--source", "--from"])
        .map(PathBuf::from)
        .or_else(|| crate::common::prompt_path("Ruta del prefijo origen: "))
        .ok_or("falta --source")?;
    let dest = value_any(args, &["--dest", "--destination", "--target"])
        .map(PathBuf::from)
        .or_else(|| crate::common::prompt_path("Ruta del nuevo prefijo: "))
        .ok_or("falta --dest")?;
    let source = canonical(&source).ok_or("el origen no existe")?;
    if !looks_like_prefix(&source) {
        return Err("el origen no parece un prefijo Wine".into());
    }
    if prefix_kind(&source) == "steam-proton" && !args.iter().any(|a| a == "--allow-steam") {
        return Err("Steam/Proton requiere --allow-steam".into());
    }
    if source
        .to_string_lossy()
        .ends_with("/files/share/default_pfx")
    {
        return Err("se bloquean plantillas default_pfx compartidas".into());
    }
    if critical_path(&source) && !args.iter().any(|a| a == "--allow-mount-root") {
        return Err("origen crítico o raíz de montaje; requiere --allow-mount-root".into());
    }
    if critical_path(&source) && !args.iter().any(|a| a == "--include") {
        return Err("una raíz de montaje solo admite rescate con --include explícitos".into());
    }
    if has_locks(&source)
        && !args.iter().any(|a| a == "--force")
        && !ask("Hay bloqueos posibles. ¿Has cerrado Wine, Steam, Lutris y Heroic?")
    {
        return Err("migración cancelada por posibles bloqueos".into());
    }
    validate_destination(&dest)?;
    if dest.starts_with(&source) {
        return Err("el destino está dentro del origen".into());
    }
    let items = selected_items(&source, args)?;
    let bytes: u64 = items
        .iter()
        .map(|i| directory_size(&source.join(i), device(&source)))
        .sum();
    let set_defaults = args
        .iter()
        .any(|a| a == "--set-defaults" || a == "--activate-shell");
    check_space(&dest, bytes)?;
    println!(
        "Tipo: {}\nOrigen: {}\nDestino: {}\nTamaño: {}",
        prefix_kind(&source),
        source.display(),
        dest.display(),
        human_bytes(bytes)
    );
    println!("Contenido directo: {}", items.join(", "));
    if set_defaults {
        println!("Después: actualizar WINEPREFIX/winetricks con backup.");
    }
    if args.iter().any(|a| a == "--rewrite-configs") {
        println!("Después: respaldar y actualizar referencias de lanzadores.");
    }
    if args.iter().any(|a| a == "--update-launchers") {
        println!("Después: actualizar defaults compatibles de Heroic con backup.");
    }
    if args.iter().any(|a| a == "--remove-source") {
        println!("Después: ofrecer retirar el origen a papelera.");
    }
    if args.iter().any(|a| a == "--activate-shell") {
        println!("Después: ofrecer activar WINEPREFIX en las shells y environment.d, con backup.");
    }
    if let Some(plan) = &ctx.plan {
        if ctx.dry_run {
            if set_defaults {
                plan.record(
                    "write-defaults",
                    &ctx.home
                        .join(".config/wine-prefix-manager/default-prefix.sh"),
                    "planned",
                    true,
                    "defaults",
                    &dest.display().to_string(),
                )
                .map_err(|e| e.to_string())?;
            }
            if args.iter().any(|a| a == "--activate-shell") {
                plan.record(
                    "activate-shell",
                    &ctx.home.join(".bashrc"),
                    "planned",
                    true,
                    "bashrc zshrc environment.d",
                    &dest.display().to_string(),
                )
                .map_err(|e| e.to_string())?;
            }
            if args.iter().any(|a| a == "--update-launchers") {
                plan.record(
                    "update-launchers",
                    &ctx.home.join(".config/heroic"),
                    "planned",
                    true,
                    "Heroic/Lutris/UMU/Steam references",
                    &dest.display().to_string(),
                )
                .map_err(|e| e.to_string())?;
            }
            if args.iter().any(|a| a == "--rewrite-configs") {
                plan.record(
                    "rewrite-configs",
                    &ctx.home.join(".config"),
                    "planned",
                    true,
                    &source.display().to_string(),
                    &dest.display().to_string(),
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    if !ask("¿Continuar con esta migración?") {
        return Ok(());
    }
    if let Some(plan) = &ctx.plan {
        plan.record(
            "copy-prefix",
            &dest,
            "planned",
            true,
            &source.display().to_string(),
            &items.join(" "),
        )
        .map_err(|e| e.to_string())?;
    }
    if ctx.dry_run {
        println!("Simulación: no se copiaría ni modificaría el origen.");
        return Ok(());
    }
    if !crate::common::command_exists("rsync") {
        return Err("rsync es necesario para verificar la copia".into());
    }
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    for item in &items {
        let args = vec![
            "-aH".into(),
            "--info=progress2".into(),
            "--partial".into(),
            source.join(item).display().to_string(),
            format!("{}/", dest.display()),
        ];
        if !run_command("rsync", &args, false).map_err(|e| e.to_string())? {
            return Err(format!("falló la copia de {item}"));
        }
    }
    for item in &items {
        let args = vec![
            "-aHn".into(),
            "--delete".into(),
            "--itemize-changes".into(),
            source.join(item).display().to_string(),
            format!("{}/", dest.display()),
        ];
        let output = Command::new("rsync")
            .args(&args)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() || !String::from_utf8_lossy(&output.stdout).trim().is_empty() {
            return Err(format!("verificación con diferencias en {item}"));
        }
    }
    if !looks_like_prefix(&dest) {
        return Err("el destino no parece un prefijo".into());
    }
    println!("Copia verificada correctamente.");
    if let Some(plan) = &ctx.plan {
        plan.record(
            "copy-prefix",
            &dest,
            "executed",
            true,
            &source.display().to_string(),
            &items.join(" "),
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(plan) = &ctx.plan {
        plan.record(
            "remove-created",
            &dest,
            "executed",
            true,
            "",
            "copia migrada",
        )
        .map_err(|e| e.to_string())?;
    }
    if set_defaults {
        write_defaults(ctx, &dest, prefix_kind(&source) == "steam-proton")?;
        if args.iter().any(|a| a == "--activate-shell") {
            activate_shell(ctx, &dest)?;
        }
    }
    if args.iter().any(|a| a == "--update-launchers") {
        update_heroic(ctx, &dest)?;
        // Los defaults estructurados de Heroic se tratan aparte; las
        // referencias por juego de Lutris, UMU, Steam y otros lanzadores se
        // actualizan mediante el mismo recorrido respaldado.
        rewrite_configs(ctx, &source, &dest)?;
    } else if args.iter().any(|a| a == "--rewrite-configs") {
        rewrite_configs(ctx, &source, &dest)?;
    }
    if args.iter().any(|a| a == "--remove-source")
        && ask("¿Mover el origen verificado a la papelera?")
        && move_to_trash(&source, false).map_err(|e| e.to_string())?
    {
        if let Some(plan) = &ctx.plan {
            plan.record(
                "trash-move",
                &source,
                "executed",
                false,
                "papelera",
                "origen migrado",
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn parse_roots(ctx: &Context, args: &[String]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for pair in args.windows(2) {
        if pair[0] == "--root" {
            roots.push(PathBuf::from(&pair[1]));
        }
    }
    default_roots(
        &ctx.home,
        args.iter().any(|a| a == "--full"),
        &roots,
        !args.iter().any(|a| a == "--no-home"),
        !args.iter().any(|a| a == "--no-mounts"),
    )
}

fn value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}
fn value_any(args: &[String], keys: &[&str]) -> Option<String> {
    args.windows(2)
        .find(|w| keys.iter().any(|key| w[0] == *key))
        .map(|w| w[1].clone())
}
fn looks_like_prefix(path: &Path) -> bool {
    path.is_dir() && (path.join("system.reg").is_file() || path.join("drive_c").is_dir())
}
fn validate_destination(path: &Path) -> Result<(), String> {
    if critical_path(path) {
        return Err(format!("destino bloqueado: {}", path.display()));
    }
    if path.exists()
        && (!path.is_dir()
            || fs::read_dir(path)
                .map_err(|e| e.to_string())?
                .next()
                .is_some())
    {
        return Err("el destino debe ser inexistente o estar vacío".into());
    }
    Ok(())
}

fn selected_items(source: &Path, args: &[String]) -> Result<Vec<String>, String> {
    let include: Vec<String> = args
        .windows(2)
        .filter(|w| w[0] == "--include")
        .map(|w| w[1].clone())
        .collect();
    let exclude: Vec<String> = args
        .windows(2)
        .filter(|w| w[0] == "--exclude")
        .map(|w| w[1].clone())
        .collect();
    let mut items: Vec<String> = if include.is_empty() {
        fs::read_dir(source)
            .map_err(|e| e.to_string())?
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .collect()
    } else {
        include
    };
    items.retain(|item| {
        !item.contains('/') && source.join(item).exists() && !exclude.contains(item)
    });
    items.sort();
    items.dedup();
    if items.is_empty() {
        Err("no hay elementos seleccionados".into())
    } else {
        Ok(items)
    }
}

fn count_suffix(path: &Path, suffixes: &[&str]) -> usize {
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    entries
        .flatten()
        .map(|e| {
            let p = e.path();
            if p.is_dir() {
                count_suffix(&p, suffixes)
            } else if p
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| suffixes.iter().any(|s| x.eq_ignore_ascii_case(s)))
            {
                1
            } else {
                0
            }
        })
        .sum()
}

fn has_locks(path: &Path) -> bool {
    fn walk(path: &Path, depth: usize) -> bool {
        if depth > 4 {
            return false;
        }
        let entries = match fs::read_dir(path) {
            Ok(v) => v,
            Err(_) => return false,
        };
        for entry in entries.flatten() {
            let child = entry.path();
            let name = child.file_name().and_then(|v| v.to_str()).unwrap_or("");
            if name == "lock" || name.ends_with(".lock") || name.ends_with(".lck") {
                println!("Posible bloqueo: {}", child.display());
                return true;
            }
            if child.is_dir() && walk(&child, depth + 1) {
                return true;
            }
        }
        false
    }
    walk(path, 0)
}

fn check_space(dest: &Path, required: u64) -> Result<(), String> {
    let parent = dest.parent().unwrap_or(Path::new("/"));
    let output = Command::new("df")
        .args(["-Pk"])
        .arg(parent)
        .output()
        .map_err(|e| e.to_string())?;
    let binding = String::from_utf8_lossy(&output.stdout);
    let line = binding.lines().last().unwrap_or("");
    let available_kb = line
        .split_whitespace()
        .nth(3)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    println!(
        "Espacio disponible: {}",
        human_bytes(available_kb.saturating_mul(1024))
    );
    if available_kb.saturating_mul(1024) <= required {
        return Err("no hay espacio suficiente en el destino".into());
    }
    Ok(())
}

fn write_defaults(ctx: &Context, dest: &Path, steam_proton: bool) -> Result<(), String> {
    let dir = ctx.home.join(".config/wine-prefix-manager");
    let file = dir.join("default-prefix.sh");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let old = if file.exists() {
        Some(backup(&file).map_err(|e| e.to_string())?)
    } else {
        None
    };
    let mut output = File::create(&file).map_err(|e| e.to_string())?;
    writeln!(
        output,
        "# Generado por ltools-rs\nexport WINEPREFIX='{}'\nwine-prefix() {{ WINEPREFIX=\"$WINEPREFIX\" wine \"$@\"; }}\nwinetricks-prefix() {{ WINEPREFIX=\"$WINEPREFIX\" winetricks \"$@\"; }}",
        dest.display().to_string().replace('\'', "'\\''")
    )
    .map_err(|e| e.to_string())?;
    if steam_proton {
        let compat = dest.parent().unwrap_or(dest);
        writeln!(
            output,
            "export PROTON_COMPAT_DATA_PATH='{}'\nproton-prefix() {{ STEAM_COMPAT_DATA_PATH=\"$PROTON_COMPAT_DATA_PATH\" proton \"$@\"; }}",
            compat.display().to_string().replace('\'', "'\\''")
        )
        .map_err(|e| e.to_string())?;
    } else {
        writeln!(
            output,
            "# Steam administra compatdata por AppID; no se fija un Proton global aquí."
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(plan) = &ctx.plan {
        if let Some(old) = old {
            plan.record(
                "restore-file",
                &file,
                "executed",
                true,
                &old.display().to_string(),
                "defaults",
            )
            .map_err(|e| e.to_string())?;
        } else {
            plan.record("remove-created", &file, "executed", true, "", "defaults")
                .map_err(|e| e.to_string())?;
        }
    }
    println!("Default Wine/winetricks actualizado: {}", file.display());
    Ok(())
}

fn activate_shell(ctx: &Context, dest: &Path) -> Result<(), String> {
    if !ask("¿Activar este WINEPREFIX en tus shells y en environment.d?") {
        println!("No se modificaron las shells.");
        return Ok(());
    }
    let env_file = ctx
        .home
        .join(".config/wine-prefix-manager/default-prefix.sh");
    for rc_name in [".bashrc", ".zshrc"] {
        let rc = ctx.home.join(rc_name);
        if !rc.is_file() {
            continue;
        }
        let data = fs::read_to_string(&rc).map_err(|e| e.to_string())?;
        let marker = env_file.display().to_string();
        if data.contains(&marker) {
            println!("Ya estaba activado: {}", rc.display());
            continue;
        }
        let old = backup(&rc).map_err(|e| e.to_string())?;
        let mut output = data;
        output.push_str(&format!(
            "\n# LTools: prefijo Wine gestionado\nsource '{}'\n",
            marker.replace('\'', "'\\''")
        ));
        fs::write(&rc, output).map_err(|e| e.to_string())?;
        if let Some(plan) = &ctx.plan {
            plan.record(
                "restore-file",
                &rc,
                "executed",
                true,
                &old.display().to_string(),
                "shell",
            )
            .map_err(|e| e.to_string())?;
        }
    }
    let environment_dir = ctx.home.join(".config/environment.d");
    fs::create_dir_all(&environment_dir).map_err(|e| e.to_string())?;
    let environment_file = environment_dir.join("90-ltools-wine.conf");
    let old = if environment_file.exists() {
        Some(backup(&environment_file).map_err(|e| e.to_string())?)
    } else {
        None
    };
    fs::write(
        &environment_file,
        format!("# LTools\nWINEPREFIX={}\n", dest.display()),
    )
    .map_err(|e| e.to_string())?;
    if let Some(plan) = &ctx.plan {
        if let Some(old) = old {
            plan.record(
                "restore-file",
                &environment_file,
                "executed",
                true,
                &old.display().to_string(),
                "environment.d",
            )
            .map_err(|e| e.to_string())?;
        } else {
            plan.record(
                "remove-created",
                &environment_file,
                "executed",
                true,
                "",
                "environment.d",
            )
            .map_err(|e| e.to_string())?;
        }
    }
    println!(
        "Defaults activados en shells disponibles y {}",
        environment_file.display()
    );
    Ok(())
}

fn update_heroic(ctx: &Context, dest: &Path) -> Result<(), String> {
    let expr = "walk(if type == \"object\" then (if has(\"defaultWinePrefix\") then .defaultWinePrefix=$parent else . end | if has(\"defaultWinePrefixDir\") then .defaultWinePrefixDir=$parent else . end | if has(\"winePrefix\") then .winePrefix=$prefix else . end) else . end)";
    for file in [
        ctx.home.join(".config/heroic/config.json"),
        ctx.home.join(".config/heroic/store/config.json"),
    ] {
        if !file.is_file() {
            continue;
        }
        let parent = dest.parent().unwrap_or(dest).display().to_string();
        if ctx.dry_run {
            println!("Simulación: se actualizaría Heroic: {}", file.display());
            continue;
        }
        if !crate::common::command_exists("jq") {
            if !crate::common::command_exists("perl")
                || parent.contains(['"', '\\'])
                || dest.display().to_string().contains(['"', '\\'])
            {
                eprintln!(
                    "Heroic detectado, pero no hay un actualizador seguro disponible: {}",
                    file.display()
                );
                continue;
            }
            let old = backup(&file).map_err(|e| e.to_string())?;
            let script = r#"
                my $parent = $ENV{CACHYOS_HEROIC_PARENT};
                my $prefix = $ENV{CACHYOS_HEROIC_PREFIX};
                s/("defaultWinePrefix"\s*:\s*")[^"]*(")/$1 . $parent . $2/ge;
                s/("defaultWinePrefixDir"\s*:\s*")[^"]*(")/$1 . $parent . $2/ge;
                s/("winePrefix"\s*:\s*")[^"]*(")/$1 . $prefix . $2/ge;
            "#;
            let ok = Command::new("perl")
                .env("CACHYOS_HEROIC_PARENT", &parent)
                .env("CACHYOS_HEROIC_PREFIX", dest.display().to_string())
                .args(["-0pi", "-e", script])
                .arg(&file)
                .status()
                .map_err(|e| e.to_string())?
                .success();
            if ok {
                if let Some(plan) = &ctx.plan {
                    plan.record(
                        "restore-file",
                        &file,
                        "executed",
                        true,
                        &old.display().to_string(),
                        "Heroic/perl",
                    )
                    .map_err(|e| e.to_string())?;
                }
                println!(
                    "Heroic actualizado con compatibilidad Perl: {}",
                    file.display()
                );
            } else {
                eprintln!(
                    "No se pudo actualizar Heroic; se conserva el backup: {}",
                    old.display()
                );
            }
            continue;
        }
        let old = backup(&file).map_err(|e| e.to_string())?;
        let tmp = file.with_extension("json.ltools-tmp");
        let status = Command::new("jq")
            .args([
                "--arg",
                "prefix",
                &dest.display().to_string(),
                "--arg",
                "parent",
                &parent,
                expr,
            ])
            .arg(&file)
            .stdout(File::create(&tmp).map_err(|e| e.to_string())?)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            let _ = fs::remove_file(&tmp);
            return Err(format!("no se pudo actualizar Heroic: {}", file.display()));
        }
        fs::rename(&tmp, &file).map_err(|e| e.to_string())?;
        if let Some(plan) = &ctx.plan {
            plan.record(
                "restore-file",
                &file,
                "executed",
                true,
                &old.display().to_string(),
                "Heroic",
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn rewrite_configs(ctx: &Context, old: &Path, new: &Path) -> Result<(), String> {
    let roots = [
        ctx.home.join(".config"),
        ctx.home.join(".local/share/lutris"),
        ctx.home.join(".local/share/umu"),
        ctx.home.join(".var/app"),
    ];
    let mut files = Vec::new();
    for root in roots {
        find_text_candidates(&root, &mut files, 0);
    }
    let old = old.display().to_string();
    let new = new.display().to_string();
    for file in files {
        if !file_contains(&file, &old) {
            continue;
        }
        let data = match fs::read_to_string(&file) {
            Ok(data) => data,
            Err(_) => {
                println!("Omitido por no ser texto UTF-8: {}", file.display());
                continue;
            }
        };
        if ctx.dry_run {
            println!("Simulación: se actualizaría {}", file.display());
            continue;
        }
        let backup = backup(&file).map_err(|e| e.to_string())?;
        fs::write(&file, replace_path_boundaries(&data, &old, &new)).map_err(|e| e.to_string())?;
        if let Some(plan) = &ctx.plan {
            plan.record(
                "restore-file",
                &file,
                "executed",
                true,
                &backup.display().to_string(),
                "launcher config",
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn find_text_candidates(path: &Path, output: &mut Vec<PathBuf>, depth: usize) {
    if depth > 7 {
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let meta = match fs::symlink_metadata(&p) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            find_text_candidates(&p, output, depth + 1);
        } else if matches!(
            p.extension().and_then(|x| x.to_str()),
            Some("json" | "yml" | "yaml" | "vdf" | "acf" | "conf" | "ini")
        ) {
            output.push(p);
        }
    }
}
fn replace_path_boundaries(data: &str, old: &str, new: &str) -> String {
    let mut out = String::new();
    let mut rest = data;
    while let Some(pos) = rest.find(old) {
        out.push_str(&rest[..pos]);
        let before = out.chars().last();
        let after = rest[pos + old.len()..].chars().next();
        let valid = before.is_none_or(|c| !c.is_ascii_alphanumeric() && !"_.-".contains(c))
            && after.is_none_or(|c| !c.is_ascii_alphanumeric() && !"_.-".contains(c));
        out.push_str(if valid { new } else { old });
        rest = &rest[pos + old.len()..];
    }
    out.push_str(rest);
    out
}
