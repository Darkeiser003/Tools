//! Mapa navegable de discos, volúmenes y rutas.
//!
//! El mapa es deliberadamente independiente de `lsblk`, PowerShell o una
//! herramienta de terceros: inspecciona las rutas que el proceso puede leer,
//! conserva los errores de permisos y calcula el tamaño acumulado de cada
//! carpeta. Las operaciones de gestión se mantienen separadas del escaneo y
//! pasan por confirmación, dry-run y los bloqueos de rutas críticas de LTools.

use crate::common::{human_bytes, move_to_trash, Context};
use std::cmp::Reverse;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(target_os = "linux")]
use std::process::Stdio;
#[cfg(target_os = "linux")]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_os = "linux")]
use std::{io, os::unix::fs::DirBuilderExt};

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) kind: &'static str,
    pub(crate) size: u64,
    pub(crate) filesystem_total: Option<u64>,
    pub(crate) filesystem_used: Option<u64>,
    pub(crate) filesystem_free: Option<u64>,
    pub(crate) filesystem_available: Option<u64>,
    pub(crate) accessible: bool,
    pub(crate) writable: bool,
    pub(crate) protected: bool,
    pub(crate) permission: String,
    pub(crate) explanation: Option<&'static str>,
    pub(crate) error: Option<String>,
    pub(crate) children: Vec<Node>,
}

#[derive(Debug, Clone, Copy)]
struct FilesystemSpace {
    total: u64,
    used: u64,
    free: u64,
    available: u64,
}

/// Directorio efímero privado para intercambiar el mapa con el proceso
/// elevado. El nombre del archivo por sí solo no protege frente a enlaces
/// simbólicos en `/tmp`; crear el directorio con modo 0700 impide que otras
/// cuentas puedan sustituirlo o leer el resultado.
#[cfg(target_os = "linux")]
struct PrivateMapTempDir(PathBuf);

#[cfg(target_os = "linux")]
impl PrivateMapTempDir {
    fn create() -> io::Result<Self> {
        let base = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default();
        for attempt in 0..32_u8 {
            let path = base.join(format!(
                "ltools-storage-map-{}-{stamp}-{attempt}",
                std::process::id()
            ));
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            match builder.create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "no se pudo reservar un directorio temporal privado",
        ))
    }
}

#[cfg(target_os = "linux")]
impl Drop for PrivateMapTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, Clone)]
struct MapOptions {
    roots: Vec<PathBuf>,
    depth: usize,
    max_children: usize,
    include_hidden: bool,
    follow_mounts: bool,
    format: String,
    output: Option<PathBuf>,
}

pub fn help() -> &'static str {
    "storage map|tree [--path RUTA] [--depth N] [--max-children N] [--format text|json|tsv] [--out FICHERO] [--no-hidden] [--follow-mounts]; storage manage delete|copy|move|zip|tar|open ..."
}

/// Variante para la GUI: usa el mismo escáner, pero publica cuántas rutas ha
/// visitado para que la ventana pueda mostrar actividad mientras el trabajo
/// se ejecuta fuera del hilo GTK.
#[cfg(target_os = "linux")]
pub(crate) fn gui_nodes_with_progress(
    progress: &std::sync::atomic::AtomicUsize,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Vec<Node> {
    let options = MapOptions {
        roots: Vec::new(),
        depth: 3,
        max_children: 100,
        include_hidden: true,
        follow_mounts: false,
        format: "text".into(),
        output: None,
    };
    let roots = std::env::var_os("LTOOLS_GUI_TREE_PATH")
        .map(PathBuf::from)
        .map(|path| vec![path])
        .unwrap_or_else(discover_roots);
    roots
        .into_iter()
        .map(|path| {
            let device = crate::common::device(&path);
            let mut node = scan_with_progress(
                &path,
                options.depth,
                &options,
                device,
                Some(progress),
                Some(cancelled),
            );
            annotate_filesystem_space(&mut node);
            node
        })
        .collect()
}

/// Repite el mapa desde un proceso autorizado por polkit. La GUI lo ofrece
/// solo cuando el escaneo normal encontró rutas inaccesibles; nunca eleva el
/// proceso silenciosamente ni usa una shell para construir la orden.
#[cfg(target_os = "linux")]
pub(crate) fn gui_nodes_with_privileges(
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<Vec<Node>, String> {
    if has_maximum_privileges() {
        return Err("el proceso ya tiene permisos máximos".into());
    }
    let use_sudo = sudo_available_without_prompt();
    if !use_sudo && !crate::common::command_exists("pkexec") {
        return Err(
            "no hay un autorizador gráfico disponible; se necesita pkexec/polkit o sudo autorizado sin prompt".into(),
        );
    }
    let executable =
        std::env::current_exe().map_err(|error| format!("no se pudo localizar LTools: {error}"))?;
    let temp_dir = PrivateMapTempDir::create()
        .map_err(|error| format!("no se pudo crear un directorio privado para el mapa: {error}"))?;
    let output = temp_dir.0.join("map.tsv");
    let roots = gui_scan_roots();
    let scoped_root = std::env::var_os("LTOOLS_GUI_TREE_PATH").map(PathBuf::from);
    let mut map_args = vec![
        "storage".to_owned(),
        "map".to_owned(),
        "--depth".to_owned(),
        "3".to_owned(),
        "--max-children".to_owned(),
        "100".to_owned(),
        "--format".to_owned(),
        "tsv".to_owned(),
        "--out".to_owned(),
        output.to_string_lossy().into_owned(),
        "--no-elevate".to_owned(),
        "--privileged-child".to_owned(),
    ];
    if let Some(root) = scoped_root.as_ref() {
        map_args.extend(["--path".to_owned(), root.display().to_string()]);
    }
    let mut command = if use_sudo {
        let mut command = Command::new("sudo");
        command.arg("--").arg(&executable).args(&map_args);
        command
    } else {
        let mut command = Command::new("pkexec");
        command.arg(&executable).args(&map_args);
        command
    };
    let mut child = command
        .env("LTOOLS_FRONTEND", "cli")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("no se pudo solicitar autorización administrativa: {error}"))?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    let status = loop {
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("el escaneo elevado se canceló al superar el tiempo límite".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("el proceso elevado superó el límite de 300 segundos".into());
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("falló la consulta del proceso elevado: {error}"));
            }
        }
    };
    if !status.success() {
        return Err("la autorización administrativa fue cancelada o rechazada".into());
    }
    let content = fs::read_to_string(&output)
        .map_err(|error| format!("no se pudo leer el mapa elevado: {error}"))?;
    parse_tsv_nodes(&content, &roots)
}

#[cfg(target_os = "linux")]
pub(crate) fn gui_can_request_privileges() -> bool {
    !has_maximum_privileges() && crate::platform::gui_privilege_available()
}

#[cfg(target_os = "linux")]
fn sudo_available_without_prompt() -> bool {
    crate::common::command_exists("sudo")
        && Command::new("sudo")
            .args(["-n", "-v"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
}

#[cfg(target_os = "linux")]
fn gui_scan_roots() -> Vec<PathBuf> {
    std::env::var_os("LTOOLS_GUI_TREE_PATH")
        .map(PathBuf::from)
        .map(|path| vec![path])
        .unwrap_or_else(discover_roots)
}

#[cfg(target_os = "linux")]
fn parse_tsv_nodes(content: &str, root_paths: &[PathBuf]) -> Result<Vec<Node>, String> {
    let mut nodes = std::collections::HashMap::<PathBuf, Node>::new();
    for (line_number, line) in content.lines().enumerate() {
        if line_number == 0 || line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 9 && fields.len() != 12 && fields.len() != 13 {
            return Err(format!(
                "el mapa elevado devolvió una fila TSV inválida en la línea {}",
                line_number + 1
            ));
        }
        let path = PathBuf::from(fields[0]);
        let size = fields[2]
            .parse::<u64>()
            .map_err(|_| format!("tamaño inválido en la línea {}", line_number + 1))?;
        let filesystem_total = fields.get(9).and_then(|value| parse_optional_u64(value));
        let filesystem_used = fields.get(10).and_then(|value| parse_optional_u64(value));
        let filesystem_free = fields.get(11).and_then(|value| parse_optional_u64(value));
        let filesystem_available = fields.get(12).and_then(|value| parse_optional_u64(value));
        let node = Node {
            name: path
                .file_name()
                .and_then(|value| value.to_str())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| path.to_str().unwrap_or("/"))
                .to_owned(),
            kind: match fields[1] {
                "file" => "file",
                "directory" => "directory",
                "symlink" => "symlink",
                "inaccessible" => "inaccessible",
                _ => "other",
            },
            path: path.clone(),
            size,
            filesystem_total,
            filesystem_used,
            filesystem_free,
            filesystem_available,
            accessible: fields[3] == "true",
            writable: fields[4] == "true",
            protected: fields[5] == "true",
            permission: fields[6].to_owned(),
            explanation: explain_path(&path),
            error: (!fields[8].is_empty()).then(|| fields[8].to_owned()),
            children: Vec::new(),
        };
        nodes.insert(path, node);
    }
    let mut paths = nodes.keys().cloned().collect::<Vec<_>>();
    paths.sort_by_key(|path| Reverse(path.components().count()));
    let mut roots = Vec::new();
    for path in paths {
        let Some(node) = nodes.remove(&path) else {
            continue;
        };
        if root_paths.iter().any(|root| root == &path) {
            roots.push(node);
            continue;
        }
        let mut parent = path.parent().map(Path::to_path_buf);
        let mut attach_to = None;
        while let Some(candidate) = parent {
            if candidate == path {
                break;
            }
            if nodes.contains_key(&candidate) {
                attach_to = Some(candidate);
                break;
            }
            parent = candidate.parent().map(Path::to_path_buf);
        }
        if let Some(candidate) = attach_to {
            if let Some(parent_node) = nodes.get_mut(&candidate) {
                parent_node.children.push(node);
            } else {
                roots.push(node);
            }
        } else {
            roots.push(node);
        }
    }
    roots.extend(nodes.into_values());
    sort_node_children(&mut roots);
    Ok(roots)
}

#[cfg(target_os = "linux")]
fn sort_node_children(nodes: &mut [Node]) {
    for node in nodes.iter_mut() {
        sort_node_children(&mut node.children);
        node.children.sort_by_key(|child| Reverse(child.size));
    }
    nodes.sort_by_key(|node| Reverse(node.size));
}

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = first_action(args).unwrap_or("map");
    match action {
        "map" | "tree" | "paths" => map(ctx, args),
        "explain" => explain(args),
        "manage" | "files" | "file-manager" => manage(ctx, args),
        "help" | "--help" | "-h" => {
            println!("Mapa de discos y rutas");
            println!("  {0}", help());
            println!("Cada raíz muestra capacidad total, espacio ocupado y espacio libre; el contenido del árbol es un dato separado.");
            println!(
                "Los errores de permisos se muestran y no se convierten en tamaño cero silencioso."
            );
            Ok(())
        }
        _ => Err(format!("acción del mapa desconocida: {action}")),
    }
}

fn first_action(args: &[String]) -> Option<&str> {
    crate::cli_args::positionals(args).first().copied()
}

fn value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .or_else(|| {
            args.iter()
                .find_map(|arg| arg.strip_prefix(&format!("{name}=")))
                .map(ToOwned::to_owned)
        })
}

fn has(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn map(ctx: &Context, args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    let roots = if options.roots.is_empty() {
        discover_roots()
    } else {
        options.roots.clone()
    };
    if roots.is_empty() {
        return Err("no se encontró ningún disco, volumen o ruta para mapear".into());
    }
    let root_devices = roots
        .iter()
        .map(|path| (path.clone(), crate::common::device(path)))
        .collect::<Vec<_>>();
    let mut nodes = Vec::new();
    for (path, device) in root_devices {
        if !path.exists() {
            nodes.push(Node {
                name: path.display().to_string(),
                path: path.clone(),
                kind: "missing",
                size: 0,
                filesystem_total: None,
                filesystem_used: None,
                filesystem_free: None,
                filesystem_available: None,
                accessible: false,
                writable: false,
                protected: crate::platform::critical_path(&path),
                permission: "no existe".into(),
                explanation: explain_path(&path),
                error: Some("la ruta no existe".into()),
                children: Vec::new(),
            });
            continue;
        }
        let mut node = scan(&path, options.depth, &options, device);
        annotate_filesystem_space(&mut node);
        nodes.push(node);
    }
    let rendered = match options.format.as_str() {
        "text" => render_text(&nodes),
        "json" => render_json(&nodes),
        "tsv" => render_tsv(&nodes),
        other => return Err(format!("--format no admite {other}; usa text, json o tsv")),
    };
    if let Some(output) = options.output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("no se pudo crear {}: {error}", parent.display()))?;
        }
        fs::write(&output, rendered.as_bytes())
            .map_err(|error| format!("no se pudo escribir {}: {error}", output.display()))?;
        println!("Mapa guardado: {}", output.display());
    } else {
        let mut stdout = std::io::stdout();
        stdout
            .write_all(rendered.as_bytes())
            .map_err(|error| error.to_string())?;
        if !rendered.ends_with('\n') {
            stdout.write_all(b"\n").map_err(|error| error.to_string())?;
        }
    }
    if has(args, "--elevated") && !has_maximum_privileges() {
        println!("Aviso: el proceso no tiene privilegios máximos; se conservaron los nodos inaccesibles para que puedas repetir el mapa con una terminal elevada.");
    }
    let _ = ctx;
    Ok(())
}

fn parse_options(args: &[String]) -> Result<MapOptions, String> {
    let roots = match value(args, "--path") {
        Some(path) => vec![validate_path(&path)?],
        None => Vec::new(),
    };
    let depth = value(args, "--depth")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "--depth debe ser un número".to_owned())
        })
        .transpose()?
        .unwrap_or(2)
        .min(12);
    let max_children = value(args, "--max-children")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "--max-children debe ser un número".to_owned())
        })
        .transpose()?
        .unwrap_or(80)
        .clamp(1, 10_000);
    let format = value(args, "--format")
        .unwrap_or_else(|| "text".into())
        .to_lowercase();
    Ok(MapOptions {
        roots,
        depth,
        max_children,
        include_hidden: !has(args, "--no-hidden"),
        follow_mounts: has(args, "--follow-mounts"),
        format,
        output: value(args, "--out").map(PathBuf::from),
    })
}

fn validate_path(raw: &str) -> Result<PathBuf, String> {
    if raw.trim().is_empty() || raw.chars().any(|character| character.is_control()) {
        return Err("la ruta está vacía o contiene caracteres de control".into());
    }
    Ok(PathBuf::from(raw))
}

fn discover_roots() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let mut roots = Vec::new();
        if let Ok(output) = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-PSDrive -PSProvider FileSystem | Select-Object -ExpandProperty Root",
            ])
            .output()
        {
            roots.extend(
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter_map(|line| {
                        let value = line.trim();
                        (!value.is_empty()).then(|| PathBuf::from(value))
                    }),
            );
        }
        if roots.is_empty() {
            roots.push(PathBuf::from("C:\\"));
        }
        roots
    }
    #[cfg(not(windows))]
    {
        let mut roots = vec![PathBuf::from("/")];
        if let Ok(content) = fs::read_to_string("/proc/self/mounts") {
            for line in content.lines() {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                if fields.len() < 3 {
                    continue;
                }
                let filesystem = fields[2];
                let mountpoint = unescape_mount(fields[1]);
                let pseudo = [
                    "proc",
                    "sysfs",
                    "devtmpfs",
                    "devpts",
                    "tmpfs",
                    "cgroup",
                    "cgroup2",
                    "overlay",
                    "squashfs",
                    "pstore",
                    "debugfs",
                    "tracefs",
                    "securityfs",
                    "configfs",
                    "fusectl",
                    "mqueue",
                    "hugetlbfs",
                    "autofs",
                ];
                if pseudo.contains(&filesystem)
                    || mountpoint == "/"
                    || mountpoint.starts_with("/proc")
                    || mountpoint.starts_with("/sys")
                    || mountpoint.starts_with("/dev")
                {
                    continue;
                }
                let path = PathBuf::from(mountpoint);
                if path.is_dir() {
                    roots.push(path);
                }
            }
        }
        roots.sort();
        roots.dedup();
        roots
    }
}

#[cfg(not(windows))]
fn unescape_mount(value: &str) -> String {
    let mut result = String::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if index + 3 < bytes.len()
            && bytes[index] == b'\\'
            && bytes[index + 1].is_ascii_digit()
            && bytes[index + 2].is_ascii_digit()
            && bytes[index + 3].is_ascii_digit()
        {
            let octal = &value[index + 1..index + 4];
            if let Ok(byte) = u8::from_str_radix(octal, 8) {
                result.push(byte as char);
                index += 4;
                continue;
            }
        }
        result.push(bytes[index] as char);
        index += 1;
    }
    result
}

fn scan(path: &Path, depth: usize, options: &MapOptions, device: Option<u64>) -> Node {
    scan_with_progress(path, depth, options, device, None, None)
}

fn scan_with_progress(
    path: &Path,
    depth: usize,
    options: &MapOptions,
    device: Option<u64>,
    progress: Option<&std::sync::atomic::AtomicUsize>,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Node {
    if let Some(progress) = progress {
        progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| path.to_str().unwrap_or("/"))
        .to_owned();
    let protected = crate::platform::critical_path(path);
    let explanation = explain_path(path);
    if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
        return inaccessible(
            name,
            path,
            protected,
            explanation,
            "escaneo cancelado".into(),
        );
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => return inaccessible(name, path, protected, explanation, error.to_string()),
    };
    let permission = permission_summary(&metadata);
    if metadata.file_type().is_symlink() {
        return Node {
            name,
            path: path.to_path_buf(),
            kind: "symlink",
            size: 0,
            filesystem_total: None,
            filesystem_used: None,
            filesystem_free: None,
            filesystem_available: None,
            accessible: true,
            writable: false,
            protected,
            permission,
            explanation,
            error: None,
            children: Vec::new(),
        };
    }
    if metadata.is_file() {
        return Node {
            name,
            path: path.to_path_buf(),
            kind: "file",
            size: metadata.len(),
            filesystem_total: None,
            filesystem_used: None,
            filesystem_free: None,
            filesystem_available: None,
            accessible: true,
            writable: writable(path, &metadata),
            protected,
            permission,
            explanation,
            error: None,
            children: Vec::new(),
        };
    }
    if !metadata.is_dir() {
        return Node {
            name,
            path: path.to_path_buf(),
            kind: "other",
            size: metadata.len(),
            filesystem_total: None,
            filesystem_used: None,
            filesystem_free: None,
            filesystem_available: None,
            accessible: true,
            writable: writable(path, &metadata),
            protected,
            permission,
            explanation,
            error: None,
            children: Vec::new(),
        };
    }
    let mut node = Node {
        name,
        path: path.to_path_buf(),
        kind: "directory",
        size: 0,
        filesystem_total: None,
        filesystem_used: None,
        filesystem_free: None,
        filesystem_available: None,
        accessible: true,
        writable: writable(path, &metadata),
        protected,
        permission,
        explanation,
        error: None,
        children: Vec::new(),
    };
    if depth == 0 {
        let (bytes, errors, first_error, root_readable) = measure_directory(
            path,
            device,
            options.follow_mounts,
            progress,
            cancelled,
            false,
        );
        node.size = bytes;
        node.accessible = root_readable;
        if errors > 0 {
            append_node_error(
                &mut node,
                format!(
                    "tamaño parcial: {errors} errores de acceso/lectura; {}",
                    first_error.unwrap_or_else(|| "detalle no disponible".into())
                ),
            );
        }
        return node;
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => {
            node.accessible = false;
            node.error = Some(error.to_string());
            return node;
        }
    };
    for entry in entries {
        if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
            node.accessible = false;
            append_node_error(
                &mut node,
                "escaneo interrumpido al alcanzar el límite de tiempo".into(),
            );
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                node.accessible = false;
                append_node_error(
                    &mut node,
                    format!("no se pudo enumerar una entrada: {error}"),
                );
                continue;
            }
        };
        let child_path = entry.path();
        let child_name = entry.file_name().to_string_lossy().into_owned();
        if !options.include_hidden && child_name.starts_with('.') {
            continue;
        }
        if !options.follow_mounts
            && device.is_some()
            && crate::common::device(&child_path) != device
        {
            continue;
        }
        node.children.push(scan_with_progress(
            &child_path,
            depth - 1,
            options,
            device,
            progress,
            cancelled,
        ));
    }
    node.children.sort_by_key(|child| Reverse(child.size));
    node.size = node.children.iter().map(|child| child.size).sum::<u64>();
    let omitted = node.children.len().saturating_sub(options.max_children);
    if omitted > 0 {
        node.children.truncate(options.max_children);
        append_node_error(
            &mut node,
            format!("{omitted} entradas omitidas por --max-children"),
        );
    }
    node
}

fn measure_directory(
    path: &Path,
    device: Option<u64>,
    follow_mounts: bool,
    progress: Option<&std::sync::atomic::AtomicUsize>,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
    count_current: bool,
) -> (u64, usize, Option<String>, bool) {
    use std::sync::atomic::Ordering;

    if count_current {
        if let Some(progress) = progress {
            progress.fetch_add(1, Ordering::Relaxed);
        }
    }
    if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        return (
            0,
            1,
            Some(format!("{}: escaneo cancelado", path.display())),
            false,
        );
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => return (0, 1, Some(format!("{}: {error}", path.display())), false),
    };
    if metadata.file_type().is_symlink()
        || (!follow_mounts
            && device.is_some_and(|expected| crate::common::device(path) != Some(expected)))
    {
        return (0, 0, None, true);
    }
    if metadata.is_file() {
        return (metadata.len(), 0, None, true);
    }
    if !metadata.is_dir() {
        return (0, 0, None, true);
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => return (0, 1, Some(format!("{}: {error}", path.display())), false),
    };
    let mut bytes = 0_u64;
    let mut errors = 0_usize;
    let mut first_error = None;
    for entry in entries {
        if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            errors = errors.saturating_add(1);
            first_error.get_or_insert_with(|| format!("{}: escaneo cancelado", path.display()));
            break;
        }
        let child_path = match entry {
            Ok(entry) => entry.path(),
            Err(error) => {
                errors = errors.saturating_add(1);
                first_error.get_or_insert_with(|| {
                    format!(
                        "{}: no se pudo enumerar una entrada: {error}",
                        path.display()
                    )
                });
                continue;
            }
        };
        let (child_bytes, child_errors, child_error, _) = measure_directory(
            &child_path,
            device,
            follow_mounts,
            progress,
            cancelled,
            true,
        );
        bytes = bytes.saturating_add(child_bytes);
        errors = errors.saturating_add(child_errors);
        if first_error.is_none() {
            first_error = child_error;
        }
    }
    (bytes, errors, first_error, true)
}

fn append_node_error(node: &mut Node, error: String) {
    match &mut node.error {
        Some(existing) => {
            existing.push_str("; ");
            existing.push_str(&error);
        }
        None => node.error = Some(error),
    }
}

fn inaccessible(
    name: String,
    path: &Path,
    protected: bool,
    explanation: Option<&'static str>,
    error: String,
) -> Node {
    Node {
        name,
        path: path.to_path_buf(),
        kind: "inaccessible",
        size: 0,
        filesystem_total: None,
        filesystem_used: None,
        filesystem_free: None,
        filesystem_available: None,
        accessible: false,
        writable: false,
        protected,
        permission: "denegado".into(),
        explanation,
        error: Some(error),
        children: Vec::new(),
    }
}

fn annotate_filesystem_space(node: &mut Node) {
    if let Some(space) = filesystem_space(&node.path) {
        node.filesystem_total = Some(space.total);
        node.filesystem_used = Some(space.used);
        node.filesystem_free = Some(space.free);
        node.filesystem_available = Some(space.available);
    }
}

fn filesystem_space(path: &Path) -> Option<FilesystemSpace> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let path = CString::new(path.as_os_str().as_bytes()).ok()?;
        let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        let result = unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) };
        if result != 0 {
            return None;
        }
        let stats = unsafe { stats.assume_init() };
        let block_size = if stats.f_frsize > 0 {
            stats.f_frsize
        } else {
            stats.f_bsize
        };
        if block_size == 0 {
            return None;
        }
        let total = stats.f_blocks.saturating_mul(block_size);
        let free = stats.f_bfree.saturating_mul(block_size).min(total);
        let available = stats.f_bavail.saturating_mul(block_size).min(free);
        return Some(FilesystemSpace {
            total,
            used: total.saturating_sub(free),
            free,
            available,
        });
    }
    #[cfg(windows)]
    {
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

        let path = path
            .as_os_str()
            .encode_wide()
            .chain(once(0))
            .collect::<Vec<_>>();
        let mut available = 0_u64;
        let mut total = 0_u64;
        let mut free = 0_u64;
        let ok =
            unsafe { GetDiskFreeSpaceExW(path.as_ptr(), &mut available, &mut total, &mut free) };
        if ok == 0 || total == 0 {
            return None;
        }
        let free = free.min(total);
        let available = available.min(free);
        return Some(FilesystemSpace {
            total,
            used: total.saturating_sub(free),
            free,
            available,
        });
    }
    #[allow(unreachable_code)]
    None
}

#[cfg(target_os = "linux")]
fn parse_optional_u64(value: &str) -> Option<u64> {
    (!value.is_empty())
        .then(|| value.parse::<u64>().ok())
        .flatten()
}

fn permission_summary(metadata: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        format!("mode={:o}", metadata.permissions().mode() & 0o7777)
    }
    #[cfg(windows)]
    {
        if metadata.permissions().readonly() {
            "readonly".into()
        } else {
            "read-write".into()
        }
    }
}

fn writable(path: &Path, metadata: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let Ok(path) = std::ffi::CString::new(path.as_os_str().as_bytes()) else {
            return false;
        };
        let mode = if metadata.is_dir() {
            libc::W_OK | libc::X_OK
        } else {
            libc::W_OK
        };
        // access() respects the effective filesystem access check and mount
        // flags, unlike inspecting mode bits alone. This field is advisory:
        // a later operation can still fail if permissions change after scan.
        unsafe { libc::faccessat(libc::AT_FDCWD, path.as_ptr(), mode, libc::AT_EACCESS) == 0 }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::Storage::FileSystem::{
            CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ,
            FILE_SHARE_WRITE, FILE_WRITE_DATA, OPEN_EXISTING,
        };

        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                FILE_WRITE_DATA,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                std::ptr::null(),
                OPEN_EXISTING,
                if metadata.is_dir() {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    0
                },
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            false
        } else {
            unsafe { CloseHandle(handle) };
            true
        }
    }
}

fn has_maximum_privileges() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        std::env::var("LTOOLS_ADMIN")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }
}

fn explain_path(path: &Path) -> Option<&'static str> {
    let key = explain_path_key(path)?;
    let explanation = crate::i18n::storage_map_text(key);
    (!explanation.is_empty()).then_some(explanation)
}

#[cfg(not(windows))]
fn path_is_within_home(path: &str, home: &Path) -> bool {
    let home = home.to_string_lossy().replace('\\', "/").to_lowercase();
    let home = home.trim_end_matches('/');
    if home.is_empty() || home == "/" {
        return path.starts_with('/');
    }
    path == home
        || path
            .strip_prefix(home)
            .is_some_and(|remainder| remainder.starts_with('/'))
}

fn explain_path_key(path: &Path) -> Option<&'static str> {
    let value = path.to_string_lossy().replace('\\', "/").to_lowercase();
    #[cfg(windows)]
    {
        let components = value.split('/').collect::<Vec<_>>();
        let root_index = components
            .iter()
            .position(|component| component.ends_with(':'))
            .or_else(|| {
                components
                    .iter()
                    .position(|component| component.starts_with("volume{"))
            })
            .map(|index| index + 1);
        let root_component = root_index.and_then(|index| components.get(index).copied());
        if root_component == Some("windows") {
            return Some("explain_windows_os");
        }
        if matches!(
            root_component,
            Some("program files" | "program files (x86)")
        ) {
            return Some("explain_managed_programs");
        }
        if root_component == Some("programdata") {
            return Some("explain_shared_app_data");
        }
        if root_component == Some("users") && components.len() == root_index.unwrap_or(0) + 1 {
            return Some("explain_user_profiles");
        }
        if components
            .iter()
            .position(|component| *component == "users")
            .is_some_and(|users_index| components[users_index + 1..].contains(&"appdata"))
        {
            return Some("explain_user_appdata");
        }
        if matches!(
            root_component,
            Some("system volume information" | "$recycle.bin")
        ) {
            return Some("explain_windows_metadata");
        }
    }
    #[cfg(not(windows))]
    {
        let configured_home = std::env::var_os("HOME").map(PathBuf::from);
        let under_user_tree = value == "/root"
            || value.starts_with("/root/")
            || value.starts_with("/home/")
            || value.starts_with("/users/")
            || configured_home
                .as_deref()
                .is_some_and(|home| path_is_within_home(&value, home));
        if under_user_tree {
            if value
                .split('/')
                .any(|component| component.eq_ignore_ascii_case(".config"))
            {
                return Some("explain_user_config");
            }
            if value
                .split('/')
                .any(|component| component.eq_ignore_ascii_case(".cache"))
            {
                return Some("explain_user_cache");
            }
        }
        let rules = [
            ("/", "explain_system_root"),
            ("/boot", "explain_boot"),
            ("/etc", "explain_system_config"),
            ("/usr", "explain_managed_programs"),
            ("/var", "explain_variable_data"),
            ("/home", "explain_user_data"),
            ("/tmp", "explain_temporary"),
            ("/proc", "explain_virtual_proc"),
            ("/sys", "explain_virtual_sys"),
            ("/dev", "explain_device_nodes"),
            ("/run", "explain_runtime"),
        ];
        for (prefix, key) in rules {
            if value == prefix || value.starts_with(&format!("{prefix}/")) {
                return Some(key);
            }
        }
    }
    None
}

fn render_text(nodes: &[Node]) -> String {
    let mut output = String::new();
    output.push_str("MAPA DE DISCOS, VOLUMENES Y RUTAS\n");
    output.push_str("Contenido = tamaño accesible acumulado; en cada raíz: total, ocupado, libre total y disponible para la cuenta; [P] ruta protegida; [!] permiso/lectura incompleta.\n\n");
    for (index, node) in nodes.iter().enumerate() {
        render_text_node(node, "", index + 1 == nodes.len(), &mut output);
    }
    output.push_str("\nAcciones: storage manage delete|copy|move|zip|tar|open ...\n");
    output.push_str("Para más profundidad: --depth N. Para todo el volumen: --max-children N y --follow-mounts.\n");
    output
}

fn render_text_node(node: &Node, prefix: &str, last: bool, output: &mut String) {
    let marker = if prefix.is_empty() {
        ""
    } else if last {
        "└── "
    } else {
        "├── "
    };
    let flags = format!(
        "{}{}{}",
        if node.protected { " [P]" } else { "" },
        if node.accessible { "" } else { " [!]" },
        if node.writable { " [rw]" } else { " [ro]" }
    );
    output.push_str(&format!(
        "{prefix}{marker}{}  contenido={}  {}{flags}{}\n",
        node.name,
        human_bytes(node.size),
        node.kind,
        filesystem_summary(node)
    ));
    if let Some(explanation) = node.explanation {
        output.push_str(&format!("{prefix}    ↳ {explanation}\n"));
    }
    if let Some(error) = &node.error {
        output.push_str(&format!("{prefix}    ↳ aviso: {error}\n"));
    }
    let child_prefix = if prefix.is_empty() {
        "".to_owned()
    } else {
        format!("{prefix}{}", if last { "    " } else { "│   " })
    };
    for (index, child) in node.children.iter().enumerate() {
        render_text_node(
            child,
            &child_prefix,
            index + 1 == node.children.len(),
            output,
        );
    }
}

fn render_json(nodes: &[Node]) -> String {
    format!(
        "{{\"schema\":\"ltools-storage-map-v1\",\"platform\":\"{}\",\"roots\":[{}]}}",
        if cfg!(windows) { "windows" } else { "linux" },
        nodes.iter().map(node_json).collect::<Vec<_>>().join(",")
    )
}

fn node_json(node: &Node) -> String {
    format!(
        "{{\"name\":\"{}\",\"path\":\"{}\",\"kind\":\"{}\",\"size\":{},\"filesystem_total\":{},\"filesystem_used\":{},\"filesystem_free\":{},\"filesystem_available\":{},\"accessible\":{},\"writable\":{},\"protected\":{},\"permission\":\"{}\",\"explanation\":{},\"error\":{},\"children\":[{}]}}",
        json_escape(&node.name),
        json_escape(&node.path.display().to_string()),
        node.kind,
        node.size,
        optional_json(node.filesystem_total),
        optional_json(node.filesystem_used),
        optional_json(node.filesystem_free),
        optional_json(node.filesystem_available),
        node.accessible,
        node.writable,
        node.protected,
        json_escape(&node.permission),
        node.explanation.map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".into()),
        node.error.as_ref().map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".into()),
        node.children.iter().map(node_json).collect::<Vec<_>>().join(",")
    )
}

fn render_tsv(nodes: &[Node]) -> String {
    let mut output = String::from(
        "path\tkind\tsize\taccessible\twritable\tprotected\tpermission\texplanation\terror\tfilesystem_total\tfilesystem_used\tfilesystem_free\tfilesystem_available\n",
    );
    for node in nodes {
        tsv_node(node, &mut output);
    }
    output
}

fn tsv_node(node: &Node, output: &mut String) {
    output.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        clean(&node.path.display().to_string()),
        node.kind,
        node.size,
        node.accessible,
        node.writable,
        node.protected,
        clean(&node.permission),
        clean(node.explanation.unwrap_or("")),
        clean(node.error.as_deref().unwrap_or("")),
        optional_tsv(node.filesystem_total),
        optional_tsv(node.filesystem_used),
        optional_tsv(node.filesystem_free),
        optional_tsv(node.filesystem_available)
    ));
    for child in &node.children {
        tsv_node(child, output);
    }
}

fn optional_json(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".into())
}

fn optional_tsv(value: Option<u64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn filesystem_summary(node: &Node) -> String {
    match (
        node.filesystem_total,
        node.filesystem_used,
        node.filesystem_free,
        node.filesystem_available,
    ) {
        (Some(total), Some(used), Some(free), Some(available)) => format!(
            " · total={} · ocupado={} · libre={} · disponible={}",
            human_bytes(total),
            human_bytes(used),
            human_bytes(free),
            human_bytes(available)
        ),
        _ => String::new(),
    }
}

fn clean(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn explain(args: &[String]) -> Result<(), String> {
    let path = value(args, "--path").ok_or("storage explain requiere --path RUTA")?;
    let path = validate_path(&path)?;
    println!("Ruta: {}", path.display());
    println!(
        "Tipo: {}",
        if path.is_dir() {
            "carpeta"
        } else if path.is_file() {
            "archivo"
        } else {
            "no accesible o inexistente"
        }
    );
    println!(
        "Protección: {}",
        if crate::platform::critical_path(&path) {
            "bloqueada para borrado/movimiento"
        } else {
            "no marcada como crítica"
        }
    );
    println!(
        "Permisos máximos del proceso: {}",
        if has_maximum_privileges() {
            "sí"
        } else {
            "no"
        }
    );
    println!(
        "Explicación: {}",
        explain_path(&path).unwrap_or(
            "No es una ruta estándar reconocida; revisa el propietario antes de modificarla."
        )
    );
    Ok(())
}

fn manage(ctx: &Context, args: &[String]) -> Result<(), String> {
    let operation = manage_operation(args)
        .ok_or("storage manage requiere delete, copy, move, zip, tar, open o permissions")?;
    match operation {
        "permissions" => explain(args),
        "open" => open_path(
            ctx,
            &value(args, "--path").ok_or("open requiere --path RUTA")?,
            has(args, "--yes"),
        ),
        "delete" => delete_path(
            ctx,
            &value(args, "--path").ok_or("delete requiere --path RUTA")?,
            has(args, "--yes"),
        ),
        "copy" | "move" => transfer(ctx, operation, args),
        "zip" | "tar" => archive(ctx, operation, args),
        _ => unreachable!(),
    }
}

fn manage_operation(args: &[String]) -> Option<&str> {
    let positionals = crate::cli_args::positionals(args);
    positionals
        .iter()
        .position(|arg| matches!(*arg, "manage" | "files" | "file-manager"))
        .and_then(|index| positionals.get(index + 1).copied())
        .filter(|arg| {
            matches!(
                *arg,
                "delete" | "copy" | "move" | "zip" | "tar" | "open" | "permissions"
            )
        })
}

fn checked_existing(raw: &str) -> Result<PathBuf, String> {
    let path = validate_path(raw)?;
    if !path.exists() {
        return Err(format!("no existe: {}", path.display()));
    }
    Ok(fs::canonicalize(&path).unwrap_or(path))
}

fn confirm(ctx: &Context, question: &str, yes: bool) -> bool {
    ctx.dry_run || yes || crate::common::ask(question)
}

fn delete_path(ctx: &Context, raw: &str, yes: bool) -> Result<(), String> {
    let path = checked_existing(raw)?;
    if crate::platform::critical_path(&path) {
        return Err(format!(
            "bloqueado por seguridad: {} es una ruta crítica",
            path.display()
        ));
    }
    if !confirm(
        ctx,
        &format!("¿Mover {} a la papelera?", path.display()),
        yes,
    ) {
        println!("Operación cancelada.");
        return Ok(());
    }
    let ok = move_to_trash(&path, ctx.dry_run).map_err(|error| error.to_string())?;
    if ok && !ctx.dry_run {
        if let Some(plan) = &ctx.plan {
            // La papelera nativa no expone aquí el destino interno; no afirmamos
            // que el plan pueda restaurarlo automáticamente.
            let _ = plan.record("trash-move", &path, "executed", false, "", "");
        }
    }
    if ok {
        Ok(())
    } else {
        Err("no se pudo mover la ruta a la papelera".into())
    }
}

fn transfer(ctx: &Context, operation: &str, args: &[String]) -> Result<(), String> {
    let source = checked_existing(
        &value(args, "--source")
            .or_else(|| value(args, "--path"))
            .ok_or("copy/move requiere --source RUTA")?,
    )?;
    let destination = resolve_future_path(&validate_path(
        &value(args, "--destination").ok_or("copy/move requiere --destination RUTA")?,
    )?)?;
    if destination == source || destination.starts_with(&source) {
        return Err("el destino no puede ser la propia fuente ni estar dentro de ella".into());
    }
    if path_exists_including_symlink(&destination) {
        return Err(format!(
            "el destino ya existe y no se sobrescribirá: {}",
            destination.display()
        ));
    }
    if crate::platform::critical_path(&source) && operation == "move" {
        return Err("no se puede mover una ruta crítica del sistema".into());
    }
    if !confirm(
        ctx,
        &format!(
            "¿{operation} {} a {}?",
            source.display(),
            destination.display()
        ),
        has(args, "--yes"),
    ) {
        println!("Operación cancelada.");
        return Ok(());
    }
    if ctx.dry_run {
        println!(
            "Simulación: se ejecutaría {operation} {} -> {}.",
            source.display(),
            destination.display()
        );
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut same_volume_move = false;
    if operation == "move" {
        match fs::rename(&source, &destination) {
            Ok(()) => same_volume_move = true,
            Err(error) if error.kind() == std::io::ErrorKind::CrossesDevices => {
                copy_new_path(&source, &destination).map_err(|copy_error| {
                    format!("no se pudo mover entre volúmenes: {copy_error}")
                })?;
                match move_to_trash(&source, false) {
                    Ok(true) => println!(
                        "El volumen era distinto: se copió a {} y la fuente se envió a la papelera.",
                        destination.display()
                    ),
                    Ok(false) => {
                        return Err(format!(
                            "se copió a {}, pero no se pudo enviar la fuente a la papelera; ambas copias se conservan",
                            destination.display()
                        ));
                    }
                    Err(trash_error) => {
                        return Err(format!(
                            "se copió a {}, pero no se pudo enviar la fuente a la papelera ({trash_error}); ambas copias se conservan",
                            destination.display()
                        ));
                    }
                }
            }
            Err(error) => return Err(format!("no se pudo mover: {error}")),
        }
    } else {
        copy_new_path(&source, &destination)?;
    }
    if let Some(plan) = &ctx.plan {
        if operation == "move" {
            let _ = plan.record(
                "path-move",
                &destination,
                "executed",
                same_volume_move,
                &source.display().to_string(),
                "",
            );
        } else {
            let _ = plan.record("remove-created", &destination, "executed", true, "", "");
        }
    }
    println!("Operación completada: {}", destination.display());
    Ok(())
}

fn path_exists_including_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// Canonicaliza también los directorios padre que ya existen, incluso si el
/// último componente todavía no. Esto impide que una ruta destino con un
/// padre simbólico eluda la comprobación de «copiar dentro de la fuente».
fn resolve_future_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("no se pudo resolver la ruta actual: {error}"))?
            .join(path)
    };
    let mut ancestor = absolute;
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(&ancestor) {
            Ok(_) => {
                let mut resolved = fs::canonicalize(&ancestor).map_err(|error| {
                    format!(
                        "no se pudo resolver el destino {}: {error}",
                        ancestor.display()
                    )
                })?;
                for component in suffix.iter().rev() {
                    resolved.push(component);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let component = ancestor
                    .components()
                    .next_back()
                    .ok_or_else(|| "la ruta de destino no tiene un componente válido".to_owned())?;
                match component {
                    std::path::Component::Normal(value) => suffix.push(value.to_os_string()),
                    std::path::Component::ParentDir => suffix.push("..".into()),
                    std::path::Component::CurDir => suffix.push(".".into()),
                    _ => {
                        return Err(format!(
                            "no se pudo resolver de forma segura el destino {}",
                            path.display()
                        ));
                    }
                }
                if !ancestor.pop() {
                    return Err(format!(
                        "no se encontró un directorio padre válido para {}",
                        path.display()
                    ));
                }
            }
            Err(error) => {
                return Err(format!(
                    "no se puede consultar el directorio destino {}: {error}",
                    ancestor.display()
                ));
            }
        }
    }
}

fn copy_new_path(source: &Path, destination: &Path) -> Result<(), String> {
    if path_exists_including_symlink(destination) {
        return Err(format!(
            "el destino ya existe y no se sobrescribirá: {}",
            destination.display()
        ));
    }
    if let Err(error) = copy_entry(source, destination) {
        let cleanup = remove_partial_copy(destination);
        return Err(match cleanup {
            Ok(()) => format!("{error}; se retiró la copia parcial"),
            Err(cleanup_error) => format!(
                "{error}; además, no se pudo retirar la copia parcial {}: {cleanup_error}",
                destination.display()
            ),
        });
    }
    Ok(())
}

fn copy_entry(source: &Path, destination: &Path) -> std::io::Result<()> {
    use std::io::Error;
    use std::io::ErrorKind;

    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "no se siguen ni copian enlaces simbólicos: {}",
                source.display()
            ),
        ));
    }
    if metadata.is_dir() {
        // create_dir falla si el destino ya existe: nunca mezcla árboles ni
        // reemplaza contenido que no pertenezca a esta operación.
        fs::create_dir(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_entry(&entry.path(), &destination.join(entry.file_name()))?;
        }
        fs::set_permissions(destination, metadata.permissions())?;
        return Ok(());
    }
    if !metadata.is_file() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("tipo de archivo no compatible: {}", source.display()),
        ));
    }
    let mut input = fs::File::open(source)?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    std::io::copy(&mut input, &mut output)?;
    drop(output);
    fs::set_permissions(destination, metadata.permissions())
}

fn remove_partial_copy(path: &Path) -> std::io::Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn archive(ctx: &Context, operation: &str, args: &[String]) -> Result<(), String> {
    let source = checked_existing(
        &value(args, "--source")
            .or_else(|| value(args, "--path"))
            .ok_or("zip/tar requiere --source RUTA")?,
    )?;
    let destination = resolve_future_path(&validate_path(
        &value(args, "--destination").ok_or("zip/tar requiere --destination ARCHIVO")?,
    )?)?;
    if path_exists_including_symlink(&destination) {
        return Err(format!(
            "el destino ya existe y no se sobrescribirá: {}",
            destination.display()
        ));
    }
    if destination == source || destination.starts_with(&source) {
        return Err("el archivo de destino no puede estar dentro de la fuente".into());
    }
    let (program, program_args) = if operation == "zip" {
        (
            "zip",
            vec![
                "-r".to_owned(),
                destination.display().to_string(),
                source.display().to_string(),
            ],
        )
    } else {
        (
            "tar",
            vec![
                "-cf".to_owned(),
                destination.display().to_string(),
                source.display().to_string(),
            ],
        )
    };
    println!("Archivo: {}", destination.display());
    println!("Fuente: {}", source.display());
    if !confirm(
        ctx,
        &format!("¿Crear archivo {operation} sin borrar la fuente?"),
        has(args, "--yes"),
    ) {
        println!("Operación cancelada.");
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: {} {}", program, program_args.join(" "));
        return Ok(());
    }
    if !crate::common::command_exists(program) {
        return Err(format!("{program} no está instalado"));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let status = Command::new(program)
        .args(&program_args)
        .status()
        .map_err(|error| error.to_string())?;
    if !status.success() {
        let _ = fs::remove_file(&destination);
        return Err(format!("{program} terminó con error"));
    }
    if let Some(plan) = &ctx.plan {
        let _ = plan.record("remove-created", &destination, "executed", true, "", "");
    }
    println!("Archivo creado: {}", destination.display());
    Ok(())
}

fn open_path(ctx: &Context, raw: &str, yes: bool) -> Result<(), String> {
    let path = checked_existing(raw)?;
    if !confirm(
        ctx,
        &format!("¿Abrir {} con el gestor nativo?", path.display()),
        yes,
    ) {
        println!("Operación cancelada.");
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se abriría {}.", path.display());
        return Ok(());
    }
    #[cfg(windows)]
    let status = Command::new("explorer.exe").arg(&path).status();
    #[cfg(not(windows))]
    let status = Command::new("xdg-open").arg(&path).status();
    status
        .map_err(|error| error.to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err("el gestor nativo terminó con error".into())
            }
        })
}

#[cfg(test)]
mod tests {
    #[cfg(not(windows))]
    use super::path_is_within_home;
    #[cfg(unix)]
    use super::resolve_future_path;
    use super::{
        copy_new_path, explain_path_key, manage_operation, parse_options, render_json, scan,
        scan_with_progress, MapOptions,
    };
    #[cfg(not(windows))]
    use std::path::Path;
    use std::path::PathBuf;

    #[cfg(target_os = "linux")]
    #[test]
    fn elevated_map_temp_directory_is_private_and_removed() {
        use super::PrivateMapTempDir;
        use std::os::unix::fs::PermissionsExt;

        let path = {
            let temp = PrivateMapTempDir::create().unwrap();
            let metadata = std::fs::metadata(&temp.0).unwrap();
            assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
            temp.0.clone()
        };
        assert!(
            !path.exists(),
            "el directorio temporal debe limpiarse al salir"
        );
    }

    #[test]
    fn parses_depth_and_machine_options() {
        let args = vec![
            "map".into(),
            "--path".into(),
            "/tmp".into(),
            "--depth".into(),
            "4".into(),
            "--format=json".into(),
        ];
        let options = parse_options(&args).unwrap();
        assert_eq!(options.depth, 4);
        assert_eq!(options.format, "json");
        assert_eq!(options.roots, vec![PathBuf::from("/tmp")]);
    }

    #[test]
    fn manage_action_does_not_confuse_path_values_with_operations() {
        let copy = vec![
            "manage".into(),
            "--path".into(),
            "delete".into(),
            "copy".into(),
        ];
        assert_eq!(manage_operation(&copy), Some("copy"));
        let delete = vec![
            "manage".into(),
            "--path".into(),
            "/tmp/target".into(),
            "delete".into(),
        ];
        assert_eq!(manage_operation(&delete), Some("delete"));
    }

    fn test_directory(label: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ltools-storage-copy-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn copy_is_non_overwriting_and_removes_partial_trees_on_error() {
        let root = test_directory("safe");
        let source = root.join("source");
        let existing = root.join("existing.txt");
        let copied = root.join("copied");
        std::fs::create_dir_all(source.join("nested")).unwrap();
        std::fs::write(source.join("nested/data.txt"), "source data").unwrap();
        std::fs::write(&existing, "keep existing data").unwrap();

        assert!(copy_new_path(&source, &existing).is_err());
        assert_eq!(
            std::fs::read_to_string(&existing).unwrap(),
            "keep existing data"
        );
        copy_new_path(&source, &copied).unwrap();
        assert_eq!(
            std::fs::read_to_string(copied.join("nested/data.txt")).unwrap(),
            "source data"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&source, source.join("recursive-link")).unwrap();
            let link_copy = root.join("link-copy");
            assert!(copy_new_path(&source, &link_copy).is_err());
            assert!(!link_copy.exists());
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn future_destination_resolves_existing_symlink_parents() {
        use std::os::unix::fs::symlink;

        let root = test_directory("symlink-parent");
        let source = root.join("source");
        let alias = root.join("source-alias");
        std::fs::create_dir(&source).unwrap();
        symlink(&source, &alias).unwrap();
        let destination = resolve_future_path(&alias.join("new/subtree")).unwrap();
        assert!(destination.starts_with(std::fs::canonicalize(&source).unwrap()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn collapsed_directory_size_reports_progress_and_totals() {
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "ltools-storage-map-depth-zero-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("nested")).unwrap();
        std::fs::write(root.join("nested/data.bin"), [7_u8; 37]).unwrap();
        let options = MapOptions {
            roots: vec![root.clone()],
            depth: 0,
            max_children: 20,
            include_hidden: true,
            follow_mounts: false,
            format: "text".into(),
            output: None,
        };
        let progress = AtomicUsize::new(0);
        let cancelled = AtomicBool::new(false);
        let node = scan_with_progress(
            &root,
            0,
            &options,
            crate::common::device(&root),
            Some(&progress),
            Some(&cancelled),
        );
        assert_eq!(node.size, 37);
        assert_eq!(node.error, None);
        assert!(progress.load(Ordering::Relaxed) >= 3);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn collapsed_directory_keeps_permission_errors_visible() {
        use std::os::unix::fs::PermissionsExt;
        use std::sync::atomic::AtomicUsize;
        use std::time::{SystemTime, UNIX_EPOCH};

        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "ltools-storage-map-denied-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o0)).unwrap();
        let (bytes, errors, detail, readable) = super::measure_directory(
            &root,
            crate::common::device(&root),
            false,
            Some(&AtomicUsize::new(0)),
            None,
            false,
        );
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(bytes, 0);
        assert_eq!(errors, 1);
        assert!(!readable);
        assert!(detail.unwrap().contains(&root.display().to_string()));
    }

    #[test]
    fn map_marks_standard_paths_with_explanations() {
        #[cfg(not(windows))]
        {
            let fixture_home = Path::new("/tmp/ltools-storage-map-fixture/home");
            assert!(path_is_within_home(
                "/tmp/ltools-storage-map-fixture/home/.cache/tool",
                fixture_home
            ));
            assert!(!path_is_within_home(
                "/tmp/ltools-storage-map-fixture/home-sibling/.cache/tool",
                fixture_home
            ));
            assert_eq!(
                explain_path_key(&PathBuf::from("/etc")),
                Some("explain_system_config")
            );
            assert_eq!(
                explain_path_key(&PathBuf::from("/home/example/.cache/tool")),
                Some("explain_user_cache")
            );
            assert_eq!(
                explain_path_key(&PathBuf::from("/home/example/.config/tool")),
                Some("explain_user_config")
            );
            assert_eq!(
                explain_path_key(&PathBuf::from("/usr/share/example/.cache")),
                Some("explain_managed_programs")
            );
        }
        #[cfg(windows)]
        {
            assert_eq!(
                explain_path_key(&PathBuf::from(r"C:\Windows\System32")),
                Some("explain_windows_os")
            );
            assert_eq!(
                explain_path_key(&PathBuf::from(r"C:\Program Files\Example")),
                Some("explain_managed_programs")
            );
            assert_eq!(
                explain_path_key(&PathBuf::from(r"C:\Users\example\AppData\Roaming")),
                Some("explain_user_appdata")
            );
        }
    }

    #[test]
    fn json_map_has_stable_schema() {
        let options = MapOptions {
            roots: vec![],
            depth: 0,
            max_children: 1,
            include_hidden: true,
            follow_mounts: false,
            format: "json".into(),
            output: None,
        };
        let node = scan(PathBuf::from(".").as_path(), options.depth, &options, None);
        assert!(render_json(&[node]).contains("ltools-storage-map-v1"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn elevated_tsv_rebuilds_roots_and_children() {
        let content = concat!(
            "path\tkind\tsize\taccessible\twritable\tprotected\tpermission\texplanation\terror\n",
            "/\tdirectory\t30\ttrue\ttrue\ttrue\tmode=0755\t\t\n",
            "/home\tdirectory\t20\ttrue\ttrue\ttrue\tmode=0755\t\t\n",
            "/home/user\tdirectory\t20\ttrue\ttrue\tfalse\tmode=0700\t\t\n",
            "/etc\tdirectory\t10\ttrue\tfalse\ttrue\tmode=0755\t\t\n",
        );
        let roots = super::parse_tsv_nodes(content, &[PathBuf::from("/")]).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].path, PathBuf::from("/"));
        assert_eq!(roots[0].children.len(), 2);
        let home = roots[0]
            .children
            .iter()
            .find(|node| node.path == std::path::Path::new("/home"))
            .unwrap();
        assert_eq!(home.children[0].path, PathBuf::from("/home/user"));
    }

    #[cfg(unix)]
    #[test]
    fn filesystem_space_reports_total_used_and_free() {
        let space = super::filesystem_space(PathBuf::from("/").as_path()).unwrap();
        assert!(space.total > 0);
        assert_eq!(space.total, space.used + space.free);
        assert!(space.free <= space.total);
        assert!(space.available <= space.free);
    }
}
