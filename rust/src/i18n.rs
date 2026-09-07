use std::env;

/// IDs de los 15 catálogos compartidos con LTerminal.
pub const SUPPORTED: &[&str] = &[
    "ar", "de", "en", "es", "fr", "hi", "it", "ja", "ko", "pl", "pt", "ro", "ru", "uk", "zh",
];

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

pub fn language_label(id: &str) -> &'static str {
    match id {
        "auto" => match current() {
            "en" => "Automatic (terminal)",
            "de" => "Automatisch (Terminal)",
            "fr" => "Automatique (terminal)",
            "pt" => "Automático (terminal)",
            "it" => "Automatico (terminale)",
            "pl" => "Automatycznie (terminal)",
            "ar" => "تلقائي (الطرفية)",
            "hi" => "स्वचालित (टर्मिनल)",
            "ja" => "自動（ターミナル）",
            "ko" => "자동 (터미널)",
            "ro" => "Automat (terminal)",
            "ru" => "Автоматически (терминал)",
            "uk" => "Автоматично (термінал)",
            "zh" => "自动（终端）",
            _ => "Automático (terminal)",
        },
        "en" => "English",
        "es" => "Español",
        "fr" => "Français",
        "de" => "Deutsch",
        "it" => "Italiano",
        "pt" => "Português",
        "ru" => "Русский",
        "zh" => "中文",
        "ja" => "日本語",
        "ko" => "한국어",
        "uk" => "Українська",
        "pl" => "Polski",
        "ro" => "Română",
        "ar" => "العربية",
        "hi" => "हिन्दी",
        _ => "Español",
    }
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
        .or_else(|| env::var("LTERMINAL_LANGUAGE").ok())
        .or_else(|| env::var("LTERMINAL_LANG").ok())
        .or_else(|| env::var("WINSLIM_TERMINAL_LANGUAGE").ok())
        .or_else(|| env::var("WINSLIM_TERMINAL_LANG").ok())
        .or_else(|| env::var("LC_ALL").ok())
        .or_else(|| env::var("LC_MESSAGES").ok())
        .or_else(|| env::var("LANG").ok())
        .unwrap_or_else(|| "es".into());
    normalize(&value)
}

/// Opciones de presentación comunes a CLI y hosts de terminal. Los valores
/// son deliberadamente legibles para que también puedan aparecer en la ayuda
/// de LTerminal o WinSlim Terminal.
pub fn visual_options() -> String {
    let themes = crate::theme::SUPPORTED.join(", ");
    match current() {
        "en" => format!("UI: --lang LANG, --theme THEME, --color auto|always|never, --no-color; themes: {themes}; JSON/TSV never include ANSI"),
        "de" => format!("UI: --lang SPRACHE, --theme THEMA, --color auto|always|never, --no-color; Themen: {themes}; JSON/TSV enthalten nie ANSI"),
        "fr" => format!("UI : --lang LANGUE, --theme THÈME, --color auto|always|never, --no-color ; thèmes : {themes} ; JSON/TSV sans ANSI"),
        "pt" => format!("UI: --lang IDIOMA, --theme TEMA, --color auto|always|never, --no-color; temas: {themes}; JSON/TSV nunca incluem ANSI"),
        "it" => format!("UI: --lang LINGUA, --theme TEMA, --color auto|always|never, --no-color; temi: {themes}; JSON/TSV senza ANSI"),
        "ca" => format!("UI: --lang IDIOMA, --theme TEMA, --color auto|always|never, --no-color; temes: {themes}; JSON/TSV sense ANSI"),
        "nl" => format!("UI: --lang TAAL, --theme THEMA, --color auto|always|never, --no-color; thema's: {themes}; JSON/TSV bevatten nooit ANSI"),
        "pl" => format!("UI: --lang JĘZYK, --theme MOTYW, --color auto|always|never, --no-color; motywy: {themes}; JSON/TSV bez ANSI"),
        _ => format!("Interfaz: --lang IDIOMA, --theme TEMA, --color auto|always|never, --no-color; temas: {themes}; JSON/TSV nunca incluyen ANSI"),
    }
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

pub fn diagnostics_label() -> &'static str {
    match current() {
        "en" => "Native host diagnostics",
        "de" => "Native Host-Diagnose",
        "fr" => "Diagnostic natif de l’hôte",
        "pt" => "Diagnóstico nativo do sistema",
        "it" => "Diagnostica nativa del sistema",
        "ca" => "Diagnòstic natiu del sistema",
        "nl" => "Native hostdiagnose",
        "pl" => "Natywna diagnostyka systemu",
        _ => "Diagnóstico nativo del sistema",
    }
}

pub fn diagnostics_help() -> &'static str {
    match current() {
        "en" => "Native read-only host diagnostics: health, network, hardware and users; --format human|tsv|json",
        "de" => "Native schreibgeschützte Host-Diagnose: Gesundheit, Netzwerk, Hardware und Benutzer; --format human|tsv|json",
        "fr" => "Diagnostic natif en lecture seule : santé, réseau, matériel et utilisateurs ; --format human|tsv|json",
        "pt" => "Diagnóstico nativo só de leitura: estado, rede, hardware e utilizadores; --format human|tsv|json",
        "it" => "Diagnostica nativa in sola lettura: stato, rete, hardware e utenti; --format human|tsv|json",
        "ca" => "Diagnòstic natiu només de lectura: salut, xarxa, maquinari i usuaris; --format human|tsv|json",
        "nl" => "Native alleen-lezen hostdiagnose: gezondheid, netwerk, hardware en gebruikers; --format human|tsv|json",
        "pl" => "Natywna diagnostyka tylko do odczytu: stan, sieć, sprzęt i użytkownicy; --format human|tsv|json",
        _ => "Diagnóstico nativo de solo lectura: salud, red, hardware y usuarios; --format human|tsv|json",
    }
}

pub fn diagnostics_available() -> &'static str {
    match current() {
        "en" => "available",
        "de" => "verfügbar",
        "fr" => "disponible",
        "pt" => "disponível",
        "it" => "disponibile",
        "ca" => "disponible",
        "nl" => "beschikbaar",
        "pl" => "dostępne",
        _ => "disponible",
    }
}

pub fn diagnostics_unavailable() -> &'static str {
    match current() {
        "en" => "unavailable",
        "de" => "nicht verfügbar",
        "fr" => "indisponible",
        "pt" => "indisponível",
        "it" => "non disponibile",
        "ca" => "no disponible",
        "nl" => "niet beschikbaar",
        "pl" => "niedostępne",
        _ => "no disponible",
    }
}

pub fn diagnostics_no_output() -> &'static str {
    match current() {
        "en" => "No output was returned.",
        "de" => "Keine Ausgabe erhalten.",
        "fr" => "Aucune sortie reçue.",
        "pt" => "Não foi obtida qualquer saída.",
        "it" => "Nessun output ricevuto.",
        "ca" => "No s’ha rebut cap sortida.",
        "nl" => "Geen uitvoer ontvangen.",
        "pl" => "Nie otrzymano żadnego wyjścia.",
        _ => "No se obtuvo salida.",
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

pub fn accounts_label() -> &'static str {
    match current() {
        "en" => "Users, groups and sessions",
        "de" => "Benutzer, Gruppen und Sitzungen",
        "fr" => "Utilisateurs, groupes et sessions",
        "pt" => "Utilizadores, grupos e sessões",
        "it" => "Utenti, gruppi e sessioni",
        "ca" => "Usuaris, grups i sessions",
        "nl" => "Gebruikers, groepen en sessies",
        "pl" => "Użytkownicy, grupy i sesje",
        _ => "Usuarios, grupos y sesiones",
    }
}

pub fn native_label() -> &'static str {
    match current() {
        "en" => "Native network, hardware, power and security tools",
        "de" => "Native Werkzeuge für Netzwerk, Hardware, Energie und Sicherheit",
        "fr" => "Outils natifs réseau, matériel, énergie et sécurité",
        "pt" => "Ferramentas nativas de rede, hardware, energia e segurança",
        "it" => "Strumenti nativi per rete, hardware, energia e sicurezza",
        "ca" => "Eines natives de xarxa, maquinari, energia i seguretat",
        "nl" => "Native netwerk-, hardware-, energie- en beveiligingshulpmiddelen",
        "pl" => "Natywne narzędzia sieci, sprzętu, energii i bezpieczeństwa",
        _ => "Herramientas nativas de red, hardware, energía y seguridad",
    }
}

pub fn boot_label() -> &'static str {
    match current() {
        "en" => "Boot, EFI and system loader",
        "de" => "Start, EFI und System-Bootloader",
        "fr" => "Démarrage, EFI et chargeur système",
        "pt" => "Arranque, EFI e carregador do sistema",
        "it" => "Avvio, EFI e bootloader di sistema",
        "pl" => "Rozruch, EFI i program ładujący systemu",
        "ar" => "الإقلاع وEFI ومحمل النظام",
        "hi" => "बूट, EFI और सिस्टम लोडर",
        "ja" => "起動、EFI、システムローダー",
        "ko" => "부팅, EFI 및 시스템 로더",
        "ro" => "Pornire, EFI și încărcătorul sistemului",
        "ru" => "Загрузка, EFI и системный загрузчик",
        "uk" => "Завантаження, EFI та системний завантажувач",
        "zh" => "启动、EFI 和系统加载程序",
        _ => "Arranque, EFI y cargador del sistema",
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

/// Acciones cortas del gestor de discos. Se usan tanto en la GUI como en
/// cualquier frontend que reutilice el catálogo de LTools.
pub fn storage_action_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "status") => "Space and mounts overview",
        ("en", "partitions") => "Disks and partitions",
        ("en", "mounts") => "Active mounts",
        ("en", "tools") => "Detected storage tools",
        ("en", "manager") => "Open native partition manager",
        ("en", "clean") => "Review cleanup",
        ("en", "guide") => "Partitioning guide and safety rules",
        ("de", "status") => "Übersicht über Speicher und Mounts",
        ("de", "partitions") => "Datenträger und Partitionen",
        ("de", "mounts") => "Aktive Mounts",
        ("de", "tools") => "Erkannte Speicherwerkzeuge",
        ("de", "manager") => "Nativen Partitionsmanager öffnen",
        ("de", "clean") => "Bereinigung prüfen",
        ("de", "guide") => "Partitionsleitfaden und Sicherheitsregeln",
        ("fr", "status") => "Vue d’ensemble de l’espace et des montages",
        ("fr", "partitions") => "Disques et partitions",
        ("fr", "mounts") => "Montages actifs",
        ("fr", "tools") => "Outils de stockage détectés",
        ("fr", "manager") => "Ouvrir le gestionnaire natif",
        ("fr", "clean") => "Vérifier le nettoyage",
        ("fr", "guide") => "Guide du partitionnement et règles de sécurité",
        ("pt", "status") => "Resumo de espaço e montagens",
        ("pt", "partitions") => "Discos e partições",
        ("pt", "mounts") => "Montagens ativas",
        ("pt", "tools") => "Ferramentas de armazenamento detetadas",
        ("pt", "manager") => "Abrir gestor nativo de partições",
        ("pt", "clean") => "Rever limpeza",
        ("pt", "guide") => "Guia de particionamento e regras de segurança",
        ("it", "status") => "Riepilogo spazio e mount",
        ("it", "partitions") => "Dischi e partizioni",
        ("it", "mounts") => "Mount attivi",
        ("it", "tools") => "Strumenti di archiviazione rilevati",
        ("it", "manager") => "Apri il gestore nativo delle partizioni",
        ("it", "clean") => "Controlla pulizia",
        ("it", "guide") => "Guida al partizionamento e regole di sicurezza",
        ("ca", "status") => "Resum d’espai i muntatges",
        ("ca", "partitions") => "Discs i particions",
        ("ca", "mounts") => "Muntatges actius",
        ("ca", "tools") => "Eines d’emmagatzematge detectades",
        ("ca", "manager") => "Obrir el gestor natiu de particions",
        ("ca", "clean") => "Revisar la neteja",
        ("ca", "guide") => "Guia de particions i regles de seguretat",
        ("nl", "status") => "Overzicht van ruimte en mounts",
        ("nl", "partitions") => "Schijven en partities",
        ("nl", "mounts") => "Actieve mounts",
        ("nl", "tools") => "Gedetecteerde opslagtools",
        ("nl", "manager") => "Native partitiebeheerder openen",
        ("nl", "clean") => "Opschoning controleren",
        ("nl", "guide") => "Handleiding voor partitioneren en veiligheidsregels",
        ("pl", "status") => "Przegląd miejsca i montowań",
        ("pl", "partitions") => "Dyski i partycje",
        ("pl", "mounts") => "Aktywne montowania",
        ("pl", "tools") => "Wykryte narzędzia pamięci masowej",
        ("pl", "manager") => "Otwórz natywny menedżer partycji",
        ("pl", "clean") => "Sprawdź czyszczenie",
        ("pl", "guide") => "Przewodnik partycjonowania i zasady bezpieczeństwa",
        ("ar", "status") => "ملخص المساحة ونقاط التحميل",
        ("ar", "partitions") => "الأقراص والأقسام",
        ("ar", "mounts") => "نقاط التحميل النشطة",
        ("ar", "tools") => "أدوات التخزين المكتشفة",
        ("ar", "manager") => "فتح مدير الأقسام الأصلي",
        ("ar", "clean") => "مراجعة التنظيف",
        ("ar", "guide") => "دليل التقسيم وقواعد الأمان",
        ("hi", "status") => "स्थान और माउंट का सारांश",
        ("hi", "partitions") => "डिस्क और पार्टीशन",
        ("hi", "mounts") => "सक्रिय माउंट",
        ("hi", "tools") => "पता चले स्टोरेज टूल",
        ("hi", "manager") => "मूल पार्टीशन प्रबंधक खोलें",
        ("hi", "clean") => "क्लीनअप की समीक्षा करें",
        ("hi", "guide") => "पार्टीशन गाइड और सुरक्षा नियम",
        ("ja", "status") => "容量とマウントの概要",
        ("ja", "partitions") => "ディスクとパーティション",
        ("ja", "mounts") => "アクティブなマウント",
        ("ja", "tools") => "検出されたストレージツール",
        ("ja", "manager") => "標準のパーティション管理ツールを開く",
        ("ja", "clean") => "クリーンアップを確認",
        ("ja", "guide") => "パーティションガイドと安全規則",
        ("ko", "status") => "공간 및 마운트 요약",
        ("ko", "partitions") => "디스크 및 파티션",
        ("ko", "mounts") => "활성 마운트",
        ("ko", "tools") => "감지된 저장소 도구",
        ("ko", "manager") => "기본 파티션 관리자 열기",
        ("ko", "clean") => "정리 검토",
        ("ko", "guide") => "파티션 안내 및 안전 규칙",
        ("ro", "status") => "Rezumat spațiu și montări",
        ("ro", "partitions") => "Discuri și partiții",
        ("ro", "mounts") => "Montări active",
        ("ro", "tools") => "Instrumente de stocare detectate",
        ("ro", "manager") => "Deschide managerul nativ de partiții",
        ("ro", "clean") => "Verifică curățarea",
        ("ro", "guide") => "Ghid de partiționare și reguli de siguranță",
        ("ru", "status") => "Обзор места и подключений",
        ("ru", "partitions") => "Диски и разделы",
        ("ru", "mounts") => "Активные подключения",
        ("ru", "tools") => "Найденные инструменты хранения",
        ("ru", "manager") => "Открыть штатный менеджер разделов",
        ("ru", "clean") => "Проверить очистку",
        ("ru", "guide") => "Руководство по разделам и правила безопасности",
        ("uk", "status") => "Огляд місця та монтувань",
        ("uk", "partitions") => "Диски та розділи",
        ("uk", "mounts") => "Активні монтування",
        ("uk", "tools") => "Виявлені інструменти сховища",
        ("uk", "manager") => "Відкрити штатний менеджер розділів",
        ("uk", "clean") => "Перевірити очищення",
        ("uk", "guide") => "Посібник із розділів і правила безпеки",
        ("zh", "status") => "空间和挂载概览",
        ("zh", "partitions") => "磁盘和分区",
        ("zh", "mounts") => "活动挂载",
        ("zh", "tools") => "检测到的存储工具",
        ("zh", "manager") => "打开原生分区管理器",
        ("zh", "clean") => "检查清理",
        ("zh", "guide") => "分区指南和安全规则",
        (_, "status") => "Resumen de espacio y montajes",
        (_, "partitions") => "Discos y particiones",
        (_, "mounts") => "Montajes activos",
        (_, "tools") => "Herramientas detectadas",
        (_, "manager") => "Abrir gestor nativo de particiones",
        (_, "clean") => "Revisar limpieza",
        (_, "guide") => "Guía de particionado y protecciones",
        _ => "",
    }
}

pub fn native_tools_label() -> &'static str {
    match current() {
        "en" => "SSH, Android, Docker and Kubernetes tools",
        "de" => "SSH-, Android-, Docker- und Kubernetes-Werkzeuge",
        "fr" => "Outils SSH, Android, Docker et Kubernetes",
        "pt" => "Ferramentas SSH, Android, Docker e Kubernetes",
        "it" => "Strumenti SSH, Android, Docker e Kubernetes",
        "ca" => "Eines SSH, Android, Docker i Kubernetes",
        "nl" => "SSH-, Android-, Docker- en Kubernetes-tools",
        "pl" => "Narzędzia SSH, Android, Docker i Kubernetes",
        "ar" => "أدوات SSH وAndroid وDocker وKubernetes",
        "hi" => "SSH, Android, Docker और Kubernetes टूल",
        "ja" => "SSH、Android、Docker、Kubernetes ツール",
        "ko" => "SSH, Android, Docker 및 Kubernetes 도구",
        "ro" => "Instrumente SSH, Android, Docker și Kubernetes",
        "ru" => "Инструменты SSH, Android, Docker и Kubernetes",
        "uk" => "Інструменти SSH, Android, Docker і Kubernetes",
        "zh" => "SSH、Android、Docker 和 Kubernetes 工具",
        _ => "Herramientas SSH, Android, Docker y Kubernetes",
    }
}

pub fn native_action_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "network_status") => "Network, routes, DNS and listening ports",
        ("en", "dns_flush") => "Flush DNS cache",
        ("en", "tools_status") => "Dependencies and versions",
        ("en", "tools_install") => "Install a dependency",
        ("en", "adb_devices") => "ADB: connected devices",
        ("en", "container_list") => "Docker/Podman: containers",
        ("en", "kubernetes_contexts") => "Kubernetes: contexts",
        ("de", "network_status") => "Netzwerk, Routen, DNS und offene Ports",
        ("de", "dns_flush") => "DNS-Cache leeren",
        ("de", "tools_status") => "Abhängigkeiten und Versionen",
        ("de", "tools_install") => "Abhängigkeit installieren",
        ("de", "adb_devices") => "ADB: verbundene Geräte",
        ("de", "container_list") => "Docker/Podman: Container",
        ("de", "kubernetes_contexts") => "Kubernetes: Kontexte",
        ("fr", "network_status") => "Réseau, routes, DNS et ports en écoute",
        ("fr", "dns_flush") => "Vider le cache DNS",
        ("fr", "tools_status") => "Dépendances et versions",
        ("fr", "tools_install") => "Installer une dépendance",
        ("fr", "adb_devices") => "ADB : appareils connectés",
        ("fr", "container_list") => "Docker/Podman : conteneurs",
        ("fr", "kubernetes_contexts") => "Kubernetes : contextes",
        ("pt", "network_status") => "Rede, rotas, DNS e portas de escuta",
        ("pt", "dns_flush") => "Limpar a cache DNS",
        ("pt", "tools_status") => "Dependências e versões",
        ("pt", "tools_install") => "Instalar uma dependência",
        ("pt", "adb_devices") => "ADB: dispositivos ligados",
        ("pt", "container_list") => "Docker/Podman: contentores",
        ("pt", "kubernetes_contexts") => "Kubernetes: contextos",
        ("it", "network_status") => "Rete, rotte, DNS e porte in ascolto",
        ("it", "dns_flush") => "Svuota cache DNS",
        ("it", "tools_status") => "Dipendenze e versioni",
        ("it", "tools_install") => "Installa una dipendenza",
        ("it", "adb_devices") => "ADB: dispositivi collegati",
        ("it", "container_list") => "Docker/Podman: container",
        ("it", "kubernetes_contexts") => "Kubernetes: contesti",
        ("ca", "network_status") => "Xarxa, rutes, DNS i ports en escolta",
        ("ca", "dns_flush") => "Buida la memòria cau DNS",
        ("ca", "tools_status") => "Dependències i versions",
        ("ca", "tools_install") => "Instal·lar una dependència",
        ("ca", "adb_devices") => "ADB: dispositius connectats",
        ("ca", "container_list") => "Docker/Podman: contenidors",
        ("ca", "kubernetes_contexts") => "Kubernetes: contextos",
        ("nl", "network_status") => "Netwerk, routes, DNS en luisterpoorten",
        ("nl", "dns_flush") => "DNS-cache wissen",
        ("nl", "tools_status") => "Afhankelijkheden en versies",
        ("nl", "tools_install") => "Afhankelijkheid installeren",
        ("nl", "adb_devices") => "ADB: verbonden apparaten",
        ("nl", "container_list") => "Docker/Podman: containers",
        ("nl", "kubernetes_contexts") => "Kubernetes: contexten",
        ("pl", "network_status") => "Sieć, trasy, DNS i nasłuchujące porty",
        ("pl", "dns_flush") => "Wyczyść pamięć DNS",
        ("pl", "tools_status") => "Zależności i wersje",
        ("pl", "tools_install") => "Zainstaluj zależność",
        ("pl", "adb_devices") => "ADB: podłączone urządzenia",
        ("pl", "container_list") => "Docker/Podman: kontenery",
        ("pl", "kubernetes_contexts") => "Kubernetes: konteksty",
        ("ar", "network_status") => "الشبكة والمسارات وDNS والمنافذ المستمعة",
        ("ar", "dns_flush") => "مسح ذاكرة DNS",
        ("ar", "tools_status") => "التبعيات والإصدارات",
        ("ar", "tools_install") => "تثبيت تبعية",
        ("ar", "adb_devices") => "ADB: الأجهزة المتصلة",
        ("ar", "container_list") => "Docker/Podman: الحاويات",
        ("ar", "kubernetes_contexts") => "Kubernetes: السياقات",
        ("hi", "network_status") => "नेटवर्क, रूट, DNS और सुनने वाले पोर्ट",
        ("hi", "dns_flush") => "DNS कैश साफ़ करें",
        ("hi", "tools_status") => "निर्भरताएँ और संस्करण",
        ("hi", "tools_install") => "निर्भरता स्थापित करें",
        ("hi", "adb_devices") => "ADB: जुड़े उपकरण",
        ("hi", "container_list") => "Docker/Podman: कंटेनर",
        ("hi", "kubernetes_contexts") => "Kubernetes: संदर्भ",
        ("ja", "network_status") => "ネットワーク、ルート、DNS、待受ポート",
        ("ja", "dns_flush") => "DNS キャッシュを消去",
        ("ja", "tools_status") => "依存関係とバージョン",
        ("ja", "tools_install") => "依存関係をインストール",
        ("ja", "adb_devices") => "ADB: 接続デバイス",
        ("ja", "container_list") => "Docker/Podman: コンテナ",
        ("ja", "kubernetes_contexts") => "Kubernetes: コンテキスト",
        ("ko", "network_status") => "네트워크, 경로, DNS 및 수신 포트",
        ("ko", "dns_flush") => "DNS 캐시 비우기",
        ("ko", "tools_status") => "종속성 및 버전",
        ("ko", "tools_install") => "종속성 설치",
        ("ko", "adb_devices") => "ADB: 연결된 장치",
        ("ko", "container_list") => "Docker/Podman: 컨테이너",
        ("ko", "kubernetes_contexts") => "Kubernetes: 컨텍스트",
        ("ro", "network_status") => "Rețea, rute, DNS și porturi de ascultare",
        ("ro", "dns_flush") => "Golește memoria cache DNS",
        ("ro", "tools_status") => "Dependențe și versiuni",
        ("ro", "tools_install") => "Instalează o dependență",
        ("ro", "adb_devices") => "ADB: dispozitive conectate",
        ("ro", "container_list") => "Docker/Podman: containere",
        ("ro", "kubernetes_contexts") => "Kubernetes: contexte",
        ("ru", "network_status") => "Сеть, маршруты, DNS и прослушиваемые порты",
        ("ru", "dns_flush") => "Очистить кэш DNS",
        ("ru", "tools_status") => "Зависимости и версии",
        ("ru", "tools_install") => "Установить зависимость",
        ("ru", "adb_devices") => "ADB: подключённые устройства",
        ("ru", "container_list") => "Docker/Podman: контейнеры",
        ("ru", "kubernetes_contexts") => "Kubernetes: контексты",
        ("uk", "network_status") => "Мережа, маршрути, DNS і порти прослуховування",
        ("uk", "dns_flush") => "Очистити кеш DNS",
        ("uk", "tools_status") => "Залежності та версії",
        ("uk", "tools_install") => "Встановити залежність",
        ("uk", "adb_devices") => "ADB: підключені пристрої",
        ("uk", "container_list") => "Docker/Podman: контейнери",
        ("uk", "kubernetes_contexts") => "Kubernetes: контексти",
        ("zh", "network_status") => "网络、路由、DNS 和监听端口",
        ("zh", "dns_flush") => "清除 DNS 缓存",
        ("zh", "tools_status") => "依赖项和版本",
        ("zh", "tools_install") => "安装依赖项",
        ("zh", "adb_devices") => "ADB：已连接设备",
        ("zh", "container_list") => "Docker/Podman：容器",
        ("zh", "kubernetes_contexts") => "Kubernetes：上下文",
        (_, "network_status") => "Red, rutas, DNS y puertos escuchando",
        (_, "dns_flush") => "Vaciar caché DNS",
        (_, "tools_status") => "Dependencias y versiones",
        (_, "tools_install") => "Instalar una dependencia",
        (_, "adb_devices") => "ADB: dispositivos conectados",
        (_, "container_list") => "Docker/Podman: contenedores",
        (_, "kubernetes_contexts") => "Kubernetes: contextos",
        _ => "",
    }
}

/// Textos del registro de acciones. Los identificadores de acción son
/// estables y no se traducen; estas etiquetas sí pueden mostrarse en botones
/// y menús de cualquier frontend.
pub fn actions_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "title") => "=== Guided system actions ===",
        ("en", "help") => "Reusable safe-default actions for CLI, GUI and terminal hosts",
        ("en", "menu") => "Guided actions",
        ("en", "list") => "Show action details",
        ("en", "hint") => "Run an ID with: actions run ID [TARGET]. Empty target input returns.",
        ("de", "title") => "=== Geführte Systemaktionen ===",
        ("de", "help") => "Wiederverwendbare sichere Aktionen für CLI, GUI und Terminalhosts",
        ("de", "menu") => "Geführte Aktionen",
        ("de", "list") => "Aktionsdetails anzeigen",
        ("de", "hint") => "ID ausführen mit: actions run ID [ZIEL]. Leeres Ziel geht zurück.",
        ("fr", "title") => "=== Actions système guidées ===",
        ("fr", "help") => "Actions sûres réutilisables pour CLI, GUI et terminaux",
        ("fr", "menu") => "Actions guidées",
        ("fr", "list") => "Afficher les détails des actions",
        ("fr", "hint") => "Exécuter un ID avec : actions run ID [CIBLE]. Entrée vide pour revenir.",
        ("pt", "title") => "=== Ações guiadas do sistema ===",
        ("pt", "help") => "Ações seguras reutilizáveis para CLI, GUI e terminais",
        ("pt", "menu") => "Ações guiadas",
        ("pt", "list") => "Mostrar detalhes das ações",
        ("pt", "hint") => "Executar um ID com: actions run ID [ALVO]. Enter vazio para voltar.",
        ("it", "title") => "=== Azioni di sistema guidate ===",
        ("it", "help") => "Azioni sicure riutilizzabili per CLI, GUI e terminali",
        ("it", "menu") => "Azioni guidate",
        ("it", "list") => "Mostra dettagli azioni",
        ("it", "hint") => "Esegui un ID con: actions run ID [OBIETTIVO]. Invio vuoto per tornare.",
        ("ca", "title") => "=== Accions guiades del sistema ===",
        ("ca", "help") => "Accions segures reutilitzables per a CLI, GUI i terminals",
        ("ca", "menu") => "Accions guiades",
        ("ca", "list") => "Mostrar detalls de les accions",
        ("ca", "hint") => "Executa un ID amb: actions run ID [OBJECTIU]. Enter buit per tornar.",
        ("nl", "title") => "=== Begeleide systeemacties ===",
        ("nl", "help") => "Herbruikbare veilige acties voor CLI, GUI en terminalhosts",
        ("nl", "menu") => "Begeleide acties",
        ("nl", "list") => "Actiedetails tonen",
        ("nl", "hint") => "Voer een ID uit met: actions run ID [DOEL]. Leeg doel gaat terug.",
        ("pl", "title") => "=== Prowadzone działania systemu ===",
        ("pl", "help") => "Bezpieczne działania wielokrotnego użytku dla CLI, GUI i terminali",
        ("pl", "menu") => "Prowadzone działania",
        ("pl", "list") => "Pokaż szczegóły działania",
        ("pl", "hint") => "Uruchom ID przez: actions run ID [CEL]. Puste pole wraca.",
        (_, "title") => "=== Acciones guiadas del sistema ===",
        (_, "help") => "Acciones seguras reutilizables para CLI, GUI y terminales",
        (_, "menu") => "Acciones guiadas",
        (_, "list") => "Mostrar detalles de las acciones",
        (_, "hint") => "Ejecuta un ID con: actions run ID [OBJETIVO]. Enter vacío para volver.",
        _ => "",
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
    if matches!(
        key,
        "theme_button"
            | "language_button"
            | "settings_button"
            | "settings_title"
            | "settings_theme"
            | "settings_language"
            | "settings_visibility"
            | "settings_restart"
            | "visible"
            | "hidden"
    ) {
        return match (current(), key) {
            ("en", "theme_button") => "Theme",
            ("en", "language_button") => "Language",
            ("en", "settings_button") => "Settings",
            ("en", "settings_title") => "LTools settings",
            ("en", "settings_theme") => "Themes",
            ("en", "settings_language") => "Languages",
            ("en", "settings_visibility") => "Menu and submenu visibility",
            ("en", "settings_restart") => {
                "Language and visibility changes apply after reopening LTools."
            }
            ("en", "visible") => "Visible",
            ("en", "hidden") => "Hidden",
            ("de", "theme_button") => "Thema",
            ("de", "language_button") => "Sprache",
            ("de", "settings_button") => "Einstellungen",
            ("de", "settings_title") => "LTools-Einstellungen",
            ("de", "settings_theme") => "Themen",
            ("de", "settings_language") => "Sprachen",
            ("de", "settings_visibility") => "Sichtbarkeit von Menüs und Untermenüs",
            ("de", "settings_restart") => {
                "Sprach- und Sichtbarkeitsänderungen gelten nach dem Neustart."
            }
            ("de", "visible") => "Sichtbar",
            ("de", "hidden") => "Ausgeblendet",
            ("fr", "theme_button") => "Thème",
            ("fr", "language_button") => "Langue",
            ("fr", "settings_button") => "Réglages",
            ("fr", "settings_title") => "Réglages de LTools",
            ("fr", "settings_theme") => "Thèmes",
            ("fr", "settings_language") => "Langues",
            ("fr", "settings_visibility") => "Visibilité des menus et sous-menus",
            ("fr", "settings_restart") => {
                "Les changements de langue et de visibilité s’appliquent au prochain démarrage."
            }
            ("fr", "visible") => "Visible",
            ("fr", "hidden") => "Masqué",
            ("pt", "theme_button") => "Tema",
            ("pt", "language_button") => "Idioma",
            ("pt", "settings_button") => "Definições",
            ("pt", "settings_title") => "Definições do LTools",
            ("pt", "settings_theme") => "Temas",
            ("pt", "settings_language") => "Idiomas",
            ("pt", "settings_visibility") => "Visibilidade dos menus e submenus",
            ("pt", "settings_restart") => {
                "As alterações de idioma e visibilidade aplicam-se ao reabrir o LTools."
            }
            ("pt", "visible") => "Visível",
            ("pt", "hidden") => "Oculto",
            ("it", "theme_button") => "Tema",
            ("it", "language_button") => "Lingua",
            ("it", "settings_button") => "Impostazioni",
            ("it", "settings_title") => "Impostazioni di LTools",
            ("it", "settings_theme") => "Temi",
            ("it", "settings_language") => "Lingue",
            ("it", "settings_visibility") => "Visibilità di menu e sottomenu",
            ("it", "settings_restart") => {
                "Le modifiche a lingua e visibilità si applicano riaprendo LTools."
            }
            ("it", "visible") => "Visibile",
            ("it", "hidden") => "Nascosto",
            ("pl", "theme_button") => "Motyw",
            ("pl", "language_button") => "Język",
            ("pl", "settings_button") => "Ustawienia",
            ("pl", "settings_title") => "Ustawienia LTools",
            ("pl", "settings_theme") => "Motywy",
            ("pl", "settings_language") => "Języki",
            ("pl", "settings_visibility") => "Widoczność menu i podmenu",
            ("pl", "settings_restart") => {
                "Zmiany języka i widoczności obowiązują po ponownym otwarciu LTools."
            }
            ("pl", "visible") => "Widoczne",
            ("pl", "hidden") => "Ukryte",
            ("ar", "theme_button") => "السمة",
            ("ar", "language_button") => "اللغة",
            ("ar", "settings_button") => "الإعدادات",
            ("ar", "settings_title") => "إعدادات LTools",
            ("ar", "settings_theme") => "السمات",
            ("ar", "settings_language") => "اللغات",
            ("ar", "settings_visibility") => "ظهور القوائم والقوائم الفرعية",
            ("ar", "settings_restart") => "تُطبَّق تغييرات اللغة والظهور عند إعادة فتح LTools.",
            ("ar", "visible") => "ظاهر",
            ("ar", "hidden") => "مخفي",
            ("hi", "theme_button") => "थीम",
            ("hi", "language_button") => "भाषा",
            ("hi", "settings_button") => "सेटिंग्स",
            ("hi", "settings_title") => "LTools सेटिंग्स",
            ("hi", "settings_theme") => "थीम",
            ("hi", "settings_language") => "भाषाएँ",
            ("hi", "settings_visibility") => "मेनू और सबमेनू दृश्यता",
            ("hi", "settings_restart") => "भाषा और दृश्यता बदलाव LTools फिर खोलने पर लागू होंगे।",
            ("hi", "visible") => "दृश्य",
            ("hi", "hidden") => "छिपा हुआ",
            ("ja", "theme_button") => "テーマ",
            ("ja", "language_button") => "言語",
            ("ja", "settings_button") => "設定",
            ("ja", "settings_title") => "LTools の設定",
            ("ja", "settings_theme") => "テーマ",
            ("ja", "settings_language") => "言語",
            ("ja", "settings_visibility") => "メニューとサブメニューの表示",
            ("ja", "settings_restart") => "言語と表示の変更は LTools を再起動すると適用されます。",
            ("ja", "visible") => "表示",
            ("ja", "hidden") => "非表示",
            ("ko", "theme_button") => "테마",
            ("ko", "language_button") => "언어",
            ("ko", "settings_button") => "설정",
            ("ko", "settings_title") => "LTools 설정",
            ("ko", "settings_theme") => "테마",
            ("ko", "settings_language") => "언어",
            ("ko", "settings_visibility") => "메뉴 및 하위 메뉴 표시",
            ("ko", "settings_restart") => "언어와 표시 변경은 LTools를 다시 열 때 적용됩니다.",
            ("ko", "visible") => "표시",
            ("ko", "hidden") => "숨김",
            ("ro", "theme_button") => "Temă",
            ("ro", "language_button") => "Limbă",
            ("ro", "settings_button") => "Setări",
            ("ro", "settings_title") => "Setările LTools",
            ("ro", "settings_theme") => "Teme",
            ("ro", "settings_language") => "Limbi",
            ("ro", "settings_visibility") => "Vizibilitatea meniurilor și submeniurilor",
            ("ro", "settings_restart") => {
                "Schimbările de limbă și vizibilitate se aplică după redeschiderea LTools."
            }
            ("ro", "visible") => "Vizibil",
            ("ro", "hidden") => "Ascuns",
            ("ru", "theme_button") => "Тема",
            ("ru", "language_button") => "Язык",
            ("ru", "settings_button") => "Настройки",
            ("ru", "settings_title") => "Настройки LTools",
            ("ru", "settings_theme") => "Темы",
            ("ru", "settings_language") => "Языки",
            ("ru", "settings_visibility") => "Видимость меню и подменю",
            ("ru", "settings_restart") => {
                "Изменения языка и видимости применяются после открытия LTools."
            }
            ("ru", "visible") => "Видимо",
            ("ru", "hidden") => "Скрыто",
            ("uk", "theme_button") => "Тема",
            ("uk", "language_button") => "Мова",
            ("uk", "settings_button") => "Налаштування",
            ("uk", "settings_title") => "Налаштування LTools",
            ("uk", "settings_theme") => "Теми",
            ("uk", "settings_language") => "Мови",
            ("uk", "settings_visibility") => "Видимість меню та підменю",
            ("uk", "settings_restart") => {
                "Зміни мови та видимості застосовуються після повторного відкриття LTools."
            }
            ("uk", "visible") => "Видимо",
            ("uk", "hidden") => "Приховано",
            ("zh", "theme_button") => "主题",
            ("zh", "language_button") => "语言",
            ("zh", "settings_button") => "设置",
            ("zh", "settings_title") => "LTools 设置",
            ("zh", "settings_theme") => "主题",
            ("zh", "settings_language") => "语言",
            ("zh", "settings_visibility") => "菜单和子菜单可见性",
            ("zh", "settings_restart") => "重新打开 LTools 后，语言和可见性更改才会生效。",
            ("zh", "visible") => "显示",
            ("zh", "hidden") => "隐藏",
            (_, "theme_button") => "Tema",
            (_, "language_button") => "Idioma",
            (_, "settings_button") => "Ajustes",
            (_, "settings_title") => "Ajustes de LTools",
            (_, "settings_theme") => "Temas",
            (_, "settings_language") => "Idiomas",
            (_, "settings_visibility") => "Visibilidad de menús y submenús",
            (_, "settings_restart") => {
                "El idioma y la visibilidad se aplican al volver a abrir LTools."
            }
            (_, "visible") => "Visible",
            (_, "hidden") => "Oculto",
            (_, _) => "",
        };
    }
    if key == "diagnostics" {
        return diagnostics_label();
    }
    if key == "native" {
        return native_label();
    }
    if key == "native_tools" {
        return native_tools_label();
    }
    if key == "containers" {
        return match current() {
            "en" => "Docker and Podman container status",
            "de" => "Docker- und Podman-Containerstatus",
            "fr" => "État des conteneurs Docker et Podman",
            "pt" => "Estado dos contentores Docker e Podman",
            "it" => "Stato dei container Docker e Podman",
            "ca" => "Estat dels contenidors Docker i Podman",
            "nl" => "Status van Docker- en Podman-containers",
            "pl" => "Stan kontenerów Docker i Podman",
            "ar" => "حالة حاويات Docker وPodman",
            "hi" => "Docker और Podman कंटेनर स्थिति",
            "ja" => "Docker と Podman コンテナの状態",
            "ko" => "Docker 및 Podman 컨테이너 상태",
            "ro" => "Starea containerelor Docker și Podman",
            "ru" => "Состояние контейнеров Docker и Podman",
            "uk" => "Стан контейнерів Docker і Podman",
            "zh" => "Docker 和 Podman 容器状态",
            _ => "Estado de contenedores Docker y Podman",
        };
    }
    if key == "kubernetes" {
        return match current() {
            "en" => "Kubernetes cluster status",
            "de" => "Kubernetes-Clusterstatus",
            "fr" => "État du cluster Kubernetes",
            "pt" => "Estado do cluster Kubernetes",
            "it" => "Stato del cluster Kubernetes",
            "ca" => "Estat del clúster Kubernetes",
            "nl" => "Kubernetes-clusterstatus",
            "pl" => "Stan klastra Kubernetes",
            "ar" => "حالة عنقود Kubernetes",
            "hi" => "Kubernetes क्लस्टर स्थिति",
            "ja" => "Kubernetes クラスターの状態",
            "ko" => "Kubernetes 클러스터 상태",
            "ro" => "Starea clusterului Kubernetes",
            "ru" => "Состояние кластера Kubernetes",
            "uk" => "Стан кластера Kubernetes",
            "zh" => "Kubernetes 集群状态",
            _ => "Estado del clúster Kubernetes",
        };
    }
    if key == "storage_guide" {
        return storage_action_text("guide");
    }
    if key == "accounts" {
        return accounts_label();
    }
    if key == "boot" {
        return boot_label();
    }
    if key == "registry" {
        return registry_label();
    }
    if key == "system_services" {
        return text("menu.system.services");
    }
    if key == "system_processes" {
        return text("menu.system.processes");
    }
    if key == "system_journal" {
        return text("menu.system.journal");
    }
    match (current(), key) {
        ("en", "title") => "LTools",
        ("en", "subtitle") => "Safe system tools and quick actions",
        ("en", "ready") => "Ready",
        ("en", "sections") => "Sections",
        ("en", "dashboard_hint") => "Choose a section to work with its tools and actions.",
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
        ("en", "confirm_storage_manager") => "Open the native storage manager? It can modify partitions and data.",
        ("en", "confirm_git_operation") => "This Git/GitHub operation can change the repository or remote state. Continue?",
        ("en", "cancelled") => "Cancelled",
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
        ("de", "confirm_storage_manager") => "Den nativen Datenträgerverwalter öffnen? Er kann Partitionen und Daten ändern.",
        ("de", "cancelled") => "Abgebrochen",
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
        ("fr", "confirm_storage_manager") => "Ouvrir le gestionnaire de stockage natif ? Il peut modifier les partitions et les données.",
        ("fr", "cancelled") => "Annulé",
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
        ("pt", "confirm_storage_manager") => "Abrir o gestor de armazenamento nativo? Pode alterar partições e dados.",
        ("pt", "cancelled") => "Cancelado",
        ("it", "title") => "LTools",
        ("it", "subtitle") => "Strumenti di sistema sicuri e azioni rapide",
        ("it", "ready") => "Pronto",
        ("it", "running") => "In esecuzione…",
        ("it", "completed") => "Completato",
        ("it", "audit") => "Controlla dischi e applicazioni",
        ("it", "games") => "Inventario di giochi e launcher",
        ("it", "packages") => "Inventario pacchetti",
        ("it", "prefixes") => "Prefissi Wine/Proton",
        ("it", "defaults") => "Percorsi predefiniti",
        ("it", "system") => "Stato del sistema",
        ("it", "doctor") => "Dipendenze e diagnostica",
        ("it", "storage") => "Dischi e partizioni",
        ("it", "stores") => "Repository dei pacchetti",
        ("it", "git") => "Stato di Git",
        ("it", "package_placeholder") => "Nome del pacchetto…",
        ("it", "search") => "Cerca pacchetto",
        ("it", "enter_package") => "Inserisci prima il nome di un pacchetto",
        ("it", "close") => "Chiudi",
        ("it", "confirm_storage_manager") => "Aprire il gestore di archiviazione nativo? Può modificare partizioni e dati.",
        ("it", "cancelled") => "Annullato",
        ("ca", "title") => "LTools",
        ("ca", "subtitle") => "Eines de sistema segures i accions ràpides",
        ("ca", "ready") => "Preparat",
        ("ca", "running") => "En execució…",
        ("ca", "completed") => "Completat",
        ("ca", "audit") => "Auditar discs i aplicacions",
        ("ca", "games") => "Inventariar jocs i llançadors",
        ("ca", "packages") => "Inventari de paquets",
        ("ca", "prefixes") => "Prefixos Wine/Proton",
        ("ca", "defaults") => "Rutes predeterminades",
        ("ca", "system") => "Estat del sistema",
        ("ca", "doctor") => "Dependències i diagnòstic",
        ("ca", "storage") => "Discs i particions",
        ("ca", "stores") => "Magatzems de paquets",
        ("ca", "git") => "Estat de Git",
        ("ca", "package_placeholder") => "Nom del paquet…",
        ("ca", "search") => "Cercar paquet",
        ("ca", "enter_package") => "Introdueix primer un nom de paquet",
        ("ca", "close") => "Tancar",
        ("ca", "confirm_storage_manager") => "Obrir el gestor d’emmagatzematge natiu? Pot modificar particions i dades.",
        ("ca", "cancelled") => "Cancel·lat",
        ("nl", "title") => "LTools",
        ("nl", "subtitle") => "Veilige systeemtools en snelle acties",
        ("nl", "ready") => "Gereed",
        ("nl", "running") => "Bezig…",
        ("nl", "completed") => "Voltooid",
        ("nl", "audit") => "Schijven en toepassingen controleren",
        ("nl", "games") => "Games en launchers inventariseren",
        ("nl", "packages") => "Pakketinventaris",
        ("nl", "prefixes") => "Wine/Proton-prefixes",
        ("nl", "defaults") => "Standaardpaden",
        ("nl", "system") => "Systeemstatus",
        ("nl", "doctor") => "Afhankelijkheden en diagnose",
        ("nl", "storage") => "Schijven en partities",
        ("nl", "stores") => "Pakketbronnen",
        ("nl", "git") => "Git-status",
        ("nl", "package_placeholder") => "Pakketnaam…",
        ("nl", "search") => "Pakket zoeken",
        ("nl", "enter_package") => "Voer eerst een pakketnaam in",
        ("nl", "close") => "Sluiten",
        ("nl", "confirm_storage_manager") => "Native opslagbeheerder openen? Deze kan partities en gegevens wijzigen.",
        ("nl", "cancelled") => "Geannuleerd",
        ("pl", "title") => "LTools",
        ("pl", "subtitle") => "Bezpieczne narzędzia systemowe i szybkie działania",
        ("pl", "ready") => "Gotowe",
        ("pl", "running") => "W toku…",
        ("pl", "completed") => "Ukończono",
        ("pl", "audit") => "Audyt dysków i aplikacji",
        ("pl", "games") => "Inwentaryzuj gry i launchery",
        ("pl", "packages") => "Spis pakietów",
        ("pl", "prefixes") => "Prefiksy Wine/Proton",
        ("pl", "defaults") => "Ścieżki domyślne",
        ("pl", "system") => "Stan systemu",
        ("pl", "doctor") => "Zależności i diagnostyka",
        ("pl", "storage") => "Dyski i partycje",
        ("pl", "stores") => "Repozytoria pakietów",
        ("pl", "git") => "Stan Git",
        ("pl", "package_placeholder") => "Nazwa pakietu…",
        ("pl", "search") => "Szukaj pakietu",
        ("pl", "enter_package") => "Najpierw wpisz nazwę pakietu",
        ("pl", "close") => "Zamknij",
        ("pl", "confirm_storage_manager") => "Otworzyć natywnego menedżera pamięci? Może zmieniać partycje i dane.",
        ("pl", "cancelled") => "Anulowano",
        ("ar", "title") => "LTools",
        ("ar", "subtitle") => "أدوات نظام آمنة وإجراءات سريعة",
        ("ar", "ready") => "جاهز",
        ("ar", "running") => "قيد التنفيذ…",
        ("ar", "completed") => "اكتمل",
        ("ar", "audit") => "تدقيق الأقراص والتطبيقات",
        ("ar", "games") => "جرد الألعاب ومشغلاتها",
        ("ar", "packages") => "جرد الحزم",
        ("ar", "prefixes") => "بادئات Wine/Proton",
        ("ar", "defaults") => "المسارات الافتراضية",
        ("ar", "system") => "حالة النظام",
        ("ar", "doctor") => "التبعيات والتشخيص",
        ("ar", "storage") => "الأقراص والأقسام",
        ("ar", "stores") => "مخازن الحزم",
        ("ar", "git") => "حالة Git",
        ("ar", "package_placeholder") => "اسم الحزمة…",
        ("ar", "search") => "البحث عن حزمة",
        ("ar", "enter_package") => "أدخل اسم حزمة أولاً",
        ("ar", "close") => "إغلاق",
        ("ar", "confirm_storage_manager") => "فتح مدير التخزين الأصلي؟ يمكنه تعديل الأقسام والبيانات.",
        ("ar", "cancelled") => "أُلغي",
        ("hi", "title") => "LTools",
        ("hi", "subtitle") => "सुरक्षित सिस्टम टूल और त्वरित कार्रवाइयाँ",
        ("hi", "ready") => "तैयार",
        ("hi", "running") => "चल रहा है…",
        ("hi", "completed") => "पूर्ण",
        ("hi", "audit") => "डिस्क और अनुप्रयोगों का ऑडिट",
        ("hi", "games") => "गेम और लॉन्चर सूचीबद्ध करें",
        ("hi", "packages") => "पैकेज सूची",
        ("hi", "prefixes") => "Wine/Proton प्रीफ़िक्स",
        ("hi", "defaults") => "डिफ़ॉल्ट पथ",
        ("hi", "system") => "सिस्टम स्थिति",
        ("hi", "doctor") => "निर्भरताएँ और निदान",
        ("hi", "storage") => "डिस्क और पार्टीशन",
        ("hi", "stores") => "पैकेज स्टोर",
        ("hi", "git") => "Git स्थिति",
        ("hi", "package_placeholder") => "पैकेज नाम…",
        ("hi", "search") => "पैकेज खोजें",
        ("hi", "enter_package") => "पहले पैकेज का नाम दर्ज करें",
        ("hi", "close") => "बंद करें",
        ("hi", "confirm_storage_manager") => "मूल स्टोरेज प्रबंधक खोलें? यह पार्टीशन और डेटा बदल सकता है।",
        ("hi", "cancelled") => "रद्द किया गया",
        ("ja", "title") => "LTools",
        ("ja", "subtitle") => "安全なシステムツールとクイックアクション",
        ("ja", "ready") => "準備完了",
        ("ja", "running") => "実行中…",
        ("ja", "completed") => "完了",
        ("ja", "audit") => "ディスクとアプリケーションを監査",
        ("ja", "games") => "ゲームとランチャーを一覧表示",
        ("ja", "packages") => "パッケージ一覧",
        ("ja", "prefixes") => "Wine/Proton プレフィックス",
        ("ja", "defaults") => "既定のパス",
        ("ja", "system") => "システム状態",
        ("ja", "doctor") => "依存関係と診断",
        ("ja", "storage") => "ディスクとパーティション",
        ("ja", "stores") => "パッケージストア",
        ("ja", "git") => "Git の状態",
        ("ja", "package_placeholder") => "パッケージ名…",
        ("ja", "search") => "パッケージを検索",
        ("ja", "enter_package") => "先にパッケージ名を入力してください",
        ("ja", "close") => "閉じる",
        ("ja", "confirm_storage_manager") => "標準のストレージ管理ツールを開きますか？パーティションとデータを変更できます。",
        ("ja", "cancelled") => "キャンセルしました",
        ("ko", "title") => "LTools",
        ("ko", "subtitle") => "안전한 시스템 도구 및 빠른 작업",
        ("ko", "ready") => "준비됨",
        ("ko", "running") => "실행 중…",
        ("ko", "completed") => "완료됨",
        ("ko", "audit") => "디스크 및 애플리케이션 감사",
        ("ko", "games") => "게임 및 런처 목록",
        ("ko", "packages") => "패키지 목록",
        ("ko", "prefixes") => "Wine/Proton 프리픽스",
        ("ko", "defaults") => "기본 경로",
        ("ko", "system") => "시스템 상태",
        ("ko", "doctor") => "종속성 및 진단",
        ("ko", "storage") => "디스크 및 파티션",
        ("ko", "stores") => "패키지 저장소",
        ("ko", "git") => "Git 상태",
        ("ko", "package_placeholder") => "패키지 이름…",
        ("ko", "search") => "패키지 검색",
        ("ko", "enter_package") => "먼저 패키지 이름을 입력하세요",
        ("ko", "close") => "닫기",
        ("ko", "confirm_storage_manager") => "기본 저장소 관리자를 여시겠습니까? 파티션과 데이터를 변경할 수 있습니다.",
        ("ko", "cancelled") => "취소됨",
        ("ro", "title") => "LTools",
        ("ro", "subtitle") => "Instrumente de sistem sigure și acțiuni rapide",
        ("ro", "ready") => "Pregătit",
        ("ro", "running") => "În execuție…",
        ("ro", "completed") => "Finalizat",
        ("ro", "audit") => "Auditează discurile și aplicațiile",
        ("ro", "games") => "Inventariază jocurile și lansatoarele",
        ("ro", "packages") => "Inventar de pachete",
        ("ro", "prefixes") => "Prefixe Wine/Proton",
        ("ro", "defaults") => "Căi implicite",
        ("ro", "system") => "Starea sistemului",
        ("ro", "doctor") => "Dependențe și diagnostic",
        ("ro", "storage") => "Discuri și partiții",
        ("ro", "stores") => "Depozite de pachete",
        ("ro", "git") => "Starea Git",
        ("ro", "package_placeholder") => "Numele pachetului…",
        ("ro", "search") => "Caută pachet",
        ("ro", "enter_package") => "Introdu mai întâi un nume de pachet",
        ("ro", "close") => "Închide",
        ("ro", "confirm_storage_manager") => "Deschizi managerul nativ de stocare? Poate modifica partiții și date.",
        ("ro", "cancelled") => "Anulat",
        ("ru", "title") => "LTools",
        ("ru", "subtitle") => "Безопасные системные инструменты и быстрые действия",
        ("ru", "ready") => "Готово",
        ("ru", "running") => "Выполняется…",
        ("ru", "completed") => "Завершено",
        ("ru", "audit") => "Проверить диски и приложения",
        ("ru", "games") => "Инвентаризация игр и лаунчеров",
        ("ru", "packages") => "Инвентаризация пакетов",
        ("ru", "prefixes") => "Префиксы Wine/Proton",
        ("ru", "defaults") => "Пути по умолчанию",
        ("ru", "system") => "Состояние системы",
        ("ru", "doctor") => "Зависимости и диагностика",
        ("ru", "storage") => "Диски и разделы",
        ("ru", "stores") => "Репозитории пакетов",
        ("ru", "git") => "Состояние Git",
        ("ru", "package_placeholder") => "Имя пакета…",
        ("ru", "search") => "Найти пакет",
        ("ru", "enter_package") => "Сначала введите имя пакета",
        ("ru", "close") => "Закрыть",
        ("ru", "confirm_storage_manager") => "Открыть штатный менеджер хранилища? Он может изменить разделы и данные.",
        ("ru", "cancelled") => "Отменено",
        ("uk", "title") => "LTools",
        ("uk", "subtitle") => "Безпечні системні інструменти та швидкі дії",
        ("uk", "ready") => "Готово",
        ("uk", "running") => "Виконується…",
        ("uk", "completed") => "Завершено",
        ("uk", "audit") => "Перевірити диски та програми",
        ("uk", "games") => "Інвентаризація ігор і запускників",
        ("uk", "packages") => "Інвентаризація пакунків",
        ("uk", "prefixes") => "Префікси Wine/Proton",
        ("uk", "defaults") => "Типові шляхи",
        ("uk", "system") => "Стан системи",
        ("uk", "doctor") => "Залежності та діагностика",
        ("uk", "storage") => "Диски та розділи",
        ("uk", "stores") => "Сховища пакунків",
        ("uk", "git") => "Стан Git",
        ("uk", "package_placeholder") => "Назва пакунка…",
        ("uk", "search") => "Знайти пакунок",
        ("uk", "enter_package") => "Спочатку введіть назву пакунка",
        ("uk", "close") => "Закрити",
        ("uk", "confirm_storage_manager") => "Відкрити штатний менеджер сховища? Він може змінити розділи й дані.",
        ("uk", "cancelled") => "Скасовано",
        ("zh", "title") => "LTools",
        ("zh", "subtitle") => "安全的系统工具和快捷操作",
        ("zh", "ready") => "就绪",
        ("zh", "running") => "运行中…",
        ("zh", "completed") => "已完成",
        ("zh", "audit") => "审计磁盘和应用程序",
        ("zh", "games") => "盘点游戏和启动器",
        ("zh", "packages") => "软件包清单",
        ("zh", "prefixes") => "Wine/Proton 前缀",
        ("zh", "defaults") => "默认路径",
        ("zh", "system") => "系统状态",
        ("zh", "doctor") => "依赖和诊断",
        ("zh", "storage") => "磁盘和分区",
        ("zh", "stores") => "软件包仓库",
        ("zh", "git") => "Git 状态",
        ("zh", "package_placeholder") => "软件包名称…",
        ("zh", "search") => "搜索软件包",
        ("zh", "enter_package") => "请先输入软件包名称",
        ("zh", "close") => "关闭",
        ("zh", "confirm_storage_manager") => "打开原生存储管理器？它可以修改分区和数据。",
        ("zh", "cancelled") => "已取消",
        (_, "title") => "LTools",
        (_, "subtitle") => "Herramientas seguras del sistema y acciones rápidas",
        (_, "ready") => "Listo",
        (_, "sections") => "Secciones",
        (_, "dashboard_hint") => "Elige una sección para trabajar con sus herramientas y acciones.",
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
        (_, "confirm_storage_manager") => "¿Abrir el gestor nativo de almacenamiento? Puede modificar particiones y datos.",
        (_, "confirm_git_operation") => "Esta operación Git/GitHub puede modificar el repositorio o el remoto. ¿Continuar?",
        (_, "cancelled") => "Cancelado",
        _ => "",
    }
}

/// Acciones del ciclo de vida del modal. Se mantienen separadas de los
/// estados (`running`, `completed`, `cancelled`) para que el botón muestre un
/// verbo claro y no parezca un resultado ya terminado.
#[cfg(any(not(windows), test))]
pub fn gui_action_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "cancel") => "Cancel",
        ("en", "cancelling") => "Cancelling…",
        ("de", "cancel") => "Abbrechen",
        ("de", "cancelling") => "Wird abgebrochen…",
        ("fr", "cancel") => "Annuler",
        ("fr", "cancelling") => "Annulation…",
        ("pt", "cancel") => "Cancelar",
        ("pt", "cancelling") => "A cancelar…",
        ("it", "cancel") => "Annulla",
        ("it", "cancelling") => "Annullamento…",
        ("ca", "cancel") => "Cancel·lar",
        ("ca", "cancelling") => "Cancel·lant…",
        ("nl", "cancel") => "Annuleren",
        ("nl", "cancelling") => "Annuleren…",
        ("pl", "cancel") => "Anuluj",
        ("pl", "cancelling") => "Anulowanie…",
        ("ar", "cancel") => "إلغاء",
        ("ar", "cancelling") => "جارٍ الإلغاء…",
        ("hi", "cancel") => "रद्द करें",
        ("hi", "cancelling") => "रद्द किया जा रहा है…",
        ("ja", "cancel") => "キャンセル",
        ("ja", "cancelling") => "キャンセル中…",
        ("ko", "cancel") => "취소",
        ("ko", "cancelling") => "취소 중…",
        ("ro", "cancel") => "Anulează",
        ("ro", "cancelling") => "Se anulează…",
        ("ru", "cancel") => "Отмена",
        ("ru", "cancelling") => "Отмена…",
        ("uk", "cancel") => "Скасувати",
        ("uk", "cancelling") => "Скасування…",
        ("zh", "cancel") => "取消",
        ("zh", "cancelling") => "正在取消…",
        (_, "cancel") => "Cancelar",
        (_, "cancelling") => "Cancelando…",
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
        ("en", "git_menu") => "Git / GitHub",
        ("en", "git_title") => "Git and GitHub operations",
        ("en", "git_clone") => "Clone a Git repository",
        ("en", "git_fetch") => "Fetch remote Git references",
        ("en", "git_pull") => "Pull and integrate Git changes",
        ("en", "git_log") => "View commit history",
        ("en", "git_add") => "Stage changes",
        ("en", "git_commit") => "Create commit",
        ("en", "git_push") => "Push changes",
        ("en", "git_branch") => "List or switch branch",
        ("en", "git_tag") => "List or create tag",
        ("en", "git_release") => "Create GitHub release",
        ("en", "git_login") => "Check Git identity and optional GitHub login",
        ("en", "gh_repo") => "GitHub repository",
        ("en", "gh_prs") => "GitHub pull requests",
        ("en", "gh_releases") => "GitHub releases",
        ("en", "gh_auth_status") => "GitHub authentication status",
        ("en", "git_repo_placeholder") => "Repository path (empty = current)",
        ("en", "git_url_placeholder") => "URL / branch / tag / path",
        ("en", "git_destination_placeholder") => "Destination / branch / title",
        ("en", "git_remote_message_placeholder") => "Remote / commit message",
        ("en", "git_notes_placeholder") => "Release notes",
        ("en", "git_limit_placeholder") => "Optional limit",
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
        ("ar", "pause") => "اضغط Enter للعودة: ",
        ("hi", "pause") => "वापस जाने के लिए Enter दबाएँ: ",
        ("ja", "pause") => "戻るにはEnterを押してください：",
        ("ko", "pause") => "돌아가려면 Enter를 누르세요: ",
        ("ro", "pause") => "Apasă Enter pentru revenire: ",
        ("ru", "pause") => "Нажмите Enter, чтобы вернуться: ",
        ("uk", "pause") => "Натисніть Enter для повернення: ",
        ("zh", "pause") => "按 Enter 返回：",
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
        (_, "git_menu") => "Git / GitHub",
        (_, "git_title") => "Operaciones Git y GitHub",
        (_, "git_clone") => "Clonar un repositorio Git",
        (_, "git_fetch") => "Descargar referencias Git remotas",
        (_, "git_pull") => "Descargar e integrar cambios Git",
        (_, "git_log") => "Ver historial de commits",
        (_, "git_add") => "Preparar cambios",
        (_, "git_commit") => "Crear commit",
        (_, "git_push") => "Subir cambios",
        (_, "git_branch") => "Listar o cambiar de rama",
        (_, "git_tag") => "Listar o crear tag",
        (_, "git_release") => "Crear release de GitHub",
        (_, "git_login") => "Comprobar identidad Git e inicio de sesión GitHub opcional",
        (_, "gh_repo") => "Repositorio de GitHub",
        (_, "gh_prs") => "Pull requests de GitHub",
        (_, "gh_releases") => "Releases de GitHub",
        (_, "gh_auth_status") => "Estado de autenticación GitHub",
        (_, "git_repo_placeholder") => "Ruta del repositorio (vacío = actual)",
        (_, "git_url_placeholder") => "URL / rama / tag / ruta",
        (_, "git_destination_placeholder") => "Destino / rama / título",
        (_, "git_remote_message_placeholder") => "Remoto / mensaje de commit",
        (_, "git_notes_placeholder") => "Notas de la release",
        (_, "git_limit_placeholder") => "Límite opcional",
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

/// Aviso mostrado cuando una salida tabular necesita desplazamiento
/// horizontal. Se mantiene fuera del texto de la acción para que los
/// informes CLI/JSON/TSV no reciban indicaciones específicas de la GUI.
#[cfg(target_os = "linux")]
pub fn horizontal_scroll_hint() -> &'static str {
    match current() {
        "en" => "Horizontal scrolling available; use the bottom scrollbar to read the full line.",
        "de" => "Horizontales Scrollen verfügbar; mit der unteren Leiste die vollständige Zeile lesen.",
        "fr" => "Défilement horizontal disponible ; utilisez la barre inférieure pour lire toute la ligne.",
        "pt" => "Deslocamento horizontal disponível; use a barra inferior para ler a linha completa.",
        "it" => "Scorrimento orizzontale disponibile; usa la barra inferiore per leggere tutta la riga.",
        "ca" => "Desplaçament horitzontal disponible; usa la barra inferior per llegir tota la línia.",
        "nl" => "Horizontaal scrollen beschikbaar; gebruik de onderste balk om de volledige regel te lezen.",
        "pl" => "Dostępne przewijanie poziome; użyj dolnego paska, aby odczytać cały wiersz.",
        "ar" => "يتوفر تمرير أفقي؛ استخدم الشريط السفلي لقراءة السطر بالكامل.",
        "hi" => "क्षैतिज स्क्रॉल उपलब्ध है; पूरी पंक्ति पढ़ने के लिए नीचे की पट्टी का उपयोग करें।",
        "ja" => "横スクロール可能です。下のスクロールバーで行全体を読めます。",
        "ko" => "가로 스크롤을 사용할 수 있습니다. 아래 스크롤바로 전체 줄을 읽으세요.",
        "ro" => "Derularea orizontală este disponibilă; folosește bara de jos pentru a citi rândul complet.",
        "ru" => "Доступна горизонтальная прокрутка; используйте нижнюю полосу, чтобы прочитать строку целиком.",
        "uk" => "Доступне горизонтальне прокручування; скористайтеся нижньою смугою, щоб прочитати весь рядок.",
        "zh" => "支持水平滚动；使用底部滚动条查看完整行。",
        _ => "Desplazamiento horizontal disponible; usa la barra inferior para leer la línea completa.",
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
        ("ar", "menu.prompt") => "اختر خيارًا (Enter للعودة): ",
        ("ar", "menu.back") => "عودة",
        ("hi", "menu.prompt") => "विकल्प चुनें (वापस जाने के लिए Enter): ",
        ("hi", "menu.back") => "वापस",
        ("ja", "menu.prompt") => "オプションを選択（戻るにはEnter）：",
        ("ja", "menu.back") => "戻る",
        ("ko", "menu.prompt") => "옵션을 선택하세요 (돌아가려면 Enter): ",
        ("ko", "menu.back") => "뒤로",
        ("ro", "menu.prompt") => "Alege o opțiune (Enter pentru revenire): ",
        ("ro", "menu.back") => "Înapoi",
        ("ru", "menu.prompt") => "Выберите пункт (Enter — назад): ",
        ("ru", "menu.back") => "Назад",
        ("uk", "menu.prompt") => "Виберіть пункт (Enter для повернення): ",
        ("uk", "menu.back") => "Назад",
        ("zh", "menu.prompt") => "选择一个选项（按 Enter 返回）：",
        ("zh", "menu.back") => "返回",
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
        ("es", "settings") => "Ajustes",
        ("es", "winslim") => "WinSlim",
        ("en", "audit_inventory") => "Audit / Inventory",
        ("en", "storage") => "Disk management",
        ("en", "services") => "Services / Dependencies",
        ("en", "defaults") => "Default paths",
        ("en", "automation") => "Automation",
        ("en", "import") => "Import scripts",
        ("en", "settings") => "Settings",
        ("en", "winslim") => "WinSlim",
        ("de", "audit_inventory") => "Prüfen / Inventarisieren",
        ("de", "storage") => "Datenträgerverwaltung",
        ("de", "services") => "Dienste / Abhängigkeiten",
        ("de", "defaults") => "Standardpfade",
        ("de", "automation") => "Automatisierung",
        ("de", "import") => "Skripte importieren",
        ("de", "settings") => "Einstellungen",
        ("de", "winslim") => "WinSlim",
        ("fr", "audit_inventory") => "Auditer / Inventorier",
        ("fr", "storage") => "Gestion des disques",
        ("fr", "services") => "Services / Dépendances",
        ("fr", "defaults") => "Chemins par défaut",
        ("fr", "automation") => "Automatisation",
        ("fr", "import") => "Importer des scripts",
        ("fr", "settings") => "Réglages",
        ("fr", "winslim") => "WinSlim",
        ("pt", "audit_inventory") => "Auditar / Inventariar",
        ("pt", "storage") => "Gestão de discos",
        ("pt", "services") => "Serviços / Dependências",
        ("pt", "defaults") => "Caminhos predefinidos",
        ("pt", "automation") => "Automação",
        ("pt", "import") => "Importar scripts",
        ("pt", "settings") => "Definições",
        ("pt", "winslim") => "WinSlim",
        ("it", "audit_inventory") => "Audit / Inventario",
        ("it", "storage") => "Gestione dischi",
        ("it", "services") => "Servizi / Dipendenze",
        ("it", "defaults") => "Percorsi predefiniti",
        ("it", "automation") => "Automazione",
        ("it", "import") => "Importa script",
        ("it", "settings") => "Impostazioni",
        ("it", "winslim") => "WinSlim",
        ("ca", "audit_inventory") => "Auditar / Inventariar",
        ("ca", "storage") => "Gestió de discs",
        ("ca", "services") => "Serveis / Dependències",
        ("ca", "defaults") => "Rutes predeterminades",
        ("ca", "automation") => "Automatització",
        ("ca", "import") => "Importar scripts",
        ("ca", "settings") => "Configuració",
        ("ca", "winslim") => "WinSlim",
        ("nl", "audit_inventory") => "Auditeren / Inventariseren",
        ("nl", "storage") => "Schijfbeheer",
        ("nl", "services") => "Diensten / Afhankelijkheden",
        ("nl", "defaults") => "Standaardpaden",
        ("nl", "automation") => "Automatisering",
        ("nl", "import") => "Scripts importeren",
        ("nl", "settings") => "Instellingen",
        ("nl", "winslim") => "WinSlim",
        ("pl", "audit_inventory") => "Audyt / Inwentaryzacja",
        ("pl", "storage") => "Zarządzanie dyskami",
        ("pl", "services") => "Usługi / Zależności",
        ("pl", "defaults") => "Ścieżki domyślne",
        ("pl", "automation") => "Automatyzacja",
        ("pl", "import") => "Import skryptów",
        ("pl", "settings") => "Ustawienia",
        ("pl", "winslim") => "WinSlim",
        ("ar", "audit_inventory") => "التدقيق / الجرد",
        ("ar", "storage") => "إدارة الأقراص",
        ("ar", "services") => "الخدمات / التبعيات",
        ("ar", "defaults") => "المسارات الافتراضية",
        ("ar", "automation") => "الأتمتة",
        ("ar", "import") => "استيراد البرامج النصية",
        ("ar", "settings") => "الإعدادات",
        ("ar", "winslim") => "WinSlim",
        ("hi", "audit_inventory") => "ऑडिट / इन्वेंटरी",
        ("hi", "storage") => "डिस्क प्रबंधन",
        ("hi", "services") => "सेवाएँ / निर्भरताएँ",
        ("hi", "defaults") => "डिफ़ॉल्ट पथ",
        ("hi", "automation") => "स्वचालन",
        ("hi", "import") => "स्क्रिप्ट आयात करें",
        ("hi", "settings") => "सेटिंग्स",
        ("hi", "winslim") => "WinSlim",
        ("ja", "audit_inventory") => "監査 / インベントリ",
        ("ja", "storage") => "ディスク管理",
        ("ja", "services") => "サービス / 依存関係",
        ("ja", "defaults") => "既定のパス",
        ("ja", "automation") => "自動化",
        ("ja", "import") => "スクリプトをインポート",
        ("ja", "settings") => "設定",
        ("ja", "winslim") => "WinSlim",
        ("ko", "audit_inventory") => "감사 / 인벤토리",
        ("ko", "storage") => "디스크 관리",
        ("ko", "services") => "서비스 / 종속성",
        ("ko", "defaults") => "기본 경로",
        ("ko", "automation") => "자동화",
        ("ko", "import") => "스크립트 가져오기",
        ("ko", "settings") => "설정",
        ("ko", "winslim") => "WinSlim",
        ("ro", "audit_inventory") => "Audit / Inventar",
        ("ro", "storage") => "Gestionarea discurilor",
        ("ro", "services") => "Servicii / Dependențe",
        ("ro", "defaults") => "Căi implicite",
        ("ro", "automation") => "Automatizare",
        ("ro", "import") => "Importă scripturi",
        ("ro", "settings") => "Setări",
        ("ro", "winslim") => "WinSlim",
        ("ru", "audit_inventory") => "Проверка / Инвентаризация",
        ("ru", "storage") => "Управление дисками",
        ("ru", "services") => "Службы / Зависимости",
        ("ru", "defaults") => "Пути по умолчанию",
        ("ru", "automation") => "Автоматизация",
        ("ru", "import") => "Импорт скриптов",
        ("ru", "settings") => "Настройки",
        ("ru", "winslim") => "WinSlim",
        ("uk", "audit_inventory") => "Аудит / Інвентаризація",
        ("uk", "storage") => "Керування дисками",
        ("uk", "services") => "Служби / Залежності",
        ("uk", "defaults") => "Типові шляхи",
        ("uk", "automation") => "Автоматизація",
        ("uk", "import") => "Імпортувати скрипти",
        ("uk", "settings") => "Налаштування",
        ("uk", "winslim") => "WinSlim",
        ("zh", "audit_inventory") => "审计 / 清单",
        ("zh", "storage") => "磁盘管理",
        ("zh", "services") => "服务 / 依赖项",
        ("zh", "defaults") => "默认路径",
        ("zh", "automation") => "自动化",
        ("zh", "import") => "导入脚本",
        ("zh", "settings") => "设置",
        ("zh", "winslim") => "WinSlim",
        ("es", "dependencies") => "Dependencias",
        ("es", "native_tools") => "Herramientas nativas",
        ("es", "installable_tools") => "Herramientas instalables",
        ("en", "dependencies") => "Dependencies",
        ("en", "native_tools") => "Native tools",
        ("en", "installable_tools") => "Installable tools",
        ("de", "dependencies") => "Abhängigkeiten",
        ("de", "native_tools") => "Native Werkzeuge",
        ("de", "installable_tools") => "Installierbare Werkzeuge",
        ("fr", "dependencies") => "Dépendances",
        ("fr", "native_tools") => "Outils natifs",
        ("fr", "installable_tools") => "Outils installables",
        ("pt", "dependencies") => "Dependências",
        ("pt", "native_tools") => "Ferramentas nativas",
        ("pt", "installable_tools") => "Ferramentas instaláveis",
        ("it", "dependencies") => "Dipendenze",
        ("it", "native_tools") => "Strumenti nativi",
        ("it", "installable_tools") => "Strumenti installabili",
        ("ca", "dependencies") => "Dependències",
        ("ca", "native_tools") => "Eines natives",
        ("ca", "installable_tools") => "Eines instal·lables",
        ("nl", "dependencies") => "Afhankelijkheden",
        ("nl", "native_tools") => "Native hulpprogramma's",
        ("nl", "installable_tools") => "Installeerbare hulpprogramma's",
        ("pl", "dependencies") => "Zależności",
        ("pl", "native_tools") => "Narzędzia natywne",
        ("pl", "installable_tools") => "Narzędzia instalowalne",
        ("ar", "dependencies") => "التبعيات",
        ("ar", "native_tools") => "الأدوات الأصلية",
        ("ar", "installable_tools") => "الأدوات القابلة للتثبيت",
        ("hi", "dependencies") => "निर्भरताएँ",
        ("hi", "native_tools") => "मूल उपकरण",
        ("hi", "installable_tools") => "इंस्टॉल करने योग्य उपकरण",
        ("ja", "dependencies") => "依存関係",
        ("ja", "native_tools") => "ネイティブツール",
        ("ja", "installable_tools") => "インストール可能なツール",
        ("ko", "dependencies") => "종속성",
        ("ko", "native_tools") => "네이티브 도구",
        ("ko", "installable_tools") => "설치 가능한 도구",
        ("ro", "dependencies") => "Dependențe",
        ("ro", "native_tools") => "Instrumente native",
        ("ro", "installable_tools") => "Instrumente instalabile",
        ("ru", "dependencies") => "Зависимости",
        ("ru", "native_tools") => "Системные инструменты",
        ("ru", "installable_tools") => "Устанавливаемые инструменты",
        ("uk", "dependencies") => "Залежності",
        ("uk", "native_tools") => "Вбудовані інструменти",
        ("uk", "installable_tools") => "Інструменти для встановлення",
        ("zh", "dependencies") => "依赖项",
        ("zh", "native_tools") => "原生工具",
        ("zh", "installable_tools") => "可安装工具",
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
        (_, "audit_inventory") => "Auditar / Inventariar",
        (_, "storage") => "Gestión de discos",
        (_, "services") => "Servicios / Dependencias",
        (_, "defaults") => "Rutas predeterminadas",
        (_, "automation") => "Automatización",
        (_, "import") => "Importar scripts",
        (_, "settings") => "Ajustes",
        (_, "winslim") => "WinSlim",
        (_, "audits") => "Auditorías e inventarios",
        (_, "cleanup") => "Limpieza y almacenamiento",
        (_, "applications") => "Aplicaciones y compatibilidad",
        (_, "system") => "Sistema y dispositivos",
        (_, "packages") => "Paquetes y Git",
        (_, "diagnostics") => "Diagnóstico y ayuda",
        (_, _) => "",
    }
}

/// Etiquetas de los ajustes de la CLI. Se mantienen fuera de `gui_text` para
/// que el binario de consola conserve portabilidad incluso en plataformas
/// donde no se compila una GUI nativa.
pub fn settings_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "theme") => "Theme",
        ("en", "language") => "Language",
        ("en", "color") => "Color mode",
        ("en", "current") => "Current",
        ("de", "theme") => "Thema",
        ("de", "language") => "Sprache",
        ("de", "color") => "Farbmodus",
        ("de", "current") => "Aktuell",
        ("fr", "theme") => "Thème",
        ("fr", "language") => "Langue",
        ("fr", "color") => "Mode de couleur",
        ("fr", "current") => "Actuel",
        ("pt", "theme") => "Tema",
        ("pt", "language") => "Idioma",
        ("pt", "color") => "Modo de cor",
        ("pt", "current") => "Atual",
        ("it", "theme") => "Tema",
        ("it", "language") => "Lingua",
        ("it", "color") => "Modalità colore",
        ("it", "current") => "Attuale",
        ("ca", "theme") => "Tema",
        ("ca", "language") => "Idioma",
        ("ca", "color") => "Mode de color",
        ("ca", "current") => "Actual",
        ("nl", "theme") => "Thema",
        ("nl", "language") => "Taal",
        ("nl", "color") => "Kleurmodus",
        ("nl", "current") => "Huidig",
        ("pl", "theme") => "Motyw",
        ("pl", "language") => "Język",
        ("pl", "color") => "Tryb kolorów",
        ("pl", "current") => "Bieżący",
        ("ar", "theme") => "السمة",
        ("ar", "language") => "اللغة",
        ("ar", "color") => "وضع الألوان",
        ("ar", "current") => "الحالي",
        ("hi", "theme") => "थीम",
        ("hi", "language") => "भाषा",
        ("hi", "color") => "रंग मोड",
        ("hi", "current") => "वर्तमान",
        ("ja", "theme") => "テーマ",
        ("ja", "language") => "言語",
        ("ja", "color") => "カラーモード",
        ("ja", "current") => "現在",
        ("ko", "theme") => "테마",
        ("ko", "language") => "언어",
        ("ko", "color") => "색상 모드",
        ("ko", "current") => "현재",
        ("ro", "theme") => "Temă",
        ("ro", "language") => "Limbă",
        ("ro", "color") => "Mod culoare",
        ("ro", "current") => "Actual",
        ("ru", "theme") => "Тема",
        ("ru", "language") => "Язык",
        ("ru", "color") => "Цветовой режим",
        ("ru", "current") => "Текущий",
        ("uk", "theme") => "Тема",
        ("uk", "language") => "Мова",
        ("uk", "color") => "Режим кольорів",
        ("uk", "current") => "Поточний",
        ("zh", "theme") => "主题",
        ("zh", "language") => "语言",
        ("zh", "color") => "颜色模式",
        ("zh", "current") => "当前",
        (_, "theme") => "Tema",
        (_, "language") => "Idioma",
        (_, "color") => "Modo de color",
        (_, "current") => "Actual",
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

/// Etiquetas de la jerarquía de submenús GUI. Se mantienen fuera de `gui.rs`
/// para que la organización visual no introduzca textos españoles al cambiar
/// el idioma compartido con la terminal.
#[allow(dead_code)]
pub fn gui_family_text(key: &str) -> &'static str {
    const FAMILY_TEXT: &[(&str, [&str; 15])] = &[
        (
            "native_storage",
            [
                "التخزين والأقسام",
                "Speicher und Partitionen",
                "Storage and partitions",
                "Almacenamiento y particiones",
                "Stockage et partitions",
                "स्टोरेज और पार्टिशन",
                "Archiviazione e partizioni",
                "ストレージとパーティション",
                "저장소 및 파티션",
                "Pamięć masowa i partycje",
                "Armazenamento e partições",
                "Stocare și partiții",
                "Хранилище и разделы",
                "Сховище та розділи",
                "存储和分区",
            ],
        ),
        (
            "native_system",
            [
                "النظام والشبكة والأمان",
                "System, Netzwerk und Sicherheit",
                "System, network and security",
                "Sistema, red y seguridad",
                "Système, réseau et sécurité",
                "सिस्टम, नेटवर्क और सुरक्षा",
                "Sistema, rete e sicurezza",
                "システム、ネットワークとセキュリティ",
                "시스템, 네트워크 및 보안",
                "System, sieć i bezpieczeństwo",
                "Sistema, rede e segurança",
                "Sistem, rețea și securitate",
                "Система, сеть и безопасность",
                "Система, мережа та безпека",
                "系统、网络和安全",
            ],
        ),
        (
            "installable_connectivity",
            [
                "SSH, SCP, SFTP وAndroid",
                "SSH, SCP, SFTP und Android",
                "SSH, SCP, SFTP and Android",
                "SSH, SCP, SFTP y Android",
                "SSH, SCP, SFTP et Android",
                "SSH, SCP, SFTP और Android",
                "SSH, SCP, SFTP e Android",
                "SSH、SCP、SFTP、Android",
                "SSH, SCP, SFTP 및 Android",
                "SSH, SCP, SFTP i Android",
                "SSH, SCP, SFTP e Android",
                "SSH, SCP, SFTP și Android",
                "SSH, SCP, SFTP и Android",
                "SSH, SCP, SFTP та Android",
                "SSH、SCP、SFTP 和 Android",
            ],
        ),
        (
            "installable_docker",
            [
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
                "Docker / Podman / Compose",
            ],
        ),
        (
            "containers",
            [
                "الحاويات",
                "Container",
                "Containers",
                "Contenedores",
                "Conteneurs",
                "कंटेनर",
                "Container",
                "コンテナ",
                "컨테이너",
                "Kontenery",
                "Contentores",
                "Containere",
                "Контейнеры",
                "Контейнери",
                "容器",
            ],
        ),
        (
            "images",
            [
                "الصور",
                "Images",
                "Images",
                "Imágenes",
                "Images",
                "इमेज",
                "Immagini",
                "イメージ",
                "이미지",
                "Obrazy",
                "Imagens",
                "Imagini",
                "Образы",
                "Образи",
                "镜像",
            ],
        ),
        (
            "volumes_networks",
            [
                "الأحجام والشبكات",
                "Volumes und Netzwerke",
                "Volumes and networks",
                "Volúmenes y redes",
                "Volumes et réseaux",
                "वॉल्यूम और नेटवर्क",
                "Volumi e reti",
                "ボリュームとネットワーク",
                "볼륨 및 네트워크",
                "Woluminy i sieci",
                "Volumes e redes",
                "Volume și rețele",
                "Тома и сети",
                "Томи та мережі",
                "卷和网络",
            ],
        ),
        (
            "compose_diagnostics",
            [
                "Compose والتشخيص",
                "Compose und Diagnose",
                "Compose and diagnostics",
                "Compose y diagnóstico",
                "Compose et diagnostic",
                "Compose और निदान",
                "Compose e diagnostica",
                "Compose と診断",
                "Compose 및 진단",
                "Compose i diagnostyka",
                "Compose e diagnóstico",
                "Compose și diagnostic",
                "Compose и диагностика",
                "Compose та діагностика",
                "Compose 和诊断",
            ],
        ),
        (
            "container_lifecycle",
            [
                "دورة حياة الحاويات والتحكم",
                "Lebenszyklus und Steuerung",
                "Container lifecycle and control",
                "Ciclo de vida y control",
                "Cycle de vie et contrôle",
                "कंटेनर जीवनचक्र और नियंत्रण",
                "Ciclo di vita e controllo",
                "コンテナのライフサイクルと制御",
                "컨테이너 수명 주기 및 제어",
                "Cykl życia i kontrola",
                "Ciclo de vida e controlo",
                "Ciclul de viață și control",
                "Жизненный цикл и управление",
                "Життєвий цикл і керування",
                "容器生命周期和控制",
            ],
        ),
        (
            "image_operations",
            [
                "عمليات الصور",
                "Image-Operationen",
                "Image operations",
                "Operaciones de imagen",
                "Opérations sur les images",
                "इमेज संचालन",
                "Operazioni sulle immagini",
                "イメージ操作",
                "이미지 작업",
                "Operacje na obrazach",
                "Operações de imagem",
                "Operații cu imagini",
                "Операции с образами",
                "Операції з образами",
                "镜像操作",
            ],
        ),
        (
            "volumes",
            [
                "الأحجام",
                "Volumes",
                "Volumes",
                "Volúmenes",
                "Volumes",
                "वॉल्यूम",
                "Volumi",
                "ボリューム",
                "볼륨",
                "Woluminy",
                "Volumes",
                "Volume",
                "Тома",
                "Томи",
                "卷",
            ],
        ),
        (
            "engine_networks",
            [
                "شبكات المحرك",
                "Engine-Netzwerke",
                "Engine networks",
                "Redes del motor",
                "Réseaux du moteur",
                "इंजन नेटवर्क",
                "Reti del motore",
                "エンジンネットワーク",
                "엔진 네트워크",
                "Sieci silnika",
                "Redes do motor",
                "Rețelele motorului",
                "Сети движка",
                "Мережі рушія",
                "引擎网络",
            ],
        ),
        (
            "compose",
            [
                "Compose", "Compose", "Compose", "Compose", "Compose", "Compose", "Compose",
                "Compose", "Compose", "Compose", "Compose", "Compose", "Compose", "Compose",
                "Compose",
            ],
        ),
        (
            "engine_diagnostics",
            [
                "تشخيص المحرك والتنظيف",
                "Engine-Diagnose und Bereinigung",
                "Engine diagnostics and cleanup",
                "Diagnóstico y limpieza del motor",
                "Diagnostic et nettoyage du moteur",
                "इंजन निदान और सफाई",
                "Diagnostica e pulizia del motore",
                "エンジンの診断とクリーンアップ",
                "엔진 진단 및 정리",
                "Diagnostyka i czyszczenie silnika",
                "Diagnóstico e limpeza do motor",
                "Diagnostic și curățare motor",
                "Диагностика и очистка движка",
                "Діагностика та очищення рушія",
                "引擎诊断和清理",
            ],
        ),
        (
            "k8s_resources",
            [
                "الموارد وعمليات النشر",
                "Ressourcen und Deployments",
                "Resources and deployments",
                "Recursos y despliegues",
                "Ressources et déploiements",
                "संसाधन और परिनियोजन",
                "Risorse e deployment",
                "リソースとデプロイ",
                "리소스 및 배포",
                "Zasoby i wdrożenia",
                "Recursos e implementações",
                "Resurse și implementări",
                "Ресурсы и развёртывания",
                "Ресурси та розгортання",
                "资源和部署",
            ],
        ),
    ];
    let language_index = SUPPORTED
        .iter()
        .position(|language| *language == current())
        .unwrap_or(3);
    FAMILY_TEXT
        .iter()
        .find(|(family, _)| *family == key)
        .map(|(_, values)| values[language_index])
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::{
        boot_label, category_text, gui_action_text, gui_family_text, gui_text, native_tools_label,
        normalize, set, settings_text, storage_action_text, SUPPORTED,
    };
    use std::sync::Mutex;

    static LANGUAGE_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn normalizes_language_variants() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
        assert_eq!(normalize("en_US.UTF-8"), "en");
        assert_eq!(normalize("pt-BR"), "pt");
        assert_eq!(normalize("unknown"), "es");
    }

    #[test]
    fn exposes_the_supported_catalog_languages() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
        assert_eq!(
            SUPPORTED,
            &[
                "ar", "de", "en", "es", "fr", "hi", "it", "ja", "ko", "pl", "pt", "ro", "ru", "uk",
                "zh",
            ]
        );
    }

    #[test]
    fn exposes_all_main_menu_categories_in_every_language() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
        for language in SUPPORTED {
            set(language);
            for category in [
                "audit_inventory",
                "dependencies",
                "native_tools",
                "installable_tools",
                "storage",
                "services",
                "defaults",
                "automation",
                "import",
                "settings",
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
            assert!(!boot_label().is_empty());
            if *language != "es" {
                assert_ne!(boot_label(), "Arranque, EFI y cargador del sistema");
            }
        }
        set("es");
    }

    #[test]
    fn exposes_cli_settings_labels_in_every_language() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
        for language in SUPPORTED {
            set(language);
            for key in ["theme", "language", "color", "current"] {
                assert!(!settings_text(key).is_empty());
            }
        }
        set("es");
    }

    #[test]
    fn exposes_automation_navigation_text() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
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

    #[test]
    fn core_ui_does_not_fall_back_to_spanish_for_supported_languages() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
        let spanish = [
            "Herramientas seguras del sistema y acciones rápidas",
            "Auditar discos y aplicaciones",
            "Discos y particiones",
            "Guía de particionado y protecciones",
            "Herramientas SSH, Android, Docker y Kubernetes",
        ];
        for language in SUPPORTED {
            set(language);
            for key in ["subtitle", "audit", "storage"] {
                let value = gui_text(key);
                assert!(!value.is_empty());
                if *language != "es" {
                    assert!(
                        !spanish.contains(&value),
                        "{language} todavía usa fallback español para {key}"
                    );
                }
            }
            let storage_guide = storage_action_text("guide");
            assert!(!storage_guide.is_empty());
            if *language != "es" {
                assert_ne!(
                    storage_guide, spanish[3],
                    "{language} todavía usa fallback español para storage guide"
                );
                assert_ne!(
                    native_tools_label(),
                    spanish[4],
                    "{language} todavía usa fallback español para native tools"
                );
            }
        }
        set("es");
    }

    #[test]
    fn settings_ui_is_available_in_every_terminal_language() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
        for language in SUPPORTED {
            set(language);
            for key in [
                "theme_button",
                "language_button",
                "settings_button",
                "settings_title",
                "settings_theme",
                "settings_language",
                "settings_visibility",
                "settings_restart",
                "visible",
                "hidden",
            ] {
                assert!(
                    !gui_text(key).is_empty(),
                    "{language} missing GUI key {key}"
                );
            }
        }
        set("es");
    }

    #[test]
    fn gui_family_labels_are_available_in_every_terminal_language() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
        let families = [
            "native_storage",
            "native_system",
            "installable_connectivity",
            "installable_docker",
            "containers",
            "images",
            "volumes_networks",
            "compose_diagnostics",
            "container_lifecycle",
            "image_operations",
            "volumes",
            "engine_networks",
            "compose",
            "engine_diagnostics",
            "k8s_resources",
        ];
        for language in SUPPORTED {
            set(language);
            for family in families {
                assert!(
                    !gui_family_text(family).is_empty(),
                    "{language} missing {family}"
                );
            }
        }
        set("es");
    }

    #[test]
    fn modal_actions_are_translated_in_every_terminal_language() {
        let _guard = LANGUAGE_TEST_LOCK.lock().unwrap();
        for language in SUPPORTED {
            set(language);
            assert!(!gui_action_text("cancel").is_empty());
            assert!(!gui_action_text("cancelling").is_empty());
        }
        set("es");
    }
}
