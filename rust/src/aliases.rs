//! Gestor persistente de alias de LTools.
//!
//! Los alias solo apuntan a comandos conocidos de LTools y conservan cada
//! argumento como un elemento independiente. No se evalúan como shell y no
//! pueden ejecutar programas arbitrarios.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA: &str = "# ltools-aliases-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Alias {
    name: String,
    command: String,
    args: Vec<String>,
    enabled: bool,
    description: String,
}

pub fn help() -> &'static str {
    "aliases list|ensure|doctor|add NOMBRE COMANDO [ARG... ]|remove NOMBRE|enable NOMBRE|disable NOMBRE|path|shell-init"
}

fn defaults() -> Vec<Alias> {
    [
        ("taudit", "audit", &[] as &[&str], "Auditoría rápida"),
        ("tpkg", "packages", &[], "Inventario de paquetes"),
        ("tgame", "games", &[], "Inventario de juegos"),
        ("tdisk", "storage", &[], "Discos y almacenamiento"),
        ("tsvc", "system", &[], "Sistema y servicios"),
        ("tproc", "system", &["processes"], "Procesos del sistema"),
        ("tjournal", "system", &["journal"], "Eventos del sistema"),
        ("tuser", "accounts", &[], "Cuentas y grupos"),
        ("tpaths", "defaults", &[], "Rutas predeterminadas"),
        ("tclean", "clean", &[], "Revisión de limpieza"),
        ("tdiag", "diagnostics", &[], "Diagnóstico nativo"),
        ("tauto", "automation", &[], "Automatización"),
        ("tnet", "native", &["network"], "Red nativa"),
        ("thw", "native", &["hardware"], "Hardware nativo"),
        ("tpower", "native", &["power"], "Energía nativa"),
        ("tsecurity", "native", &["security"], "Seguridad nativa"),
        ("tboot", "boot", &[], "Arranque y EFI"),
    ]
    .into_iter()
    .map(|(name, command, args, description)| Alias {
        name: name.to_owned(),
        command: command.to_owned(),
        args: args.iter().map(|value| (*value).to_owned()).collect(),
        enabled: true,
        description: description.to_owned(),
    })
    .collect()
}

fn registry_path() -> PathBuf {
    if let Some(path) = std::env::var_os("LTOOLS_ALIAS_FILE") {
        return PathBuf::from(path);
    }
    if let Some(root) = std::env::var_os("LTOOLS_ALIAS_HOME") {
        return PathBuf::from(root).join("aliases.tsv");
    }
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::common::home_dir().join("AppData/Roaming"))
            .join("LTools/aliases.tsv")
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::common::home_dir().join(".config"))
            .join("ltools/aliases.tsv")
    }
}

fn managed_bin_dir() -> PathBuf {
    if let Some(root) = std::env::var_os("LTOOLS_ALIAS_BIN") {
        return PathBuf::from(root);
    }
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::common::home_dir().join("AppData/Local"))
            .join("LTools/bin")
    }
    #[cfg(not(windows))]
    {
        crate::common::home_dir().join(".local/bin")
    }
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        && !value.starts_with('-')
}

fn valid_value(value: &str) -> bool {
    !value.chars().any(|character| character.is_control())
}

fn valid_command(value: &str) -> bool {
    matches!(
        value,
        "audit"
            | "games"
            | "packages"
            | "report"
            | "software"
            | "git"
            | "guide"
            | "automation"
            | "actions"
            | "tools"
            | "clean"
            | "prefix"
            | "defaults"
            | "system"
            | "boot"
            | "accounts"
            | "native"
            | "doctor"
            | "diagnostics"
            | "storage"
            | "registry"
            | "capabilities"
    )
}

fn validate(alias: &Alias) -> Result<(), String> {
    if !valid_name(&alias.name) {
        return Err(format!("nombre de alias inválido: {}", alias.name));
    }
    if !valid_command(&alias.command) {
        return Err(format!("comando de LTools no permitido: {}", alias.command));
    }
    if !alias.args.iter().all(|value| valid_value(value)) || !valid_value(&alias.description) {
        return Err("el alias contiene caracteres de control".into());
    }
    Ok(())
}

fn encode(value: &str) -> String {
    STANDARD.encode(value.as_bytes())
}

fn decode(value: &str) -> Result<String, String> {
    let bytes = STANDARD
        .decode(value)
        .map_err(|_| "registro de alias corrupto: base64 inválido".to_owned())?;
    String::from_utf8(bytes).map_err(|_| "registro de alias corrupto: UTF-8 inválido".into())
}

fn parse_registry(path: &Path) -> Result<Vec<Alias>, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("no se pudo leer {}: {error}", path.display()))?;
    let mut aliases = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 5 {
            return Err(format!(
                "registro de alias inválido en la línea {}",
                line_number + 1
            ));
        }
        let args = if fields[2].is_empty() {
            Vec::new()
        } else {
            fields[2]
                .split(',')
                .map(decode)
                .collect::<Result<Vec<_>, _>>()?
        };
        let alias = Alias {
            name: fields[0].to_owned(),
            command: fields[1].to_owned(),
            args,
            enabled: match fields[3] {
                "1" => true,
                "0" => false,
                _ => return Err(format!("estado inválido en la línea {}", line_number + 1)),
            },
            description: decode(fields[4])?,
        };
        validate(&alias)?;
        if aliases.iter().any(|item: &Alias| item.name == alias.name) {
            return Err(format!("alias duplicado: {}", alias.name));
        }
        aliases.push(alias);
    }
    Ok(aliases)
}

fn write_registry(path: &Path, aliases: &[Alias]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("no se pudo crear {}: {error}", parent.display()))?;
    }
    let mut output = String::from(SCHEMA);
    output.push_str(
        "\n# name\tcommand\targs(base64 comma-separated)\tenabled\tdescription(base64)\n",
    );
    for alias in aliases {
        validate(alias)?;
        let encoded_args = alias
            .args
            .iter()
            .map(|value| encode(value))
            .collect::<Vec<_>>()
            .join(",");
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            alias.name,
            alias.command,
            encoded_args,
            if alias.enabled { 1 } else { 0 },
            encode(&alias.description)
        ));
    }
    let temporary = path.with_extension("tsv.tmp");
    fs::write(&temporary, output)
        .map_err(|error| format!("no se pudo escribir {}: {error}", temporary.display()))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("no se pudo publicar {}: {error}", path.display()))
}

fn load_or_default() -> Result<(Vec<Alias>, bool), String> {
    let path = registry_path();
    if !path.is_file() {
        return Ok((defaults(), true));
    }
    Ok((parse_registry(&path)?, false))
}

/// Crea el registro base y añade alias predeterminados que aún no existan.
/// No sobrescribe alias personalizados ni alias desactivados.
pub fn ensure_defaults() -> Result<PathBuf, String> {
    let path = registry_path();
    let (mut aliases, missing_file) = load_or_default()?;
    let mut changed = missing_file;
    for default in defaults() {
        if !aliases.iter().any(|alias| alias.name == default.name) {
            aliases.push(default);
            changed = true;
        }
    }
    if changed {
        write_registry(&path, &aliases)?;
    }
    Ok(path)
}

/// Expande un alias gestionado antes de que el despachador procese la orden.
/// `Some(args)` también se devuelve para un alias desactivado: así evita que
/// una segunda capa de alias lo reactive silenciosamente.
pub fn expand(args: &[String]) -> Option<Vec<String>> {
    let name = args.first()?;
    let path = registry_path();
    let aliases = parse_registry(&path).ok()?;
    let alias = aliases.iter().find(|alias| alias.name == *name)?;
    if !alias.enabled {
        return Some(args.to_vec());
    }
    let mut expanded = vec![alias.command.clone()];
    expanded.extend(alias.args.clone());
    expanded.extend_from_slice(&args[1..]);
    Some(expanded)
}

fn alias_display(alias: &Alias) -> String {
    let args = alias
        .args
        .iter()
        .map(|value| crate::common::shell_display(value))
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        format!("{} -> {}", alias.name, alias.command)
    } else {
        format!("{} -> {} {args}", alias.name, alias.command)
    }
}

fn create_launcher() -> Result<PathBuf, String> {
    let executable = std::env::var_os("LTOOLS_ALIAS_EXECUTABLE")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .ok_or_else(|| "no se pudo localizar el ejecutable de LTools".to_owned())?;
    let directory = managed_bin_dir();
    fs::create_dir_all(&directory)
        .map_err(|error| format!("no se pudo crear {}: {error}", directory.display()))?;
    #[cfg(windows)]
    let path = directory.join("ltools.cmd");
    #[cfg(not(windows))]
    let path = directory.join("ltools");
    if path.exists() {
        let current = fs::read_to_string(&path).unwrap_or_default();
        if !current.contains("LTOOLS MANAGED ALIAS") {
            return Err(format!(
                "no se sobrescribe el lanzador existente: {}",
                path.display()
            ));
        }
    }
    #[cfg(windows)]
    let content = format!(
        "@echo off\r\nrem LTOOLS MANAGED ALIAS\r\n\"{}\" %*\r\n",
        executable.display()
    );
    #[cfg(not(windows))]
    let content = format!(
        "#!/bin/sh\n# LTOOLS MANAGED ALIAS\nexec '{}' \"$@\"\n",
        executable.display().to_string().replace('\'', "'\\''")
    );
    fs::write(&path, content)
        .map_err(|error| format!("no se pudo crear {}: {error}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).map_err(|error| error.to_string())?;
    }
    Ok(path)
}

fn path_contains(directory: &Path) -> bool {
    std::env::var_os("PATH")
        .map(|value| std::env::split_paths(&value).any(|entry| entry == directory))
        .unwrap_or(false)
}

fn print_help() {
    println!("Gestor de alias de LTools");
    println!("  {}", help());
    println!("Los alias apuntan a comandos LTools conocidos y conservan argumentos separados.");
}

pub fn run(args: &[String]) -> Result<(), String> {
    let operation = args.first().map(String::as_str).unwrap_or("list");
    match operation {
        "help" | "--help" | "-h" => print_help(),
        "path" => println!("{}", registry_path().display()),
        "list" => {
            let path = ensure_defaults()?;
            let aliases = parse_registry(&path)?;
            println!("Registro: {}", path.display());
            for alias in aliases {
                println!(
                    "{}{}  — {}",
                    if alias.enabled { "" } else { "[desactivado] " },
                    alias_display(&alias),
                    alias.description
                );
            }
        }
        "ensure" => {
            let path = ensure_defaults()?;
            let launcher = create_launcher()?;
            println!("Alias predeterminados listos: {}", path.display());
            println!("Lanzador creado: {}", launcher.display());
            println!(
                "PATH contiene el directorio: {}",
                path_contains(launcher.parent().unwrap_or(Path::new(".")))
            );
        }
        "doctor" => {
            let path = ensure_defaults()?;
            let launcher = managed_bin_dir().join(if cfg!(windows) {
                "ltools.cmd"
            } else {
                "ltools"
            });
            let aliases = parse_registry(&path)?;
            println!("Registro: {} ({})", path.display(), aliases.len());
            println!(
                "Lanzador: {}",
                if launcher.is_file() {
                    launcher.display().to_string()
                } else {
                    "no creado".into()
                }
            );
            println!(
                "PATH contiene el directorio: {}",
                path_contains(launcher.parent().unwrap_or(Path::new(".")))
            );
            println!(
                "Alias activos: {}",
                aliases.iter().filter(|alias| alias.enabled).count()
            );
        }
        "add" => {
            let name = args.get(1).ok_or("aliases add requiere NOMBRE")?;
            let command = args.get(2).ok_or("aliases add requiere COMANDO")?;
            let path = ensure_defaults()?;
            let mut aliases = parse_registry(&path)?;
            if aliases.iter().any(|alias| alias.name == *name) {
                return Err(format!(
                    "el alias ya existe: {name}; usa enable/disable o remove"
                ));
            }
            let alias = Alias {
                name: name.clone(),
                command: command.clone(),
                args: args[3..].to_vec(),
                enabled: true,
                description: "Alias personalizado".into(),
            };
            validate(&alias)?;
            aliases.push(alias.clone());
            write_registry(&path, &aliases)?;
            println!("Alias creado: {}", alias_display(&alias));
        }
        "remove" => {
            let name = args.get(1).ok_or("aliases remove requiere NOMBRE")?;
            let path = ensure_defaults()?;
            let mut aliases = parse_registry(&path)?;
            let Some(index) = aliases.iter().position(|alias| alias.name == *name) else {
                return Err(format!("alias no encontrado: {name}"));
            };
            if defaults().iter().any(|alias| alias.name == *name) {
                return Err(format!(
                    "{name} es predeterminado; usa aliases disable {name}"
                ));
            }
            aliases.remove(index);
            write_registry(&path, &aliases)?;
            println!("Alias retirado: {name}");
        }
        "enable" | "disable" => {
            let name = args
                .get(1)
                .ok_or("aliases enable/disable requiere NOMBRE")?;
            let path = ensure_defaults()?;
            let mut aliases = parse_registry(&path)?;
            let enabled = operation == "enable";
            let alias = aliases
                .iter_mut()
                .find(|alias| alias.name == *name)
                .ok_or_else(|| format!("alias no encontrado: {name}"))?;
            alias.enabled = enabled;
            write_registry(&path, &aliases)?;
            println!(
                "Alias {}: {name}",
                if enabled { "activado" } else { "desactivado" }
            );
        }
        "shell-init" => {
            println!(
                "Linux: añade {} a PATH para usar `ltools` y los alias gestionados.",
                managed_bin_dir().display()
            );
            println!(
                "Windows: añade {} a PATH para usar ltools.cmd desde PowerShell o CMD.",
                managed_bin_dir().display()
            );
        }
        _ => {
            print_help();
            return Err(format!("operación de alias desconocida: {operation}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{defaults, parse_registry, validate, Alias};

    #[test]
    fn defaults_are_valid_and_unique() {
        let aliases = defaults();
        assert!(!aliases.is_empty());
        for alias in &aliases {
            validate(alias).unwrap();
        }
        assert!(aliases.windows(2).all(|pair| pair[0].name != pair[1].name));
    }

    #[test]
    fn default_aliases_keep_arguments_as_values() {
        let alias = defaults()
            .into_iter()
            .find(|alias| alias.name == "tnet")
            .unwrap();
        assert_eq!(alias.command, "native");
        assert_eq!(alias.args, ["network"]);
    }

    #[test]
    fn registry_rejects_shell_like_commands() {
        let alias = Alias {
            name: "bad".into(),
            command: "sh -c".into(),
            args: Vec::new(),
            enabled: true,
            description: "bad".into(),
        };
        assert!(validate(&alias).is_err());
    }

    #[test]
    fn registry_parser_rejects_duplicates() {
        let path =
            std::env::temp_dir().join(format!("ltools-alias-test-{}.tsv", std::process::id()));
        std::fs::write(
            &path,
            "# ltools-aliases-v1\naudit\taudit\t\t1\tYQ==\naudit\taudit\t\t1\tYg==\n",
        )
        .unwrap();
        assert!(parse_registry(&path).is_err());
        let _ = std::fs::remove_file(path);
    }
}
