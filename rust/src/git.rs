//! Operaciones Git explícitas, sin shell y con protección para repositorios.

use crate::common::{self, Context};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn help() -> &'static str {
    "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|gh|login [opciones seguras]"
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

fn gh(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_gh(ctx)?;
    let (operation, repo, yes) = parse_gh(args)?;
    let mut command_args = match operation.as_str() {
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
            || operation == "releases",
        "¿Ejecutar esta operación de GitHub?",
    )
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
                validate_ref(value, "repositorio GitHub")?;
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
        "auth-status" | "login" | "repo" | "prs" | "releases"
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
    if url.starts_with("https://")
        || url.starts_with("http://")
        || url.starts_with("ssh://")
        || url.starts_with("git@")
    {
        Ok(())
    } else {
        Err("solo se aceptan URLs Git HTTP(S), SSH o formato git@host:repo".into())
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
    #[test]
    fn accepts_supported_git_urls() {
        assert!(validate_url("https://github.com/a/b.git").is_ok());
        assert!(validate_url("git@github.com:a/b.git").is_ok());
        assert!(validate_url("file:///tmp/a").is_err());
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
}
