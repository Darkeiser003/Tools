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
        "\"audit\", \"games\", \"packages\", \"protected-cleanup\",\n    \"storage\", \"snapshots\", \"registry\", \"defaults\", \"system-control\", \"native-diagnostics\", \"native-actions\",\n    \"rollback\", \"dry-run\", \"plans\", \"tsv-export\", \"json-export\", \"verified-updates\", \"verified-update-download\""
    } else {
        "\"audit\", \"games\", \"packages\", \"protected-cleanup\", \"wine-prefixes\",\n    \"storage\", \"snapshots\", \"registry\", \"defaults\", \"system-control\", \"native-diagnostics\", \"native-actions\",\n    \"rollback\", \"dry-run\", \"plans\", \"tsv-export\", \"json-export\", \"verified-updates\", \"verified-update-download\""
    };
    let language_values = crate::i18n::SUPPORTED
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    let theme_values = crate::theme::SUPPORTED
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
  "schema": "ltools-capabilities-v1",
  "application": "{}",
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
  "ui_context": {{
    "language": {{
      "argument": "--lang",
      "environment": ["LTOOLS_LANG", "LTERMINAL_LANGUAGE", "LTERMINAL_LANG", "WTOOLS_TERMINAL_LANGUAGE", "WTOOLS_TERMINAL_LANG", "WINSLIM_TERMINAL_LANGUAGE", "WINSLIM_TERMINAL_LANG"],
      "values": [{}],
      "fallback": "locale"
    }},
    "theme": {{
      "argument": "--theme",
      "environment": ["LTOOLS_THEME", "LTERMINAL_THEME", "WTOOLS_TERMINAL_THEME", "WINSLIM_TERMINAL_THEME", "TERMINAL_THEME"],
      "default": "ocean",
      "values": [{}],
      "color_argument": "--color",
      "color_values": ["auto", "always", "never"]
    }}
  }},
  "distribution": {{
    "linux": {{
      "artifacts": ["AppImage", "AppImage-cli", "tar.gz"],
      "standalone": true,
      "no_arguments": "opens-native-graphical-window",
      "shell": "LTOOLS_SHELL or SHELL, then bash/sh",
      "updates": {{
        "check_args": ["update", "check"],
        "download_args": ["update", "download"],
        "package_selection": "AppImage when running inside AppImage, otherwise tar.gz; matches GUI/CLI profile",
        "destination": "user Downloads directory; never overwrites an existing file",
        "automatic_install": false,
        "integrity": "Ed25519 update signature and OpenSSH SSHSIG release signature, signed manifest checksum, artifact SHA-256 and size"
      }}
    }},
    "windows": {{
      "artifacts": ["exe", "portable-zip"],
      "standalone": true,
      "no_arguments": "opens-native-graphical-window",
      "shell": "cmd.exe or PowerShell host",
      "updates": {{
        "check_args": ["update", "check"],
        "download_args": ["update", "download"],
        "package_selection": "Windows EXE matching GUI/CLI profile and architecture",
        "destination": "user Downloads directory; never overwrites an existing file",
        "automatic_install": false,
        "integrity": "Ed25519 update signature and OpenSSH SSHSIG release signature, signed manifest checksum, artifact SHA-256 and size"
      }}
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
        json_escape(crate::i18n::product_name()),
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
        language_values,
        theme_values,
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
        ("ltools.exe", "WTools", "wtools")
    } else {
        ("ltools", "LTerminal", "lterminal")
    };
    let language_values = crate::i18n::SUPPORTED
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    let theme_values = crate::theme::SUPPORTED
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
  "schema": "ltools-terminal-integration-v1",
  "application": "{}",
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
    "known_products": ["LTerminal", "WTools"]
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
  "ui_context": {{
    "language": {{
      "argument": "--lang",
      "environment": ["LTOOLS_LANG", "LTERMINAL_LANGUAGE", "LTERMINAL_LANG", "WTOOLS_TERMINAL_LANGUAGE", "WTOOLS_TERMINAL_LANG", "WINSLIM_TERMINAL_LANGUAGE", "WINSLIM_TERMINAL_LANG"],
      "values": [{}],
      "fallback": "locale"
    }},
    "theme": {{
      "argument": "--theme",
      "environment": ["LTOOLS_THEME", "LTERMINAL_THEME", "WTOOLS_TERMINAL_THEME", "WINSLIM_TERMINAL_THEME", "TERMINAL_THEME"],
      "default": "ocean",
      "values": [{}],
      "color_argument": "--color",
      "color_values": ["auto", "always", "never"]
    }}
  }},
  "action_catalog": {{
    "schema": "ltools-actions-v1",
    "command": "{}",
    "args": ["actions", "list", "--format", "json"],
    "shell": "none",
    "target_policy": "explicit-only",
    "safe_defaults": true
  }},
  "actions": [
{}
  ]
}}"#,
        json_escape(crate::i18n::product_name()),
        json_escape(VERSION),
        platform,
        host_id,
        host_product,
        command,
        language_values,
        theme_values,
        command,
        terminal_actions_json(platform)
    )
}

/// Acciones listas para convertirse en botones de LTerminal o WTools
/// Terminal. `args` es la forma canónica: la terminal no tiene que dividir ni
/// reinterpretar una cadena de shell. `command` se conserva como texto
/// legible para hosts antiguos que solo conocían ese campo.
fn terminal_actions_json(platform: &str) -> String {
    let product_name = crate::i18n::product_name();
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
    let native_diagnostics_requirements: &[&str] = if platform == "windows" {
        &["powershell"]
    } else {
        &["uname"]
    };
    let native_dns_flush_requirements: &[&str] = if platform == "windows" {
        &["ipconfig"]
    } else {
        &["resolvectl"]
    };
    // Estas acciones tienen rutas integradas de compatibilidad. No deben
    // bloquearse en una GUI solo porque falte PowerShell: en Windows pueden
    // usar ipconfig/route/netstat/netsh, systeminfo/WMIC/cmd o netsh.
    let native_network_requirements: &[&str] = &[];
    let native_hardware_requirements: &[&str] = &[];
    let native_power_requirements: &[&str] = if platform == "windows" {
        &["powercfg"]
    } else {
        &[]
    };
    let native_security_requirements: &[&str] = &[];
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
            "Rutas predeterminadas",
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
            "native-diagnostics",
            "Diagnóstico nativo del anfitrión",
            "Salud del sistema",
            "Sistema",
            "Consulta salud, red, hardware y sesiones con herramientas nativas, sin modificar el sistema.",
            command,
            &["diagnostics", "health", "--format", "json"],
            native_diagnostics_requirements,
            false,
            false,
            "none",
            true,
        ),
        action_json(
            "help",
            &format!("Mostrar ayuda de {product_name}"),
            "Ayuda",
            product_name,
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
            "Rutas de configuración",
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
    actions.insert(
        11,
        action_json(
            "snapshots-status",
            "Consultar instantáneas y puntos de restauración",
            "Instantáneas",
            "Sistema",
            "Detecta backends de snapshot y permite revisar sus puntos de restauración.",
            command,
            &["snapshots", "status"],
            &[],
            false,
            false,
            "none",
            true,
        ),
    );
    actions.push(action_json(
        "native-network",
        "Consultar red nativa",
        "Red",
        "Sistema",
        "Consulta interfaces, rutas, DNS y puertos con las herramientas nativas de la plataforma.",
        command,
        &["native", "network", "status"],
        native_network_requirements,
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "native-hardware",
        "Consultar hardware nativo",
        "Hardware",
        "Sistema",
        "Consulta CPU, memoria y dispositivos con herramientas nativas, sin modificar el sistema.",
        command,
        &["native", "hardware", "status"],
        native_hardware_requirements,
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "native-power",
        "Consultar energía",
        "Energía",
        "Sistema",
        "Muestra el perfil de energía y los dispositivos de batería disponibles.",
        command,
        &["native", "power", "status"],
        native_power_requirements,
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "native-security",
        "Consultar seguridad nativa",
        "Seguridad",
        "Sistema",
        "Consulta el estado del firewall y las protecciones nativas sin realizar cambios.",
        command,
        &["native", "security", "status"],
        native_security_requirements,
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "native-security-scanners",
        "Consultar analizadores de código y CI",
        "Analizadores CI",
        "Sistema",
        "Enumera los analizadores locales y de CI disponibles, sin ejecutarlos ni modificar el repositorio.",
        command,
        &["native", "security", "scanners"],
        &[],
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "boot-status",
        if platform == "windows" {
            "Consultar BCD, EFI y recuperación"
        } else {
            "Consultar GRUB, EFI y systemd-boot"
        },
        "Arranque",
        "Sistema",
        "Inspecciona el arranque nativo sin modificar GRUB, EFI, BCD ni la NVRAM.",
        command,
        &["boot", "status"],
        if platform == "windows" {
            &["bcdedit"][..]
        } else {
            &[][..]
        },
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "boot-plan",
        "Preparar plan de arranque",
        "Plan de arranque",
        "Sistema",
        "Explica el flujo protegido para revisar y aplicar cambios de arranque.",
        command,
        &["boot", "plan"],
        &[],
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "native-dns-flush",
        "Vaciar caché DNS",
        "Vaciar DNS",
        "Sistema",
        "Vacía la caché DNS nativa tras confirmación explícita.",
        command,
        &["native", "network", "flush-dns"],
        native_dns_flush_requirements,
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "native-tools-menu",
        "Abrir herramientas operativas",
        "Herramientas",
        "Sistema",
        "Abre los flujos guiados de SSH, SCP, SFTP, ADB, contenedores y Kubernetes.",
        command,
        &["native", "tools", "menu"],
        &[],
        true,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "native-tools-install",
        "Instalar una dependencia nativa",
        "Instalar dependencia",
        "Herramientas",
        "Permite elegir una dependencia ausente, muestra el paquete y el gestor, y confirma antes de instalarla.",
        command,
        &["native", "tools", "install"],
        &[],
        true,
        true,
        "required",
        false,
    ));
    actions.push(action_json(
        "native-ssh-connect",
        "Conectar por SSH",
        "SSH",
        "Herramientas",
        "Conecta a un destino explícito mediante OpenSSH con confirmación.",
        command,
        &["native", "tools", "ssh-connect"],
        &["ssh"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "native-adb-install",
        "Instalar APK con ADB",
        "ADB install",
        "Herramientas",
        "Instala un APK en el dispositivo elegido mediante Android Debug Bridge.",
        command,
        &["native", "tools", "adb-install"],
        &["adb"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "native-container-run",
        "Crear y ejecutar contenedor",
        "Contenedor run",
        "Herramientas",
        "Crea un contenedor con un motor Docker o Podman explícito.",
        command,
        &["native", "tools", "container-run"],
        // El flujo pide el motor explícitamente y admite Docker o Podman;
        // exigir Docker aquí ocultaría una acción válida en instalaciones
        // que solo tienen Podman.
        &[],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "native-container-compose",
        "Gestionar Docker Compose / Podman Compose",
        "Compose",
        "Herramientas",
        "Expone las operaciones Compose guiadas: ciclo de vida, servicios, logs, build, run, exec y limpieza.",
        command,
        &["native", "tools", "container-compose"],
        &[],
        true,
        true,
        "required",
        false,
    ));
    actions.push(action_json(
        "native-container-prune",
        "Limpiar contenedores detenidos",
        "Prune de contenedores",
        "Herramientas",
        "Elimina solo contenedores detenidos después de una confirmación explícita.",
        command,
        &["native", "tools", "container-prune"],
        &[],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "native-image-build",
        "Construir imagen",
        "Build de imagen",
        "Herramientas",
        "Construye una imagen desde un Dockerfile o Containerfile y permite etiquetarla.",
        command,
        &["native", "tools", "image-build"],
        &[],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "native-system-df",
        "Consultar uso del motor",
        "Uso Docker/Podman",
        "Herramientas",
        "Muestra el uso de espacio de imágenes, contenedores, volúmenes y cachés.",
        command,
        &["native", "tools", "system-df"],
        &[],
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "native-kubernetes-apply",
        "Aplicar manifiesto Kubernetes",
        "Kubernetes apply",
        "Herramientas",
        "Aplica un manifiesto explícito al clúster seleccionado.",
        command,
        &["native", "tools", "kubernetes-apply"],
        &["kubectl"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "package-search",
        "Buscar un paquete en stores disponibles",
        "Buscar paquetes",
        "Paquetes",
        "Consulta solo los gestores instalados y devuelve candidatos por gestor y versión.",
        command,
        &["software", "search", "--format", "json"],
        &[],
        true,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "package-install",
        "Elegir e instalar un paquete",
        "Instalar paquete",
        "Paquetes",
        "Busca en los gestores disponibles, permite elegir un candidato y confirma antes de instalar.",
        command,
        &["software", "install"],
        &[],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-status",
        "Estado de repositorio Git",
        "Git status",
        "Git",
        "Consulta el repositorio actual o el indicado sin modificarlo.",
        command,
        &["git", "status"],
        &["git"],
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "git-lfs",
        "Consultar y gestionar Git LFS",
        "Git LFS",
        "Git",
        "Usa la integración instalada de Git LFS con argumentos separados; las consultas son de solo lectura y las descargas/subidas requieren confirmación.",
        command,
        &["git", "lfs"],
        &["git-lfs"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-clone",
        "Clonar repositorio Git",
        "Git clone",
        "Git",
        "Clona una URL confirmada en un destino nuevo, sin sobrescribir carpetas existentes.",
        command,
        &["git", "clone"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-fetch",
        "Actualizar referencias Git",
        "Git fetch",
        "Git",
        "Actualiza referencias remotas en un repositorio elegido tras confirmación.",
        command,
        &["git", "fetch"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-pull",
        "Integrar cambios Git",
        "Git pull",
        "Git",
        "Protege repositorios con cambios sin confirmar y confirma antes de integrar cambios.",
        command,
        &["git", "pull"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-login",
        "Comprobar identidad y autenticación Git",
        "Git login",
        "Git",
        "Muestra la identidad configurada y ofrece gh auth login si GitHub CLI está instalado; nunca maneja secretos.",
        command,
        &["git", "login"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-log",
        "Ver historial Git",
        "Git log",
        "Git",
        "Muestra el historial del repositorio sin modificarlo.",
        command,
        &["git", "log"],
        &["git"],
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "git-add",
        "Preparar cambios Git",
        "Git add",
        "Git",
        "Prepara rutas concretas o todos los cambios tras confirmación explícita.",
        command,
        &["git", "add"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-commit",
        "Crear commit Git",
        "Git commit",
        "Git",
        "Crea un commit con un mensaje proporcionado por el usuario y confirmación explícita.",
        command,
        &["git", "commit"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-push",
        "Publicar cambios Git",
        "Git push",
        "Git",
        "Publica cambios en el remoto elegido; nunca fuerza el push por defecto.",
        command,
        &["git", "push"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-branch",
        "Gestionar ramas Git",
        "Git branch",
        "Git",
        "Lista, crea, cambia o elimina ramas mediante una operación explícita.",
        command,
        &["git", "branch"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "git-tag",
        "Gestionar tags Git",
        "Git tag",
        "Git",
        "Lista o crea tags y permite publicar uno solo tras confirmación.",
        command,
        &["git", "tag"],
        &["git"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "gh-release",
        "Crear release de GitHub",
        "GitHub release",
        "Git",
        "Crea una release mediante gh, con tag, título y notas explícitos.",
        command,
        &["git", "release"],
        &["git", "gh"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "gh-login",
        "Iniciar sesión en GitHub",
        "GitHub login",
        "Git",
        "Abre el flujo oficial de autenticación de gh; LTools nunca recoge ni almacena credenciales.",
        command,
        &["git", "gh", "login"],
        &["gh"],
        true,
        false,
        "required",
        false,
    ));
    actions.push(action_json(
        "gh-repo",
        "Consultar repositorio GitHub",
        "GitHub repo",
        "Git",
        "Consulta los datos del repositorio actual o de owner/repo mediante gh.",
        command,
        &["git", "gh", "repo"],
        &["gh"],
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "gh-prs",
        "Consultar pull requests GitHub",
        "GitHub pull requests",
        "Git",
        "Lista pull requests del repositorio elegido mediante gh.",
        command,
        &["git", "gh", "prs"],
        &["gh"],
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "gh-releases",
        "Consultar releases GitHub",
        "GitHub releases",
        "Git",
        "Lista releases del repositorio elegido mediante gh.",
        command,
        &["git", "gh", "releases"],
        &["gh"],
        false,
        false,
        "none",
        true,
    ));
    actions.push(action_json(
        "gh-auth-status",
        "Consultar autenticación GitHub",
        "GitHub auth status",
        "Git",
        "Consulta el estado de autenticación de gh sin mostrar ni guardar secretos.",
        command,
        &["git", "gh", "auth-status"],
        &["gh"],
        false,
        false,
        "none",
        true,
    ));
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
    // `id` se conserva por compatibilidad con integraciones anteriores.  La
    // identidad canónica es namespaced y no depende de una etiqueta visible,
    // que puede repetirse entre grupos o idiomas.
    let action_key = canonical_action_key(id);
    let platform = if command.ends_with(".exe") {
        "windows"
    } else {
        "linux"
    };
    let qualified_action_key = format!("{platform}.{action_key}");
    let platform_label = if platform == "windows" {
        "Windows"
    } else {
        "Linux"
    };
    let display_name = format!("{platform_label} · {group} · {label}");
    let scope = action_key.split('.').next().unwrap_or("action");
    // Algunos hosts muestran `operation` sin el campo `scope`. Conservamos
    // toda la identidad para que nunca aparezca un ambiguo `status` repetido.
    let operation = qualified_operation(&action_key);
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
    // El registro interno conserva el nombre histórico `required`, pero el
    // contrato JSON usa únicamente valores versionados por su esquema.
    let confirmation = match confirmation {
        "required" => "before-run",
        value => value,
    };
    format!(
        "    {{\"id\":\"{}\",\"legacyId\":\"{}\",\"actionId\":\"{}\",\"actionKey\":\"{}\",\"qualifiedActionKey\":\"{}\",\"canonicalKey\":\"{}\",\"scope\":\"{}\",\"operation\":\"{}\",\"label\":\"{}\",\"shortLabel\":\"{}\",\"displayName\":\"{}\",\"menuPath\":[\"{}\",\"{}\"],\"group\":\"{}\",\"description\":\"{}\",\"command\":\"{}\",\"executable\":\"{}\",\"args\":[{}],\"aliases\":{},\"shell\":\"none\",\"workingDirectory\":\"current\",\"terminal\":true,\"interactive\":{},\"requiresAdmin\":{},\"confirmation\":\"{}\",\"safe\":{},\"supports\":[\"dry-run\"],\"requiresCommands\":[{}]}}",
        json_escape(&qualified_action_key),
        json_escape(id),
        json_escape(&qualified_action_key),
        json_escape(&action_key),
        json_escape(&qualified_action_key),
        json_escape(&qualified_action_key),
        json_escape(scope),
        json_escape(&operation),
        json_escape(label),
        json_escape(short_label),
        json_escape(&display_name),
        json_escape(group),
        json_escape(label),
        json_escape(group),
        json_escape(description),
        json_escape(&command_line),
        json_escape(command),
        args_json,
        crate::shortcuts::json(id),
        interactive,
        requires_admin,
        json_escape(confirmation),
        safe,
        requires_json
    )
}

fn qualified_operation(action_key: &str) -> String {
    action_key.replace('.', "-")
}

/// Returns the stable, namespaced identity exposed to host integrations.
///
/// The short `id` values predate the integration contract and are kept as
/// compatibility selectors.  These keys are deliberately independent from
/// translated labels and from the order of the catalog.
fn canonical_action_key(id: &str) -> String {
    let mapped = match id {
        "audit" => "audit.system",
        "games" => "audit.games",
        "packages" => "audit.packages",
        "clean-preview" => "maintenance.clean.preview",
        "defaults" => "diagnostics.defaults",
        "system-status" => "system.status",
        "system-services" => "system.services",
        "system-processes" => "system.processes",
        "system-journal" => "system.journal",
        "storage-partitions" => "storage.partitions",
        "registry-paths" => "system.registry.paths",
        "snapshots-status" => "snapshots.status",
        "storage" => "storage.status",
        "registry" => "system.registry.status",
        "doctor" => "maintenance.doctor",
        "native-diagnostics" => "diagnostics.native.health",
        "help" => "ltools.help",
        "native-network" => "native.network.status",
        "native-hardware" => "native.hardware.status",
        "native-power" => "native.power.status",
        "native-security" => "native.security.status",
        "native-security-scanners" => "native.security.scanners",
        "boot-status" => "boot.status",
        "boot-plan" => "boot.plan",
        "native-dns-flush" => "native.network.flush-dns",
        "native-tools-menu" => "native.tools.menu",
        "native-tools-install" => "native.tools.install",
        "native-ssh-connect" => "native.ssh.connect",
        "native-adb-install" => "native.adb.install",
        "native-container-run" => "native.containers.run",
        "native-container-compose" => "native.containers.compose",
        "native-container-prune" => "native.containers.prune",
        "native-image-build" => "native.containers.image-build",
        "native-system-df" => "native.containers.disk-usage",
        "native-kubernetes-apply" => "native.kubernetes.apply",
        "package-search" => "packages.search",
        "package-install" => "packages.install",
        "git-status" => "git.status",
        "git-lfs" => "git.lfs",
        "git-clone" => "git.clone",
        "git-fetch" => "git.fetch",
        "git-pull" => "git.pull",
        "git-login" => "git.identity",
        "git-log" => "git.log",
        "git-add" => "git.stage",
        "git-commit" => "git.commit",
        "git-push" => "git.push",
        "git-branch" => "git.branches",
        "git-tag" => "git.tags",
        "gh-release" => "github.release",
        "gh-login" => "github.login",
        "gh-repo" => "github.repository",
        "gh-prs" => "github.pull-requests",
        "gh-releases" => "github.releases",
        "gh-auth-status" => "github.authentication",
        "prefixes" => "wine.prefixes",
        _ => return id.replace('-', "."),
    };
    mapped.to_string()
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

    fn namespaced_key_is_valid(value: &str) -> bool {
        let mut parts = value.split('.');
        let Some(first) = parts.next() else {
            return false;
        };
        !first.is_empty()
            && parts.clone().next().is_some()
            && value.chars().all(|character| {
                character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || character == '.'
                    || character == '-'
            })
            && parts.all(|part| !part.is_empty())
    }

    #[test]
    fn descriptor_declara_esquema_y_arranque_de_terminal() {
        let json = descriptor_json();
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("el descriptor de capacidades debe ser JSON válido");
        assert!(json.contains("ltools-capabilities-v1"));
        assert!(json.contains("lterminal-startup-v1"));
        assert!(json.contains("--open-path"));
        assert!(json.contains("\"host_tools\""));
        assert!(json.contains("native-tools-install"));
        assert!(json.contains("native-container-compose"));
        assert!(json.contains("native-container-prune"));
        let expected_application = if cfg!(windows) {
            "\"application\": \"WTools\""
        } else {
            "\"application\": \"LTools\""
        };
        assert!(json.contains(expected_application));
        assert!(json.contains("\"installable\":true"));
        assert!(json.contains("\"storage\""));
        assert!(json.contains("\"registry\""));
        assert!(json.contains("\"actions\": ["));
        assert!(json.contains("\"requiresCommands\""));
        assert!(json.contains("\"workingDirectory\":\"current\""));
        assert!(json.contains("\"cli\": {"));
        assert!(json.contains("\"ui_context\": {"));
        assert!(json.contains("LTERMINAL_LANGUAGE"));
        assert!(json.contains("LTERMINAL_THEME"));
        assert!(json.contains("\"default\": \"ocean\""));
        assert!(json.contains("\"winslim\""));
        assert!(json.contains("\"zh\""));
        assert!(parsed["features"]
            .as_array()
            .unwrap()
            .iter()
            .any(|feature| feature == "verified-updates"));
        assert_eq!(
            parsed["distribution"]["linux"]["updates"]["automatic_install"],
            false
        );
        assert_eq!(
            parsed["distribution"]["windows"]["updates"]["automatic_install"],
            false
        );
        assert_eq!(
            parsed["distribution"]["linux"]["updates"]["download_args"][1],
            "download"
        );
        assert_eq!(
            parsed["distribution"]["windows"]["updates"]["download_args"][1],
            "download"
        );
        // El catálogo de herramientas es específico de cada plataforma:
        // Linux expone auditoría/prefijos y Windows herramientas nativas.
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
            assert!(json.contains("\"category\":\"utilities\""));
            assert!(json.contains("\"category\":\"development\""));
            assert!(json.contains("\"install_package\":\"rsync\""));
            assert!(!json.contains("\"category\":\"games\""));
            assert!(!json.contains("\"category\":\"virtualization\""));
            assert!(!json.contains("\"command\":\"wine\""));
            assert!(json.contains("\"command\":\"docker\""));
            assert!(json.contains("\"command\":\"git\""));
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
        let expected_application = if cfg!(windows) {
            "\"application\": \"WTools\""
        } else {
            "\"application\": \"LTools\""
        };
        assert!(json.contains(expected_application));
        assert!(json.contains("\"legacyId\":\"audit\""));
        assert!(json.contains("\"args\":[\"audit\"]"));
        assert!(json.contains("\"confirmation\":\"none\""));
        assert!(json.contains("LTerminal"));
        assert!(json.contains("WTools"));
        assert!(json.contains(crate::VERSION));
        assert!(json.contains("\"action_catalog\": {"));
        assert!(json.contains("ltools-actions-v1"));
        assert!(json.contains("\"ui_context\": {"));
        assert!(json.contains("\"color_values\": [\"auto\", \"always\", \"never\"]"));
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
        assert!(json.contains("\"id\": \"wtools\""));
        assert!(json.contains("\"product\": \"WTools\""));
        assert!(json.contains("\"command\": \"ltools.exe\""));
        assert!(json.contains("\"standalone_releases_require_it\": false"));
        assert!(json.contains("\"legacyId\":\"storage\""));
        assert!(json.contains("\"executable\":\"ltools.exe\""));
        assert!(!json.contains("\"confirmation\":\"required\""));
        assert!(!json.contains("\"legacyId\":\"prefixes\""));
    }

    #[test]
    fn acciones_de_terminal_tienen_identidad_y_etiqueta_sin_colisiones() {
        let parsed: serde_json::Value =
            serde_json::from_str(&super::terminal_descriptor_json_for(if cfg!(windows) {
                "windows"
            } else {
                "linux"
            }))
            .expect("el descriptor de terminal debe ser JSON válido");
        let actions = parsed["actions"]
            .as_array()
            .expect("el descriptor debe publicar una lista de acciones");
        assert!(!actions.is_empty());

        let mut ids = std::collections::HashSet::new();
        let mut action_ids = std::collections::HashSet::new();
        let mut keys = std::collections::HashSet::new();
        let mut qualified_keys = std::collections::HashSet::new();
        let mut canonical_keys = std::collections::HashSet::new();
        let mut operations = std::collections::HashSet::new();
        let mut labels = std::collections::HashSet::new();
        let mut short_labels = std::collections::HashSet::new();
        let mut display_names = std::collections::HashSet::new();
        let mut checked_dns_requirements = false;
        let mut checked_optional_native_requirements = false;
        for action in actions {
            let id = action["id"].as_str().expect("cada acción necesita id");
            let legacy_id = action["legacyId"]
                .as_str()
                .expect("cada acción necesita legacyId");
            let action_id = action["actionId"]
                .as_str()
                .expect("cada acción necesita actionId");
            let key = action["actionKey"]
                .as_str()
                .expect("cada acción necesita actionKey");
            let qualified_key = action["qualifiedActionKey"]
                .as_str()
                .expect("cada acción necesita qualifiedActionKey");
            let canonical_key = action["canonicalKey"]
                .as_str()
                .expect("cada acción necesita canonicalKey");
            let scope = action["scope"]
                .as_str()
                .expect("cada acción necesita scope");
            let operation = action["operation"]
                .as_str()
                .expect("cada acción necesita operation");
            let label = action["label"]
                .as_str()
                .expect("cada acción necesita label");
            let short_label = action["shortLabel"]
                .as_str()
                .expect("cada acción necesita shortLabel");
            let display_name = action["displayName"]
                .as_str()
                .expect("cada acción necesita displayName");
            let menu_path = action["menuPath"]
                .as_array()
                .expect("cada acción necesita menuPath");
            let group = action["group"]
                .as_str()
                .expect("cada acción necesita group");
            let description = action["description"]
                .as_str()
                .expect("cada acción necesita description");
            let safe = action["safe"]
                .as_bool()
                .expect("cada acción necesita declarar safe");
            let confirmation = action["confirmation"]
                .as_str()
                .expect("cada acción necesita declarar confirmation");

            assert!(ids.insert(id), "id de acción duplicado: {id}");
            assert!(
                action_ids.insert(action_id),
                "actionId duplicado: {action_id}"
            );
            assert!(keys.insert(key), "actionKey duplicado: {key}");
            assert!(
                qualified_keys.insert(qualified_key),
                "qualifiedActionKey duplicado: {qualified_key}"
            );
            assert!(
                canonical_keys.insert(canonical_key),
                "canonicalKey duplicado: {canonical_key}"
            );
            assert!(
                labels.insert(label),
                "etiqueta de acción duplicada: {label}"
            );
            assert!(
                short_labels.insert(short_label),
                "shortLabel de acción duplicado: {short_label}"
            );
            assert!(
                display_names.insert(display_name),
                "displayName de acción duplicado: {display_name}"
            );
            assert!(namespaced_key_is_valid(key), "actionKey inválido: {key}");
            assert_eq!(key.split('.').next(), Some(scope));
            assert!(
                operations.insert(operation),
                "operation duplicada: {operation}"
            );
            assert_eq!(operation, key.replace('.', "-"));
            let expected_platform = if cfg!(windows) { "windows" } else { "linux" };
            assert_eq!(qualified_key, format!("{expected_platform}.{key}"));
            assert_eq!(canonical_key, qualified_key);
            assert_eq!(id, action_id);
            assert!(!legacy_id.trim().is_empty());
            assert!(!label.trim().is_empty());
            assert!(!short_label.trim().is_empty());
            assert!(!group.trim().is_empty());
            assert!(!description.trim().is_empty());
            assert_eq!(menu_path.len(), 2);
            assert_eq!(menu_path[0].as_str(), Some(group));
            assert_eq!(menu_path[1].as_str(), Some(label));
            assert!(display_name.starts_with(if cfg!(windows) {
                "Windows · "
            } else {
                "Linux · "
            }));
            assert_eq!(
                safe,
                confirmation == "none",
                "safe y confirmation deben describir la misma política para {key}"
            );

            if key == "native.network.flush-dns" {
                let requirements = action["requiresCommands"]
                    .as_array()
                    .expect("flush-dns necesita declarar requisitos");
                let expected = if cfg!(windows) {
                    "ipconfig"
                } else {
                    "resolvectl"
                };
                assert!(requirements.iter().any(|value| value == expected));
                assert!(requirements.iter().all(|value| value != "ip"));
                checked_dns_requirements = true;
            }
            if matches!(
                key,
                "native.network.status" | "native.hardware.status" | "native.security.status"
            ) {
                let requirements = action["requiresCommands"]
                    .as_array()
                    .expect("la acción nativa necesita declarar requisitos");
                assert!(requirements.iter().all(|value| value != "ip"));
                checked_optional_native_requirements = true;
            }
        }
        assert!(checked_dns_requirements);
        assert!(checked_optional_native_requirements);
    }
}
