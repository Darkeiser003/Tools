use crate::common::{ask, command_exists, Context};
use std::path::Path;
use std::process::Command;

/// Administración nativa de cuentas locales Windows mediante PowerShell.
/// Las consultas usan tablas y las operaciones mutantes requieren
/// confirmación, privilegios y dry-run para previsualizar.
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    match first(args).unwrap_or("menu") {
        "list" | "users" => powershell("Get-LocalUser | Sort-Object Name | Format-Table Name,Enabled,LastLogon,PasswordRequired -AutoSize"),
        "groups" | "group-list" => powershell("Get-LocalGroup | Sort-Object Name | Format-Table Name,Description -AutoSize"),
        "sessions" => powershell(
            "Get-CimInstance Win32_ComputerSystem | Select-Object @{Name='USUARIO_ACTIVO';Expression={$_.UserName}},@{Name='EQUIPO';Expression={$_.Name}} | Format-Table -AutoSize",
        ),
        "inspect" | "details" => inspect(target_after(args, first(args).unwrap_or("inspect"))?),
        "add" | "create" => create_user(ctx, args),
        "modify" | "edit" => modify_user(ctx, args),
        "password" | "passwd" => change_password(ctx, args),
        "enable" | "unlock" => simple_user_mutation(ctx, args, "activar la cuenta", "Enable-LocalUser"),
        "disable" | "lock" => simple_user_mutation(ctx, args, "desactivar la cuenta", "Disable-LocalUser"),
        "delete" | "remove" => delete_user(ctx, args),
        "expire" => expire_user(ctx, args),
        "group-create" => group_mutation(ctx, args, true, false),
        "group-delete" => group_mutation(ctx, args, false, false),
        "group-add" => group_mutation(ctx, args, true, true),
        "group-remove" => group_mutation(ctx, args, false, true),
        "set-primary-group" => Err("Windows no expone un grupo primario local equivalente; usa group-add para administrar membresías.".into()),
        "open-lusrmgr" => open_lusrmgr(ctx),
        "menu" => menu(ctx),
        _ => Err("accounts admite list, groups, sessions, inspect USER, create USER, modify USER, password USER, enable USER, disable USER, delete USER, expire USER, group-create GROUP, group-delete GROUP, group-add USER:GROUP, group-remove USER:GROUP, open-lusrmgr o menu".into()),
    }
}

fn first(args: &[String]) -> Option<&str> {
    args.iter()
        .map(String::as_str)
        .find(|value| !value.starts_with('-'))
}

fn positional_after<'a>(args: &'a [String], action: &str) -> Option<&'a str> {
    args.iter()
        .position(|value| value == action)
        .and_then(|index| {
            args.get(index + 1)
                .map(String::as_str)
                .filter(|value| !value.starts_with('-'))
        })
}

fn target_after<'a>(args: &'a [String], action: &str) -> Result<&'a str, String> {
    positional_after(args, action)
        .or_else(|| {
            args.iter()
                .position(|value| value == "--user")
                .and_then(|index| args.get(index + 1))
                .map(String::as_str)
        })
        .or_else(|| {
            args.iter()
                .position(|value| value == "--group")
                .and_then(|index| args.get(index + 1))
                .map(String::as_str)
        })
        .ok_or_else(|| format!("{action} requiere un usuario o grupo"))
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|value| value == name)
        .and_then(|index| args.get(index + 1))
        .filter(|value| !value.starts_with('-'))
        .cloned()
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|value| value == name)
}

fn valid_name(raw: &str) -> Result<&str, String> {
    let value = raw.trim();
    if value.is_empty()
        || value.len() > 64
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".@_-".contains(c))
    {
        Err("nombre de cuenta o grupo no válido".into())
    } else {
        Ok(value)
    }
}

fn valid_value<'a>(raw: &'a str, label: &str) -> Result<&'a str, String> {
    if raw.is_empty() || raw.contains(['\0', '\r', '\n', '\'']) {
        Err(format!("{label} no válido"))
    } else {
        Ok(raw)
    }
}

fn ps_quote(value: &str, label: &str) -> Result<String, String> {
    Ok(format!(
        "'{}'",
        valid_value(value, label)?.replace('\'', "''")
    ))
}

fn inspect(user: &str) -> Result<(), String> {
    let user = valid_name(user)?;
    let quoted = ps_quote(user, "usuario")?;
    powershell(&format!("$u=Get-LocalUser -Name {quoted}; $u | Select-Object Name,Enabled,Description,FullName,LastLogon,PasswordRequired,UserMayChangePassword,PasswordExpires,AccountExpires | Format-Table -AutoSize; Write-Output '--- Grupos ---'; Get-LocalGroup | ForEach-Object {{ if (Get-LocalGroupMember -Group $_.Name -ErrorAction SilentlyContinue | Where-Object Name -like '*\\\\{user}') {{ [PSCustomObject]@{{Grupo=$_.Name; Usuario={quoted}}} }} }} | Format-Table -AutoSize"))
}

fn create_user(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, first(args).unwrap_or("create"))?)?;
    let quoted = ps_quote(user, "usuario")?;
    let mut parts = vec![format!("New-LocalUser -Name {quoted}")];
    if let Some(value) = option_value(args, "--description") {
        parts.push(format!("-Description {}", ps_quote(&value, "descripción")?));
    }
    if let Some(value) = option_value(args, "--full-name") {
        parts.push(format!(
            "-FullName {}",
            ps_quote(&value, "nombre completo")?
        ));
    }
    if let Some(value) = option_value(args, "--password-file") {
        parts.push(format!("-Password (ConvertTo-SecureString (Get-Content -Raw -LiteralPath {}) -AsPlainText -Force)", ps_quote(&value, "archivo de contraseña")?));
    }
    if has_flag(args, "--password-never-expires") {
        parts.push("-PasswordNeverExpires $true".into());
    }
    run_mutation(
        ctx,
        user,
        "crear la cuenta",
        parts.join(" "),
        "New-LocalUser",
    )
}

fn modify_user(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, first(args).unwrap_or("modify"))?)?;
    let quoted = ps_quote(user, "usuario")?;
    if let Some(new_name) = option_value(args, "--login") {
        let script = format!(
            "Rename-LocalUser -Name {quoted} -NewName {}",
            ps_quote(&new_name, "nuevo nombre")?
        );
        return run_mutation(ctx, user, "renombrar la cuenta", script, "Rename-LocalUser");
    }
    let mut parts = vec![format!("Set-LocalUser -Name {quoted}")];
    if let Some(value) = option_value(args, "--description") {
        parts.push(format!("-Description {}", ps_quote(&value, "descripción")?));
    }
    if let Some(value) = option_value(args, "--full-name") {
        parts.push(format!(
            "-FullName {}",
            ps_quote(&value, "nombre completo")?
        ));
    }
    if let Some(value) = option_value(args, "--password-never-expires") {
        parts.push(format!(
            "-PasswordNeverExpires {}",
            if value.eq_ignore_ascii_case("true") {
                "$true"
            } else {
                "$false"
            }
        ));
    }
    if parts.len() == 1 {
        return Err(
            "modify requiere --description, --full-name, --password-never-expires o --login".into(),
        );
    }
    run_mutation(
        ctx,
        user,
        "editar la cuenta",
        parts.join(" "),
        "Set-LocalUser",
    )
}

fn change_password(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, first(args).unwrap_or("password"))?)?;
    let file = option_value(args, "--password-file")
        .ok_or("password en Windows requiere --password-file desde la GUI o un archivo seguro")?;
    let script = format!("$p=ConvertTo-SecureString (Get-Content -Raw -LiteralPath {}) -AsPlainText -Force; Set-LocalUser -Name {} -Password $p; Remove-Item -LiteralPath {} -Force -ErrorAction SilentlyContinue", ps_quote(&file, "archivo de contraseña")?, ps_quote(user, "usuario")?, ps_quote(&file, "archivo de contraseña")?);
    run_mutation(
        ctx,
        user,
        "cambiar la contraseña",
        script,
        "Set-LocalUser -Password <stdin/file>",
    )
}

fn simple_user_mutation(
    ctx: &Context,
    args: &[String],
    description: &str,
    cmdlet: &str,
) -> Result<(), String> {
    let user = valid_name(target_after(args, first(args).unwrap_or("user"))?)?;
    run_mutation(
        ctx,
        user,
        description,
        format!("{cmdlet} -Name {}", ps_quote(user, "usuario")?),
        cmdlet,
    )
}

fn delete_user(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, first(args).unwrap_or("delete"))?)?;
    run_mutation(
        ctx,
        user,
        "eliminar la cuenta",
        format!(
            "Remove-LocalUser -Name {} -Confirm:$false",
            ps_quote(user, "usuario")?
        ),
        "Remove-LocalUser",
    )
}

fn expire_user(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, "expire")?)?;
    let date = option_value(args, "--date").ok_or("expire requiere --date YYYY-MM-DD")?;
    let script = format!(
        "Set-LocalUser -Name {} -AccountExpires (Get-Date {})",
        ps_quote(user, "usuario")?,
        ps_quote(&date, "fecha")?
    );
    run_mutation(
        ctx,
        user,
        "cambiar la caducidad de la cuenta",
        script,
        "Set-LocalUser -AccountExpires",
    )
}

fn group_mutation(
    ctx: &Context,
    args: &[String],
    add: bool,
    membership: bool,
) -> Result<(), String> {
    if membership {
        let (user, group) = if let (Some(user), Some(group)) =
            (option_value(args, "--user"), option_value(args, "--group"))
        {
            (user, group)
        } else {
            let raw = positional_after(args, first(args).unwrap_or("group-add"))
                .ok_or("usa el formato USUARIO:GRUPO")?;
            let (user, group) = raw.split_once(':').ok_or("usa el formato USUARIO:GRUPO")?;
            (user.to_owned(), group.to_owned())
        };
        let user = valid_name(&user)?;
        let group = valid_name(&group)?;
        let script = if add {
            format!(
                "Add-LocalGroupMember -Group {} -Member {}",
                ps_quote(group, "grupo")?,
                ps_quote(user, "usuario")?
            )
        } else {
            format!(
                "Remove-LocalGroupMember -Group {} -Member {} -Confirm:$false",
                ps_quote(group, "grupo")?,
                ps_quote(user, "usuario")?
            )
        };
        return run_mutation(
            ctx,
            user,
            if add {
                "añadir el usuario al grupo"
            } else {
                "retirar el usuario del grupo"
            },
            script,
            if add {
                "Add-LocalGroupMember"
            } else {
                "Remove-LocalGroupMember"
            },
        );
    }
    let group = option_value(args, "--group")
        .or_else(|| positional_after(args, first(args).unwrap_or("group")).map(str::to_owned))
        .ok_or("falta el grupo")?;
    let group = valid_name(&group)?;
    let script = if add {
        format!("New-LocalGroup -Name {}", ps_quote(group, "grupo")?)
    } else {
        format!(
            "Remove-LocalGroup -Name {} -Confirm:$false",
            ps_quote(group, "grupo")?
        )
    };
    run_mutation(
        ctx,
        group,
        if add {
            "crear el grupo"
        } else {
            "eliminar el grupo"
        },
        script,
        if add {
            "New-LocalGroup"
        } else {
            "Remove-LocalGroup"
        },
    )
}

fn run_mutation(
    ctx: &Context,
    target: &str,
    description: &str,
    script: String,
    record_data: &str,
) -> Result<(), String> {
    let preconfirmed = std::env::var_os("LTOOLS_ACCOUNT_GUI_PRECONFIRMED").is_some()
        && std::env::var("LTOOLS_FRONTEND").ok().as_deref() == Some("gui");
    if !ctx.dry_run && !preconfirmed && !ask(&format!("¿Quieres {description} '{target}'?")) {
        record(ctx, target, "cancelled", record_data)?;
        return Ok(());
    }
    if ctx.dry_run {
        println!("=== Operación de cuentas ===\nSimulación: se ejecutaría {script}");
        record(ctx, target, "planned", record_data)?;
        return Ok(());
    }
    let result = powershell(&script);
    record(
        ctx,
        target,
        if result.is_ok() { "executed" } else { "failed" },
        record_data,
    )?;
    result
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
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn record(ctx: &Context, target: &str, status: &str, data: &str) -> Result<(), String> {
    if let Some(plan) = &ctx.plan {
        plan.record(
            "account-mutation",
            Path::new(target),
            status,
            false,
            data,
            "accounts",
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn powershell(script: &str) -> Result<(), String> {
    let program = if command_exists("powershell") {
        "powershell"
    } else {
        "pwsh"
    };
    let output = Command::new(program)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|error| error.to_string())?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Usuarios, grupos y sesiones Windows ===");
        println!(" 1) Cuentas  2) Grupos  3) Sesiones  4) Inspeccionar  5) Crear  6) Editar");
        println!(" 7) Contraseña  8) Activar  9) Desactivar  10) Eliminar  11) Caducidad");
        println!("12) Crear grupo  13) Eliminar grupo  14) Añadir miembro  15) Retirar miembro");
        println!("16) Administración avanzada  q) Volver");
        let choice =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        let result = match choice.trim() {
            "1" => run(ctx, &["list".into()]),
            "2" => run(ctx, &["groups".into()]),
            "3" => run(ctx, &["sessions".into()]),
            "4" => prompt(ctx, "Usuario: ", |_, value| inspect(value)),
            "5" => prompt(ctx, "Nuevo usuario: ", |c, value| {
                create_user(c, &vec!["create".into(), value.into()])
            }),
            "6" => prompt(ctx, "Usuario: ", |c, value| {
                modify_user(c, &vec!["modify".into(), value.into()])
            }),
            "7" => prompt(ctx, "Usuario: ", |c, value| {
                change_password(c, &vec!["password".into(), value.into()])
            }),
            "8" => prompt(ctx, "Usuario: ", |c, value| {
                simple_user_mutation(
                    c,
                    &vec!["enable".into(), value.into()],
                    "activar la cuenta",
                    "Enable-LocalUser",
                )
            }),
            "9" => prompt(ctx, "Usuario: ", |c, value| {
                simple_user_mutation(
                    c,
                    &vec!["disable".into(), value.into()],
                    "desactivar la cuenta",
                    "Disable-LocalUser",
                )
            }),
            "10" => prompt(ctx, "Usuario: ", |c, value| {
                delete_user(c, &vec!["delete".into(), value.into()])
            }),
            "11" => prompt(ctx, "Usuario: ", |c, value| {
                expire_user(c, &vec!["expire".into(), value.into()])
            }),
            "12" => prompt(ctx, "Grupo nuevo: ", |c, value| {
                group_mutation(c, &vec!["group-create".into(), value.into()], true, false)
            }),
            "13" => prompt(ctx, "Grupo: ", |c, value| {
                group_mutation(c, &vec!["group-delete".into(), value.into()], false, false)
            }),
            "14" => prompt(ctx, "Usuario:Grupo: ", |c, value| {
                group_mutation(c, &vec!["group-add".into(), value.into()], true, true)
            }),
            "15" => prompt(ctx, "Usuario:Grupo: ", |c, value| {
                group_mutation(c, &vec!["group-remove".into(), value.into()], false, true)
            }),
            "16" => open_lusrmgr(ctx),
            "" | "q" | "Q" => return Ok(()),
            _ => Ok(()),
        };
        if let Err(error) = result {
            println!("Error: {error}");
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
    if let Some(value) = crate::common::prompt_path(question) {
        let value = value.to_string_lossy();
        if !value.trim().is_empty() {
            action(ctx, &value)?;
        }
    }
    Ok(())
}
