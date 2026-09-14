use crate::common::{ask, command_exists, run_with_sudo, run_with_sudo_input, Context};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Administración Linux de usuarios, grupos y sesiones. Las consultas se
/// presentan como tablas estables; las mutaciones construyen argumentos
/// separados, muestran la orden y exigen confirmación.
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    match first(args).unwrap_or("menu") {
        "list" | "users" => list_users(args),
        "identity" | "current" => identity(),
        "groups" => match option_value(args, "--user") {
            Some(user) => user_groups(&user),
            None => match positional_after(args, "groups") {
                Some(user) => user_groups(user),
                None => list_groups(),
            },
        },
        "group-list" => list_groups(),
        "sessions" => sessions(),
        "inspect" | "details" => inspect(target_after(args, first(args).unwrap_or("inspect"))?),
        "add" | "create" => create_user(ctx, args),
        "modify" | "edit" => modify_user(ctx, args),
        "password" | "passwd" => change_password(ctx, args),
        "lock" => simple_user_mutation(ctx, args, "bloquear la cuenta", vec!["usermod".into(), "--lock".into()]),
        "unlock" => simple_user_mutation(ctx, args, "desbloquear la cuenta", vec!["usermod".into(), "--unlock".into()]),
        "delete" | "remove" => delete_user(ctx, args),
        "expire" => expire_user(ctx, args),
        "group-create" => group_mutation(ctx, args, true, false),
        "group-delete" => group_mutation(ctx, args, false, false),
        "group-add" => group_mutation(ctx, args, true, true),
        "group-remove" => group_mutation(ctx, args, false, true),
        "admin-add" | "grant-admin" => add_administrator_access(ctx, args),
        "admin-groups" => list_administrator_groups(),
        "set-primary-group" => set_primary_group(ctx, args),
        "menu" => menu(ctx),
        _ => Err("accounts admite list, identity, groups [USER], group-list, sessions, inspect USER, create USER, modify USER, password USER, lock USER, unlock USER, delete USER, expire USER, group-create GROUP, group-delete GROUP, group-add USER:GROUP, group-remove USER:GROUP, admin-groups, admin-add [USER] [--group sudo|wheel|admin], set-primary-group USER:GROUP o menu".into()),
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
        || value.starts_with('-')
        || value.len() > 64
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".@_-+".contains(c))
    {
        return Err("nombre de usuario o grupo no válido".into());
    }
    Ok(value)
}

fn valid_path(raw: &str, label: &str) -> Result<String, String> {
    let path = PathBuf::from(raw);
    if !path.is_absolute() || raw.contains('\0') || raw.contains('\n') {
        return Err(format!(
            "{label} debe ser una ruta absoluta sin saltos de línea"
        ));
    }
    Ok(path.display().to_string())
}

fn list_users(args: &[String]) -> Result<(), String> {
    let output = capture_output("getent", &["passwd"])?;
    let mut rows = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let fields = line.splitn(7, ':').collect::<Vec<_>>();
        if fields.len() == 7 {
            rows.push(vec![
                fields[0].to_owned(),
                fields[2].to_owned(),
                fields[3].to_owned(),
                fields[4].to_owned(),
                fields[5].to_owned(),
                fields[6].to_owned(),
            ]);
        }
    }
    if has_flag(args, "--human") {
        rows.retain(|row| {
            !matches!(
                row[5].as_str(),
                "/usr/sbin/nologin"
                    | "/usr/bin/nologin"
                    | "/sbin/nologin"
                    | "/bin/nologin"
                    | "/bin/false"
            )
        });
    }
    println!("=== Cuentas Linux ===");
    println!(
        "{}",
        crate::formatting::table(
            &["USUARIO", "UID", "GID", "DESCRIPCIÓN", "HOME", "SHELL"],
            &rows
        )
    );
    Ok(())
}

fn list_groups() -> Result<(), String> {
    let output = capture_output("getent", &["group"])?;
    let mut rows = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        if fields.len() == 4 {
            rows.push(vec![
                fields[0].to_owned(),
                fields[2].to_owned(),
                fields[3].to_owned(),
            ]);
        }
    }
    println!("=== Grupos Linux ===");
    println!(
        "{}",
        crate::formatting::table(&["GRUPO", "GID", "MIEMBROS"], &rows)
    );
    Ok(())
}

fn user_groups(user: &str) -> Result<(), String> {
    let user = valid_name(user)?;
    let groups = capture_output("id", &["-Gn", user])?;
    let rows = groups
        .split_whitespace()
        .map(|group| vec![user.to_owned(), group.to_owned()])
        .collect::<Vec<_>>();
    println!("=== Grupos de {user} ===");
    println!("{}", crate::formatting::table(&["USUARIO", "GRUPO"], &rows));
    Ok(())
}

fn identity() -> Result<(), String> {
    let user = capture_output("id", &["-un"])?;
    let uid = capture_output("id", &["-u"])?;
    let gid = capture_output("id", &["-g"])?;
    let primary = capture_output("id", &["-gn"])?;
    let groups = capture_output("id", &["-Gn"])?;
    let rows = vec![
        vec!["Usuario".into(), user],
        vec!["UID".into(), uid],
        vec!["GID principal".into(), gid],
        vec!["Grupo principal".into(), primary],
        vec!["Grupos".into(), groups],
    ];
    println!("=== Identidad actual ===");
    println!(
        "{}",
        crate::formatting::table(&["PROPIEDAD", "VALOR"], &rows)
    );
    Ok(())
}

fn sessions() -> Result<(), String> {
    println!("=== Sesiones abiertas ===");
    if command_exists("who") {
        let output = capture_output("who", &[])?;
        let rows = output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                vec![
                    fields.first().copied().unwrap_or_default().to_owned(),
                    fields.get(1).copied().unwrap_or_default().to_owned(),
                    format!(
                        "{} {}",
                        fields.get(2).copied().unwrap_or_default(),
                        fields.get(3).copied().unwrap_or_default()
                    )
                    .trim()
                    .to_owned(),
                    fields
                        .get(4)
                        .unwrap_or(&"")
                        .trim_matches(['(', ')'])
                        .to_owned(),
                ]
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            crate::formatting::table(&["USUARIO", "TTY", "INICIO", "ORIGEN"], &rows)
        );
    }
    if command_exists("loginctl") {
        let output = capture_output("loginctl", &["list-users", "--no-legend"])?;
        let rows = output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                vec![
                    fields.first().copied().unwrap_or_default().to_owned(),
                    fields.get(1).copied().unwrap_or_default().to_owned(),
                ]
            })
            .collect::<Vec<_>>();
        println!("\n=== Usuarios de logind ===");
        println!("{}", crate::formatting::table(&["UID", "USUARIO"], &rows));
    }
    Ok(())
}

fn inspect(user: &str) -> Result<(), String> {
    let user = valid_name(user)?;
    let passwd = capture_output("getent", &["passwd", user])?;
    let fields = passwd.splitn(7, ':').collect::<Vec<_>>();
    if fields.len() != 7 {
        return Err(format!("no se encontró la cuenta {user}"));
    }
    let rows = vec![
        vec!["Usuario".into(), fields[0].into()],
        vec!["UID".into(), fields[2].into()],
        vec!["GID".into(), fields[3].into()],
        vec!["Descripción".into(), fields[4].into()],
        vec!["Home".into(), fields[5].into()],
        vec!["Shell".into(), fields[6].into()],
    ];
    println!("=== Cuenta {user} ===");
    println!(
        "{}",
        crate::formatting::table(&["PROPIEDAD", "VALOR"], &rows)
    );
    user_groups(user)
}

fn create_user(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = target_after(args, first(args).unwrap_or("create"))?;
    let user = valid_name(user)?;
    let mut command = vec!["useradd".to_owned()];
    if !has_flag(args, "--no-create-home") {
        command.push("--create-home".into());
    }
    add_option(&mut command, args, "--uid", "--uid", "UID")?;
    add_path_option(&mut command, args, "--home", "--home", "home")?;
    add_path_option(&mut command, args, "--shell", "--shell", "shell")?;
    add_name_option(
        &mut command,
        args,
        "--primary-group",
        "--gid",
        "grupo principal",
    )?;
    add_option(&mut command, args, "--groups", "--groups", "grupos")?;
    add_option(&mut command, args, "--comment", "--comment", "descripción")?;
    if has_flag(args, "--system") {
        command.push("--system".into());
    }
    command.push(user.into());
    execute_user_command(ctx, user, "crear la cuenta", command)
}

fn modify_user(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, first(args).unwrap_or("modify"))?)?;
    let mut command = vec!["usermod".to_owned()];
    add_name_option(&mut command, args, "--login", "--login", "nuevo nombre")?;
    add_path_option(&mut command, args, "--home", "--home", "home")?;
    if has_flag(args, "--move-home") {
        command.push("--move-home".into());
    }
    add_path_option(&mut command, args, "--shell", "--shell", "shell")?;
    add_name_option(
        &mut command,
        args,
        "--primary-group",
        "--gid",
        "grupo principal",
    )?;
    add_name_option(&mut command, args, "--groups", "--groups", "grupos")?;
    if has_flag(args, "--append") {
        command.push("--append".into());
    }
    add_option(&mut command, args, "--comment", "--comment", "descripción")?;
    if has_flag(args, "--lock") {
        command.push("--lock".into());
    }
    if has_flag(args, "--unlock") {
        command.push("--unlock".into());
    }
    command.push(user.into());
    execute_user_command(ctx, user, "editar la cuenta", command)
}

fn delete_user(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, first(args).unwrap_or("delete"))?)?;
    let mut command = vec!["userdel".to_owned()];
    if has_flag(args, "--remove-home") {
        command.push("--remove".into());
    }
    if has_flag(args, "--force") {
        command.push("--force".into());
    }
    command.push(user.into());
    execute_user_command(ctx, user, "eliminar la cuenta", command)
}

fn expire_user(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, "expire")?)?;
    let mut command = vec!["chage".to_owned()];
    add_option(
        &mut command,
        args,
        "--date",
        "--expiredate",
        "fecha de expiración",
    )?;
    add_option(
        &mut command,
        args,
        "--inactive",
        "--inactive",
        "días inactivos",
    )?;
    add_option(&mut command, args, "--mindays", "--mindays", "días mínimos")?;
    add_option(&mut command, args, "--maxdays", "--maxdays", "días máximos")?;
    add_option(
        &mut command,
        args,
        "--warndays",
        "--warndays",
        "días de aviso",
    )?;
    if command.len() == 1 {
        return Err(
            "expire requiere al menos --date, --inactive, --mindays, --maxdays o --warndays".into(),
        );
    }
    command.push(user.into());
    execute_user_command(ctx, user, "cambiar la caducidad de la cuenta", command)
}

fn simple_user_mutation(
    ctx: &Context,
    args: &[String],
    description: &str,
    mut command: Vec<String>,
) -> Result<(), String> {
    let action = first(args).unwrap_or("user");
    let user = valid_name(target_after(args, action)?)?;
    command.push(user.into());
    execute_user_command(ctx, user, description, command)
}

fn change_password(ctx: &Context, args: &[String]) -> Result<(), String> {
    let user = valid_name(target_after(args, first(args).unwrap_or("password"))?)?;
    let preconfirmed = std::env::var_os("LTOOLS_ACCOUNT_GUI_PRECONFIRMED").is_some()
        && std::env::var("LTOOLS_FRONTEND").ok().as_deref() == Some("gui");
    if !ctx.dry_run
        && !preconfirmed
        && !ask(&format!("¿Quieres cambiar la contraseña de '{user}'?"))
    {
        record(
            ctx,
            "account-password",
            user,
            "cancelled",
            "chpasswd <stdin>",
        )?;
        return Ok(());
    }
    if ctx.dry_run {
        println!("=== Cambio de contraseña ===");
        println!("Se ejecutaría chpasswd leyendo la contraseña desde stdin para {user}.");
        record(ctx, "account-password", user, "planned", "chpasswd <stdin>")?;
        return Ok(());
    }
    if let Some(file) = option_value(args, "--password-file") {
        let file = valid_path(&file, "archivo de contraseña")?;
        let password = std::fs::read_to_string(&file)
            .map_err(|error| format!("no se pudo leer el archivo de contraseña: {error}"))?;
        let _ = std::fs::remove_file(&file);
        return finish_password(ctx, user, &password);
    }
    println!("Se abrirá el cambio interactivo de contraseña para {user}.");
    let ok =
        run_with_sudo("passwd", &[user.to_owned()], false).map_err(|error| error.to_string())?;
    record(
        ctx,
        "account-password",
        user,
        if ok { "executed" } else { "failed" },
        "passwd <interactive>",
    )?;
    if ok {
        Ok(())
    } else {
        Err(format!("passwd no pudo cambiar la contraseña de {user}"))
    }
}

fn finish_password(ctx: &Context, user: &str, password: &str) -> Result<(), String> {
    if password.contains('\n') || password.contains('\r') || password.is_empty() {
        return Err("la contraseña no puede estar vacía ni contener saltos de línea".into());
    }
    let input = format!("{user}:{password}\n");
    let ok = run_with_sudo_input("chpasswd", &[], input.as_bytes(), false)
        .map_err(|error| error.to_string())?;
    record(
        ctx,
        "account-password",
        user,
        if ok { "executed" } else { "failed" },
        "chpasswd <stdin>",
    )?;
    if ok {
        Ok(())
    } else {
        Err(format!("chpasswd no pudo cambiar la contraseña de {user}"))
    }
}

fn set_primary_group(ctx: &Context, args: &[String]) -> Result<(), String> {
    let (user, group) = user_group_args(args, "set-primary-group")?;
    valid_name(&user)?;
    valid_name(&group)?;
    let command = vec![
        "usermod".into(),
        "--gid".into(),
        group.clone(),
        user.clone(),
    ];
    execute_user_command(ctx, &user, "cambiar el grupo principal", command)
}

fn group_mutation(
    ctx: &Context,
    args: &[String],
    add: bool,
    membership: bool,
) -> Result<(), String> {
    if membership {
        let (user, group) = user_group_args(args, first(args).unwrap_or("group-add"))?;
        valid_name(&user)?;
        valid_name(&group)?;
        let command = if add {
            vec![
                "usermod".into(),
                "--append".into(),
                "--groups".into(),
                group.clone(),
                user.clone(),
            ]
        } else {
            vec![
                "gpasswd".into(),
                "--delete".into(),
                user.clone(),
                group.clone(),
            ]
        };
        return execute_user_command(
            ctx,
            &user,
            if add {
                "añadir el usuario al grupo"
            } else {
                "retirar el usuario del grupo"
            },
            command,
        );
    }
    let group = option_value(args, "--group")
        .or_else(|| positional_after(args, first(args).unwrap_or("group")).map(str::to_owned))
        .ok_or("falta el grupo")?;
    let group = valid_name(&group)?;
    let command = if add {
        vec!["groupadd".into(), group.into()]
    } else {
        vec!["groupdel".into(), group.into()]
    };
    execute_user_command(
        ctx,
        group,
        if add {
            "crear el grupo"
        } else {
            "eliminar el grupo"
        },
        command,
    )
}

/// Añade una cuenta a un grupo administrativo común de la distribución. Si no
/// se indica grupo, solo selecciona entre `sudo`, `wheel` o `admin` cuando ese
/// grupo existe realmente; nunca crea grupos ni presupone que exista una regla
/// sudoers para un grupo personalizado.
fn add_administrator_access(ctx: &Context, args: &[String]) -> Result<(), String> {
    let mut user = None;
    let mut group = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--user" => {
                user = Some(
                    args.get(index + 1)
                        .ok_or("--user requiere un usuario")?
                        .clone(),
                );
                index += 2;
            }
            "--group" => {
                group = Some(
                    args.get(index + 1)
                        .ok_or("--group requiere un grupo")?
                        .clone(),
                );
                index += 2;
            }
            "--yes" => index += 1,
            value if value.starts_with('-') => {
                return Err(format!("opción admin-add desconocida: {value}"));
            }
            value if user.is_none() => {
                user = Some(value.to_owned());
                index += 1;
            }
            value => return Err(format!("argumento inesperado: {value}")),
        }
    }

    let user = match user {
        Some(user) => user,
        None => capture_output("id", &["-un"])?,
    };
    let user = valid_name(&user)?.to_owned();
    let group = match group {
        Some(group) => {
            let group = valid_name(&group)?.to_owned();
            if !system_group_exists(&group) {
                return Err(format!("el grupo '{group}' no existe en esta máquina"));
            }
            group
        }
        None => detected_administrator_group().ok_or_else(|| {
            "no se detectó un grupo administrativo común (sudo, wheel o admin); indica --group tras comprobar la política sudoers".to_owned()
        })?,
    };

    println!("Se añadirá {user} al grupo {group}. La pertenencia administrativa puede otorgar control total; cierra sesión y vuelve a entrar para que se aplique. Verifica la política sudoers de la distribución.");
    execute_user_command(
        ctx,
        &format!("{user}:{group}"),
        "añadir la cuenta al grupo administrativo",
        vec![
            "usermod".into(),
            "--append".into(),
            "--groups".into(),
            group,
            user,
        ],
    )
}

fn list_administrator_groups() -> Result<(), String> {
    let user = capture_output("id", &["-un"])?;
    let memberships = capture_output("id", &["-Gn", &user])?;
    println!("Grupos administrativos convencionales (la regla sudoers depende de la máquina):");
    for group in ["sudo", "wheel", "admin"] {
        println!(
            "  {group}: {}{}",
            if system_group_exists(group) {
                "existe"
            } else {
                "no existe"
            },
            if memberships.split_whitespace().any(|member| member == group) {
                " · tu usuario pertenece"
            } else {
                ""
            }
        );
    }
    println!("Para añadir una cuenta: accounts admin-add [USUARIO] [--group GRUPO]. La acción pide confirmación y privilegios.");
    Ok(())
}

fn system_group_exists(group: &str) -> bool {
    if !command_exists("getent") {
        return false;
    }
    Command::new("getent")
        .args(["group", group])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8(output.stdout)
                .ok()
                .map(|line| line.split(':').next() == Some(group))
        })
        .unwrap_or(false)
}

fn detected_administrator_group() -> Option<String> {
    let existing = ["sudo", "wheel", "admin"]
        .into_iter()
        .filter(|group| system_group_exists(group))
        .collect::<Vec<_>>();
    preferred_administrator_group(&existing).map(str::to_owned)
}

fn preferred_administrator_group(existing: &[&str]) -> Option<&'static str> {
    ["sudo", "wheel", "admin"]
        .into_iter()
        .find(|candidate| existing.contains(candidate))
}

fn user_group_args(args: &[String], action: &str) -> Result<(String, String), String> {
    if let (Some(user), Some(group)) = (option_value(args, "--user"), option_value(args, "--group"))
    {
        return Ok((user, group));
    }
    let raw = positional_after(args, action)
        .ok_or_else(|| format!("{action} requiere USUARIO:GRUPO o --user USUARIO --group GRUPO"))?;
    let (user, group) = raw
        .split_once(':')
        .ok_or_else(|| format!("{action} usa el formato USUARIO:GRUPO"))?;
    Ok((user.to_owned(), group.to_owned()))
}

fn execute_user_command(
    ctx: &Context,
    target: &str,
    description: &str,
    command: Vec<String>,
) -> Result<(), String> {
    if command.is_empty() {
        return Err("orden de cuenta vacía".into());
    }
    let preconfirmed = std::env::var_os("LTOOLS_ACCOUNT_GUI_PRECONFIRMED").is_some()
        && std::env::var("LTOOLS_FRONTEND").ok().as_deref() == Some("gui");
    if !ctx.dry_run && !preconfirmed && !ask(&format!("¿Quieres {description} '{target}'?")) {
        record(
            ctx,
            "account-mutation",
            target,
            "cancelled",
            &redacted_command(&command),
        )?;
        println!("Operación cancelada; no se modificó ninguna cuenta.");
        return Ok(());
    }
    let program = command[0].clone();
    let args = command[1..].to_vec();
    let ok = run_with_sudo(&program, &args, ctx.dry_run).map_err(|error| error.to_string())?;
    let status = if ctx.dry_run {
        "planned"
    } else if ok {
        "executed"
    } else {
        "failed"
    };
    record(
        ctx,
        "account-mutation",
        target,
        status,
        &redacted_command(&command),
    )?;
    if ok {
        Ok(())
    } else {
        Err(format!(
            "{program} no pudo completar la operación sobre {target}"
        ))
    }
}

fn add_option(
    command: &mut Vec<String>,
    args: &[String],
    input: &str,
    output: &str,
    label: &str,
) -> Result<(), String> {
    if let Some(value) = option_value(args, input) {
        validate_token(&value, label)?;
        command.push(output.into());
        command.push(value);
    }
    Ok(())
}

fn add_name_option(
    command: &mut Vec<String>,
    args: &[String],
    input: &str,
    output: &str,
    _label: &str,
) -> Result<(), String> {
    if let Some(value) = option_value(args, input) {
        valid_name(&value)?;
        command.push(output.into());
        command.push(value);
    }
    Ok(())
}

fn add_path_option(
    command: &mut Vec<String>,
    args: &[String],
    input: &str,
    output: &str,
    label: &str,
) -> Result<(), String> {
    if let Some(value) = option_value(args, input) {
        command.push(output.into());
        command.push(valid_path(&value, label)?);
    }
    Ok(())
}

fn validate_token(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty() || value.contains('\0') || value.contains('\n') || value.contains('\r') {
        Err(format!("{label} no válido"))
    } else {
        Ok(())
    }
}

fn redacted_command(command: &[String]) -> String {
    command
        .iter()
        .map(|value| crate::common::shell_display(value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn record(
    ctx: &Context,
    operation: &str,
    target: &str,
    status: &str,
    data: &str,
) -> Result<(), String> {
    if let Some(plan) = &ctx.plan {
        plan.record(
            operation,
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

fn capture_output(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("no se pudo ejecutar {program}: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Usuarios, grupos y sesiones Linux ===");
        println!(" 1) Cuentas    2) Grupos    3) Mi identidad    4) Sesiones    5) Inspeccionar");
        println!(" 6) Crear      7) Editar    8) Contraseña    9) Bloquear    10) Desbloquear");
        println!("11) Eliminar   12) Caducidad 13) Crear grupo 14) Eliminar grupo");
        println!("15) Añadir miembro  16) Retirar miembro  17) Grupo principal");
        println!(
            "18) Añadir usuario actual a sudo/wheel  19) Ver grupos administrativos  q) Volver"
        );
        let choice =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        let result = match choice.trim() {
            "1" => list_users(&[]),
            "2" => list_groups(),
            "3" => identity(),
            "4" => sessions(),
            "5" => prompt(ctx, "Usuario: ", |_, value| inspect(value)),
            "6" => prompt(ctx, "Nuevo usuario: ", |c, value| {
                create_user(c, &["create".into(), value.into()])
            }),
            "7" => prompt(ctx, "Usuario: ", |c, value| {
                modify_user(c, &["modify".into(), value.into()])
            }),
            "8" => prompt(ctx, "Usuario: ", |c, value| {
                change_password(c, &["password".into(), value.into()])
            }),
            "9" => prompt(ctx, "Usuario: ", |c, value| {
                simple_user_mutation(
                    c,
                    &["lock".into(), value.into()],
                    "bloquear la cuenta",
                    vec!["usermod".into(), "--lock".into()],
                )
            }),
            "10" => prompt(ctx, "Usuario: ", |c, value| {
                simple_user_mutation(
                    c,
                    &["unlock".into(), value.into()],
                    "desbloquear la cuenta",
                    vec!["usermod".into(), "--unlock".into()],
                )
            }),
            "11" => prompt(ctx, "Usuario: ", |c, value| {
                delete_user(c, &["delete".into(), value.into()])
            }),
            "12" => prompt(ctx, "Usuario: ", |c, value| {
                expire_user(c, &["expire".into(), value.into()])
            }),
            "13" => prompt(ctx, "Grupo nuevo: ", |c, value| {
                group_mutation(c, &["group-create".into(), value.into()], true, false)
            }),
            "14" => prompt(ctx, "Grupo: ", |c, value| {
                group_mutation(c, &["group-delete".into(), value.into()], false, false)
            }),
            "15" => prompt(ctx, "Usuario:Grupo: ", |c, value| {
                group_mutation(c, &["group-add".into(), value.into()], true, true)
            }),
            "16" => prompt(ctx, "Usuario:Grupo: ", |c, value| {
                group_mutation(c, &["group-remove".into(), value.into()], false, true)
            }),
            "17" => prompt(ctx, "Usuario:Grupo: ", |c, value| {
                set_primary_group(c, &["set-primary-group".into(), value.into()])
            }),
            "18" => add_administrator_access(ctx, &["admin-add".into()]),
            "19" => list_administrator_groups(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_the_conventional_admin_group_without_inventing_one() {
        assert_eq!(
            preferred_administrator_group(&["wheel", "sudo"]),
            Some("sudo")
        );
        assert_eq!(
            preferred_administrator_group(&["admin", "wheel"]),
            Some("wheel")
        );
        assert_eq!(preferred_administrator_group(&["users"]), None);
    }

    #[test]
    fn linux_account_names_reject_option_injection_for_admin_membership() {
        assert!(valid_name("alice").is_ok());
        assert!(valid_name("--help").is_err());
        assert!(valid_name("alice:wheel").is_err());
    }

    #[test]
    fn names_reject_shell_fragments() {
        assert!(valid_name("user;rm").is_err());
        assert!(valid_name("usuario_valido").is_ok());
    }

    #[test]
    fn command_display_keeps_arguments_separate_and_readable() {
        let command = vec![
            "usermod".into(),
            "--shell".into(),
            "/bin/zsh".into(),
            "ana".into(),
        ];
        assert_eq!(redacted_command(&command), "usermod --shell /bin/zsh ana");
    }
}
