use crate::common::{ask, command_exists, run_with_sudo, Context};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let area = first(args).unwrap_or("menu");
    match area {
        "network" | "net" => network(ctx, sub(args, area)),
        "hardware" | "hw" => hardware(ctx, sub(args, area)),
        "power" | "energy" => power(ctx, sub(args, area)),
        "security" | "firewall" => security(ctx, sub(args, area)),
        "tools" | "devtools" | "tooling" => tools(ctx, sub(args, area), args),
        "containers" | "container" => {
            let action = sub(args, area);
            if action == "menu" { container_menu(ctx) } else { containers(ctx, action) }
        }
        "kubernetes" | "k8s" | "cluster" => {
            let action = sub(args, area);
            if action == "menu" { kubernetes_menu(ctx) } else { kubernetes(ctx, action) }
        }
        "menu" => menu(ctx),
        _ => Err("native admite network, hardware, power, security, tools, containers, kubernetes o menu".into()),
    }
}
fn first(args: &[String]) -> Option<&str> {
    args.iter()
        .map(String::as_str)
        .find(|v| !v.starts_with('-'))
}
fn sub<'a>(args: &'a [String], area: &str) -> &'a str {
    args.iter()
        .position(|v| v == area)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .filter(|v| !v.starts_with('-'))
        .unwrap_or("status")
}
fn network(ctx: &Context, action: &str) -> Result<(), String> {
    match action {
        "status" | "overview" => {
            if !offer(ctx, "Get-NetIPConfiguration") {
                return Err("no se pudo preparar la consulta de red".into());
            }
            powershell("Get-NetIPConfiguration; Get-NetRoute -AddressFamily IPv4 | Format-Table -AutoSize; Get-DnsClientServerAddress | Format-Table -AutoSize; Get-NetTCPConnection -State Listen | Sort-Object LocalPort | Format-Table -AutoSize")
        }
        "flush-dns" | "dns-flush" => {
            if !offer(ctx, "ipconfig") {
                return Err("ipconfig no está disponible".into());
            }
            if !ctx.dry_run && !ask("¿Vaciar la caché DNS de Windows?") {
                let _ = record_native(
                    ctx,
                    "flush-dns",
                    "ipconfig",
                    "cancelled",
                    &["/flushdns".into()],
                );
                println!("Operación cancelada.");
                return Ok(());
            }
            let command = vec!["/flushdns".into()];
            let ok = run_with_sudo("ipconfig", &command, ctx.dry_run).map_err(|e| e.to_string())?;
            record_native(
                ctx,
                "flush-dns",
                "ipconfig",
                if ctx.dry_run {
                    "planned"
                } else if ok {
                    "executed"
                } else {
                    "failed"
                },
                &command,
            )?;
            if ok {
                if ctx.dry_run {
                    println!("Simulación: se vaciaría la caché DNS; no se modificó el sistema.");
                }
                Ok(())
            } else {
                Err("ipconfig /flushdns devolvió un error".into())
            }
        }
        _ => Err("network admite status u flush-dns".into()),
    }
}
fn hardware(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("hardware admite status".into());
    }
    if !offer(ctx, "Get-CimInstance") {
        return Err("PowerShell/CIM no está disponible".into());
    }
    powershell("Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,OSArchitecture,LastBootUpTime | Format-List; Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors | Format-Table -AutoSize; Get-CimInstance Win32_ComputerSystem | Select-Object TotalPhysicalMemory | Format-List; Get-CimInstance Win32_VideoController | Select-Object Name,AdapterRAM,DriverVersion | Format-Table -AutoSize")
}
fn power(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" && action != "plans" {
        return Err("power admite status o plans".into());
    }
    if !offer(ctx, "powercfg") {
        return Err("powercfg no está disponible".into());
    }
    native(
        "powercfg",
        if action == "plans" {
            &["/list"]
        } else {
            &["/getactivescheme"]
        },
    )
}
fn security(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("security admite status".into());
    }
    if !offer(ctx, "Get-NetFirewallProfile") {
        return Err("Get-NetFirewallProfile no está disponible".into());
    }
    powershell("Get-NetFirewallProfile | Select-Object Name,Enabled,DefaultInboundAction,DefaultOutboundAction | Format-Table -AutoSize; Get-MpComputerStatus | Select-Object AMServiceEnabled,AntivirusEnabled,RealTimeProtectionEnabled | Format-List")
}

fn tools(ctx: &Context, action: &str, args: &[String]) -> Result<(), String> {
    match action {
        "menu" => tools_menu(ctx),
        "ssh" | "remote" => ssh_menu(ctx),
        "adb" | "android" => adb_menu(ctx),
        "containers" | "container" => container_menu(ctx),
        "images" | "image" => container_images_menu(ctx),
        "volumes" | "volume" => container_resource_menu(
            ctx,
            "Volúmenes",
            "volume-list",
            "volume-inspect",
            "volume-create",
            "volume-remove",
            "volume-prune",
        ),
        "networks" | "network" => container_resource_menu(
            ctx,
            "Redes",
            "network-list",
            "network-inspect",
            "network-create",
            "network-remove",
            "network-prune",
        ),
        "compose" => container_compose_menu(ctx),
        "kubernetes" | "k8s" => kubernetes_menu(ctx),
        "adb-status" => {
            if !offer(ctx, "adb") {
                return Err("ADB no está disponible".into());
            }
            run_native_command(
                ctx,
                "adb-status",
                "adb",
                &["devices".into(), "-l".into()],
                false,
                "",
            )
        }
        "container-list" => {
            let engine = if command_exists("docker") {
                "docker"
            } else {
                "podman"
            };
            if !command_exists(engine) {
                return Err("Docker y Podman no están disponibles".into());
            }
            run_native_command(
                ctx,
                "container-list",
                engine,
                &["ps".into(), "-a".into()],
                false,
                "",
            )
        }
        "kubernetes-contexts" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            run_native_command(
                ctx,
                "kubernetes-contexts",
                "kubectl",
                &["config".into(), "get-contexts".into()],
                false,
                "",
            )
        }
        "ssh-connect"
        | "scp-copy"
        | "sftp"
        | "adb-shell"
        | "adb-install"
        | "adb-push"
        | "adb-pull"
        | "adb-reboot"
        | "container-pull"
        | "container-run"
        | "container-start"
        | "container-stop"
        | "container-restart"
        | "container-remove"
        | "container-logs"
        | "container-exec"
        | "container-inspect"
        | "container-stats"
        | "container-top"
        | "container-port"
        | "container-diff"
        | "container-pause"
        | "container-unpause"
        | "container-kill"
        | "container-rename"
        | "container-cp"
        | "container-prune"
        | "image-inspect"
        | "image-history"
        | "image-build"
        | "image-tag"
        | "image-remove"
        | "image-prune"
        | "volume-list"
        | "volume-inspect"
        | "volume-create"
        | "volume-remove"
        | "volume-prune"
        | "network-list"
        | "network-inspect"
        | "network-create"
        | "network-remove"
        | "network-prune"
        | "system-info"
        | "system-df"
        | "system-prune"
        | "container-compose"
        | "kubernetes-apply"
        | "kubernetes-delete"
        | "kubernetes-scale"
        | "kubernetes-rollout"
        | "kubernetes-port-forward" => tools_direct(ctx, action, args),
        "status" | "list" | "overview" => tools_status(),
        "install" | "install-dependency" => tools_install(ctx, args),
        _ => Err("tools admite status, install, menu, ssh, adb, containers, images, volumes, networks, compose o kubernetes".into()),
    }
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|value| value == name)
        .and_then(|index| args.get(index + 1))
        .filter(|value| !value.starts_with('-'))
        .cloned()
}

fn required_option(args: &[String], name: &str) -> Result<String, String> {
    option_value(args, name).ok_or_else(|| format!("falta el argumento {name}"))
}

fn direct_engine(args: &[String]) -> Result<String, String> {
    let engine = option_value(args, "--engine").unwrap_or_else(|| {
        if command_exists("docker") {
            "docker".into()
        } else {
            "podman".into()
        }
    });
    if !matches!(engine.as_str(), "docker" | "podman") {
        return Err("motor no válido; usa docker o podman".into());
    }
    if !command_exists(&engine) {
        return Err(format!("{engine} no está instalado"));
    }
    Ok(engine)
}

fn direct_compose_engine(args: &[String]) -> Result<String, String> {
    if let Some(engine) = option_value(args, "--engine") {
        if !matches!(
            engine.as_str(),
            "docker" | "podman" | "docker-compose" | "podman-compose"
        ) {
            return Err("motor Compose no válido".into());
        }
        if !command_exists(&engine) {
            return Err(format!("{engine} no está instalado"));
        }
        return Ok(engine);
    }
    for candidate in ["docker", "podman", "docker-compose", "podman-compose"] {
        if command_exists(candidate) {
            return Ok(candidate.into());
        }
    }
    Err("no se encontró Docker, Podman, docker-compose ni podman-compose".into())
}

fn tools_direct(ctx: &Context, action: &str, args: &[String]) -> Result<(), String> {
    match action {
        "ssh-connect" => {
            if !offer(ctx, "ssh") {
                return Err("OpenSSH no está disponible".into());
            }
            let mut command = Vec::new();
            if let Some(port) = option_value(args, "--port") {
                command.extend(["-p".into(), port]);
            }
            if let Some(identity) = option_value(args, "--identity") {
                command.extend(["-i".into(), identity]);
            }
            command.push(required_option(args, "--target")?);
            if let Some(remote) = option_value(args, "--command") {
                command.push(remote);
            }
            run_native_command(
                ctx,
                action,
                "ssh",
                &command,
                true,
                "¿Abrir esta conexión SSH?",
            )
        }
        "scp-copy" => {
            if !offer(ctx, "scp") {
                return Err("scp no está disponible".into());
            }
            let mut command = Vec::new();
            if args.iter().any(|value| value == "--recursive") {
                command.push("-r".into());
            }
            command.extend([
                required_option(args, "--source")?,
                required_option(args, "--destination")?,
            ]);
            run_native_command(
                ctx,
                action,
                "scp",
                &command,
                true,
                "¿Ejecutar esta copia SCP?",
            )
        }
        "sftp" => {
            if !offer(ctx, "sftp") {
                return Err("sftp no está disponible".into());
            }
            run_native_command(
                ctx,
                action,
                "sftp",
                &[required_option(args, "--target")?],
                true,
                "¿Abrir esta sesión SFTP?",
            )
        }
        "adb-shell" => {
            if !offer(ctx, "adb") {
                return Err("ADB no está disponible".into());
            }
            let mut command = vec!["shell".into()];
            if let Some(serial) = option_value(args, "--serial") {
                command.extend(["-s".into(), serial]);
            }
            command.push(required_option(args, "--command")?);
            run_native_command(ctx, action, "adb", &command, true, "¿Abrir esta shell ADB?")
        }
        "adb-install" => {
            if !offer(ctx, "adb") {
                return Err("ADB no está disponible".into());
            }
            let mut command = vec!["install".into()];
            if let Some(serial) = option_value(args, "--serial") {
                command.extend(["-s".into(), serial]);
            }
            command.push(required_option(args, "--apk")?);
            run_native_command(ctx, action, "adb", &command, true, "¿Instalar este APK?")
        }
        "adb-push" | "adb-pull" => {
            if !offer(ctx, "adb") {
                return Err("ADB no está disponible".into());
            }
            let mut command = vec![action.strip_prefix("adb-").unwrap().to_owned()];
            if let Some(serial) = option_value(args, "--serial") {
                command.extend(["-s".into(), serial]);
            }
            command.extend([
                required_option(args, "--source")?,
                required_option(args, "--destination")?,
            ]);
            run_native_command(
                ctx,
                action,
                "adb",
                &command,
                true,
                "¿Ejecutar esta transferencia ADB?",
            )
        }
        "adb-reboot" => {
            if !offer(ctx, "adb") {
                return Err("ADB no está disponible".into());
            }
            let mut command = vec!["reboot".into()];
            if let Some(serial) = option_value(args, "--serial") {
                command.extend(["-s".into(), serial]);
            }
            if let Some(mode) = option_value(args, "--mode") {
                command.push(mode);
            }
            run_native_command(
                ctx,
                action,
                "adb",
                &command,
                true,
                "¿Reiniciar el dispositivo Android?",
            )
        }
        "container-pull" => {
            let engine = direct_engine(args)?;
            run_native_command(
                ctx,
                action,
                &engine,
                &["pull".into(), required_option(args, "--image")?],
                true,
                "¿Descargar esta imagen?",
            )
        }
        "container-run" => {
            let engine = direct_engine(args)?;
            let mut command = vec!["run".into(), "-d".into()];
            if let Some(name) = option_value(args, "--name") {
                command.extend(["--name".into(), name]);
            }
            command.push(required_option(args, "--image")?);
            if let Some(start) = option_value(args, "--command") {
                command.push(start);
            }
            run_native_command(
                ctx,
                action,
                &engine,
                &command,
                true,
                "¿Crear y ejecutar este contenedor?",
            )
        }
        "container-start" | "container-stop" | "container-restart" | "container-remove" => {
            let engine = direct_engine(args)?;
            let operation = match action {
                "container-remove" => "rm",
                other => other.strip_prefix("container-").unwrap_or(other),
            };
            run_native_command(
                ctx,
                action,
                &engine,
                &[operation.into(), required_option(args, "--name")?],
                true,
                "¿Aplicar esta acción al contenedor?",
            )
        }
        "container-logs" => {
            let engine = direct_engine(args)?;
            run_native_command(
                ctx,
                action,
                &engine,
                &[
                    "logs".into(),
                    "--tail".into(),
                    "200".into(),
                    required_option(args, "--name")?,
                ],
                false,
                "",
            )
        }
        "container-exec" => {
            let engine = direct_engine(args)?;
            run_native_command(
                ctx,
                action,
                &engine,
                &[
                    "exec".into(),
                    "-i".into(),
                    required_option(args, "--name")?,
                    required_option(args, "--command")?,
                ],
                true,
                "¿Ejecutar este comando en el contenedor?",
            )
        }
        "container-inspect" | "container-stats" | "container-top" | "container-port"
        | "container-diff" => {
            let engine = direct_engine(args)?;
            let operation = action.strip_prefix("container-").unwrap();
            let mut command = vec![operation.into()];
            if action == "container-stats" {
                command.push("--no-stream".into());
            }
            command.push(required_option(args, "--name")?);
            run_native_command(ctx, action, &engine, &command, false, "")
        }
        "container-pause" | "container-unpause" | "container-kill" => {
            let engine = direct_engine(args)?;
            let operation = action.strip_prefix("container-").unwrap();
            run_native_command(
                ctx,
                action,
                &engine,
                &[operation.into(), required_option(args, "--name")?],
                true,
                "¿Aplicar esta acción al contenedor?",
            )
        }
        "container-rename" => {
            let engine = direct_engine(args)?;
            run_native_command(
                ctx,
                action,
                &engine,
                &[
                    "rename".into(),
                    required_option(args, "--name")?,
                    required_option(args, "--new-name")?,
                ],
                true,
                "¿Renombrar este contenedor?",
            )
        }
        "container-cp" => {
            let engine = direct_engine(args)?;
            run_native_command(
                ctx,
                action,
                &engine,
                &[
                    "cp".into(),
                    required_option(args, "--source")?,
                    required_option(args, "--destination")?,
                ],
                true,
                "¿Copiar estos archivos?",
            )
        }
        "container-prune" => {
            let engine = direct_engine(args)?;
            run_native_command(
                ctx,
                action,
                &engine,
                &["container".into(), "prune".into()],
                true,
                "¿Eliminar todos los contenedores detenidos?",
            )
        }
        "image-inspect" | "image-history" => {
            let engine = direct_engine(args)?;
            let operation = if action == "image-inspect" {
                "image"
            } else {
                "history"
            };
            let mut command = if operation == "image" {
                vec!["image".into(), "inspect".into()]
            } else {
                vec!["history".into()]
            };
            command.push(required_option(args, "--image")?);
            run_native_command(ctx, action, &engine, &command, false, "")
        }
        "image-build" => {
            let engine = direct_engine(args)?;
            let mut command = vec!["build".into()];
            if let Some(tag) = option_value(args, "--tag") {
                command.extend(["-t".into(), tag]);
            }
            command.push(required_option(args, "--path")?);
            run_native_command(
                ctx,
                action,
                &engine,
                &command,
                true,
                "¿Construir esta imagen?",
            )
        }
        "image-tag" => {
            let engine = direct_engine(args)?;
            run_native_command(
                ctx,
                action,
                &engine,
                &[
                    "tag".into(),
                    required_option(args, "--image")?,
                    required_option(args, "--tag")?,
                ],
                true,
                "¿Etiquetar esta imagen?",
            )
        }
        "image-remove" => {
            let engine = direct_engine(args)?;
            let mut command = vec!["rmi".into()];
            if args.iter().any(|value| value == "--force") {
                command.push("--force".into());
            }
            command.push(required_option(args, "--image")?);
            run_native_command(
                ctx,
                action,
                &engine,
                &command,
                true,
                "¿Eliminar esta imagen?",
            )
        }
        "image-prune" => {
            let engine = direct_engine(args)?;
            let mut command = vec!["image".into(), "prune".into()];
            if args.iter().any(|value| value == "--all") {
                command.push("--all".into());
            }
            run_native_command(
                ctx,
                action,
                &engine,
                &command,
                true,
                "¿Limpiar imágenes no utilizadas?",
            )
        }
        "volume-list" | "volume-prune" => {
            let engine = direct_engine(args)?;
            let operation = if action == "volume-list" {
                "ls"
            } else {
                "prune"
            };
            run_native_command(
                ctx,
                action,
                &engine,
                &["volume".into(), operation.into()],
                action == "volume-prune",
                if action == "volume-prune" {
                    "¿Eliminar volúmenes no utilizados?"
                } else {
                    ""
                },
            )
        }
        "volume-inspect" | "volume-create" | "volume-remove" => {
            let engine = direct_engine(args)?;
            let operation = match action {
                "volume-inspect" => "inspect",
                "volume-create" => "create",
                _ => "rm",
            };
            let mut command = vec!["volume".into(), operation.into()];
            command.push(required_option(args, "--name")?);
            run_native_command(
                ctx,
                action,
                &engine,
                &command,
                action != "volume-inspect",
                if action == "volume-inspect" {
                    ""
                } else if action == "volume-create" {
                    "¿Crear este volumen?"
                } else {
                    "¿Eliminar este volumen?"
                },
            )
        }
        "network-list" | "network-prune" => {
            let engine = direct_engine(args)?;
            let operation = if action == "network-list" {
                "ls"
            } else {
                "prune"
            };
            run_native_command(
                ctx,
                action,
                &engine,
                &["network".into(), operation.into()],
                action == "network-prune",
                if action == "network-prune" {
                    "¿Eliminar redes no utilizadas?"
                } else {
                    ""
                },
            )
        }
        "network-inspect" | "network-create" | "network-remove" => {
            let engine = direct_engine(args)?;
            let operation = match action {
                "network-inspect" => "inspect",
                "network-create" => "create",
                _ => "rm",
            };
            let mut command = vec!["network".into(), operation.into()];
            command.push(required_option(args, "--name")?);
            run_native_command(
                ctx,
                action,
                &engine,
                &command,
                action != "network-inspect",
                if action == "network-inspect" {
                    ""
                } else if action == "network-create" {
                    "¿Crear esta red?"
                } else {
                    "¿Eliminar esta red?"
                },
            )
        }
        "system-info" | "system-df" | "system-prune" => {
            let engine = direct_engine(args)?;
            let command = match action {
                "system-info" => vec!["info".into()],
                "system-df" => vec!["system".into(), "df".into()],
                _ => vec!["system".into(), "prune".into()],
            };
            run_native_command(
                ctx,
                action,
                &engine,
                &command,
                action == "system-prune",
                if action == "system-prune" {
                    "¿Limpiar recursos no utilizados del motor?"
                } else {
                    ""
                },
            )
        }
        "container-compose" => {
            let operation = required_option(args, "--operation")?;
            if !matches!(
                operation.as_str(),
                "up" | "down"
                    | "start"
                    | "stop"
                    | "restart"
                    | "ps"
                    | "logs"
                    | "pull"
                    | "build"
                    | "config"
                    | "images"
                    | "top"
                    | "run"
                    | "exec"
                    | "rm"
                    | "pause"
                    | "unpause"
            ) {
                return Err("operación Compose no reconocida".into());
            }
            let engine = direct_compose_engine(args)?;
            let standalone = matches!(engine.as_str(), "docker-compose" | "podman-compose");
            let mut command = if standalone {
                Vec::new()
            } else {
                vec!["compose".into()]
            };
            if let Some(file) = option_value(args, "--file") {
                command.extend(["-f".into(), file]);
            }
            command.push(operation.clone());
            if operation == "up" && !args.iter().any(|value| value == "--foreground") {
                command.push("-d".into());
            }
            if operation == "down" && args.iter().any(|value| value == "--volumes") {
                command.push("--volumes".into());
            }
            if matches!(operation.as_str(), "logs" | "ps" | "images" | "top") {
                if let Some(service) = option_value(args, "--service") {
                    command.push(service);
                }
            }
            if matches!(operation.as_str(), "run" | "exec") {
                command.push(required_option(args, "--service")?);
                if let Some(command_text) = option_value(args, "--command") {
                    command.push(command_text);
                }
            }
            run_native_command(
                ctx,
                action,
                &engine,
                &command,
                if matches!(
                    operation.as_str(),
                    "ps" | "logs" | "config" | "images" | "top"
                ) {
                    false
                } else {
                    true
                },
                if matches!(
                    operation.as_str(),
                    "ps" | "logs" | "config" | "images" | "top"
                ) {
                    ""
                } else {
                    "¿Ejecutar esta operación Compose?"
                },
            )
        }
        "kubernetes-apply" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            run_native_command(
                ctx,
                action,
                "kubectl",
                &[
                    "apply".into(),
                    "-f".into(),
                    required_option(args, "--file")?,
                ],
                true,
                "¿Aplicar este manifiesto al clúster?",
            )
        }
        "kubernetes-delete" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            run_native_command(
                ctx,
                action,
                "kubectl",
                &["delete".into(), required_option(args, "--resource")?],
                true,
                "¿Eliminar este recurso del clúster?",
            )
        }
        "kubernetes-scale" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            run_native_command(
                ctx,
                action,
                "kubectl",
                &[
                    "scale".into(),
                    "deployment".into(),
                    required_option(args, "--deployment")?,
                    format!("--replicas={}", required_option(args, "--replicas")?),
                ],
                true,
                "¿Cambiar el número de réplicas?",
            )
        }
        "kubernetes-rollout" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            run_native_command(
                ctx,
                action,
                "kubectl",
                &[
                    "rollout".into(),
                    "restart".into(),
                    format!("deployment/{}", required_option(args, "--deployment")?),
                ],
                true,
                "¿Reiniciar este rollout?",
            )
        }
        "kubernetes-port-forward" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            run_native_command(
                ctx,
                action,
                "kubectl",
                &[
                    "port-forward".into(),
                    required_option(args, "--target")?,
                    required_option(args, "--ports")?,
                ],
                true,
                "¿Abrir este port-forward?",
            )
        }
        _ => Err(format!("acción nativa no soportada: {action}")),
    }
}

fn tools_status() -> Result<(), String> {
    println!("=== Herramientas del anfitrión Windows ===");
    println!("La consulta recorre el catálogo completo, no instala nada ni inicia servicios.");
    for tool in crate::platform::host_tools() {
        let state = if crate::platform::host_tool_available(tool) {
            "disponible"
        } else if tool.installable && !tool.install_package.is_empty() {
            "ausente (instalable desde LTools)"
        } else {
            "ausente (componente del sistema o instalación manual)"
        };
        let version = crate::platform::host_tool_version(tool)
            .map(|value| format!(" | {}", value.trim()))
            .unwrap_or_default();
        println!("{:<22} {:<45}{}", tool.id, state, version);
    }
    println!("\nPara instalar una dependencia concreta: ltools native tools install --tool <id>");
    println!("La instalación siempre pide confirmación y muestra el proveedor elegido.");
    Ok(())
}

fn tools_install(ctx: &Context, args: &[String]) -> Result<(), String> {
    let requested = option_value(args, "--tool")
        .or_else(|| option_value(args, "--id"))
        .or_else(|| {
            args.iter()
                .skip_while(|value| {
                    value.as_str() != "install" && value.as_str() != "install-dependency"
                })
                .nth(1)
                .filter(|value| !value.starts_with('-'))
                .cloned()
        });
    let id = if let Some(id) = requested {
        id
    } else {
        println!("=== Instalar dependencia Windows ===");
        println!("Selecciona una herramienta ausente. No se instala nada sin confirmación.");
        for tool in crate::platform::host_tools().iter().filter(|tool| {
            !crate::platform::host_tool_available(tool)
                && tool.installable
                && !tool.install_package.is_empty()
        }) {
            println!(
                "  {:<18} paquete: {:<24} función: {}",
                tool.id, tool.install_package, tool.feature
            );
        }
        crate::menu_input("Identificador de la dependencia (vacío para cancelar): ")
            .unwrap_or_default()
    };
    if id.trim().is_empty() {
        println!("Instalación cancelada; no se modifica el sistema.");
        return Ok(());
    }
    let tool = crate::platform::host_tools()
        .iter()
        .find(|tool| tool.id == id.trim())
        .ok_or_else(|| format!("dependencia no catalogada: {}", id.trim()))?;
    if crate::platform::host_tool_available(tool) {
        println!("Ya está disponible: {}", id.trim());
        return Ok(());
    }
    if !crate::platform::install_tool(id.trim(), ctx.dry_run)? {
        return Err(format!(
            "No se pudo instalar {}. Revisa winget, Chocolatey o Scoop y vuelve a intentarlo.",
            id.trim()
        ));
    }
    println!("Dependencia preparada: {}", id.trim());
    Ok(())
}

fn containers(ctx: &Context, action: &str) -> Result<(), String> {
    if !matches!(action, "status" | "overview") {
        return Err("containers admite status".into());
    }
    println!("=== Contenedores Windows (solo lectura) ===");
    if offer(ctx, "docker") {
        best_effort("docker", &["version"]);
        best_effort("docker", &["info"]);
        if command_exists("docker-compose") {
            best_effort("docker-compose", &["version"]);
        } else if command_exists("docker") {
            best_effort("docker", &["compose", "version"]);
        }
    } else {
        println!("Docker no está disponible; se conserva el sistema sin cambios.");
    }
    if command_exists("podman") {
        println!("\n=== Podman alternativo ===");
        best_effort("podman", &["version"]);
        best_effort("podman", &["info"]);
    }
    println!("No se crean, eliminan ni inician contenedores desde esta consulta.");
    Ok(())
}

fn kubernetes(ctx: &Context, action: &str) -> Result<(), String> {
    if !matches!(action, "status" | "overview") {
        return Err("kubernetes admite status".into());
    }
    println!("=== Kubernetes Windows (solo lectura) ===");
    if !offer(ctx, "kubectl") {
        println!("kubectl no está disponible; no se consulta ningún clúster.");
        return Ok(());
    }
    best_effort("kubectl", &["version", "--client=true"]);
    best_effort("kubectl", &["config", "current-context"]);
    best_effort("kubectl", &["config", "get-contexts"]);
    best_effort("kubectl", &["cluster-info", "--request-timeout=3s"]);
    best_effort("kubectl", &["get", "nodes", "--request-timeout=3s"]);
    println!("Usa `native tools menu` para abrir los flujos operativos.");
    Ok(())
}

fn tools_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Herramientas operativas Windows ===");
        println!("  1) SSH / SCP / SFTP\n  2) Android (ADB)\n  3) Docker / Podman / Compose\n  4) Kubernetes\n  q) Volver");
        let answer = crate::menu_input("Elige una opción: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => ssh_menu(ctx),
            "2" => adb_menu(ctx),
            "3" => container_menu(ctx),
            "4" => kubernetes_menu(ctx),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn ssh_menu(ctx: &Context) -> Result<(), String> {
    if !offer(ctx, "ssh") {
        return Err("OpenSSH no está disponible".into());
    }
    loop {
        crate::clear_screen();
        println!("=== SSH / SCP / SFTP ===");
        println!("  1) Conectar por SSH\n  2) Copiar con SCP\n  3) Abrir SFTP\n  4) Comprobar versión\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => ssh_connect(ctx),
            "2" => scp_copy(ctx),
            "3" => sftp_open(ctx),
            "4" => {
                best_effort("ssh", &["-V"]);
                Ok(())
            }
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn remote_target() -> Option<String> {
    let host = crate::menu_input("Host o IP: ")?;
    if host.is_empty() {
        return None;
    }
    let user = crate::menu_input("Usuario (vacío para el actual): ")?;
    Some(if user.is_empty() {
        host
    } else {
        format!("{user}@{host}")
    })
}

fn ssh_connect(ctx: &Context) -> Result<(), String> {
    let target = remote_target().ok_or("se requiere un host")?;
    let port = crate::menu_input("Puerto (vacío para 22): ").unwrap_or_default();
    let mut args = Vec::new();
    if !port.is_empty() {
        args.extend(["-p".into(), port]);
    }
    args.push(target);
    run_native_command(
        ctx,
        "ssh-connect",
        "ssh",
        &args,
        true,
        "¿Abrir esta conexión SSH?",
    )
}

fn scp_copy(ctx: &Context) -> Result<(), String> {
    if !offer(ctx, "scp") {
        return Err("scp no está disponible".into());
    }
    let source = crate::menu_input("Origen local o remoto: ").ok_or("falta el origen")?;
    let destination = crate::menu_input("Destino local o remoto: ").ok_or("falta el destino")?;
    if source.is_empty() || destination.is_empty() {
        return Err("origen y destino son obligatorios".into());
    }
    let recursive =
        crate::menu_yes_no("¿Copiar directorios recursivamente? [y/N] ", false).unwrap_or(false);
    let mut args = Vec::new();
    if recursive {
        args.push("-r".into());
    }
    args.extend([source, destination]);
    run_native_command(
        ctx,
        "scp-copy",
        "scp",
        &args,
        true,
        "¿Ejecutar esta copia SCP?",
    )
}

fn sftp_open(ctx: &Context) -> Result<(), String> {
    if !offer(ctx, "sftp") {
        return Err("sftp no está disponible".into());
    }
    let target = remote_target().ok_or("se requiere un host")?;
    run_native_command(
        ctx,
        "sftp",
        "sftp",
        &[target],
        true,
        "¿Abrir esta sesión SFTP?",
    )
}

fn adb_menu(ctx: &Context) -> Result<(), String> {
    if !offer(ctx, "adb") {
        return Err("ADB no está disponible; instala Android Platform Tools".into());
    }
    loop {
        crate::clear_screen();
        println!("=== Android Debug Bridge ===");
        println!("  1) Dispositivos\n  2) Shell remoto\n  3) Instalar APK\n  4) Enviar archivo\n  5) Extraer archivo\n  6) Reiniciar dispositivo\n  7) Logcat\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => {
                best_effort("adb", &["devices", "-l"]);
                Ok(())
            }
            "2" => adb_shell(ctx),
            "3" => adb_install(ctx),
            "4" => adb_transfer(ctx, false),
            "5" => adb_transfer(ctx, true),
            "6" => adb_reboot(ctx),
            "7" => {
                best_effort("adb", &["logcat", "-d", "-t", "200"]);
                Ok(())
            }
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn adb_device() -> String {
    crate::menu_input("ID del dispositivo (vacío si solo hay uno): ").unwrap_or_default()
}

fn adb_with_device(device: String, args: &mut Vec<String>) {
    if !device.is_empty() {
        args.splice(0..0, ["-s".into(), device]);
    }
}

fn adb_shell(ctx: &Context) -> Result<(), String> {
    let device = adb_device();
    let command = crate::menu_input("Comando shell: ").ok_or("falta el comando")?;
    if command.is_empty() {
        return Err("el comando shell es obligatorio".into());
    }
    let mut args = vec!["shell".into(), command];
    adb_with_device(device, &mut args);
    run_native_command(
        ctx,
        "adb-shell",
        "adb",
        &args,
        true,
        "¿Ejecutar este comando en Android?",
    )
}

fn adb_install(ctx: &Context) -> Result<(), String> {
    let apk = crate::menu_input("Ruta del APK: ").ok_or("falta el APK")?;
    if apk.is_empty() {
        return Err("la ruta del APK es obligatoria".into());
    }
    let device = adb_device();
    let mut args = vec!["install".into(), apk];
    adb_with_device(device, &mut args);
    run_native_command(
        ctx,
        "adb-install",
        "adb",
        &args,
        true,
        "¿Instalar este APK?",
    )
}

fn adb_transfer(ctx: &Context, pull: bool) -> Result<(), String> {
    let first = crate::menu_input(if pull {
        "Ruta remota: "
    } else {
        "Ruta local: "
    })
    .ok_or("falta la ruta")?;
    let second = crate::menu_input(if pull {
        "Destino local: "
    } else {
        "Destino remoto: "
    })
    .ok_or("falta el destino")?;
    let device = adb_device();
    let mut args = vec![if pull { "pull" } else { "push" }.into(), first, second];
    adb_with_device(device, &mut args);
    run_native_command(
        ctx,
        if pull { "adb-pull" } else { "adb-push" },
        "adb",
        &args,
        true,
        "¿Ejecutar esta transferencia ADB?",
    )
}

fn adb_reboot(ctx: &Context) -> Result<(), String> {
    let device = adb_device();
    let mode = crate::menu_input("Modo (Enter normal, bootloader, recovery): ").unwrap_or_default();
    let mut args = vec!["reboot".into()];
    if !mode.is_empty() {
        args.push(mode);
    }
    adb_with_device(device, &mut args);
    run_native_command(
        ctx,
        "adb-reboot",
        "adb",
        &args,
        true,
        "¿Reiniciar el dispositivo Android?",
    )
}

fn container_engine() -> Result<String, String> {
    let default = if command_exists("docker") {
        "docker"
    } else if command_exists("podman") {
        "podman"
    } else {
        "docker"
    };
    let engine = crate::menu_input(&format!("Motor (docker/podman; Enter para {default}): "))
        .unwrap_or_default();
    let engine = if engine.is_empty() {
        default.into()
    } else {
        engine
    };
    if !matches!(engine.as_str(), "docker" | "podman") {
        return Err("motor no válido; usa docker o podman".into());
    }
    if !command_exists(&engine) {
        return Err(format!("{engine} no está instalado"));
    }
    Ok(engine)
}

fn compose_engine() -> Result<String, String> {
    let default = if command_exists("docker") {
        "docker"
    } else if command_exists("podman") {
        "podman"
    } else if command_exists("docker-compose") {
        "docker-compose"
    } else {
        "podman-compose"
    };
    let engine = crate::menu_input(&format!(
        "Motor Compose (docker/podman/docker-compose/podman-compose; Enter para {default}): "
    ))
    .unwrap_or_default();
    let engine = if engine.is_empty() {
        default.into()
    } else {
        engine
    };
    if !matches!(
        engine.as_str(),
        "docker" | "podman" | "docker-compose" | "podman-compose"
    ) {
        return Err("motor Compose no válido".into());
    }
    if !command_exists(&engine) {
        return Err(format!("{engine} no está instalado"));
    }
    Ok(engine)
}

fn container_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Docker / Podman ===");
        println!("  1) Contenedores\n  2) Imágenes\n  3) Volúmenes\n  4) Redes\n  5) Docker Compose / Podman Compose\n  6) Diagnóstico y limpieza del motor\n q) Volver");
        let answer = crate::menu_input("Elige una categoría: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => container_objects_menu(ctx),
            "2" => container_images_menu(ctx),
            "3" => container_resource_menu(
                ctx,
                "Volúmenes",
                "volume-list",
                "volume-inspect",
                "volume-create",
                "volume-remove",
                "volume-prune",
            ),
            "4" => container_resource_menu(
                ctx,
                "Redes",
                "network-list",
                "network-inspect",
                "network-create",
                "network-remove",
                "network-prune",
            ),
            "5" => container_compose_menu(ctx),
            "6" => container_engine_menu(ctx),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn direct_container_action(
    ctx: &Context,
    action: &str,
    options: Vec<String>,
) -> Result<(), String> {
    let engine = container_engine()?;
    let mut args = vec!["--engine".into(), engine];
    args.extend(options);
    tools_direct(ctx, action, &args)
}

fn required_menu_value(prompt: &str) -> Result<String, String> {
    let value = crate::menu_input(prompt).ok_or("operación cancelada")?;
    if value.trim().is_empty() {
        Err("este campo es obligatorio".into())
    } else {
        Ok(value)
    }
}

fn container_objects_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Contenedores ===");
        println!("  1) Listar\n  2) Crear y ejecutar\n  3) Iniciar\n  4) Detener\n  5) Reiniciar\n  6) Eliminar\n  7) Logs\n  8) Ejecutar comando\n  9) Inspeccionar\n 10) Estadísticas\n 11) Procesos\n 12) Puertos\n 13) Cambios\n 14) Pausar\n 15) Reanudar\n 16) Terminar\n 17) Renombrar\n 18) Copiar archivos\n 19) Limpiar detenidos\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => container_status(ctx, false),
            "2" => container_run(ctx),
            "3" => container_simple(ctx, "start"),
            "4" => container_simple(ctx, "stop"),
            "5" => container_simple(ctx, "restart"),
            "6" => container_simple(ctx, "rm"),
            "7" => container_logs(ctx),
            "8" => container_exec(ctx),
            "9" => container_named_action(ctx, "container-inspect"),
            "10" => container_named_action(ctx, "container-stats"),
            "11" => container_named_action(ctx, "container-top"),
            "12" => container_named_action(ctx, "container-port"),
            "13" => container_named_action(ctx, "container-diff"),
            "14" => container_named_action(ctx, "container-pause"),
            "15" => container_named_action(ctx, "container-unpause"),
            "16" => container_named_action(ctx, "container-kill"),
            "17" => {
                let old = required_menu_value("Contenedor actual: ")?;
                let new_name = required_menu_value("Nuevo nombre: ")?;
                direct_container_action(
                    ctx,
                    "container-rename",
                    vec!["--name".into(), old, "--new-name".into(), new_name],
                )
            }
            "18" => {
                let source = required_menu_value("Origen: ")?;
                let destination = required_menu_value("Destino: ")?;
                direct_container_action(
                    ctx,
                    "container-cp",
                    vec![
                        "--source".into(),
                        source,
                        "--destination".into(),
                        destination,
                    ],
                )
            }
            "19" => direct_container_action(ctx, "container-prune", Vec::new()),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn container_named_action(ctx: &Context, action: &str) -> Result<(), String> {
    let name = required_menu_value("Contenedor: ")?;
    direct_container_action(ctx, action, vec!["--name".into(), name])
}

fn container_images_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Imágenes ===");
        println!("  1) Listar\n  2) Descargar\n  3) Inspeccionar\n  4) Historial\n  5) Construir\n  6) Etiquetar\n  7) Eliminar\n  8) Limpiar no utilizadas\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => container_status(ctx, true),
            "2" => container_image(ctx),
            "3" => image_named_action(ctx, "image-inspect"),
            "4" => image_named_action(ctx, "image-history"),
            "5" => {
                let path = required_menu_value("Directorio de construcción: ")?;
                let tag = crate::menu_input("Etiqueta opcional: ").unwrap_or_default();
                let mut options = vec!["--path".into(), path];
                if !tag.is_empty() {
                    options.extend(["--tag".into(), tag]);
                }
                direct_container_action(ctx, "image-build", options)
            }
            "6" => {
                let image = required_menu_value("Imagen origen: ")?;
                let tag = required_menu_value("Nueva etiqueta: ")?;
                direct_container_action(
                    ctx,
                    "image-tag",
                    vec!["--image".into(), image, "--tag".into(), tag],
                )
            }
            "7" => image_named_action(ctx, "image-remove"),
            "8" => direct_container_action(ctx, "image-prune", Vec::new()),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn image_named_action(ctx: &Context, action: &str) -> Result<(), String> {
    let image = required_menu_value("Imagen: ")?;
    direct_container_action(ctx, action, vec!["--image".into(), image])
}

fn container_resource_menu(
    ctx: &Context,
    title: &str,
    list: &str,
    inspect: &str,
    create: &str,
    remove: &str,
    prune: &str,
) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== {title} ===");
        println!("  1) Listar\n  2) Inspeccionar\n  3) Crear\n  4) Eliminar\n  5) Limpiar no utilizados\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => direct_container_action(ctx, list, Vec::new()),
            "2" | "3" | "4" => {
                let name = required_menu_value("Nombre: ")?;
                let action = match answer.as_str() {
                    "2" => inspect,
                    "3" => create,
                    _ => remove,
                };
                direct_container_action(ctx, action, vec!["--name".into(), name])
            }
            "5" => direct_container_action(ctx, prune, Vec::new()),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn container_compose_menu(ctx: &Context) -> Result<(), String> {
    let engine = compose_engine()?;
    let operation = required_menu_value("Operación Compose (up/down/start/stop/restart/ps/logs/pull/build/config/images/top/run/exec/rm/pause/unpause): ")?;
    if !matches!(
        operation.as_str(),
        "up" | "down"
            | "start"
            | "stop"
            | "restart"
            | "ps"
            | "logs"
            | "pull"
            | "build"
            | "config"
            | "images"
            | "top"
            | "run"
            | "exec"
            | "rm"
            | "pause"
            | "unpause"
    ) {
        return Err("operación Compose no reconocida".into());
    }
    let file = crate::menu_input("Fichero compose (vacío para el actual): ").unwrap_or_default();
    let service = crate::menu_input("Servicio opcional: ").unwrap_or_default();
    let command = crate::menu_input("Comando opcional para run/exec: ").unwrap_or_default();
    let foreground =
        crate::menu_yes_no("¿Mantener Compose en primer plano? [y/N] ", false).unwrap_or(false);
    let volumes =
        crate::menu_yes_no("¿Incluir volúmenes al hacer down? [y/N] ", false).unwrap_or(false);
    let mut options = vec!["--operation".into(), operation];
    if !file.is_empty() {
        options.extend(["--file".into(), file]);
    }
    if !service.is_empty() {
        options.extend(["--service".into(), service]);
    }
    if !command.is_empty() {
        options.extend(["--command".into(), command]);
    }
    if foreground {
        options.push("--foreground".into());
    }
    if volumes {
        options.push("--volumes".into());
    }
    let mut args = vec!["--engine".into(), engine];
    args.extend(options);
    tools_direct(ctx, "container-compose", &args)
}

fn container_engine_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Diagnóstico y limpieza del motor ===");
        println!("  1) Información del motor\n  2) Uso de espacio\n  3) Limpiar contenedores detenidos\n  4) Limpiar imágenes\n  5) Limpiar recursos globales\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => direct_container_action(ctx, "system-info", Vec::new()),
            "2" => direct_container_action(ctx, "system-df", Vec::new()),
            "3" => direct_container_action(ctx, "container-prune", Vec::new()),
            "4" => direct_container_action(ctx, "image-prune", Vec::new()),
            "5" => direct_container_action(ctx, "system-prune", Vec::new()),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn container_status(ctx: &Context, images: bool) -> Result<(), String> {
    let engine = container_engine()?;
    let args = if images {
        vec!["images".into()]
    } else {
        vec!["ps".into(), "-a".into()]
    };
    run_native_command(ctx, "container-status", &engine, &args, false, "")
}

fn container_image(ctx: &Context) -> Result<(), String> {
    let engine = container_engine()?;
    let image = crate::menu_input("Imagen (ej. alpine:latest): ").ok_or("falta la imagen")?;
    run_native_command(
        ctx,
        "container-pull",
        &engine,
        &["pull".into(), image],
        true,
        "¿Descargar esta imagen?",
    )
}

fn container_run(ctx: &Context) -> Result<(), String> {
    let engine = container_engine()?;
    let image = crate::menu_input("Imagen: ").ok_or("falta la imagen")?;
    let name = crate::menu_input("Nombre opcional: ").unwrap_or_default();
    let command = crate::menu_input("Comando opcional: ").unwrap_or_default();
    let mut args = vec!["run".into(), "-d".into()];
    if !name.is_empty() {
        args.extend(["--name".into(), name]);
    }
    args.push(image);
    if !command.is_empty() {
        args.push(command);
    }
    run_native_command(
        ctx,
        "container-run",
        &engine,
        &args,
        true,
        "¿Crear y ejecutar este contenedor?",
    )
}

fn container_simple(ctx: &Context, operation: &str) -> Result<(), String> {
    let engine = container_engine()?;
    let name = crate::menu_input("Contenedor: ").ok_or("falta el contenedor")?;
    run_native_command(
        ctx,
        &format!("container-{operation}"),
        &engine,
        &[operation.into(), name],
        true,
        "¿Aplicar esta acción al contenedor?",
    )
}

fn container_logs(ctx: &Context) -> Result<(), String> {
    let engine = container_engine()?;
    let name = crate::menu_input("Contenedor: ").ok_or("falta el contenedor")?;
    run_native_command(
        ctx,
        "container-logs",
        &engine,
        &["logs".into(), "--tail".into(), "200".into(), name],
        false,
        "",
    )
}

fn container_exec(ctx: &Context) -> Result<(), String> {
    let engine = container_engine()?;
    let name = crate::menu_input("Contenedor: ").ok_or("falta el contenedor")?;
    let command = crate::menu_input("Comando: ").ok_or("falta el comando")?;
    run_native_command(
        ctx,
        "container-exec",
        &engine,
        &["exec".into(), "-it".into(), name, command],
        true,
        "¿Ejecutar este comando en el contenedor?",
    )
}

fn kubernetes_menu(ctx: &Context) -> Result<(), String> {
    if !offer(ctx, "kubectl") {
        return Err("kubectl no está disponible".into());
    }
    loop {
        crate::clear_screen();
        println!("=== Kubernetes ===");
        println!("  1) Contextos\n  2) Recursos\n  3) Aplicar manifiesto\n  4) Eliminar recurso\n  5) Escalar deployment\n  6) Reiniciar rollout\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => kubectl_read(ctx, vec!["config".into(), "get".into(), "contexts".into()]),
            "2" => kubectl_resources(ctx),
            "3" => kubectl_file(ctx, "apply"),
            "4" => kubectl_resource_mutation(ctx, "delete"),
            "5" => kubectl_scale(ctx),
            "6" => kubectl_rollout(ctx),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        if !answer.is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn kubectl_read(ctx: &Context, args: Vec<String>) -> Result<(), String> {
    run_native_command(ctx, "kubernetes-read", "kubectl", &args, false, "")
}

fn kubectl_resources(ctx: &Context) -> Result<(), String> {
    let resource = crate::menu_input("Recurso (Enter para all): ").unwrap_or_default();
    let resource = if resource.is_empty() {
        "all".into()
    } else {
        resource
    };
    kubectl_read(ctx, vec!["get".into(), resource])
}

fn kubectl_file(ctx: &Context, operation: &str) -> Result<(), String> {
    let file = crate::menu_input("Manifiesto YAML/JSON: ").ok_or("falta el manifiesto")?;
    run_native_command(
        ctx,
        "kubernetes-apply",
        "kubectl",
        &[operation.into(), "-f".into(), file],
        true,
        "¿Aplicar este manifiesto al clúster?",
    )
}

fn kubectl_resource_mutation(ctx: &Context, operation: &str) -> Result<(), String> {
    let resource = crate::menu_input("Tipo y nombre del recurso: ").ok_or("falta el recurso")?;
    run_native_command(
        ctx,
        "kubernetes-delete",
        "kubectl",
        &[operation.into(), resource],
        true,
        "¿Eliminar este recurso del clúster?",
    )
}

fn kubectl_scale(ctx: &Context) -> Result<(), String> {
    let deployment = crate::menu_input("Deployment: ").ok_or("falta el deployment")?;
    let replicas = crate::menu_input("Réplicas: ").ok_or("falta el número de réplicas")?;
    run_native_command(
        ctx,
        "kubernetes-scale",
        "kubectl",
        &[
            "scale".into(),
            "deployment".into(),
            deployment,
            format!("--replicas={replicas}"),
        ],
        true,
        "¿Cambiar el número de réplicas?",
    )
}

fn kubectl_rollout(ctx: &Context) -> Result<(), String> {
    let deployment = crate::menu_input("Deployment: ").ok_or("falta el deployment")?;
    run_native_command(
        ctx,
        "kubernetes-rollout",
        "kubectl",
        &[
            "rollout".into(),
            "restart".into(),
            format!("deployment/{deployment}"),
        ],
        true,
        "¿Reiniciar este rollout?",
    )
}

fn run_native_command(
    ctx: &Context,
    operation: &str,
    program: &str,
    args: &[String],
    confirm: bool,
    question: &str,
) -> Result<(), String> {
    let rendered = args
        .iter()
        .map(|arg| crate::common::shell_display(arg))
        .collect::<Vec<_>>()
        .join(" ");
    println!("$ {program} {rendered}");
    if ctx.dry_run {
        record_native(ctx, operation, program, "planned", args)?;
        println!("Simulación: no se ejecutó el comando.");
        return Ok(());
    }
    if confirm && !ask(question) {
        record_native(ctx, operation, program, "cancelled", args)?;
        println!("Operación cancelada.");
        return Ok(());
    }
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("no se pudo ejecutar {program}: {error}"))?;
    let state = if status.success() {
        "executed"
    } else {
        "failed"
    };
    record_native(ctx, operation, program, state, args)?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} terminó con {status}"))
    }
}

fn record_native(
    ctx: &Context,
    operation: &str,
    program: &str,
    state: &str,
    args: &[String],
) -> Result<(), String> {
    if let Some(plan) = &ctx.plan {
        plan.record(
            operation,
            Path::new(program),
            state,
            false,
            &args.join(" "),
            "native command",
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn powershell(script: &str) -> Result<(), String> {
    let shell = if command_exists("powershell") {
        "powershell"
    } else {
        "pwsh"
    };
    native(
        shell,
        &["-NoProfile", "-NonInteractive", "-Command", script],
    )
}
fn native(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
fn best_effort(program: &str, args: &[&str]) -> bool {
    match native(program, args) {
        Ok(()) => true,
        Err(error) => {
            println!("Aviso: no se pudo consultar {program}: {error}");
            false
        }
    }
}
fn offer(ctx: &Context, id: &str) -> bool {
    if command_exists(id)
        || crate::platform::host_tools()
            .iter()
            .find(|tool| tool.id == id)
            .is_some_and(crate::platform::host_tool_available)
    {
        return true;
    }
    if crate::platform::host_tools()
        .iter()
        .any(|tool| tool.id == id)
    {
        println!("{id} no está disponible; LTools puede ofrecer su instalación.");
        match crate::common::ensure_tool(ctx, id) {
            Ok(true) => return true,
            Ok(false) => {}
            Err(error) => println!("No se pudo preparar {id}: {error}"),
        }
    } else {
        println!(
            "{id} no está catalogado para instalación automática; puedes solicitarlo explícitamente con: doctor --install {id}"
        );
    }
    false
}
fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Red, hardware, energía y seguridad Windows ===");
        println!("  1) Estado de red, rutas, DNS y puertos\n  2) Vaciar caché DNS\n  3) Hardware\n  4) Planes de energía\n  5) Firewall y Defender\n  6) SSH, Android, contenedores y Kubernetes\n  7) Gestionar contenedores Docker/Podman\n  8) Gestionar Kubernetes\n  q) Volver");
        print!("Elige una opción (Enter para volver): ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Ok(());
        }
        let result = match input.trim() {
            "1" => network(ctx, "status"),
            "2" => network(ctx, "flush-dns"),
            "3" => hardware(ctx, "status"),
            "4" => power(ctx, "plans"),
            "5" => security(ctx, "status"),
            "6" => tools_menu(ctx),
            "7" => container_menu(ctx),
            "8" => kubernetes_menu(ctx),
            "" | "q" | "Q" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(e) = result {
            println!("Error: {e}");
        }
        if !input.trim().is_empty() {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}
