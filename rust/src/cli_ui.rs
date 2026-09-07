//! Primitivas visuales compartidas por la CLI interactiva.
//!
//! La CLI no intenta imitar una GUI: adopta el mismo modelo mental que
//! LTerminal —cabecera estable, contexto visible, área de trabajo y una
//! leyenda de navegación— para que los comandos lanzados desde la terminal y
//! la aplicación independiente se comporten igual.

use crate::{i18n, theme, VERSION};

/// Pinta una cabecera compacta, equivalente a la barra superior de LTerminal.
/// `section` es la ruta actual; `None` representa el inicio.
pub fn header(section: Option<&str>) {
    let title = format!("{} Rust {VERSION}", i18n::text("menu.title"));
    println!("{}", theme::current().paint(theme::Role::Title, title));
    match section {
        Some(section) => println!(
            "  {}  ›  {}",
            theme::current().paint(theme::Role::Muted, i18n::product_name()),
            theme::current().paint(theme::Role::Section, i18n::category_text(section))
        ),
        None => println!(
            "  {}",
            theme::current().paint(theme::Role::Muted, i18n::product_name())
        ),
    }
    println!(
        "{}",
        theme::current().paint(
            theme::Role::Muted,
            "────────────────────────────────────────"
        )
    );
}

/// Divide visualmente grupos de operaciones sin introducir otra navegación.
pub fn group(label: &str) {
    println!("\n{}", theme::current().paint(theme::Role::Info, label));
}

/// Leyenda común para todos los submenús.
pub fn footer(submenu: bool) {
    let back = if submenu {
        format!("[Enter/b] {}", i18n::text("menu.back"))
    } else {
        format!("[Enter] {}", i18n::text("menu.quit"))
    };
    println!();
    println!(
        "{}",
        theme::current().paint(
            theme::Role::Muted,
            format!(
                "{}  ·  [h/?] {}  ·  [q] {}",
                back,
                i18n::text("menu.help"),
                i18n::text("menu.quit")
            ),
        )
    );
}
