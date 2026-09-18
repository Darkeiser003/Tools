//! Instantáneas y puntos de restauración nativos.
//!
//! Este módulo no inventa un formato de snapshot propio. Descubre y delega en
//! el backend del sistema (Timeshift, Snapper, Btrfs, ZFS o VSS/WBAdmin),
//! manteniendo los argumentos separados y una frontera explícita de plan,
//! confirmación y elevación.

use crate::cli_args::option_value;
use crate::common::{command_exists, command_output_detailed, run_with_sudo, Context};
use std::path::Path;

pub fn help() -> &'static str {
    "snapshots [status|list|create|delete|restore] [--backend timeshift|snapper|btrfs|zfs|vss] [--name NOMBRE] [--path RUTA] [--volume VOLUMEN] [--target ID] [--config CONFIG] [--yes] [--dry-run]"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    Timeshift,
    Snapper,
    Btrfs,
    Zfs,
    Vss,
}

impl Backend {
    fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "timeshift" => Ok(Self::Timeshift),
            "snapper" => Ok(Self::Snapper),
            "btrfs" => Ok(Self::Btrfs),
            "zfs" => Ok(Self::Zfs),
            "vss" | "shadow-copy" | "shadowcopy" | "wbadmin" => Ok(Self::Vss),
            _ => Err(format!(
                "backend desconocido: {value}; usa timeshift, snapper, btrfs, zfs o vss"
            )),
        }
    }

    #[cfg(not(windows))]
    fn program(self) -> &'static str {
        match self {
            Self::Timeshift => "timeshift",
            Self::Snapper => "snapper",
            Self::Btrfs => "btrfs",
            Self::Zfs => "zfs",
            Self::Vss => "vssadmin",
        }
    }

    #[cfg(windows)]
    fn program(self) -> &'static str {
        match self {
            Self::Vss => "vssadmin.exe",
            Self::Timeshift | Self::Snapper | Self::Btrfs | Self::Zfs => "vssadmin.exe",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Operation {
    backend: Backend,
    action: &'static str,
    program: String,
    args: Vec<String>,
    target: String,
    destructive: bool,
}

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action: &'static str = match crate::cli_args::positionals(args)
        .first()
        .copied()
        .unwrap_or("status")
    {
        "help" => "help",
        "status" => "status",
        "list" | "inventory" => "list",
        "create" => "create",
        "delete" => "delete",
        "restore" => "restore",
        "menu" => "menu",
        other => {
            return Err(format!(
            "acción de snapshots desconocida: {other}; usa status, list, create, delete o restore"
        ))
        }
    };
    match action {
        "help" => {
            println!("{}", help());
            Ok(())
        }
        "status" => status(),
        "list" => list(args),
        "create" | "delete" | "restore" => mutate(ctx, args, action),
        "menu" => menu(),
        _ => Err(format!(
            "acción de snapshots desconocida: {action}; usa status, list, create, delete o restore"
        )),
    }
}

fn menu() -> Result<(), String> {
    println!("INSTANTÁNEAS Y PUNTOS DE RESTAURACIÓN");
    println!("  status   detecta backends y capacidades sin modificar nada");
    println!("  list     enumera instantáneas existentes");
    println!("  create   crea una instantánea con backend y objetivo explícitos");
    println!("  delete   elimina una instantánea con objetivo explícito");
    println!("  restore  restaura solo backends que lo declaran compatible");
    println!();
    println!("Los cambios requieren --yes y admiten --dry-run/--plan.");
    println!(
        "Una instantánea no sustituye una copia externa ni garantiza recuperación del sistema."
    );
    Ok(())
}

fn status() -> Result<(), String> {
    println!("=== Backends de instantáneas disponibles ===");
    #[cfg(not(windows))]
    {
        for (name, command, description) in [
            (
                "timeshift",
                "timeshift",
                "puntos de restauración del sistema",
            ),
            ("snapper", "snapper", "configuraciones y snapshots Btrfs"),
            ("btrfs", "btrfs", "subvolúmenes Btrfs"),
            ("zfs", "zfs", "datasets y snapshots ZFS"),
        ] {
            println!(
                "{name}: {} — {description}",
                if command_exists(command) {
                    "disponible"
                } else {
                    "no detectado"
                }
            );
        }
        println!("Sistema: Linux; VSS/WBAdmin no aplican.");
    }
    #[cfg(windows)]
    {
        for (name, command, description) in [
            ("vss", "vssadmin.exe", "Volume Shadow Copy Service"),
            ("wbadmin", "wbadmin.exe", "copias de seguridad de Windows"),
            (
                "powershell",
                "powershell.exe",
                "System Restore y consultas CIM",
            ),
        ] {
            println!(
                "{name}: {} — {description}",
                if command_exists(command) {
                    "disponible"
                } else {
                    "no detectado"
                }
            );
        }
        println!("Sistema: Windows; Timeshift/Snapper/Btrfs/ZFS no aplican al ejecutable nativo.");
    }
    Ok(())
}

fn list(args: &[String]) -> Result<(), String> {
    #[cfg(windows)]
    {
        if let Some(value) = option_value(args, "--backend") {
            if Backend::parse(value)? != Backend::Vss {
                return Err("en Windows nativo solo está disponible el backend vss; Timeshift, Snapper, Btrfs y ZFS no aplican".into());
            }
        }
        let mut found = false;
        for (program, native_args, label) in [
            (
                "vssadmin.exe",
                vec!["list".to_owned(), "shadows".to_owned()],
                "VSS",
            ),
            (
                "wbadmin.exe",
                vec!["get".to_owned(), "versions".to_owned()],
                "WBAdmin",
            ),
        ] {
            if !command_exists(program) {
                continue;
            }
            found = true;
            println!("=== {label} ===");
            print_query(program, &native_args);
        }
        if !found {
            println!("No hay un backend de instantáneas disponible en este equipo.");
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let requested = option_value(args, "--backend")
            .map(Backend::parse)
            .transpose()?;
        let candidates = [
            (Backend::Timeshift, vec!["--list".to_owned()]),
            (Backend::Snapper, vec!["list".to_owned()]),
            (
                Backend::Btrfs,
                vec!["subvolume".to_owned(), "list".to_owned(), "/".to_owned()],
            ),
            (
                Backend::Zfs,
                vec![
                    "list".to_owned(),
                    "-t".to_owned(),
                    "snapshot".to_owned(),
                    "-o".to_owned(),
                    "name,creation,used,refer".to_owned(),
                ],
            ),
        ];
        let mut found = false;
        for (backend, native_args) in candidates {
            if requested.is_some_and(|selected| selected != backend)
                || !command_exists(backend.program())
            {
                continue;
            }
            found = true;
            println!("=== {} ===", backend_name(backend));
            print_query(backend.program(), &native_args);
        }
        if !found {
            println!("No hay un backend de instantáneas disponible o instalado.");
        }
        Ok(())
    }
}

fn print_query(program: &str, args: &[String]) {
    let borrowed = args.iter().map(String::as_str).collect::<Vec<_>>();
    match command_output_detailed(program, &borrowed) {
        Ok(output) if output.success() => {
            let text = output.stdout.trim();
            if text.is_empty() {
                println!("(sin instantáneas registradas)");
            } else {
                println!("{text}");
            }
        }
        Ok(output) => {
            let detail = if output.stderr.trim().is_empty() {
                output.stdout.trim()
            } else {
                output.stderr.trim()
            };
            println!(
                "No se pudo consultar el backend (código {:?}): {}",
                output.status_code, detail
            );
        }
        Err(error) => println!("No se pudo ejecutar la consulta: {error}"),
    }
}

fn mutate(ctx: &Context, args: &[String], action: &'static str) -> Result<(), String> {
    let backend = option_value(args, "--backend")
        .ok_or_else(|| {
            "la acción necesita --backend para evitar elegir un backend equivocado".to_owned()
        })
        .and_then(Backend::parse)?;
    let operation = build_operation(action, backend, args)?;
    println!("=== Snapshot {} ({}) ===", action, backend_name(backend));
    println!("Objetivo: {}", operation.target);
    println!(
        "Comando: {} {}",
        operation.program,
        operation.args.join(" ")
    );
    if operation.destructive {
        println!("ADVERTENCIA: esta operación puede eliminar o revertir datos.");
    }
    if !args.iter().any(|value| value == "--yes") && !ctx.dry_run {
        return Err(
            "la operación modificadora requiere --yes; usa --dry-run para revisar el plan".into(),
        );
    }
    if let Some(plan) = &ctx.plan {
        plan.record(
            &format!("snapshot-{action}"),
            Path::new(&operation.target),
            if ctx.dry_run { "planned" } else { "requested" },
            false,
            &operation.program,
            &operation.args.join(" "),
        )
        .map_err(|error| format!("no se pudo registrar el plan: {error}"))?;
    }
    let ok = run_with_sudo(&operation.program, &operation.args, ctx.dry_run)
        .map_err(|error| format!("no se pudo ejecutar el backend: {error}"))?;
    if !ok {
        return Err(format!(
            "{} terminó con error; revisa permisos y el backend",
            operation.program
        ));
    }
    if ctx.dry_run {
        println!("Simulación completada: no se modificó el sistema.");
    }
    Ok(())
}

fn build_operation(
    action: &'static str,
    backend: Backend,
    args: &[String],
) -> Result<Operation, String> {
    #[cfg(windows)]
    if backend != Backend::Vss {
        return Err(
            "en Windows nativo solo está disponible el backend vss; no se ejecutan comandos POSIX"
                .into(),
        );
    }
    #[cfg(not(windows))]
    if backend == Backend::Vss {
        return Err("vss solo está disponible en Windows nativo; no se usa bajo Linux/Wine".into());
    }
    let name = option_value(args, "--name");
    let path = option_value(args, "--path");
    let volume = option_value(args, "--volume");
    let target = option_value(args, "--target");
    let config = option_value(args, "--config")
        .map(str::to_owned)
        .unwrap_or_else(|| "root".to_owned());
    if config.is_empty() || !valid_identifier(&config) {
        return Err("--config solo admite un identificador simple sin separadores".into());
    }
    match backend {
        Backend::Timeshift => {
            let snapshot = match action {
                "create" => {
                    name.ok_or_else(|| "Timeshift create necesita --name DESCRIPCIÓN".to_owned())?
                }
                _ => target
                    .ok_or_else(|| "Timeshift delete/restore necesita --target ID".to_owned())?,
            }
            .to_owned();
            ensure_simple(&snapshot, "--target/--name")?;
            let mut native = match action {
                "create" => vec![
                    "--create".into(),
                    "--comments".into(),
                    snapshot.clone(),
                    "--scripted".into(),
                ],
                "delete" => vec![
                    "--delete".into(),
                    "--snapshot".into(),
                    snapshot.clone(),
                    "--scripted".into(),
                ],
                "restore" => vec![
                    "--restore".into(),
                    "--snapshot".into(),
                    snapshot.clone(),
                    "--scripted".into(),
                ],
                _ => return Err("acción no válida".into()),
            };
            if let Some(value) = volume.or(path).map(str::to_owned) {
                validate_path_like(&value, "--volume/--path")?;
                native.extend(["--target-device".into(), value]);
            }
            Ok(Operation {
                backend,
                action,
                program: backend.program().into(),
                args: native,
                target: snapshot,
                destructive: action != "create",
            })
        }
        Backend::Snapper => {
            let snapshot = match action {
                "create" => {
                    name.ok_or_else(|| "Snapper create necesita --name DESCRIPCIÓN".to_owned())?
                }
                _ => target
                    .ok_or_else(|| "Snapper delete/restore necesita --target ID".to_owned())?,
            }
            .to_owned();
            ensure_simple(&snapshot, "--target/--name")?;
            let native = match action {
                "create" => vec![
                    "-c".into(),
                    config,
                    "create".into(),
                    "--description".into(),
                    snapshot.clone(),
                ],
                "delete" => vec!["-c".into(), config, "delete".into(), snapshot.clone()],
                "restore" => vec!["-c".into(), config, "rollback".into(), snapshot.clone()],
                _ => return Err("acción no válida".into()),
            };
            Ok(Operation {
                backend,
                action,
                program: backend.program().into(),
                args: native,
                target: snapshot,
                destructive: action != "create",
            })
        }
        Backend::Btrfs => {
            let source = path
                .map(str::to_owned)
                .ok_or_else(|| "Btrfs necesita --path RUTA_ORIGEN".to_owned())?;
            validate_path_like(&source, "--path")?;
            let destination = target
                .ok_or_else(|| "Btrfs necesita --target RUTA_DESTINO".to_owned())?
                .to_owned();
            validate_path_like(&destination, "--target/--name")?;
            let native = match action {
                "create" => vec!["subvolume".into(), "snapshot".into(), source, destination.clone()],
                "delete" => vec!["subvolume".into(), "delete".into(), destination.clone()],
                "restore" => return Err("Btrfs no restaura una instantánea automáticamente: usa una ruta de destino explícita y copia/verifica los datos".into()),
                _ => return Err("acción no válida".into()),
            };
            Ok(Operation {
                backend,
                action,
                program: backend.program().into(),
                args: native,
                target: destination,
                destructive: action == "delete",
            })
        }
        Backend::Zfs => {
            let dataset = volume
                .or(path)
                .map(str::to_owned)
                .ok_or_else(|| "ZFS necesita --volume DATASET".to_owned())?;
            let snapshot = match action {
                "create" => name.ok_or_else(|| "ZFS create necesita --name SNAPSHOT".to_owned())?,
                _ => target.ok_or_else(|| {
                    "ZFS delete/restore necesita --target DATASET@SNAPSHOT".to_owned()
                })?,
            }
            .to_owned();
            ensure_zfs_name(&dataset, "--volume")?;
            let full = if snapshot.contains('@') {
                snapshot
            } else {
                format!("{dataset}@{snapshot}")
            };
            ensure_zfs_name(&full, "--target/--name")?;
            let native = match action {
                "create" => vec!["snapshot".into(), full.clone()],
                "delete" => vec!["destroy".into(), full.clone()],
                "restore" => vec!["rollback".into(), "-r".into(), full.clone()],
                _ => return Err("acción no válida".into()),
            };
            Ok(Operation {
                backend,
                action,
                program: backend.program().into(),
                args: native,
                target: full,
                destructive: action != "create",
            })
        }
        Backend::Vss => {
            #[cfg(not(windows))]
            return Err(
                "vss solo está disponible en Windows nativo; no se usa bajo Linux/Wine".into(),
            );
            #[cfg(windows)]
            {
                let volume = volume.or(path).ok_or("VSS necesita --volume C:\\")?;
                validate_windows_volume(volume)?;
                let id = target.or(name);
                let native = match action {
                    "create" => vec!["create".into(), "shadow".into(), format!("/for={volume}")],
                    "delete" => vec!["delete".into(), "shadows".into(), format!("/shadow={}", id.ok_or("VSS delete necesita --target ID")?)],
                    "restore" => return Err("VSS no ofrece restauración arbitraria con vssadmin; monta/copia desde la sombra o usa Windows System Restore/WBAdmin".into()),
                    _ => return Err("acción no válida".into()),
                };
                Ok(Operation {
                    backend,
                    action,
                    program: backend.program().into(),
                    args: native,
                    target: id.unwrap_or(volume).to_owned(),
                    destructive: action == "delete",
                })
            }
        }
    }
}

fn backend_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Timeshift => "Timeshift",
        Backend::Snapper => "Snapper",
        Backend::Btrfs => "Btrfs",
        Backend::Zfs => "ZFS",
        Backend::Vss => "VSS",
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-@:+".contains(c))
}

fn ensure_simple(value: &str, option: &str) -> Result<(), String> {
    if valid_identifier(value) && !value.contains('/') && !value.contains('\\') {
        Ok(())
    } else {
        Err(format!(
            "{option} no admite separadores ni caracteres inseguros"
        ))
    }
}

fn validate_path_like(value: &str, option: &str) -> Result<(), String> {
    if value.is_empty() || value.starts_with('-') || value.contains('\0') {
        return Err(format!("{option} no es una ruta válida"));
    }
    Ok(())
}

fn ensure_zfs_name(value: &str, option: &str) -> Result<(), String> {
    if valid_identifier(value)
        || value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-/@:+".contains(c))
    {
        Ok(())
    } else {
        Err(format!("{option} no es un nombre ZFS válido"))
    }
}

#[cfg(windows)]
fn validate_windows_volume(value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    if bytes.len() != 2 || !bytes[0].is_ascii_alphabetic() || bytes[1] != b':' {
        return Err("--volume debe ser una letra de unidad como C:".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn backend_parser_is_explicit_and_platform_neutral() {
        assert_eq!(Backend::parse("shadow-copy"), Ok(Backend::Vss));
        assert!(Backend::parse("unknown").is_err());
    }

    #[test]
    fn btrfs_operation_keeps_paths_as_separate_arguments() {
        let op = build_operation(
            "create",
            Backend::Btrfs,
            &args(&["--path", "/source dir", "--target", "/snap dir"]),
        )
        .unwrap();
        assert_eq!(
            op.args,
            vec!["subvolume", "snapshot", "/source dir", "/snap dir"]
        );
    }

    #[test]
    fn dangerous_operations_require_explicit_targets() {
        assert!(
            build_operation("delete", Backend::Zfs, &args(&["--volume", "pool/data"])).is_err()
        );
        assert!(build_operation("delete", Backend::Vss, &args(&["--volume", "C:"])).is_err());
    }

    #[test]
    fn restore_is_not_advertised_when_backend_cannot_restore_safely() {
        assert!(build_operation(
            "restore",
            Backend::Btrfs,
            &args(&["--path", "/data", "--target", "/snap"])
        )
        .is_err());
        #[cfg(not(windows))]
        assert!(build_operation("create", Backend::Vss, &args(&["--volume", "C:"])).is_err());
    }

    #[test]
    fn paths_reject_option_injection() {
        assert!(validate_path_like("--help", "--path").is_err());
        assert!(ensure_simple("bad/name", "--name").is_err());
    }
}
