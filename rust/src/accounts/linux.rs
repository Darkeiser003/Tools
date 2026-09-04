use crate::common::{ask, command_exists, run_with_sudo, Context};
use std::io::{self, Write};
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = first(args).unwrap_or("menu");
    match action {
        "list" | "users" => capture("getent", &["passwd"]),
        "identity" | "current" => identity(),
        "groups" => groups(target_after(args, "groups")?),
        "sessions" => sessions(),
        "inspect" | "details" => inspect(target_after(args, action)?),
        "add" | "create" => mutate(ctx, target_after(args, action)?, "crear la cuenta", &["useradd", "--create-home"]),
        "lock" => mutate(ctx, target_after(args, action)?, "bloquear la cuenta", &["usermod", "--lock"]),
        "unlock" => mutate(ctx, target_after(args, action)?, "desbloquear la cuenta", &["usermod", "--unlock"]),
        "delete" | "remove" => mutate(ctx, target_after(args, action)?, "eliminar la cuenta (sin borrar su carpeta personal)", &["userdel"]),
        "group-add" => group_mutation(ctx, target_after(args, action)?, true),
        "group-remove" => group_mutation(ctx, target_after(args, action)?, false),
        "menu" => menu(ctx),
        _ => Err("accounts admite list, identity, groups USER, sessions, inspect USER, add USER, lock USER, unlock USER, delete USER, group-add USER:GROUP, group-remove USER:GROUP o menu".into()),
    }
}

fn first(args: &[String]) -> Option<&str> {
    args.iter()
        .map(String::as_str)
        .find(|value| !value.starts_with('-'))
}
fn target_after<'a>(args: &'a [String], action: &str) -> Result<&'a str, String> {
    let index = args
        .iter()
        .position(|value| value == action)
        .ok_or("falta la acción")?;
    args.get(index + 1)
        .map(String::as_str)
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| format!("{action} requiere un objetivo"))
}
fn valid_name(raw: &str) -> Result<&str, String> {
    let value = raw.trim();
    if value.is_empty()
        || value.len() > 64
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".@_-:+".contains(c))
    {
        return Err("nombre de usuario o grupo no válido".into());
    }
    Ok(value)
}
fn groups(user: &str) -> Result<(), String> {
    capture("id", &["--groups", "--name", valid_name(user)?])
}
fn identity() -> Result<(), String> {
    println!("=== Identidad actual ===");
    println!("Identidad completa:");
    capture("id", &[])?;
    println!("Usuario:");
    capture("id", &["--user", "--name"])?;
    println!("Grupo principal:");
    capture("id", &["--group", "--name"])?;
    println!("Grupos:");
    capture("id", &["--groups", "--name"])
}
fn sessions() -> Result<(), String> {
    println!("=== Sesiones abiertas ===");
    if command_exists("who") {
        capture("who", &[])?;
    }
    if command_exists("loginctl") {
        println!("\n=== Usuarios de logind ===");
        capture("loginctl", &["list-users", "--no-legend"])?;
    }
    Ok(())
}
fn inspect(user: &str) -> Result<(), String> {
    let user = valid_name(user)?;
    println!("=== Cuenta: {user} ===");
    if command_exists("getent") {
        capture("getent", &["passwd", user])?;
    }
    groups(user)
}
fn mutate(ctx: &Context, user: &str, description: &str, command: &[&str]) -> Result<(), String> {
    let user = valid_name(user)?;
    if !ctx.dry_run && !ask(&format!("¿Quieres {description} '{user}'?")) {
        println!("Operación cancelada; no se modificó ninguna cuenta.");
        return Ok(());
    }
    let mut args = command
        .iter()
        .skip(1)
        .map(|v| (*v).to_string())
        .collect::<Vec<_>>();
    args.push(user.to_string());
    let ok = run_with_sudo(command[0], &args, ctx.dry_run).map_err(|e| e.to_string())?;
    if !ok {
        return Err(format!(
            "{} no pudo completar la operación sobre {user}",
            command[0]
        ));
    }
    Ok(())
}
fn group_mutation(ctx: &Context, raw: &str, add: bool) -> Result<(), String> {
    let (user, group) = raw.split_once(':').ok_or("usa el formato USUARIO:GRUPO")?;
    let user = valid_name(user)?;
    let group = valid_name(group)?;
    if !ctx.dry_run
        && !ask(&format!(
            "¿Quieres {} el grupo '{group}' {}?",
            if add { "añadir" } else { "retirar" },
            user
        ))
    {
        return Ok(());
    }
    let args = if add {
        vec!["-aG".into(), group.into(), user.into()]
    } else {
        vec!["-d".into(), group.into(), user.into()]
    };
    if !run_with_sudo("usermod", &args, ctx.dry_run).map_err(|e| e.to_string())? {
        return Err("usermod no pudo modificar el grupo".into());
    }
    Ok(())
}
fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Usuarios, grupos y sesiones Linux ===");
        println!("  1) Listar cuentas\n  2) Mi identidad y grupos\n  3) Sesiones abiertas\n  4) Inspeccionar una cuenta\n  5) Crear cuenta\n  6) Bloquear cuenta\n  7) Desbloquear cuenta\n  8) Eliminar cuenta sin borrar su carpeta\n  9) Añadir grupo\n 10) Retirar grupo\n  q) Volver");
        print!("Elige una opción (Enter para volver): ");
        let _ = io::stdout().flush();
        let mut answer = String::new();
        if io::stdin().read_line(&mut answer).is_err() {
            return Ok(());
        }
        let result = match answer.trim() {
            "1" => capture("getent", &["passwd"]),
            "2" => identity(),
            "3" => sessions(),
            "4" => prompt(ctx, "Usuario: ", |_, v| inspect(v)),
            "5" => prompt(ctx, "Nuevo usuario: ", |c, v| {
                mutate(c, v, "crear la cuenta", &["useradd", "--create-home"])
            }),
            "6" => prompt(ctx, "Usuario: ", |c, v| {
                mutate(c, v, "bloquear la cuenta", &["usermod", "--lock"])
            }),
            "7" => prompt(ctx, "Usuario: ", |c, v| {
                mutate(c, v, "desbloquear la cuenta", &["usermod", "--unlock"])
            }),
            "8" => prompt(ctx, "Usuario: ", |c, v| {
                mutate(
                    c,
                    v,
                    "eliminar la cuenta (sin borrar su carpeta personal)",
                    &["userdel"],
                )
            }),
            "9" => prompt(ctx, "Usuario:Grupo: ", |c, v| group_mutation(c, v, true)),
            "10" => prompt(ctx, "Usuario:Grupo: ", |c, v| group_mutation(c, v, false)),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.trim().is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}
fn prompt<F>(ctx: &Context, question: &str, action: F) -> Result<(), String>
where
    F: FnOnce(&Context, &str) -> Result<(), String>,
{
    if let Some(value) = crate::common::prompt_path(question) {
        let value = value.to_string_lossy();
        if !value.trim().is_empty() {
            action(ctx, &value)?;
        }
    }
    Ok(())
}
fn capture(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("no se pudo ejecutar {program}: {e}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::valid_name;
    #[test]
    fn rechaza_fragmentos_de_shell() {
        assert!(valid_name("user;rm -rf /").is_err());
        assert!(valid_name("romen").is_ok());
    }
}
