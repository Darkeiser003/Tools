//! Contrato de capacidades para integraciones externas.
//!
//! El JSON permite que una terminal, un lanzador o un frontend descubra qué
//! puede ejecutar LTools sin tener que interpretar la ayuda humana. No abre
//! ventanas por sí mismo: la terminal anfitriona debe implementar el protocolo
//! indicado en `terminal_integration`.

use crate::VERSION;

pub fn run(args: &[String]) -> Result<(), String> {
    let format = option_value(args, "--format").unwrap_or_else(|| "json".into());
    match format.as_str() {
        "json" => println!("{}", descriptor_json()),
        "terminal-json" => println!("{}", terminal_descriptor_json()),
        _ => return Err("capabilities admite --format json o --format terminal-json".into()),
    }
    Ok(())
}

pub fn descriptor_json() -> String {
    let platform = if cfg!(windows) { "windows" } else { "linux" };
    let command = if cfg!(windows) {
        "ltools.exe"
    } else {
        "ltools"
    };
    let features = if cfg!(windows) {
        "\"audit\", \"games\", \"packages\", \"protected-cleanup\",\n    \"storage\", \"registry\", \"defaults\", \"system-control\",\n    \"rollback\", \"dry-run\", \"plans\", \"tsv-export\", \"json-export\""
    } else {
        "\"audit\", \"games\", \"packages\", \"protected-cleanup\", \"wine-prefixes\",\n    \"storage\", \"registry\", \"defaults\", \"system-control\",\n    \"rollback\", \"dry-run\", \"plans\", \"tsv-export\", \"json-export\""
    };
    format!(
        r#"{{
  "schema": "ltools-capabilities-v1",
  "application": "LTools",
  "version": "{}",
  "platform": "{}",
  "entrypoints": {{
    "menu": {{ "command": "{}", "args": ["menu"], "interactive": true }},
    "help": {{ "command": "{}", "args": ["--help"], "interactive": false }},
    "doctor": {{ "command": "{}", "args": ["doctor"], "interactive": false }},
    "cli": {{ "command": "{}", "args": [], "interactive": false, "no_arguments": "shows-help" }}
  }},
  "features": [
    {}
  ],
  "terminal_integration": {{
    "schema": "lterminal-startup-v1",
    "capability_request": ["--ltools-capabilities", "--format", "json"],
    "open_request": ["--open-path", "PATH", "--command", "COMMAND", "--", "menu"],
    "working_directory": "PATH",
    "requires_host_terminal": true,
    "fallback_allowed": true
  }},
  "actions": [
{}
  ],
  "environment": {{
    "language": "LTOOLS_LANG",
    "state_directory": "XDG_STATE_HOME/ltools",
    "no_auto_terminal": "LTOOLS_NO_AUTO_TERMINAL",
    "cli_profile": "LTOOLS_CLI"
  }},
  "distribution": {{
    "linux": {{
      "artifacts": ["AppImage", "AppImage-cli", "tar.gz"],
      "standalone": true,
      "no_arguments": "opens-a-new-system-terminal-window",
      "shell": "LTOOLS_SHELL or SHELL, then bash/sh"
    }},
    "windows": {{
      "artifacts": ["exe", "portable-zip"],
      "standalone": true,
      "no_arguments": "opens-the-native-console-menu",
      "shell": "cmd.exe or PowerShell host"
    }}
  }},
  "host_tools": [
{}  ],
  "external_integrations": {{
    "lterminal": "ltools-terminal.json",
    "optional": true,
    "standalone_releases_require_it": false
  }}
}}"#,
        json_escape(VERSION),
        platform,
        command,
        command,
        command,
        if cfg!(windows) {
            "ltools-cli.exe"
        } else {
            "ltools-cli"
        },
        features,
        terminal_actions_json(platform),
        host_tools_json()
    )
}

/// Descriptor pequeño y estable para que una terminal sepa cómo integrar el
/// menú sin tener que interpretar el descriptor completo de LTools.
pub fn terminal_descriptor_json() -> String {
    terminal_descriptor_json_for(if cfg!(windows) { "windows" } else { "linux" })
}

fn terminal_descriptor_json_for(platform: &str) -> String {
    let (command, host_product, host_id) = if platform == "windows" {
        ("ltools.exe", "WinSlim Terminal", "winslim-terminal")
    } else {
        ("ltools", "LTerminal", "lterminal")
    };
    format!(
        r#"{{
  "schema": "ltools-terminal-integration-v1",
  "application": "LTools",
  "version": "{}",
  "platform": "{}",
  "integration": {{
    "optional": true,
    "standalone_releases_require_it": false,
    "exclusive_host_family": "lterminal"
  }},
  "host": {{
    "id": "{}",
    "family": "lterminal",
    "product": "{}",
    "known_products": ["LTerminal", "WinSlim Terminal"]
  }},
  "entrypoint": {{
    "command": "{}",
    "args": ["menu"],
    "interactive": true
  }},
  "working_directory_argument": "--open-path",
  "command_argument": "--command",
  "open_arguments": ["--open-path", "PATH", "--command", "COMMAND", "--", "menu"],
  "capability_request": ["--ltools-capabilities", "--format", "json"],
  "required_terminal_capability": "lterminal-startup-v1",
  "fallback": "explicit-only",
  "actions": [
{}
  ]
}}"#,
        json_escape(VERSION),
        platform,
        host_id,
        host_product,
        command,
        terminal_actions_json(platform)
    )
}

/// Acciones listas para convertirse en botones de LTerminal o WinSlim
/// Terminal. `args` es la forma canónica: la terminal no tiene que dividir ni
/// reinterpretar una cadena de shell. `command` se conserva como texto
/// legible para hosts antiguos que solo conocían ese campo.
fn terminal_actions_json(platform: &str) -> String {
    let command = if platform == "windows" {
        "ltools.exe"
    } else {
        "ltools"
    };
    let system_requirements: &[&str] = if platform == "windows" {
        &["tasklist"]
    } else {
        &["systemctl"]
    };
    let storage_requirements: &[&str] = if platform == "windows" {
        &["powershell"]
    } else {
        &["lsblk"]
    };
    let registry_requirements: &[&str] = if platform == "windows" {
        &["reg.exe"]
    } else {
        &[]
    };
    let mut actions = vec![
        action_json(
            "audit",
            "Auditar sistema",
            "Auditar",
            "Auditoría",
            "Inventario de discos, paquetes, aplicaciones y archivos grandes.",
            command,
            &["audit"],
            &[],
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "games",
            if platform == "windows" {
                "Inventariar juegos Windows"
            } else {
                "Auditar juegos, Wine y Proton"
            },
            "Juegos",
            "Auditoría",
            "Detecta lanzadores y configuraciones propias de la plataforma.",
            command,
            &["games"],
            &[],
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "packages",
            "Inventariar paquetes y almacenes",
            "Paquetes",
            "Auditoría",
            "Enumera gestores y formatos disponibles sin instalar nada.",
            command,
            &["packages"],
            &[],
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "clean-preview",
            "Previsualizar limpieza segura",
            "Limpieza",
            "Mantenimiento",
            "Genera un plan de limpieza; no borra nada por defecto.",
            command,
            &["clean", "--dry-run"],
            &[],
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "defaults",
            "Mostrar rutas predeterminadas",
            "Rutas",
            "Diagnóstico",
            "Muestra las rutas efectivas de las herramientas compatibles.",
            command,
            &["defaults"],
            &[],
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "system-status",
            if platform == "windows" {
                "Estado de servicios y procesos"
            } else {
                "Estado de systemd y procesos"
            },
            "Estado",
            "Sistema",
            "Consulta salud, servicios, procesos y registros sin cambiar el sistema.",
            command,
            &["system", "status"],
            system_requirements,
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "storage",
            "Discos y particiones",
            "Discos",
            "Sistema",
            "Consulta discos, volúmenes, montajes y particiones con herramientas nativas.",
            command,
            &["storage", "status"],
            storage_requirements,
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "registry",
            if platform == "windows" {
                "Consultar Registro de Windows"
            } else {
                "Consultar configuración del sistema"
            },
            "Configuración",
            "Sistema",
            "Consulta configuración nativa; en Windows usa reg.exe y en Linux rutas estándar.",
            command,
            &["registry", "status"],
            registry_requirements,
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "doctor",
            "Diagnosticar dependencias",
            "Diagnóstico",
            "Mantenimiento",
            "Comprueba solo las herramientas que LTools puede necesitar.",
            command,
            &["doctor"],
            &[],
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "help",
            "Mostrar ayuda de LTools",
            "Ayuda",
            "LTools",
            "Muestra todos los comandos y opciones disponibles.",
            command,
            &["--help"],
            &[],
            false,
            false,
            "none",
            true,
        ),
    ];
    actions.insert(
        6,
        action_json(
            "system-services",
            "Listar servicios",
            "Servicios",
            "Sistema",
            "Lista servicios relevantes y permite revisar su estado sin modificarlos.",
            command,
            &[
                "system",
                "services",
                "--scope",
                "both",
                "--filter",
                "noteworthy",
            ],
            system_requirements,
            false,
            false,
            "none",
            true,
        ),
    );
    actions.insert(
        7,
        action_json(
            "system-processes",
            "Ver procesos",
            "Procesos",
            "Sistema",
            "Muestra los procesos que más recursos consumen.",
            command,
            &["system", "processes", "--sort", "memory", "--limit", "20"],
            system_requirements,
            false,
            false,
            "none",
            true,
        ),
    );
    actions.insert(
        8,
        action_json(
            "system-journal",
            "Consultar errores recientes",
            "Journal",
            "Sistema",
            "Consulta errores recientes del sistema sin cambiar servicios.",
            command,
            &[
                "system", "journal", "--level", "error", "--hours", "24", "--limit", "100",
            ],
            system_requirements,
            false,
            false,
            "none",
            true,
        ),
    );
    actions.insert(
        9,
        action_json(
            "storage-partitions",
            "Ver particiones",
            "Particiones",
            "Sistema",
            "Lista particiones y volúmenes con el inventario nativo de la plataforma.",
            command,
            &["storage", "partitions"],
            storage_requirements,
            false,
            false,
            "none",
            true,
        ),
    );
    actions.insert(
        10,
        action_json(
            "registry-paths",
            if platform == "windows" {
                "Exportar rutas del Registro"
            } else {
                "Ver rutas de configuración"
            },
            "Rutas",
            "Sistema",
            "Muestra las ubicaciones de configuración sin editar datos.",
            command,
            &["registry", "paths"],
            registry_requirements,
            false,
            false,
            "none",
            true,
        ),
    );
    if platform != "windows" {
        actions.push(action_json(
            "prefixes",
            "Listar prefijos Wine y Proton",
            "Prefijos",
            "Compatibilidad",
            "Localiza prefijos Linux sin modificar ninguno.",
            command,
            &["prefix", "list"],
            &["wine"],
            false,
            false,
            "none",
            true,
        ));
    }
    actions.join(",\n")
}

#[allow(clippy::too_many_arguments)]
fn action_json(
    id: &str,
    label: &str,
    short_label: &str,
    group: &str,
    description: &str,
    command: &str,
    args: &[&str],
    requires_commands: &[&str],
    interactive: bool,
    requires_admin: bool,
    confirmation: &str,
    safe: bool,
) -> String {
    let args_json = args
        .iter()
        .map(|arg| format!("\"{}\"", json_escape(arg)))
        .collect::<Vec<_>>()
        .join(", ");
    let requires_json = requires_commands
        .iter()
        .map(|arg| format!("\"{}\"", json_escape(arg)))
        .collect::<Vec<_>>()
        .join(", ");
    let command_line = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };
    format!(
        "    {{\"id\":\"{}\",\"label\":\"{}\",\"shortLabel\":\"{}\",\"group\":\"{}\",\"description\":\"{}\",\"command\":\"{}\",\"executable\":\"{}\",\"args\":[{}],\"shell\":\"none\",\"workingDirectory\":\"current\",\"terminal\":true,\"interactive\":{},\"requiresAdmin\":{},\"confirmation\":\"{}\",\"safe\":{},\"supports\":[\"dry-run\"],\"requiresCommands\":[{}]}}",
        json_escape(id),
        json_escape(label),
        json_escape(short_label),
        json_escape(group),
        json_escape(description),
        json_escape(&command_line),
        json_escape(command),
        args_json,
        interactive,
        requires_admin,
        json_escape(confirmation),
        safe,
        requires_json
    )
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].clone())
}

fn host_tools_json() -> String {
    crate::platform::host_tools()
        .iter()
        .enumerate()
        .map(|(index, tool)| {
            let comma = if index + 1 == crate::platform::host_tools().len() {
                ""
            } else {
                ","
            };
            format!(
                "    {{\"id\":\"{}\",\"command\":\"{}\",\"category\":\"{}\",\"feature\":\"{}\",\"required\":{},\"installable\":{},\"install_package\":\"{}\",\"available\":{},\"version\":\"{}\"}}{}\n",
                json_escape(tool.id),
                json_escape(tool.command),
                json_escape(tool.category),
                json_escape(tool.feature),
                tool.required,
                tool.installable,
                json_escape(tool.install_package),
                crate::platform::host_tool_available(tool),
                json_escape(
                    &crate::platform::host_tool_version(tool).unwrap_or_default(),
                ),
                comma
            )
        })
        .collect()
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::{descriptor_json, terminal_descriptor_json};

    #[test]
    fn descriptor_declara_esquema_y_arranque_de_terminal() {
        let json = descriptor_json();
        assert!(json.contains("ltools-capabilities-v1"));
        assert!(json.contains("lterminal-startup-v1"));
        assert!(json.contains("--open-path"));
        assert!(json.contains("\"host_tools\""));
        assert!(json.contains("\"installable\":true"));
        assert!(json.contains("\"storage\""));
        assert!(json.contains("\"registry\""));
        assert!(json.contains("\"actions\": ["));
        assert!(json.contains("\"requiresCommands\""));
        assert!(json.contains("\"workingDirectory\":\"current\""));
        assert!(json.contains("\"cli\": {"));

        // The host-tool catalog is deliberately platform-specific.  Linux
        // exposes audit/prefix tools, while Windows exposes native system,
        // storage and registry tools.  Do not make a Windows build validate
        // Linux-only catalog entries (or vice versa).
        if cfg!(windows) {
            assert!(json.contains("\"category\":\"system\""));
            assert!(json.contains("\"category\":\"storage\""));
            assert!(json.contains("\"category\":\"registry\""));
            assert!(json.contains("\"install_package\":\"Docker.DockerCompose\""));
            assert!(json.contains("\"install_package\":\"Kubernetes.kubectl\""));
            assert!(!json.contains("\"category\":\"audit\""));
            assert!(!json.contains("wine-prefixes"));
            assert!(json.contains("\"command\": \"ltools.exe\""));
        } else {
            assert!(json.contains("\"category\":\"audit\""));
            assert!(json.contains("\"install_package\":\"rsync\""));
            assert!(!json.contains("\"category\":\"games\""));
            assert!(!json.contains("\"category\":\"virtualization\""));
            assert!(!json.contains("\"category\":\"development\""));
            assert!(!json.contains("\"command\":\"wine\""));
            assert!(json.contains("\"command\":\"docker\""));
            assert!(!json.contains("\"command\":\"git\""));
            assert!(json.contains("wine-prefixes"));
            assert!(json.contains("\"command\": \"ltools\""));
        }
    }

    #[test]
    fn descriptor_de_terminal_es_independiente_y_versionado() {
        let json = terminal_descriptor_json();
        assert!(json.contains("ltools-terminal-integration-v1"));
        assert!(json.contains("lterminal-startup-v1"));
        assert!(json.contains("\"optional\": true"));
        assert!(json.contains("\"standalone_releases_require_it\": false"));
        assert!(json.contains("\"exclusive_host_family\": \"lterminal\""));
        assert!(json.contains("\"id\":\"audit\""));
        assert!(json.contains("\"args\":[\"audit\"]"));
        assert!(json.contains("\"confirmation\":\"none\""));
        assert!(json.contains("LTerminal"));
        assert!(json.contains("WinSlim Terminal"));
        assert!(json.contains(crate::VERSION));
        if cfg!(windows) {
            assert!(json.contains("\"executable\":\"ltools.exe\""));
            assert!(json.contains("\"platform\": \"windows\""));
        } else {
            assert!(json.contains("\"executable\":\"ltools\""));
            assert!(json.contains("\"platform\": \"linux\""));
        }
    }

    #[test]
    fn descriptor_de_terminal_declara_la_variante_windows() {
        let json = super::terminal_descriptor_json_for("windows");
        assert!(json.contains("\"platform\": \"windows\""));
        assert!(json.contains("\"id\": \"winslim-terminal\""));
        assert!(json.contains("\"product\": \"WinSlim Terminal\""));
        assert!(json.contains("\"command\": \"ltools.exe\""));
        assert!(json.contains("\"standalone_releases_require_it\": false"));
        assert!(json.contains("\"id\":\"storage\""));
        assert!(json.contains("\"executable\":\"ltools.exe\""));
        assert!(!json.contains("\"id\":\"prefixes\""));
    }
}
