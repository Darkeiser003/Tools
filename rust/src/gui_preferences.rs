//! Preferencias ligeras de la GUI independiente.
//!
//! La GUI no comparte el fichero de preferencias de LTerminal: solo recibe
//! su contexto por variables de entorno. Una elección manual de LTools se
//! guarda aquí y pasa a tener prioridad sobre ese contexto en los siguientes
//! arranques. El formato es deliberadamente plano para que se pueda recuperar
//! incluso si una versión futura cambia su implementación interna.

use crate::{common, i18n, theme};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;

const HEADER: &str = "# ltools-gui-preferences-v1";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Preferences {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub hidden_categories: BTreeSet<String>,
    pub elevate_by_default: bool,
}

pub fn path() -> PathBuf {
    let base = if let Some(value) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(value)
    } else if cfg!(windows) {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| common::home_dir().join("AppData/Roaming"))
    } else {
        common::home_dir().join(".config")
    };
    base.join("ltools").join("gui-preferences.conf")
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 40
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn parse_set(value: &str) -> BTreeSet<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| valid_id(value))
        .map(str::to_owned)
        .collect()
}

pub fn load() -> Preferences {
    let Ok(contents) = fs::read_to_string(path()) else {
        return Preferences::default();
    };
    let mut preferences = Preferences::default();
    for line in contents.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "theme" if valid_id(value) && theme::SUPPORTED.contains(&value) => {
                preferences.theme = Some(value.to_owned());
            }
            "language" if value == "auto" => preferences.language = None,
            "language" if valid_id(value) && i18n::SUPPORTED.contains(&value) => {
                preferences.language = Some(value.to_owned());
            }
            "hidden_categories" => preferences.hidden_categories = parse_set(value),
            "elevate_by_default" => {
                preferences.elevate_by_default = matches!(value, "1" | "true" | "yes" | "si")
            }
            _ => {}
        }
    }
    preferences
}

fn encode_set(values: &BTreeSet<String>) -> String {
    values.iter().cloned().collect::<Vec<_>>().join(",")
}

pub fn save(preferences: &Preferences) -> Result<(), String> {
    let target = path();
    let parent = target
        .parent()
        .ok_or_else(|| "la ruta de preferencias no tiene directorio padre".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let contents = format!(
        "{HEADER}\ntheme={}\nlanguage={}\nhidden_categories={}\nelevate_by_default={}\n",
        preferences.theme.as_deref().unwrap_or("auto"),
        preferences.language.as_deref().unwrap_or("auto"),
        encode_set(&preferences.hidden_categories),
        if preferences.elevate_by_default {
            "true"
        } else {
            "false"
        },
    );
    let temporary = target.with_extension(format!("conf.tmp.{}", std::process::id()));
    fs::write(&temporary, contents).map_err(|error| error.to_string())?;
    // No borres el fichero anterior antes del intercambio: si el rename
    // fallara en Windows, se perderían las preferencias que sí eran válidas.
    #[cfg(windows)]
    replace_file_windows(&temporary, &target).map_err(|error| error.to_string())?;
    #[cfg(not(windows))]
    return fs::rename(&temporary, &target).map_err(|error| error.to_string());
    #[cfg(windows)]
    return Ok(());
}

#[cfg(windows)]
fn replace_file_windows(
    temporary: &std::path::Path,
    target: &std::path::Path,
) -> std::io::Result<()> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let temporary = temporary
        .as_os_str()
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<_>>();
    let target = target
        .as_os_str()
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        MoveFileExW(
            temporary.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn set_elevate_by_default(enabled: bool) -> Result<(), String> {
    let mut preferences = load();
    preferences.elevate_by_default = enabled;
    save(&preferences)
}

pub fn apply_environment() {
    let preferences = load();
    if let Some(theme) = preferences.theme {
        env::set_var("LTOOLS_GUI_THEME", theme);
    }
    if let Some(language) = preferences.language {
        env::set_var("LTOOLS_LANG", language);
    }
}

pub fn set_theme(value: &str) -> Result<(), String> {
    let normalized = theme::normalize(value).to_owned();
    if !theme::SUPPORTED.contains(&normalized.as_str()) {
        return Err(format!("tema no soportado: {value}"));
    }
    let mut preferences = load();
    preferences.theme = Some(normalized);
    save(&preferences)
}

pub fn set_language(value: &str) -> Result<(), String> {
    let mut preferences = load();
    if value.eq_ignore_ascii_case("auto") {
        preferences.language = None;
    } else {
        let normalized = i18n::normalize(value).to_owned();
        if !i18n::SUPPORTED.contains(&normalized.as_str()) {
            return Err(format!("idioma no soportado: {value}"));
        }
        preferences.language = Some(normalized);
    }
    save(&preferences)
}

pub fn set_category_visible(category: &str, visible: bool) -> Result<(), String> {
    if !valid_id(category) {
        return Err("identificador de categoría no válido".into());
    }
    let mut preferences = load();
    if visible {
        preferences.hidden_categories.remove(category);
    } else {
        preferences.hidden_categories.insert(category.to_owned());
    }
    save(&preferences)
}

pub fn category_hidden(category: &str) -> bool {
    load().hidden_categories.contains(category)
}

#[cfg(test)]
mod tests {
    use super::{encode_set, parse_set};
    use std::collections::BTreeSet;

    #[test]
    fn sets_are_stable_and_safe() {
        let parsed = parse_set("storage, services, ../../unsafe, automation");
        assert_eq!(
            parsed,
            BTreeSet::from(["automation".into(), "services".into(), "storage".into()])
        );
        assert_eq!(encode_set(&parsed), "automation,services,storage");
    }
}
