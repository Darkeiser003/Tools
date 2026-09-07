use crate::common::{command_exists, ensure_tool, run_command, run_with_sudo, Context};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

/// Gestión Linux de almacenamiento. Las operaciones destructivas de tabla de
/// particiones se delegan al gestor nativo tras una confirmación explícita.
pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = first_action(args).unwrap_or("menu");
    match action {
        "status" | "disks" | "overview" => status(ctx),
        "partitions" | "partition" => partitions(ctx),
        "partition-table" | "partition-inspect" => {
            partition_table(ctx, target_after(args, action)?)
        }
        "guide" | "partition-guide" => partition_guide(),
        "mounts" | "mountpoints" => mounts(),
        "inspect" | "details" => inspect(ctx, target_after(args, action)?),
        "mount" => mount(ctx, target_after(args, action)?),
        "unmount" | "umount" => unmount(ctx, target_after(args, action)?),
        "health" | "smart" => health(ctx, target_after(args, action)?),
        "check" | "filesystem-check" => filesystem_check(ctx, target_after(args, action)?),
        "usage" | "space" | "inodes" => usage(),
        "filesystems" | "filesystem-info" | "uuid" | "labels" => filesystems(),
        "volume-stack" | "lvm" | "btrfs" | "zfs" | "raid" => volume_stack(),
        "blockdev" | "block-device" => block_device_info(ctx, target_after(args, action)?),
        "open-gparted" | "gparted" | "partition-manager" => open_gparted(ctx, args),
        "open" | "browse" => open_path(ctx, target_after(args, action)?),
        "tools" => tools(),
        "menu" => menu(ctx),
        _ => Err(format!("acción de almacenamiento desconocida: {action}")),
    }
}

fn first_action(args: &[String]) -> Option<&str> {
    args.iter()
        .map(String::as_str)
        .find(|arg| !arg.starts_with('-'))
}

fn target_after<'a>(args: &'a [String], action: &str) -> Result<&'a str, String> {
    let index = args
        .iter()
        .position(|arg| arg == action)
        .ok_or_else(|| format!("no se encontró la acción {action}"))?;
    args.get(index + 1)
        .map(String::as_str)
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| format!("{action} necesita un dispositivo o ruta"))
}

fn status(ctx: &Context) -> Result<(), String> {
    require("df")?;
    println!("=== Resumen de almacenamiento Linux ===");
    println!("Home: {}", ctx.home.display());
    run_capture("df", &["-h"])?;
    mounts()
}

fn partitions(ctx: &Context) -> Result<(), String> {
    println!("=== Discos y particiones Linux ===");
    if command_exists("lsblk") {
        run_capture(
            "lsblk",
            &[
                "-e7",
                "-o",
                "NAME,PATH,SIZE,FSTYPE,LABEL,UUID,MOUNTPOINTS,TYPE,RO,RM",
            ],
        )?;
    } else if !ensure_tool(ctx, "lsblk")? {
        return Err("lsblk es necesario para listar discos y particiones".into());
    }
    if command_exists("parted") {
        println!("\n=== Tablas de particiones (solo lectura) ===");
        run_capture("parted", &["-l"])?;
    } else if !ensure_tool(ctx, "parted")? {
        println!("parted: no instalado; lsblk sigue disponible como alternativa segura.");
    } else {
        println!("parted se ha preparado; vuelve a ejecutar la consulta para incluir sus datos.");
    }
    println!(
        "\nGestor completo: {}",
        if command_exists("gparted") {
            "gparted disponible (abrir desde el menú)"
        } else {
            "gparted no instalado; puede instalarse bajo demanda"
        }
    );
    Ok(())
}

/// Consulta varias vistas de una tabla sin modificarla. Las herramientas se
/// prueban una a una para que la ausencia de una alternativa no oculte las
/// demás; nunca se llama a parted/fdisk en modo de escritura.
fn partition_table(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    println!(
        "=== Tabla de particiones (solo lectura): {} ===",
        target.display()
    );
    let target = target.display().to_string();
    if ctx.dry_run {
        println!("Simulación: se consultarían lsblk, parted print, fdisk -l y sfdisk --dump sobre {target}.");
        return Ok(());
    }
    let mut found = false;
    if command_exists("lsblk") {
        found = true;
        run_capture_owned(
            "lsblk",
            &[
                "-o".into(),
                "NAME,PATH,TYPE,SIZE,FSTYPE,FSVER,LABEL,UUID,MOUNTPOINTS,RO,RM".into(),
                target.clone(),
            ],
        )?;
    }
    if command_exists("parted") {
        found = true;
        println!("\n--- parted print (solo lectura) ---");
        run_capture_owned(
            "parted",
            &[
                "-s".into(),
                target.clone(),
                "unit".into(),
                "MiB".into(),
                "print".into(),
            ],
        )?;
    }
    if command_exists("fdisk") {
        found = true;
        println!("\n--- fdisk -l (solo lectura) ---");
        run_capture_owned("fdisk", &["-l".into(), target.clone()])?;
    }
    if command_exists("sfdisk") {
        found = true;
        println!("\n--- sfdisk --dump (solo lectura) ---");
        run_capture_owned("sfdisk", &["--dump".into(), target])?;
    }
    if !found {
        return Err("no se encontró una herramienta de inspección de particiones".into());
    }
    Ok(())
}

fn partition_guide() -> Result<(), String> {
    println!("=== Guía de particionado Linux: discos y particiones ===");
    println!();
    println!("Esta guía explica qué mirar, qué herramienta elegir y en qué orden trabajar.");
    println!("LTools consulta el sistema y puede abrir gestores nativos; no aplica cambios");
    println!("destructivos de particionado por su cuenta.");
    println!();
    println!("1. Conceptos básicos");
    println!("   Disco físico: /dev/sda, /dev/sdb o /dev/nvme0n1.");
    println!("   Partición:   /dev/sda1, /dev/sdb2 o /dev/nvme0n1p1.");
    println!("   Sistema de archivos: ext4, btrfs, xfs, ntfs, vfat, swap, etc.");
    println!("   Montaje: la ruta donde Linux presenta una partición, por ejemplo /home.");
    println!("   No confundas el disco completo con una de sus particiones.");
    println!();
    println!("2. Orden recomendado para cualquier tarea");
    println!("   a) Haz copia de los datos importantes y cierra aplicaciones que usen el disco.");
    println!("   b) Consulta el resumen:     ltools storage status");
    println!("   c) Consulta el árbol:       ltools storage partitions");
    println!("   d) Inspecciona el objetivo: ltools storage inspect /dev/sdX1");
    println!("   e) Revisa la tabla:         ltools storage partition-table /dev/sdX");
    println!("   f) Decide qué va a cambiar antes de abrir una herramienta de escritura.");
    println!("   g) Aplica una sola operación, revisa el resumen y vuelve a verificar.");
    println!();
    println!("3. Qué consulta hace cada herramienta");
    println!("   lsblk       árbol de discos, particiones, tipos, tamaños y montajes.");
    println!("   blkid       UUID, etiqueta y tipo del sistema de archivos.");
    println!("   findmnt     montajes activos y sus opciones reales.");
    println!("   parted print tabla GPT/MBR, límites y tamaños; consulta de solo lectura.");
    println!("   fdisk       vista compatible de discos y particiones; aquí se usa -l.");
    println!("   sfdisk      volcado legible de la tabla; aquí se usa --dump, solo lectura.");
    println!();
    println!("4. Qué herramienta elegir");
    println!("   GParted                 cambios visuales y guiados en particiones.");
    println!("   GNOME Disks             inspección, montaje y tareas sencillas.");
    println!("   KDE Partition Manager   alternativa completa para escritorios KDE.");
    println!("   parted/fdisk/sfdisk     trabajo CLI avanzado; exige conocer el objetivo.");
    println!("   LVM, cryptsetup, Btrfs, ZFS y mdadm deben gestionarse con sus propias");
    println!("   herramientas, no tratando sus capas como si fueran discos normales.");
    println!();
    println!("5. Tareas habituales");
    println!("   Disco nuevo");
    println!("     1) Identifica el disco completo por modelo y tamaño.");
    println!("     2) Comprueba que no tenga montajes ni datos que conservar.");
    println!("     3) Elige GPT salvo que necesites compatibilidad MBR antigua.");
    println!("     4) Crea la partición, el sistema de archivos y una etiqueta clara.");
    println!("     5) Monta y verifica UUID, capacidad y permisos.");
    println!("   Montar una partición existente");
    println!("     1) Consulta: ltools storage inspect /dev/sdX1");
    println!("     2) Comprueba que el tipo y el UUID sean los esperados.");
    println!("     3) Ejecuta: ltools storage mount /dev/sdX1");
    println!("     4) Verifica con: ltools storage mounts");
    println!("   Desmontar antes de trabajar");
    println!("     1) Comprueba quién usa el punto de montaje.");
    println!("     2) Cierra terminales, juegos, máquinas virtuales y exploradores allí.");
    println!("     3) Ejecuta: ltools storage unmount /dev/sdX1");
    println!("     4) Si es raíz, /home, swap o un volumen ocupado, detente y usa un");
    println!("        sistema live o el procedimiento específico de esa capa.");
    println!();
    println!("6. Operaciones de particionado: todas las capacidades");
    println!("   La guía documenta acciones destructivas para que puedas reconocerlas.");
    println!("   LTools no las ejecuta desde esta pantalla: el selector, la confirmación");
    println!("   y la herramienta nativa deben mostrar el objetivo antes de escribir.");
    println!("   Nivel CRÍTICO: borrar, clean, wipefs, mkfs, luksFormat, destroy y dd.");
    println!();
    println!("   6.1 Tabla de particiones");
    println!("     Crear GPT:     parted /dev/sdX mklabel gpt");
    println!("     Crear MBR:     parted /dev/sdX mklabel msdos");
    println!("     Crear:         parted /dev/sdX mkpart primary ext4 1MiB 100%");
    println!("     Nombrar GPT:   parted /dev/sdX name 1 Datos");
    println!("     Borrar:        parted /dev/sdX rm N");
    println!("     Redimensionar: parted /dev/sdX resizepart N 100%");
    println!("     Cambiar flags: parted /dev/sdX set N esp on");
    println!("     Recuperar:     parted /dev/sdX rescue INICIO FIN");
    println!("     Otras órdenes parted: print, select, unit, align-check, disk_set y");
    println!("     disk_toggle. print consulta; el resto puede escribir o cambiar metadatos.");
    println!();
    println!("     fdisk permite: p listar, g GPT, o MBR, n crear, d borrar, t tipo,");
    println!("     l tipos, x expertos, v validar, w escribir y q salir sin guardar.");
    println!("     sfdisk permite volcar, restaurar, borrar y cambiar tipo, etiqueta,");
    println!("     UUID y tamaño. Usa --dump y --backup antes de cualquier escritura.");
    println!("     gdisk ofrece operaciones equivalentes para GPT y reparación de cabeceras.");
    println!("     Nunca pulses w, Write, Apply o Save sin revisar disco, partición y resumen.");
    println!();
    println!("   6.2 Crear, borrar o limpiar un sistema de archivos");
    println!("     Crear ext4:   mkfs.ext4 /dev/sdX1");
    println!("     Crear Btrfs:  mkfs.btrfs /dev/sdX1");
    println!("     Crear NTFS:   mkfs.ntfs /dev/sdX1");
    println!("     Cambiar label: e2label /dev/sdX1 Datos");
    println!("     Limpiar firmas: wipefs -a /dev/sdX1");
    println!("     Descartar SSD: blkdiscard /dev/sdX");
    println!("     Sobrescribir:   dd if=/dev/zero of=/dev/sdX status=progress");
    println!("     mkfs borra el contenido; wipefs, blkdiscard y dd pueden impedir");
    println!("     la recuperación. Confirma siempre que X no sea el disco del sistema.");
    println!();
    println!("   6.3 Redimensionar y mover");
    println!("     Aumentar: primero amplía la partición y después el sistema de archivos.");
    println!("     Reducir: primero reduce el sistema de archivos y después la partición.");
    println!("     Mover el inicio puede copiar muchos datos y dejar el disco inutilizable");
    println!("     si se interrumpe. Haz copia, usa corriente estable y no fuerces el cierre.");
    println!("     ext4: resize2fs; xfs: xfs_growfs; Btrfs: btrfs filesystem resize;");
    println!("     NTFS: ntfsresize. Cada sistema tiene límites y pasos propios.");
    println!();
    println!("   6.4 Montaje, desmontaje y swap");
    println!("     mount /dev/sdX1 /mnt/datos       monta manualmente.");
    println!("     mount -o ro /dev/sdX1 /mnt/datos  monta solo lectura.");
    println!("     umount /mnt/datos                  desmonta por ruta.");
    println!("     umount -l solo debe usarse cuando conozcas el efecto de lazy unmount.");
    println!("     mkswap /dev/sdX2; swapon /dev/sdX2; swapoff /dev/sdX2 gestionan swap.");
    println!("     Para permanencia, usa UUID en /etc/fstab y prueba con mount -a.");
    println!();
    println!("   6.5 Cifrado LUKS");
    println!("     luksFormat BORRA el contenedor; guarda la cabecera y las claves.");
    println!("     cryptsetup luksFormat /dev/sdX1");
    println!("     cryptsetup open /dev/sdX1 datos_crypt");
    println!("     Trabaja después sobre /dev/mapper/datos_crypt, no sobre el contenedor.");
    println!("     cryptsetup resize, close y luksHeaderBackup requieren revisar la capa.");
    println!("     Sin la clave o la cabecera correcta, los datos cifrados no son recuperables.");
    println!();
    println!("   6.6 LVM");
    println!("     Flujo: pvcreate -> vgcreate -> lvcreate -> mkfs -> mount.");
    println!("     Ampliar: lvextend -r -L +10G /dev/VG/LV, y después verifica el FS.");
    println!("     Reducir: desmonta, comprueba, reduce FS, reduce LV y vuelve a montar.");
    println!("     pvremove, vgremove, lvremove y lvreduce pueden destruir volúmenes.");
    println!("     No uses GParted sobre un LV como si fuera una partición física.");
    println!();
    println!("   6.7 Btrfs");
    println!("     btrfs subvolume create/delete gestiona subvolúmenes.");
    println!("     btrfs subvolume snapshot crea copias; delete y snapshot -r requieren cuidado.");
    println!("     btrfs filesystem resize, balance, device add/remove y replace cambian el FS.");
    println!("     btrfs check --readonly es consulta; no uses --repair sin copia y diagnóstico.");
    println!();
    println!("   6.8 ZFS");
    println!("     zpool create/destroy/add/remove/replace gestiona el pool físico.");
    println!("     zpool scrub comprueba; zpool export/import desconecta y vuelve a importar.");
    println!("     zfs create/destroy, snapshot, rollback, set y rename gestionan datasets.");
    println!("     destroy y rollback pueden eliminar datos o volver atrás en el tiempo.");
    println!();
    println!("   6.9 RAID por software");
    println!("     mdadm --detail consulta; --create, --add, --fail y --remove cambian el array.");
    println!("     --grow puede cambiar nivel o tamaño; espera a que termine el rebuild.");
    println!("     No formatees ni retires un miembro sin revisar UUID, estado y redundancia.");
    println!();
    println!("   6.10 EFI, recuperación y borrado seguro");
    println!("     Conserva la partición EFI, su flag esp y sus montajes antes de editar.");
    println!("     Para una tabla dañada: deja de escribir, guarda un dump y usa recuperación");
    println!("     especializada como testdisk o ddrescue; no pruebes reparaciones al azar.");
    println!("     clean, clean all, wipefs -a, blkdiscard y dd son irreversibles en la práctica.");
    println!();
    println!("7. Redimensionar: el orden importa");
    println!("   Aumentar: primero amplía la partición y después el sistema de archivos.");
    println!("   Reducir: primero reduce el sistema de archivos y después la partición.");
    println!("   Nunca reduzcas sin copia de seguridad y sin confirmar el espacio usado.");
    println!("   ext4, xfs, btrfs y ntfs tienen reglas distintas; no intercambies sus pasos.");
    println!("   Una operación que mueve el inicio de una partición puede tardar mucho.");
    println!();
    println!("8. Capas avanzadas: identifica la capa antes de actuar");
    println!("   LUKS:       desbloquear el contenedor antes de trabajar con su contenido.");
    println!("   LVM:        PV -> VG -> LV -> sistema de archivos -> montaje.");
    println!("   Btrfs:      puede contener subvolúmenes y snapshots dentro del mismo FS.");
    println!("   ZFS:        pool y datasets; no trates cada miembro como un disco aislado.");
    println!("   RAID mdadm: conserva la alineación y el estado del conjunto completo.");
    println!("   NVMe:       el disco suele acabar en n1 y sus particiones en p1, p2, etc.");
    println!("   Consulta primero: ltools storage volume-stack");
    println!();
    println!("9. Comprobaciones después de un cambio");
    println!("   ltools storage partitions      árbol y tamaños actualizados.");
    println!("   ltools storage filesystems     UUID, etiquetas y tipos.");
    println!("   ltools storage mounts          montajes y opciones activas.");
    println!("   ltools storage usage           espacio e inodos disponibles.");
    println!("   ltools storage health /dev/sdX salud SMART cuando el dispositivo lo permite.");
    println!(
        "   No ejecutes fsck sobre un sistema montado ni confundas fsck -N con una reparación."
    );
    println!();
    println!("10. Señales para detenerse");
    println!("   El tamaño, modelo, UUID o etiqueta no coincide con lo esperado.");
    println!("   El objetivo está montado, es swap, pertenece a RAID/LVM/LUKS o contiene /home.");
    println!("   La herramienta propone borrar, limpiar, formatear o mover sin copia verificada.");
    println!("   No sabes si estás seleccionando el disco completo o una partición.");
    println!("   En cualquiera de estos casos, cancela y vuelve a inspeccionar.");
    println!();
    println!("Protecciones de LTools");
    println!("   Objetivos protegidos por defecto: /, /boot, /home, /usr, /var, /etc, /run");
    println!("   y las raíces de montaje. Las acciones sensibles exigen objetivo explícito,");
    println!("   confirmación y, cuando corresponde, autorización administrativa.");
    println!();
    println!("Acceso rápido: storage menu abre el flujo completo de gestión de almacenamiento.");
    Ok(())
}

fn mounts() -> Result<(), String> {
    println!("\n=== Montajes activos ===");
    if command_exists("findmnt") {
        run_capture("findmnt", &["-r", "-o", "SOURCE,TARGET,FSTYPE,OPTIONS"])
    } else if command_exists("df") {
        run_capture("df", &["-hT"])
    } else {
        Err("no se encontró findmnt ni df para consultar montajes".into())
    }
}

fn inspect(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    println!("=== Detalles de {} ===", target.display());
    if command_exists("lsblk") {
        run_capture_owned(
            "lsblk",
            &["-f".into(), "-p".into(), target.display().to_string()],
        )?;
    }
    if command_exists("blkid") {
        run_capture_owned("blkid", &[target.display().to_string()])?;
    } else {
        println!("blkid: no disponible; instala util-linux si necesitas UUID y tipo de sistema de archivos.");
    }
    Ok(())
}

fn mount(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    if !confirm_or_simulate(ctx, &format!("¿Montar {}?", target.display())) {
        println!("Montaje cancelado.");
        return Ok(());
    }
    let (program, args): (&str, Vec<String>) = if command_exists("udisksctl") {
        (
            "udisksctl",
            vec!["mount".into(), "-b".into(), target.display().to_string()],
        )
    } else {
        ("mount", vec![target.display().to_string()])
    };
    let ok = if program == "mount" {
        run_with_sudo(program, &args, ctx.dry_run)
    } else {
        run_command(program, &args, ctx.dry_run)
    }
    .map_err(|e| e.to_string())?;
    record(ctx, "storage-mount", &target, ok);
    if !ok {
        return Err(format!("no se pudo montar {}", target.display()));
    }
    Ok(())
}

fn unmount(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_unmount_target(raw, ctx.dry_run)?;
    if is_protected_mount(&target) {
        return Err(format!(
            "desmontaje bloqueado para una ruta crítica: {}",
            target.display()
        ));
    }
    if !confirm_or_simulate(ctx, &format!("¿Desmontar {}?", target.display())) {
        println!("Desmontaje cancelado.");
        return Ok(());
    }
    let (program, args): (&str, Vec<String>) =
        if command_exists("udisksctl") && target.starts_with("/dev/") {
            (
                "udisksctl",
                vec!["unmount".into(), "-b".into(), target.display().to_string()],
            )
        } else {
            ("umount", vec![target.display().to_string()])
        };
    let ok = run_with_sudo(program, &args, ctx.dry_run).map_err(|e| e.to_string())?;
    record(ctx, "storage-unmount", &target, ok);
    if !ok {
        return Err(format!("no se pudo desmontar {}", target.display()));
    }
    Ok(())
}

fn health(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    if !ensure_tool(ctx, "smartctl")? {
        println!("smartctl no está disponible; no se ejecutó la consulta SMART.");
        return Ok(());
    }
    println!("=== Salud SMART (solo lectura) ===");
    let ok = run_capture_owned(
        "smartctl",
        &["-H".into(), "-A".into(), target.display().to_string()],
    )
    .is_ok();
    if !ok {
        return Err("smartctl no pudo consultar el dispositivo".into());
    }
    Ok(())
}

fn filesystem_check(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    println!("=== Comprobación del sistema de archivos (no repara) ===");
    let ok = run_command(
        "fsck",
        &["-N".into(), target.display().to_string()],
        ctx.dry_run,
    )
    .map_err(|e| e.to_string())?;
    record(ctx, "storage-filesystem-check", &target, ok);
    if !ok {
        return Err("fsck -N no pudo analizar el dispositivo".into());
    }
    Ok(())
}

fn open_gparted(ctx: &Context, args: &[String]) -> Result<(), String> {
    if !command_exists("gparted") && !ensure_tool(ctx, "gparted")? {
        return Err("gparted no está instalado y la instalación fue cancelada".into());
    }
    let graphical_frontend = std::env::var_os("LTOOLS_FRONTEND")
        .is_some_and(|value| value == "gui")
        && (std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some());
    if graphical_frontend && !command_exists("pkexec") && !ensure_tool(ctx, "pkexec")? {
        return Err(
            "GParted necesita pkexec/polkit para autorizar su backend gráfico; instalación cancelada."
                .into(),
        );
    }
    // GParted ejecuta su backend como root mediante polkit. En sesiones
    // Wayland con Xwayland necesita `xhost` para conceder temporalmente al
    // usuario root acceso a DISPLAY; sin él la contraseña puede aceptarse y
    // aun así gpartedbin termina con "cannot open display".
    if std::env::var_os("DISPLAY").is_some()
        && !command_exists("xhost")
        && !ensure_tool(ctx, "xhost")?
    {
        return Err(
            "GParted necesita xhost para autorizar su ventana en esta sesión gráfica; instalación cancelada."
                .into(),
        );
    }
    if !args.iter().any(|arg| arg == "--yes")
        && !confirm_or_simulate(ctx, "¿Abrir GParted? Puede modificar particiones y datos.")
    {
        println!("Apertura cancelada.");
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se abriría gparted.");
        return Ok(());
    }

    // GParted inicia su backend privilegiado con polkit. En X11/Xwayland el
    // backend root no puede abrir el display del usuario por sí solo. Se
    // concede únicamente el permiso local para root, se lanza GParted con
    // sus pipes desacoplados de LTools y se revoca cuando el proceso termina.
    // No se usa `xhost +`, que abriría el display a cualquier cliente.
    let display_access = if std::env::var_os("DISPLAY").is_some() {
        grant_root_display_access()?;
        true
    } else {
        false
    };
    let mut command = Command::new("gparted");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if display_access {
        command.env("GDK_BACKEND", "x11");
    }
    let child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            if display_access {
                let _ = revoke_root_display_access();
            }
            return Err(format!("no se pudo abrir gparted: {error}"));
        }
    };
    if display_access {
        thread::spawn(move || {
            let mut child = child;
            let _ = child.wait();
            let _ = revoke_root_display_access();
        });
    }
    println!("GParted se ha iniciado. Las operaciones de particionado se realizan allí.");
    record(ctx, "storage-open-manager", Path::new("gparted"), true);
    Ok(())
}

fn grant_root_display_access() -> Result<bool, String> {
    let output = Command::new("xhost")
        .args(["+SI:localuser:root"])
        .output()
        .map_err(|error| format!("no se pudo ejecutar xhost para autorizar GParted: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if detail.is_empty() {
            "xhost no pudo conceder acceso temporal a root; GParted no se abrirá".into()
        } else {
            format!("xhost no pudo conceder acceso temporal a root: {detail}")
        });
    }
    Ok(true)
}

fn revoke_root_display_access() -> Result<(), String> {
    let output = Command::new("xhost")
        .args(["-SI:localuser:root"])
        .output()
        .map_err(|error| format!("no se pudo revocar el acceso temporal de GParted: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if detail.is_empty() {
            "xhost no pudo revocar el acceso temporal de root".into()
        } else {
            format!("xhost no pudo revocar el acceso temporal de root: {detail}")
        })
    }
}

fn open_path(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_path(raw)?;
    if !command_exists("xdg-open") {
        return Err("xdg-open no está disponible".into());
    }
    if !confirm_or_simulate(
        ctx,
        &format!("¿Abrir {} en el explorador?", target.display()),
    ) {
        return Ok(());
    }
    let ok = run_command("xdg-open", &[target.display().to_string()], ctx.dry_run)
        .map_err(|e| e.to_string())?;
    record(ctx, "storage-open-path", &target, ok);
    if !ok {
        return Err("no se pudo abrir la ruta".into());
    }
    Ok(())
}

fn usage() -> Result<(), String> {
    println!("=== Uso de espacio ===");
    require("df")?;
    run_capture("df", &["-hT", "-x", "tmpfs", "-x", "devtmpfs"])?;
    println!("\n=== Uso de inodos ===");
    run_capture("df", &["-hi", "-x", "tmpfs", "-x", "devtmpfs"])
}

fn filesystems() -> Result<(), String> {
    println!("=== Sistemas de archivos, UUID y etiquetas ===");
    if command_exists("lsblk") {
        run_capture(
            "lsblk",
            &[
                "-e7",
                "-fp",
                "-o",
                "NAME,FSTYPE,FSVER,LABEL,UUID,FSAVAIL,FSUSE%,MOUNTPOINTS",
            ],
        )?;
    }
    if command_exists("blkid") {
        println!("\n=== blkid ===");
        run_capture("blkid", &[])?;
    }
    Ok(())
}

fn volume_stack() -> Result<(), String> {
    println!("=== Capas de almacenamiento detectadas ===");
    let tools = [
        (
            "pvs",
            &["pvs", "--options", "pv_name,vg_name,pv_size,pv_free"][..],
        ),
        ("vgs", &["vgs", "--options", "vg_name,vg_size,vg_free"][..]),
        (
            "lvs",
            &["lvs", "--options", "lv_name,vg_name,lv_size,lv_attr"][..],
        ),
        ("cryptsetup", &["cryptsetup", "status"][..]),
        ("btrfs", &["btrfs", "filesystem", "show"][..]),
        ("zpool", &["zpool", "list"][..]),
        ("zfs", &["zfs", "list"][..]),
        ("mdadm", &["mdadm", "--detail", "--scan"][..]),
    ];
    let mut found = false;
    for (name, args) in tools {
        if command_exists(name) {
            found = true;
            println!("\n--- {name} ---");
            if let Err(error) = run_capture(name, &args[1..]) {
                println!("No se pudo consultar {name}: {error}");
            }
        }
    }
    if !found {
        println!("No se detectaron capas LVM, cifrado, Btrfs, ZFS o RAID adicionales.");
    }
    Ok(())
}

fn block_device_info(ctx: &Context, raw: &str) -> Result<(), String> {
    let target = validate_device(raw, ctx.dry_run)?;
    println!("=== Información de bloque: {} ===", target.display());
    if command_exists("blockdev") {
        for query in ["--getsize64", "--getss", "--getro"] {
            let output = Command::new("blockdev")
                .args([query, target.to_string_lossy().as_ref()])
                .output()
                .map_err(|error| format!("no se pudo ejecutar blockdev: {error}"))?;
            println!(
                "{query}: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            );
        }
    } else {
        println!("blockdev no está instalado; lsblk sigue disponible como alternativa.");
    }
    inspect(ctx, raw)
}

fn open_external_manager(ctx: &Context, program: &str, label: &str) -> Result<(), String> {
    if !command_exists(program) {
        println!("{program} no está instalado; puedes instalarlo bajo demanda desde doctor.");
        return Ok(());
    }
    if !confirm_or_simulate(
        ctx,
        &format!("¿Abrir {label}? Revisa cada cambio antes de aplicarlo."),
    ) {
        println!("Apertura cancelada.");
        return Ok(());
    }
    if ctx.dry_run {
        println!("Simulación: se abriría {program}.");
        return Ok(());
    }
    Command::new(program)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("no se pudo abrir {label}: {error}"))?;
    println!("{label} se ha iniciado; LTools no aplica cambios destructivos automáticamente.");
    Ok(())
}

fn managers_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Gestores nativos de almacenamiento ===");
        println!("  1) GParted");
        println!("  2) GNOME Disks");
        println!("  3) KDE Partition Manager");
        println!("  4) Abrir una ruta en el explorador");
        println!("  5) Herramientas detectadas");
        println!("  q) Volver");
        let choice =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        match choice.trim() {
            "1" => open_gparted(ctx, &[])?,
            "2" => open_external_manager(ctx, "gnome-disks", "GNOME Disks")?,
            "3" => open_external_manager(ctx, "partitionmanager", "KDE Partition Manager")?,
            "4" => prompt_then(ctx, "Ruta: ", open_path)?,
            "5" => tools()?,
            "" | "q" | "Q" => return Ok(()),
            _ => println!("Opción no válida."),
        }
        if choice.trim() != "" {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn advanced_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("=== Consultas avanzadas de almacenamiento ===");
        println!("  1) Uso de espacio e inodos");
        println!("  2) Sistemas de archivos, UUID y etiquetas");
        println!("  3) LVM, cifrado, Btrfs, ZFS y RAID");
        println!("  4) Información detallada de un dispositivo");
        println!("  5) Gestores nativos");
        println!("  q) Volver");
        let choice =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        match choice.trim() {
            "1" => usage()?,
            "2" => filesystems()?,
            "3" => volume_stack()?,
            "4" => prompt_then(ctx, "Dispositivo (ej. /dev/sdb1): ", block_device_info)?,
            "5" => managers_menu(ctx)?,
            "" | "q" | "Q" => return Ok(()),
            _ => println!("Opción no válida."),
        }
        if choice.trim() != "" {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

fn tools() -> Result<(), String> {
    println!("=== Herramientas de almacenamiento Linux ===");
    for name in [
        "lsblk",
        "findmnt",
        "df",
        "blkid",
        "mount",
        "umount",
        "udisksctl",
        "parted",
        "fdisk",
        "sfdisk",
        "gdisk",
        "blockdev",
        "fsck",
        "smartctl",
        "nvme",
        "cryptsetup",
        "pvs",
        "vgs",
        "lvs",
        "btrfs",
        "zpool",
        "zfs",
        "mdadm",
        "gparted",
        "gnome-disks",
        "partitionmanager",
        "duf",
        "ncdu",
        "xdg-open",
    ] {
        println!(
            "{name}: {}",
            if command_exists(name) {
                "disponible"
            } else {
                "no instalado"
            }
        );
    }
    println!("\nConsultas: status, partitions, mounts, usage, filesystems, volume-stack, inspect, blockdev, health, check.");
    println!("Acciones con confirmación: mount, unmount, open-gparted, open.");
    Ok(())
}

fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("\n=== Gestión de discos y almacenamiento Linux ===");
        println!("  1) Resumen de espacio y montajes");
        println!("  2) Discos, particiones y tablas");
        println!("  3) Montajes activos");
        println!("  4) Inspeccionar un dispositivo");
        println!("  5) Montar un dispositivo");
        println!("  6) Desmontar un dispositivo o ruta");
        println!("  7) Salud SMART");
        println!("  8) Comprobar sistema de archivos (no repara)");
        println!("  9) Abrir GParted");
        println!(" 10) Abrir una ruta");
        println!(" 11) Herramientas detectadas");
        println!(" 12) Seleccionar un objetivo y elegir una acción");
        println!("\n Consultas avanzadas de almacenamiento");
        println!(" 13) Uso, inodos, UUID y etiquetas");
        println!(" 14) LVM, cifrado, Btrfs, ZFS y RAID");
        println!(" 15) Gestores nativos alternativos");
        println!(" 16) Tabla de particiones de un dispositivo (solo lectura)");
        println!(" 17) Guía de operaciones y protecciones");
        println!("  q) Volver");
        let answer =
            crate::menu_input("Elige una opción (Enter para volver): ").unwrap_or_default();
        match answer.trim() {
            "1" => status(ctx)?,
            "2" => partitions(ctx)?,
            "3" => mounts()?,
            "4" => prompt_then(ctx, "Dispositivo (ej. /dev/sdb1): ", inspect)?,
            "5" => prompt_then(ctx, "Dispositivo a montar: ", mount)?,
            "6" => prompt_then(ctx, "Dispositivo o ruta a desmontar: ", unmount)?,
            "7" => prompt_then(ctx, "Disco (ej. /dev/sda): ", health)?,
            "8" => prompt_then(ctx, "Dispositivo a comprobar: ", filesystem_check)?,
            "9" => open_gparted(ctx, &[])?,
            "10" => prompt_then(ctx, "Ruta: ", open_path)?,
            "11" => tools()?,
            "12" => guided_target(ctx)?,
            "13" => advanced_menu(ctx)?,
            "14" => volume_stack()?,
            "15" => managers_menu(ctx)?,
            "16" => prompt_then(ctx, "Dispositivo (ej. /dev/sda): ", partition_table)?,
            "17" => partition_guide()?,
            "q" | "Q" | "" => return Ok(()),
            _ => println!("Opción no válida."),
        }
        if answer.trim() != "" {
            let _ = crate::menu_input("Pulsa Enter para continuar...");
        }
    }
}

/// Flujo orientado a personas: primero limita la elección a dispositivos
/// detectados y después muestra solo acciones que aceptan ese tipo de
/// objetivo. No incluye borrado ni particionado automatizado: esas acciones
/// se delegan al gestor nativo, donde el usuario puede revisar cada cambio.
fn guided_target(ctx: &Context) -> Result<(), String> {
    let mut targets = Vec::new();
    if command_exists("lsblk") {
        if let Ok(output) = Command::new("lsblk")
            .args(["-nrpo", "NAME,TYPE,SIZE,FSTYPE,MOUNTPOINTS"])
            .output()
        {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                if fields.len() >= 2 && matches!(fields[1], "disk" | "part" | "crypt") {
                    targets.push(fields.join(" | "));
                }
            }
        }
    }
    println!("=== Selección segura de almacenamiento ===");
    if targets.is_empty() {
        println!("No se detectaron dispositivos; puedes introducir una ruta /dev/... manualmente.");
    } else {
        for (index, target) in targets.iter().enumerate() {
            println!("  {}) {}", index + 1, target);
        }
    }
    let Some(answer) = crate::menu_input("Número, ruta /dev/... o Enter para volver: ") else {
        return Ok(());
    };
    if answer.is_empty() {
        return Ok(());
    }
    let target = if let Ok(index) = answer.parse::<usize>() {
        targets
            .get(index.saturating_sub(1))
            .and_then(|value| value.split('|').next())
            .map(str::trim)
            .unwrap_or_default()
            .to_string()
    } else {
        answer
    };
    validate_device(&target, ctx.dry_run)?;
    loop {
        crate::clear_screen();
        println!("=== Acciones para {} ===", target);
        println!("  1) Inspeccionar detalles");
        println!("  2) Consultar salud SMART");
        println!("  3) Comprobar sistema de archivos (no repara)");
        println!("  4) Montar (requiere confirmación)");
        println!("  5) Desmontar (requiere confirmación y bloquea rutas críticas)");
        println!("  6) Abrir GParted para particionar manualmente");
        println!("  q) Volver");
        let choice =
            crate::menu_input("Elige una acción (Enter para volver): ").unwrap_or_default();
        let result = match choice.as_str() {
            "1" => inspect(ctx, &target),
            "2" => health(ctx, &target),
            "3" => filesystem_check(ctx, &target),
            "4" => mount(ctx, &target),
            "5" => unmount(ctx, &target),
            "6" => open_gparted(ctx, &[]),
            "" | "q" | "quit" | "salir" => return Ok(()),
            _ => {
                println!("Opción no válida.");
                Ok(())
            }
        };
        if let Err(error) = result {
            println!("Error: {error}");
        }
        let _ = crate::menu_input("Pulsa Enter para continuar...");
    }
}

fn prompt_then(
    ctx: &Context,
    question: &str,
    action: fn(&Context, &str) -> Result<(), String>,
) -> Result<(), String> {
    if let Some(value) = crate::common::prompt_path(question) {
        action(ctx, &value.to_string_lossy())?;
    }
    Ok(())
}

fn validate_device(raw: &str, dry_run: bool) -> Result<PathBuf, String> {
    if raw.chars().any(|c| c.is_control()) {
        return Err("el dispositivo contiene caracteres de control".into());
    }
    let value = raw.trim();
    if value.is_empty()
        || value.chars().any(|c| {
            c.is_control()
                || c.is_whitespace()
                || !c.is_ascii_alphanumeric() && !"/._-:".contains(c)
        })
        || value.split('/').any(|part| matches!(part, "." | ".."))
        || !value.starts_with("/dev/")
    {
        return Err("el dispositivo debe ser una ruta /dev/... con caracteres válidos".into());
    }
    let path = PathBuf::from(value);
    if !dry_run && !path.exists() {
        return Err(format!("no existe el dispositivo: {value}"));
    }
    Ok(path)
}

fn validate_unmount_target(raw: &str, dry_run: bool) -> Result<PathBuf, String> {
    if raw.chars().any(|c| c.is_control()) {
        return Err("la ruta contiene caracteres de control".into());
    }
    let value = raw.trim();
    if value.is_empty()
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
        || value.split('/').any(|part| matches!(part, "." | ".."))
        || (!value.starts_with("/dev/") && !value.starts_with('/'))
    {
        return Err("indica un dispositivo /dev/... o una ruta absoluta de montaje".into());
    }
    let path = PathBuf::from(value);
    if !dry_run && !path.exists() {
        return Err(format!("no existe la ruta: {value}"));
    }
    Ok(path)
}

fn validate_path(raw: &str) -> Result<PathBuf, String> {
    let value = raw.trim();
    if value.is_empty() || value.chars().any(|c| c.is_control()) {
        return Err("ruta vacía o no válida".into());
    }
    Ok(PathBuf::from(value))
}
fn is_protected_mount(path: &Path) -> bool {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if matches!(
        canonical.to_string_lossy().as_ref(),
        "/" | "/home" | "/boot" | "/usr" | "/var" | "/etc" | "/run"
    ) {
        return true;
    }
    // Un dispositivo como /dev/nvme0n1p1 puede ser la fuente de `/` aunque
    // no tenga una ruta crítica como destino. Resolverlo evita que el selector
    // seguro pueda desmontar la raíz por haber elegido el nombre del bloque.
    if path.to_string_lossy().starts_with("/dev/") {
        if !command_exists("findmnt") {
            // Sin una respuesta fiable, bloquear es más seguro que adivinar.
            return true;
        }
        let output = Command::new("findmnt")
            .args(["-rn", "-S"])
            .arg(path)
            .args(["-o", "TARGET"])
            .output();
        let Ok(output) = output else {
            return true;
        };
        if !output.status.success() {
            return true;
        }
        return String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|target| {
                matches!(
                    target.trim(),
                    "/" | "/home" | "/boot" | "/usr" | "/var" | "/etc" | "/run"
                )
            });
    }
    false
}
fn confirm_or_simulate(ctx: &Context, question: &str) -> bool {
    ctx.dry_run || crate::common::ask(question)
}
fn record(ctx: &Context, operation: &str, target: &Path, ok: bool) {
    if let Some(plan) = &ctx.plan {
        let _ = plan.record(
            operation,
            target,
            if ok { "executed" } else { "failed" },
            false,
            "",
            "",
        );
    }
}
fn require(name: &str) -> Result<(), String> {
    if command_exists(name) {
        Ok(())
    } else {
        Err(format!(
            "{name} no está instalado; usa doctor --install {name} si tu distribución lo ofrece"
        ))
    }
}
fn run_capture(program: &str, args: &[&str]) -> Result<(), String> {
    run_capture_owned(
        program,
        &args
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>(),
    )
}
fn run_capture_owned(program: &str, args: &[String]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("no se pudo ejecutar {program}: {e}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if error.is_empty() {
            format!("{program} terminó con código {}", output.status)
        } else {
            error
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn solo_acepta_dispositivos_linux() {
        assert!(validate_device("/dev/sda", true).is_ok());
        assert!(validate_device("/etc/passwd", true).is_err());
        assert!(validate_device("/dev/sda;rm", true).is_err());
    }
    #[test]
    fn rechaza_espacios_y_control_en_destinos() {
        assert!(validate_device("/dev/sda 1", true).is_err());
        assert!(validate_unmount_target("/dev/sda\n", true).is_err());
        assert!(validate_device("/dev/../etc/passwd", true).is_err());
        assert!(validate_unmount_target("/run/../etc", true).is_err());
    }
    #[test]
    fn protege_montajes_criticos() {
        assert!(is_protected_mount(Path::new("/")));
        assert!(is_protected_mount(Path::new("/home")));
        assert!(!is_protected_mount(Path::new("/mnt/data")));
    }
}
