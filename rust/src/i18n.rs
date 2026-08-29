use std::env;

pub const SUPPORTED: &[&str] = &["es", "en", "de", "fr", "pt", "it"];

pub fn normalize(value: &str) -> &'static str {
    let code = value
        .trim()
        .to_lowercase()
        .split(['_', '-'])
        .next()
        .unwrap_or("es")
        .to_string();
    SUPPORTED
        .iter()
        .copied()
        .find(|candidate| *candidate == code)
        .unwrap_or("es")
}

pub fn set(value: &str) {
    env::set_var("LTOOLS_LANG", normalize(value));
}

pub fn current() -> &'static str {
    let value = env::var("LTOOLS_LANG")
        .ok()
        .or_else(|| env::var("LC_ALL").ok())
        .or_else(|| env::var("LC_MESSAGES").ok())
        .or_else(|| env::var("LANG").ok())
        .unwrap_or_else(|| "es".into());
    normalize(&value)
}

pub fn text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "app.title") => "LTools",
        ("en", "usage") => "Usage: ltools [command] [options]",
        ("en", "commands") => "Commands:",
        ("en", "help.audit") => "Disk, package and application audit",
        ("en", "help.games") => "Games, Wine and Proton audit",
        ("en", "help.packages") => "Package managers, packages and artifacts",
        ("en", "help.clean") => "Protected cleanup of packages, caches and paths",
        ("en", "help.prefix") => "List, inspect, create and migrate prefixes",
        ("en", "help.defaults") => "Show effective Wine, Proton and launcher defaults",
        ("en", "help.system") => "systemd, services, processes and journal",
        ("en", "help.doctor") => "Host dependencies, FUSE and runtime diagnostics",
        ("en", "help.rollback") => "Restore reversible operations from a plan",
        ("en", "help.common") => "Common options: --dry-run, --plan FILE, --lang LANG, --help, --version",
        ("en", "help.clean.options") => "clean: --package NAME [--manager ...] --orphans --package-caches --flatpak-unused --path PATH [--force]",
        ("en", "help.prefix.options") => "prefix migrate: --source PATH --dest PATH [--include NAME] [--exclude NAME]",
        ("en", "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        ("en", "help.compat") => "TSV report format is compatible with the Bash implementation.",
        ("en", "menu.title") => "=== LTools ===",
        ("en", "menu.audit") => "Audit disks, packages and applications",
        ("en", "menu.games") => "Audit games, Wine and Proton",
        ("en", "menu.clean") => "Clean packages, caches and paths",
        ("en", "menu.prefix") => "Manage Wine/Proton prefixes",
        ("en", "menu.defaults") => "Show default paths",
        ("en", "menu.system") => "Manage services, processes and journal",
        ("en", "menu.doctor") => "Dependency and runtime diagnostics",
        ("en", "menu.help") => "Help",
        ("en", "menu.quit") => "Quit",
        ("en", "menu.prompt") => "Choose an option: ",
        ("de", "app.title") => "LTools",
        ("de", "usage") => "Verwendung: ltools [Befehl] [Optionen]",
        ("de", "commands") => "Befehle:",
        ("de", "help.doctor") => "Abhängigkeiten, FUSE und Laufzeit diagnostizieren",
        ("de", "menu.title") => "=== LTools ===",
        ("de", "menu.audit") => "Datenträger, Pakete und Anwendungen prüfen",
        ("de", "menu.games") => "Spiele, Wine und Proton prüfen",
        ("de", "menu.clean") => "Pakete, Caches und Pfade bereinigen",
        ("de", "menu.prefix") => "Wine-/Proton-Präfixe verwalten",
        ("de", "menu.defaults") => "Standardpfade anzeigen",
        ("de", "menu.system") => "Dienste, Prozesse und Journal verwalten",
        ("de", "menu.doctor") => "Abhängigkeiten und Laufzeit diagnostizieren",
        ("de", "menu.help") => "Hilfe",
        ("de", "menu.quit") => "Beenden",
        ("de", "menu.prompt") => "Option wählen: ",
        ("fr", "app.title") => "LTools",
        ("fr", "usage") => "Utilisation : ltools [commande] [options]",
        ("fr", "commands") => "Commandes :",
        ("fr", "help.doctor") => "Diagnostiquer les dépendances, FUSE et l’exécution",
        ("fr", "menu.title") => "=== LTools ===",
        ("fr", "menu.audit") => "Auditer les disques, paquets et applications",
        ("fr", "menu.games") => "Auditer les jeux, Wine et Proton",
        ("fr", "menu.clean") => "Nettoyer les paquets, caches et chemins",
        ("fr", "menu.prefix") => "Gérer les préfixes Wine/Proton",
        ("fr", "menu.defaults") => "Afficher les chemins par défaut",
        ("fr", "menu.system") => "Gérer les services, processus et journal",
        ("fr", "menu.doctor") => "Diagnostiquer les dépendances et l’exécution",
        ("fr", "menu.help") => "Aide",
        ("fr", "menu.quit") => "Quitter",
        ("fr", "menu.prompt") => "Choisissez une option : ",
        ("pt", "app.title") => "LTools",
        ("pt", "usage") => "Uso: ltools [comando] [opções]",
        ("pt", "commands") => "Comandos:",
        ("pt", "help.doctor") => "Diagnosticar dependências, FUSE e execução",
        ("pt", "menu.title") => "=== LTools ===",
        ("pt", "menu.audit") => "Auditar discos, pacotes e aplicações",
        ("pt", "menu.games") => "Auditar jogos, Wine e Proton",
        ("pt", "menu.clean") => "Limpar pacotes, caches e caminhos",
        ("pt", "menu.prefix") => "Gerir prefixos Wine/Proton",
        ("pt", "menu.defaults") => "Mostrar caminhos predefinidos",
        ("pt", "menu.system") => "Gerir serviços, processos e journal",
        ("pt", "menu.doctor") => "Diagnóstico de dependências e execução",
        ("pt", "menu.help") => "Ajuda",
        ("pt", "menu.quit") => "Sair",
        ("pt", "menu.prompt") => "Escolha uma opção: ",
        ("it", "app.title") => "LTools",
        ("it", "usage") => "Uso: ltools [comando] [opzioni]",
        ("it", "commands") => "Comandi:",
        ("it", "help.doctor") => "Diagnostica dipendenze, FUSE ed esecuzione",
        ("it", "menu.title") => "=== LTools ===",
        ("it", "menu.audit") => "Verifica dischi, pacchetti e applicazioni",
        ("it", "menu.games") => "Verifica giochi, Wine e Proton",
        ("it", "menu.clean") => "Pulisci pacchetti, cache e percorsi",
        ("it", "menu.prefix") => "Gestisci prefissi Wine/Proton",
        ("it", "menu.defaults") => "Mostra percorsi predefiniti",
        ("it", "menu.system") => "Gestisci servizi, processi e journal",
        ("it", "menu.doctor") => "Diagnostica dipendenze ed esecuzione",
        ("it", "menu.help") => "Aiuto",
        ("it", "menu.quit") => "Esci",
        ("it", "menu.prompt") => "Scegli un’opzione: ",
        (_, "app.title") => "LTools",
        (_, "usage") => "Uso: ltools [comando] [opciones]",
        (_, "commands") => "Comandos:",
        (_, "help.audit") => "Auditoría de discos, paquetes y aplicaciones",
        (_, "help.games") => "Auditoría de juegos, Wine y Proton",
        (_, "help.packages") => "Inventario de gestores, paquetes y artefactos",
        (_, "help.clean") => "Limpieza protegida de paquetes, cachés y rutas",
        (_, "help.prefix") => "Listar, inspeccionar, crear y migrar prefijos",
        (_, "help.defaults") => "Rutas efectivas de Wine, Proton y lanzadores",
        (_, "help.system") => "systemd, servicios, procesos y journal",
        (_, "help.doctor") => "Diagnóstico de dependencias, FUSE y ejecución",
        (_, "help.rollback") => "Recuperar operaciones reversibles de un plan",
        (_, "help.common") => "Opciones comunes: --dry-run, --plan FICHERO, --lang IDIOMA, --help, --version",
        (_, "help.clean.options") => "clean: --package NOMBRE [--manager ...] --orphans --package-caches --flatpak-unused --path RUTA [--force]",
        (_, "help.prefix.options") => "prefix migrate: --source RUTA --dest RUTA [--include NOMBRE] [--exclude NOMBRE]",
        (_, "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        (_, "help.compat") => "El formato TSV de los informes es compatible con la implementación Bash.",
        (_, "menu.title") => "=== LTools ===",
        (_, "menu.audit") => "Auditar discos, paquetes y aplicaciones",
        (_, "menu.games") => "Auditar juegos, Wine y Proton",
        (_, "menu.clean") => "Limpiar paquetes, cachés y rutas",
        (_, "menu.prefix") => "Gestionar prefijos Wine/Proton",
        (_, "menu.defaults") => "Mostrar rutas predeterminadas",
        (_, "menu.system") => "Gestionar servicios, procesos y journal",
        (_, "menu.doctor") => "Diagnóstico de dependencias y ejecución",
        (_, "menu.help") => "Ayuda",
        (_, "menu.quit") => "Salir",
        (_, "menu.prompt") => "Elige una opción: ",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize, SUPPORTED};

    #[test]
    fn normalizes_language_variants() {
        assert_eq!(normalize("en_US.UTF-8"), "en");
        assert_eq!(normalize("pt-BR"), "pt");
        assert_eq!(normalize("unknown"), "es");
    }

    #[test]
    fn exposes_the_supported_catalog_languages() {
        assert_eq!(SUPPORTED, &["es", "en", "de", "fr", "pt", "it"]);
    }
}
