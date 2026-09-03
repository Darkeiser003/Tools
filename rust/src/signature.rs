//! Firma Ed25519 compatible con el contrato de releases de LTerminal.
//!
//! La firma es deliberadamente un fichero separado: contiene Base64 de la
//! firma Ed25519 del contenido exacto de `SHA256SUMS.txt`, seguido de salto de
//! línea. No se imprimen ni se registran claves privadas.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use ed25519_dalek::pkcs8::DecodePrivateKey;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run(args: &[String]) -> Result<(), String> {
    let manifest = required_path(args, "--manifest")?;
    let signature = option(args, "--signature")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("{}.sig", manifest.display())));
    let public_path = option(args, "--public-key-file")
        .map(PathBuf::from)
        .or_else(|| env_path("LTOOLS_UPDATE_PUBLIC_KEY_FILE"))
        .or_else(|| env_path("LTERMINAL_UPDATE_PUBLIC_KEY_FILE"));
    let private_path = option(args, "--private-key-file")
        .map(PathBuf::from)
        .or_else(|| env_path("LTOOLS_SIGNING_PRIVATE_KEY_FILE"))
        .or_else(|| env_path("LTERMINAL_SIGNING_PRIVATE_KEY_FILE"));
    let verify_only = args.iter().any(|arg| arg == "--verify");
    let contents =
        fs::read(&manifest).map_err(|e| format!("no se pudo leer {}: {e}", manifest.display()))?;
    if contents.is_empty() {
        return Err("no se puede firmar o verificar un manifiesto vacío".into());
    }

    let public_key = load_public_key(public_path.as_deref())?;
    if verify_only {
        verify(&contents, &signature, &public_key)?;
        println!("Firma verificada: {}", signature.display());
        return Ok(());
    }

    let signing_key = load_private_key(private_path.as_deref())?;
    if signing_key.verifying_key() != public_key {
        return Err("la clave pública no corresponde con la clave privada".into());
    }
    let signed = signing_key.sign(&contents);
    let encoded = format!("{}\n", STANDARD.encode(signed.to_bytes()));
    write_atomic(&signature, encoded.as_bytes())?;
    verify(&contents, &signature, &public_key)?;
    println!("Manifiesto firmado y verificado: {}", signature.display());
    Ok(())
}

fn verify(contents: &[u8], signature_path: &Path, public_key: &VerifyingKey) -> Result<(), String> {
    let encoded = fs::read_to_string(signature_path)
        .map_err(|e| format!("no se pudo leer {}: {e}", signature_path.display()))?;
    let compact: String = encoded
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let bytes = STANDARD
        .decode(compact.as_bytes())
        .map_err(|e| format!("firma Base64 inválida en {}: {e}", signature_path.display()))?;
    let signature_bytes: [u8; 64] = bytes.try_into().map_err(|_| {
        format!(
            "la firma de {} no contiene 64 bytes",
            signature_path.display()
        )
    })?;
    let signature = Signature::from_bytes(&signature_bytes);
    public_key
        .verify(contents, &signature)
        .map_err(|_| "la firma Ed25519 no es válida para este manifiesto".to_string())
}

fn load_private_key(path: Option<&Path>) -> Result<SigningKey, String> {
    if let Some(path) = path {
        let pem = fs::read_to_string(path)
            .map_err(|e| format!("no se pudo leer la clave privada {}: {e}", path.display()))?;
        return SigningKey::from_pkcs8_pem(&pem).map_err(|e| {
            format!(
                "la clave privada {} no es un Ed25519 PKCS#8 válido: {e}",
                path.display()
            )
        });
    }
    if let Some(pem) = std::env::var("LTOOLS_SIGNING_PRIVATE_KEY")
        .ok()
        .or_else(|| std::env::var("LTERMINAL_SIGNING_PRIVATE_KEY").ok())
    {
        return SigningKey::from_pkcs8_pem(&pem)
            .map_err(|e| format!("la clave privada de entorno no es Ed25519 PKCS#8 válida: {e}"));
    }
    let default = default_key_path("release-signing-private.pem")?;
    let pem = fs::read_to_string(&default)
        .map_err(|e| format!("falta la clave privada Ed25519 {}: {e}", default.display()))?;
    SigningKey::from_pkcs8_pem(&pem).map_err(|e| {
        format!(
            "la clave privada {} no es un Ed25519 PKCS#8 válido: {e}",
            default.display()
        )
    })
}

fn load_public_key(path: Option<&Path>) -> Result<VerifyingKey, String> {
    let (text, source) = if let Some(path) = path {
        (
            fs::read_to_string(path)
                .map_err(|e| format!("no se pudo leer la clave pública {}: {e}", path.display()))?,
            path.display().to_string(),
        )
    } else if let Some(value) = std::env::var("LTOOLS_UPDATE_PUBLIC_KEY")
        .ok()
        .or_else(|| std::env::var("LTERMINAL_UPDATE_PUBLIC_KEY").ok())
    {
        (value, "entorno".to_string())
    } else {
        let default = default_key_path("release-signing-public.hex")?;
        (
            fs::read_to_string(&default).map_err(|e| {
                format!("falta la clave pública Ed25519 {}: {e}", default.display())
            })?,
            default.display().to_string(),
        )
    };
    let compact: String = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let bytes = decode_hex(&compact)
        .map_err(|error| format!("clave pública {source} inválida: {error}"))?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        format!("clave pública {source} debe contener exactamente 32 bytes (64 caracteres hex)")
    })?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|e| format!("clave pública {source} no es una clave Ed25519 válida: {e}"))
}

fn default_key_path(filename: &str) -> Result<PathBuf, String> {
    let config = std::env::var_os("LTOOLS_CONFIG_HOME")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or_else(|| {
            "no se pudo determinar la carpeta de configuración; usa --*-key-file".to_string()
        })?;
    Ok(config.join("lterminal").join(filename))
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from)
}

fn required_path(args: &[String], name: &str) -> Result<PathBuf, String> {
    option(args, name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("release-signature requiere {name} FICHERO"))
}

fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].clone())
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err("la longitud hexadecimal debe ser par".into());
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    let mut chars = value.chars();
    while let (Some(high), Some(low)) = (chars.next(), chars.next()) {
        let high = high
            .to_digit(16)
            .ok_or_else(|| "contiene caracteres no hexadecimales".to_string())?;
        let low = low
            .to_digit(16)
            .ok_or_else(|| "contiene caracteres no hexadecimales".to_string())?;
        bytes.push(((high << 4) | low) as u8);
    }
    Ok(bytes)
}

fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|e| format!("no se pudo crear {}: {e}", parent.display()))?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("SHA256SUMS.txt.sig");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let temporary = parent.join(format!(".{name}.tmp-{}-{nonce}", std::process::id()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| format!("no se pudo crear {}: {e}", temporary.display()))?;
        file.write_all(contents)
            .map_err(|e| format!("no se pudo escribir {}: {e}", temporary.display()))?;
        file.sync_all()
            .map_err(|e| format!("no se pudo confirmar {}: {e}", temporary.display()))?;
        #[cfg(windows)]
        {
            let backup = parent.join(format!(".{name}.bak-{}-{nonce}", std::process::id()));
            let had_previous = path.exists();
            if had_previous {
                fs::rename(path, &backup)
                    .map_err(|e| format!("no se pudo reservar la firma anterior: {e}"))?;
            }
            if let Err(error) = fs::rename(&temporary, path) {
                if had_previous {
                    let _ = fs::rename(&backup, path);
                }
                return Err(format!("no se pudo publicar {}: {error}", path.display()));
            }
            if had_previous {
                let _ = fs::remove_file(backup);
            }
        }
        #[cfg(not(windows))]
        fs::rename(&temporary, path)
            .map_err(|e| format!("no se pudo publicar {}: {e}", path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::decode_hex;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn firma_rfc8032_con_ed25519() {
        let seed =
            decode_hex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60").unwrap();
        let seed: [u8; 32] = seed.try_into().unwrap();
        let key = SigningKey::from_bytes(&seed);
        let signature = key.sign(b"");
        assert_eq!(hex(&signature.to_bytes()), "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b");
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
