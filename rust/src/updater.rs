//! Comprobación y preparación segura de actualizaciones publicadas en GitHub.
//!
//! La aplicación nunca se eleva para actualizarse ni ejecuta automáticamente
//! el artefacto descargado. Primero verifica el manifiesto firmado y los
//! SHA-256; después deja el paquete listo para que el usuario sustituya su
//! instalación portable o abra el instalador/gestor apropiado.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const METADATA_LIMIT: u64 = 4 * 1024 * 1024;
const MAX_ARTIFACT_SIZE: u64 = 2 * 1024 * 1024 * 1024;

struct UpdateWorkspace(PathBuf);

impl Drop for UpdateWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Debug)]
struct ReleaseArtifact {
    filename: String,
    download_url: String,
    size_bytes: u64,
    sha256: String,
}

struct LatestRelease {
    workspace: UpdateWorkspace,
    version: String,
    page: String,
    artifact: ReleaseArtifact,
    verified: bool,
}

pub fn help() -> &'static str {
    "update [check|download] [--repository OWNER/REPO] [--pause] — comprueba una release GitHub, verifica firma/hash y prepara el paquete sin privilegios"
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args
        .iter()
        .any(|value| matches!(value.as_str(), "--help" | "-h" | "help"))
    {
        println!("{}", help());
        println!("  check      consulta la última release estable; no instala ni cambia archivos");
        println!("  download   descarga solo si manifiesto y firma/hash son válidos");
        println!("  --repository OWNER/REPO  reemplaza el repositorio predeterminado");
        println!("  --pause    espera Enter al final, útil desde una ventana GUI");
        return Ok(());
    }

    let result = run_request(args);
    if args.iter().any(|value| value == "--pause") {
        if let Err(error) = &result {
            eprintln!("Error: {error}");
        }
        print!("\nPulsa Enter para cerrar esta consulta...");
        let _ = io::stdout().flush();
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }
    result
}

fn run_request(args: &[String]) -> Result<(), String> {
    let verb_args = positionals(args);
    if verb_args.len() > 1 {
        return Err("update acepta una sola acción: check o download".into());
    }
    for (index, argument) in args.iter().enumerate() {
        if argument == "--repository"
            && args
                .get(index + 1)
                .is_none_or(|value| value.starts_with('-'))
        {
            return Err("--repository requiere OWNER/REPO".into());
        }
        if argument.starts_with('-')
            && argument != "--pause"
            && argument != "--repository"
            && !argument.starts_with("--repository=")
        {
            return Err(format!("opción update desconocida: {argument}"));
        }
    }
    let verb = verb_args.first().copied().unwrap_or("check");
    let repository = option(args, "--repository")
        .or_else(|| std::env::var("LTOOLS_GITHUB_REPOSITORY").ok())
        .or_else(|| std::env::var("LTERMINAL_GITHUB_REPOSITORY").ok())
        .unwrap_or_else(|| "Darkeiser003/Tools".to_owned());
    validate_repository(&repository)?;
    match verb {
        "check" | "status" => check_for_update(&repository),
        "download" => download_update(&repository),
        "install" => Err("la sustitución automática no está habilitada; usa `update download` y sigue las instrucciones para tu instalación portable".into()),
        _ => Err(format!("acción update desconocida: {verb}; usa `ltools update --help`")),
    }
}

fn check_for_update(repository: &str) -> Result<(), String> {
    let release = fetch_latest(repository)?;
    report_release(&release);
    if release.verified {
        println!("Integridad: firma Ed25519 y checksum del manifiesto verificados.");
    } else {
        println!("Integridad: no verificada localmente (falta configurar la clave pública); solo se informa, no se permite descargar.");
    }
    if version_is_newer(&release.version, crate::VERSION)? {
        println!(
            "Hay una versión más reciente: {} (actual {}).",
            release.version,
            crate::VERSION
        );
        println!("Ejecuta `ltools update download` para descargar el paquete verificado; la aplicación no se sustituye automáticamente.");
    } else {
        println!("LTools está actualizado ({}).", crate::VERSION);
    }
    Ok(())
}

fn download_update(repository: &str) -> Result<(), String> {
    let release = fetch_latest(repository)?;
    report_release(&release);
    if !version_is_newer(&release.version, crate::VERSION)? {
        println!(
            "No hay una versión posterior a {} que descargar.",
            crate::VERSION
        );
        return Ok(());
    }
    if !release.verified {
        return Err("descarga cancelada: no se pudo verificar una firma Ed25519 con la clave pública configurada".into());
    }

    let checksums = fs::read(release.workspace.0.join("SHA256SUMS.txt"))
        .map_err(|error| format!("no se pudo leer SHA256SUMS.txt descargado: {error}"))?;
    let expected_checksum = checksum_for(&checksums, &release.artifact.filename)?;
    if expected_checksum != release.artifact.sha256 {
        return Err(
            "el hash del artefacto en ltools-release.json no coincide con SHA256SUMS.txt".into(),
        );
    }
    let download_directory = downloads_directory()?;
    let destination =
        download_verified_artifact(&release.artifact, &expected_checksum, &download_directory)?;
    println!("Paquete descargado y verificado: {}", destination.display());
    println!("No se ejecutó ni reemplazó nada. Cierra LTools antes de sustituir un ejecutable; conserva una copia de seguridad y usa el procedimiento de tu instalación portable.");
    println!("Release: {}", release.page);
    Ok(())
}

fn download_verified_artifact(
    artifact: &ReleaseArtifact,
    expected_checksum: &str,
    download_directory: &Path,
) -> Result<PathBuf, String> {
    if expected_checksum != artifact.sha256 {
        return Err("el hash esperado no coincide con el artefacto del manifiesto".into());
    }
    fs::create_dir_all(download_directory).map_err(|error| {
        format!(
            "no se pudo crear la carpeta de descargas {}: {error}",
            download_directory.display()
        )
    })?;
    // Mantén el paquete sin verificar en un directorio 0700, fuera de
    // Descargas: no debe poder sustituirse mediante una carrera en un destino
    // compartido entre el hash y la promoción final.
    let temporary_workspace = create_workspace()?;
    let temporary = temporary_workspace.0.join("download.part");
    let download_result = (|| {
        download_artifact(artifact, &temporary)?;
        let actual_size = fs::metadata(&temporary)
            .map_err(|error| format!("no se pudo inspeccionar el artefacto descargado: {error}"))?
            .len();
        if actual_size != artifact.size_bytes {
            return Err(format!(
                "tamaño descargado incorrecto: {} bytes, manifiesto declara {}",
                actual_size, artifact.size_bytes
            ));
        }
        let actual_hash = sha256_file(&temporary)?;
        if actual_hash != artifact.sha256 || actual_hash != expected_checksum {
            return Err("el artefacto descargado no coincide con los hashes firmados".into());
        }
        #[cfg(unix)]
        if artifact.filename.ends_with(".AppImage") {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&temporary)
                .map_err(|error| format!("no se pudieron leer permisos del artefacto: {error}"))?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&temporary, permissions)
                .map_err(|error| format!("no se pudieron activar permisos de AppImage: {error}"))?;
        }
        promote_without_overwrite(&temporary, download_directory, &artifact.filename)
    })();
    if download_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    let destination = download_result?;

    Ok(destination)
}

fn report_release(release: &LatestRelease) {
    println!("LTools actual: {}", crate::VERSION);
    println!("Última release: {}", release.version);
    println!(
        "Plataforma/paquete compatible: {}",
        release.artifact.filename
    );
    println!("Página: {}", release.page);
}

fn fetch_latest(repository: &str) -> Result<LatestRelease, String> {
    let base = metadata_base_url(repository)?;
    fetch_latest_at(repository, &base, None)
}

fn fetch_latest_at(
    repository: &str,
    base: &str,
    public_key_override: Option<&Path>,
) -> Result<LatestRelease, String> {
    let workspace = create_workspace()?;
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(25)))
        .build()
        .into();
    let manifest_bytes = fetch_bytes(
        &agent,
        &format!("{base}/ltools-release.json"),
        METADATA_LIMIT,
    )?;
    let checksum_bytes = fetch_bytes(&agent, &format!("{base}/SHA256SUMS.txt"), METADATA_LIMIT)?;
    let signature_bytes = fetch_bytes(&agent, &format!("{base}/SHA256SUMS.txt.sig"), 64 * 1024)?;
    let manifest_path = workspace.0.join("ltools-release.json");
    let checksum_path = workspace.0.join("SHA256SUMS.txt");
    let signature_path = workspace.0.join("SHA256SUMS.txt.sig");
    write_private(&manifest_path, &manifest_bytes)?;
    write_private(&checksum_path, &checksum_bytes)?;
    write_private(&signature_path, &signature_bytes)?;

    let parsed: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("ltools-release.json no contiene JSON válido: {error}"))?;
    let version = string_field(&parsed, "version")?.to_owned();
    let tag = string_field(&parsed, "tag")?;
    let release_page = string_field(&parsed, "release_page")?.to_owned();
    if string_field(&parsed, "schema")? != "ltools-release-v1"
        || !matches!(
            string_field(&parsed, "application")?,
            "LTools" | "WinSlim-Tools"
        )
        || string_field(&parsed, "repository")? != repository
        || string_field(&parsed, "manifest_name")? != "ltools-release.json"
        || string_field(&parsed, "hash_algorithm")? != "sha256"
        || !valid_tag(tag)
    {
        return Err(
            "el manifiesto no coincide con el contrato de release y repositorio esperado".into(),
        );
    }
    validate_repository(repository)?;
    validate_version(&version)?;
    validate_release_page(repository, tag, &release_page)?;
    let manifest_hash = hex(&Sha256::digest(&manifest_bytes));
    if checksum_for(&checksum_bytes, "ltools-release.json")? != manifest_hash {
        return Err("el manifiesto descargado no coincide con el SHA-256 publicado".into());
    }

    let artifact = select_artifact(&parsed, repository, &version, tag)?;
    let key_available = public_key_override.is_some() || public_key_available();
    let verified = if key_available {
        let mut arguments = vec![
            "--manifest".to_owned(),
            checksum_path.to_string_lossy().into_owned(),
            "--signature".to_owned(),
            signature_path.to_string_lossy().into_owned(),
            "--verify".to_owned(),
        ];
        let public_key_path = if let Some(path) = public_key_override {
            path.to_path_buf()
        } else if let Some(key) = configured_public_key_value() {
            let path = workspace.0.join("release-signing-public.hex");
            write_private(&path, key.as_bytes())?;
            path
        } else if let Some(path) = configured_public_key_file() {
            path
        } else if let Some(key) = option_env!("LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY") {
            let path = workspace.0.join("release-signing-public.hex");
            write_private(&path, key.as_bytes())?;
            path
        } else {
            let path = workspace.0.join("release-signing-public.hex");
            let default_path = default_public_key_file()
                .ok_or_else(|| "no hay clave pública de release disponible".to_owned())?;
            fs::copy(&default_path, &path).map_err(|error| {
                format!(
                    "no se pudo copiar la clave pública {}: {error}",
                    default_path.display()
                )
            })?;
            path
        };
        arguments.extend([
            "--public-key-file".to_owned(),
            public_key_path.to_string_lossy().into_owned(),
        ]);
        crate::signature::run(&arguments)
            .map_err(|error| format!("firma de release inválida: {error}"))?;
        true
    } else {
        false
    };
    let listed_artifact_hash = checksum_for(&checksum_bytes, &artifact.filename)?;
    if listed_artifact_hash != artifact.sha256 {
        return Err(
            "el SHA-256 del artefacto en ltools-release.json no coincide con SHA256SUMS.txt".into(),
        );
    }
    Ok(LatestRelease {
        workspace,
        version,
        page: release_page,
        artifact,
        verified,
    })
}

fn select_artifact(
    manifest: &Value,
    repository: &str,
    version: &str,
    tag: &str,
) -> Result<ReleaseArtifact, String> {
    let (platform, architecture, kind) = platform_artifact();
    if architecture == "unsupported" {
        return Err(format!(
            "no hay artefacto publicado para la arquitectura {}",
            std::env::consts::ARCH
        ));
    }
    let entries = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| "el manifiesto no contiene una lista de artefactos".to_owned())?;
    let matches: Vec<&Value> = entries
        .iter()
        .filter(|entry| {
            entry.get("platform").and_then(Value::as_str) == Some(platform)
                && entry.get("architecture").and_then(Value::as_str) == Some(architecture)
                && entry.get("kind").and_then(Value::as_str) == Some(kind)
        })
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "se esperaba un único paquete {platform}/{architecture}/{kind}; encontrados: {}",
            matches.len()
        ));
    }
    let entry = matches[0];
    let filename = string_field(entry, "filename")?.to_owned();
    let expected_filename = match (platform, kind) {
        ("linux", "appimage") => format!("ltools-{version}-linux-{architecture}.AppImage"),
        ("linux", "appimage-cli") => format!("ltools-{version}-linux-{architecture}-cli.AppImage"),
        ("linux", "tarball") => format!("ltools-{version}-linux-{architecture}.tar.gz"),
        ("windows", "exe") => format!("ltools-{version}-windows-{architecture}.exe"),
        ("windows", "exe-cli") => format!("ltools-{version}-windows-{architecture}-cli.exe"),
        _ => {
            return Err(format!(
                "combinación de paquete no admitida: {platform}/{kind}"
            ))
        }
    };
    if filename != expected_filename {
        return Err(
            "el nombre del artefacto no coincide con el formato publicado para esta plataforma"
                .into(),
        );
    }
    let expected_url = format!(
        "https://github.com/{repository}/releases/download/{tag}/{}",
        encode_release_filename(&filename)
    );
    let download_url = string_field(entry, "download_url")?.to_owned();
    if download_url != expected_url {
        return Err("la URL del paquete no corresponde al repositorio/tag GitHub esperado".into());
    }
    let size_bytes = entry
        .get("size_bytes")
        .and_then(Value::as_u64)
        .filter(|size| *size > 0 && *size <= MAX_ARTIFACT_SIZE)
        .ok_or_else(|| "tamaño de artefacto inválido o superior al límite de 2 GiB".to_owned())?;
    let sha256 = string_field(entry, "sha256")?.to_ascii_lowercase();
    if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("el SHA-256 del artefacto no tiene 64 caracteres hexadecimales".into());
    }
    Ok(ReleaseArtifact {
        filename,
        download_url,
        size_bytes,
        sha256,
    })
}

fn artifact_architecture(platform: &str, architecture: &str) -> &'static str {
    match (platform, architecture) {
        ("windows", "x86_64") => "x86_64",
        ("windows", "aarch64") => "arm64",
        ("windows", "x86") => "x86",
        ("linux", "x86_64") => "x86_64",
        ("linux", "aarch64") => "aarch64",
        ("linux", "x86") => "i686",
        ("linux", "arm" | "armv7" | "armv7l") => "armv7l",
        _ => "unsupported",
    }
}

fn platform_artifact() -> (&'static str, &'static str, &'static str) {
    #[cfg(windows)]
    let platform = "windows";
    #[cfg(not(windows))]
    let platform = "linux";

    let architecture = artifact_architecture(platform, std::env::consts::ARCH);
    #[cfg(windows)]
    let kind = if cfg!(feature = "cli") {
        "exe-cli"
    } else {
        "exe"
    };
    #[cfg(not(windows))]
    let kind = if std::env::var_os("APPIMAGE").is_some() {
        if cfg!(feature = "cli") {
            "appimage-cli"
        } else {
            "appimage"
        }
    } else {
        "tarball"
    };
    (platform, architecture, kind)
}

fn metadata_base_url(repository: &str) -> Result<String, String> {
    if let Ok(test_url) = std::env::var("LTOOLS_UPDATE_TEST_BASE_URL") {
        return validate_test_base_url(&test_url);
    }
    Ok(format!(
        "https://github.com/{repository}/releases/latest/download"
    ))
}

fn fetch_bytes(agent: &ureq::Agent, url: &str, limit: u64) -> Result<Vec<u8>, String> {
    let mut response = agent
        .get(url)
        .header("User-Agent", concat!("LTools/", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| format!("no se pudo consultar {url}: {error}"))?;
    response
        .body_mut()
        .with_config()
        .limit(limit)
        .read_to_vec()
        .map_err(|error| format!("respuesta demasiado grande o ilegible desde {url}: {error}"))
}

fn download_artifact(artifact: &ReleaseArtifact, path: &Path) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|error| {
        format!(
            "no se pudo reservar el temporal de descarga {}: {error}",
            path.display()
        )
    })?;
    let result = (|| {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(120)))
            .build()
            .into();
        let mut response = agent
            .get(&artifact.download_url)
            .header("User-Agent", concat!("LTools/", env!("CARGO_PKG_VERSION")))
            .call()
            .map_err(|error| format!("no se pudo descargar el paquete verificado: {error}"))?;
        let mut reader = response
            .body_mut()
            .with_config()
            .limit(artifact.size_bytes + 1)
            .reader();
        let copied = io::copy(&mut reader, &mut file)
            .map_err(|error| format!("falló la escritura del paquete: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("no se pudo vaciar el paquete al disco: {error}"))?;
        if copied != artifact.size_bytes {
            return Err(format!(
                "tamaño descargado inesperado: {copied} bytes; esperado {}",
                artifact.size_bytes
            ));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        File::open(path).map_err(|error| format!("no se pudo leer {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("falló la lectura de {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex(&hasher.finalize()))
}

fn checksum_for(contents: &[u8], filename: &str) -> Result<String, String> {
    let text =
        std::str::from_utf8(contents).map_err(|_| "SHA256SUMS.txt no es UTF-8".to_owned())?;
    let mut found = None;
    for line in text.lines() {
        let Some((hash, name)) = line.split_once("  ") else {
            continue;
        };
        let name = name.strip_prefix('*').unwrap_or(name);
        if name == filename {
            let normalized = hash.to_ascii_lowercase();
            if normalized.len() == 64 && normalized.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                if found.replace(normalized).is_some() {
                    return Err(format!(
                        "SHA256SUMS.txt contiene más de una entrada para {filename}"
                    ));
                }
            } else {
                return Err(format!("checksum inválido para {filename}"));
            }
        }
    }
    found.ok_or_else(|| format!("SHA256SUMS.txt no contiene {filename}"))
}

fn version_is_newer(candidate: &str, current: &str) -> Result<bool, String> {
    let candidate = semver::Version::parse(candidate.strip_prefix('v').unwrap_or(candidate))
        .map_err(|_| format!("versión semver no compatible: {candidate}"))?;
    let current = semver::Version::parse(current.strip_prefix('v').unwrap_or(current))
        .map_err(|_| format!("versión semver no compatible: {current}"))?;
    Ok(candidate.cmp_precedence(&current).is_gt())
}

fn validate_version(value: &str) -> Result<(), String> {
    semver::Version::parse(value.strip_prefix('v').unwrap_or(value))
        .map_err(|_| format!("versión semver no compatible: {value}"))?;
    if value
        .bytes()
        .any(|byte| !(byte.is_ascii_alphanumeric() || b".-+".contains(&byte)))
    {
        return Err("versión con caracteres no permitidos".into());
    }
    Ok(())
}

fn validate_test_base_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim_end_matches('/');
    let Some(authority_and_path) = trimmed.strip_prefix("http://") else {
        return Err("LTOOLS_UPDATE_TEST_BASE_URL solo admite HTTP loopback".into());
    };
    let authority = authority_and_path.split('/').next().unwrap_or_default();
    let Some((host, port)) = authority.rsplit_once(':') else {
        return Err("LTOOLS_UPDATE_TEST_BASE_URL requiere puerto explícito".into());
    };
    let valid_port = port.parse::<u16>().ok().is_some_and(|port| port != 0);
    if !matches!(host, "127.0.0.1" | "localhost")
        || !valid_port
        || trimmed
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte == b'#' || byte == b'?')
    {
        return Err("LTOOLS_UPDATE_TEST_BASE_URL solo admite un servidor loopback HTTP".into());
    }
    Ok(trimmed.to_owned())
}

fn validate_repository(repository: &str) -> Result<(), String> {
    let mut parts = repository.split('/');
    let owner = parts.next().unwrap_or_default();
    let name = parts.next().unwrap_or_default();
    let safe = |part: &str| {
        !part.is_empty()
            && part != "."
            && part != ".."
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
    };
    if parts.next().is_some() || !safe(owner) || !safe(name) {
        return Err("--repository debe tener formato OWNER/REPO con caracteres seguros".into());
    }
    Ok(())
}

fn validate_release_page(repository: &str, tag: &str, page: &str) -> Result<(), String> {
    let expected = format!("https://github.com/{repository}/releases/tag/{tag}");
    if page == expected {
        Ok(())
    } else {
        Err("release_page no apunta exactamente al repositorio y tag declarados".into())
    }
}

fn valid_tag(tag: &str) -> bool {
    let Some(version) = tag.strip_prefix('v') else {
        return false;
    };
    !version.is_empty()
        && !version.starts_with('.')
        && !version.ends_with('.')
        && !version.contains("..")
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
}

fn encode_release_filename(filename: &str) -> String {
    filename
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F")
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("el manifiesto no contiene el campo de texto {field}"))
}

fn configured_public_key_file() -> Option<PathBuf> {
    for name in [
        "LTOOLS_UPDATE_PUBLIC_KEY_FILE",
        "LTERMINAL_UPDATE_PUBLIC_KEY_FILE",
    ] {
        if let Some(path) = std::env::var_os(name)
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
        {
            return Some(path);
        }
    }
    None
}

fn default_public_key_file() -> Option<PathBuf> {
    let config = nonempty_path_environment("LTOOLS_CONFIG_HOME")
        .or_else(|| nonempty_path_environment("XDG_CONFIG_HOME"))
        .or_else(|| nonempty_path_environment("USERPROFILE").map(|home| home.join(".config")))
        .or_else(|| nonempty_path_environment("HOME").map(|home| home.join(".config")))?;
    let path = config.join("lterminal").join("release-signing-public.hex");
    path.is_file().then_some(path)
}

fn configured_public_key_value() -> Option<String> {
    std::env::var("LTOOLS_UPDATE_PUBLIC_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("LTERMINAL_UPDATE_PUBLIC_KEY")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

fn public_key_available() -> bool {
    configured_public_key_file().is_some()
        || configured_public_key_value().is_some()
        || option_env!("LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY").is_some()
        || default_public_key_file().is_some()
}

fn nonempty_path_environment(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn create_workspace() -> Result<UpdateWorkspace, String> {
    let path = (0..8)
        .map(|_| {
            std::env::temp_dir().join(format!("ltools-update-{}-{}", std::process::id(), nonce()))
        })
        .find(|candidate| !candidate.exists())
        .ok_or_else(|| "no se pudo reservar un espacio temporal de actualización".to_owned())?;
    fs::create_dir(&path).map_err(|error| {
        format!("no se pudo crear el espacio temporal de actualización: {error}")
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).map_err(|error| {
            let _ = fs::remove_dir(&path);
            format!("no se pudo proteger el espacio temporal: {error}")
        })?;
    }
    Ok(UpdateWorkspace(path))
}

fn write_private(path: &Path, contents: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|error| format!("no se pudo crear {}: {error}", path.display()))?;
    file.write_all(contents)
        .map_err(|error| format!("no se pudo guardar {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("no se pudo vaciar {}: {error}", path.display()))
}

fn downloads_directory() -> Result<PathBuf, String> {
    #[cfg(windows)]
    let home = nonempty_path_environment("USERPROFILE");
    #[cfg(not(windows))]
    let home = nonempty_path_environment("HOME");
    let home = home
        .ok_or_else(|| "no se pudo determinar la carpeta de usuario para Downloads".to_owned())?;
    #[cfg(not(windows))]
    if let Some(path) = std::env::var_os("XDG_DOWNLOAD_DIR").map(PathBuf::from) {
        if path.is_absolute() {
            return Ok(path);
        }
    }
    #[cfg(not(windows))]
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        if let Some(path) = read_xdg_download_dir(&config_home.join("user-dirs.dirs"), &home) {
            return Ok(path);
        }
    } else if let Some(path) =
        read_xdg_download_dir(&home.join(".config").join("user-dirs.dirs"), &home)
    {
        return Ok(path);
    }
    Ok(home.join("Downloads"))
}

#[cfg(not(windows))]
fn read_xdg_download_dir(config_file: &Path, home: &Path) -> Option<PathBuf> {
    let contents = fs::read_to_string(config_file).ok()?;
    parse_xdg_download_dir(&contents, home)
}

#[cfg(not(windows))]
fn parse_xdg_download_dir(contents: &str, home: &Path) -> Option<PathBuf> {
    let value = contents.lines().find_map(|line| {
        let value = line.trim().strip_prefix("XDG_DOWNLOAD_DIR=")?.trim();
        let value = value.strip_prefix('"')?.strip_suffix('"')?;
        Some(
            value
                .replace("${HOME}", &home.to_string_lossy())
                .replace("$HOME", &home.to_string_lossy()),
        )
    })?;
    let path = PathBuf::from(value);
    path.is_absolute().then_some(path)
}

fn unused_destination(directory: &Path, filename: &str) -> PathBuf {
    let original = directory.join(filename);
    if !original.exists() {
        return original;
    }
    let (stem, extension) = if let Some(stem) = filename.strip_suffix(".tar.gz") {
        (stem, ".tar.gz".to_owned())
    } else if let Some((stem, extension)) = filename.rsplit_once('.') {
        (stem, format!(".{extension}"))
    } else {
        (filename, String::new())
    };
    for index in 1..1000 {
        let candidate = directory.join(format!("{stem}-verified-{index}{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    directory.join(format!("{filename}.verified-{}", nonce()))
}

fn promote_without_overwrite(
    temporary: &Path,
    directory: &Path,
    filename: &str,
) -> Result<PathBuf, String> {
    let mut destination = unused_destination(directory, filename);
    for attempt in 0..1000 {
        match fs::hard_link(temporary, &destination) {
            Ok(()) => {
                if let Err(error) = fs::remove_file(temporary) {
                    eprintln!(
                        "Aviso: paquete verificado; no se pudo retirar el temporal {}: {error}",
                        temporary.display()
                    );
                }
                return Ok(destination);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                destination = directory.join(format!("{filename}.verified-{}-{attempt}", nonce()));
            }
            Err(link_error) => {
                match copy_without_overwrite(temporary, &destination) {
                    Ok(()) => {
                        if let Err(error) = fs::remove_file(temporary) {
                            eprintln!("Aviso: paquete verificado; no se pudo retirar el temporal {}: {error}", temporary.display());
                        }
                        return Ok(destination);
                    }
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                        destination = directory.join(format!("{filename}.verified-{}-{attempt}", nonce()));
                    }
                    Err(error) => return Err(format!("no se pudo promover el paquete verificado sin sobrescribir ({link_error}; {error})")),
                }
            }
        }
    }
    Err("no se encontró un nombre libre para el paquete verificado".into())
}

fn copy_without_overwrite(temporary: &Path, destination: &Path) -> io::Result<()> {
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let result = (|| {
        let mut input = File::open(temporary)?;
        io::copy(&mut input, &mut output)?;
        let permissions = fs::metadata(temporary)?.permissions();
        fs::set_permissions(destination, permissions)?;
        output.sync_all()
    })();
    if result.is_err() {
        let _ = fs::remove_file(destination);
    }
    result
}

fn nonce() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}

fn option(args: &[String], name: &str) -> Option<String> {
    args.iter().enumerate().find_map(|(index, value)| {
        value
            .strip_prefix(&format!("{name}="))
            .map(str::to_owned)
            .or_else(|| {
                (value == name)
                    .then(|| args.get(index + 1).cloned())
                    .flatten()
            })
    })
}

fn positionals(args: &[String]) -> Vec<&str> {
    let mut output = Vec::new();
    let mut skip = false;
    for value in args {
        if skip {
            skip = false;
            continue;
        }
        if value == "--repository" {
            skip = true;
            continue;
        }
        if value.starts_with('-') {
            continue;
        }
        output.push(value.as_str());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn compares_release_and_prerelease_versions() {
        assert!(version_is_newer("1.2.0", "1.1.99").unwrap());
        assert!(version_is_newer("1.2.0", "1.2.0-rc.1").unwrap());
        assert!(version_is_newer("1.2.0-beta.2", "1.2.0-beta.1").unwrap());
        assert!(!version_is_newer("1.2.0", "1.2.0").unwrap());
        assert!(version_is_newer("1.2.0-rc.10", "1.2.0-rc.2").unwrap());
        assert!(!version_is_newer("1.2.0+build.2", "1.2.0+build.1").unwrap());
    }

    #[test]
    fn rejects_unsafe_repository_and_versions() {
        assert!(validate_repository("owner/repo").is_ok());
        assert!(validate_repository("owner/repo/extra").is_err());
        assert!(validate_repository("owner;rm/repo").is_err());
        assert!(validate_version("1.2.3").is_ok());
        assert!(valid_tag("v1.2.3-rc.1"));
        assert!(!valid_tag("../v1.2.3"));
        assert!(validate_version("../other").is_err());
    }

    #[test]
    fn release_architecture_names_match_builder_packages() {
        assert_eq!(artifact_architecture("linux", "x86_64"), "x86_64");
        assert_eq!(artifact_architecture("linux", "aarch64"), "aarch64");
        assert_eq!(artifact_architecture("linux", "x86"), "i686");
        assert_eq!(artifact_architecture("linux", "arm"), "armv7l");
        assert_eq!(artifact_architecture("linux", "armv7l"), "armv7l");
        assert_eq!(artifact_architecture("windows", "x86_64"), "x86_64");
        assert_eq!(artifact_architecture("windows", "aarch64"), "arm64");
        assert_eq!(artifact_architecture("windows", "x86"), "x86");
        assert_eq!(artifact_architecture("macos", "aarch64"), "unsupported");
        assert_eq!(artifact_architecture("windows", "arm"), "unsupported");
    }

    #[test]
    fn release_page_must_match_repository_and_tag_exactly() {
        let expected = "https://github.com/owner/project/releases/tag/v1.2.3";
        assert!(validate_release_page("owner/project", "v1.2.3", expected).is_ok());
        assert!(validate_release_page(
            "owner/project",
            "v1.2.3",
            "https://github.com/owner/project/releases/tag/v1.2.3-attacker-v1.2.3"
        )
        .is_err());
        assert!(validate_release_page(
            "owner/project",
            "v1.2.3",
            "https://github.com/attacker/project/releases/tag/v1.2.3"
        )
        .is_err());
    }

    #[test]
    fn checksum_parser_requires_exact_filename_and_valid_hex() {
        let checksums = format!(
            "{}  ltools-release.json\n{}  app.tar.gz\n",
            "a".repeat(64),
            "b".repeat(64)
        );
        assert_eq!(
            checksum_for(checksums.as_bytes(), "app.tar.gz").unwrap(),
            "b".repeat(64)
        );
        assert!(checksum_for(checksums.as_bytes(), "other.tar.gz").is_err());
        assert!(checksum_for(b"bad  app.tar.gz\n", "app.tar.gz").is_err());
        let duplicates = format!(
            "{}  app.tar.gz\n{}  app.tar.gz\n",
            "a".repeat(64),
            "b".repeat(64)
        );
        assert!(checksum_for(duplicates.as_bytes(), "app.tar.gz")
            .unwrap_err()
            .contains("más de una"));
    }

    #[test]
    fn selects_only_exact_platform_architecture_and_repository_asset() {
        let platform = if cfg!(windows) { "windows" } else { "linux" };
        let (real_platform, arch, kind) = platform_artifact();
        let version = "1.2.3";
        let filename = match (real_platform, kind) {
            ("windows", "exe-cli") => format!("ltools-{version}-windows-{arch}-cli.exe"),
            ("windows", _) => format!("ltools-{version}-windows-{arch}.exe"),
            ("linux", "appimage-cli") => format!("ltools-{version}-linux-{arch}-cli.AppImage"),
            ("linux", "appimage") => format!("ltools-{version}-linux-{arch}.AppImage"),
            _ => format!("ltools-{version}-linux-{arch}.tar.gz"),
        };
        let tag = format!("v{version}-stable");
        let url = format!("https://github.com/example/project/releases/download/{tag}/{filename}");
        let manifest = serde_json::json!({
            "artifacts": [{"platform": platform, "architecture": arch, "kind": kind, "filename": filename,
                "download_url": url, "size_bytes": 42, "sha256": "a".repeat(64)}]
        });
        let artifact = select_artifact(&manifest, "example/project", version, &tag).unwrap();
        assert_eq!(artifact.filename, filename);
        assert!(select_artifact(&manifest, "evil/project", version, &tag).is_err());
        let mut malformed = manifest.clone();
        let unexpected_filename = format!("ltools-{version}-linux-{arch}-unexpected.tar.gz");
        malformed["artifacts"][0]["filename"] = Value::String(unexpected_filename.clone());
        malformed["artifacts"][0]["download_url"] = Value::String(format!(
            "https://github.com/example/project/releases/download/{tag}/{unexpected_filename}"
        ));
        assert!(select_artifact(&malformed, "example/project", version, &tag).is_err());
    }

    #[test]
    fn metadata_endpoint_override_is_loopback_only() {
        assert!(validate_test_base_url("http://127.0.0.1:45871/update").is_ok());
        assert!(validate_test_base_url("http://localhost:45871/update").is_ok());
        assert!(validate_test_base_url("http://evil.example/update").is_err());
    }

    #[cfg(not(windows))]
    #[test]
    fn parses_xdg_download_directory_without_evaluating_shell_syntax() {
        let home = Path::new("/home/test-user");
        assert_eq!(
            parse_xdg_download_dir(
                "XDG_DOCUMENTS_DIR=\"$HOME/Documents\"\nXDG_DOWNLOAD_DIR=\"$HOME/Descargas\"\n",
                home,
            ),
            Some(PathBuf::from("/home/test-user/Descargas"))
        );
        assert_eq!(
            parse_xdg_download_dir(
                "XDG_DOWNLOAD_DIR=\"/tmp/$(touch /tmp/should-not-run)\"\n",
                home,
            ),
            Some(PathBuf::from("/tmp/$(touch /tmp/should-not-run)")),
        );
        assert_eq!(
            parse_xdg_download_dir("XDG_DOWNLOAD_DIR=relative\n", home),
            None
        );
    }

    #[test]
    fn local_release_metadata_signature_and_artifact_contract_are_checked() {
        let (platform, architecture, kind) = platform_artifact();
        assert_ne!(
            architecture, "unsupported",
            "this updater target has no release artifact"
        );
        let version = crate::VERSION;
        let tag = format!("v{version}");
        let filename = match (platform, kind) {
            ("windows", "exe-cli") => format!("ltools-{version}-windows-{architecture}-cli.exe"),
            ("windows", _) => format!("ltools-{version}-windows-{architecture}.exe"),
            ("linux", "appimage-cli") => {
                format!("ltools-{version}-linux-{architecture}-cli.AppImage")
            }
            ("linux", "appimage") => format!("ltools-{version}-linux-{architecture}.AppImage"),
            _ => format!("ltools-{version}-linux-{architecture}.tar.gz"),
        };
        let release_url =
            format!("https://github.com/example/project/releases/download/{tag}/{filename}");
        let artifact_hash = "b".repeat(64);
        let manifest = serde_json::json!({
            "schema":"ltools-release-v1", "application":crate::i18n::product_name(),
            "version":version, "tag":tag, "repository":"example/project",
            "release_page":format!("https://github.com/example/project/releases/tag/{tag}"),
            "manifest_name":"ltools-release.json", "hash_algorithm":"sha256",
            "artifacts":[{"platform":platform,"architecture":architecture,"kind":kind,
                "filename":filename,"download_url":release_url,"size_bytes":42,"sha256":artifact_hash}]
        });
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        let checksums = format!(
            "{}  ltools-release.json\n{}  {filename}\n",
            hex(&Sha256::digest(&manifest_bytes)),
            artifact_hash
        );
        let signing_key = SigningKey::from_bytes(&[42; 32]);
        let signature = format!(
            "{}\n",
            STANDARD.encode(signing_key.sign(checksums.as_bytes()).to_bytes())
        );
        let verifying_key = hex(signing_key.verifying_key().as_bytes());
        let mut documents = BTreeMap::new();
        documents.insert("/ltools-release.json".to_owned(), manifest_bytes);
        documents.insert("/SHA256SUMS.txt".to_owned(), checksums.into_bytes());
        documents.insert("/SHA256SUMS.txt.sig".to_owned(), signature.into_bytes());
        let (base, server) = local_http_server(documents.clone());
        let key_path = std::env::temp_dir().join(format!(
            "ltools-updater-test-key-{}-{}",
            std::process::id(),
            nonce()
        ));
        fs::write(&key_path, verifying_key).unwrap();
        let result = fetch_latest_at("example/project", &base, Some(&key_path));
        let _ = fs::remove_file(&key_path);
        server.join().unwrap();
        let release = result.expect("el manifiesto y su firma de prueba deben verificarse");
        assert!(release.verified);
        assert_eq!(release.artifact.filename, filename);
        assert_eq!(release.version, version);

        let wrong_key = SigningKey::from_bytes(&[7; 32]);
        let wrong_key_path = std::env::temp_dir().join(format!(
            "ltools-updater-wrong-key-{}-{}",
            std::process::id(),
            nonce()
        ));
        fs::write(&wrong_key_path, hex(wrong_key.verifying_key().as_bytes())).unwrap();
        let (bad_base, bad_server) = local_http_server(documents);
        let bad_result = fetch_latest_at("example/project", &bad_base, Some(&wrong_key_path));
        let _ = fs::remove_file(&wrong_key_path);
        bad_server.join().unwrap();
        assert!(
            bad_result.is_err(),
            "un manifiesto firmado por otra clave debe rechazarse"
        );
    }

    #[test]
    fn local_artifact_download_checks_size_and_writes_only_to_reserved_path() {
        let contents = b"fixture package bytes".to_vec();
        let mut documents = BTreeMap::new();
        documents.insert("/asset.bin".to_owned(), contents.clone());
        let (base, server) = local_http_server(documents);
        let artifact = ReleaseArtifact {
            filename: "asset.bin".into(),
            download_url: format!("{base}/asset.bin"),
            size_bytes: contents.len() as u64,
            sha256: hex(&Sha256::digest(&contents)),
        };
        let workspace = create_workspace().unwrap();
        let destination = workspace.0.join("download.part");
        let result = download_artifact(&artifact, &destination);
        server.join().unwrap();
        result.unwrap();
        assert_eq!(fs::read(&destination).unwrap(), contents);
        assert!(
            download_artifact(&artifact, &destination).is_err(),
            "no debe sobrescribir el archivo reservado"
        );
    }

    #[test]
    fn verified_download_hashes_before_promoting_and_preserves_existing_files() {
        let contents = b"a verified portable update".to_vec();
        let hash = hex(&Sha256::digest(&contents));
        let mut documents = BTreeMap::new();
        documents.insert("/asset.bin".to_owned(), contents.clone());
        documents.insert("/bad-asset.bin".to_owned(), contents.clone());
        let (base, server) = local_http_server(documents);
        let workspace = create_workspace().unwrap();
        let downloads = workspace.0.join("Downloads");
        fs::create_dir(&downloads).unwrap();
        let existing = downloads.join("asset.bin");
        fs::write(&existing, b"keep the user's existing file").unwrap();
        let artifact = ReleaseArtifact {
            filename: "asset.bin".into(),
            download_url: format!("{base}/asset.bin"),
            size_bytes: contents.len() as u64,
            sha256: hash.clone(),
        };
        let promoted = download_verified_artifact(&artifact, &hash, &downloads).unwrap();
        assert_ne!(promoted, existing);
        assert_eq!(
            fs::read(&existing).unwrap(),
            b"keep the user's existing file"
        );
        assert_eq!(fs::read(&promoted).unwrap(), contents);
        assert_eq!(sha256_file(&promoted).unwrap(), hash);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&promoted).unwrap().permissions().mode() & 0o777,
                0o600,
                "una actualización no ejecutable queda privada para el usuario"
            );
        }

        let wrong_hash = "f".repeat(64);
        let invalid_artifact = ReleaseArtifact {
            filename: "bad-asset.bin".into(),
            download_url: format!("{base}/bad-asset.bin"),
            size_bytes: contents.len() as u64,
            sha256: wrong_hash.clone(),
        };
        assert!(download_verified_artifact(&invalid_artifact, &wrong_hash, &downloads).is_err());
        assert!(!downloads.join("bad-asset.bin").exists());
        assert_eq!(
            fs::read_dir(&downloads).unwrap().count(),
            2,
            "un temporal fallido no debe quedar publicado"
        );
        server.join().unwrap();
    }

    #[test]
    fn promotion_never_overwrites_an_existing_release_file() {
        let workspace = create_workspace().unwrap();
        let directory = workspace.0.join("downloads");
        fs::create_dir(&directory).unwrap();
        let original = directory.join("ltools-1.2.3-linux-x86_64.tar.gz");
        fs::write(&original, b"existing user file").unwrap();
        let temporary = directory.join(".verified.part");
        fs::write(&temporary, b"verified release").unwrap();
        let destination =
            promote_without_overwrite(&temporary, &directory, "ltools-1.2.3-linux-x86_64.tar.gz")
                .unwrap();
        assert_ne!(destination, original);
        assert_eq!(fs::read(&original).unwrap(), b"existing user file");
        assert_eq!(fs::read(&destination).unwrap(), b"verified release");
        assert!(!temporary.exists());
    }

    #[cfg(unix)]
    #[test]
    fn verified_appimage_is_executable_when_published() {
        use std::os::unix::fs::PermissionsExt;

        let contents = b"verified appimage fixture".to_vec();
        let hash = hex(&Sha256::digest(&contents));
        let mut documents = BTreeMap::new();
        documents.insert("/asset.AppImage".to_owned(), contents.clone());
        let (base, server) = local_http_server(documents);
        let workspace = create_workspace().unwrap();
        let artifact = ReleaseArtifact {
            filename: "ltools-1.2.3-linux-x86_64.AppImage".into(),
            download_url: format!("{base}/asset.AppImage"),
            size_bytes: contents.len() as u64,
            sha256: hash.clone(),
        };
        let destination = download_verified_artifact(&artifact, &hash, &workspace.0).unwrap();
        server.join().unwrap();
        assert_eq!(fs::read(&destination).unwrap(), contents);
        assert_eq!(
            fs::metadata(destination).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }

    #[cfg(unix)]
    #[test]
    fn fallback_copy_keeps_appimage_executable_mode() {
        use std::os::unix::fs::PermissionsExt;

        let workspace = create_workspace().unwrap();
        let temporary = workspace.0.join("verified.AppImage.part");
        let destination = workspace.0.join("verified.AppImage");
        fs::write(&temporary, b"verified appimage").unwrap();
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o755)).unwrap();
        copy_without_overwrite(&temporary, &destination).unwrap();
        assert_eq!(
            fs::metadata(destination).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }

    fn local_http_server(documents: BTreeMap<String, Vec<u8>>) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let expected_requests = documents.len();
        let server = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            let mut served = 0;
            while served < expected_requests && std::time::Instant::now() < deadline {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => panic!("falló el servidor HTTP de prueba: {error}"),
                };
                let mut request = [0_u8; 4096];
                let count = stream.read(&mut request).unwrap();
                let first_line = String::from_utf8_lossy(&request[..count]);
                let path = first_line.split_whitespace().nth(1).unwrap_or("/");
                if let Some(body) = documents.get(path) {
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .unwrap();
                    stream.write_all(body).unwrap();
                } else {
                    write!(
                        stream,
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    )
                    .unwrap();
                }
                served += 1;
            }
            assert_eq!(
                served, expected_requests,
                "el cliente no solicitó todos los documentos antes del timeout"
            );
        });
        (format!("http://{address}"), server)
    }
}
