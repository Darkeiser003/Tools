use crate::common::{ask, command_exists, run_command, Context};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const NATIVE_COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let area = first(args).unwrap_or("menu");
    match area {
        "network" | "net" => {
            let action = sub(args, area);
            if action == "menu" {
                network_menu(ctx)
            } else {
                network_with_args(ctx, action, args)
            }
        }
        "hardware" | "hw" => hardware(ctx, sub(args, area)),
        "power" | "energy" => {
            let action = sub(args, area);
            if action == "menu" { power_menu(ctx) } else { power(ctx, action) }
        }
        "security" | "firewall" => {
            let action = sub(args, area);
            if action == "menu" { security_menu(ctx) } else { security(ctx, action) }
        }
        "utilities" | "utility" => utilities_dispatch(ctx, args),
        "install-utility" => install_utility(ctx, args),
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
        _ => Err("native admite network, hardware, power, security, utilities, tools, containers, kubernetes o menu".into()),
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
    network_with_args(ctx, action, &[])
}

fn network_with_args(ctx: &Context, action: &str, raw_args: &[String]) -> Result<(), String> {
    match action {
        "status" | "overview" => {
            println!("=== Red Linux ===");
            if offer(ctx, "ip") {
                best_effort("ip", &["-brief", "address"]);
            }
            println!("\n=== Rutas ===");
            if offer(ctx, "ip") {
                best_effort("ip", &["route"]);
            }
            println!("\n=== DNS ===");
            if offer(ctx, "resolvectl") {
                best_effort("resolvectl", &["status"]);
            } else {
                println!("resolvectl no está disponible.");
            }
            println!("\n=== Puertos escuchando ===");
            if offer(ctx, "ss") {
                best_effort("ss", &["-tuln"]);
            } else {
                println!("ss no está disponible.");
            }
            Ok(())
        }
        "interfaces" | "ifaces" => {
            println!("=== Interfaces de red ===");
            if offer(ctx, "ip") { best_effort("ip", &["-brief", "address"]); }
            else { println!("ip no está disponible."); }
            Ok(())
        }
        "routes" | "route" => {
            println!("=== Rutas de red ===");
            if offer(ctx, "ip") { best_effort("ip", &["route"]); }
            else { println!("ip no está disponible."); }
            Ok(())
        }
        "dns" => {
            println!("=== DNS ===");
            if offer(ctx, "resolvectl") { best_effort("resolvectl", &["status"]); }
            else { println!("resolvectl no está disponible."); }
            Ok(())
        }
        "listening" | "ports" | "listeners" => {
            println!("=== Puertos escuchando ===");
            if offer(ctx, "ss") { best_effort("ss", &["-tuln"]); }
            else { println!("ss no está disponible."); }
            Ok(())
        }
        "connections" | "nmcli" => {
            println!("=== Conexiones NetworkManager ===");
            if offer(ctx, "nmcli") { best_effort("nmcli", &["connection", "show"]); }
            else { println!("nmcli no está disponible."); }
            Ok(())
        }
        "set-interface" => {
            let interface = option(raw_args, "--interface").ok_or("falta --interface")?;
            let state = option(raw_args, "--state").ok_or("falta --state")?;
            if !interface
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || "._:-".contains(value))
                || !matches!(state.as_str(), "up" | "down")
            {
                return Err("interfaz o estado no válido; usa --state up|down".into());
            }
            run_native_command(
                ctx,
                "set-interface",
                "ip",
                &[
                    "link".into(),
                    "set".into(),
                    "dev".into(),
                    interface,
                    state,
                ],
                true,
                "¿Cambiar el estado de esta interfaz?",
            )
        }
        "connection-up" | "connection-down" => {
            let connection = option(raw_args, "--connection").ok_or("falta --connection")?;
            if connection.trim().is_empty() {
                return Err("la conexión no puede estar vacía".into());
            }
            if !offer(ctx, "nmcli") {
                return Err("nmcli no está disponible".into());
            }
            let state = if action == "connection-up" { "up" } else { "down" };
            run_native_command(
                ctx,
                "networkmanager-connection",
                "nmcli",
                &["connection".into(), state.into(), connection],
                true,
                "¿Aplicar esta acción de NetworkManager?",
            )
        }
        "flush-dns" | "dns-flush" => {
            if !offer(ctx, "resolvectl") {
                return Err("resolvectl no está disponible en este sistema".into());
            }
            let confirmed = raw_args.iter().any(|value| value == "--yes");
            if !ctx.dry_run && !confirmed && !ask("¿Vaciar la caché DNS de resolvectl?") {
                let _ = record_native(
                    ctx,
                    "flush-dns",
                    "resolvectl",
                    "cancelled",
                    &["flush-caches".into()],
                );
                println!("Operación cancelada.");
                return Ok(());
            }
            let command = vec!["flush-caches".into()];
            let ok = run_command("resolvectl", &command, ctx.dry_run).map_err(|e| e.to_string())?;
            record_native(
                ctx,
                "flush-dns",
                "resolvectl",
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
                } else {
                    println!("Caché DNS vaciada.");
                }
                Ok(())
            } else {
                Err("resolvectl no pudo vaciar la caché DNS".into())
            }
        }
        _ => Err("network admite menu, status, interfaces, routes, dns, listening, connections, set-interface, connection-up, connection-down o flush-dns".into()),
    }
}

fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
}

fn hardware(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("hardware admite status".into());
    }
    println!("=== Hardware Linux ===");
    if offer(ctx, "uname") {
        best_effort("uname", &["-srmo"]);
    }
    println!("\n=== CPU ===");
    if offer(ctx, "lscpu") {
        best_effort("lscpu", &[]);
    }
    println!("\n=== Memoria ===");
    if offer(ctx, "free") {
        best_effort("free", &["-h"]);
    }
    println!("\n=== PCI ===");
    if offer(ctx, "lspci") {
        best_effort("lspci", &[]);
    }
    println!("\n=== USB ===");
    if offer(ctx, "lsusb") {
        best_effort("lsusb", &[]);
    }
    Ok(())
}

fn power(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("power admite status".into());
    }
    println!("=== Energía Linux ===");
    if offer(ctx, "powerprofilesctl") {
        if best_effort("powerprofilesctl", &["get"]) {
            best_effort("powerprofilesctl", &["list"]);
        }
    } else {
        println!("powerprofilesctl no está disponible.");
    }
    if offer(ctx, "upower") {
        best_effort("upower", &["-e"]);
    } else {
        println!("upower no está disponible; no se instala automáticamente.");
    }
    if offer(ctx, "systemd-inhibit") {
        println!("\n=== Bloqueos e inhibidores ===");
        best_effort("systemd-inhibit", &["--list"]);
    }
    Ok(())
}

fn security(ctx: &Context, action: &str) -> Result<(), String> {
    if action != "status" && action != "overview" {
        return Err("security admite status".into());
    }
    println!("=== Firewall Linux ===");
    let mut found = false;
    if offer(ctx, "firewall-cmd") {
        found = true;
        println!("firewalld:");
        best_effort("firewall-cmd", &["--state"]);
    }
    if offer(ctx, "ufw") {
        found = true;
        println!("ufw:");
        best_effort("ufw", &["status"]);
    }
    if offer(ctx, "nft") {
        found = true;
        println!("nftables (consulta):");
        best_effort("nft", &["list", "ruleset"]);
    }
    if !found {
        println!("No se detectó firewalld, ufw ni nftables.");
    }
    println!("LTools solo consulta el firewall; los cambios se realizan en su gestor nativo.");
    Ok(())
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
        "utilities" | "utility" => utilities_dispatch(ctx, args),
        "install-utility" => install_utility(ctx, args),
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
        _ => Err("tools admite status, install, menu, utilities, ssh, adb, containers, images, volumes, networks, compose o kubernetes".into()),
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
                return Err("SSH no está disponible".into());
            }
            let target = required_option(args, "--target")?;
            let mut command = Vec::new();
            if let Some(port) = option_value(args, "--port") {
                command.extend(["-p".into(), port]);
            }
            if let Some(identity) = option_value(args, "--identity") {
                command.extend(["-i".into(), identity]);
            }
            command.push(target);
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
            let source = required_option(args, "--source")?;
            let destination = required_option(args, "--destination")?;
            let mut command = Vec::new();
            if args.iter().any(|value| value == "--recursive") {
                command.push("-r".into());
            }
            command.extend([source, destination]);
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
            let target = required_option(args, "--target")?;
            run_native_command(
                ctx,
                action,
                "sftp",
                &[target],
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
            let operation = action.strip_prefix("adb-").unwrap().to_owned();
            let mut command = vec![operation];
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
                !matches!(
                    operation.as_str(),
                    "ps" | "logs" | "config" | "images" | "top"
                ),
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
            let mut command = vec![
                "apply".into(),
                "-f".into(),
                required_option(args, "--file")?,
            ];
            if let Some(namespace) = option_value(args, "--namespace") {
                command.extend(["--namespace".into(), namespace]);
            }
            run_native_command(
                ctx,
                action,
                "kubectl",
                &command,
                true,
                "¿Aplicar este manifiesto al clúster?",
            )
        }
        "kubernetes-delete" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            let mut command = vec!["delete".into(), required_option(args, "--resource")?];
            if let Some(namespace) = option_value(args, "--namespace") {
                command.extend(["--namespace".into(), namespace]);
            }
            run_native_command(
                ctx,
                action,
                "kubectl",
                &command,
                true,
                "¿Eliminar este recurso del clúster?",
            )
        }
        "kubernetes-scale" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            let deployment = required_option(args, "--deployment")?;
            let replicas = required_option(args, "--replicas")?;
            run_native_command(
                ctx,
                action,
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
        "kubernetes-rollout" => {
            if !offer(ctx, "kubectl") {
                return Err("kubectl no está disponible".into());
            }
            let deployment = required_option(args, "--deployment")?;
            run_native_command(
                ctx,
                action,
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
    println!("=== Herramientas del anfitrión Linux ===");
    println!("La consulta recorre el catálogo completo, no instala nada ni inicia servicios.");
    for tool in crate::platform::host_tools() {
        let state = if crate::platform::host_tool_available(tool) {
            "disponible"
        } else if tool.installable && !tool.install_package.is_empty() {
            "ausente (instalable desde LTools)"
        } else {
            "ausente (instalación manual o nativa)"
        };
        let version = crate::platform::host_tool_version(tool)
            .map(|value| format!(" | {}", value.trim()))
            .unwrap_or_default();
        println!("{:<22} {:<38}{}", tool.id, state, version);
    }
    println!("\nPara instalar una dependencia concreta: ltools native tools install --tool <id>");
    println!("La instalación siempre pide confirmación y muestra el gestor y el paquete.");
    Ok(())
}

fn utilities_dispatch(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .position(|value| value == "utilities" || value == "utility")
        .and_then(|index| args.get(index + 1))
        .filter(|value| !value.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("menu");
    match action {
        "menu" => utilities_menu(ctx),
        "status" | "list" | "overview" => utilities_status(),
        "install" | "install-dependency" => tools_install(ctx, args),
        _ => Err("utilities admite menu, status o install".into()),
    }
}

fn install_utility(ctx: &Context, args: &[String]) -> Result<(), String> {
    let mut scoped = args.to_vec();
    scoped.extend(["--category".into(), "utilities".into()]);
    tools_install(ctx, &scoped)
}

fn utilities_status() -> Result<(), String> {
    println!("=== Utilidades del sistema Linux ===");
    println!("Consulta e instalación explícita por utilidad; no se inicia ningún servicio.");
    for tool in crate::platform::host_tools()
        .iter()
        .filter(|tool| tool.category == "utilities")
    {
        let state = if crate::platform::host_tool_available(tool) {
            "disponible"
        } else if tool.installable && !tool.install_package.is_empty() {
            "ausente (instalable desde LTools)"
        } else {
            "ausente (instalación manual o nativa)"
        };
        println!("{:<18} {:<38} {}", tool.id, state, tool.feature);
    }
    Ok(())
}

fn utilities_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Utilidades del sistema Linux ===");
        println!("  1) Detectar estado y versiones\n  2) Instalar una utilidad\n  q) Volver");
        let answer = crate::menu_input("Elige una opción: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => utilities_status(),
            "2" => tools_install(
                ctx,
                &[
                    "utilities".into(),
                    "install".into(),
                    "--category".into(),
                    "utilities".into(),
                ],
            ),
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

fn tools_install(ctx: &Context, args: &[String]) -> Result<(), String> {
    let category = option_value(args, "--category");
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
        println!("=== Instalar dependencia ===");
        println!("Selecciona una herramienta ausente. No se instala nada sin confirmación.");
        for tool in crate::platform::host_tools().iter().filter(|tool| {
            category
                .as_deref()
                .is_none_or(|value| tool.category == value)
                && !crate::platform::host_tool_available(tool)
                && tool.installable
                && !tool.install_package.is_empty()
        }) {
            println!(
                "  {:<18} paquete: {:<20} función: {}",
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
    let known = crate::platform::host_tools()
        .iter()
        .find(|tool| tool.id == id.trim());
    if known.is_none() {
        return Err(format!("dependencia no catalogada: {}", id.trim()));
    }
    if crate::platform::host_tool_available(known.unwrap()) {
        println!("Ya está disponible: {}", id.trim());
        return Ok(());
    }
    let installed = crate::platform::install_tool(id.trim(), ctx.dry_run)?;
    if !installed {
        return Err(format!(
            "No se pudo instalar {}. Revisa el gestor de paquetes o la guía de instalación manual.",
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
    println!("=== Contenedores Linux (solo lectura) ===");
    if offer(ctx, "docker") {
        best_effort("docker", &["version"]);
        best_effort("docker", &["info"]);
        if command_exists("docker-compose") {
            best_effort("docker-compose", &["version"]);
        } else {
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
    println!("=== Kubernetes Linux (solo lectura) ===");
    if !offer(ctx, "kubectl") {
        println!("kubectl no está disponible; no se consulta ningún clúster.");
        return Ok(());
    }
    best_effort("kubectl", &["version", "--client=true"]);
    best_effort("kubectl", &["config", "current-context"]);
    best_effort("kubectl", &["config", "get-contexts"]);
    best_effort("kubectl", &["cluster-info", "--request-timeout=3s"]);
    best_effort("kubectl", &["get", "nodes", "--request-timeout=3s"]);
    println!("No se aplican manifiestos ni cambios en el clúster desde esta consulta.");
    Ok(())
}

fn tools_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Herramientas operativas Linux ===");
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
        return Err("SSH no está disponible; instala OpenSSH y vuelve a intentarlo".into());
    }
    loop {
        crate::clear_screen();
        println!("=== SSH / SCP / SFTP ===");
        println!("  1) Conectar por SSH\n  2) Copiar con SCP\n  3) Abrir SFTP\n  4) Generar clave Ed25519\n  5) Instalar clave pública (ssh-copy-id)\n  6) Comprobar versión\n  q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => ssh_connect(ctx),
            "2" => scp_copy(ctx),
            "3" => sftp_open(ctx),
            "4" => ssh_keygen(ctx),
            "5" => ssh_copy_id(ctx),
            "6" => {
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

fn ssh_target() -> Option<String> {
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
    let target = ssh_target().ok_or("se requiere un host")?;
    let port = crate::menu_input("Puerto (vacío para 22): ").unwrap_or_default();
    let identity =
        crate::menu_input("Clave privada (vacío para la configuración SSH): ").unwrap_or_default();
    let command = crate::menu_input("Comando remoto (vacío para una sesión interactiva): ")
        .unwrap_or_default();
    let mut args = Vec::new();
    if !port.is_empty() {
        args.extend(["-p".into(), port]);
    }
    if !identity.is_empty() {
        args.extend(["-i".into(), identity]);
    }
    args.push(target);
    if !command.is_empty() {
        args.push(command);
    }
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
    let target = ssh_target().ok_or("se requiere un host")?;
    run_native_command(
        ctx,
        "sftp",
        "sftp",
        &[target],
        true,
        "¿Abrir esta sesión SFTP?",
    )
}

fn ssh_keygen(ctx: &Context) -> Result<(), String> {
    if !offer(ctx, "ssh-keygen") {
        return Err("ssh-keygen no está disponible".into());
    }
    let default_path = ctx.home.join(".ssh/id_ed25519");
    let path = crate::menu_input(&format!(
        "Ruta de la clave (Enter para {}): ",
        default_path.display()
    ))
    .unwrap_or_default();
    let path = if path.is_empty() {
        default_path.display().to_string()
    } else {
        path
    };
    let comment = crate::menu_input("Comentario opcional: ").unwrap_or_default();
    let mut args = vec!["-t".into(), "ed25519".into(), "-f".into(), path];
    if !comment.is_empty() {
        args.extend(["-C".into(), comment]);
    }
    run_native_command(
        ctx,
        "ssh-keygen",
        "ssh-keygen",
        &args,
        true,
        "¿Generar esta clave SSH?",
    )
}

fn ssh_copy_id(ctx: &Context) -> Result<(), String> {
    if !offer(ctx, "ssh-copy-id") {
        return Err("ssh-copy-id no está disponible".into());
    }
    let target = ssh_target().ok_or("se requiere un host")?;
    run_native_command(
        ctx,
        "ssh-copy-id",
        "ssh-copy-id",
        &[target],
        true,
        "¿Instalar la clave pública en el host remoto?",
    )
}

fn adb_menu(ctx: &Context) -> Result<(), String> {
    if !offer(ctx, "adb") {
        return Err("ADB no está disponible; instala Android Platform Tools".into());
    }
    loop {
        crate::clear_screen();
        println!("=== Android Debug Bridge ===");
        println!("  1) Dispositivos\n  2) Shell remoto\n  3) Instalar APK\n  4) Enviar archivo\n  5) Extraer archivo\n  6) Reiniciar dispositivo\n  7) Logcat\n  8) Reiniciar servidor ADB\n  q) Volver");
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
            "8" => run_native_command(
                ctx,
                "adb-server",
                "adb",
                &["kill-server".into()],
                true,
                "¿Reiniciar el servidor ADB?",
            )
            .and_then(|_| {
                run_native_command(
                    ctx,
                    "adb-server",
                    "adb",
                    &["start-server".into()],
                    false,
                    "",
                )
            }),
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
        println!("  1) Listar contenedores\n  2) Crear y ejecutar\n  3) Iniciar\n  4) Detener\n  5) Reiniciar\n  6) Eliminar\n  7) Ver logs\n  8) Ejecutar comando\n  9) Inspeccionar\n 10) Estadísticas\n 11) Procesos\n 12) Puertos\n 13) Cambios\n 14) Pausar\n 15) Reanudar\n 16) Terminar\n 17) Renombrar\n 18) Copiar archivos\n 19) Limpiar detenidos\n q) Volver");
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
                let current = required_menu_value("Contenedor actual: ")?;
                let new_name = required_menu_value("Nuevo nombre: ")?;
                direct_container_action(
                    ctx,
                    "container-rename",
                    vec!["--name".into(), current, "--new-name".into(), new_name],
                )
            }
            "18" => {
                let source = required_menu_value("Origen (host:contenedor o contenedor:ruta): ")?;
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
        println!("  1) Listar imágenes\n  2) Descargar imagen\n  3) Inspeccionar\n  4) Ver historial\n  5) Construir desde Dockerfile/Containerfile\n  6) Etiquetar\n  7) Eliminar\n  8) Limpiar no utilizadas\n q) Volver");
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
        println!("  1) Contextos\n  2) Recursos\n  3) Logs de pod\n  4) Aplicar manifiesto\n  5) Eliminar recurso\n  6) Escalar deployment\n  7) Reiniciar rollout\n  8) Port-forward\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => kubectl_read(ctx, vec!["config".into(), "get".into(), "contexts".into()]),
            "2" => kubectl_resources(ctx),
            "3" => kubectl_logs(ctx),
            "4" => kubectl_file(ctx, "apply"),
            "5" => kubectl_resource_mutation(ctx, "delete"),
            "6" => kubectl_scale(ctx),
            "7" => kubectl_rollout(ctx),
            "8" => kubectl_port_forward(ctx),
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
    let namespace = crate::menu_input("Namespace (vacío para todos): ").unwrap_or_default();
    let resource = if resource.is_empty() {
        "all".into()
    } else {
        resource
    };
    let mut args = vec!["get".into(), resource];
    if !namespace.is_empty() {
        args.extend(["-n".into(), namespace]);
    }
    kubectl_read(ctx, args)
}

fn kubectl_logs(ctx: &Context) -> Result<(), String> {
    let pod = crate::menu_input("Pod: ").ok_or("falta el pod")?;
    let namespace = crate::menu_input("Namespace (vacío para default): ").unwrap_or_default();
    let mut args = vec!["logs".into(), pod, "--tail=200".into()];
    if !namespace.is_empty() {
        args.extend(["-n".into(), namespace]);
    }
    kubectl_read(ctx, args)
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

fn kubectl_port_forward(ctx: &Context) -> Result<(), String> {
    let target = crate::menu_input("Pod o servicio: ").ok_or("falta el objetivo")?;
    let ports = crate::menu_input("Puertos (ej. 8080:80): ").ok_or("faltan los puertos")?;
    run_native_command(
        ctx,
        "kubernetes-port-forward",
        "kubectl",
        &["port-forward".into(), target, ports],
        true,
        "¿Abrir este port-forward?",
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
    let command_args: Vec<String> = args
        .iter()
        .filter(|arg| arg.as_str() != "--yes")
        .cloned()
        .collect();
    let auto_confirmed = args.iter().any(|arg| arg == "--yes");
    let rendered = command_args
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
    if confirm && !auto_confirmed && !ask(question) {
        record_native(ctx, operation, program, "cancelled", &command_args)?;
        println!("Operación cancelada.");
        return Ok(());
    }
    let ok = if requires_privilege(operation) {
        crate::common::run_with_sudo(program, &command_args, false)
            .map_err(|error| format!("no se pudo ejecutar {program} con elevación: {error}"))?
    } else {
        Command::new(program)
            .args(&command_args)
            .status()
            .map_err(|error| format!("no se pudo ejecutar {program}: {error}"))?
            .success()
    };
    let state = if ok { "executed" } else { "failed" };
    record_native(ctx, operation, program, state, &command_args)?;
    if ok {
        Ok(())
    } else {
        Err(if requires_privilege(operation) {
            format!("{program} no pudo ejecutarse con permisos elevados")
        } else {
            format!("{program} terminó con error")
        })
    }
}

fn requires_privilege(operation: &str) -> bool {
    matches!(
        operation,
        "set-interface"
            | "networkmanager-connection"
            | "suspend"
            | "hibernate"
            | "reboot"
            | "shutdown"
            | "firewall-enable"
            | "firewall-disable"
            | "firewall-reload"
    )
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

fn best_effort(program: &str, args: &[&str]) -> bool {
    if let Err(error) = capture(program, args) {
        println!("Aviso: no se pudo consultar {program}: {error}");
        false
    } else {
        true
    }
}
fn capture(program: &str, args: &[&str]) -> Result<(), String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("no se pudo ejecutar {program}: {e}"))?;
    // Leer stdout y stderr en paralelo evita que una consulta voluminosa
    // bloquee el proceso hijo al llenar uno de los pipes antes de que expire
    // el timeout. Es especialmente importante para `ip`, `lscpu`, `nft` y
    // diagnósticos de contenedores.
    let mut stdout_reader = child.stdout.take().map(|mut reader| {
        thread::spawn(move || {
            let mut data = Vec::new();
            let _ = reader.read_to_end(&mut data);
            data
        })
    });
    let mut stderr_reader = child.stderr.take().map(|mut reader| {
        thread::spawn(move || {
            let mut data = Vec::new();
            let _ = reader.read_to_end(&mut data);
            data
        })
    });
    let deadline = Instant::now() + NATIVE_COMMAND_TIMEOUT;
    loop {
        match child
            .try_wait()
            .map_err(|e| format!("no se pudo consultar {program}: {e}"))?
        {
            Some(status) => {
                let stdout = stdout_reader
                    .take()
                    .and_then(|reader| reader.join().ok())
                    .unwrap_or_default();
                let stderr = stderr_reader
                    .take()
                    .and_then(|reader| reader.join().ok())
                    .unwrap_or_default();
                print!("{}", String::from_utf8_lossy(&stdout));
                if status.success() {
                    return Ok(());
                }
                let error = String::from_utf8_lossy(&stderr).trim().to_string();
                return if error.is_empty() { Ok(()) } else { Err(error) };
            }
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.take().and_then(|reader| reader.join().ok());
                let _ = stderr_reader.take().and_then(|reader| reader.join().ok());
                return Err(format!("{program} superó el límite de 10 segundos"));
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Red, hardware, energía y seguridad Linux ===");
        println!("  1) Gestionar red, interfaces y DNS\n  2) Vaciar caché DNS\n  3) Hardware y dispositivos\n  4) Energía y perfiles\n  5) Firewall y seguridad\n  q) Volver");
        print!("Elige una opción (Enter para volver): ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Ok(());
        }
        let result = match input.trim() {
            "1" => network_menu(ctx),
            "2" => network(ctx, "flush-dns"),
            "3" => hardware(ctx, "status"),
            "4" => power_menu(ctx),
            "5" => security_menu(ctx),
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

fn network_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Gestión de red Linux ===");
        println!("  1) Estado completo\n  2) Activar interfaz\n  3) Desactivar interfaz\n  4) Vaciar caché DNS\n  5) Conectar con NetworkManager\n  6) Desconectar con NetworkManager\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => network(ctx, "status"),
            "2" | "3" => {
                let interface = crate::menu_input("Interfaz: ").ok_or("falta la interfaz")?;
                if interface.is_empty() {
                    Err("la interfaz es obligatoria".into())
                } else {
                    let state = if answer == "2" { "up" } else { "down" };
                    run_native_command(
                        ctx,
                        "set-interface",
                        "ip",
                        &[
                            "link".into(),
                            "set".into(),
                            "dev".into(),
                            interface,
                            state.into(),
                        ],
                        true,
                        "¿Cambiar el estado de esta interfaz?",
                    )
                }
            }
            "4" => network(ctx, "flush-dns"),
            "5" | "6" => {
                if !offer(ctx, "nmcli") {
                    Err("nmcli no está disponible".into())
                } else {
                    let connection =
                        crate::menu_input("Nombre de conexión: ").ok_or("falta la conexión")?;
                    let action = if answer == "5" { "up" } else { "down" };
                    run_native_command(
                        ctx,
                        "networkmanager-connection",
                        "nmcli",
                        &["connection".into(), action.into(), connection],
                        true,
                        "¿Aplicar esta acción de NetworkManager?",
                    )
                }
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

fn power_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Gestión de energía Linux ===");
        println!("  1) Estado\n  2) Listar perfiles\n  3) Cambiar perfil\n  4) Suspender\n  5) Hibernar\n  6) Reiniciar\n  7) Apagar\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => power(ctx, "status"),
            "2" => {
                if offer(ctx, "powerprofilesctl") {
                    run_native_command(
                        ctx,
                        "power-profile-list",
                        "powerprofilesctl",
                        &["list".into()],
                        false,
                        "",
                    )
                } else {
                    Err("powerprofilesctl no está disponible".into())
                }
            }
            "3" => {
                let profile = crate::menu_input("Perfil (power-saver/balanced/performance): ")
                    .ok_or("falta el perfil")?;
                run_native_command(
                    ctx,
                    "profile-set",
                    "powerprofilesctl",
                    &["set".into(), profile],
                    true,
                    "¿Cambiar el perfil de energía?",
                )
            }
            "4" => run_native_command(
                ctx,
                "suspend",
                "systemctl",
                &["suspend".into()],
                true,
                "¿Suspender el sistema?",
            ),
            "5" => run_native_command(
                ctx,
                "hibernate",
                "systemctl",
                &["hibernate".into()],
                true,
                "¿Hibernar el sistema?",
            ),
            "6" => run_native_command(
                ctx,
                "reboot",
                "systemctl",
                &["reboot".into()],
                true,
                "¿Reiniciar el sistema?",
            ),
            "7" => run_native_command(
                ctx,
                "shutdown",
                "systemctl",
                &["poweroff".into()],
                true,
                "¿Apagar el sistema?",
            ),
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

fn security_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Gestión de firewall Linux ===");
        println!("  1) Estado\n  2) Activar UFW\n  3) Desactivar UFW\n  4) Recargar firewalld\n  5) Ver reglas nftables\n q) Volver");
        let answer = crate::menu_input("Elige una operación: ").unwrap_or_default();
        let result = match answer.as_str() {
            "1" => security(ctx, "status"),
            "2" | "3" => {
                if !offer(ctx, "ufw") {
                    Err("ufw no está disponible".into())
                } else {
                    let action = if answer == "2" { "enable" } else { "disable" };
                    let operation = if action == "enable" {
                        "firewall-enable"
                    } else {
                        "firewall-disable"
                    };
                    run_native_command(
                        ctx,
                        operation,
                        "ufw",
                        &[action.into()],
                        true,
                        "¿Cambiar el estado del firewall UFW?",
                    )
                }
            }
            "4" => run_native_command(
                ctx,
                "firewall-reload",
                "firewall-cmd",
                &["--reload".into()],
                true,
                "¿Recargar firewalld?",
            ),
            "5" => {
                if offer(ctx, "nft") {
                    run_native_command(
                        ctx,
                        "firewall-rules",
                        "nft",
                        &["list".into(), "ruleset".into()],
                        false,
                        "",
                    )
                } else {
                    Err("nft no está disponible".into())
                }
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
