use crate::common::{ask, command_exists, Context};
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    match first(args).unwrap_or("menu") {
        "list" | "users" => powershell("Get-LocalUser | Sort-Object Name | Format-Table Name,Enabled,LastLogon,PasswordRequired -AutoSize"),
        "groups" => powershell("Get-LocalGroup | Sort-Object Name | Format-Table Name,Description -AutoSize"),
        "sessions" => native("query", &[]),
        "inspect" | "details" => inspect(target_after(args, first(args).unwrap_or("inspect"))?),
        "enable" => mutate(ctx, target_after(args, "enable")?, "activar la cuenta", "Enable-LocalUser"),
        "disable" => mutate(ctx, target_after(args, "disable")?, "desactivar la cuenta", "Disable-LocalUser"),
        "delete" | "remove" => mutate(ctx, target_after(args, first(args).unwrap_or("delete"))?, "eliminar la cuenta", "Remove-LocalUser"),
        "open-lusrmgr" => open_lusrmgr(ctx), "menu" => menu(ctx),
        _ => Err("accounts admite list, groups, sessions, inspect USER, enable USER, disable USER, delete USER, open-lusrmgr o menu".into()),
    }
}
fn first(args: &[String]) -> Option<&str> {
    args.iter()
        .map(String::as_str)
        .find(|v| !v.starts_with('-'))
}
fn target_after<'a>(args: &'a [String], action: &str) -> Result<&'a str, String> {
    let i = args
        .iter()
        .position(|v| v == action)
        .ok_or("falta la acción")?;
    args.get(i + 1)
        .map(String::as_str)
        .filter(|v| !v.starts_with('-'))
        .ok_or_else(|| format!("{action} requiere un usuario"))
}
fn valid_name(raw: &str) -> Result<&str, String> {
    let v = raw.trim();
    if v.is_empty()
        || v.len() > 64
        || !v
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".@_-".contains(c))
    {
        Err("nombre de cuenta no válido".into())
    } else {
        Ok(v)
    }
}
fn inspect(user: &str) -> Result<(), String> {
    let user = valid_name(user)?;
    powershell(&format!("Get-LocalUser -Name '{user}' | Format-List *; Get-LocalGroup | ForEach-Object {{ if (Get-LocalGroupMember -Group $_.Name -ErrorAction SilentlyContinue | Where-Object Name -like '*\\{user}') {{ $_.Name }} }}"))
}
fn mutate(ctx: &Context, user: &str, description: &str, cmdlet: &str) -> Result<(), String> {
    let user = valid_name(user)?;
    if !ctx.dry_run && !ask(&format!("¿Quieres {description} '{user}'?")) {
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se ejecutaría {cmdlet} -Name {user}.");
        return Ok(());
    }
    powershell(&format!("{cmdlet} -Name '{user}'"))
}
fn open_lusrmgr(ctx: &Context) -> Result<(), String> {
    if !command_exists("lusrmgr.msc") {
        return Err("lusrmgr.msc no está disponible en esta edición de Windows".into());
    }
    if !ctx.dry_run && !ask("¿Abrir Administración de usuarios locales?") {
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se abriría lusrmgr.msc.");
        return Ok(());
    }
    Command::new("lusrmgr.msc")
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Usuarios, grupos y sesiones Windows ===\n  1) Listar cuentas\n  2) Listar grupos\n  3) Sesiones abiertas\n  4) Inspeccionar cuenta\n  5) Activar cuenta\n  6) Desactivar cuenta\n  7) Eliminar cuenta\n  8) Abrir lusrmgr.msc\n  q) Volver");
        let choice =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        let result = match choice.trim() { "1" => powershell("Get-LocalUser | Sort-Object Name | Format-Table Name,Enabled,LastLogon,PasswordRequired -AutoSize"), "2" => powershell("Get-LocalGroup | Sort-Object Name | Format-Table Name,Description -AutoSize"), "3" => native("query", &[]), "4" => prompt(ctx, "Usuario: ", |_,v| inspect(v)), "5" => prompt(ctx, "Usuario: ", |c,v| mutate(c,v,"activar la cuenta","Enable-LocalUser")), "6" => prompt(ctx, "Usuario: ", |c,v| mutate(c,v,"desactivar la cuenta","Disable-LocalUser")), "7" => prompt(ctx, "Usuario: ", |c,v| mutate(c,v,"eliminar la cuenta","Remove-LocalUser")), "8" => open_lusrmgr(ctx), ""|"q"|"Q" => return Ok(()), _ => Ok(()) };
        if let Err(e) = result {
            println!("Error: {e}");
        }
        if !choice.trim().is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}
fn prompt<F>(ctx: &Context, question: &str, action: F) -> Result<(), String>
where
    F: FnOnce(&Context, &str) -> Result<(), String>,
{
    if let Some(v) = crate::common::prompt_path(question) {
        let v = v.to_string_lossy();
        if !v.trim().is_empty() {
            action(ctx, &v)?;
        }
    }
    Ok(())
}
fn powershell(script: &str) -> Result<(), String> {
    let p = if command_exists("powershell") {
        "powershell"
    } else {
        "pwsh"
    };
    let o = Command::new(p)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| e.to_string())?;
    print!("{}", String::from_utf8_lossy(&o.stdout));
    if o.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
    }
}
fn native(program: &str, args: &[&str]) -> Result<(), String> {
    let o = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    print!("{}", String::from_utf8_lossy(&o.stdout));
    if o.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
    }
}
