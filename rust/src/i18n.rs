use std::env;

pub const SUPPORTED: &[&str] = &["es", "en", "de", "fr", "pt", "it", "ca", "nl", "pl"];

/// Nombre visible de producto. El binario y sus identificadores técnicos
/// siguen llamándose `ltools` en ambas plataformas para conservar compatibilidad.
#[cfg(windows)]
pub const PRODUCT_NAME: &str = "WinSlim-Tools";
#[cfg(not(windows))]
pub const PRODUCT_NAME: &str = "LTools";

#[cfg(windows)]
const MENU_TITLE: &str = "=== WinSlim-Tools ===";
#[cfg(not(windows))]
const MENU_TITLE: &str = "=== LTools ===";

pub fn product_name() -> &'static str {
    PRODUCT_NAME
}

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
    if value.trim().eq_ignore_ascii_case("auto") {
        env::remove_var("LTOOLS_LANG");
    } else {
        env::set_var("LTOOLS_LANG", normalize(value));
    }
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

/// Etiquetas cuyo significado cambia por plataforma. En Windows el
/// ejecutable inspecciona lanzadores y rutas nativas; nunca anuncia ni
/// ejecuta el catálogo Linux de Wine/Lutris/Heroic/UMU.
pub fn games_label() -> &'static str {
    #[cfg(windows)]
    {
        return match current() {
            "en" => "Inventory native Windows games and launchers",
            "de" => "Native Windows-Spiele und Launcher inventarisieren",
            "fr" => "Inventorier les jeux et lanceurs Windows natifs",
            "pt" => "Inventariar jogos e lançadores nativos do Windows",
            "it" => "Inventaria giochi e launcher Windows nativi",
            "ca" => "Inventariar jocs i llançadors natius de Windows",
            "nl" => "Native Windows-games en launchers inventariseren",
            "pl" => "Inwentaryzuj natywne gry i launchery Windows",
            _ => "Inventario de juegos y lanzadores Windows nativos",
        };
    }
    #[cfg(not(windows))]
    {
        text("menu.games")
    }
}

pub fn games_help() -> &'static str {
    #[cfg(windows)]
    {
        return match current() {
            "en" => "Native Windows game and launcher inventory",
            "de" => "Inventar nativer Windows-Spiele und Launcher",
            "fr" => "Inventaire des jeux et lanceurs Windows natifs",
            "pt" => "Inventário de jogos e lançadores nativos do Windows",
            "it" => "Inventario di giochi e launcher Windows nativi",
            "ca" => "Inventari de jocs i llançadors natius de Windows",
            "nl" => "Inventaris van native Windows-games en launchers",
            "pl" => "Inwentaryzacja natywnych gier i launcherów Windows",
            _ => "Inventario nativo de juegos y lanzadores Windows",
        };
    }
    #[cfg(not(windows))]
    {
        text("help.games")
    }
}

#[allow(dead_code)]
pub fn prefix_label() -> &'static str {
    #[cfg(windows)]
    {
        return match current() {
            "en" => "Wine/Proton prefixes (not applicable on Windows)",
            "de" => "Wine-/Proton-Präfixe (unter Windows nicht anwendbar)",
            "fr" => "Préfixes Wine/Proton (inapplicables sous Windows)",
            "pt" => "Prefixos Wine/Proton (não aplicáveis no Windows)",
            "it" => "Prefissi Wine/Proton (non applicabili su Windows)",
            "ca" => "Prefixos Wine/Proton (no aplicables a Windows)",
            "nl" => "Wine-/Proton-prefixes (niet van toepassing op Windows)",
            "pl" => "Prefiksy Wine/Proton (nie dotyczą Windows)",
            _ => "Prefijos Wine/Proton (no aplicable en Windows)",
        };
    }
    #[cfg(not(windows))]
    {
        text("menu.prefix")
    }
}

pub fn prefix_help() -> &'static str {
    #[cfg(windows)]
    {
        return match current() {
            "en" => "Not available in native Windows mode; no Wine/Proton paths are scanned",
            "de" => "Im nativen Windows-Modus nicht verfügbar; keine Wine-/Proton-Pfade werden geprüft",
            "fr" => "Indisponible en mode Windows natif ; aucun chemin Wine/Proton n’est analysé",
            "pt" => "Indisponível no modo Windows nativo; não são analisados caminhos Wine/Proton",
            "it" => "Non disponibile in modalità Windows nativa; nessun percorso Wine/Proton viene analizzato",
            "ca" => "No disponible en mode Windows natiu; no s’analitzen rutes Wine/Proton",
            "nl" => "Niet beschikbaar in native Windows-modus; er worden geen Wine-/Proton-paden gescand",
            "pl" => "Niedostępne w natywnym trybie Windows; ścieżki Wine/Proton nie są skanowane",
            _ => "No aplicable en Windows nativo; no se escanean rutas Wine/Proton",
        };
    }
    #[cfg(not(windows))]
    {
        text("help.prefix")
    }
}

pub fn defaults_help() -> &'static str {
    #[cfg(windows)]
    {
        return match current() {
            "en" => "Show native Windows launcher locations",
            "de" => "Native Windows-Launcherpfade anzeigen",
            "fr" => "Afficher les emplacements des lanceurs Windows natifs",
            "pt" => "Mostrar localizações dos lançadores nativos do Windows",
            "it" => "Mostra i percorsi dei launcher Windows nativi",
            "ca" => "Mostrar ubicacions dels llançadors natius de Windows",
            "nl" => "Native Windows-launcherlocaties tonen",
            "pl" => "Pokaż lokalizacje natywnych launcherów Windows",
            _ => "Mostrar rutas de lanzadores nativos de Windows",
        };
    }
    #[cfg(not(windows))]
    {
        text("help.defaults")
    }
}

pub fn system_help() -> &'static str {
    #[cfg(windows)]
    {
        return match current() {
            "en" => "Windows services, processes, event log and controlled actions",
            "de" => "Windows-Dienste, Prozesse, Ereignisprotokoll und kontrollierte Aktionen",
            "fr" => "Services Windows, processus, journal des événements et actions contrôlées",
            "pt" => "Serviços, processos, eventos e ações controladas do Windows",
            "it" => "Servizi Windows, processi, registro eventi e azioni controllate",
            "ca" => "Serveis, processos, registre d’esdeveniments i accions controlades de Windows",
            "nl" => "Windows-services, processen, gebeurtenislogboek en gecontroleerde acties",
            "pl" => "Usługi Windows, procesy, dziennik zdarzeń i kontrolowane działania",
            _ => "Servicios, procesos, eventos y acciones controladas de Windows",
        };
    }
    #[cfg(not(windows))]
    {
        text("help.system")
    }
}

pub fn system_options() -> &'static str {
    #[cfg(windows)]
    {
        match current() {
            "en" => "system: status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NAME --limit N | service ACTION NAME | process ACTION PID | export --format json|tsv --out FILE",
            "de" => "system: status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NAME --limit N | service AKTION NAME | process AKTION PID | export --format json|tsv --out DATEI",
            "fr" => "system : status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NOM --limit N | service ACTION NOM | process ACTION PID | export --format json|tsv --out FICHIER",
            "pt" => "system: status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NOME --limit N | service AÇÃO NOME | process AÇÃO PID | export --format json|tsv --out FICHEIRO",
            "it" => "system: status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NOME --limit N | service AZIONE NOME | process AZIONE PID | export --format json|tsv --out FILE",
            "ca" => "system: status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NOM --limit N | service ACCIÓ NOM | process ACCIÓ PID | export --format json|tsv --out FITXER",
            "nl" => "system: status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NAAM --limit N | service ACTIE NAAM | process ACTIE PID | export --format json|tsv --out BESTAND",
            "pl" => "system: status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NAZWA --limit N | service AKCJA NAZWA | process AKCJA PID | export --format json|tsv --out PLIK",
            _ => "system: status | services --filter active|stopped|all | processes --sort cpu|memory --limit N | journal --channel NOMBRE --limit N | service ACCIÓN NOMBRE | process ACCIÓN PID | export --format json|tsv --out FICHERO",
        }
    }
    #[cfg(not(windows))]
    {
        text("help.system.options")
    }
}

pub fn storage_label() -> &'static str {
    #[cfg(windows)]
    {
        return match current() {
            "en" => "Manage Windows disks and partitions",
            "de" => "Windows-Datenträger und Partitionen verwalten",
            "fr" => "Gérer les disques et partitions Windows",
            "pt" => "Gerir discos e partições do Windows",
            "it" => "Gestisci dischi e partizioni Windows",
            "ca" => "Gestionar discs i particions de Windows",
            "nl" => "Windows-schijven en partities beheren",
            "pl" => "Zarządzaj dyskami i partycjami Windows",
            _ => "Gestionar discos y particiones Windows",
        };
    }
    #[cfg(not(windows))]
    {
        match current() {
            "en" => "Manage Linux disks and partitions",
            "de" => "Linux-Datenträger und Partitionen verwalten",
            "fr" => "Gérer les disques et partitions Linux",
            "pt" => "Gerir discos e partições Linux",
            "it" => "Gestisci dischi e partizioni Linux",
            "ca" => "Gestionar discs i particions de Linux",
            "nl" => "Linux-schijven en partities beheren",
            "pl" => "Zarządzaj dyskami i partycjami Linux",
            _ => "Gestionar discos y particiones Linux",
        }
    }
}

pub fn registry_label() -> &'static str {
    #[cfg(windows)]
    {
        return match current() {
            "en" => "Inspect Windows Registry",
            "de" => "Windows-Registrierung prüfen",
            "fr" => "Inspecter le registre Windows",
            "pt" => "Inspecionar o Registo do Windows",
            "it" => "Ispeziona il Registro di Windows",
            "ca" => "Inspeccionar el Registre de Windows",
            "nl" => "Windows-register inspecteren",
            "pl" => "Inspekcja rejestru Windows",
            _ => "Inspeccionar el Registro de Windows",
        };
    }
    #[cfg(not(windows))]
    {
        match current() {
            "en" => "Inspect Linux logs and configuration paths",
            "de" => "Linux-Protokolle und Konfigurationspfade prüfen",
            "fr" => "Inspecter les journaux et chemins de configuration Linux",
            "pt" => "Inspecionar registos e caminhos de configuração Linux",
            "it" => "Ispeziona log e percorsi di configurazione Linux",
            "ca" => "Inspeccionar registres i rutes de configuració de Linux",
            "nl" => "Linux-logs en configuratiepaden inspecteren",
            "pl" => "Sprawdź dzienniki i ścieżki konfiguracji Linuksa",
            _ => "Registros y configuración Linux",
        }
    }
}

pub fn storage_help() -> &'static str {
    #[cfg(windows)]
    {
        match current() {
            "en" => "Native disk, volume and partition inventory (PowerShell/diskpart detection)",
            "de" => "Native Inventarisierung von Datenträgern, Volumes und Partitionen (PowerShell/DiskPart)",
            "fr" => "Inventaire natif des disques, volumes et partitions (détection PowerShell/DiskPart)",
            "pt" => "Inventário nativo de discos, volumes e partições (deteção PowerShell/DiskPart)",
            "it" => "Inventario nativo di dischi, volumi e partizioni (rilevamento PowerShell/DiskPart)",
            "ca" => "Inventari natiu de discs, volums i particions (detecció PowerShell/DiskPart)",
            "nl" => "Native inventarisatie van schijven, volumes en partities (PowerShell/DiskPart)",
            "pl" => "Natywny spis dysków, woluminów i partycji (wykrywanie PowerShell/DiskPart)",
            _ => "Inventario nativo de discos, volúmenes y particiones (detección PowerShell/DiskPart)",
        }
    }
    #[cfg(not(windows))]
    {
        match current() {
            "en" => "Linux disks, mounts and partitions (lsblk/parted/gparted)",
            "de" => "Linux-Datenträger, Mounts und Partitionen (lsblk/parted/gparted)",
            "fr" => "Disques, montages et partitions Linux (lsblk/parted/gparted)",
            "pt" => "Discos, montagens e partições Linux (lsblk/parted/gparted)",
            "it" => "Dischi, mount e partizioni Linux (lsblk/parted/gparted)",
            "ca" => "Discs, muntatges i particions de Linux (lsblk/parted/gparted)",
            "nl" => "Linux-schijven, mounts en partities (lsblk/parted/gparted)",
            "pl" => "Dyski, montowania i partycje Linuksa (lsblk/parted/gparted)",
            _ => "Inventario de discos, montajes y particiones (lsblk/parted/gparted)",
        }
    }
}

pub fn registry_help() -> &'static str {
    #[cfg(windows)]
    {
        match current() {
            "en" => "Read or export Windows Registry keys with reg.exe",
            "de" => "Windows-Registrierungsschlüssel mit reg.exe lesen oder exportieren",
            "fr" => "Lire ou exporter des clés du registre Windows avec reg.exe",
            "pt" => "Ler ou exportar chaves do Registo do Windows com reg.exe",
            "it" => "Leggi o esporta chiavi del Registro di Windows con reg.exe",
            "ca" => "Llegir o exportar claus del Registre de Windows amb reg.exe",
            "nl" => "Windows-registersleutels lezen of exporteren met reg.exe",
            "pl" => "Odczytuj lub eksportuj klucze rejestru Windows za pomocą reg.exe",
            _ => "Leer o exportar claves del Registro de Windows con reg.exe",
        }
    }
    #[cfg(not(windows))]
    {
        match current() {
            "en" => "Configuration paths and alternatives to a central registry",
            "de" => "Konfigurationspfade und Alternativen zu einer zentralen Registrierung",
            "fr" => "Chemins de configuration et alternatives au registre central",
            "pt" => "Caminhos de configuração e alternativas a um registo central",
            "it" => "Percorsi di configurazione e alternative a un registro centrale",
            "ca" => "Rutes de configuració i alternatives al registre central",
            "nl" => "Configuratiepaden en alternatieven voor een centraal register",
            "pl" => "Ścieżki konfiguracji i alternatywy dla centralnego rejestru",
            _ => "Rutas de configuración y alternativas al registro central",
        }
    }
}

pub fn prefix_options() -> &'static str {
    #[cfg(windows)]
    {
        "prefix: no aplicable en Windows; el EXE no busca ni migra prefijos Wine/Proton"
    }
    #[cfg(not(windows))]
    {
        text("help.prefix.options")
    }
}

pub fn prefix_flags() -> &'static str {
    #[cfg(windows)]
    {
        ""
    }
    #[cfg(not(windows))]
    {
        text("help.prefix.flags")
    }
}

/// Textos de la ventana gráfica. Se mantienen aquí para que GUI y CLI
/// compartan el idioma seleccionado y no introduzcan cadenas de plataforma
/// cruzada en los módulos gráficos.
#[cfg(any(target_os = "linux", windows))]
pub fn gui_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "title") => "LTools",
        ("en", "subtitle") => "Safe system tools and quick actions",
        ("en", "ready") => "Ready",
        ("en", "running") => "Running…",
        ("en", "completed") => "Completed",
        ("en", "audit") => "Audit disks and applications",
        ("en", "games") => "Inventory games and launchers",
        ("en", "packages") => "Package inventory",
        ("en", "prefixes") => "Wine/Proton prefixes",
        ("en", "defaults") => "Default paths",
        ("en", "system") => "System status",
        ("en", "doctor") => "Dependencies and diagnostics",
        ("en", "storage") => "Disks and partitions",
        ("en", "stores") => "Package stores",
        ("en", "git") => "Git status",
        ("en", "package_placeholder") => "Package name…",
        ("en", "search") => "Search package",
        ("en", "enter_package") => "Enter a package name first",
        ("en", "close") => "Close",
        ("de", "title") => "LTools",
        ("de", "subtitle") => "Sichere Systemwerkzeuge und Schnellaktionen",
        ("de", "ready") => "Bereit",
        ("de", "running") => "Wird ausgeführt…",
        ("de", "completed") => "Abgeschlossen",
        ("de", "audit") => "Datenträger und Anwendungen prüfen",
        ("de", "games") => "Spiele und Launcher inventarisieren",
        ("de", "packages") => "Paketinventar",
        ("de", "prefixes") => "Wine-/Proton-Präfixe",
        ("de", "defaults") => "Standardpfade",
        ("de", "system") => "Systemstatus",
        ("de", "doctor") => "Abhängigkeiten und Diagnose",
        ("de", "storage") => "Datenträger und Partitionen",
        ("de", "stores") => "Paketquellen",
        ("de", "git") => "Git-Status",
        ("de", "package_placeholder") => "Paketname…",
        ("de", "search") => "Paket suchen",
        ("de", "enter_package") => "Zuerst einen Paketnamen eingeben",
        ("de", "close") => "Schließen",
        ("fr", "title") => "LTools",
        ("fr", "subtitle") => "Outils système sûrs et actions rapides",
        ("fr", "ready") => "Prêt",
        ("fr", "running") => "Exécution…",
        ("fr", "completed") => "Terminé",
        ("fr", "audit") => "Auditer les disques et applications",
        ("fr", "games") => "Inventorier jeux et lanceurs",
        ("fr", "packages") => "Inventaire des paquets",
        ("fr", "prefixes") => "Préfixes Wine/Proton",
        ("fr", "defaults") => "Chemins par défaut",
        ("fr", "system") => "État du système",
        ("fr", "doctor") => "Dépendances et diagnostic",
        ("fr", "storage") => "Disques et partitions",
        ("fr", "stores") => "Sources de paquets",
        ("fr", "git") => "État Git",
        ("fr", "package_placeholder") => "Nom du paquet…",
        ("fr", "search") => "Rechercher un paquet",
        ("fr", "enter_package") => "Saisissez d’abord un nom de paquet",
        ("fr", "close") => "Fermer",
        ("pt", "title") => "LTools",
        ("pt", "subtitle") => "Ferramentas de sistema seguras e ações rápidas",
        ("pt", "ready") => "Pronto",
        ("pt", "running") => "A executar…",
        ("pt", "completed") => "Concluído",
        ("pt", "audit") => "Auditar discos e aplicações",
        ("pt", "games") => "Inventariar jogos e lançadores",
        ("pt", "packages") => "Inventário de pacotes",
        ("pt", "prefixes") => "Prefixos Wine/Proton",
        ("pt", "defaults") => "Caminhos predefinidos",
        ("pt", "system") => "Estado do sistema",
        ("pt", "doctor") => "Dependências e diagnóstico",
        ("pt", "storage") => "Discos e partições",
        ("pt", "stores") => "Fontes de pacotes",
        ("pt", "git") => "Estado do Git",
        ("pt", "package_placeholder") => "Nome do pacote…",
        ("pt", "search") => "Pesquisar pacote",
        ("pt", "enter_package") => "Introduza primeiro um nome de pacote",
        ("pt", "close") => "Fechar",
        (_, "title") => "LTools",
        (_, "subtitle") => "Herramientas seguras del sistema y acciones rápidas",
        (_, "ready") => "Listo",
        (_, "running") => "Ejecutando…",
        (_, "completed") => "Terminado",
        (_, "audit") => "Auditar discos y aplicaciones",
        (_, "games") => "Inventariar juegos y lanzadores",
        (_, "packages") => "Inventario de paquetes",
        (_, "prefixes") => "Prefijos Wine/Proton",
        (_, "defaults") => "Rutas predeterminadas",
        (_, "winslim") => "WinSlim",
        (_, "system") => "Estado del sistema",
        (_, "doctor") => "Dependencias y diagnóstico",
        (_, "storage") => "Discos y particiones",
        (_, "clean") => "Revisar limpieza",
        (_, "stores") => "Almacenes de paquetes",
        (_, "git") => "Estado de Git",
        (_, "registry") => "Registros y configuración",
        (_, "automation_name") => "Nombre de la automatización",
        (_, "automation_program") => "Programa o ruta del script",
        (_, "automation_cwd") => "Directorio de trabajo (opcional)",
        (_, "automation_args") => "Argumentos (comillas para espacios)",
        (_, "register") => "Registrar script",
        (_, "required") => "Nombre y programa son obligatorios",
        (_, "package_placeholder") => "Nombre del paquete…",
        (_, "search") => "Buscar paquete",
        (_, "enter_package") => "Introduce primero un nombre de paquete",
        (_, "close") => "Cerrar",
        _ => "",
    }
}

/// Textos del módulo de paquetes/Git. Las operaciones y sus argumentos son
/// estables para automatización; solo se traduce la interfaz visible.
pub fn tools_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "menu") => "Packages, stores and Git",
        ("en", "help") => "Search/install from detected stores and perform guarded Git operations",
        ("en", "title") => "=== Packages, stores and Git ===",
        ("en", "search") => "Search a package in available stores",
        ("en", "install") => "Choose and install a package",
        ("en", "git_status") => "Git repository status",
        ("en", "git_clone") => "Clone a Git repository",
        ("en", "git_fetch") => "Fetch remote Git references",
        ("en", "git_pull") => "Pull and integrate Git changes",
        ("en", "git_login") => "Check Git identity and optional GitHub login",
        ("en", "stores_title") => "Detected package stores",
        ("en", "search_title") => "Package search",
        ("en", "no_candidates") => "No candidates found in the available stores.",
        ("en", "no_results") => "No package candidate was found.",
        ("en", "query_prompt") => "Package name (Enter to go back): ",
        ("en", "select_candidate") => "Choose a candidate number: ",
        ("en", "selected") => "Selected:",
        ("en", "confirm_install") => "Install this package?",
        ("en", "cancelled") => "Operation cancelled.",
        ("en", "dry_run") => "Dry run: nothing was changed.",
        ("en", "done") => "Operation completed.",
        ("en", "pause") => "Press Enter to return: ",
        ("de", "menu") => "Pakete, Quellen und Git",
        ("de", "help") => "Suche/Installation aus erkannten Quellen und geschützte Git-Operationen",
        ("de", "title") => "=== Pakete, Quellen und Git ===",
        ("de", "search") => "Paket in verfügbaren Quellen suchen",
        ("de", "install") => "Paket auswählen und installieren",
        ("de", "git_status") => "Git-Repository-Status",
        ("de", "git_clone") => "Git-Repository klonen",
        ("de", "git_fetch") => "Remote-Git-Referenzen abrufen",
        ("de", "git_pull") => "Git-Änderungen abrufen und integrieren",
        ("de", "git_login") => "Git-Identität und optionale GitHub-Anmeldung prüfen",
        ("de", "stores_title") => "Erkannte Paketquellen",
        ("de", "search_title") => "Paketsuche",
        ("de", "no_candidates") => "Keine Kandidaten in den verfügbaren Quellen gefunden.",
        ("de", "no_results") => "Kein Paketkandidat gefunden.",
        ("de", "query_prompt") => "Paketname (Enter zum Zurückgehen): ",
        ("de", "select_candidate") => "Nummer des Kandidaten: ",
        ("de", "selected") => "Ausgewählt:",
        ("de", "confirm_install") => "Dieses Paket installieren?",
        ("de", "cancelled") => "Vorgang abgebrochen.",
        ("de", "dry_run") => "Simulation: Es wurde nichts geändert.",
        ("de", "done") => "Vorgang abgeschlossen.",
        ("de", "pause") => "Enter zum Zurückgehen: ",
        ("fr", "menu") => "Paquets, sources et Git",
        ("fr", "help") => {
            "Rechercher/installer depuis les sources détectées et gérer Git prudemment"
        }
        ("fr", "title") => "=== Paquets, sources et Git ===",
        ("fr", "search") => "Rechercher un paquet dans les sources disponibles",
        ("fr", "install") => "Choisir et installer un paquet",
        ("fr", "git_status") => "État du dépôt Git",
        ("fr", "git_clone") => "Cloner un dépôt Git",
        ("fr", "git_fetch") => "Récupérer les références Git distantes",
        ("fr", "git_pull") => "Récupérer et intégrer les changements Git",
        ("fr", "git_login") => "Vérifier l’identité Git et la connexion GitHub optionnelle",
        ("fr", "stores_title") => "Sources de paquets détectées",
        ("fr", "search_title") => "Recherche de paquets",
        ("fr", "no_candidates") => "Aucun candidat trouvé dans les sources disponibles.",
        ("fr", "no_results") => "Aucun candidat de paquet trouvé.",
        ("fr", "query_prompt") => "Nom du paquet (Entrée pour revenir) : ",
        ("fr", "select_candidate") => "Numéro du candidat : ",
        ("fr", "selected") => "Sélectionné :",
        ("fr", "confirm_install") => "Installer ce paquet ?",
        ("fr", "cancelled") => "Opération annulée.",
        ("fr", "dry_run") => "Simulation : aucune modification.",
        ("fr", "done") => "Opération terminée.",
        ("fr", "pause") => "Entrée pour revenir : ",
        ("pt", "menu") => "Pacotes, fontes e Git",
        ("pt", "help") => {
            "Pesquisar/instalar nas fontes detetadas e executar operações Git protegidas"
        }
        ("pt", "title") => "=== Pacotes, fontes e Git ===",
        ("pt", "search") => "Pesquisar um pacote nas fontes disponíveis",
        ("pt", "install") => "Escolher e instalar um pacote",
        ("pt", "git_status") => "Estado do repositório Git",
        ("pt", "git_clone") => "Clonar um repositório Git",
        ("pt", "git_fetch") => "Obter referências Git remotas",
        ("pt", "git_pull") => "Obter e integrar alterações Git",
        ("pt", "git_login") => "Verificar identidade Git e início de sessão GitHub opcional",
        ("pt", "stores_title") => "Fontes de pacotes detetadas",
        ("pt", "search_title") => "Pesquisa de pacotes",
        ("pt", "no_candidates") => "Não foram encontrados candidatos nas fontes disponíveis.",
        ("pt", "no_results") => "Não foi encontrado nenhum pacote.",
        ("pt", "query_prompt") => "Nome do pacote (Enter para voltar): ",
        ("pt", "select_candidate") => "Número do candidato: ",
        ("pt", "selected") => "Selecionado:",
        ("pt", "confirm_install") => "Instalar este pacote?",
        ("pt", "cancelled") => "Operação cancelada.",
        ("pt", "dry_run") => "Simulação: nada foi alterado.",
        ("pt", "done") => "Operação concluída.",
        ("pt", "pause") => "Enter para voltar: ",
        ("it", "menu") => "Pacchetti, fonti e Git",
        ("it", "help") => "Cerca/installa dalle fonti rilevate ed esegui operazioni Git protette",
        ("it", "title") => "=== Pacchetti, fonti e Git ===",
        ("it", "search") => "Cerca un pacchetto nelle fonti disponibili",
        ("it", "install") => "Scegli e installa un pacchetto",
        ("ca", "menu") => "Paquets, fonts i Git",
        ("ca", "help") => {
            "Cerca/instal·la des de les fonts detectades i executa operacions Git protegides"
        }
        ("ca", "title") => "=== Paquets, fonts i Git ===",
        ("ca", "search") => "Cercar un paquet a les fonts disponibles",
        ("ca", "install") => "Triar i instal·lar un paquet",
        ("nl", "menu") => "Pakketten, bronnen en Git",
        ("nl", "help") => {
            "Zoek/installeer uit gevonden bronnen en voer beveiligde Git-bewerkingen uit"
        }
        ("nl", "title") => "=== Pakketten, bronnen en Git ===",
        ("nl", "search") => "Zoek een pakket in beschikbare bronnen",
        ("nl", "install") => "Kies en installeer een pakket",
        ("pl", "menu") => "Pakiety, źródła i Git",
        ("pl", "help") => "Szukaj/instaluj z wykrytych źródeł i wykonuj chronione operacje Git",
        ("pl", "title") => "=== Pakiety, źródła i Git ===",
        ("pl", "search") => "Szukaj pakietu w dostępnych źródłach",
        ("pl", "install") => "Wybierz i zainstaluj pakiet",
        (_, "menu") => "Paquetes, almacenes y Git",
        (_, "help") => "Buscar/instalar en stores detectadas y ejecutar operaciones Git protegidas",
        (_, "title") => "=== Paquetes, almacenes y Git ===",
        (_, "search") => "Buscar un paquete en las stores disponibles",
        (_, "install") => "Elegir e instalar un paquete",
        (_, "git_status") => "Estado del repositorio Git",
        (_, "git_clone") => "Clonar un repositorio Git",
        (_, "git_fetch") => "Descargar referencias Git remotas",
        (_, "git_pull") => "Descargar e integrar cambios Git",
        (_, "git_login") => "Comprobar identidad Git e inicio de sesión GitHub opcional",
        (_, "stores_title") => "Stores de paquetes detectadas",
        (_, "search_title") => "Búsqueda de paquetes",
        (_, "no_candidates") => "No se encontraron candidatos en las stores disponibles.",
        (_, "no_results") => "No se encontró ningún candidato de paquete.",
        (_, "query_prompt") => "Nombre del paquete (Enter para volver): ",
        (_, "select_candidate") => "Elige el número del candidato: ",
        (_, "selected") => "Seleccionado:",
        (_, "confirm_install") => "¿Instalar este paquete?",
        (_, "cancelled") => "Operación cancelada.",
        (_, "dry_run") => "Simulación: no se ha modificado nada.",
        (_, "done") => "Operación terminada.",
        (_, "pause") => "Pulsa Enter para volver: ",
        (_, _) => "",
    }
}

pub fn text(key: &str) -> &'static str {
    if key == "app.title" {
        return PRODUCT_NAME;
    }
    if key == "menu.title" {
        return MENU_TITLE;
    }
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
        ("en", "help.system") => "systemd health, services, processes, journal and controlled actions",
        ("en", "help.system.options") => "system: status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service ACTION UNIT | dependencies --unit UNIT | export --format tsv|json --out FILE",
        ("en", "help.doctor") => "Host dependencies, FUSE and runtime diagnostics",
        ("en", "help.rollback") => "Restore reversible operations from a plan",
        ("en", "help.capabilities") => "Print the machine-readable JSON integration contract",
        ("en", "help.common") => "Common options: --dry-run, --plan FILE, --lang LANG, --help, --version",
        ("en", "help.clean.options") => "clean: --package NAME [--manager ...] --orphans --package-caches --flatpak-unused --path PATH [--force]",
        ("en", "help.prefix.options") => "prefix list: --root PATH [--include-mount-roots]\nprefix migrate: --source PATH --dest PATH [--include NAME] [--exclude NAME]",
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
        ("en", "menu.packages") => "Inventory packages and stores",
        ("en", "menu.help") => "Help",
        ("en", "menu.quit") => "Quit",
        ("en", "menu.prompt") => "Choose an option (Enter to go back): ",
        ("en", "menu.back") => "Back",
        ("en", "menu.invalid") => "Invalid option.",
        ("en", "menu.system.title") => "=== Services, processes and journal ===",
        ("en", "menu.system.status") => "systemd status and failed services",
        ("en", "menu.system.services") => "System services",
        ("en", "menu.system.user_services") => "User services",
        ("en", "menu.system.processes") => "Processes by memory",
        ("en", "menu.system.journal") => "Journal: today's warnings",
        ("en", "menu.system.failed") => "Failed services and related journal",
        ("en", "menu.system.manage") => "Manage one service",
        ("en", "menu.system.dependencies") => "Service dependencies and reverse tree",
        ("en", "menu.system.export") => "Export TSV/JSON report",
        ("en", "menu.clean.title") => "=== Protected cleanup ===",
        ("en", "menu.clean.orphans") => "Review orphan packages",
        ("en", "menu.clean.caches") => "Clean package-manager caches",
        ("en", "menu.clean.flatpak") => "Review unused Flatpak runtimes",
        ("en", "menu.clean.path") => "Review a specific path",
        ("en", "menu.clean.package") => "Review a specific package",
        ("de", "app.title") => "LTools",
        ("de", "usage") => "Verwendung: ltools [Befehl] [Optionen]",
        ("de", "commands") => "Befehle:",
        ("de", "help.audit") => "Datenträger-, Paket- und Anwendungsprüfung",
        ("de", "help.games") => "Prüfung von Spielen, Wine und Proton",
        ("de", "help.packages") => "Paketmanager, Pakete und Artefakte",
        ("de", "help.clean") => "Geschützte Bereinigung von Paketen, Caches und Pfaden",
        ("de", "help.prefix") => "Präfixe auflisten, prüfen, erstellen und migrieren",
        ("de", "help.defaults") => "Effektive Standardpfade für Wine, Proton und Launcher anzeigen",
        ("de", "help.system") => "systemd-Gesundheit, Dienste, Prozesse, Journal und kontrollierte Aktionen",
        ("de", "help.system.options") => "system: status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service AKTION EINHEIT | dependencies --unit EINHEIT | export --format tsv|json --out DATEI",
        ("de", "help.doctor") => "Abhängigkeiten, FUSE und Laufzeit diagnostizieren",
        ("de", "help.rollback") => "Umkehrbare Vorgänge aus einem Plan wiederherstellen",
        ("de", "help.capabilities") => "Maschinenlesbaren JSON-Integrationsvertrag ausgeben",
        ("de", "help.common") => "Allgemeine Optionen: --dry-run, --plan DATEI, --lang SPRACHE, --help, --version",
        ("de", "help.clean.options") => "clean: --package NAME [--manager ...] --orphans --package-caches --flatpak-unused --path PFAD [--force]",
        ("de", "help.prefix.options") => "prefix list: --root PFAD [--include-mount-roots]\nprefix migrate: --source PFAD --dest PFAD [--include NAME] [--exclude NAME]",
        ("de", "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        ("de", "help.compat") => "Das TSV-Berichtsformat ist mit der Bash-Implementierung kompatibel.",
        ("de", "menu.title") => "=== LTools ===",
        ("de", "menu.audit") => "Datenträger, Pakete und Anwendungen prüfen",
        ("de", "menu.games") => "Spiele, Wine und Proton prüfen",
        ("de", "menu.clean") => "Pakete, Caches und Pfade bereinigen",
        ("de", "menu.prefix") => "Wine-/Proton-Präfixe verwalten",
        ("de", "menu.defaults") => "Standardpfade anzeigen",
        ("de", "menu.system") => "Dienste, Prozesse und Journal verwalten",
        ("de", "menu.doctor") => "Abhängigkeiten und Laufzeit diagnostizieren",
        ("de", "menu.packages") => "Pakete und Quellen inventarisieren",
        ("de", "menu.help") => "Hilfe",
        ("de", "menu.quit") => "Beenden",
        ("de", "menu.prompt") => "Option wählen (Enter zum Zurückgehen): ",
        ("de", "menu.back") => "Zurück",
        ("de", "menu.invalid") => "Ungültige Option.",
        ("de", "menu.system.title") => "=== Dienste, Prozesse und Journal ===",
        ("de", "menu.system.status") => "systemd-Status und fehlgeschlagene Dienste",
        ("de", "menu.system.services") => "Systemdienste",
        ("de", "menu.system.user_services") => "Benutzerdienste",
        ("de", "menu.system.processes") => "Prozesse nach Speicher",
        ("de", "menu.system.journal") => "Journal: Warnungen von heute",
        ("de", "menu.system.failed") => "Fehlgeschlagene Dienste und zugehöriges Journal",
        ("de", "menu.system.manage") => "Einen Dienst verwalten",
        ("de", "menu.system.dependencies") => "Dienstabhängigkeiten und umgekehrter Baum",
        ("de", "menu.system.export") => "TSV/JSON-Bericht exportieren",
        ("de", "menu.clean.title") => "=== Geschützte Bereinigung ===",
        ("de", "menu.clean.orphans") => "Verwaiste Pakete prüfen",
        ("de", "menu.clean.caches") => "Paketmanager-Caches bereinigen",
        ("de", "menu.clean.flatpak") => "Nicht verwendete Flatpak-Laufzeiten prüfen",
        ("de", "menu.clean.path") => "Einen bestimmten Pfad prüfen",
        ("de", "menu.clean.package") => "Ein bestimmtes Paket prüfen",
        ("fr", "app.title") => "LTools",
        ("fr", "usage") => "Utilisation : ltools [commande] [options]",
        ("fr", "commands") => "Commandes :",
        ("fr", "help.audit") => "Audit des disques, paquets et applications",
        ("fr", "help.games") => "Audit des jeux, de Wine et de Proton",
        ("fr", "help.packages") => "Gestionnaires de paquets, paquets et artefacts",
        ("fr", "help.clean") => "Nettoyage protégé des paquets, caches et chemins",
        ("fr", "help.prefix") => "Lister, inspecter, créer et migrer des préfixes",
        ("fr", "help.defaults") => "Afficher les chemins effectifs par défaut de Wine, Proton et des lanceurs",
        ("fr", "help.system") => "État de systemd, services, processus, journal et actions contrôlées",
        ("fr", "help.system.options") => "system : status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service ACTION UNITÉ | dependencies --unit UNITÉ | export --format tsv|json --out FICHIER",
        ("fr", "help.doctor") => "Diagnostiquer les dépendances, FUSE et l’exécution",
        ("fr", "help.rollback") => "Restaurer les opérations réversibles d’un plan",
        ("fr", "help.capabilities") => "Afficher le contrat d’intégration JSON lisible par machine",
        ("fr", "help.common") => "Options communes : --dry-run, --plan FICHIER, --lang LANGUE, --help, --version",
        ("fr", "help.clean.options") => "clean : --package NOM [--manager ...] --orphans --package-caches --flatpak-unused --path CHEMIN [--force]",
        ("fr", "help.prefix.options") => "prefix list : --root CHEMIN [--include-mount-roots]\nprefix migrate : --source CHEMIN --dest CHEMIN [--include NOM] [--exclude NOM]",
        ("fr", "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        ("fr", "help.compat") => "Le format des rapports TSV est compatible avec l’implémentation Bash.",
        ("fr", "menu.title") => "=== LTools ===",
        ("fr", "menu.audit") => "Auditer les disques, paquets et applications",
        ("fr", "menu.games") => "Auditer les jeux, Wine et Proton",
        ("fr", "menu.clean") => "Nettoyer les paquets, caches et chemins",
        ("fr", "menu.prefix") => "Gérer les préfixes Wine/Proton",
        ("fr", "menu.defaults") => "Afficher les chemins par défaut",
        ("fr", "menu.system") => "Gérer les services, processus et journal",
        ("fr", "menu.doctor") => "Diagnostiquer les dépendances et l’exécution",
        ("fr", "menu.packages") => "Inventorier les paquets et magasins",
        ("fr", "menu.help") => "Aide",
        ("fr", "menu.quit") => "Quitter",
        ("fr", "menu.prompt") => "Choisissez une option (Entrée pour revenir) : ",
        ("fr", "menu.back") => "Retour",
        ("fr", "menu.invalid") => "Option invalide.",
        ("fr", "menu.system.title") => "=== Services, processus et journal ===",
        ("fr", "menu.system.status") => "État de systemd et services en échec",
        ("fr", "menu.system.services") => "Services système",
        ("fr", "menu.system.user_services") => "Services utilisateur",
        ("fr", "menu.system.processes") => "Processus par mémoire",
        ("fr", "menu.system.journal") => "Journal : avertissements du jour",
        ("fr", "menu.system.failed") => "Services en échec et journal associé",
        ("fr", "menu.system.manage") => "Gérer un service",
        ("fr", "menu.system.dependencies") => "Dépendances et arbre inverse",
        ("fr", "menu.system.export") => "Exporter le rapport TSV/JSON",
        ("fr", "menu.clean.title") => "=== Nettoyage protégé ===",
        ("fr", "menu.clean.orphans") => "Vérifier les paquets orphelins",
        ("fr", "menu.clean.caches") => "Nettoyer les caches des gestionnaires",
        ("fr", "menu.clean.flatpak") => "Vérifier les runtimes Flatpak inutilisés",
        ("fr", "menu.clean.path") => "Vérifier un chemin précis",
        ("fr", "menu.clean.package") => "Vérifier un paquet précis",
        ("pt", "app.title") => "LTools",
        ("pt", "usage") => "Uso: ltools [comando] [opções]",
        ("pt", "commands") => "Comandos:",
        ("pt", "help.audit") => "Auditoria de discos, pacotes e aplicações",
        ("pt", "help.games") => "Auditoria de jogos, Wine e Proton",
        ("pt", "help.packages") => "Gestores de pacotes, pacotes e artefactos",
        ("pt", "help.clean") => "Limpeza protegida de pacotes, caches e caminhos",
        ("pt", "help.prefix") => "Listar, inspecionar, criar e migrar prefixos",
        ("pt", "help.defaults") => "Mostrar os caminhos predefinidos efetivos do Wine, Proton e lançadores",
        ("pt", "help.system") => "Estado do systemd, serviços, processos, journal e ações controladas",
        ("pt", "help.system.options") => "system: status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service AÇÃO UNIDADE | dependencies --unit UNIDADE | export --format tsv|json --out FICHEIRO",
        ("pt", "help.doctor") => "Diagnosticar dependências, FUSE e execução",
        ("pt", "help.rollback") => "Restaurar operações reversíveis de um plano",
        ("pt", "help.capabilities") => "Imprimir o contrato de integração JSON legível por máquina",
        ("pt", "help.common") => "Opções comuns: --dry-run, --plan FICHEIRO, --lang IDIOMA, --help, --version",
        ("pt", "help.clean.options") => "clean: --package NOME [--manager ...] --orphans --package-caches --flatpak-unused --path CAMINHO [--force]",
        ("pt", "help.prefix.options") => "prefix list: --root CAMINHO [--include-mount-roots]\nprefix migrate: --source CAMINHO --dest CAMINHO [--include NOME] [--exclude NOME]",
        ("pt", "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        ("pt", "help.compat") => "O formato dos relatórios TSV é compatível com a implementação Bash.",
        ("pt", "menu.title") => "=== LTools ===",
        ("pt", "menu.audit") => "Auditar discos, pacotes e aplicações",
        ("pt", "menu.games") => "Auditar jogos, Wine e Proton",
        ("pt", "menu.clean") => "Limpar pacotes, caches e caminhos",
        ("pt", "menu.prefix") => "Gerir prefixos Wine/Proton",
        ("pt", "menu.defaults") => "Mostrar caminhos predefinidos",
        ("pt", "menu.system") => "Gerir serviços, processos e journal",
        ("pt", "menu.doctor") => "Diagnóstico de dependências e execução",
        ("pt", "menu.packages") => "Inventariar pacotes e lojas",
        ("pt", "menu.help") => "Ajuda",
        ("pt", "menu.quit") => "Sair",
        ("pt", "menu.prompt") => "Escolha uma opção (Enter para voltar): ",
        ("pt", "menu.back") => "Voltar",
        ("pt", "menu.invalid") => "Opção inválida.",
        ("pt", "menu.system.title") => "=== Serviços, processos e journal ===",
        ("pt", "menu.system.status") => "Estado do systemd e serviços falhados",
        ("pt", "menu.system.services") => "Serviços do sistema",
        ("pt", "menu.system.user_services") => "Serviços do utilizador",
        ("pt", "menu.system.processes") => "Processos por memória",
        ("pt", "menu.system.journal") => "Journal: avisos de hoje",
        ("pt", "menu.system.failed") => "Serviços falhados e journal relacionado",
        ("pt", "menu.system.manage") => "Gerir um serviço",
        ("pt", "menu.system.dependencies") => "Dependências e árvore inversa",
        ("pt", "menu.system.export") => "Exportar relatório TSV/JSON",
        ("pt", "menu.clean.title") => "=== Limpeza protegida ===",
        ("pt", "menu.clean.orphans") => "Rever pacotes órfãos",
        ("pt", "menu.clean.caches") => "Limpar caches dos gestores de pacotes",
        ("pt", "menu.clean.flatpak") => "Rever runtimes Flatpak não utilizados",
        ("pt", "menu.clean.path") => "Rever um caminho específico",
        ("pt", "menu.clean.package") => "Rever um pacote específico",
        ("it", "app.title") => "LTools",
        ("it", "usage") => "Uso: ltools [comando] [opzioni]",
        ("it", "commands") => "Comandi:",
        ("it", "help.audit") => "Verifica di dischi, pacchetti e applicazioni",
        ("it", "help.games") => "Verifica di giochi, Wine e Proton",
        ("it", "help.packages") => "Gestori di pacchetti, pacchetti e artefatti",
        ("it", "help.clean") => "Pulizia protetta di pacchetti, cache e percorsi",
        ("it", "help.prefix") => "Elenca, ispeziona, crea e migra prefissi",
        ("it", "help.defaults") => "Mostra i percorsi predefiniti effettivi di Wine, Proton e dei launcher",
        ("it", "help.system") => "Stato di systemd, servizi, processi, journal e azioni controllate",
        ("it", "help.system.options") => "system: status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service AZIONE UNITÀ | dependencies --unit UNITÀ | export --format tsv|json --out FILE",
        ("it", "help.doctor") => "Diagnostica dipendenze, FUSE ed esecuzione",
        ("it", "help.rollback") => "Ripristina le operazioni reversibili da un piano",
        ("it", "help.capabilities") => "Stampa il contratto di integrazione JSON leggibile dalle macchine",
        ("it", "help.common") => "Opzioni comuni: --dry-run, --plan FILE, --lang LINGUA, --help, --version",
        ("it", "help.clean.options") => "clean: --package NOME [--manager ...] --orphans --package-caches --flatpak-unused --path PERCORSO [--force]",
        ("it", "help.prefix.options") => "prefix list: --root PERCORSO [--include-mount-roots]\nprefix migrate: --source PERCORSO --dest PERCORSO [--include NOME] [--exclude NOME]",
        ("it", "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        ("it", "help.compat") => "Il formato dei rapporti TSV è compatibile con l’implementazione Bash.",
        ("it", "menu.title") => "=== LTools ===",
        ("it", "menu.audit") => "Verifica dischi, pacchetti e applicazioni",
        ("it", "menu.games") => "Verifica giochi, Wine e Proton",
        ("it", "menu.clean") => "Pulisci pacchetti, cache e percorsi",
        ("it", "menu.prefix") => "Gestisci prefissi Wine/Proton",
        ("it", "menu.defaults") => "Mostra percorsi predefiniti",
        ("it", "menu.system") => "Gestisci servizi, processi e journal",
        ("it", "menu.doctor") => "Diagnostica dipendenze ed esecuzione",
        ("it", "menu.packages") => "Inventaria pacchetti e archivi",
        ("it", "menu.help") => "Aiuto",
        ("it", "menu.quit") => "Esci",
        ("it", "menu.prompt") => "Scegli un’opzione (Invio per tornare indietro): ",
        ("it", "menu.back") => "Indietro",
        ("it", "menu.invalid") => "Opzione non valida.",
        ("it", "menu.system.title") => "=== Servizi, processi e journal ===",
        ("it", "menu.system.status") => "Stato di systemd e servizi falliti",
        ("it", "menu.system.services") => "Servizi di sistema",
        ("it", "menu.system.user_services") => "Servizi utente",
        ("it", "menu.system.processes") => "Processi per memoria",
        ("it", "menu.system.journal") => "Journal: avvisi di oggi",
        ("it", "menu.system.failed") => "Servizi falliti e journal correlato",
        ("it", "menu.system.manage") => "Gestisci un servizio",
        ("it", "menu.system.dependencies") => "Dipendenze e albero inverso",
        ("it", "menu.system.export") => "Esporta rapporto TSV/JSON",
        ("it", "menu.clean.title") => "=== Pulizia protetta ===",
        ("it", "menu.clean.orphans") => "Controlla i pacchetti orfani",
        ("it", "menu.clean.caches") => "Pulisci le cache dei gestori pacchetti",
        ("it", "menu.clean.flatpak") => "Controlla i runtime Flatpak inutilizzati",
        ("it", "menu.clean.path") => "Controlla un percorso specifico",
        ("it", "menu.clean.package") => "Controlla un pacchetto specifico",
        ("ca", "app.title") => "LTools",
        ("ca", "usage") => "Ús: ltools [ordre] [opcions]",
        ("ca", "commands") => "Ordres:",
        ("ca", "help.audit") => "Auditoria de discs, paquets i aplicacions",
        ("ca", "help.games") => "Auditoria de jocs, Wine i Proton",
        ("ca", "help.packages") => "Gestors de paquets, paquets i artefactes",
        ("ca", "help.clean") => "Neteja protegida de paquets, cau i rutes",
        ("ca", "help.prefix") => "Llistar, inspeccionar, crear i migrar prefixos",
        ("ca", "help.defaults") => "Mostrar els valors predeterminats efectius",
        ("ca", "help.system") => "systemd, serveis, processos i journal",
        ("ca", "help.doctor") => "Diagnòstic de dependències, FUSE i execució",
        ("ca", "help.rollback") => "Recuperar operacions reversibles d’un pla",
        ("ca", "help.capabilities") => "Imprimir el contracte d’integració JSON llegible per màquines",
        ("ca", "help.system.options") => "system: status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service ACCIÓ UNITAT | dependencies --unit UNITAT | export --format tsv|json --out FITXER",
        ("ca", "help.common") => "Opcions comunes: --dry-run, --plan FITXER, --lang IDIOMA, --help, --version",
        ("ca", "help.clean.options") => "clean: --package NOM --orphans --package-caches --flatpak-unused --path RUTA",
        ("ca", "help.prefix.options") => "prefix list: --root RUTA; prefix migrate: --source RUTA --dest RUTA",
        ("ca", "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        ("ca", "help.compat") => "El format TSV és compatible amb la implementació Bash.",
        ("ca", "menu.title") => "=== LTools ===",
        ("ca", "menu.audit") => "Auditar discs, paquets i aplicacions",
        ("ca", "menu.games") => "Auditar jocs, Wine i Proton",
        ("ca", "menu.clean") => "Netejar paquets, cau i rutes",
        ("ca", "menu.prefix") => "Gestionar prefixos Wine/Proton",
        ("ca", "menu.defaults") => "Mostrar rutes predeterminades",
        ("ca", "menu.system") => "Gestionar serveis, processos i journal",
        ("ca", "menu.doctor") => "Diagnòstic de dependències i execució",
        ("ca", "menu.packages") => "Inventari de paquets i botigues",
        ("ca", "menu.help") => "Ajuda",
        ("ca", "menu.quit") => "Sortir",
        ("ca", "menu.prompt") => "Tria una opció (Enter per tornar): ",
        ("ca", "menu.back") => "Tornar",
        ("ca", "menu.invalid") => "Opció no vàlida.",
        ("ca", "menu.system.title") => "=== Serveis, processos i journal ===",
        ("ca", "menu.system.status") => "Estat de systemd i serveis fallits",
        ("ca", "menu.system.services") => "Serveis del sistema",
        ("ca", "menu.system.user_services") => "Serveis de l’usuari",
        ("ca", "menu.system.processes") => "Processos per memòria",
        ("ca", "menu.system.journal") => "Journal: avisos d’avui",
        ("ca", "menu.system.failed") => "Serveis fallits i journal relacionat",
        ("ca", "menu.system.manage") => "Gestionar un servei",
        ("ca", "menu.system.dependencies") => "Dependències i arbre invers",
        ("ca", "menu.system.export") => "Exportar informe TSV/JSON",
        ("ca", "menu.clean.title") => "=== Neteja protegida ===",
        ("ca", "menu.clean.orphans") => "Revisar paquets orfes",
        ("ca", "menu.clean.caches") => "Netejar cau dels gestors de paquets",
        ("ca", "menu.clean.flatpak") => "Revisar runtimes Flatpak sense ús",
        ("ca", "menu.clean.path") => "Revisar una ruta concreta",
        ("ca", "menu.clean.package") => "Revisar un paquet concret",
        ("nl", "app.title") => "LTools",
        ("nl", "usage") => "Gebruik: ltools [opdracht] [opties]",
        ("nl", "commands") => "Opdrachten:",
        ("nl", "help.audit") => "Schijf-, pakket- en toepassingscontrole",
        ("nl", "help.games") => "Controle van games, Wine en Proton",
        ("nl", "help.packages") => "Pakketbeheerders, pakketten en artefacten",
        ("nl", "help.clean") => "Veilig opruimen van pakketten, caches en paden",
        ("nl", "help.prefix") => "Prefixes weergeven, inspecteren, maken en migreren",
        ("nl", "help.defaults") => "Effectieve standaardpaden tonen",
        ("nl", "help.system") => "systemd, diensten, processen en journal",
        ("nl", "help.doctor") => "Diagnose van afhankelijkheden, FUSE en runtime",
        ("nl", "help.rollback") => "Herstel omkeerbare bewerkingen uit een plan",
        ("nl", "help.capabilities") => "Het machineleesbare JSON-integratiecontract afdrukken",
        ("nl", "help.system.options") => "system: status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service ACTIE EENHEID | dependencies --unit EENHEID | export --format tsv|json --out BESTAND",
        ("nl", "help.common") => "Gemeenschappelijke opties: --dry-run, --plan BESTAND, --lang TAAL, --help, --version",
        ("nl", "help.clean.options") => "clean: --package NAAM --orphans --package-caches --flatpak-unused --path PAD",
        ("nl", "help.prefix.options") => "prefix list: --root PAD; prefix migrate: --source PAD --dest PAD",
        ("nl", "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        ("nl", "help.compat") => "TSV-rapporten zijn compatibel met de Bash-implementatie.",
        ("nl", "menu.title") => "=== LTools ===",
        ("nl", "menu.audit") => "Schijven, pakketten en toepassingen controleren",
        ("nl", "menu.games") => "Games, Wine en Proton controleren",
        ("nl", "menu.clean") => "Pakketten, caches en paden opruimen",
        ("nl", "menu.prefix") => "Wine-/Proton-prefixes beheren",
        ("nl", "menu.defaults") => "Standaardpaden tonen",
        ("nl", "menu.system") => "Diensten, processen en journal beheren",
        ("nl", "menu.doctor") => "Afhankelijkheden en runtime diagnosticeren",
        ("nl", "menu.packages") => "Pakketten en stores inventariseren",
        ("nl", "menu.help") => "Help",
        ("nl", "menu.quit") => "Afsluiten",
        ("nl", "menu.prompt") => "Kies een optie (Enter om terug te gaan): ",
        ("nl", "menu.back") => "Terug",
        ("nl", "menu.invalid") => "Ongeldige optie.",
        ("nl", "menu.system.title") => "=== Diensten, processen en journal ===",
        ("nl", "menu.system.status") => "systemd-status en mislukte diensten",
        ("nl", "menu.system.services") => "Systeemdiensten",
        ("nl", "menu.system.user_services") => "Gebruikersdiensten",
        ("nl", "menu.system.processes") => "Processen op geheugen",
        ("nl", "menu.system.journal") => "Journal: waarschuwingen van vandaag",
        ("nl", "menu.system.failed") => "Mislukte services en gerelateerd journal",
        ("nl", "menu.system.manage") => "Een service beheren",
        ("nl", "menu.system.dependencies") => "Serviceafhankelijkheden en omgekeerde boom",
        ("nl", "menu.system.export") => "TSV/JSON-rapport exporteren",
        ("nl", "menu.clean.title") => "=== Beschermde opruiming ===",
        ("nl", "menu.clean.orphans") => "Weespakketten controleren",
        ("nl", "menu.clean.caches") => "Caches van pakketbeheerders opruimen",
        ("nl", "menu.clean.flatpak") => "Ongebruikte Flatpak-runtimes controleren",
        ("nl", "menu.clean.path") => "Een specifiek pad controleren",
        ("nl", "menu.clean.package") => "Een specifiek pakket controleren",
        ("pl", "app.title") => "LTools",
        ("pl", "usage") => "Użycie: ltools [polecenie] [opcje]",
        ("pl", "commands") => "Polecenia:",
        ("pl", "help.audit") => "Audyt dysków, pakietów i aplikacji",
        ("pl", "help.games") => "Audyt gier, Wine i Proton",
        ("pl", "help.packages") => "Menedżery pakietów, pakiety i artefakty",
        ("pl", "help.clean") => "Bezpieczne czyszczenie pakietów, pamięci podręcznych i ścieżek",
        ("pl", "help.prefix") => "Wyświetlanie, inspekcja, tworzenie i migracja prefiksów",
        ("pl", "help.defaults") => "Pokaż aktywne ścieżki domyślne",
        ("pl", "help.system") => "systemd, usługi, procesy i dziennik",
        ("pl", "help.doctor") => "Diagnostyka zależności, FUSE i środowiska",
        ("pl", "help.rollback") => "Przywróć odwracalne operacje z planu",
        ("pl", "help.capabilities") => "Wyświetl czytelny maszynowo kontrakt integracji JSON",
        ("pl", "help.system.options") => "system: status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service AKCJA JEDNOSTKA | dependencies --unit JEDNOSTKA | export --format tsv|json --out PLIK",
        ("pl", "help.common") => "Opcje wspólne: --dry-run, --plan PLIK, --lang JĘZYK, --help, --version",
        ("pl", "help.clean.options") => "clean: --package NAZWA --orphans --package-caches --flatpak-unused --path ŚCIEŻKA",
        ("pl", "help.prefix.options") => "prefix list: --root ŚCIEŻKA; prefix migrate: --source ŚCIEŻKA --dest ŚCIEŻKA",
        ("pl", "help.prefix.flags") => "             --set-defaults --activate-shell --update-launchers --rewrite-configs",
        ("pl", "help.compat") => "Raporty TSV są zgodne z implementacją Bash.",
        ("pl", "menu.title") => "=== LTools ===",
        ("pl", "menu.audit") => "Audyt dysków, pakietów i aplikacji",
        ("pl", "menu.games") => "Audyt gier, Wine i Proton",
        ("pl", "menu.clean") => "Czyszczenie pakietów, pamięci podręcznych i ścieżek",
        ("pl", "menu.prefix") => "Zarządzanie prefiksami Wine/Proton",
        ("pl", "menu.defaults") => "Pokaż ścieżki domyślne",
        ("pl", "menu.system") => "Zarządzanie usługami, procesami i dziennikiem",
        ("pl", "menu.doctor") => "Diagnostyka zależności i środowiska",
        ("pl", "menu.packages") => "Inwentaryzacja pakietów i sklepów",
        ("pl", "menu.help") => "Pomoc",
        ("pl", "menu.quit") => "Wyjście",
        ("pl", "menu.prompt") => "Wybierz opcję (Enter, aby wrócić): ",
        ("pl", "menu.back") => "Wstecz",
        ("pl", "menu.invalid") => "Nieprawidłowa opcja.",
        ("pl", "menu.system.title") => "=== Usługi, procesy i dziennik ===",
        ("pl", "menu.system.status") => "Stan systemd i usługi zakończone błędem",
        ("pl", "menu.system.services") => "Usługi systemowe",
        ("pl", "menu.system.user_services") => "Usługi użytkownika",
        ("pl", "menu.system.processes") => "Procesy według pamięci",
        ("pl", "menu.system.journal") => "Dziennik: dzisiejsze ostrzeżenia",
        ("pl", "menu.system.failed") => "Usługi zakończone błędem i powiązany dziennik",
        ("pl", "menu.system.manage") => "Zarządzaj usługą",
        ("pl", "menu.system.dependencies") => "Zależności usługi i drzewo odwrotne",
        ("pl", "menu.system.export") => "Eksportuj raport TSV/JSON",
        ("pl", "menu.clean.title") => "=== Chronione czyszczenie ===",
        ("pl", "menu.clean.orphans") => "Sprawdź osierocone pakiety",
        ("pl", "menu.clean.caches") => "Wyczyść pamięci podręczne menedżerów pakietów",
        ("pl", "menu.clean.flatpak") => "Sprawdź nieużywane runtime Flatpak",
        ("pl", "menu.clean.path") => "Sprawdź konkretną ścieżkę",
        ("pl", "menu.clean.package") => "Sprawdź konkretny pakiet",
        (_, "app.title") => "LTools",
        (_, "usage") => "Uso: ltools [comando] [opciones]",
        (_, "commands") => "Comandos:",
        (_, "help.audit") => "Auditoría de discos, paquetes y aplicaciones",
        (_, "help.games") => "Auditoría de juegos, Wine y Proton",
        (_, "help.packages") => "Inventario de gestores, paquetes y artefactos",
        (_, "help.clean") => "Limpieza protegida de paquetes, cachés y rutas",
        (_, "help.prefix") => "Listar, inspeccionar, crear y migrar prefijos",
        (_, "help.defaults") => "Rutas efectivas de Wine, Proton y lanzadores",
        (_, "help.system") => "Salud de systemd, servicios, procesos, journal y acciones controladas",
        (_, "help.system.options") => "system: status | failed [--journal] | services --filter noteworthy|active|enabled|all | processes --sort cpu|memory --limit N | journal --level error|warning|info --hours N | service ACCIÓN UNIDAD | dependencies --unit UNIDAD | export --format tsv|json --out FICHERO",
        (_, "help.doctor") => "Diagnóstico de dependencias, FUSE y ejecución",
        (_, "help.rollback") => "Recuperar operaciones reversibles de un plan",
        (_, "help.capabilities") => "Imprimir el contrato JSON de integración para aplicaciones",
        (_, "help.common") => "Opciones comunes: --dry-run, --plan FICHERO, --lang IDIOMA, --help, --version",
        (_, "help.clean.options") => "clean: --package NOMBRE [--manager ...] --orphans --package-caches --flatpak-unused --path RUTA [--force]",
        (_, "help.prefix.options") => "prefix list: --root RUTA [--include-mount-roots]\nprefix migrate: --source RUTA --dest RUTA [--include NOMBRE] [--exclude NOMBRE]",
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
        (_, "menu.packages") => "Inventario de paquetes y almacenes",
        (_, "menu.help") => "Ayuda",
        (_, "menu.quit") => "Salir",
        (_, "menu.prompt") => "Elige una opción (Enter para volver): ",
        (_, "menu.back") => "Volver",
        (_, "menu.invalid") => "Opción no válida.",
        (_, "menu.system.title") => "=== Servicios, procesos y journal ===",
        (_, "menu.system.status") => "Estado de systemd y servicios fallidos",
        (_, "menu.system.services") => "Servicios del sistema",
        (_, "menu.system.user_services") => "Servicios del usuario",
        (_, "menu.system.processes") => "Procesos por memoria",
        (_, "menu.system.journal") => "Journal: avisos de hoy",
        (_, "menu.system.failed") => "Servicios fallidos y journal relacionado",
        (_, "menu.system.manage") => "Gestionar un servicio",
        (_, "menu.system.dependencies") => "Dependencias y árbol inverso de un servicio",
        (_, "menu.system.export") => "Exportar informe TSV/JSON",
        (_, "menu.clean.title") => "=== Limpieza protegida ===",
        (_, "menu.clean.orphans") => "Revisar paquetes huérfanos",
        (_, "menu.clean.caches") => "Limpiar cachés de gestores",
        (_, "menu.clean.flatpak") => "Revisar runtimes Flatpak sin uso",
        (_, "menu.clean.path") => "Revisar una ruta concreta",
        (_, "menu.clean.package") => "Revisar un paquete concreto",
        _ => "",
    }
}

/// Categorías del menú principal. Se mantienen separadas del catálogo de
/// comandos para que la CLI pueda cambiar su jerarquía sin cambiar el
/// contrato de automatización ni los nombres de las acciones.
pub fn category_text(key: &str) -> &'static str {
    match (current(), key) {
        ("es", "audit_inventory") => "Auditar / Inventariar",
        ("es", "storage") => "Gestión de discos",
        ("es", "services") => "Servicios / Dependencias",
        ("es", "defaults") => "Rutas predeterminadas",
        ("es", "automation") => "Automatización",
        ("es", "import") => "Importar scripts",
        ("es", "winslim") => "WinSlim",
        ("en", "audit_inventory") => "Audit / Inventory",
        ("en", "storage") => "Disk management",
        ("en", "services") => "Services / Dependencies",
        ("en", "defaults") => "Default paths",
        ("en", "automation") => "Automation",
        ("en", "import") => "Import scripts",
        ("en", "winslim") => "WinSlim",
        ("de", "audit_inventory") => "Prüfen / Inventarisieren",
        ("de", "storage") => "Datenträgerverwaltung",
        ("de", "services") => "Dienste / Abhängigkeiten",
        ("de", "defaults") => "Standardpfade",
        ("de", "automation") => "Automatisierung",
        ("de", "import") => "Skripte importieren",
        ("de", "winslim") => "WinSlim",
        ("fr", "audit_inventory") => "Auditer / Inventorier",
        ("fr", "storage") => "Gestion des disques",
        ("fr", "services") => "Services / Dépendances",
        ("fr", "defaults") => "Chemins par défaut",
        ("fr", "automation") => "Automatisation",
        ("fr", "import") => "Importer des scripts",
        ("fr", "winslim") => "WinSlim",
        ("pt", "audit_inventory") => "Auditar / Inventariar",
        ("pt", "storage") => "Gestão de discos",
        ("pt", "services") => "Serviços / Dependências",
        ("pt", "defaults") => "Caminhos predefinidos",
        ("pt", "automation") => "Automação",
        ("pt", "import") => "Importar scripts",
        ("pt", "winslim") => "WinSlim",
        ("it", "audit_inventory") => "Audit / Inventario",
        ("it", "storage") => "Gestione dischi",
        ("it", "services") => "Servizi / Dipendenze",
        ("it", "defaults") => "Percorsi predefiniti",
        ("it", "automation") => "Automazione",
        ("it", "import") => "Importa script",
        ("it", "winslim") => "WinSlim",
        ("ca", "audit_inventory") => "Auditar / Inventariar",
        ("ca", "storage") => "Gestió de discs",
        ("ca", "services") => "Serveis / Dependències",
        ("ca", "defaults") => "Rutes predeterminades",
        ("ca", "automation") => "Automatització",
        ("ca", "import") => "Importar scripts",
        ("ca", "winslim") => "WinSlim",
        ("nl", "audit_inventory") => "Auditeren / Inventariseren",
        ("nl", "storage") => "Schijfbeheer",
        ("nl", "services") => "Diensten / Afhankelijkheden",
        ("nl", "defaults") => "Standaardpaden",
        ("nl", "automation") => "Automatisering",
        ("nl", "import") => "Scripts importeren",
        ("nl", "winslim") => "WinSlim",
        ("pl", "audit_inventory") => "Audyt / Inwentaryzacja",
        ("pl", "storage") => "Zarządzanie dyskami",
        ("pl", "services") => "Usługi / Zależności",
        ("pl", "defaults") => "Ścieżki domyślne",
        ("pl", "automation") => "Automatyzacja",
        ("pl", "import") => "Import skryptów",
        ("pl", "winslim") => "WinSlim",
        ("en", "audits") => "Audits and inventories",
        ("en", "cleanup") => "Cleanup and storage",
        ("en", "applications") => "Applications and compatibility",
        ("en", "system") => "System and devices",
        ("en", "packages") => "Packages and Git",
        ("en", "diagnostics") => "Diagnostics and help",
        ("de", "audits") => "Prüfungen und Inventare",
        ("de", "cleanup") => "Bereinigung und Speicher",
        ("de", "applications") => "Anwendungen und Kompatibilität",
        ("de", "system") => "System und Geräte",
        ("de", "packages") => "Pakete und Git",
        ("de", "diagnostics") => "Diagnose und Hilfe",
        ("fr", "audits") => "Audits et inventaires",
        ("fr", "cleanup") => "Nettoyage et stockage",
        ("fr", "applications") => "Applications et compatibilité",
        ("fr", "system") => "Système et périphériques",
        ("fr", "packages") => "Paquets et Git",
        ("fr", "diagnostics") => "Diagnostic et aide",
        ("pt", "audits") => "Auditorias e inventários",
        ("pt", "cleanup") => "Limpeza e armazenamento",
        ("pt", "applications") => "Aplicações e compatibilidade",
        ("pt", "system") => "Sistema e dispositivos",
        ("pt", "packages") => "Pacotes e Git",
        ("pt", "diagnostics") => "Diagnóstico e ajuda",
        ("it", "audits") => "Audit e inventari",
        ("it", "cleanup") => "Pulizia e archiviazione",
        ("it", "applications") => "Applicazioni e compatibilità",
        ("it", "system") => "Sistema e dispositivi",
        ("it", "packages") => "Pacchetti e Git",
        ("it", "diagnostics") => "Diagnostica e aiuto",
        ("ca", "audits") => "Auditories i inventaris",
        ("ca", "cleanup") => "Neteja i emmagatzematge",
        ("ca", "applications") => "Aplicacions i compatibilitat",
        ("ca", "system") => "Sistema i dispositius",
        ("ca", "packages") => "Paquets i Git",
        ("ca", "diagnostics") => "Diagnòstic i ajuda",
        ("nl", "audits") => "Audits en inventarissen",
        ("nl", "cleanup") => "Opschonen en opslag",
        ("nl", "applications") => "Toepassingen en compatibiliteit",
        ("nl", "system") => "Systeem en apparaten",
        ("nl", "packages") => "Pakketten en Git",
        ("nl", "diagnostics") => "Diagnose en help",
        ("pl", "audits") => "Audyty i inwentaryzacja",
        ("pl", "cleanup") => "Czyszczenie i pamięć masowa",
        ("pl", "applications") => "Aplikacje i zgodność",
        ("pl", "system") => "System i urządzenia",
        ("pl", "packages") => "Pakiety i Git",
        ("pl", "diagnostics") => "Diagnostyka i pomoc",
        (_, "audits") => "Auditorías e inventarios",
        (_, "cleanup") => "Limpieza y almacenamiento",
        (_, "applications") => "Aplicaciones y compatibilidad",
        (_, "system") => "Sistema y dispositivos",
        (_, "packages") => "Paquetes y Git",
        (_, "diagnostics") => "Diagnóstico y ayuda",
        (_, _) => "",
    }
}

/// Textos del registro de automatizaciones. Las automatizaciones son datos
/// del usuario, no plugins con código embebido; este catálogo solo traduce la
/// navegación y sus mensajes básicos.
pub fn automation_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "title") => "=== Automation / imported scripts ===",
        ("en", "help") => "register and run user scripts without a shell",
        ("en", "menu") => "Imported scripts and automations",
        ("en", "list") => "List registered scripts",
        ("en", "list_title") => "Registered automations",
        ("en", "add") => "Register a script",
        ("en", "run") => "Run a registered script",
        ("en", "remove") => "Remove a registration",
        ("en", "name") => "Name",
        ("en", "program") => "Program or script path",
        ("en", "working_directory") => "Working directory",
        ("en", "arguments") => "Arguments",
        ("en", "current") => "current directory",
        ("en", "registry") => "Registry",
        ("en", "none") => "No scripts registered.",
        ("en", "saved") => "Automation registered.",
        ("en", "removed") => "Registration removed.",
        ("en", "command") => "Command",
        ("en", "winslim_ready") => "WinSlim integration surface detected at:",
        ("en", "winslim_placeholder") => "Reserved surface: WSCore actions are not executed yet.",
        ("en", "winslim_unavailable") => {
            "WinSlim integration is only available on Windows when C:\\WSCore exists."
        }
        ("de", "title") => "=== Automatisierung / importierte Skripte ===",
        ("de", "menu") => "Importierte Skripte und Automatisierungen",
        ("fr", "title") => "=== Automatisation / scripts importés ===",
        ("fr", "menu") => "Scripts importés et automatisations",
        ("pt", "title") => "=== Automação / scripts importados ===",
        ("pt", "menu") => "Scripts importados e automações",
        ("it", "title") => "=== Automazione / script importati ===",
        ("it", "menu") => "Script importati e automazioni",
        ("ca", "title") => "=== Automatització / scripts importats ===",
        ("ca", "menu") => "Scripts importats i automatitzacions",
        ("nl", "title") => "=== Automatisering / geïmporteerde scripts ===",
        ("nl", "menu") => "Geïmporteerde scripts en automatiseringen",
        ("pl", "title") => "=== Automatyzacja / importowane skrypty ===",
        ("pl", "menu") => "Importowane skrypty i automatyzacje",
        (_, "title") => "=== Automatización / scripts importados ===",
        (_, "help") => "registrar y ejecutar scripts del usuario sin shell",
        (_, "menu") => "Scripts importados y automatizaciones",
        (_, "list") => "Listar scripts registrados",
        (_, "list_title") => "Automatizaciones registradas",
        (_, "add") => "Registrar un script",
        (_, "run") => "Ejecutar un script registrado",
        (_, "remove") => "Eliminar un registro",
        (_, "name") => "Nombre",
        (_, "program") => "Ruta del programa o script",
        (_, "working_directory") => "Directorio de trabajo",
        (_, "arguments") => "Argumentos",
        (_, "current") => "directorio actual",
        (_, "registry") => "Registro",
        (_, "none") => "No hay scripts registrados.",
        (_, "saved") => "Automatización registrada.",
        (_, "removed") => "Registro eliminado.",
        (_, "command") => "Comando",
        (_, "winslim_ready") => "Superficie de integración WinSlim detectada en:",
        (_, "winslim_placeholder") => {
            "Superficie reservada: todavía no se ejecutan acciones de WSCore."
        }
        (_, "winslim_unavailable") => {
            "La integración WinSlim solo está disponible en Windows cuando existe C:\\WSCore."
        }
        (_, _) => "",
    }
}

#[cfg(test)]
mod tests {
    use super::{category_text, normalize, set, SUPPORTED};

    #[test]
    fn normalizes_language_variants() {
        assert_eq!(normalize("en_US.UTF-8"), "en");
        assert_eq!(normalize("pt-BR"), "pt");
        assert_eq!(normalize("unknown"), "es");
    }

    #[test]
    fn exposes_the_supported_catalog_languages() {
        assert_eq!(
            SUPPORTED,
            &["es", "en", "de", "fr", "pt", "it", "ca", "nl", "pl"]
        );
    }

    #[test]
    fn exposes_all_main_menu_categories_in_every_language() {
        for language in SUPPORTED {
            set(language);
            for category in [
                "audit_inventory",
                "storage",
                "services",
                "defaults",
                "automation",
                "import",
                "winslim",
                "audits",
                "cleanup",
                "applications",
                "system",
                "packages",
                "diagnostics",
            ] {
                assert!(!category_text(category).is_empty());
            }
        }
        set("es");
    }

    #[test]
    fn exposes_automation_navigation_text() {
        for language in SUPPORTED {
            set(language);
            for key in [
                "title",
                "menu",
                "list",
                "add",
                "run",
                "remove",
                "program",
                "arguments",
            ] {
                assert!(!super::automation_text(key).is_empty());
            }
        }
        set("es");
    }
}
