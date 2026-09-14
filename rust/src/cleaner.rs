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
    remove_root: bool,
}

pub fn help() -> &'static str {
    "clean --automatic [--preview] [--include-personal|--all-known] [--ask-each]"
}

/// Ejecuta el inventario y el flujo interactivo. La opción `--preview` solo
/// calcula tamaños; nunca pregunta ni modifica nada.
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    if let Some(index) = args
        .iter()
        .position(|arg| arg == "--_remove-system-candidate")
    {
        let raw = args
            .get(index + 1)
            .ok_or("falta la ruta para la operación interna de limpieza")?;
        return remove_system_candidate(ctx, raw, args.iter().any(|arg| arg == "--_remove-root"));
    }
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
            .fold(0_u64, |total, candidate| {
                total.saturating_add(candidate.bytes)
            });
        potential = potential.saturating_add(reclaimable);
        if kind == Kind::Applications {
            println!(
                "\n{}: se detectaron {} rutas ({}). No se borran carpetas de aplicaciones: usa `packages`/`software` para desinstalarlas correctamente.",
                kind.label(),
                group.len(),
                human_bytes(group.iter().fold(0_u64, |total, candidate| {
                    total.saturating_add(candidate.bytes)
                }))
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
        let removal = remove_contents(&candidate.path, candidate.remove_root);
        if let Err(error) = removal {
            if error.kind() != std::io::ErrorKind::PermissionDenied
                || !is_elevatable_system_candidate(&candidate.path, candidate.remove_root)
            {
                return Err(error.to_string());
            }
            if !ctx.elevate_by_default
                && !ask(&format!(
                    "{} necesita permisos administrativos. ¿Reintentar solo esta ruta como administrador?",
                    candidate.path.display()
                ))
            {
                return Err("reintento administrativo cancelado".into());
            }
            elevate_system_candidate(&candidate.path)?;
        }
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

fn remove_contents(path: &Path, remove_root: bool) -> std::io::Result<()> {
    remove_contents_inner(path, remove_root)
}

fn remove_contents_inner(path: &Path, remove_root: bool) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "no se siguen enlaces simbólicos",
        ));
    }
    if metadata.is_file() {
        return fs::remove_file(path);
    }

    ensure_no_mounts(path, crate::common::device(path))?;
    remove_contents_tree(path)?;
    if remove_root {
        fs::remove_dir(path)?;
    }
    Ok(())
}

fn ensure_no_mounts(path: &Path, device: Option<u64>) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let root = fs::canonicalize(path)?;
        let mountinfo = fs::read_to_string("/proc/self/mountinfo")?;
        for line in mountinfo.lines() {
            let Some(encoded) = line.split_whitespace().nth(4) else {
                continue;
            };
            let decoded = encoded
                .replace("\\040", " ")
                .replace("\\011", "\t")
                .replace("\\012", "\n")
                .replace("\\134", "\\");
            let mount = PathBuf::from(decoded);
            let mount = fs::canonicalize(&mount).unwrap_or(mount);
            if mount == root || mount.starts_with(&root) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("se omite el punto de montaje {}", mount.display()),
                ));
            }
        }
    }
    ensure_device_tree(path, device)
}

fn ensure_device_tree(path: &Path, device: Option<u64>) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let child = entry?.path();
        let metadata = fs::symlink_metadata(&child)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        if device.is_some() && crate::common::device(&child) != device {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "se omite el sistema de archivos montado en {}",
                    child.display()
                ),
            ));
        }
        ensure_device_tree(&child, device)?;
    }
    Ok(())
}

fn remove_contents_tree(path: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let child = entry?.path();
        let metadata = fs::symlink_metadata(&child)?;
        if !metadata.is_dir() {
            fs::remove_file(child)?;
        } else {
            remove_contents_tree(&child)?;
            fs::remove_dir(child)?;
        }
    }
    Ok(())
}

fn is_elevatable_system_candidate(path: &Path, remove_root: bool) -> bool {
    if remove_root || fs::symlink_metadata(path).is_err() {
        return false;
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return false;
    }
    let Ok(candidate) = fs::canonicalize(path) else {
        return false;
    };
    #[cfg(target_os = "linux")]
    {
        [
            "/var/cache/pacman/pkg",
            "/var/cache/apt/archives",
            "/var/cache/dnf",
            "/var/cache/zypp",
            "/var/cache/apk",
            "/var/cache/xbps",
        ]
        .iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .any(|root| root == candidate)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = candidate;
        false
    }
}

fn elevate_system_candidate(path: &Path) -> Result<(), String> {
    if !is_elevatable_system_candidate(path, false) {
        return Err("la ruta ya no coincide con una caché del sistema autorizada".into());
    }
    let executable = std::env::current_exe()
        .map_err(|error| format!("no se pudo localizar LTools para elevar: {error}"))?;
    let args = vec![
        "clean".to_owned(),
        "--_remove-system-candidate".to_owned(),
        path.display().to_string(),
        "--privileged-child".to_owned(),
    ];
    match crate::common::run_with_sudo(&executable.to_string_lossy(), &args, false) {
        Ok(true) => Ok(()),
        Ok(false) => Err("no se pudo completar la limpieza con elevación".into()),
        Err(error) => Err(format!("no se pudo solicitar elevación: {error}")),
    }
}

fn remove_system_candidate(ctx: &Context, raw: &str, remove_root: bool) -> Result<(), String> {
    if !ctx.privileged_child || !crate::platform::is_elevated() {
        return Err(
            "la limpieza privilegiada requiere una autorización administrativa real".into(),
        );
    }
    let path = PathBuf::from(raw);
    if !is_elevatable_system_candidate(&path, remove_root) {
        return Err(
            "ruta rechazada: solo se permiten cachés de paquetes del sistema conocidas".into(),
        );
    }
    remove_contents_inner(&path, false).map_err(|error| error.to_string())
}

fn discover(ctx: &Context, include_personal: bool) -> Vec<Candidate> {
    let home = &ctx.home;
    let mut result = Vec::new();
    #[cfg(windows)]
    {
        for (kind, path, description) in windows_safe_paths(
            std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
            std::env::var_os("TEMP").map(PathBuf::from),
            std::env::var_os("SystemRoot")
                .or_else(|| std::env::var_os("windir"))
                .map(PathBuf::from),
        ) {
            add_path(&mut result, kind, &path, description);
        }
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
        // /tmp y, especialmente, /var/tmp pueden contener datos activos o
        // temporales que deben sobrevivir a un reinicio. Presentar elementos
        // individuales permite revisarlos uno a uno; no ofrecer la raíz como
        // una única operación silenciosa de borrado.
        for root in ["/tmp", "/var/tmp"] {
            add_children(
                &mut result,
                Kind::Temporary,
                Path::new(root),
                "elemento temporal del sistema (revisar antes de borrar)",
            );
        }
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
    if result
        .iter()
        .any(|candidate| same_path(&candidate.path, path))
    {
        return;
    }
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
            remove_root: false,
        });
    }
}

/// Solo ofrece ubicaciones de caché/temporales concretas. Enumerar todos los
/// hijos de `%LOCALAPPDATA%` podía incluir perfiles completos de navegador,
/// configuración y datos de aplicaciones, y no es una política de limpieza
/// segura.
#[cfg(any(windows, test))]
fn windows_safe_paths(
    local_app_data: Option<PathBuf>,
    temp: Option<PathBuf>,
    system_root: Option<PathBuf>,
) -> Vec<(Kind, PathBuf, &'static str)> {
    let mut paths = Vec::new();
    if let Some(local) = local_app_data {
        for (relative, kind, description) in [
            ("Temp", Kind::Temporary, "temporales del usuario Windows"),
            ("D3DSCache", Kind::Cache, "caché Direct3D del usuario"),
            (
                "Microsoft/Windows/INetCache",
                Kind::Cache,
                "caché web de Windows del usuario",
            ),
        ] {
            push_unique_path(&mut paths, kind, local.join(relative), description);
        }
    }
    if let Some(temp) = temp {
        push_unique_path(
            &mut paths,
            Kind::Temporary,
            temp,
            "temporales indicados por Windows",
        );
    }
    let system_root = system_root.unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    push_unique_path(
        &mut paths,
        Kind::Temporary,
        system_root.join("Temp"),
        "temporales del sistema Windows",
    );
    // `$Recycle.Bin` contiene subdirectorios de todos los SID del volumen.
    // No se ofrece como una carpeta normal porque vaciarla así podría borrar
    // también la papelera de otros usuarios. La gestión por usuario requiere
    // la API nativa de Windows (no un borrado recursivo del árbol compartido).
    paths
}

#[cfg(any(windows, test))]
fn push_unique_path(
    paths: &mut Vec<(Kind, PathBuf, &'static str)>,
    kind: Kind,
    path: PathBuf,
    description: &'static str,
) {
    if paths
        .iter()
        .any(|(_, existing, _)| same_path(existing, &path))
    {
        return;
    }
    paths.push((kind, path, description));
}

fn same_path(left: &Path, right: &Path) -> bool {
    let left = fs::canonicalize(left).unwrap_or_else(|_| left.to_path_buf());
    let right = fs::canonicalize(right).unwrap_or_else(|_| right.to_path_buf());
    #[cfg(windows)]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
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
                remove_root: true,
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
        let total = group.iter().fold(0_u64, |total, candidate| {
            total.saturating_add(candidate.bytes)
        });
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
    use super::{is_elevatable_system_candidate, remove_contents, windows_safe_paths, Kind};
    use std::path::PathBuf;

    #[test]
    fn personal_categories_are_not_automatic() {
        assert!(!Kind::Personal.automatic());
        assert!(!Kind::Applications.automatic());
        assert!(Kind::Cache.automatic());
    }

    #[test]
    fn windows_cleanup_only_targets_known_cache_and_temp_locations() {
        let local = PathBuf::from("/profiles/alice/AppData/Local");
        let paths = windows_safe_paths(
            Some(local.clone()),
            Some(local.join("Temp")),
            Some(PathBuf::from("/Windows")),
        );

        assert!(!paths.iter().any(|(_, path, _)| path == &local));
        assert!(!paths.iter().any(|(_, path, _)| {
            path.to_string_lossy().contains("Edge")
                || path.to_string_lossy().contains("Chrome")
                || path.to_string_lossy().contains("Packages")
        }));
        assert_eq!(
            paths
                .iter()
                .filter(|(_, path, _)| path == &local.join("Temp"))
                .count(),
            1,
            "TEMP y LOCALAPPDATA\\Temp no deben duplicar la selección ni el tamaño"
        );
        assert!(!paths
            .iter()
            .any(|(_, path, _)| path.ends_with("$Recycle.Bin")));
    }

    fn temporary_directory(label: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ltools-cleaner-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn cleanup_preserves_or_removes_only_the_selected_root() {
        let preserved = temporary_directory("preserved");
        std::fs::create_dir(preserved.join("nested")).unwrap();
        std::fs::write(preserved.join("nested/data"), "temporary").unwrap();
        remove_contents(&preserved, false).unwrap();
        assert!(preserved.is_dir());
        assert!(std::fs::read_dir(&preserved).unwrap().next().is_none());

        let removed = temporary_directory("removed");
        std::fs::write(removed.join("data"), "temporary").unwrap();
        remove_contents(&removed, true).unwrap();
        assert!(!removed.exists());
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_unlinks_symlinks_without_following_their_targets() {
        use std::os::unix::fs::symlink;
        let root = temporary_directory("symlink");
        let outside = temporary_directory("outside");
        std::fs::write(outside.join("keep"), "keep me").unwrap();
        symlink(&outside, root.join("link")).unwrap();

        remove_contents(&root, false).unwrap();
        assert!(outside.join("keep").exists());
        assert!(std::fs::read_dir(&root).unwrap().next().is_none());
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn elevated_cleanup_rejects_non_allowlisted_and_root_removal_paths() {
        assert!(!is_elevatable_system_candidate(
            std::path::Path::new("/etc"),
            false
        ));
        assert!(!is_elevatable_system_candidate(
            std::path::Path::new("/var/cache/apt/archives"),
            true
        ));
    }
}
