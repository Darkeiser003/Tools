//! Operaciones Git explícitas, sin shell y con protección para repositorios.

use crate::common::{self, Context};
use std::fs::{self, File, OpenOptions};
use std::io;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn help() -> &'static str {
    "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|gh|login [opciones seguras]"
}

struct PushOptions {
    repo: PathBuf,
    remote: Option<String>,
    branch: Option<String>,
    tags: bool,
    force_with_lease: bool,
    yes: bool,
}

struct BranchOptions {
    repo: PathBuf,
    operation: Option<String>,
    name: Option<String>,
    yes: bool,
}

struct TagOptions {
    repo: PathBuf,
    name: Option<String>,
    message: Option<String>,
    push: bool,
    remote: Option<String>,
    yes: bool,
}

struct ReleaseOptions {
    repo: Option<String>,
    tag: String,
    title: Option<String>,
    notes: Option<String>,
    yes: bool,
}

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let operation = args.first().map(String::as_str).unwrap_or("status");
    match operation {
        "status" => status(ctx, args.get(1..).unwrap_or_default()),
        "clone" => clone(ctx, args.get(1..).unwrap_or_default()),
        "fetch" => fetch(ctx, args.get(1..).unwrap_or_default()),
        "pull" => pull(ctx, args.get(1..).unwrap_or_default()),
        "log" => log_history(ctx, args.get(1..).unwrap_or_default()),
        "add" => add(ctx, args.get(1..).unwrap_or_default()),
        "commit" => commit(ctx, args.get(1..).unwrap_or_default()),
        "push" => push(ctx, args.get(1..).unwrap_or_default()),
        "branch" => branch(ctx, args.get(1..).unwrap_or_default()),
        "tag" => tag(ctx, args.get(1..).unwrap_or_default()),
        "release" => release(ctx, args.get(1..).unwrap_or_default()),
        "diagnose" | "doctor" => diagnose(ctx, args.get(1..).unwrap_or_default()),
        "repair" => repair(ctx, args.get(1..).unwrap_or_default()),
        "gh" | "github" => gh(ctx, args.get(1..).unwrap_or_default()),
        "login" | "auth" => login(ctx),
        _ => Err(format!("operación git desconocida: {operation}")),
    }
}

fn require_git(ctx: &Context) -> Result<(), String> {
    if common::command_exists("git") {
        Ok(())
    } else if ctx.dry_run {
        println!("Simulación: Git se prepararía antes de ejecutar esta acción.");
        Ok(())
    } else {
        println!("Git no está instalado; LTools puede ofrecer su instalación desde el gestor disponible.");
        if crate::common::ensure_tool(ctx, "git")? && common::command_exists("git") {
            Ok(())
        } else {
            Err("Git sigue sin estar disponible; la instalación fue cancelada o falló.".into())
        }
    }
}

fn status(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let repo = parse_repo(args)?;
    let output = git_output(&repo, &["status", "--short", "--branch"])?;
    print_output(&output);
    Ok(())
}

fn clone(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let (url, destination, branch, yes) = parse_clone(args)?;
    validate_url(&url)?;
    if let Some(path) = &destination {
        if path.exists()
            && path
                .read_dir()
                .map(|mut entries| entries.next().is_some())
                .unwrap_or(true)
        {
            return Err(format!("el destino no está vacío: {}", path.display()));
        }
    }
    let mut command_args = vec!["clone".to_string()];
    if let Some(branch) = &branch {
        command_args.extend(["--branch".into(), branch.clone()]);
    }
    command_args.push(url.clone());
    if let Some(destination) = &destination {
        command_args.push(destination.display().to_string());
    }
    println!(
        "$ git {}",
        command_args
            .iter()
            .map(|arg| common::shell_display(arg))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if ctx.dry_run {
        record(
            ctx,
            "git-clone",
            destination.as_deref().unwrap_or(Path::new(".")),
            "planned",
            &command_args,
        );
        return Ok(());
    }
    if !yes && !common::ask("¿Clonar este repositorio?") {
        record(
            ctx,
            "git-clone",
            destination.as_deref().unwrap_or(Path::new(".")),
            "cancelled",
            &command_args,
        );
        return Err("operación cancelada".into());
    }
    let success = Command::new("git")
        .args(&command_args)
        .status()
        .map_err(|e| e.to_string())?
        .success();
    record(
        ctx,
        "git-clone",
        destination.as_deref().unwrap_or(Path::new(".")),
        if success { "executed" } else { "failed" },
        &command_args,
    );
    if success {
        Ok(())
    } else {
        Err("git clone falló".into())
    }
}

fn fetch(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let (repo, remote, prune, yes) = parse_repo_action(args)?;
    let mut command_args = vec!["fetch".to_string()];
    if prune {
        command_args.push("--prune".into());
    }
    if let Some(remote) = &remote {
        command_args.push(remote.clone());
    }
    println!(
        "$ git -C {} {}",
        common::shell_display(&repo.display().to_string()),
        command_args
            .iter()
            .map(|arg| common::shell_display(arg))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if ctx.dry_run {
        record(ctx, "git-fetch", &repo, "planned", &command_args);
        return Ok(());
    }
    if !yes && !common::ask("¿Actualizar las referencias remotas con fetch?") {
        record(ctx, "git-fetch", &repo, "cancelled", &command_args);
        return Err("operación cancelada".into());
    }
    let success = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(&command_args)
        .status()
        .map_err(|e| e.to_string())?
        .success();
    record(
        ctx,
        "git-fetch",
        &repo,
        if success { "executed" } else { "failed" },
        &command_args,
    );
    if success {
        Ok(())
    } else {
        Err("git fetch falló".into())
    }
}

fn pull(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let (repo, remote, rebase, yes) = parse_pull(args)?;
    let status = git_output(&repo, &["status", "--porcelain"])?;
    if !status.trim().is_empty() && !has_flag(args, "--allow-dirty") {
        return Err("el repositorio tiene cambios sin confirmar; usa --allow-dirty solo si has revisado el riesgo".into());
    }
    let mut command_args = vec!["pull".to_string()];
    if rebase {
        command_args.push("--rebase".into());
    }
    if let Some(remote) = &remote {
        command_args.push(remote.clone());
    }
    println!(
        "$ git -C {} {}",
        common::shell_display(&repo.display().to_string()),
        command_args
            .iter()
            .map(|arg| common::shell_display(arg))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if ctx.dry_run {
        record(ctx, "git-pull", &repo, "planned", &command_args);
        return Ok(());
    }
    if !yes && !common::ask("¿Descargar e integrar cambios con pull?") {
        record(ctx, "git-pull", &repo, "cancelled", &command_args);
        return Err("operación cancelada".into());
    }
    let success = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(&command_args)
        .status()
        .map_err(|e| e.to_string())?
        .success();
    record(
        ctx,
        "git-pull",
        &repo,
        if success { "executed" } else { "failed" },
        &command_args,
    );
    if success {
        Ok(())
    } else {
        Err("git pull falló".into())
    }
}

fn log_history(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let (repo, limit) = parse_repo_limit(args)?;
    let limit = limit.unwrap_or(30).clamp(1, 500);
    let limit_arg = limit.to_string();
    let output = git_output(
        &repo,
        &[
            "log",
            "--oneline",
            "--decorate",
            "--graph",
            "-n",
            limit_arg.as_str(),
        ],
    )?;
    print_output(&output);
    Ok(())
}

fn add(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let (repo, paths, all, yes) = parse_add(args)?;
    let mut command_args = vec!["add".to_owned()];
    if all {
        command_args.push("--all".into());
    } else {
        command_args.extend(paths);
    }
    run_repo_mutation(
        ctx,
        "git-add",
        &repo,
        &command_args,
        yes,
        "¿Preparar los cambios seleccionados?",
    )
}

fn commit(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let (repo, message, all, yes) = parse_commit(args)?;
    let mut command_args = vec!["commit".to_owned(), "--message".into(), message];
    if all {
        command_args.push("--all".into());
    }
    run_repo_mutation(
        ctx,
        "git-commit",
        &repo,
        &command_args,
        yes,
        "¿Crear este commit?",
    )
}

fn push(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let options = parse_push(args)?;
    let mut command_args = vec!["push".to_owned()];
    if options.force_with_lease {
        command_args.push("--force-with-lease".into());
    }
    if options.tags {
        command_args.push("--tags".into());
    }
    command_args.push(options.remote.unwrap_or_else(|| "origin".into()));
    if let Some(branch) = options.branch {
        command_args.push(branch);
    }
    run_repo_mutation(
        ctx,
        "git-push",
        &options.repo,
        &command_args,
        options.yes,
        "¿Subir cambios al repositorio remoto?",
    )
}

fn branch(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let options = parse_branch(args)?;
    let command_args = match options.operation.as_deref() {
        None => vec!["branch".to_owned(), "--list".to_owned()],
        Some("create") => vec![
            "branch".to_owned(),
            options
                .name
                .clone()
                .ok_or("la rama a crear no está definida")?,
        ],
        Some("switch") => vec![
            "switch".to_owned(),
            options
                .name
                .clone()
                .ok_or("la rama a cambiar no está definida")?,
        ],
        Some("delete") => vec![
            "branch".to_owned(),
            "--delete".to_owned(),
            options
                .name
                .clone()
                .ok_or("la rama a borrar no está definida")?,
        ],
        _ => return Err("operación de rama no válida".into()),
    };
    if options.operation.is_none() {
        let output = git_output(&options.repo, &["branch", "--list"])?;
        print_output(&output);
        return Ok(());
    }
    run_repo_mutation(
        ctx,
        "git-branch",
        &options.repo,
        &command_args,
        options.yes,
        "¿Aplicar la operación sobre la rama?",
    )
}

fn tag(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let options = parse_tag(args)?;
    let Some(name) = options.name.clone() else {
        let output = git_output(&options.repo, &["tag", "--list"])?;
        print_output(&output);
        return Ok(());
    };
    let mut command_args = vec!["tag".to_owned(), name.clone()];
    if let Some(message) = options.message {
        command_args.extend(["--message".to_owned(), message]);
    }
    run_repo_mutation(
        ctx,
        "git-tag",
        &options.repo,
        &command_args,
        options.yes && !options.push,
        "¿Crear este tag?",
    )?;
    if options.push {
        let push_args = vec![
            "push".to_owned(),
            options.remote.unwrap_or_else(|| "origin".into()),
            name,
        ];
        run_repo_mutation(
            ctx,
            "git-tag-push",
            &options.repo,
            &push_args,
            options.yes,
            "¿Publicar este tag en el remoto?",
        )?;
    }
    Ok(())
}

fn release(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_gh(ctx)?;
    let options = parse_release(args)?;
    let mut command_args = vec!["release".to_owned(), "create".to_owned(), options.tag];
    if let Some(title) = options.title {
        command_args.extend(["--title".to_owned(), title]);
    }
    if let Some(notes) = options.notes {
        command_args.extend(["--notes".to_owned(), notes]);
    }
    if let Some(repo) = options.repo {
        command_args.extend(["--repo".to_owned(), repo]);
    }
    run_external_mutation(
        ctx,
        "gh-release-create",
        "gh",
        &command_args,
        options.yes,
        "¿Crear y publicar la release en GitHub?",
    )
}

fn diagnose(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let repo = parse_admin_repo(args, false)?.0;
    let root = match git_query(&repo, &["rev-parse", "--show-toplevel"]) {
        Ok(value) => value,
        Err(error) => {
            println!(
                "No se pudo abrir un repositorio Git en {}: {error}",
                repo.display()
            );
            println!("No se modificó nada. Si falta o está dañado .git, restaura una copia de seguridad o vuelve a clonar; `git init` no recupera historial ni objetos perdidos.");
            return Err("directorio Git no reconocible; no se aplicó reparación automática".into());
        }
    };
    let git_dir = git_query(&repo, &["rev-parse", "--git-dir"])?;
    let index = git_path(&repo, "index")?;
    let index_metadata = fs::symlink_metadata(&index);
    let index_kind = match &index_metadata {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            "enlace simbólico (no reparable automáticamente)"
        }
        Ok(metadata) if !metadata.is_file() => "objeto especial (no reparable automáticamente)",
        Ok(_) => "archivo",
        Err(error) if error.kind() == io::ErrorKind::NotFound => "no existe",
        Err(_) => "no se pudo inspeccionar",
    };
    let index_readable = index_metadata
        .as_ref()
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        && git_command(&repo, &["ls-files", "--stage"])
            .map(|output| output.status.success())
            .unwrap_or(false);
    let tree_ok = git_command(&repo, &["rev-parse", "--verify", "HEAD^{tree}"])
        .map(|output| output.status.success())
        .unwrap_or(false);
    let fsck = git_command(
        &repo,
        &["fsck", "--full", "--no-reflogs", "--no-progress", "HEAD"],
    )?;
    println!("=== Diagnóstico del repositorio Git ===");
    println!("Raíz: {root}");
    println!("Directorio Git: {git_dir}");
    println!(
        "Índice: {} ({index_kind}; {})",
        index.display(),
        if index_readable {
            "legible"
        } else {
            "no legible"
        }
    );
    println!(
        "Árbol HEAD: {}",
        if tree_ok {
            "disponible"
        } else {
            "no disponible; repositorio sin HEAD o historial dañado"
        }
    );
    println!(
        "Objetos alcanzables desde HEAD: {}",
        if fsck.status.success() {
            "fsck correcto"
        } else {
            "fsck detectó problemas"
        }
    );
    let fsck_output = format!(
        "{}{}",
        String::from_utf8_lossy(&fsck.stdout),
        String::from_utf8_lossy(&fsck.stderr)
    );
    if !fsck_output.trim().is_empty() {
        println!(
            "\nSalida de fsck:\n{}",
            fsck_output.chars().take(6_000).collect::<String>()
        );
    }
    let index_is_regular_or_missing = index_metadata
        .as_ref()
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        || index_metadata
            .as_ref()
            .is_err_and(|error| error.kind() == io::ErrorKind::NotFound);
    if !index_readable && index_is_regular_or_missing && tree_ok && fsck.status.success() {
        println!("\nSe puede reconstruir el índice desde HEAD con `ltools git repair --repo RUTA`. El proceso conserva una copia previa del índice y no modifica los archivos del árbol de trabajo.");
    } else if !fsck.status.success() || !tree_ok {
        println!("\nNo hay reparación automática segura para el historial/base de objetos. Conserva la carpeta y restaura desde una copia o clona de nuevo a otra ruta.");
    } else if !index_is_regular_or_missing {
        println!("\nEl índice es un enlace, un objeto especial o no se puede inspeccionar. LTools no lo reemplazará automáticamente; revisa permisos y restaura la ruta de forma manual.");
    } else {
        println!("\nNo se detecta un índice que necesite reconstrucción.");
    }
    Ok(())
}

fn repair(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git(ctx)?;
    let (repo, yes, remote, branch) = parse_repair_options(args)?;
    if git_query(&repo, &["rev-parse", "--show-toplevel"]).is_err() {
        if git_metadata_is_missing(&repo.join(".git"))? {
            if let Some(remote) = remote.as_deref() {
                return repair_missing_git_directory(ctx, &repo, remote, branch.as_deref(), yes);
            }
            return Err(format!(
                "no existe un directorio .git recuperable en {}; para reconstruirlo sin tocar los archivos de trabajo indica --remote URL y, opcionalmente, --branch RAMA. Sin URL/respaldo no es posible recuperar el historial automáticamente",
                repo.display()
            ));
        }
        return Err(format!(
            "no se reconoce .git en {}; no se ejecutó git init ni se alteró la carpeta. Restaura .git desde una copia o clona el remoto en otra ruta",
            repo.display()
        ));
    }
    if remote.is_some() {
        return Err("--remote/--branch solo se usan para recuperar una carpeta .git ausente; el repositorio ya es reconocible. Para reconstruir su índice omite esas opciones, y para objetos perdidos recupera desde un respaldo/remoto mediante una operación Git revisada".into());
    }
    let tree = git_query(&repo, &["rev-parse", "--verify", "HEAD^{tree}"])
        .map_err(|_| "no hay un árbol HEAD íntegro desde el que reconstruir el índice; no se modificó el repositorio".to_owned())?;
    let fsck = git_command(
        &repo,
        &["fsck", "--full", "--no-reflogs", "--no-progress", "HEAD"],
    )?;
    if !fsck.status.success() {
        return Err(format!(
            "git fsck detecta problemas en la base de objetos; no se intentó reparar. Conserva una copia de .git y recupera los objetos desde un remoto o respaldo. {}",
            String::from_utf8_lossy(&fsck.stderr).trim()
        ));
    }
    let index = git_path(&repo, "index")?;
    let index_ok = fs::symlink_metadata(&index)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        && git_command(&repo, &["ls-files", "--stage"])
            .map(|output| output.status.success())
            .unwrap_or(false);
    if index_ok {
        println!("El índice ya es legible; no hace falta repararlo. No se modificó nada.");
        return Ok(());
    }
    if let Ok(metadata) = fs::symlink_metadata(&index) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("el índice no es un archivo regular; se rechazó modificar una ruta especial o enlace".into());
        }
    }
    if ctx.dry_run {
        println!("Simulación: se conservaría el índice actual (si existe) en una copia de seguridad y se reconstruiría desde HEAD {tree}. Los cambios preparados podrían dejar de estar en el índice; los archivos del árbol de trabajo no se modifican.");
        return Ok(());
    }
    if !yes && !common::ask("¿Reconstruir el índice dañado desde HEAD? Se guardará una copia previa y los archivos de trabajo no cambiarán.") {
        return Err("reparación cancelada; no se modificó el repositorio".into());
    }

    let backup = if index.exists() {
        Some(copy_to_unique_backup(&index)?)
    } else {
        None
    };
    if index.exists() {
        fs::remove_file(&index).map_err(|error| {
            format!("se guardó una copia del índice en {}, pero no se pudo apartar el original: {error}", backup.as_ref().map(|path| path.display().to_string()).unwrap_or_default())
        })?;
    }
    let read_tree = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["read-tree", "HEAD"])
        .status();
    if !read_tree.as_ref().is_ok_and(|status| status.success()) {
        restore_index(&index, backup.as_deref())?;
        let detail = read_tree
            .err()
            .map(|error| format!("no se pudo iniciar git read-tree: {error}"))
            .unwrap_or_else(|| "git read-tree terminó con error".to_owned());
        return Err(restored_index_message(&detail, backup.as_deref()));
    }

    let verification = (|| {
        let status = git_command(&repo, &["ls-files", "--stage"])?;
        if !status.status.success() {
            return Err("git ls-files --stage no pudo leer el índice reconstruido".to_owned());
        }
        let fsck_after = git_command(
            &repo,
            &["fsck", "--full", "--no-reflogs", "--no-progress", "HEAD"],
        )?;
        if !fsck_after.status.success() {
            return Err(format!(
                "git fsck posterior detectó problemas: {}",
                String::from_utf8_lossy(&fsck_after.stderr).trim()
            ));
        }
        Ok::<(), String>(())
    })();
    if let Err(detail) = verification {
        restore_index(&index, backup.as_deref())?;
        return Err(restored_index_message(&detail, backup.as_deref()));
    }
    if let Some(backup) = &backup {
        println!(
            "Copia de seguridad del índice anterior: {}",
            backup.display()
        );
    }
    println!(
        "Índice reconstruido desde HEAD y verificado. Los archivos de trabajo no se modificaron."
    );
    record(
        ctx,
        "git-index-repair",
        &repo,
        "executed",
        &["read-tree".into(), "HEAD".into()],
    );
    Ok(())
}

fn parse_repair_options(
    args: &[String],
) -> Result<(PathBuf, bool, Option<String>, Option<String>), String> {
    let mut repo = PathBuf::from(".");
    let mut remote = None;
    let mut branch = None;
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                let value = args.get(index + 1).ok_or("--repo requiere una ruta")?;
                if value.starts_with('-') {
                    return Err("--repo requiere una ruta válida".into());
                }
                repo = PathBuf::from(value);
                index += 2;
            }
            "--remote" => {
                let value = args.get(index + 1).ok_or("--remote requiere una URL Git")?;
                validate_url(value)?;
                remote = Some(value.clone());
                index += 2;
            }
            "--branch" => {
                let value = args.get(index + 1).ok_or("--branch requiere una rama")?;
                validate_ref(value, "rama")?;
                branch = Some(value.clone());
                index += 2;
            }
            "--yes" => {
                yes = true;
                index += 1;
            }
            value => return Err(format!("opción Git repair desconocida: {value}")),
        }
    }
    if !repo.is_dir() {
        return Err(format!("no es una carpeta: {}", repo.display()));
    }
    if branch.is_some() && remote.is_none() {
        return Err("--branch solo puede usarse junto con --remote".into());
    }
    Ok((repo, yes, remote, branch))
}

/// Recupera únicamente metadatos ausentes desde un remoto explícito. El
/// clonado se hace sin checkout a un directorio temporal vecino; después se
/// instala solo `.git`, reconstruye el índice desde HEAD y nunca sobrescribe
/// ni elimina archivos del árbol de trabajo.
fn repair_missing_git_directory(
    ctx: &Context,
    repo: &Path,
    remote: &str,
    branch: Option<&str>,
    yes: bool,
) -> Result<(), String> {
    validate_url(remote)?;
    let canonical_repo = fs::canonicalize(repo)
        .map_err(|error| format!("no se pudo resolver la ruta del repositorio: {error}"))?;
    let git_dir = canonical_repo.join(".git");
    if !git_metadata_is_missing(&git_dir)? {
        return Err(".git existe como archivo, enlace u objeto dañado; se rechazó reemplazarlo. Conserva una copia y diagnostica la estructura manualmente".into());
    }
    let Some(parent) = canonical_repo.parent() else {
        return Err(
            "el repositorio no tiene una carpeta padre segura para preparar la recuperación".into(),
        );
    };
    if ctx.dry_run {
        println!("Simulación: se clonaría el remoto sin checkout a una carpeta temporal y se instalaría solo su metadata .git en {}. Los archivos locales no se borrarían ni sobrescribirían; después debes revisar su diferencia con HEAD.", canonical_repo.display());
        record(
            ctx,
            "git-directory-recover",
            &canonical_repo,
            "planned",
            &["clone-no-checkout".into()],
        );
        return Ok(());
    }
    if !yes
        && !common::ask("No se encontró .git. Se recuperarán el historial y la referencia desde el remoto indicado, sin checkout ni sobrescritura del árbol de trabajo; los cambios locales aparecerán al comparar con HEAD. ¿Continuar?")
    {
        return Err("recuperación de .git cancelada; no se modificó nada".into());
    }

    // Reserva atómicamente un directorio propio antes de que `git clone`
    // escriba nada. No se usa el patrón inseguro exists()-y-luego-crear ni
    // se elimina una ruta temporal que otro proceso pudiera haber creado.
    let mut staging = None;
    for attempt in 0..16_u8 {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let candidate = parent.join(format!(
            ".ltools-git-recovery-{}-{nonce}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => {
                #[cfg(unix)]
                if let Err(error) =
                    fs::set_permissions(&candidate, fs::Permissions::from_mode(0o700))
                {
                    let _ = fs::remove_dir(&candidate);
                    return Err(format!("no se pudieron restringir los permisos del temporal de recuperación: {error}"));
                }
                staging = Some(candidate);
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "no se pudo reservar una carpeta temporal de recuperación: {error}"
                ));
            }
        }
    }
    let staging = staging.ok_or("no se pudo reservar una carpeta temporal de recuperación")?;
    let clone_dir = staging.join("clone");
    let mut clone = Command::new("git");
    clone.args(["clone", "--no-checkout", "--origin", "origin"]);
    if let Some(branch) = branch {
        clone.args(["--branch", branch]);
    }
    let clone_result = clone.arg(remote).arg(&clone_dir).output();
    let clone_output = match clone_result {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(format!(
                "el clonado de recuperación falló con {}; no se modificó el repositorio. La salida se omitió para no exponer datos del remoto",
                output.status
            ));
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(format!("no se pudo iniciar git clone: {error}"));
        }
    };
    drop(clone_output);

    let tree = git_query(&clone_dir, &["rev-parse", "--verify", "HEAD^{tree}"]);
    let fsck = git_command(
        &clone_dir,
        &["fsck", "--full", "--no-reflogs", "--no-progress", "HEAD"],
    );
    let integrity_ok = tree.is_ok() && fsck.as_ref().is_ok_and(|output| output.status.success());
    if !integrity_ok {
        let _ = fs::remove_dir_all(&staging);
        return Err("el remoto no contiene una rama HEAD verificable o la comprobación de objetos falló; no se instaló .git ni se tocaron los archivos locales".into());
    }
    match git_metadata_is_missing(&git_dir) {
        Ok(true) => {}
        Ok(false) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(
                "apareció una ruta .git durante la recuperación; se canceló sin reemplazarla"
                    .into(),
            );
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    }
    let staged_git = clone_dir.join(".git");
    if let Err(error) = fs::rename(&staged_git, &git_dir) {
        let _ = fs::remove_dir_all(&staging);
        return Err(format!("no se pudo instalar la metadata recuperada: {error}; el árbol de trabajo permanece intacto"));
    }
    let index = git_command(&canonical_repo, &["read-tree", "HEAD"]);
    let index_ok = index.is_ok_and(|output| output.status.success())
        && git_command(&canonical_repo, &["ls-files", "--stage"])
            .is_ok_and(|output| output.status.success());
    if !index_ok {
        let _ = fs::remove_dir_all(&staging);
        record(
            ctx,
            "git-directory-recover",
            &canonical_repo,
            "partial",
            &["metadata-installed-index-needs-review".into()],
        );
        return Err("la metadata de .git se recuperó y quedó instalada, pero no se pudo reconstruir/verificar el índice. No se modificaron archivos del árbol; conserva la carpeta y ejecuta diagnose antes de continuar".into());
    }
    let status = git_command(&canonical_repo, &["status", "--short", "--branch"])
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "estado disponible mediante git status".into());
    let cleanup_ok = fs::remove_dir_all(&staging).is_ok();
    println!(".git se recuperó desde el remoto; el árbol de trabajo no se sobrescribió. Revisa los cambios locales comparados con la rama recuperada:\n{status}");
    if !cleanup_ok {
        println!("Aviso: la carpeta temporal de recuperación quedó en {} y debe retirarse manualmente tras revisarla.", staging.display());
    }
    record(
        ctx,
        "git-directory-recover",
        &canonical_repo,
        "executed",
        &["clone-no-checkout".into()],
    );
    Ok(())
}

fn restored_index_message(detail: &str, backup: Option<&Path>) -> String {
    match backup {
        Some(path) => format!(
            "{detail}; se restauró el índice original desde {}",
            path.display()
        ),
        None => format!("{detail}; el índice parcial se retiró"),
    }
}

fn restore_index(index: &Path, backup: Option<&Path>) -> Result<(), String> {
    match fs::symlink_metadata(index) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(format!("no se reemplazó el índice porque la ruta se convirtió en un enlace u objeto especial: {}", index.display()));
        }
        Ok(_) => fs::remove_file(index).map_err(|error| {
            format!("no se pudo retirar el índice parcial {}; conserva el directorio Git y revisa manualmente: {error}", index.display())
        })?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("no se pudo inspeccionar el índice durante la restauración: {error}")),
    }
    if let Some(backup) = backup {
        let parent = index
            .parent()
            .ok_or("el índice no tiene directorio padre")?;
        let mut restore_temp = None;
        for attempt in 0..16_u8 {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let candidate = parent.join(format!(
                ".index.ltools-restore-{}-{nonce}-{attempt}.tmp",
                std::process::id()
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate)
            {
                Ok(file) => {
                    restore_temp = Some((candidate, file));
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "no se pudo preparar restauración del índice; la copia original sigue en {}: {error}",
                        backup.display()
                    ));
                }
            }
        }
        let (temp_path, mut temp_file) = restore_temp.ok_or_else(|| {
            format!(
                "no se pudo reservar un archivo temporal; la copia original sigue en {}",
                backup.display()
            )
        })?;
        let restore_result = (|| {
            let mut source = File::open(backup)?;
            io::copy(&mut source, &mut temp_file)?;
            temp_file.sync_all()?;
            let metadata = fs::metadata(backup)?;
            fs::set_permissions(&temp_path, metadata.permissions())?;
            fs::rename(&temp_path, index)
        })();
        if let Err(error) = restore_result {
            let _ = fs::remove_file(&temp_path);
            return Err(format!(
                "no se pudo restaurar el índice; la copia original sigue en {}: {error}",
                backup.display()
            ));
        }
    }
    Ok(())
}

fn parse_admin_repo(args: &[String], allow_yes: bool) -> Result<(PathBuf, bool), String> {
    let mut repo = PathBuf::from(".");
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                let value = args.get(index + 1).ok_or("--repo requiere una ruta")?;
                if value.starts_with('-') {
                    return Err("--repo requiere una ruta válida".into());
                }
                repo = PathBuf::from(value);
                index += 2;
            }
            "--yes" if allow_yes => {
                yes = true;
                index += 1;
            }
            value => return Err(format!("opción Git diagnose/repair desconocida: {value}")),
        }
    }
    if !repo.is_dir() {
        return Err(format!("no es una carpeta: {}", repo.display()));
    }
    Ok((repo, yes))
}

fn git_command(repo: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| format!("no se pudo ejecutar git {}: {error}", args.join(" ")))
}

fn git_metadata_is_missing(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!(
            "no se pudo inspeccionar {}: {error}",
            path.display()
        )),
    }
}

fn git_query(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = git_command(repo, args)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

fn git_path(repo: &Path, name: &str) -> Result<PathBuf, String> {
    let value = git_query(repo, &["rev-parse", "--git-path", name])?;
    let path = PathBuf::from(value);
    let path = if path.is_absolute() {
        path
    } else {
        repo.join(path)
    };
    if let Some(parent) = path.parent() {
        fs::canonicalize(parent)
            .map(|parent| parent.join(path.file_name().unwrap_or_default()))
            .map_err(|error| format!("no se pudo resolver la ruta del índice: {error}"))
    } else {
        Err("Git devolvió una ruta de índice sin directorio padre".into())
    }
}

fn copy_to_unique_backup(index: &Path) -> Result<PathBuf, String> {
    let parent = index
        .parent()
        .ok_or("el índice no tiene directorio padre")?;
    let metadata = fs::symlink_metadata(index).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("el índice no es un archivo regular; no se creó copia ni se modificó".into());
    }
    for attempt in 0..16_u8 {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let backup = parent.join(format!(
            "index.ltools-corrupt-{}-{nonce}-{attempt}.bak",
            std::process::id()
        ));
        let mut destination = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "no se pudo crear la copia de seguridad del índice: {error}"
                ))
            }
        };
        let copy_result = (|| {
            let mut source = File::open(index)?;
            io::copy(&mut source, &mut destination)?;
            destination.sync_all()?;
            fs::set_permissions(&backup, metadata.permissions())?;
            Ok::<(), io::Error>(())
        })();
        if let Err(error) = copy_result {
            let _ = fs::remove_file(&backup);
            return Err(format!(
                "no se completó la copia de seguridad del índice: {error}"
            ));
        }
        return Ok(backup);
    }
    Err("no se pudo reservar un nombre único para la copia del índice".into())
}

fn gh(ctx: &Context, args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|operation| operation == "native") {
        return gh_native(ctx, args.get(1..).unwrap_or_default());
    }
    require_gh(ctx)?;
    let (operation, repo, yes) = parse_gh(args)?;
    let mut command_args = match operation.as_str() {
        "version" => vec!["--version".to_owned()],
        "help" => vec!["help".to_owned()],
        "auth-status" => vec!["auth".to_owned(), "status".to_owned()],
        "login" => vec!["auth".to_owned(), "login".to_owned()],
        "repo" => vec!["repo".to_owned(), "view".to_owned()],
        "prs" => vec!["pr".to_owned(), "list".to_owned()],
        "releases" => vec!["release".to_owned(), "list".to_owned()],
        _ => return Err("operación gh no permitida por LTools".into()),
    };
    if let Some(repo) = repo {
        command_args.extend(["--repo".to_owned(), repo]);
    }
    run_external_mutation(
        ctx,
        "gh-operation",
        "gh",
        &command_args,
        yes || operation == "auth-status"
            || operation == "repo"
            || operation == "prs"
            || operation == "releases"
            || operation == "version"
            || operation == "help",
        "¿Ejecutar esta operación de GitHub?",
    )
}

/// Acceso al CLI nativo de `gh` para subcomandos/extensiones que cambian entre
/// versiones. Ejecuta argumentos separados, no una cadena de shell. La salida
/// y los argumentos completos no se guardan en el plan porque pueden contener
/// información privada; los subcomandos mutadores desconocidos siempre piden
/// confirmación.
fn gh_native(ctx: &Context, raw_args: &[String]) -> Result<(), String> {
    require_gh(ctx)?;
    let (args, confirmed) = parse_gh_native_args(raw_args);
    if args.len() > 64
        || args.iter().any(|value| {
            value.is_empty() || value.len() > 4_096 || value.chars().any(char::is_control)
        })
    {
        return Err(
            "el comando gh nativo supera los límites de argumentos o contiene controles".into(),
        );
    }
    if args.first().is_some_and(|value| value == "auth")
        && args
            .iter()
            .skip(1)
            .take_while(|value| value.as_str() != "--")
            .any(|value| value == "token")
    {
        return Err("ltools bloquea `gh auth token` para evitar mostrar o registrar credenciales; usa el almacén seguro de gh en su lugar".into());
    }
    if args.iter().any(|value| value == "--insecure-storage") {
        return Err(
            "ltools bloquea --insecure-storage porque guardaría credenciales de gh en texto plano"
                .into(),
        );
    }
    let read_only = gh_native_is_read_only(&args);
    println!(
        "$ gh {}{}",
        args.first().map(String::as_str).unwrap_or("help"),
        if args.len() > 1 {
            " … (argumentos omitidos del registro)"
        } else {
            ""
        }
    );
    if ctx.dry_run {
        record(
            ctx,
            "gh-native",
            Path::new("gh"),
            "planned",
            &[args[0].clone()],
        );
        println!("Simulación: no se ejecutó GitHub CLI. Consulta `ltools guide gh` para distinguir consultas y cambios.");
        return Ok(());
    }
    if !read_only && !confirmed && !common::ask("Este subcomando gh no está clasificado como consulta; ¿ejecutarlo con sus efectos nativos?") {
        record(ctx, "gh-native", Path::new("gh"), "cancelled", &[args[0].clone()]);
        return Err("operación gh cancelada".into());
    }
    let status = Command::new("gh")
        .args(&args)
        .status()
        .map_err(|error| format!("no se pudo iniciar gh: {error}"))?;
    record(
        ctx,
        "gh-native",
        Path::new("gh"),
        if status.success() {
            "executed"
        } else {
            "failed"
        },
        &[args[0].clone()],
    );
    if status.success() {
        Ok(())
    } else {
        Err(format!("gh terminó con {status}"))
    }
}

/// Separa la confirmación interna de LTools sin apropiarse de flags nativos
/// que gh pueda añadir en versiones futuras (por ejemplo, su propio `--yes`).
fn parse_gh_native_args(raw_args: &[String]) -> (Vec<String>, bool) {
    let confirmed = raw_args.iter().any(|value| value == "--ltools-confirmed");
    let mut args = raw_args
        .iter()
        .filter(|value| value.as_str() != "--ltools-confirmed")
        .cloned()
        .collect::<Vec<_>>();
    if args.is_empty() {
        args.push("help".into());
    }
    (args, confirmed)
}

fn gh_native_is_read_only(args: &[String]) -> bool {
    let Some(root) = args.first().map(String::as_str) else {
        return true;
    };
    let subcommand = args.get(1).map(String::as_str).unwrap_or_default();
    matches!(
        root,
        "help" | "--help" | "--version" | "version" | "status" | "search"
    ) || matches!(
        (root, subcommand),
        ("auth", "status")
            | ("repo", "view" | "list" | "forks")
            | ("issue", "list" | "view" | "status")
            | ("pr", "list" | "view" | "status" | "diff" | "checks")
            | ("release", "list" | "view" | "verify")
            | ("workflow", "list" | "view")
            | ("run", "list" | "view" | "watch")
    )
}

/// Convierte el campo de argumentos avanzado de la GUI en argv sin invocar
/// ningún shell. Admite espacios y comillas simples/dobles; no expande $, ``,
/// comodines, tuberías ni sustituciones.
#[cfg(any(unix, test))]
pub(crate) fn split_native_argument_line(input: &str) -> Result<Vec<String>, String> {
    if input.len() > 32_768 || input.chars().any(char::is_control) {
        return Err(
            "los argumentos de gh superan el límite permitido o contienen controles".into(),
        );
    }
    let mut result = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut started = false;
    for character in input.chars() {
        if escaped {
            token.push(character);
            escaped = false;
            started = true;
            continue;
        }
        match quote {
            Some('\'') if character != '\'' => token.push(character),
            Some('"') if character == '\\' => escaped = true,
            Some(active) if character == active => quote = None,
            Some(_) => token.push(character),
            None if character == '\\' => {
                escaped = true;
                started = true;
            }
            None if character == '\'' || character == '"' => {
                quote = Some(character);
                started = true;
            }
            None if character.is_whitespace() => {
                if started {
                    result.push(std::mem::take(&mut token));
                    started = false;
                }
            }
            None => {
                token.push(character);
                started = true;
            }
        }
        if token.len() > 4_096 {
            return Err("un argumento de gh supera el máximo de 4096 bytes".into());
        }
    }
    if escaped {
        return Err("los argumentos de gh terminan con una barra invertida sin escapar".into());
    }
    if quote.is_some() {
        return Err("las comillas de los argumentos de gh no están cerradas".into());
    }
    if started {
        result.push(token);
    }
    if result.len() > 63 {
        return Err("la GUI admite como máximo 63 argumentos nativos de gh".into());
    }
    Ok(result)
}

/// Construye argv para el formulario GUI de `gh` y evita duplicar el ámbito
/// del repositorio cuando se proporciona tanto en el campo común como en la
/// línea avanzada de argumentos.
#[cfg(any(unix, test))]
pub(crate) fn gh_gui_arguments(
    command: &str,
    repository: &str,
    argument_line: &str,
) -> Result<Vec<String>, String> {
    let command = command.trim();
    if command.is_empty()
        || command.len() > 4_096
        || command.chars().any(char::is_control)
        || command.split_whitespace().count() != 1
    {
        return Err("el comando gh debe ser una sola palabra sin caracteres de control".into());
    }
    let native_args = split_native_argument_line(argument_line)?;
    let repository = repository.trim();
    if !repository.is_empty()
        && matches!(
            command,
            "auth" | "api" | "help" | "version" | "--help" | "--version"
        )
    {
        return Err(format!(
            "el campo Repositorio no se aplica a `gh {command}`; indica el ámbito en los argumentos o deja ese campo vacío"
        ));
    }
    if !repository.is_empty()
        && native_args
            .iter()
            .take_while(|argument| argument.as_str() != "--")
            .any(|argument| {
                argument == "--repo"
                    || argument.starts_with("--repo=")
                    || argument == "-R"
                    || argument.starts_with("-R=")
                    || (argument.starts_with("-R") && argument.len() > 2)
            })
    {
        return Err(
            "indica el repositorio en el campo Repositorio o en Argumentos, no en ambos".into(),
        );
    }
    if !repository.is_empty() {
        validate_gh_repository(repository)?;
    }
    let mut args = vec!["gh".to_owned(), "native".to_owned(), command.to_owned()];
    args.extend(native_args);
    if !repository.is_empty() {
        let end_of_options = args
            .iter()
            .position(|argument| argument == "--")
            .unwrap_or(args.len());
        args.splice(
            end_of_options..end_of_options,
            ["--repo".to_owned(), repository.to_owned()],
        );
    }
    Ok(args)
}

fn run_repo_mutation(
    ctx: &Context,
    operation: &str,
    repo: &Path,
    args: &[String],
    yes: bool,
    question: &str,
) -> Result<(), String> {
    run_command_mutation(ctx, operation, "git", Some(repo), args, yes, question)
}

fn run_external_mutation(
    ctx: &Context,
    operation: &str,
    program: &str,
    args: &[String],
    yes: bool,
    question: &str,
) -> Result<(), String> {
    run_command_mutation(ctx, operation, program, None, args, yes, question)
}

fn run_command_mutation(
    ctx: &Context,
    operation: &str,
    program: &str,
    repo: Option<&Path>,
    args: &[String],
    yes: bool,
    question: &str,
) -> Result<(), String> {
    let displayed = args
        .iter()
        .map(|arg| common::shell_display(arg))
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(repo) = repo {
        println!(
            "$ {program} -C {} {displayed}",
            common::shell_display(&repo.display().to_string())
        );
    } else {
        println!("$ {program} {displayed}");
    }
    let target = repo.unwrap_or_else(|| Path::new(program));
    if ctx.dry_run {
        record(ctx, operation, target, "planned", args);
        return Ok(());
    }
    if !yes && !common::ask(question) {
        record(ctx, operation, target, "cancelled", args);
        return Err("operación cancelada".into());
    }
    let mut command = Command::new(program);
    if let Some(repo) = repo {
        command.arg("-C").arg(repo);
    }
    let output = command
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&stderr);
    }
    print_output(&text);
    record(
        ctx,
        operation,
        target,
        if output.status.success() {
            "executed"
        } else {
            "failed"
        },
        args,
    );
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("{program} terminó con {}", output.status))
    }
}

fn require_gh(ctx: &Context) -> Result<(), String> {
    if common::command_exists("gh") {
        Ok(())
    } else if ctx.dry_run {
        println!("Simulación: GitHub CLI se prepararía antes de ejecutar esta acción.");
        Ok(())
    } else {
        println!("GitHub CLI (gh) no está instalado; LTools puede ofrecer su instalación desde el gestor disponible.");
        if crate::common::ensure_tool(ctx, "gh")? && common::command_exists("gh") {
            Ok(())
        } else {
            Err(
                "GitHub CLI sigue sin estar disponible; la instalación fue cancelada o falló."
                    .into(),
            )
        }
    }
}

fn login(ctx: &Context) -> Result<(), String> {
    require_git(ctx)?;
    println!("Git no almacena una sesión universal. LTools no solicita ni guarda contraseñas, tokens ni claves privadas.");
    let name = command_value(&["config", "--global", "--get", "user.name"])
        .unwrap_or_else(|| "no definido".into());
    let email = command_value(&["config", "--global", "--get", "user.email"])
        .unwrap_or_else(|| "no definido".into());
    println!("Identidad Git: {name} <{email}>");
    if common::command_exists("gh") {
        println!(
            "GitHub CLI detectado; puede gestionar la autenticación de GitHub de forma segura."
        );
        if !ctx.dry_run && common::ask("¿Abrir el inicio de sesión de GitHub CLI?") {
            let success = Command::new("gh")
                .args(["auth", "login"])
                .status()
                .map_err(|e| e.to_string())?
                .success();
            if !success {
                return Err("gh auth login falló".into());
            }
        } else if ctx.dry_run {
            println!("Simulación: se abriría gh auth login.");
        }
    } else {
        println!("No se encontró gh. Configura SSH o un gestor de credenciales del sistema; no se instalará desde LTools.");
    }
    if let Some(plan) = &ctx.plan {
        let _ = plan.record(
            "git-login",
            Path::new("git"),
            if ctx.dry_run { "planned" } else { "inspected" },
            false,
            "gh auth login",
            "credentials-never-stored",
        );
    }
    Ok(())
}

fn parse_repo_limit(args: &[String]) -> Result<(PathBuf, Option<usize>), String> {
    let mut repo = PathBuf::from(".");
    let mut limit = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                repo = PathBuf::from(args.get(index + 1).ok_or("--repo requiere una ruta")?);
                index += 2;
            }
            "--limit" => {
                let value = args.get(index + 1).ok_or("--limit requiere un número")?;
                limit = Some(value.parse::<usize>().map_err(|_| "--limit no es válido")?);
                index += 2;
            }
            value => return Err(format!("opción git desconocida: {value}")),
        }
    }
    verify_repo(&repo)?;
    Ok((repo, limit))
}

fn parse_add(args: &[String]) -> Result<(PathBuf, Vec<String>, bool, bool), String> {
    let mut repo = PathBuf::from(".");
    let mut paths = Vec::new();
    let mut all = false;
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                repo = PathBuf::from(args.get(index + 1).ok_or("--repo requiere una ruta")?);
                index += 2;
            }
            "--all" => {
                all = true;
                index += 1;
            }
            "--yes" => {
                yes = true;
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(format!("opción git desconocida: {value}"))
            }
            value => {
                validate_path_argument(value)?;
                paths.push(value.to_owned());
                index += 1;
            }
        }
    }
    if !all && paths.is_empty() {
        return Err("git add requiere rutas o --all".into());
    }
    verify_repo(&repo)?;
    Ok((repo, paths, all, yes))
}

fn parse_commit(args: &[String]) -> Result<(PathBuf, String, bool, bool), String> {
    let mut repo = PathBuf::from(".");
    let mut message = None;
    let mut all = false;
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                repo = PathBuf::from(args.get(index + 1).ok_or("--repo requiere una ruta")?);
                index += 2;
            }
            "--message" | "-m" => {
                message = Some(
                    args.get(index + 1)
                        .ok_or("--message requiere texto")?
                        .clone(),
                );
                index += 2;
            }
            "--all" => {
                all = true;
                index += 1;
            }
            "--yes" => {
                yes = true;
                index += 1;
            }
            value => return Err(format!("opción git desconocida: {value}")),
        }
    }
    let message = message.ok_or("git commit requiere --message")?;
    validate_message(&message)?;
    verify_repo(&repo)?;
    Ok((repo, message, all, yes))
}

fn parse_push(args: &[String]) -> Result<PushOptions, String> {
    let mut repo = PathBuf::from(".");
    let mut remote = None;
    let mut branch = None;
    let mut tags = false;
    let mut force_with_lease = false;
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                repo = PathBuf::from(args.get(index + 1).ok_or("--repo requiere una ruta")?);
                index += 2;
            }
            "--remote" => {
                let value = args.get(index + 1).ok_or("--remote requiere un nombre")?;
                validate_ref(value, "remoto")?;
                remote = Some(value.clone());
                index += 2;
            }
            "--branch" => {
                let value = args.get(index + 1).ok_or("--branch requiere un nombre")?;
                validate_ref(value, "rama")?;
                branch = Some(value.clone());
                index += 2;
            }
            "--tags" => {
                tags = true;
                index += 1;
            }
            "--force-with-lease" => {
                force_with_lease = true;
                index += 1;
            }
            "--yes" => {
                yes = true;
                index += 1;
            }
            value => return Err(format!("opción git desconocida: {value}")),
        }
    }
    verify_repo(&repo)?;
    Ok(PushOptions {
        repo,
        remote,
        branch,
        tags,
        force_with_lease,
        yes,
    })
}

fn parse_branch(args: &[String]) -> Result<BranchOptions, String> {
    let mut repo = PathBuf::from(".");
    let mut operation = None;
    let mut name = None;
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                repo = PathBuf::from(args.get(index + 1).ok_or("--repo requiere una ruta")?);
                index += 2;
            }
            "--create" | "--switch" | "--delete" => {
                let option = args[index].trim_start_matches("--").to_owned();
                let value = args
                    .get(index + 1)
                    .ok_or("la operación de rama requiere un nombre")?;
                validate_ref(value, "rama")?;
                operation = Some(option);
                name = Some(value.clone());
                index += 2;
            }
            "--list" => index += 1,
            "--yes" => {
                yes = true;
                index += 1;
            }
            value => return Err(format!("opción git desconocida: {value}")),
        }
    }
    verify_repo(&repo)?;
    Ok(BranchOptions {
        repo,
        operation,
        name,
        yes,
    })
}

fn parse_tag(args: &[String]) -> Result<TagOptions, String> {
    let mut repo = PathBuf::from(".");
    let mut name = None;
    let mut message = None;
    let mut push = false;
    let mut remote = None;
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                repo = PathBuf::from(args.get(index + 1).ok_or("--repo requiere una ruta")?);
                index += 2;
            }
            "--name" => {
                let value = args.get(index + 1).ok_or("--name requiere un tag")?;
                validate_ref(value, "tag")?;
                name = Some(value.clone());
                index += 2;
            }
            "--message" => {
                let value = args.get(index + 1).ok_or("--message requiere texto")?;
                validate_message(value)?;
                message = Some(value.clone());
                index += 2;
            }
            "--push" => {
                push = true;
                index += 1;
            }
            "--remote" => {
                let value = args.get(index + 1).ok_or("--remote requiere un nombre")?;
                validate_ref(value, "remoto")?;
                remote = Some(value.clone());
                index += 2;
            }
            "--list" => index += 1,
            "--yes" => {
                yes = true;
                index += 1;
            }
            value => return Err(format!("opción git desconocida: {value}")),
        }
    }
    verify_repo(&repo)?;
    Ok(TagOptions {
        repo,
        name,
        message,
        push,
        remote,
        yes,
    })
}

fn parse_release(args: &[String]) -> Result<ReleaseOptions, String> {
    let mut repo = None;
    let mut tag = None;
    let mut title = None;
    let mut notes = None;
    let mut yes = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                let value = args.get(index + 1).ok_or("--repo requiere owner/repo")?;
                validate_gh_repository(value)?;
                repo = Some(value.clone());
                index += 2;
            }
            "--tag" => {
                let value = args.get(index + 1).ok_or("--tag requiere un nombre")?;
                validate_ref(value, "tag")?;
                tag = Some(value.clone());
                index += 2;
            }
            "--title" => {
                let value = args.get(index + 1).ok_or("--title requiere texto")?;
                validate_message(value)?;
                title = Some(value.clone());
                index += 2;
            }
            "--notes" => {
                let value = args.get(index + 1).ok_or("--notes requiere texto")?;
                validate_message(value)?;
                notes = Some(value.clone());
                index += 2;
            }
            "--yes" => {
                yes = true;
                index += 1;
            }
            value => return Err(format!("opción release desconocida: {value}")),
        }
    }
    Ok(ReleaseOptions {
        repo,
        tag: tag.ok_or("release requiere --tag")?,
        title,
        notes,
        yes,
    })
}

fn parse_gh(args: &[String]) -> Result<(String, Option<String>, bool), String> {
    let operation = args.first().ok_or("git gh requiere una operación")?.clone();
    if !matches!(
        operation.as_str(),
        "auth-status" | "login" | "repo" | "prs" | "releases" | "version" | "help"
    ) {
        return Err("operación gh no permitida por LTools".into());
    }
    let mut repo = None;
    let mut yes = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                let value = args.get(index + 1).ok_or("--repo requiere owner/repo")?;
                validate_ref(value, "repositorio GitHub")?;
                repo = Some(value.clone());
                index += 2;
            }
            "--yes" => {
                yes = true;
                index += 1;
            }
            value => return Err(format!("opción gh desconocida: {value}")),
        }
    }
    if repo.is_some() && !matches!(operation.as_str(), "repo" | "prs" | "releases") {
        return Err(format!(
            "--repo no se aplica a `gh {operation}`; úsalo solo con repo, prs o releases"
        ));
    }
    Ok((operation, repo, yes))
}

fn validate_path_argument(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 400
        || value.starts_with('-')
        || value.chars().any(char::is_control)
    {
        Err("ruta Git no válida".into())
    } else {
        Ok(())
    }
}

fn validate_message(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 5_000 || value.chars().any(char::is_control) {
        Err("mensaje Git vacío, demasiado largo o con caracteres de control".into())
    } else {
        Ok(())
    }
}

fn validate_ref(value: &str, kind: &str) -> Result<(), String> {
    if safe_word(value) {
        Ok(())
    } else {
        Err(format!("{kind} no válido"))
    }
}

fn validate_gh_repository(value: &str) -> Result<(), String> {
    let components = value.split('/').collect::<Vec<_>>();
    let valid_component = |component: &str| {
        !component.is_empty()
            && component != "."
            && component != ".."
            && component
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    };
    if !matches!(components.len(), 2 | 3) || !components.iter().all(|part| valid_component(part)) {
        return Err("repositorio gh no válido; usa OWNER/REPO o HOST/OWNER/REPO".into());
    }
    Ok(())
}

fn parse_repo(args: &[String]) -> Result<PathBuf, String> {
    let mut repo = PathBuf::from(".");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                repo = PathBuf::from(args.get(index + 1).ok_or("--repo requiere una ruta")?);
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("opción git desconocida: {value}"))
            }
            _ => return Err(format!("argumento inesperado: {}", args[index])),
        }
    }
    verify_repo(&repo)?;
    Ok(repo)
}

fn parse_repo_action(args: &[String]) -> Result<(PathBuf, Option<String>, bool, bool), String> {
    let repo = parse_repo_allow_flags(args)?;
    let remote = option_value(args, "--remote");
    let prune = has_flag(args, "--prune");
    let yes = has_flag(args, "--yes");
    if remote.as_ref().is_some_and(|value| !safe_word(value)) {
        return Err("nombre de remoto no válido".into());
    }
    verify_repo(&repo)?;
    Ok((repo, remote, prune, yes))
}

fn parse_pull(args: &[String]) -> Result<(PathBuf, Option<String>, bool, bool), String> {
    let repo = parse_repo_allow_flags(args)?;
    let remote = option_value(args, "--remote");
    if remote.as_ref().is_some_and(|value| !safe_word(value)) {
        return Err("nombre de remoto no válido".into());
    }
    verify_repo(&repo)?;
    Ok((
        repo,
        remote,
        has_flag(args, "--rebase"),
        has_flag(args, "--yes"),
    ))
}

fn parse_repo_allow_flags(args: &[String]) -> Result<PathBuf, String> {
    let mut repo = PathBuf::from(".");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                let value = args.get(index + 1).ok_or("--repo requiere una ruta")?;
                if value.starts_with('-') {
                    return Err("--repo requiere una ruta válida".into());
                }
                repo = PathBuf::from(value);
                index += 2;
            }
            "--remote" => {
                let value = args.get(index + 1).ok_or("--remote requiere un nombre")?;
                if !safe_word(value) {
                    return Err("nombre de remoto no válido".into());
                }
                index += 2;
            }
            "--prune" | "--rebase" | "--yes" | "--allow-dirty" => index += 1,
            value if value.starts_with('-') => {
                return Err(format!("opción git desconocida: {value}"))
            }
            _ => return Err(format!("argumento inesperado: {}", args[index])),
        }
    }
    Ok(repo)
}

fn parse_clone(args: &[String]) -> Result<(String, Option<PathBuf>, Option<String>, bool), String> {
    let mut url = None;
    let mut destination = None;
    let mut branch = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--branch" => {
                let value = args.get(index + 1).ok_or("--branch requiere un nombre")?;
                if !safe_word(value) {
                    return Err("nombre de rama no válido".into());
                }
                branch = Some(value.clone());
                index += 2;
            }
            "--yes" => index += 1,
            value if value.starts_with('-') => {
                return Err(format!("opción git desconocida: {value}"))
            }
            value if url.is_none() => {
                url = Some(value.to_string());
                index += 1;
            }
            value if destination.is_none() => {
                destination = Some(PathBuf::from(value));
                index += 1;
            }
            _ => return Err("git clone solo acepta URL y un destino opcional".into()),
        }
    }
    let url = url.ok_or("git clone requiere una URL")?;
    if branch.as_ref().is_some_and(|value| !safe_word(value)) {
        return Err("nombre de rama no válido".into());
    }
    Ok((url, destination, branch, has_flag(args, "--yes")))
}

fn verify_repo(repo: &Path) -> Result<(), String> {
    if !repo.is_dir() {
        return Err(format!("no es una carpeta: {}", repo.display()));
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("no es un repositorio Git: {}", repo.display()))
    }
}

fn git_output(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn print_output(output: &str) {
    print!("{output}");
}
fn command_value(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}
fn option_value(args: &[String], option: &str) -> Option<String> {
    args.windows(2)
        .find(|values| values[0] == option)
        .map(|values| values[1].clone())
        .or_else(|| {
            args.iter().find_map(|value| {
                value
                    .strip_prefix(&format!("{option}="))
                    .map(ToOwned::to_owned)
            })
        })
}
fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|value| value == flag)
}
fn safe_word(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._/-".contains(character))
}
fn validate_url(url: &str) -> Result<(), String> {
    if url.is_empty()
        || url.len() > 2_048
        || url.starts_with('-')
        || url.chars().any(char::is_whitespace)
        || url.chars().any(char::is_control)
        || url.contains('?')
        || url.contains('#')
    {
        return Err("URL Git vacía, demasiado larga, con espacios o con parámetros/secreto en query; usa una URL limpia y credenciales del gestor del sistema".into());
    }
    let authority_ok = ["https://", "http://", "ssh://"]
        .iter()
        .find_map(|scheme| url.strip_prefix(scheme))
        .is_some_and(|rest| {
            let authority = rest.split('/').next().unwrap_or_default();
            !authority.is_empty() && !authority.contains('@')
        });
    let scp_style_ok = url.strip_prefix("git@").is_some_and(|rest| {
        rest.split_once(':')
            .is_some_and(|(host, path)| !host.is_empty() && !path.is_empty())
    });
    if authority_ok || scp_style_ok {
        Ok(())
    } else {
        Err("solo se aceptan URLs Git HTTP(S), SSH o formato git@host:repo, sin credenciales incrustadas ni query".into())
    }
}
fn record(ctx: &Context, operation: &str, target: &Path, status: &str, args: &[String]) {
    if let Some(plan) = &ctx.plan {
        let _ = plan.record(operation, target, status, false, "git", &args.join(" "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRepo(PathBuf);

    impl TestRepo {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|value| value.as_nanos())
                .unwrap_or_default();
            let path = std::env::temp_dir()
                .join(format!("ltools-git-test-{}-{nonce}", std::process::id()));
            fs::create_dir(&path).expect("temporary test repo directory");
            let init = Command::new("git")
                .arg("-C")
                .arg(&path)
                .args(["init", "-q"])
                .status()
                .expect("git init available for tests");
            assert!(init.success());
            fs::write(path.join("tracked.txt"), "test data\n").expect("test file");
            let add = Command::new("git")
                .arg("-C")
                .arg(&path)
                .args(["add", "tracked.txt"])
                .status()
                .expect("git add");
            assert!(add.success());
            let commit = Command::new("git")
                .arg("-C")
                .arg(&path)
                .args([
                    "-c",
                    "user.name=LTools Test",
                    "-c",
                    "user.email=ltools@example.invalid",
                    "commit",
                    "-m",
                    "test",
                    "-q",
                ])
                .status()
                .expect("git commit");
            assert!(commit.success());
            Self(path)
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn test_context(dry_run: bool) -> Context {
        Context {
            home: std::env::temp_dir(),
            dry_run,
            elevate_by_default: false,
            privileged_child: false,
            plan_path: None,
            plan: None,
        }
    }

    #[test]
    fn accepts_supported_git_urls() {
        assert!(validate_url("https://github.com/a/b.git").is_ok());
        assert!(validate_url("git@github.com:a/b.git").is_ok());
        assert!(validate_url("file:///tmp/a").is_err());
        assert!(validate_url("https://token@example.com/a/b.git").is_err());
        assert!(validate_url("https://example.com/a/b.git?token=secret").is_err());
    }
    #[test]
    fn rejects_unsafe_ref_names() {
        assert!(!safe_word("main branch"));
        assert!(safe_word("release/v1.0"));
    }
    #[test]
    fn clone_parser_keeps_branch_value_out_of_the_url() {
        let (url, destination, branch, yes) = parse_clone(&[
            "--branch".into(),
            "release/v1".into(),
            "https://github.com/example/project.git".into(),
            "/tmp/project".into(),
            "--yes".into(),
        ])
        .expect("clone arguments should parse");
        assert_eq!(url, "https://github.com/example/project.git");
        assert_eq!(destination, Some(PathBuf::from("/tmp/project")));
        assert_eq!(branch.as_deref(), Some("release/v1"));
        assert!(yes);
    }

    #[test]
    fn repairs_only_a_corrupt_index_when_head_and_objects_are_valid() {
        let repo = TestRepo::new();
        let index = repo.0.join(".git/index");
        fs::write(&index, b"not a git index").expect("corrupt test index");
        repair(
            &test_context(false),
            &[
                "--repo".into(),
                repo.0.display().to_string(),
                "--yes".into(),
            ],
        )
        .expect("repair should rebuild the index");
        let listed = Command::new("git")
            .arg("-C")
            .arg(&repo.0)
            .args(["ls-files", "--stage"])
            .output()
            .expect("list repaired index");
        assert!(listed.status.success());
        assert!(String::from_utf8_lossy(&listed.stdout).contains("tracked.txt"));
        assert!(fs::read_dir(repo.0.join(".git"))
            .expect("git directory")
            .filter_map(Result::ok)
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("index.ltools-corrupt-")
                && fs::read(entry.path()).is_ok_and(|value| value == b"not a git index")));
        assert_eq!(
            fs::read(repo.0.join("tracked.txt")).unwrap(),
            b"test data\n"
        );
    }

    #[test]
    fn reconstructs_a_missing_index_and_refuses_a_missing_git_directory() {
        let repo = TestRepo::new();
        let index = repo.0.join(".git/index");
        fs::remove_file(&index).expect("remove index fixture");
        repair(
            &test_context(false),
            &[
                "--repo".into(),
                repo.0.display().to_string(),
                "--yes".into(),
            ],
        )
        .expect("a missing index can be rebuilt from HEAD");
        assert!(index.is_file());
        let tracked = git_command(&repo.0, &["ls-files", "--error-unmatch", "tracked.txt"])
            .expect("verify tracked file");
        assert!(tracked.status.success());

        let no_git = std::env::temp_dir().join(format!(
            "ltools-not-a-git-repo-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
        ));
        fs::create_dir(&no_git).expect("non-repository fixture");
        let error = repair(
            &test_context(false),
            &[
                "--repo".into(),
                no_git.display().to_string(),
                "--yes".into(),
            ],
        )
        .expect_err("repair must not invent a git directory or history");
        assert!(error.contains("no existe un directorio .git"));
        assert!(!no_git.join(".git").exists());
        fs::remove_dir(&no_git).expect("remove non-repository fixture");
    }

    #[test]
    fn missing_git_directory_recovery_requires_remote_and_dry_run_preserves_files() {
        let repo = std::env::temp_dir().join(format!(
            "ltools-git-recovery-dry-run-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
        ));
        fs::create_dir(&repo).expect("workspace fixture");
        let file = repo.join("local-work.txt");
        fs::write(&file, "keep this file\n").expect("local worktree file");

        let error = repair(
            &test_context(false),
            &["--repo".into(), repo.display().to_string()],
        )
        .expect_err("missing history cannot be invented without a remote");
        assert!(error.contains("--remote URL"));
        assert_eq!(fs::read(&file).unwrap(), b"keep this file\n");

        repair(
            &test_context(true),
            &[
                "--repo".into(),
                repo.display().to_string(),
                "--remote".into(),
                "https://example.invalid/team/project.git".into(),
            ],
        )
        .expect("dry-run of safe metadata recovery");
        assert_eq!(fs::read(&file).unwrap(), b"keep this file\n");
        assert!(!repo.join(".git").exists());
        fs::remove_dir_all(repo).expect("remove exact temp fixture");
    }

    #[test]
    fn gh_native_queries_are_version_independent_but_mutations_are_confirmed() {
        assert!(gh_native_is_read_only(&["issue".into(), "list".into()]));
        assert!(gh_native_is_read_only(&["pr".into(), "view".into()]));
        assert!(!gh_native_is_read_only(&["workflow".into(), "run".into()]));
        assert!(!gh_native_is_read_only(&[
            "extension".into(),
            "install".into()
        ]));
        assert!(parse_gh(&["version".into()]).is_ok());
        assert!(parse_gh(&["help".into()]).is_ok());
    }

    #[test]
    fn gh_native_preserves_native_yes_and_only_removes_its_own_confirmation_marker() {
        let raw = [
            "issue".into(),
            "create".into(),
            "--yes".into(),
            "--ltools-confirmed".into(),
        ];
        let (args, confirmed) = parse_gh_native_args(&raw);
        assert!(confirmed);
        assert_eq!(args, ["issue", "create", "--yes"]);
    }

    #[test]
    fn gh_gui_argument_line_splits_without_shell_expansion() {
        assert_eq!(
            split_native_argument_line(
                "issue create --title 'Fix spaces' --body \"$HOME; echo no\" escaped\\ value"
            )
            .unwrap(),
            [
                "issue",
                "create",
                "--title",
                "Fix spaces",
                "--body",
                "$HOME; echo no",
                "escaped value"
            ]
        );
        assert!(split_native_argument_line("issue 'unfinished").is_err());
        assert!(split_native_argument_line("issue trailing\\").is_err());
        assert!(split_native_argument_line("issue\nlist").is_err());
    }

    #[test]
    fn gh_gui_rejects_duplicate_repository_and_keeps_native_arguments() {
        assert_eq!(
            gh_gui_arguments("issue", "OWNER/REPO", "list --state open").unwrap(),
            [
                "gh",
                "native",
                "issue",
                "list",
                "--state",
                "open",
                "--repo",
                "OWNER/REPO"
            ]
        );
        assert!(gh_gui_arguments("issue", "OWNER/REPO", "list --repo OTHER/REPO").is_err());
        assert!(gh_gui_arguments("issue", "OWNER/REPO", "list --repo=OTHER/REPO").is_err());
        assert!(gh_gui_arguments("issue", "OWNER/REPO", "list -R OTHER/REPO").is_err());
        assert!(gh_gui_arguments("issue", "OWNER/REPO", "list -ROWNER/REPO").is_err());
        assert!(gh_gui_arguments("issue", "OWNER/REPO", "list -- --repo positional").is_ok());
        assert!(gh_gui_arguments("issue list", "", "").is_err());
        assert!(gh_gui_arguments("api", "OWNER/REPO", "repos/{owner}/{repo}").is_err());
        assert!(gh_gui_arguments("issue", "OWNER", "list").is_err());
        assert!(gh_gui_arguments("issue", "../OWNER/REPO", "list").is_err());
        assert!(parse_gh(&["auth-status".into(), "--repo".into(), "OWNER/REPO".into()]).is_err());
        assert!(parse_gh(&["repo".into(), "--repo".into(), "OWNER/REPO".into()]).is_ok());
        assert!(parse_gh(&[
            "repo".into(),
            "--repo".into(),
            "github.example/OWNER/REPO".into()
        ])
        .is_ok());
    }

    #[test]
    fn git_gui_action_labels_exist_for_each_supported_language() {
        let _language_guard = crate::i18n::language_test_guard();
        for language in crate::i18n::SUPPORTED {
            crate::i18n::set(language);
            for key in [
                "version",
                "help",
                "native",
                "diagnose",
                "repair_index",
                "repair_remote",
                "recovery_heading",
                "native_title",
                "native_command",
                "native_arguments",
                "remote_url",
                "branch",
                "execute",
            ] {
                assert!(
                    !crate::i18n::git_action_text(key).is_empty(),
                    "{language}: {key}"
                );
            }
        }
    }
}
