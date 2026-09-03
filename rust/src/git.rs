//! Operaciones Git explícitas, sin shell y con protección para repositorios.

use crate::common::{self, Context};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn help() -> &'static str {
    "git status [--repo PATH] | git clone URL [DEST] [--branch BRANCH] | git fetch [--repo PATH] [--prune] | git pull [--repo PATH] [--rebase] | git login"
}

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let operation = args.first().map(String::as_str).unwrap_or("status");
    match operation {
        "status" => status(args.get(1..).unwrap_or_default()),
        "clone" => clone(ctx, args.get(1..).unwrap_or_default()),
        "fetch" => fetch(ctx, args.get(1..).unwrap_or_default()),
        "pull" => pull(ctx, args.get(1..).unwrap_or_default()),
        "login" | "auth" => login(ctx),
        _ => Err(format!("operación git desconocida: {operation}")),
    }
}

fn require_git() -> Result<(), String> {
    if common::command_exists("git") {
        Ok(())
    } else {
        Err("Git no está instalado; LTools no lo instala automáticamente. Instálalo desde el gestor de tu sistema y vuelve a intentarlo.".into())
    }
}

fn status(args: &[String]) -> Result<(), String> {
    require_git()?;
    let repo = parse_repo(args)?;
    let output = git_output(&repo, &["status", "--short", "--branch"])?;
    print_output(&output);
    Ok(())
}

fn clone(ctx: &Context, args: &[String]) -> Result<(), String> {
    require_git()?;
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
    require_git()?;
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
    require_git()?;
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

fn login(ctx: &Context) -> Result<(), String> {
    require_git()?;
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
