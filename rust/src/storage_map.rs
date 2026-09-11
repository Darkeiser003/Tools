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

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) kind: &'static str,
    pub(crate) size: u64,
    pub(crate) accessible: bool,
    pub(crate) writable: bool,
    pub(crate) protected: bool,
    pub(crate) permission: String,
    pub(crate) explanation: Option<&'static str>,
    pub(crate) error: Option<String>,
    pub(crate) children: Vec<Node>,
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

/// Snapshot used by the native GUI tree. It deliberately shares the scanner
/// with the CLI so the expandable view, JSON and text output never disagree.
#[cfg(target_os = "linux")]
pub(crate) fn gui_nodes() -> Vec<Node> {
    let options = MapOptions {
        roots: Vec::new(),
        depth: 4,
        max_children: 250,
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
            scan(&path, options.depth, &options, device)
        })
        .collect()
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
            println!("El tamaño de una carpeta incluye su contenido accesible.");
            println!(
                "Los errores de permisos se muestran y no se convierten en tamaño cero silencioso."
            );
            Ok(())
        }
        _ => Err(format!("acción del mapa desconocida: {action}")),
    }
}

fn first_action(args: &[String]) -> Option<&str> {
    args.iter()
        .map(String::as_str)
        .find(|value| !value.starts_with('-'))
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
        nodes.push(scan(&path, options.depth, &options, device));
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
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| path.to_str().unwrap_or("/"))
        .to_owned();
    let protected = crate::platform::critical_path(path);
    let explanation = explain_path(path);
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
            accessible: true,
            writable: writable(&metadata),
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
            accessible: true,
            writable: writable(&metadata),
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
        accessible: true,
        writable: writable(&metadata),
        protected,
        permission,
        explanation,
        error: None,
        children: Vec::new(),
    };
    if depth == 0 {
        node.size = crate::common::directory_size(path, device);
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
    for entry in entries.flatten() {
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
        node.children
            .push(scan(&child_path, depth - 1, options, device));
    }
    node.children.sort_by_key(|child| Reverse(child.size));
    node.size = node.children.iter().map(|child| child.size).sum::<u64>();
    let omitted = node.children.len().saturating_sub(options.max_children);
    if omitted > 0 {
        node.children.truncate(options.max_children);
        node.error = Some(format!("{omitted} entradas omitidas por --max-children"));
    }
    node
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
        accessible: false,
        writable: false,
        protected,
        permission: "denegado".into(),
        explanation,
        error: Some(error),
        children: Vec::new(),
    }
}

fn permission_summary(metadata: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        format!("mode={:o}", metadata.permissions().mode() & 0o7777)
    }
    #[cfg(windows)]
    {
        return if metadata.permissions().readonly() {
            "readonly".into()
        } else {
            "read-write".into()
        };
    }
}

fn writable(metadata: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o222 != 0
    }
    #[cfg(windows)]
    {
        !metadata.permissions().readonly()
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
    let value = path.to_string_lossy().replace('\\', "/").to_lowercase();
    #[cfg(windows)]
    {
        if value.ends_with("/windows") {
            return Some("Sistema Windows: no mover ni borrar; contiene el sistema operativo.");
        }
        if value.ends_with("/program files") || value.ends_with("/program files (x86)") {
            return Some("Programas instalados: no mover manualmente; usa su desinstalador.");
        }
        if value.ends_with("/programdata") {
            return Some("Datos compartidos de aplicaciones: modificar solo si conoces el programa propietario.");
        }
        if value.ends_with("/users") {
            return Some(
                "Perfiles de usuarios: cada carpeta contiene datos y configuración personales.",
            );
        }
        if value.ends_with("/appdata") {
            return Some("Configuración y cachés por usuario: algunas subcarpetas se pueden limpiar, otras son necesarias.");
        }
        if value.ends_with("/system volume information") || value.ends_with("/$recycle.bin") {
            return Some(
                "Metadatos protegidos de Windows: no modificar desde un gestor de archivos normal.",
            );
        }
    }
    #[cfg(not(windows))]
    {
        let rules = [
            ("/", "Raíz del sistema: contiene todo el árbol POSIX; no borrar ni mover."),
            ("/boot", "Arranque: kernel, initramfs y cargadores; no modificar sin un plan de recuperación."),
            ("/etc", "Configuración del sistema y servicios; editar solo archivos concretos y con copia."),
            ("/usr", "Programas y librerías gestionados por la distribución; no mover manualmente."),
            ("/var", "Datos variables, cachés, logs y bases de servicios; limpiar solo con el gestor adecuado."),
            ("/home", "Datos de usuarios; cada carpeta puede contener configuración y documentos personales."),
            ("/tmp", "Temporales; se pueden purgar según la política del sistema, no asumir que todo es prescindible."),
            ("/proc", "Vista virtual del kernel; no contiene archivos normales que se puedan copiar o borrar."),
            ("/sys", "Vista virtual del hardware y kernel; no modificar desde este gestor."),
            ("/dev", "Dispositivos especiales; no tratar como archivos comunes ni borrar nodos."),
            ("/run", "Estado efímero de servicios y sesiones; no mover ni borrar manualmente."),
            ("/.config", "Configuración de usuario solo si la ruta corresponde al perfil y no al sistema."),
            ("/.cache", "Caché de usuario: suele poder limpiarse, pero puede ralentizar el siguiente arranque."),
        ];
        for (prefix, explanation) in rules {
            if value == prefix || value.starts_with(&format!("{prefix}/")) {
                return Some(explanation);
            }
        }
    }
    None
}

fn render_text(nodes: &[Node]) -> String {
    let mut output = String::new();
    output.push_str("MAPA DE DISCOS, VOLUMENES Y RUTAS\n");
    output.push_str("Tamaño = contenido accesible acumulado; [P] ruta protegida; [!] permiso/lectura incompleta.\n\n");
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
        "{prefix}{marker}{}  {}  {}{flags}\n",
        node.name,
        human_bytes(node.size),
        node.kind
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
    format!("{{\"name\":\"{}\",\"path\":\"{}\",\"kind\":\"{}\",\"size\":{},\"accessible\":{},\"writable\":{},\"protected\":{},\"permission\":\"{}\",\"explanation\":{},\"error\":{},\"children\":[{}]}}", json_escape(&node.name), json_escape(&node.path.display().to_string()), node.kind, node.size, node.accessible, node.writable, node.protected, json_escape(&node.permission), node.explanation.map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".into()), node.error.as_ref().map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".into()), node.children.iter().map(node_json).collect::<Vec<_>>().join(","))
}

fn render_tsv(nodes: &[Node]) -> String {
    let mut output = String::from(
        "path\tkind\tsize\taccessible\twritable\tprotected\tpermission\texplanation\terror\n",
    );
    for node in nodes {
        tsv_node(node, &mut output);
    }
    output
}

fn tsv_node(node: &Node, output: &mut String) {
    output.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        clean(&node.path.display().to_string()),
        node.kind,
        node.size,
        node.accessible,
        node.writable,
        node.protected,
        clean(&node.permission),
        clean(node.explanation.unwrap_or("")),
        clean(node.error.as_deref().unwrap_or(""))
    ));
    for child in &node.children {
        tsv_node(child, output);
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
    let operation = args
        .iter()
        .position(|arg| {
            matches!(
                arg.as_str(),
                "delete" | "copy" | "move" | "zip" | "tar" | "open" | "permissions"
            )
        })
        .and_then(|index| args.get(index).map(String::as_str))
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
    let destination = validate_path(
        &value(args, "--destination").ok_or("copy/move requiere --destination RUTA")?,
    )?;
    if destination == source || destination.starts_with(&source) {
        return Err("el destino no puede ser la propia fuente ni estar dentro de ella".into());
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
    if operation == "move" {
        fs::rename(&source, &destination).map_err(|error| format!("no se pudo mover: {error}"))?;
    } else {
        copy_recursive(&source, &destination)?;
    }
    if let Some(plan) = &ctx.plan {
        let _ = plan.record("remove-created", &destination, "executed", true, "", "");
    }
    println!("Operación completada: {}", destination.display());
    Ok(())
}

fn copy_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    if source.is_dir() {
        fs::create_dir_all(destination).map_err(|error| error.to_string())?;
        for entry in fs::read_dir(source)
            .map_err(|error| error.to_string())?
            .flatten()
        {
            copy_recursive(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else {
        fs::copy(source, destination).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn archive(ctx: &Context, operation: &str, args: &[String]) -> Result<(), String> {
    let source = checked_existing(
        &value(args, "--source")
            .or_else(|| value(args, "--path"))
            .ok_or("zip/tar requiere --source RUTA")?,
    )?;
    let destination = validate_path(
        &value(args, "--destination").ok_or("zip/tar requiere --destination ARCHIVO")?,
    )?;
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
    use super::{explain_path, parse_options, render_json, scan, MapOptions};
    use std::path::PathBuf;

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
    fn map_marks_standard_paths_with_explanations() {
        #[cfg(not(windows))]
        assert!(explain_path(&PathBuf::from("/etc")).is_some());
        #[cfg(windows)]
        assert!(explain_path(&PathBuf::from(r"C:\Windows")).is_some());
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
}
