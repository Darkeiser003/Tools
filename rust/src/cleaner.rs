//! Limpieza guiada de espacio: inventaría rutas conocidas y separa cachés
//! regenerables de datos personales y aplicaciones.

use crate::common::{ask, directory_size, human_bytes, move_to_trash, Context};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Cache,
    Temporary,
    PackageCache,
    RecycleBin,
    Personal,
    Applications,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Self::Cache => "cachés regenerables",
            Self::Temporary => "temporales conocidos",
            Self::PackageCache => "cachés de gestores de paquetes",
            Self::RecycleBin => "papelera de reciclaje",
            Self::Personal => "datos personales (descargas, documentos y similares)",
            Self::Applications => "aplicaciones instaladas",
        }
    }

    fn automatic(self) -> bool {
        matches!(self, Self::Cache | Self::Temporary | Self::PackageCache)
    }
}

#[derive(Clone, Debug)]
struct Candidate {
    kind: Kind,
    path: PathBuf,
    bytes: u64,
    description: String,
}

pub fn help() -> &'static str {
    "clean --automatic [--preview] [--include-personal|--all-known] [--ask-each]"
}

/// Ejecuta el inventario y el flujo interactivo. La opción `--preview` solo
/// calcula tamaños; nunca pregunta ni modifica nada.
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let preview = args.iter().any(|arg| arg == "--preview");
    let include_personal = args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--include-personal" | "--all-known"));
    let ask_each = args.iter().any(|arg| arg == "--ask-each");
    let candidates = discover(ctx, include_personal);
    report(&candidates, include_personal);
    if candidates.is_empty() {
        println!("No se detectaron rutas conocidas con datos para revisar.");
        return Ok(());
    }
    if preview || ctx.dry_run {
        println!("Vista previa: no se ha modificado ningún archivo.");
        return Ok(());
    }

    let mut potential = 0_u64;
    let mut selected = 0_u64;
    let mut liberated = 0_u64;
    for kind in [
        Kind::Cache,
        Kind::Temporary,
        Kind::PackageCache,
        Kind::RecycleBin,
        Kind::Personal,
        Kind::Applications,
    ] {
        let group = candidates
            .iter()
            .filter(|candidate| candidate.kind == kind)
            .collect::<Vec<_>>();
        if group.is_empty() {
            continue;
        }
        let reclaimable = group
            .iter()
            .filter(|candidate| candidate.kind != Kind::Applications)
            .map(|candidate| candidate.bytes)
            .sum::<u64>();
        potential = potential.saturating_add(reclaimable);
        if kind == Kind::Applications {
            println!(
                "\n{}: se detectaron {} rutas ({}). No se borran carpetas de aplicaciones: usa `packages`/`software` para desinstalarlas correctamente.",
                kind.label(),
                group.len(),
                human_bytes(group.iter().map(|candidate| candidate.bytes).sum())
            );
            continue;
        }
        if !ask(&format!(
            "¿Revisar y limpiar {} (hasta {})?",
            kind.label(),
            human_bytes(reclaimable)
        )) {
            continue;
        }
        let all = !ask_each
            && ask("¿Borrar todos los elementos de esta categoría? [No = preguntar uno a uno]");
        for candidate in group {
            if !all
                && !ask(&format!(
                    "¿Borrar {} — {}?",
                    candidate.path.display(),
                    human_bytes(candidate.bytes)
                ))
            {
                continue;
            }
            selected = selected.saturating_add(candidate.bytes);
            match remove_candidate(ctx, candidate) {
                Ok(Removal::Freed(bytes)) => {
                    liberated = liberated.saturating_add(bytes);
                    println!(
                        "  Liberado: {} ({})",
                        candidate.path.display(),
                        human_bytes(bytes)
                    );
                }
                Ok(Removal::Trashed(bytes)) => {
                    liberated = liberated.saturating_add(bytes);
                    println!(
                        "  Enviado a la papelera: {} ({})",
                        candidate.path.display(),
                        human_bytes(bytes)
                    );
                }
                Err(error) => {
                    eprintln!("  No se pudo limpiar {}: {error}", candidate.path.display())
                }
            }
        }
    }
    println!(
        "\nResumen: potencial detectado {}, seleccionado {}, liberado/separado {}.",
        human_bytes(potential),
        human_bytes(selected),
        human_bytes(liberated)
    );
    println!("La papelera puede seguir ocupando espacio físico hasta vaciarse explícitamente.");
    Ok(())
}

enum Removal {
    Freed(u64),
    Trashed(u64),
}

fn remove_candidate(ctx: &Context, candidate: &Candidate) -> Result<Removal, String> {
    if crate::platform::critical_path(&candidate.path) {
        return Err("ruta crítica protegida".into());
    }
    let bytes = candidate.bytes;
    if candidate.kind == Kind::Personal {
        if move_to_trash(&candidate.path, false).map_err(|error| error.to_string())? {
            record(ctx, "automatic-personal-trash", &candidate.path, "papelera")?;
            return Ok(Removal::Trashed(bytes));
        }
        return Err("la papelera nativa no aceptó la ruta".into());
    }
    if candidate.kind == Kind::RecycleBin || candidate.kind.automatic() {
        remove_contents(&candidate.path)?;
        record(
            ctx,
            "automatic-clean",
            &candidate.path,
            candidate.kind.label(),
        )?;
        return Ok(Removal::Freed(bytes));
    }
    Err("la categoría no tiene borrado automático".into())
}

fn record(ctx: &Context, operation: &str, path: &Path, note: &str) -> Result<(), String> {
    if let Some(plan) = &ctx.plan {
        plan.record(operation, path, "executed", false, note, "")
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn remove_contents(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("no se siguen enlaces simbólicos".into());
    }
    if metadata.is_file() {
        return fs::remove_file(path).map_err(|error| error.to_string());
    }
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let child = entry.map_err(|error| error.to_string())?.path();
        let child_meta = fs::symlink_metadata(&child).map_err(|error| error.to_string())?;
        if child_meta.file_type().is_symlink() || child_meta.is_file() {
            fs::remove_file(child).map_err(|error| error.to_string())?;
        } else if child_meta.is_dir() {
            fs::remove_dir_all(child).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn discover(ctx: &Context, include_personal: bool) -> Vec<Candidate> {
    let home = &ctx.home;
    let mut result = Vec::new();
    #[cfg(windows)]
    {
        for raw in [
            std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
            std::env::var_os("TEMP").map(PathBuf::from),
        ]
        .into_iter()
        .flatten()
        {
            add_children(&mut result, Kind::Cache, &raw, "caché/temporal Windows");
        }
        add_path(
            &mut result,
            Kind::Temporary,
            Path::new(r"C:\Windows\Temp"),
            "temporales del sistema",
        );
        add_path(
            &mut result,
            Kind::RecycleBin,
            Path::new(r"C:\$Recycle.Bin"),
            "papelera Windows",
        );
        for root in [
            std::env::var_os("ProgramFiles").map(PathBuf::from),
            std::env::var_os("ProgramFiles(x86)").map(PathBuf::from),
            std::env::var_os("ProgramData").map(PathBuf::from),
        ]
        .into_iter()
        .flatten()
        {
            add_path(
                &mut result,
                Kind::Applications,
                &root,
                "aplicaciones/datos compartidos Windows",
            );
        }
    }
    #[cfg(not(windows))]
    {
        add_children(
            &mut result,
            Kind::Cache,
            &home.join(".cache"),
            "caché de usuario",
        );
        for root in [
            "/var/cache/pacman/pkg",
            "/var/cache/apt/archives",
            "/var/cache/dnf",
            "/var/cache/zypp",
            "/var/cache/apk",
            "/var/cache/xbps",
        ] {
            add_path(
                &mut result,
                Kind::PackageCache,
                Path::new(root),
                "caché de paquetes",
            );
        }
        add_path(
            &mut result,
            Kind::Temporary,
            Path::new("/var/tmp"),
            "temporales persistentes del sistema",
        );
        add_path(
            &mut result,
            Kind::RecycleBin,
            &home.join(".local/share/Trash/files"),
            "papelera de usuario",
        );
        for root in [
            PathBuf::from("/usr"),
            PathBuf::from("/opt"),
            PathBuf::from("/var/lib/flatpak"),
            home.join(".local/share/flatpak"),
        ] {
            add_path(
                &mut result,
                Kind::Applications,
                &root,
                "aplicaciones instaladas",
            );
        }
    }
    if include_personal {
        for name in [
            "Downloads",
            "Documents",
            "Desktop",
            "Pictures",
            "Videos",
            "Music",
        ] {
            add_children(
                &mut result,
                Kind::Personal,
                &home.join(name),
                "datos personales",
            );
        }
    }
    result.sort_by_key(|candidate| std::cmp::Reverse(candidate.bytes));
    result
}

fn add_path(result: &mut Vec<Candidate>, kind: Kind, path: &Path, description: &str) {
    if !path.exists() {
        return;
    }
    let bytes = directory_size(path, None);
    if bytes > 0 {
        result.push(Candidate {
            kind,
            path: path.to_path_buf(),
            bytes,
            description: description.to_owned(),
        });
    }
}

fn add_children(result: &mut Vec<Candidate>, kind: Kind, path: &Path, description: &str) {
    if !path.is_dir() {
        return;
    }
    let mut found = false;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let child = entry.path();
            let bytes = directory_size(&child, None);
            if bytes == 0 {
                continue;
            }
            found = true;
            result.push(Candidate {
                kind,
                path: child,
                bytes,
                description: description.to_owned(),
            });
        }
    }
    if !found {
        add_path(result, kind, path, description);
    }
}

fn report(candidates: &[Candidate], include_personal: bool) {
    println!("=== Limpieza automática guiada ===");
    println!("Modo: rutas conocidas; no se tocan datos personales por defecto.");
    println!(
        "Datos personales incluidos en el análisis: {}",
        if include_personal { "sí" } else { "no" }
    );
    for kind in [
        Kind::Cache,
        Kind::Temporary,
        Kind::PackageCache,
        Kind::RecycleBin,
        Kind::Personal,
        Kind::Applications,
    ] {
        let group = candidates
            .iter()
            .filter(|candidate| candidate.kind == kind)
            .collect::<Vec<_>>();
        if group.is_empty() {
            continue;
        }
        let total = group.iter().map(|candidate| candidate.bytes).sum::<u64>();
        println!(
            "\n{} — {} en {} rutas",
            kind.label(),
            human_bytes(total),
            group.len()
        );
        for candidate in group.iter().take(30) {
            println!(
                "  {} · {} · {}",
                human_bytes(candidate.bytes),
                candidate.path.display(),
                candidate.description
            );
        }
        if group.len() > 30 {
            println!("  … {} rutas adicionales no mostradas", group.len() - 30);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Kind;

    #[test]
    fn personal_categories_are_not_automatic() {
        assert!(!Kind::Personal.automatic());
        assert!(!Kind::Applications.automatic());
        assert!(Kind::Cache.automatic());
    }
}
