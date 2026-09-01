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
    format!(
        r#"{{
  "schema": "ltools-capabilities-v1",
  "application": "LTools",
  "version": "{}",
  "platform": "{}",
  "entrypoints": {{
    "menu": {{ "command": "ltools", "args": ["menu"], "interactive": true }},
    "help": {{ "command": "ltools", "args": ["--help"], "interactive": false }},
    "doctor": {{ "command": "ltools", "args": ["doctor"], "interactive": false }}
  }},
  "features": [
    "audit", "games", "packages", "protected-cleanup", "wine-prefixes",
    "defaults", "system-control", "rollback", "dry-run", "plans",
    "tsv-export", "json-export"
  ],
  "terminal_integration": {{
    "schema": "lterminal-startup-v1",
    "capability_request": ["--ltools-capabilities", "--format", "json"],
    "open_request": ["--open-path", "PATH", "--command", "COMMAND", "--", "menu"],
    "working_directory": "PATH",
    "requires_host_terminal": true,
    "fallback_allowed": true
  }},
  "environment": {{
    "language": "LTOOLS_LANG",
    "state_directory": "XDG_STATE_HOME/ltools",
    "no_auto_terminal": "LTOOLS_NO_AUTO_TERMINAL"
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
  "fallback": "explicit-only"
}}"#,
        json_escape(VERSION),
        platform,
        host_id,
        host_product,
        command
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
                "    {{\"id\":\"{}\",\"command\":\"{}\",\"category\":\"{}\",\"feature\":\"{}\",\"required\":{},\"installable\":{},\"install_package\":\"{}\",\"available\":{}}}{}\n",
                json_escape(tool.id),
                json_escape(tool.command),
                json_escape(tool.category),
                json_escape(tool.feature),
                tool.required,
                tool.installable,
                json_escape(tool.install_package),
                crate::platform::host_tool_available(tool),
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
        assert!(json.contains("\"category\":\"audit\""));
        assert!(json.contains("\"installable\":true"));
        assert!(json.contains("\"install_package\":\"rsync\""));
        assert!(!json.contains("\"category\":\"games\""));
        assert!(!json.contains("\"category\":\"virtualization\""));
        assert!(!json.contains("\"category\":\"development\""));
        assert!(!json.contains("\"command\":\"wine\""));
        assert!(!json.contains("\"command\":\"docker\""));
        assert!(!json.contains("\"command\":\"git\""));
    }

    #[test]
    fn descriptor_de_terminal_es_independiente_y_versionado() {
        let json = terminal_descriptor_json();
        assert!(json.contains("ltools-terminal-integration-v1"));
        assert!(json.contains("lterminal-startup-v1"));
        assert!(json.contains("\"optional\": true"));
        assert!(json.contains("\"standalone_releases_require_it\": false"));
        assert!(json.contains("\"exclusive_host_family\": \"lterminal\""));
        assert!(json.contains("LTerminal"));
        assert!(json.contains("WinSlim Terminal"));
        assert!(json.contains(crate::VERSION));
    }

    #[test]
    fn descriptor_de_terminal_declara_la_variante_windows() {
        let json = super::terminal_descriptor_json_for("windows");
        assert!(json.contains("\"platform\": \"windows\""));
        assert!(json.contains("\"id\": \"winslim-terminal\""));
        assert!(json.contains("\"product\": \"WinSlim Terminal\""));
        assert!(json.contains("\"command\": \"ltools.exe\""));
        assert!(json.contains("\"standalone_releases_require_it\": false"));
    }
}
