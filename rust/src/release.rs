//! Generación nativa del manifiesto verificable de una release.
//!
//! El builder solo compila y empaqueta; este módulo es quien inspecciona los
//! artefactos, calcula SHA-256 y genera el JSON que se puede subir a GitHub.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::{common, i18n, VERSION};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Artifact {
    platform: &'static str,
    architecture: String,
    kind: &'static str,
    filename: String,
    download_url: String,
    size_bytes: u64,
    sha256: String,
    executable: bool,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let output = option(args, "--output")
        .map(PathBuf::from)
        .ok_or_else(|| "release-manifest requiere --output FICHERO".to_string())?;
    let repository = option(args, "--repository")
        .or_else(|| std::env::var("LTOOLS_GITHUB_REPOSITORY").ok())
        .unwrap_or_else(|| "Darkeiser003/Tools".into());
    validate_repository(&repository)?;
    let tag = option(args, "--tag").unwrap_or_else(|| format!("v{VERSION}"));
    if !tag.starts_with('v') || tag.len() < 2 {
        return Err("--tag debe tener el formato vVERSION".into());
    }
    let dirs = repeated_options(args, "--artifacts-dir");
    if dirs.is_empty() {
        return Err("release-manifest requiere al menos un --artifacts-dir".into());
    }
    let artifacts = collect_artifacts(&dirs, &repository, &tag)?;
    if artifacts.is_empty() {
        return Err(
            "no se encontraron artefactos LTools reconocibles en las carpetas indicadas".into(),
        );
    }
    let json = render_manifest(&repository, &tag, &artifacts);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("no se pudo crear {}: {e}", parent.display()))?;
    }
    fs::write(&output, json)
        .map_err(|e| format!("no se pudo escribir {}: {e}", output.display()))?;
    println!("Manifiesto de release: {}", output.display());
    println!("Artefactos incluidos: {}", artifacts.len());
    for artifact in &artifacts {
        println!(
            "  {} {} {}",
            artifact.platform, artifact.kind, artifact.filename
        );
    }
    Ok(())
}

fn collect_artifacts(
    dirs: &[String],
    repository: &str,
    tag: &str,
) -> Result<Vec<Artifact>, String> {
    let mut artifacts = BTreeMap::new();
    for directory in dirs {
        let path = Path::new(directory);
        if !path.is_dir() {
            return Err(format!(
                "no existe la carpeta de artefactos: {}",
                path.display()
            ));
        }
        let entries =
            fs::read_dir(path).map_err(|e| format!("no se pudo leer {}: {e}", path.display()))?;
        for entry in entries {
            let entry = entry
                .map_err(|e| format!("no se pudo leer una entrada de {}: {e}", path.display()))?;
            let file = entry.path();
            if !file.is_file() {
                continue;
            }
            let Some((platform, architecture, kind, executable)) =
                classify(entry.file_name().to_string_lossy().as_ref())
            else {
                continue;
            };
            let filename = file_name(&file)?;
            let metadata = fs::metadata(&file)
                .map_err(|e| format!("no se pudo inspeccionar {}: {e}", file.display()))?;
            let sha256 = sha256_file(&file)?;
            let download_url = format!(
                "https://github.com/{repository}/releases/download/{tag}/{}",
                percent_safe_filename(&filename)
            );
            artifacts.insert(
                filename.clone(),
                Artifact {
                    platform,
                    architecture,
                    kind,
                    filename,
                    download_url,
                    size_bytes: metadata.len(),
                    sha256,
                    executable,
                },
            );
        }
    }
    Ok(artifacts.into_values().collect())
}

fn classify(filename: &str) -> Option<(&'static str, String, &'static str, bool)> {
    if !filename.starts_with("ltools-") || filename == "ltools-release.json" {
        return None;
    }
    let parts: Vec<&str> = filename.split('-').collect();
    if parts.len() < 4 {
        return None;
    }
    if parts[2] == "linux" {
        let remainder = filename.strip_prefix(&format!("ltools-{}-linux-", parts[1]))?;
        let remainder = remainder
            .strip_suffix(".AppImage")
            .or_else(|| remainder.strip_suffix(".tar.gz"))?;
        let architecture = remainder
            .strip_suffix("-cli")
            .unwrap_or(remainder)
            .to_string();
        let kind = if filename.ends_with("-cli.AppImage") {
            "appimage-cli"
        } else if filename.ends_with(".AppImage") {
            "appimage"
        } else if filename.ends_with(".tar.gz") {
            "tarball"
        } else {
            return None;
        };
        return Some(("linux", architecture, kind, kind != "tarball"));
    }
    if parts[2] == "windows" {
        let architecture = filename
            .strip_prefix(&format!("ltools-{}-windows-", parts[1]))?
            .strip_suffix(".exe")
            .or_else(|| {
                filename
                    .strip_prefix(&format!("ltools-{}-windows-", parts[1]))
                    .and_then(|value| value.strip_suffix(".zip"))
            })?;
        let architecture = architecture
            .strip_suffix("-cli")
            .unwrap_or(architecture)
            .to_string();
        let kind = if filename.ends_with("-cli.exe") {
            "exe-cli"
        } else if filename.ends_with(".exe") {
            "exe"
        } else if filename.ends_with(".zip") {
            "portable-zip"
        } else {
            return None;
        };
        return Some((
            "windows",
            architecture,
            kind,
            matches!(kind, "exe" | "exe-cli"),
        ));
    }
    None
}

fn render_manifest(repository: &str, tag: &str, artifacts: &[Artifact]) -> String {
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str("  \"schema\": \"ltools-release-v1\",\n");
    json.push_str(&format!(
        "  \"application\": \"{}\",\n",
        json_escape(i18n::product_name())
    ));
    json.push_str(&format!("  \"version\": \"{}\",\n", json_escape(VERSION)));
    json.push_str(&format!("  \"tag\": \"{}\",\n", json_escape(tag)));
    json.push_str(&format!(
        "  \"repository\": \"{}\",\n",
        json_escape(repository)
    ));
    json.push_str(&format!(
        "  \"release_page\": \"https://github.com/{}/releases/tag/{}\",\n",
        json_escape(repository),
        json_escape(tag)
    ));
    json.push_str("  \"manifest_name\": \"ltools-release.json\",\n");
    json.push_str(&format!(
        "  \"generated_at\": \"{}\",\n",
        json_escape(&common::timestamp())
    ));
    json.push_str("  \"hash_algorithm\": \"sha256\",\n");
    json.push_str("  \"artifacts\": [\n");
    for (index, artifact) in artifacts.iter().enumerate() {
        json.push_str("    {");
        json.push_str(&format!("\"platform\":\"{}\",", artifact.platform));
        json.push_str(&format!(
            "\"architecture\":\"{}\",",
            json_escape(&artifact.architecture)
        ));
        json.push_str(&format!("\"kind\":\"{}\",", artifact.kind));
        json.push_str(&format!(
            "\"filename\":\"{}\",",
            json_escape(&artifact.filename)
        ));
        json.push_str(&format!(
            "\"download_url\":\"{}\",",
            json_escape(&artifact.download_url)
        ));
        json.push_str(&format!("\"size_bytes\":{},", artifact.size_bytes));
        json.push_str(&format!("\"sha256\":\"{}\",", artifact.sha256));
        json.push_str(&format!("\"executable\":{}", artifact.executable));
        json.push('}');
        if index + 1 != artifacts.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ]\n}\n");
    json
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        File::open(path).map_err(|e| format!("no se pudo abrir {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("no se pudo leer {}: {e}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].clone())
}

fn repeated_options(args: &[String], name: &str) -> Vec<String> {
    args.windows(2)
        .filter(|window| window[0] == name)
        .map(|window| window[1].clone())
        .collect()
}

fn validate_repository(repository: &str) -> Result<(), String> {
    let mut parts = repository.split('/');
    if parts.next().is_none() || parts.next().is_none() || parts.next().is_some() {
        return Err("--repository debe tener el formato propietario/repositorio".into());
    }
    Ok(())
}

fn file_name(path: &Path) -> Result<String, String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| format!("ruta de artefacto inválida: {}", path.display()))
}

fn percent_safe_filename(filename: &str) -> String {
    filename
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F")
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
    use super::classify;

    #[test]
    fn clasifica_assets_linux_y_windows() {
        assert_eq!(
            classify(concat!(
                "ltools-",
                env!("CARGO_PKG_VERSION"),
                "-linux-x86_64.AppImage"
            )),
            Some(("linux", "x86_64".into(), "appimage", true))
        );
        assert_eq!(
            classify(concat!(
                "ltools-",
                env!("CARGO_PKG_VERSION"),
                "-linux-x86_64-cli.AppImage"
            )),
            Some(("linux", "x86_64".into(), "appimage-cli", true))
        );
        assert_eq!(
            classify(concat!(
                "ltools-",
                env!("CARGO_PKG_VERSION"),
                "-windows-x86_64.exe"
            )),
            Some(("windows", "x86_64".into(), "exe", true))
        );
        assert_eq!(
            classify(concat!(
                "ltools-",
                env!("CARGO_PKG_VERSION"),
                "-windows-x86_64-cli.exe"
            )),
            Some(("windows", "x86_64".into(), "exe-cli", true))
        );
        assert_eq!(
            classify(concat!(
                "ltools-",
                env!("CARGO_PKG_VERSION"),
                "-windows-x86_64.zip"
            )),
            Some(("windows", "x86_64".into(), "portable-zip", false))
        );
    }
}
