//! Contexto visual compartido por la CLI, la GUI y los hosts de terminal.
//!
//! LTerminal puede propagar el idioma y el tema mediante las variables del
//! contrato de integración. LTools también funciona de forma autónoma: si no
//! recibe contexto externo aplica `ocean`, una paleta oscura inspirada en la
//! terminal. Los nombres de las variables son deliberadamente estables para
//! que otros hosts puedan integrarse sin depender de código Rust.

use std::env;
use std::io::{self, IsTerminal};

pub const SUPPORTED: &[&str] = &[
    "ocean", "forest", "amber", "nordic", "matrix", "contrast", "slate", "plum", "teal", "crimson",
    "silver", "violet",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub background: &'static str,
    pub surface: &'static str,
    pub surface_alt: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    pub muted: &'static str,
    pub accent: &'static str,
    pub output_background: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub id: &'static str,
    pub palette: Palette,
    pub color_mode: ColorMode,
}

impl Theme {
    pub fn colors_enabled(self) -> bool {
        match self.color_mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal(),
        }
    }

    pub fn paint(self, role: Role, value: impl AsRef<str>) -> String {
        let value = value.as_ref();
        if !self.colors_enabled() {
            return value.to_owned();
        }
        let code = role.code(self.palette);
        format!("\x1b[38;2;{code}m{value}\x1b[0m")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Title,
    Section,
    Info,
    Muted,
    Success,
    Warning,
    Error,
}

impl Role {
    fn code(self, palette: Palette) -> String {
        let hex = match self {
            Self::Title | Self::Section => palette.accent,
            Self::Info => palette.text,
            Self::Muted => palette.muted,
            Self::Success => palette.success,
            Self::Warning => palette.warning,
            Self::Error => palette.error,
        };
        rgb_code(hex)
    }
}

pub fn normalize(value: &str) -> &'static str {
    let value = value.trim().to_ascii_lowercase().replace('_', "-");
    let value = match value.as_str() {
        "greenphosphor" | "green-phosphor" | "matrix" => "matrix",
        "highcontrast" | "high-contrast" | "contrast" => "contrast",
        "techcyan" | "tech-cyan" | "turquoise" | "teal" => "teal",
        "blue" | "ocean-dark" => "ocean",
        "purple" => "violet",
        other => other,
    };
    SUPPORTED
        .iter()
        .copied()
        .find(|candidate| *candidate == value)
        .unwrap_or("ocean")
}

pub fn color_mode(value: &str) -> ColorMode {
    match value.trim().to_ascii_lowercase().as_str() {
        "always" | "on" | "yes" | "true" | "1" => ColorMode::Always,
        "never" | "off" | "no" | "false" | "0" => ColorMode::Never,
        _ => ColorMode::Auto,
    }
}

pub fn set(value: &str) {
    env::set_var("LTOOLS_THEME", normalize(value));
}

pub fn set_color_mode(value: &str) {
    env::set_var(
        "LTOOLS_COLOR",
        match color_mode(value) {
            ColorMode::Always => "always",
            ColorMode::Never => "never",
            ColorMode::Auto => "auto",
        },
    );
}

/// Tema para texto CLI. Los nombres propios de LTools tienen precedencia;
/// después se consultan las variables que puede exportar LTerminal o WinSlim
/// Terminal; finalmente se usa la paleta `ocean`.
pub fn current() -> Theme {
    let id = first_env(&[
        "LTOOLS_THEME",
        "LTERMINAL_THEME",
        "WINSLIM_TERMINAL_THEME",
        "TERMINAL_THEME",
    ])
    .map(|value| normalize(&value))
    .unwrap_or("ocean");
    let mode = first_env(&["LTOOLS_COLOR", "LTERMINAL_COLOR", "TERMINAL_COLOR"])
        .map(|value| color_mode(&value))
        .unwrap_or(ColorMode::Auto);
    Theme {
        id,
        palette: palette(id),
        color_mode: mode,
    }
}

/// Tema de la GUI independiente. No hereda por accidente el tema de una
/// terminal anfitriona; solo `LTOOLS_GUI_THEME` lo puede cambiar de forma
/// explícita. Así una AppImage abierta desde un lanzador conserva su aspecto.
pub fn gui() -> Theme {
    let id = env::var("LTOOLS_GUI_THEME")
        .ok()
        .map(|value| normalize(&value))
        .unwrap_or("ocean");
    Theme {
        id,
        palette: palette(id),
        color_mode: ColorMode::Never,
    }
}

pub fn palette(id: &str) -> Palette {
    match normalize(id) {
        "forest" => Palette {
            background: "#101b16",
            surface: "#1b3228",
            surface_alt: "#254836",
            border: "#4b8067",
            text: "#e2f5e9",
            muted: "#a5c5b2",
            accent: "#78e0ad",
            output_background: "#09110d",
            success: "#8be9b2",
            warning: "#f0c674",
            error: "#ff9389",
        },
        "amber" => Palette {
            background: "#1c1710",
            surface: "#382919",
            surface_alt: "#4b3520",
            border: "#9a6d3e",
            text: "#fff1d1",
            muted: "#d2b98c",
            accent: "#ffc857",
            output_background: "#110d08",
            success: "#a8e6a3",
            warning: "#ffd166",
            error: "#ff9285",
        },
        "nordic" => Palette {
            background: "#101820",
            surface: "#202c3b",
            surface_alt: "#2b3c50",
            border: "#607d9a",
            text: "#e5edf5",
            muted: "#a8b9c9",
            accent: "#88c0d0",
            output_background: "#0a1016",
            success: "#a3be8c",
            warning: "#ebcb8b",
            error: "#bf616a",
        },
        "matrix" => Palette {
            background: "#07110b",
            surface: "#0d2415",
            surface_alt: "#12391f",
            border: "#2c8f4b",
            text: "#d7ffdf",
            muted: "#8cc99a",
            accent: "#66e27f",
            output_background: "#030804",
            success: "#8aff9b",
            warning: "#d7e86c",
            error: "#ff7b72",
        },
        "contrast" => Palette {
            background: "#000000",
            surface: "#151515",
            surface_alt: "#252525",
            border: "#ffffff",
            text: "#ffffff",
            muted: "#d0d0d0",
            accent: "#00e5ff",
            output_background: "#000000",
            success: "#00ff66",
            warning: "#ffff00",
            error: "#ff5050",
        },
        "slate" => Palette {
            background: "#11151b",
            surface: "#242c38",
            surface_alt: "#303b4b",
            border: "#687a91",
            text: "#edf2f7",
            muted: "#aeb9c7",
            accent: "#9cc4e4",
            output_background: "#090c11",
            success: "#9bd3ae",
            warning: "#efd28a",
            error: "#ff8d83",
        },
        "plum" => Palette {
            background: "#19111b",
            surface: "#34213a",
            surface_alt: "#472c50",
            border: "#8b5c94",
            text: "#f7eafa",
            muted: "#c7accb",
            accent: "#d49bea",
            output_background: "#0e090f",
            success: "#a9e6b0",
            warning: "#f2c777",
            error: "#ff8e9b",
        },
        "teal" => Palette {
            background: "#0d1919",
            surface: "#173535",
            surface_alt: "#215252",
            border: "#4a9994",
            text: "#e0f7f4",
            muted: "#9fc8c4",
            accent: "#6ee7d8",
            output_background: "#071010",
            success: "#92e6b0",
            warning: "#eed27e",
            error: "#ff8d82",
        },
        "crimson" => Palette {
            background: "#1b1014",
            surface: "#392027",
            surface_alt: "#502b36",
            border: "#a35d6d",
            text: "#fae9ed",
            muted: "#d2abb4",
            accent: "#ff8fa3",
            output_background: "#10090c",
            success: "#a8e6ad",
            warning: "#f2ca7b",
            error: "#ff6575",
        },
        "silver" => Palette {
            background: "#151719",
            surface: "#2b3035",
            surface_alt: "#3a4148",
            border: "#89949e",
            text: "#f2f4f5",
            muted: "#bac2c9",
            accent: "#c7e2f2",
            output_background: "#0a0c0e",
            success: "#a8e0b4",
            warning: "#ecd285",
            error: "#ff8b82",
        },
        "violet" => Palette {
            background: "#14101d",
            surface: "#29203e",
            surface_alt: "#3a2d57",
            border: "#8066af",
            text: "#f0eaff",
            muted: "#bbaed2",
            accent: "#bda4ff",
            output_background: "#0b0810",
            success: "#a9e6b2",
            warning: "#f1ca7b",
            error: "#ff8994",
        },
        _ => Palette {
            background: "#10161b",
            surface: "#1c2b34",
            surface_alt: "#28586d",
            border: "#456878",
            text: "#e6edf3",
            muted: "#9fb3be",
            accent: "#65c7e8",
            output_background: "#090d10",
            success: "#7ee787",
            warning: "#e6b450",
            error: "#ff7b72",
        },
    }
}

fn first_env(names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| env::var(name).ok().filter(|value| !value.trim().is_empty()))
}

fn rgb_code(hex: &str) -> String {
    let value = hex.trim_start_matches('#');
    if value.len() != 6 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return "230;237;243".into();
    }
    let red = u8::from_str_radix(&value[0..2], 16).unwrap_or(230);
    let green = u8::from_str_radix(&value[2..4], 16).unwrap_or(237);
    let blue = u8::from_str_radix(&value[4..6], 16).unwrap_or(243);
    format!("{red};{green};{blue}")
}

#[cfg(test)]
mod tests {
    use super::{normalize, palette, ColorMode, Role, Theme, SUPPORTED};

    #[test]
    fn normalizes_terminal_theme_aliases() {
        assert_eq!(normalize("greenPhosphor"), "matrix");
        assert_eq!(normalize("high-contrast"), "contrast");
        assert_eq!(normalize("techCyan"), "teal");
        assert_eq!(normalize("unknown"), "ocean");
    }

    #[test]
    fn every_theme_has_a_complete_dark_palette() {
        for id in SUPPORTED {
            let colors = palette(id);
            assert!(colors.background.starts_with('#'));
            assert!(colors.output_background.starts_with('#'));
            assert!(!colors.accent.is_empty());
        }
    }

    #[test]
    fn forced_color_mode_is_only_for_human_output() {
        let theme = Theme {
            id: "ocean",
            palette: palette("ocean"),
            color_mode: ColorMode::Always,
        };
        assert!(theme.paint(Role::Title, "LTools").contains("\x1b["));
    }
}
