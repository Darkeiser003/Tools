use std::env;
#[cfg(test)]
static LANGUAGE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) struct LanguageTestGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous_language: Option<String>,
}

#[cfg(test)]
impl Drop for LanguageTestGuard {
    fn drop(&mut self) {
        match &self.previous_language {
            Some(language) => env::set_var("LTOOLS_LANG", language),
            None => env::remove_var("LTOOLS_LANG"),
        }
    }
}

#[cfg(test)]
pub(crate) fn language_test_guard() -> LanguageTestGuard {
    let lock = LANGUAGE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    LanguageTestGuard {
        _lock: lock,
        previous_language: env::var("LTOOLS_LANG").ok(),
    }
}

/// IDs de los 15 catálogos compartidos con LTerminal.
pub const SUPPORTED: &[&str] = &[
    "ar", "de", "en", "es", "fr", "hi", "it", "ja", "ko", "pl", "pt", "ro", "ru", "uk", "zh",
];

/// Categorías configurables en Ajustes; se comparte con la GUI y su guía.
pub const SETTINGS_CATEGORY_KEYS: [&str; 6] = [
    "audit_inventory",
    "dependencies",
    "native_tools",
    "defaults",
    "installable_tools",
    "automation",
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
        match current() {
            "en" => "Inventory native Windows games and launchers",
            "de" => "Native Windows-Spiele und Launcher inventarisieren",
            "fr" => "Inventorier les jeux et lanceurs Windows natifs",
            "pt" => "Inventariar jogos e lançadores nativos do Windows",
            "it" => "Inventaria giochi e launcher Windows nativi",
            "ca" => "Inventariar jocs i llançadors natius de Windows",
            "nl" => "Native Windows-games en launchers inventariseren",
            "pl" => "Inwentaryzuj natywne gry i launchery Windows",
            _ => "Inventario de juegos y lanzadores Windows nativos",
        }
    }
    #[cfg(not(windows))]
    {
        text("menu.games")
    }
}

pub fn games_help() -> &'static str {
    #[cfg(windows)]
    {
        match current() {
            "en" => "Native Windows game and launcher inventory",
            "de" => "Inventar nativer Windows-Spiele und Launcher",
            "fr" => "Inventaire des jeux et lanceurs Windows natifs",
            "pt" => "Inventário de jogos e lançadores nativos do Windows",
            "it" => "Inventario di giochi e launcher Windows nativi",
            "ca" => "Inventari de jocs i llançadors natius de Windows",
            "nl" => "Inventaris van native Windows-games en launchers",
            "pl" => "Inwentaryzacja natywnych gier i launcherów Windows",
            _ => "Inventario nativo de juegos y lanzadores Windows",
        }
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
        match current() {
            "en" => "Wine/Proton prefixes (not applicable on Windows)",
            "de" => "Wine-/Proton-Präfixe (unter Windows nicht anwendbar)",
            "fr" => "Préfixes Wine/Proton (inapplicables sous Windows)",
            "pt" => "Prefixos Wine/Proton (não aplicáveis no Windows)",
            "it" => "Prefissi Wine/Proton (non applicabili su Windows)",
            "ca" => "Prefixos Wine/Proton (no aplicables a Windows)",
            "nl" => "Wine-/Proton-prefixes (niet van toepassing op Windows)",
            "pl" => "Prefiksy Wine/Proton (nie dotyczą Windows)",
            _ => "Prefijos Wine/Proton (no aplicable en Windows)",
        }
    }
    #[cfg(not(windows))]
    {
        text("menu.prefix")
    }
}

pub fn prefix_help() -> &'static str {
    #[cfg(windows)]
    {
        match current() {
            "en" => "Not available in native Windows mode; no Wine/Proton paths are scanned",
            "de" => "Im nativen Windows-Modus nicht verfügbar; keine Wine-/Proton-Pfade werden geprüft",
            "fr" => "Indisponible en mode Windows natif ; aucun chemin Wine/Proton n’est analysé",
            "pt" => "Indisponível no modo Windows nativo; não são analisados caminhos Wine/Proton",
            "it" => "Non disponibile in modalità Windows nativa; nessun percorso Wine/Proton viene analizzato",
            "ca" => "No disponible en mode Windows natiu; no s’analitzen rutes Wine/Proton",
            "nl" => "Niet beschikbaar in native Windows-modus; er worden geen Wine-/Proton-paden gescand",
            "pl" => "Niedostępne w natywnym trybie Windows; ścieżki Wine/Proton nie są skanowane",
            _ => "No aplicable en Windows nativo; no se escanean rutas Wine/Proton",
        }
    }
    #[cfg(not(windows))]
    {
        text("help.prefix")
    }
}

pub fn defaults_help() -> &'static str {
    #[cfg(windows)]
    {
        match current() {
            "en" => "Show native Windows launcher locations",
            "de" => "Native Windows-Launcherpfade anzeigen",
            "fr" => "Afficher les emplacements des lanceurs Windows natifs",
            "pt" => "Mostrar localizações dos lançadores nativos do Windows",
            "it" => "Mostra i percorsi dei launcher Windows nativi",
            "ca" => "Mostrar ubicacions dels llançadors natius de Windows",
            "nl" => "Native Windows-launcherlocaties tonen",
            "pl" => "Pokaż lokalizacje natywnych launcherów Windows",
            _ => "Mostrar rutas de lanzadores nativos de Windows",
        }
    }
    #[cfg(not(windows))]
    {
        text("help.defaults")
    }
}

pub fn system_help() -> &'static str {
    #[cfg(windows)]
    {
        match current() {
            "en" => "Windows services, processes, event log and controlled actions",
            "de" => "Windows-Dienste, Prozesse, Ereignisprotokoll und kontrollierte Aktionen",
            "fr" => "Services Windows, processus, journal des événements et actions contrôlées",
            "pt" => "Serviços, processos, eventos e ações controladas do Windows",
            "it" => "Servizi Windows, processi, registro eventi e azioni controllate",
            "ca" => "Serveis, processos, registre d’esdeveniments i accions controlades de Windows",
            "nl" => "Windows-services, processen, gebeurtenislogboek en gecontroleerde acties",
            "pl" => "Usługi Windows, procesy, dziennik zdarzeń i kontrolowane działania",
            _ => "Servicios, procesos, eventos y acciones controladas de Windows",
        }
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
        match current() {
            "en" => "Manage Windows disks and partitions",
            "de" => "Windows-Datenträger und Partitionen verwalten",
            "fr" => "Gérer les disques et partitions Windows",
            "pt" => "Gerir discos e partições do Windows",
            "it" => "Gestisci dischi e partizioni Windows",
            "ca" => "Gestionar discs i particions de Windows",
            "nl" => "Windows-schijven en partities beheren",
            "pl" => "Zarządzaj dyskami i partycjami Windows",
            _ => "Gestionar discos y particiones Windows",
        }
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
    gui_account_text("title")
}

/// Labels of the account-management GUI. Keep the same order as `SUPPORTED`
/// so both the GTK and Win32 screens use the selected locale consistently.
pub fn gui_account_text(key: &str) -> &'static str {
    const TEXT: &[(&str, [&str; 15])] = &[
        ("title", [
            "المستخدمون والمجموعات والجلسات", "Benutzer, Gruppen und Sitzungen", "Users, groups and sessions", "Usuarios, grupos y sesiones", "Utilisateurs, groupes et sessions", "उपयोगकर्ता, समूह और सत्र", "Utenti, gruppi e sessioni", "ユーザー、グループ、セッション", "사용자, 그룹 및 세션", "Użytkownicy, grupy i sesje", "Utilizadores, grupos e sessões", "Utilizatori, grupuri și sesiuni", "Пользователи, группы и сеансы", "Користувачі, групи та сеанси", "用户、组和会话",
        ]),
        ("manage_accounts", [
            "إدارة الحسابات", "Konten verwalten", "Manage accounts", "Gestionar cuentas", "Gérer les comptes", "खाते प्रबंधित करें", "Gestisci account", "アカウントの管理", "계정 관리", "Zarządzaj kontami", "Gerir contas", "Gestionați conturile", "Управление учетными записями", "Керування обліковими записами", "管理账户",
        ]),
        ("manage_groups", [
            "إدارة المجموعات والعضوية", "Gruppen und Mitgliedschaften verwalten", "Manage groups and membership", "Gestionar grupos y membresías", "Gérer les groupes et les membres", "समूह और सदस्यता प्रबंधित करें", "Gestisci gruppi e appartenenze", "グループとメンバーシップの管理", "그룹 및 구성원 관리", "Zarządzaj grupami i członkostwem", "Gerir grupos e associações", "Gestionați grupurile și apartenența", "Управление группами и участниками", "Керування групами та учасниками", "管理组和成员关系",
        ]),
        ("team_admin", [
            "إدارة مسؤولي الجهاز", "Computeradministratoren verwalten", "Manage device administrators", "Administración del equipo", "Gérer les administrateurs de l’ordinateur", "डिवाइस व्यवस्थापकों को प्रबंधित करें", "Gestisci gli amministratori del dispositivo", "デバイス管理者の管理", "장치 관리자 관리", "Zarządzaj administratorami urządzenia", "Gerir administradores do computador", "Gestionați administratorii dispozitivului", "Управление администраторами устройства", "Керування адміністраторами пристрою", "管理设备管理员",
        ]),
        ("list", [
            "عرض الحسابات المحلية", "Lokale Konten auflisten", "List local accounts", "Listar cuentas locales", "Lister les comptes locaux", "स्थानीय खाते सूचीबद्ध करें", "Elenca account locali", "ローカルアカウント一覧", "로컬 계정 목록", "Wyświetl konta lokalne", "Listar contas locais", "Listați conturile locale", "Список локальных учетных записей", "Список локальних облікових записів", "列出本地账户",
        ]),
        ("groups", [
            "عرض المجموعات والأعضاء", "Gruppen und Mitglieder auflisten", "List groups and members", "Listar grupos y miembros", "Lister les groupes et membres", "समूह और सदस्य सूचीबद्ध करें", "Elenca gruppi e membri", "グループとメンバー一覧", "그룹 및 구성원 목록", "Wyświetl grupy i członków", "Listar grupos e membros", "Listați grupurile și membrii", "Список групп и участников", "Список груп і учасників", "列出组和成员",
        ]),
        ("identity", [
            "هويتي ومجموعاتي", "Meine Identität und Gruppen", "My identity and groups", "Ver mi identidad y grupos", "Mon identité et mes groupes", "मेरी पहचान और समूह", "La mia identità e i gruppi", "自分のIDとグループ", "내 ID 및 그룹", "Moja tożsamość i grupy", "A minha identidade e grupos", "Identitatea și grupurile mele", "Моя учетная запись и группы", "Моя особа та групи", "我的身份和组",
        ]),
        ("sessions", [
            "الجلسات المفتوحة", "Offene Sitzungen", "Open sessions", "Ver sesiones abiertas", "Sessions ouvertes", "खुले सत्र", "Sessioni aperte", "開いているセッション", "열린 세션", "Otwarte sesje", "Sessões abertas", "Sesiuni deschise", "Открытые сеансы", "Відкриті сеанси", "已打开的会话",
        ]),
        ("inspect", [
            "فحص حساب", "Konto untersuchen", "Inspect account", "Inspeccionar una cuenta", "Inspecter un compte", "खाते की जाँच करें", "Ispeziona account", "アカウントを調査", "계정 검사", "Sprawdź konto", "Inspecionar conta", "Inspectați contul", "Проверить учетную запись", "Перевірити обліковий запис", "检查账户",
        ]),
        ("create", [
            "إنشاء حساب", "Konto erstellen", "Create account", "Crear cuenta", "Créer un compte", "खाता बनाएँ", "Crea account", "アカウントを作成", "계정 만들기", "Utwórz konto", "Criar conta", "Creați un cont", "Создать учетную запись", "Створити обліковий запис", "创建账户",
        ]),
        ("modify", [
            "تعديل حساب", "Konto bearbeiten", "Edit account", "Editar cuenta y grupos", "Modifier un compte", "खाता संपादित करें", "Modifica account", "アカウントを編集", "계정 편집", "Edytuj konto", "Editar conta", "Editați contul", "Изменить учетную запись", "Змінити обліковий запис", "编辑账户",
        ]),
        ("password", [
            "تغيير كلمة المرور", "Passwort ändern", "Change password", "Cambiar contraseña", "Changer le mot de passe", "पासवर्ड बदलें", "Cambia password", "パスワードを変更", "암호 변경", "Zmień hasło", "Alterar palavra-passe", "Schimbați parola", "Сменить пароль", "Змінити пароль", "更改密码",
        ]),
        ("lock", [
            "قفل الحساب", "Konto sperren", "Lock account", "Bloquear cuenta", "Verrouiller le compte", "खाता लॉक करें", "Blocca account", "アカウントをロック", "계정 잠금", "Zablokuj konto", "Bloquear conta", "Blocați contul", "Заблокировать учетную запись", "Заблокувати обліковий запис", "锁定账户",
        ]),
        ("unlock", [
            "إلغاء قفل الحساب", "Konto entsperren", "Unlock account", "Desbloquear cuenta", "Déverrouiller le compte", "खाता अनलॉक करें", "Sblocca account", "アカウントのロックを解除", "계정 잠금 해제", "Odblokuj konto", "Desbloquear conta", "Deblocați contul", "Разблокировать учетную запись", "Розблокувати обліковий запис", "解锁账户",
        ]),
        ("delete", [
            "حذف الحساب", "Konto löschen", "Delete account", "Eliminar cuenta", "Supprimer le compte", "खाता हटाएँ", "Elimina account", "アカウントを削除", "계정 삭제", "Usuń konto", "Eliminar conta", "Ștergeți contul", "Удалить учетную запись", "Видалити обліковий запис", "删除账户",
        ]),
        ("expire", [
            "تعيين انتهاء الصلاحية", "Ablauf festlegen", "Set expiry", "Configurar caducidad", "Définir l’expiration", "समाप्ति सेट करें", "Imposta scadenza", "有効期限を設定", "만료 설정", "Ustaw wygaśnięcie", "Definir validade", "Setați expirarea", "Настроить срок действия", "Налаштувати термін дії", "设置到期时间",
        ]),
        ("group_create", [
            "إنشاء مجموعة", "Gruppe erstellen", "Create group", "Crear grupo", "Créer un groupe", "समूह बनाएँ", "Crea gruppo", "グループを作成", "그룹 만들기", "Utwórz grupę", "Criar grupo", "Creați un grup", "Создать группу", "Створити групу", "创建组",
        ]),
        ("group_delete", [
            "حذف مجموعة", "Gruppe löschen", "Delete group", "Eliminar grupo", "Supprimer un groupe", "समूह हटाएँ", "Elimina gruppo", "グループを削除", "그룹 삭제", "Usuń grupę", "Eliminar grupo", "Ștergeți grupul", "Удалить группу", "Видалити групу", "删除组",
        ]),
        ("group_primary", [
            "تغيير المجموعة الأساسية", "Primärgruppe ändern", "Change primary group", "Cambiar grupo principal", "Changer le groupe principal", "प्राथमिक समूह बदलें", "Cambia gruppo primario", "プライマリグループを変更", "기본 그룹 변경", "Zmień grupę podstawową", "Alterar grupo principal", "Schimbați grupul principal", "Изменить основную группу", "Змінити основну групу", "更改主组",
        ]),
        ("group_add", [
            "إضافة مستخدم إلى مجموعة", "Benutzer zu Gruppe hinzufügen", "Add user to group", "Añadir usuario a grupo", "Ajouter un utilisateur au groupe", "समूह में उपयोगकर्ता जोड़ें", "Aggiungi utente al gruppo", "ユーザーをグループに追加", "그룹에 사용자 추가", "Dodaj użytkownika do grupy", "Adicionar utilizador ao grupo", "Adăugați utilizatorul în grup", "Добавить пользователя в группу", "Додати користувача до групи", "将用户添加到组",
        ]),
        ("group_remove", [
            "إزالة مستخدم من مجموعة", "Benutzer aus Gruppe entfernen", "Remove user from group", "Retirar usuario de grupo", "Retirer un utilisateur du groupe", "समूह से उपयोगकर्ता हटाएँ", "Rimuovi utente dal gruppo", "ユーザーをグループから削除", "그룹에서 사용자 제거", "Usuń użytkownika z grupy", "Remover utilizador do grupo", "Eliminați utilizatorul din grup", "Удалить пользователя из группы", "Вилучити користувача з групи", "从组中移除用户",
        ]),
        ("admin_add", [
            "منح صلاحيات المسؤول", "Administratorrechte vergeben", "Grant administrator rights", "Conceder permisos de administrador", "Accorder les droits administrateur", "व्यवस्थापक अधिकार दें", "Concedi diritti di amministratore", "管理者権限を付与", "관리자 권한 부여", "Nadaj uprawnienia administratora", "Conceder permissões de administrador", "Acordați drepturi de administrator", "Предоставить права администратора", "Надати права адміністратора", "授予管理员权限",
        ]),
        ("admin_groups", [
            "مجموعة المسؤولين وأعضاؤها", "Administratorgruppe und Mitglieder", "Admin group and members", "Ver grupo y miembros administradores", "Groupe administrateur et membres", "व्यवस्थापक समूह और सदस्य", "Gruppo amministratori e membri", "管理者グループとメンバー", "관리자 그룹 및 구성원", "Grupa administratorów i członkowie", "Grupo de administradores e membros", "Grupul administratorilor și membrii", "Группа администраторов и участники", "Група адміністраторів та учасники", "管理员组及成员",
        ]),
        ("guide", [
            "دليل الحسابات والأذونات", "Leitfaden für Konten und Berechtigungen", "Accounts and permissions guide", "Guía de cuentas y permisos", "Guide des comptes et autorisations", "खातों और अनुमतियों की मार्गदर्शिका", "Guida ad account e autorizzazioni", "アカウントと権限のガイド", "계정 및 권한 안내", "Przewodnik po kontach i uprawnieniach", "Guia de contas e permissões", "Ghid pentru conturi și permisiuni", "Руководство по учетным записям и разрешениям", "Посібник з облікових записів і дозволів", "账户和权限指南",
        ]),
        ("field_user_group", [
            "المستخدم أو المجموعة", "Benutzer oder Gruppe", "User or group", "Usuario o grupo", "Utilisateur ou groupe", "उपयोगकर्ता या समूह", "Utente o gruppo", "ユーザーまたはグループ", "사용자 또는 그룹", "Użytkownik lub grupa", "Utilizador ou grupo", "Utilizator sau grup", "Пользователь или группа", "Користувач або група", "用户或组",
        ]),
        ("field_details", [
            "الوصف / كلمة المرور / المجموعة", "Beschreibung / Passwort / Gruppe", "Description / password / group", "Descripción / contraseña / grupo", "Description / mot de passe / groupe", "विवरण / पासवर्ड / समूह", "Descrizione / password / gruppo", "説明 / パスワード / グループ", "설명 / 암호 / 그룹", "Opis / hasło / grupa", "Descrição / palavra-passe / grupo", "Descriere / parolă / grup", "Описание / пароль / группа", "Опис / пароль / група", "描述 / 密码 / 组",
        ]),
        ("field_confirm", [
            "التأكيد / الاسم الكامل", "Bestätigung / vollständiger Name", "Confirmation / full name", "Confirmación / nombre completo", "Confirmation / nom complet", "पुष्टि / पूरा नाम", "Conferma / nome completo", "確認 / 氏名", "확인 / 전체 이름", "Potwierdzenie / pełna nazwa", "Confirmação / nome completo", "Confirmare / nume complet", "Подтверждение / полное имя", "Підтвердження / повне ім’я", "确认 / 全名",
        ]),
        ("field_optional", [
            "اختياري / انتهاء / تاريخ ISO", "Optional / Ablauf / ISO-Datum", "Optional / expiry / ISO date", "Opcional / vence / fecha ISO", "Optionnel / expiration / date ISO", "वैकल्पिक / समाप्ति / ISO तिथि", "Opzionale / scadenza / data ISO", "任意 / 期限 / ISO日付", "선택 사항 / 만료 / ISO 날짜", "Opcjonalnie / wygaśnięcie / data ISO", "Opcional / validade / data ISO", "Opțional / expirare / dată ISO", "Необязательно / срок / дата ISO", "Необов’язково / термін / дата ISO", "可选 / 到期 / ISO 日期",
        ]),
        ("local_user", [
            "المستخدم المحلي", "Lokaler Benutzer", "Local user", "Usuario local", "Utilisateur local", "स्थानीय उपयोगकर्ता", "Utente locale", "ローカルユーザー", "로컬 사용자", "Użytkownik lokalny", "Utilizador local", "Utilizator local", "Локальный пользователь", "Локальний користувач", "本地用户",
        ]),
        ("new_password", [
            "كلمة المرور الجديدة", "Neues Passwort", "New password", "Contraseña nueva", "Nouveau mot de passe", "नया पासवर्ड", "Nuova password", "新しいパスワード", "새 암호", "Nowe hasło", "Nova palavra-passe", "Parolă nouă", "Новый пароль", "Новий пароль", "新密码",
        ]),
        ("repeat_password", [
            "أعد إدخال كلمة المرور", "Passwort wiederholen", "Repeat password", "Repite la contraseña", "Répéter le mot de passe", "पासवर्ड दोहराएँ", "Ripeti la password", "パスワードを再入力", "암호 다시 입력", "Powtórz hasło", "Repetir palavra-passe", "Repetați parola", "Повторите пароль", "Повторіть пароль", "再次输入密码",
        ]),
        ("guide_fields", [
            "تختلف الحقول حسب الإجراء: المستخدم أو المجموعة أو الوصف أو كلمة المرور أو التواريخ. راجع الهدف قبل التنفيذ. إجراء صلاحيات المسؤول يقبل حسابًا اختياريًا؛ والفراغ يعني الحساب الحالي، ويستخدم SID مجموعة Administrators المضمنة.",
            "Die Felder hängen von der Aktion ab: Benutzer, Gruppe, Beschreibung, Kennwort oder Datum. Ziel vor dem Ausführen prüfen. Die Administratoraktion akzeptiert ein optionales Konto; leer bedeutet das aktuelle Konto und verwendet die integrierte Administratoren-SID.",
            "Fields depend on the action: user, group, description, password or dates. Verify the target before running. The administrator-rights action accepts an optional account; blank means the current account and uses the built-in Administrators SID.",
            "Los campos dependen de la acción: usuario, grupo, descripción, contraseña o fechas; se validan antes de ejecutar. «Conceder permisos de administrador» acepta una cuenta opcional (vacío = actual) y usa el SID integrado de Administradores.",
            "Les champs dépendent de l’action : utilisateur, groupe, description, mot de passe ou dates. Vérifiez la cible avant l’exécution. L’action d’administration accepte un compte facultatif ; vide signifie le compte actuel et utilise le SID intégré des Administrateurs.",
            "कार्रवाई के अनुसार फ़ील्ड बदलते हैं: उपयोगकर्ता, समूह, विवरण, पासवर्ड या तारीखें। चलाने से पहले लक्ष्य जाँचें। व्यवस्थापक अधिकार वाली कार्रवाई वैकल्पिक खाता लेती है; खाली छोड़ने पर वर्तमान खाता और अंतर्निहित Administrators SID उपयोग होता है।",
            "I campi dipendono dall’azione: utente, gruppo, descrizione, password o date. Verifica la destinazione prima di eseguire. L’azione per i diritti di amministratore accetta un account facoltativo; vuoto indica l’account corrente e usa il SID integrato Administrators.",
            "項目は操作によって異なります（ユーザー、グループ、説明、パスワード、日付）。実行前に対象を確認してください。管理者権限の操作ではアカウントを省略できます。空欄は現在のアカウントを意味し、組み込み Administrators の SID を使います。",
            "필드는 작업에 따라 달라집니다(사용자, 그룹, 설명, 암호, 날짜). 실행 전에 대상을 확인하세요. 관리자 권한 작업은 계정을 선택적으로 받으며, 비워 두면 현재 계정과 기본 제공 Administrators SID를 사용합니다.",
            "Pola zależą od działania: użytkownik, grupa, opis, hasło lub daty. Przed wykonaniem sprawdź cel. Nadanie praw administratora przyjmuje opcjonalne konto; puste pole oznacza bieżące konto i używa wbudowanego identyfikatora SID grupy Administratorzy.",
            "Os campos dependem da ação: utilizador, grupo, descrição, palavra-passe ou datas. Confirme o destino antes de executar. A ação de administração aceita uma conta opcional; vazio significa a conta atual e utiliza o SID integrado Administradores.",
            "Câmpurile depind de acțiune: utilizator, grup, descriere, parolă sau date. Verificați ținta înainte de executare. Acțiunea pentru drepturi de administrator acceptă un cont opțional; gol înseamnă contul curent și folosește SID-ul integrat Administrators.",
            "Поля зависят от действия: пользователь, группа, описание, пароль или даты. Проверьте цель перед запуском. Действие назначения прав администратора принимает необязательную учетную запись; пустое поле означает текущую учетную запись и использует встроенный SID группы Administrators.",
            "Поля залежать від дії: користувач, група, опис, пароль або дати. Перевірте ціль перед запуском. Дія надання прав адміністратора приймає необов’язковий обліковий запис; порожнє поле означає поточний обліковий запис і використовує вбудований SID групи Administrators.",
            "字段取决于操作：用户、组、描述、密码或日期。执行前请核对目标。授予管理员权限的操作可选填账户；留空表示当前账户，并使用内置 Administrators 组 SID。",
        ]),
        ("guide_simple", [
            "راجع الحسابات والمجموعات والهوية والجلسات المفتوحة قبل إجراء أي تغيير.",
            "Konten, Gruppen, Identität und offene Sitzungen prüfen, bevor Änderungen vorgenommen werden.",
            "Review accounts, groups, identity and open sessions before making a change.",
            "Consulta cuentas, grupos, identidad y sesiones antes de editar.",
            "Consultez les comptes, groupes, l’identité et les sessions ouvertes avant toute modification.",
            "बदलाव करने से पहले खाते, समूह, पहचान और खुले सत्र देखें।",
            "Esamina account, gruppi, identità e sessioni aperte prima di apportare modifiche.",
            "変更前にアカウント、グループ、ID、開いているセッションを確認します。",
            "변경하기 전에 계정, 그룹, ID 및 열린 세션을 확인하세요.",
            "Przed zmianą sprawdź konta, grupy, tożsamość i otwarte sesje.",
            "Consulte contas, grupos, identidade e sessões abertas antes de editar.",
            "Verificați conturile, grupurile, identitatea și sesiunile deschise înainte de modificări.",
            "Перед изменением проверьте учетные записи, группы, идентификатор и открытые сеансы.",
            "Перед зміною перевірте облікові записи, групи, особу та відкриті сеанси.",
            "更改前先检查账户、组、身份和已打开的会话。",
        ]),
        ("guide_complex", [
            "اختر الإجراء، وأدخل الهدف بدقة، وراجع تأكيد UAC عند ظهوره، ثم أعد الاستعلام للتحقق. TrustedInstaller هوية خدمة وليس مجموعة مستخدمين.",
            "Aktion wählen, genaues Ziel eingeben, eine UAC-Bestätigung prüfen und den Status erneut abfragen. TrustedInstaller ist eine Dienstidentität, keine Benutzergruppe.",
            "Choose an action, enter the exact target, review any UAC prompt, then query again to verify. TrustedInstaller is a service identity, not a user group.",
            "Selecciona la acción, completa el objetivo exacto, revisa la confirmación UAC cuando proceda y vuelve a consultar el estado. TrustedInstaller es una identidad de servicio, no un grupo de usuarios.",
            "Choisissez une action, indiquez la cible exacte, vérifiez toute confirmation UAC, puis relancez une consultation. TrustedInstaller est une identité de service, pas un groupe d’utilisateurs.",
            "कार्रवाई चुनें, सटीक लक्ष्य दें, UAC पुष्टि की समीक्षा करें और सत्यापन के लिए फिर से स्थिति देखें। TrustedInstaller सेवा पहचान है, उपयोगकर्ता समूह नहीं।",
            "Scegli un’azione, indica la destinazione esatta, verifica la richiesta UAC e poi controlla di nuovo lo stato. TrustedInstaller è un’identità di servizio, non un gruppo di utenti.",
            "操作を選び、正確な対象を入力し、UAC確認を確認してから再照会します。TrustedInstaller はサービス ID であり、ユーザーグループではありません。",
            "작업을 선택하고 정확한 대상을 입력한 뒤 UAC 확인을 검토하고 다시 조회해 확인하세요. TrustedInstaller는 서비스 ID이지 사용자 그룹이 아닙니다.",
            "Wybierz działanie, podaj dokładny cel, sprawdź potwierdzenie UAC i ponownie odczytaj stan. TrustedInstaller to tożsamość usługi, a nie grupa użytkowników.",
            "Escolha a ação, indique o destino exato, reveja a confirmação UAC e consulte novamente para verificar. TrustedInstaller é uma identidade de serviço, não um grupo de utilizadores.",
            "Alegeți acțiunea, introduceți ținta exactă, verificați confirmarea UAC și consultați din nou starea. TrustedInstaller este o identitate de serviciu, nu un grup de utilizatori.",
            "Выберите действие, укажите точную цель, проверьте запрос UAC и повторно запросите состояние. TrustedInstaller — это идентификатор службы, а не группа пользователей.",
            "Виберіть дію, вкажіть точну ціль, перегляньте підтвердження UAC і повторно перевірте стан. TrustedInstaller — це обліковий запис служби, а не група користувачів.",
            "选择操作并输入准确目标，检查 UAC 确认，然后重新查询以验证。TrustedInstaller 是服务身份，不是用户组。",
        ]),
        ("guide_fields_linux", [
            "تختلف الحقول حسب الإجراء: حساب محلي، وصف، مجلد منزلي، shell، مجموعات، UID، انتهاء الصلاحية وخيارات إضافية. تُدخل كلمة المرور مرتين ولا توضع في الوسائط.",
            "Die Felder hängen von der Aktion ab: lokales Konto, Beschreibung, Home-Verzeichnis, Shell, Gruppen, UID, Ablauf und optionale Einstellungen. Das Kennwort wird zweimal eingegeben und nie als Argument übergeben.",
            "Fields vary by action: local account, description, home directory, shell, groups, UID, expiry and optional flags. Enter a password twice; it is never passed as an argument.",
            "Los campos dependen de la acción: cuenta local, descripción, directorio personal, shell, grupos, UID, caducidad y opciones. La contraseña se introduce dos veces y nunca se pasa como argumento.",
            "Les champs varient selon l’action : compte local, description, dossier personnel, shell, groupes, UID, expiration et options. Saisissez le mot de passe deux fois ; il n’est jamais transmis comme argument.",
            "कार्रवाई के अनुसार फ़ील्ड बदलते हैं: स्थानीय खाता, विवरण, होम निर्देशिका, शेल, समूह, UID, समाप्ति और वैकल्पिक विकल्प। पासवर्ड दो बार दर्ज करें; इसे तर्क के रूप में कभी नहीं भेजा जाता।",
            "I campi variano in base all’azione: account locale, descrizione, directory home, shell, gruppi, UID, scadenza e opzioni. Inserisci la password due volte; non viene mai passata come argomento.",
            "項目は操作によって異なります（ローカルアカウント、説明、ホームディレクトリ、シェル、グループ、UID、有効期限、オプション）。パスワードは2回入力し、引数として渡しません。",
            "작업에 따라 로컬 계정, 설명, 홈 디렉터리, 셸, 그룹, UID, 만료 및 선택 항목을 입력합니다. 암호는 두 번 입력하며 인수로 전달되지 않습니다.",
            "Pola zależą od działania: konto lokalne, opis, katalog domowy, powłoka, grupy, UID, wygaśnięcie i opcje. Hasło wpisuje się dwukrotnie i nigdy nie jest przekazywane jako argument.",
            "Os campos variam conforme a ação: conta local, descrição, diretório pessoal, shell, grupos, UID, validade e opções. Introduza a palavra-passe duas vezes; nunca é passada como argumento.",
            "Câmpurile variază în funcție de acțiune: cont local, descriere, director personal, shell, grupuri, UID, expirare și opțiuni. Introduceți parola de două ori; nu este transmisă ca argument.",
            "Поля зависят от действия: локальная учетная запись, описание, домашний каталог, оболочка, группы, UID, срок действия и параметры. Пароль вводится дважды и не передается как аргумент.",
            "Поля залежать від дії: локальний обліковий запис, опис, домашній каталог, оболонка, групи, UID, термін дії та параметри. Пароль вводиться двічі й не передається як аргумент.",
            "字段因操作而异：本地账户、描述、主目录、shell、组、UID、到期时间和可选项。密码需输入两次，绝不会作为参数传递。",
        ]),
        ("guide_complex_linux", [
            "اختر الإجراء والحساب أو المجموعة المحلية بدقة، وراجع التأكيد وطلب الصلاحيات، ثم أعد الاستعلام للتحقق. أضف المسؤول فقط إلى مجموعة sudo أو wheel أو admin موجودة، وراجع قواعد sudoers.",
            "Aktion und genaues lokales Konto oder Gruppe wählen, Bestätigung und Berechtigungsabfrage prüfen und den Status erneut abfragen. Administratorzugriff nur über eine vorhandene sudo-, wheel- oder admin-Gruppe vergeben; sudoers-Regeln prüfen.",
            "Choose the action and exact local account or group, review confirmation and privilege prompts, then query again to verify. Grant admin access only through an existing sudo, wheel or admin group; review sudoers rules.",
            "Elige la acción y la cuenta o grupo local exacto, revisa la confirmación y la solicitud de permisos, y vuelve a consultar para verificar. Concede administración solo mediante un grupo sudo, wheel o admin existente; revisa sudoers.",
            "Choisissez l’action et le compte ou groupe local exacts, vérifiez la confirmation et la demande d’autorisation, puis relancez une consultation. N’accordez les droits admin que via un groupe sudo, wheel ou admin existant ; vérifiez sudoers.",
            "कार्रवाई और सटीक स्थानीय खाता या समूह चुनें, पुष्टि और अनुमति अनुरोध देखें, फिर सत्यापन के लिए दोबारा जाँचें। व्यवस्थापक पहुँच केवल मौजूदा sudo, wheel या admin समूह से दें; sudoers नियम देखें।",
            "Scegli l’azione e l’account o gruppo locale esatto, verifica la conferma e la richiesta di privilegi, poi controlla di nuovo. Concedi l’accesso admin solo tramite un gruppo sudo, wheel o admin esistente; verifica le regole sudoers.",
            "操作と正確なローカルアカウントまたはグループを選び、確認と権限要求を確認してから再照会します。管理者権限は既存の sudo、wheel、admin グループ経由でのみ付与し、sudoers の規則を確認してください。",
            "작업과 정확한 로컬 계정 또는 그룹을 선택하고 확인 및 권한 요청을 검토한 뒤 다시 조회하세요. 관리자 권한은 기존 sudo, wheel 또는 admin 그룹을 통해서만 부여하고 sudoers 규칙을 확인하세요.",
            "Wybierz działanie oraz dokładne konto lub grupę lokalną, sprawdź potwierdzenie i prośbę o uprawnienia, a potem ponownie odczytaj stan. Dostęp administratora nadaj wyłącznie przez istniejącą grupę sudo, wheel lub admin; sprawdź reguły sudoers.",
            "Escolha a ação e a conta ou grupo local exato, reveja a confirmação e o pedido de permissões e consulte novamente para verificar. Conceda administração apenas através de um grupo sudo, wheel ou admin existente; reveja as regras sudoers.",
            "Alegeți acțiunea și contul sau grupul local exact, verificați confirmarea și solicitarea de privilegii, apoi consultați din nou starea. Acordați acces de administrator numai printr-un grup sudo, wheel sau admin existent; verificați regulile sudoers.",
            "Выберите действие и точную локальную учетную запись или группу, проверьте подтверждение и запрос прав, затем повторно запросите состояние. Права администратора выдавайте только через существующую группу sudo, wheel или admin; проверьте правила sudoers.",
            "Виберіть дію та точний локальний обліковий запис або групу, перегляньте підтвердження й запит прав, а потім повторно перевірте стан. Права адміністратора надавайте лише через наявну групу sudo, wheel або admin; перевірте правила sudoers.",
            "选择操作和准确的本地账户或组，检查确认与权限提示，然后重新查询验证。仅通过现有 sudo、wheel 或 admin 组授予管理员权限；检查 sudoers 规则。",
        ]),
    ];
    let language = current();
    let index = SUPPORTED
        .iter()
        .position(|candidate| *candidate == language)
        .unwrap_or(3);
    TEXT.iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, values)| values[index])
        .unwrap_or("")
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
        match current() {
            "en" => "Inspect Windows Registry",
            "de" => "Windows-Registrierung prüfen",
            "fr" => "Inspecter le registre Windows",
            "pt" => "Inspecionar o Registo do Windows",
            "it" => "Ispeziona il Registro di Windows",
            "ca" => "Inspeccionar el Registre de Windows",
            "nl" => "Windows-register inspecteren",
            "pl" => "Inspekcja rejestru Windows",
            _ => "Inspeccionar el Registro de Windows",
        }
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
            "en" => "Linux disks, mounts and partitions, including guided parted, filesystem and volume operations",
            "de" => "Linux-Datenträger, Mounts und Partitionen (lsblk/parted/gparted)",
            "fr" => "Disques, montages et partitions Linux (lsblk/parted/gparted)",
            "pt" => "Discos, montagens e partições Linux (lsblk/parted/gparted)",
            "it" => "Dischi, mount e partizioni Linux (lsblk/parted/gparted)",
            "ca" => "Discs, muntatges i particions de Linux (lsblk/parted/gparted)",
            "nl" => "Linux-schijven, mounts en partities (lsblk/parted/gparted)",
            "pl" => "Dyski, montowania i partycje Linuksa (lsblk/parted/gparted)",
            _ => "Discos, montajes y particiones Linux con operaciones guiadas de parted, sistemas de archivos y volúmenes",
        }
    }
}

/// Acciones cortas del gestor de discos. Se usan tanto en la GUI como en
/// cualquier frontend que reutilice el catálogo de LTools.
pub fn storage_action_text(key: &str) -> &'static str {
    if matches!(
        key,
        "filesystem_format_check" | "mount_swap" | "luks_headers" | "volume_layers"
    ) {
        return match (current(), key) {
            ("en", "filesystem_format_check") => "Formatting and checks",
            ("en", "mount_swap") => "Mounting and swap",
            ("en", "luks_headers") => "LUKS: encryption and headers",
            ("en", "volume_layers") => "Volume layers",
            ("de", "filesystem_format_check") => "Formatierung und Prüfungen",
            ("de", "mount_swap") => "Mounts und Swap",
            ("de", "luks_headers") => "LUKS: Verschlüsselung und Header",
            ("de", "volume_layers") => "Volume-Ebenen",
            ("fr", "filesystem_format_check") => "Formatage et vérifications",
            ("fr", "mount_swap") => "Montages et swap",
            ("fr", "luks_headers") => "LUKS : chiffrement et en-têtes",
            ("fr", "volume_layers") => "Couches de volumes",
            ("pt", "filesystem_format_check") => "Formatação e verificações",
            ("pt", "mount_swap") => "Montagens e swap",
            ("pt", "luks_headers") => "LUKS: encriptação e cabeçalhos",
            ("pt", "volume_layers") => "Camadas de volume",
            ("it", "filesystem_format_check") => "Formattazione e controlli",
            ("it", "mount_swap") => "Mount e swap",
            ("it", "luks_headers") => "LUKS: cifratura e intestazioni",
            ("it", "volume_layers") => "Livelli dei volumi",
            ("ca", "filesystem_format_check") => "Formatació i comprovacions",
            ("ca", "mount_swap") => "Muntatges i swap",
            ("ca", "luks_headers") => "LUKS: xifratge i capçaleres",
            ("ca", "volume_layers") => "Capes de volum",
            ("nl", "filesystem_format_check") => "Formatteren en controles",
            ("nl", "mount_swap") => "Mounts en swap",
            ("nl", "luks_headers") => "LUKS: versleuteling en headers",
            ("nl", "volume_layers") => "Volumelagen",
            ("pl", "filesystem_format_check") => "Formatowanie i sprawdzanie",
            ("pl", "mount_swap") => "Montowania i swap",
            ("pl", "luks_headers") => "LUKS: szyfrowanie i nagłówki",
            ("pl", "volume_layers") => "Warstwy woluminów",
            ("ar", "filesystem_format_check") => "التهيئة والفحوصات",
            ("ar", "mount_swap") => "نقاط التحميل وSwap",
            ("ar", "luks_headers") => "LUKS: التشفير والرؤوس",
            ("ar", "volume_layers") => "طبقات وحدات التخزين",
            ("hi", "filesystem_format_check") => "फ़ॉर्मेट और जाँच",
            ("hi", "mount_swap") => "माउंट और स्वैप",
            ("hi", "luks_headers") => "LUKS: एन्क्रिप्शन और हेडर",
            ("hi", "volume_layers") => "वॉल्यूम परतें",
            ("ja", "filesystem_format_check") => "フォーマットと検査",
            ("ja", "mount_swap") => "マウントとスワップ",
            ("ja", "luks_headers") => "LUKS: 暗号化とヘッダー",
            ("ja", "volume_layers") => "ボリューム層",
            ("ko", "filesystem_format_check") => "포맷 및 검사",
            ("ko", "mount_swap") => "마운트 및 스왑",
            ("ko", "luks_headers") => "LUKS: 암호화 및 헤더",
            ("ko", "volume_layers") => "볼륨 계층",
            ("ro", "filesystem_format_check") => "Formatare și verificare",
            ("ro", "mount_swap") => "Montări și swap",
            ("ro", "luks_headers") => "LUKS: criptare și antete",
            ("ro", "volume_layers") => "Straturi de volume",
            ("ru", "filesystem_format_check") => "Форматирование и проверка",
            ("ru", "mount_swap") => "Подключение и swap",
            ("ru", "luks_headers") => "LUKS: шифрование и заголовки",
            ("ru", "volume_layers") => "Слои томов",
            ("uk", "filesystem_format_check") => "Форматування та перевірка",
            ("uk", "mount_swap") => "Монтування та swap",
            ("uk", "luks_headers") => "LUKS: шифрування та заголовки",
            ("uk", "volume_layers") => "Шари томів",
            ("zh", "filesystem_format_check") => "格式化和检查",
            ("zh", "mount_swap") => "挂载和交换空间",
            ("zh", "luks_headers") => "LUKS：加密和头部",
            ("zh", "volume_layers") => "卷层",
            (_, "filesystem_format_check") => "Formato y comprobación",
            (_, "mount_swap") => "Montaje y swap",
            (_, "luks_headers") => "LUKS: cifrado y cabeceras",
            (_, "volume_layers") => "Capas de volumen",
            _ => "",
        };
    }
    match (current(), key) {
        ("en", "status") => "Space and mounts overview",
        ("en", "partitions") => "Disks and partitions",
        ("en", "mounts") => "Active mounts",
        ("en", "usage") => "Space usage by volume",
        ("en", "pools") => "Storage Spaces and virtual disks",
        ("en", "bitlocker") => "BitLocker status",
        ("en", "tools") => "Detected storage tools",
        ("en", "manager") => "Open native partition manager",
        ("en", "clean") => "Review cleanup",
        ("en", "guide") => "Partitioning guide and safety rules",
        ("en", "map") => "Interactive disk and path map",
        ("de", "status") => "Übersicht über Speicher und Mounts",
        ("de", "partitions") => "Datenträger und Partitionen",
        ("de", "mounts") => "Aktive Mounts",
        ("de", "usage") => "Speicherplatz nach Volume",
        ("de", "pools") => "Speicherplätze und virtuelle Datenträger",
        ("de", "bitlocker") => "BitLocker-Status",
        ("de", "tools") => "Erkannte Speicherwerkzeuge",
        ("de", "manager") => "Nativen Partitionsmanager öffnen",
        ("de", "clean") => "Bereinigung prüfen",
        ("de", "guide") => "Partitionsleitfaden und Sicherheitsregeln",
        ("fr", "status") => "Vue d’ensemble de l’espace et des montages",
        ("fr", "partitions") => "Disques et partitions",
        ("fr", "mounts") => "Montages actifs",
        ("fr", "usage") => "Utilisation de l’espace par volume",
        ("fr", "pools") => "Espaces de stockage et disques virtuels",
        ("fr", "bitlocker") => "État de BitLocker",
        ("fr", "tools") => "Outils de stockage détectés",
        ("fr", "manager") => "Ouvrir le gestionnaire natif",
        ("fr", "clean") => "Vérifier le nettoyage",
        ("fr", "guide") => "Guide du partitionnement et règles de sécurité",
        ("pt", "status") => "Resumo de espaço e montagens",
        ("pt", "partitions") => "Discos e partições",
        ("pt", "mounts") => "Montagens ativas",
        ("pt", "usage") => "Uso de espaço por volume",
        ("pt", "pools") => "Espaços de armazenamento e discos virtuais",
        ("pt", "bitlocker") => "Estado do BitLocker",
        ("pt", "tools") => "Ferramentas de armazenamento detetadas",
        ("pt", "manager") => "Abrir gestor nativo de partições",
        ("pt", "clean") => "Rever limpeza",
        ("pt", "guide") => "Guia de particionamento e regras de segurança",
        ("it", "status") => "Riepilogo spazio e mount",
        ("it", "partitions") => "Dischi e partizioni",
        ("it", "mounts") => "Mount attivi",
        ("it", "usage") => "Uso dello spazio per volume",
        ("it", "pools") => "Spazi di archiviazione e dischi virtuali",
        ("it", "bitlocker") => "Stato di BitLocker",
        ("it", "tools") => "Strumenti di archiviazione rilevati",
        ("it", "manager") => "Apri il gestore nativo delle partizioni",
        ("it", "clean") => "Controlla pulizia",
        ("it", "guide") => "Guida al partizionamento e regole di sicurezza",
        ("ca", "status") => "Resum d’espai i muntatges",
        ("ca", "partitions") => "Discs i particions",
        ("ca", "mounts") => "Muntatges actius",
        ("ca", "usage") => "Ús de l’espai per volum",
        ("ca", "pools") => "Espais d’emmagatzematge i discs virtuals",
        ("ca", "bitlocker") => "Estat de BitLocker",
        ("ca", "tools") => "Eines d’emmagatzematge detectades",
        ("ca", "manager") => "Obrir el gestor natiu de particions",
        ("ca", "clean") => "Revisar la neteja",
        ("ca", "guide") => "Guia de particions i regles de seguretat",
        ("nl", "status") => "Overzicht van ruimte en mounts",
        ("nl", "partitions") => "Schijven en partities",
        ("nl", "mounts") => "Actieve mounts",
        ("nl", "usage") => "Ruimtegebruik per volume",
        ("nl", "pools") => "Opslagruimten en virtuele schijven",
        ("nl", "bitlocker") => "BitLocker-status",
        ("nl", "tools") => "Gedetecteerde opslagtools",
        ("nl", "manager") => "Native partitiebeheerder openen",
        ("nl", "clean") => "Opschoning controleren",
        ("nl", "guide") => "Handleiding voor partitioneren en veiligheidsregels",
        ("pl", "status") => "Przegląd miejsca i montowań",
        ("pl", "partitions") => "Dyski i partycje",
        ("pl", "mounts") => "Aktywne montowania",
        ("pl", "usage") => "Wykorzystanie miejsca według woluminu",
        ("pl", "pools") => "Miejsca do magazynowania i dyski wirtualne",
        ("pl", "bitlocker") => "Stan funkcji BitLocker",
        ("pl", "tools") => "Wykryte narzędzia pamięci masowej",
        ("pl", "manager") => "Otwórz natywny menedżer partycji",
        ("pl", "clean") => "Sprawdź czyszczenie",
        ("pl", "guide") => "Przewodnik partycjonowania i zasady bezpieczeństwa",
        ("ar", "status") => "ملخص المساحة ونقاط التحميل",
        ("ar", "partitions") => "الأقراص والأقسام",
        ("ar", "mounts") => "نقاط التحميل النشطة",
        ("ar", "usage") => "استخدام المساحة حسب وحدة التخزين",
        ("ar", "pools") => "مساحات التخزين والأقراص الافتراضية",
        ("ar", "bitlocker") => "حالة BitLocker",
        ("ar", "tools") => "أدوات التخزين المكتشفة",
        ("ar", "manager") => "فتح مدير الأقسام الأصلي",
        ("ar", "clean") => "مراجعة التنظيف",
        ("ar", "guide") => "دليل التقسيم وقواعد الأمان",
        ("hi", "status") => "स्थान और माउंट का सारांश",
        ("hi", "partitions") => "डिस्क और पार्टीशन",
        ("hi", "mounts") => "सक्रिय माउंट",
        ("hi", "usage") => "वॉल्यूम के अनुसार स्थान उपयोग",
        ("hi", "pools") => "स्टोरेज स्पेस और वर्चुअल डिस्क",
        ("hi", "bitlocker") => "BitLocker स्थिति",
        ("hi", "tools") => "पता चले स्टोरेज टूल",
        ("hi", "manager") => "मूल पार्टीशन प्रबंधक खोलें",
        ("hi", "clean") => "क्लीनअप की समीक्षा करें",
        ("hi", "guide") => "पार्टीशन गाइड और सुरक्षा नियम",
        ("ja", "status") => "容量とマウントの概要",
        ("ja", "partitions") => "ディスクとパーティション",
        ("ja", "mounts") => "アクティブなマウント",
        ("ja", "usage") => "ボリューム別の容量使用量",
        ("ja", "pools") => "記憶域と仮想ディスク",
        ("ja", "bitlocker") => "BitLocker の状態",
        ("ja", "tools") => "検出されたストレージツール",
        ("ja", "manager") => "標準のパーティション管理ツールを開く",
        ("ja", "clean") => "クリーンアップを確認",
        ("ja", "guide") => "パーティションガイドと安全規則",
        ("ko", "status") => "공간 및 마운트 요약",
        ("ko", "partitions") => "디스크 및 파티션",
        ("ko", "mounts") => "활성 마운트",
        ("ko", "usage") => "볼륨별 공간 사용량",
        ("ko", "pools") => "저장소 공간 및 가상 디스크",
        ("ko", "bitlocker") => "BitLocker 상태",
        ("ko", "tools") => "감지된 저장소 도구",
        ("ko", "manager") => "기본 파티션 관리자 열기",
        ("ko", "clean") => "정리 검토",
        ("ko", "guide") => "파티션 안내 및 안전 규칙",
        ("ro", "status") => "Rezumat spațiu și montări",
        ("ro", "partitions") => "Discuri și partiții",
        ("ro", "mounts") => "Montări active",
        ("ro", "usage") => "Utilizarea spațiului pe volum",
        ("ro", "pools") => "Spații de stocare și discuri virtuale",
        ("ro", "bitlocker") => "Starea BitLocker",
        ("ro", "tools") => "Instrumente de stocare detectate",
        ("ro", "manager") => "Deschide managerul nativ de partiții",
        ("ro", "clean") => "Verifică curățarea",
        ("ro", "guide") => "Ghid de partiționare și reguli de siguranță",
        ("ru", "status") => "Обзор места и подключений",
        ("ru", "partitions") => "Диски и разделы",
        ("ru", "mounts") => "Активные подключения",
        ("ru", "usage") => "Использование места по томам",
        ("ru", "pools") => "Дисковые пространства и виртуальные диски",
        ("ru", "bitlocker") => "Состояние BitLocker",
        ("ru", "tools") => "Найденные инструменты хранения",
        ("ru", "manager") => "Открыть штатный менеджер разделов",
        ("ru", "clean") => "Проверить очистку",
        ("ru", "guide") => "Руководство по разделам и правила безопасности",
        ("uk", "status") => "Огляд місця та монтувань",
        ("uk", "partitions") => "Диски та розділи",
        ("uk", "mounts") => "Активні монтування",
        ("uk", "usage") => "Використання місця за томами",
        ("uk", "pools") => "Дискові простори та віртуальні диски",
        ("uk", "bitlocker") => "Стан BitLocker",
        ("uk", "tools") => "Виявлені інструменти сховища",
        ("uk", "manager") => "Відкрити штатний менеджер розділів",
        ("uk", "clean") => "Перевірити очищення",
        ("uk", "guide") => "Посібник із розділів і правила безпеки",
        ("zh", "status") => "空间和挂载概览",
        ("zh", "partitions") => "磁盘和分区",
        ("zh", "mounts") => "活动挂载",
        ("zh", "usage") => "按卷查看空间使用情况",
        ("zh", "pools") => "存储空间和虚拟磁盘",
        ("zh", "bitlocker") => "BitLocker 状态",
        ("zh", "tools") => "检测到的存储工具",
        ("zh", "manager") => "打开原生分区管理器",
        ("zh", "clean") => "检查清理",
        ("zh", "guide") => "分区指南和安全规则",
        (_, "status") => "Resumen de espacio y montajes",
        (_, "partitions") => "Discos y particiones",
        (_, "mounts") => "Montajes activos",
        (_, "usage") => "Uso de espacio por volumen",
        (_, "pools") => "Espacios de almacenamiento y discos virtuales",
        (_, "bitlocker") => "Estado de BitLocker",
        (_, "tools") => "Herramientas detectadas",
        (_, "manager") => "Abrir gestor nativo de particiones",
        (_, "clean") => "Revisar limpieza",
        (_, "guide") => "Guía de particionado y protecciones",
        (_, "map") => "Mapa interactivo de discos y rutas",
        _ => "",
    }
}

/// Separadores de las páginas avanzadas del gestor de almacenamiento. Estos
/// textos también forman parte del contrato visible de la GUI: no deben
/// volver a quedar hardcodeados en español cuando cambia el idioma.
#[allow(dead_code)]
pub fn storage_section_text(key: &str) -> &'static str {
    let labels: &[(&str, &str)] = match current() {
        "en" => &[
            ("query_map", "Inspection and map"),
            ("file_paths", "Files and paths"),
            ("disk_volumes", "Disks and volumes"),
            ("tools_cleanup", "Tools and cleanup"),
            ("partition_inspection", "Inspection and diagnosis"),
            ("partition_tables", "Tables and partitions"),
            ("partition_recovery", "Flags and recovery"),
            ("partition_destruction", "Explicit destruction"),
        ],
        "de" => &[
            ("query_map", "Prüfung und Karte"),
            ("file_paths", "Dateien und Pfade"),
            ("disk_volumes", "Datenträger und Volumes"),
            ("tools_cleanup", "Werkzeuge und Bereinigung"),
            ("partition_inspection", "Prüfung und Diagnose"),
            ("partition_tables", "Tabellen und Partitionen"),
            ("partition_recovery", "Flags und Wiederherstellung"),
            ("partition_destruction", "Explizite Zerstörung"),
        ],
        "fr" => &[
            ("query_map", "Inspection et carte"),
            ("file_paths", "Fichiers et chemins"),
            ("disk_volumes", "Disques et volumes"),
            ("tools_cleanup", "Outils et nettoyage"),
            ("partition_inspection", "Inspection et diagnostic"),
            ("partition_tables", "Tables et partitions"),
            ("partition_recovery", "Indicateurs et récupération"),
            ("partition_destruction", "Destruction explicite"),
        ],
        "pt" => &[
            ("query_map", "Consulta e mapa"),
            ("file_paths", "Ficheiros e caminhos"),
            ("disk_volumes", "Discos e volumes"),
            ("tools_cleanup", "Ferramentas e limpeza"),
            ("partition_inspection", "Consulta e diagnóstico"),
            ("partition_tables", "Tabelas e partições"),
            ("partition_recovery", "Flags e recuperação"),
            ("partition_destruction", "Destruição explícita"),
        ],
        "it" => &[
            ("query_map", "Controllo e mappa"),
            ("file_paths", "File e percorsi"),
            ("disk_volumes", "Dischi e volumi"),
            ("tools_cleanup", "Strumenti e pulizia"),
            ("partition_inspection", "Controllo e diagnosi"),
            ("partition_tables", "Tabelle e partizioni"),
            ("partition_recovery", "Flag e ripristino"),
            ("partition_destruction", "Distruzione esplicita"),
        ],
        "ca" => &[
            ("query_map", "Consulta i mapa"),
            ("file_paths", "Fitxers i camins"),
            ("disk_volumes", "Discs i volums"),
            ("tools_cleanup", "Eines i neteja"),
            ("partition_inspection", "Consulta i diagnòstic"),
            ("partition_tables", "Taules i particions"),
            ("partition_recovery", "Flags i recuperació"),
            ("partition_destruction", "Destrucció explícita"),
        ],
        "nl" => &[
            ("query_map", "Inspectie en kaart"),
            ("file_paths", "Bestanden en paden"),
            ("disk_volumes", "Schijven en volumes"),
            ("tools_cleanup", "Hulpmiddelen en opschoning"),
            ("partition_inspection", "Inspectie en diagnose"),
            ("partition_tables", "Tabellen en partities"),
            ("partition_recovery", "Vlaggen en herstel"),
            ("partition_destruction", "Expliciete vernietiging"),
        ],
        "pl" => &[
            ("query_map", "Inspekcja i mapa"),
            ("file_paths", "Pliki i ścieżki"),
            ("disk_volumes", "Dyski i woluminy"),
            ("tools_cleanup", "Narzędzia i czyszczenie"),
            ("partition_inspection", "Inspekcja i diagnostyka"),
            ("partition_tables", "Tablice i partycje"),
            ("partition_recovery", "Flagi i odzyskiwanie"),
            ("partition_destruction", "Jawne niszczenie"),
        ],
        "ar" => &[
            ("query_map", "الفحص والخريطة"),
            ("file_paths", "الملفات والمسارات"),
            ("disk_volumes", "الأقراص ووحدات التخزين"),
            ("tools_cleanup", "الأدوات والتنظيف"),
            ("partition_inspection", "الفحص والتشخيص"),
            ("partition_tables", "الجداول والأقسام"),
            ("partition_recovery", "العلامات والاسترداد"),
            ("partition_destruction", "الإتلاف الصريح"),
        ],
        "hi" => &[
            ("query_map", "जाँच और मानचित्र"),
            ("file_paths", "फ़ाइलें और पथ"),
            ("disk_volumes", "डिस्क और वॉल्यूम"),
            ("tools_cleanup", "टूल और सफाई"),
            ("partition_inspection", "जाँच और निदान"),
            ("partition_tables", "तालिकाएँ और पार्टीशन"),
            ("partition_recovery", "फ़्लैग और पुनर्प्राप्ति"),
            ("partition_destruction", "स्पष्ट विनाश"),
        ],
        "ja" => &[
            ("query_map", "検査とマップ"),
            ("file_paths", "ファイルとパス"),
            ("disk_volumes", "ディスクとボリューム"),
            ("tools_cleanup", "ツールとクリーンアップ"),
            ("partition_inspection", "検査と診断"),
            ("partition_tables", "テーブルとパーティション"),
            ("partition_recovery", "フラグと復旧"),
            ("partition_destruction", "明示的な破壊"),
        ],
        "ko" => &[
            ("query_map", "검사 및 지도"),
            ("file_paths", "파일 및 경로"),
            ("disk_volumes", "디스크 및 볼륨"),
            ("tools_cleanup", "도구 및 정리"),
            ("partition_inspection", "검사 및 진단"),
            ("partition_tables", "테이블 및 파티션"),
            ("partition_recovery", "플래그 및 복구"),
            ("partition_destruction", "명시적 삭제"),
        ],
        "ro" => &[
            ("query_map", "Inspecție și hartă"),
            ("file_paths", "Fișiere și căi"),
            ("disk_volumes", "Discuri și volume"),
            ("tools_cleanup", "Instrumente și curățare"),
            ("partition_inspection", "Inspecție și diagnosticare"),
            ("partition_tables", "Tabele și partiții"),
            ("partition_recovery", "Indicatoare și recuperare"),
            ("partition_destruction", "Ștergere explicită"),
        ],
        "ru" => &[
            ("query_map", "Проверка и карта"),
            ("file_paths", "Файлы и пути"),
            ("disk_volumes", "Диски и тома"),
            ("tools_cleanup", "Инструменты и очистка"),
            ("partition_inspection", "Проверка и диагностика"),
            ("partition_tables", "Таблицы и разделы"),
            ("partition_recovery", "Флаги и восстановление"),
            ("partition_destruction", "Явное уничтожение"),
        ],
        "uk" => &[
            ("query_map", "Перевірка та карта"),
            ("file_paths", "Файли та шляхи"),
            ("disk_volumes", "Диски та томи"),
            ("tools_cleanup", "Інструменти та очищення"),
            ("partition_inspection", "Перевірка та діагностика"),
            ("partition_tables", "Таблиці та розділи"),
            ("partition_recovery", "Прапорці та відновлення"),
            ("partition_destruction", "Явне знищення"),
        ],
        "zh" => &[
            ("query_map", "检查和地图"),
            ("file_paths", "文件和路径"),
            ("disk_volumes", "磁盘和卷"),
            ("tools_cleanup", "工具和清理"),
            ("partition_inspection", "检查和诊断"),
            ("partition_tables", "分区表和分区"),
            ("partition_recovery", "标志和恢复"),
            ("partition_destruction", "明确销毁"),
        ],
        _ => &[
            ("query_map", "Consulta y mapa"),
            ("file_paths", "Archivos y rutas"),
            ("disk_volumes", "Discos y volúmenes"),
            ("tools_cleanup", "Herramientas y limpieza"),
            ("partition_inspection", "Consulta y diagnóstico"),
            ("partition_tables", "Tablas y particiones"),
            ("partition_recovery", "Flags y recuperación"),
            ("partition_destruction", "Destrucción explícita"),
        ],
    };
    labels
        .iter()
        .find_map(|(label_key, value)| (*label_key == key).then_some(*value))
        .unwrap_or("")
}

/// Textos del diálogo interactivo del mapa. El diálogo se crea desde el
/// backend GTK, así que sus botones también deben respetar el idioma activo.
pub fn storage_map_text(key: &str) -> &'static str {
    const MAP_TEXT: &[(&str, [&str; 15])] = &[
        (
            "hint",
            [
                "تُحسَب الخريطة في الخلفية؛ استخدم الأسهم لفتح المجلدات. تظل المسارات المحمية وأخطاء الأذونات ظاهرة.",
                "Die Karte wird im Hintergrund berechnet; Ordner lassen sich mit den Pfeilen öffnen. Geschützte Pfade und Berechtigungsfehler bleiben sichtbar.",
                "The map runs in the background; use the arrows to open folders. Protected paths and permission errors remain visible.",
                "El mapa se calcula en segundo plano; usa las flechas para abrir carpetas. Se conservan las rutas protegidas y los errores de permisos.",
                "La carte est calculée en arrière-plan ; utilisez les flèches pour ouvrir les dossiers. Les chemins protégés et les erreurs de permissions restent visibles.",
                "मानचित्र पृष्ठभूमि में चलता है; फ़ोल्डर खोलने के लिए तीरों का उपयोग करें। सुरक्षित पथ और अनुमति त्रुटियाँ दिखाई देती रहेंगी।",
                "La mappa viene calcolata in background; usa le frecce per aprire le cartelle. I percorsi protetti e gli errori di autorizzazione restano visibili.",
                "マップはバックグラウンドで計算されます。矢印でフォルダーを開けます。保護されたパスと権限エラーは表示されます。",
                "지도는 백그라운드에서 계산됩니다. 화살표로 폴더를 여세요. 보호된 경로와 권한 오류는 계속 표시됩니다.",
                "Mapa jest obliczana w tle; użyj strzałek, aby otwierać foldery. Chronione ścieżki i błędy uprawnień pozostają widoczne.",
                "O mapa é calculado em segundo plano; usa as setas para abrir pastas. Os caminhos protegidos e os erros de permissões permanecem visíveis.",
                "Harta este calculată în fundal; folosește săgețile pentru a deschide dosare. Căile protejate și erorile de permisiuni rămân vizibile.",
                "Карта рассчитывается в фоне; используйте стрелки, чтобы открывать папки. Защищённые пути и ошибки прав остаются видимыми.",
                "Карта обчислюється у фоновому режимі; використовуйте стрілки, щоб відкривати папки. Захищені шляхи та помилки прав залишаються видимими.",
                "地图在后台计算；使用箭头打开文件夹。受保护的路径和权限错误会继续显示。",
            ],
        ),
        (
            "column",
            [
                "المسار / الحجم / الحالة", "Pfad / Größe / Status", "Path / size / status", "Ruta / tamaño / estado", "Chemin / taille / état", "पथ / आकार / स्थिति", "Percorso / dimensione / stato", "パス / サイズ / 状態", "경로 / 크기 / 상태", "Ścieżka / rozmiar / stan", "Caminho / tamanho / estado", "Cale / dimensiune / stare", "Путь / размер / состояние", "Шлях / розмір / стан", "路径 / 大小 / 状态",
            ],
        ),
        (
            "expand", ["توسيع","Erweitern","Expand","Expandir","Développer","विस्तार","Espandi","展開","펼치기","Rozwiń","Expandir","Extinde","Развернуть","Розгорнути","展开"]
        ),
        (
            "collapse", ["طي","Reduzieren","Collapse","Colapsar","Réduire","समेटें","Comprimi","折りたたむ","접기","Zwiń","Recolher","Restrânge","Свернуть","Згорнути","收起"]
        ),
        (
            "elevate", ["إعادة المحاولة كمسؤول","Als Administrator erneut versuchen","Retry as administrator","Reintentar como administrador","Réessayer en administrateur","व्यवस्थापक के रूप में पुनः प्रयास करें","Riprova come amministratore","管理者として再試行","관리자로 다시 시도","Ponów jako administrator","Tentar novamente como administrador","Reîncearcă drept administrator","Повторить с правами администратора","Повторити як адміністратор","以管理员身份重试"]
        ),
        (
            "copy", ["نسخ","Kopieren","Copy","Copiar","Copier","कॉपी","Copia","コピー","복사","Kopiuj","Copiar","Copiază","Копировать","Копіювати","复制"]
        ),
        (
            "move", ["نقل","Verschieben","Move","Mover","Déplacer","स्थानांतरित करें","Sposta","移動","이동","Przenieś","Mover","Mută","Переместить","Перемістити","移动"]
        ),
        (
            "delete", ["إرسال إلى سلة المهملات","In den Papierkorb","Send to trash","Enviar a papelera","Mettre à la corbeille","ट्रैश में भेजें","Sposta nel cestino","ゴミ箱へ移動","휴지통으로 보내기","Przenieś do kosza","Enviar para o lixo","Trimite în coș","Отправить в корзину","Надіслати до кошика","移至回收站"]
        ),
        (
            "close", ["إغلاق","Schließen","Close","Cerrar","Fermer","बंद करें","Chiudi","閉じる","닫기","Zamknij","Fechar","Închide","Закрыть","Закрити","关闭"]
        ),
        (
            "scan_initial",
            [
                "جارٍ فحص الأقراص والمسارات…", "Datenträger und Pfade werden geprüft…", "Scanning disks and paths…", "Analizando discos y rutas…", "Analyse des disques et des chemins…", "डिस्क और पथ स्कैन हो रहे हैं…", "Analisi di dischi e percorsi…", "ディスクとパスをスキャン中…", "디스크 및 경로 검사 중…", "Skanowanie dysków i ścieżek…", "A analisar discos e caminhos…", "Se scanează discurile și căile…", "Сканирование дисков и путей…", "Сканування дисків і шляхів…", "正在扫描磁盘和路径…",
            ],
        ),
        (
            "scan_progress",
            [
                "جارٍ فحص الأقراص والمسارات… تمت زيارة {visited} مسارًا. سيُحسب الإجمالي والمستخدم والمتاح لكل جذر.",
                "Datenträger und Pfade werden geprüft… {visited} Pfade besucht. Für jede Wurzel werden Gesamt-, belegter und verfügbarer Speicher berechnet.",
                "Scanning disks and paths… {visited} paths visited. Total, used, free and available space are calculated for each root.",
                "Analizando discos y rutas… {visited} rutas visitadas. Se calcularán el total, el espacio ocupado, el libre y el disponible de cada raíz.",
                "Analyse des disques et des chemins… {visited} chemins parcourus. Le total, l’espace utilisé, libre et disponible sont calculés pour chaque racine.",
                "डिस्क और पथ स्कैन हो रहे हैं… {visited} पथ देखे गए। हर रूट का कुल, उपयोग किया गया, खाली और उपलब्ध स्थान गिना जा रहा है।",
                "Analisi di dischi e percorsi… visitati {visited} percorsi. Per ogni radice si calcolano spazio totale, usato, libero e disponibile.",
                "ディスクとパスをスキャン中… {visited} 個のパスを確認しました。各ルートの合計、使用済み、空き、利用可能な容量を計算します。",
                "디스크 및 경로 검사 중… 경로 {visited}개를 확인했습니다. 각 루트의 전체, 사용, 여유 및 사용 가능 공간을 계산합니다.",
                "Skanowanie dysków i ścieżek… odwiedzono {visited} ścieżek. Obliczanie pojemności całkowitej, użytej, wolnej i dostępnej dla każdego katalogu głównego.",
                "A analisar discos e caminhos… {visited} caminhos visitados. A calcular o espaço total, usado, livre e disponível de cada raiz.",
                "Se scanează discurile și căile… au fost parcurse {visited} căi. Se calculează spațiul total, utilizat, liber și disponibil pentru fiecare rădăcină.",
                "Сканирование дисков и путей… просмотрено путей: {visited}. Для каждого корня вычисляются общий, занятый, свободный и доступный объёмы.",
                "Сканування дисків і шляхів… відвідано шляхів: {visited}. Для кожного кореня обчислюються загальний, зайнятий, вільний і доступний обсяги.",
                "正在扫描磁盘和路径…已访问 {visited} 个路径。正在计算每个根目录的总计、已用、空闲和可用空间。",
            ],
        ),
        (
            "scan_timeout",
            [
                "تجاوزت الخريطة الحد البالغ {seconds} ثانية بعد زيارة {visited} مسارًا. يمكنك الإغلاق وإعادة المحاولة بحدود مختلفة.",
                "Die Karte überschritt nach {visited} besuchten Pfaden das Zeitlimit von {seconds} Sekunden. Schließe sie und versuche es mit anderen Grenzen erneut.",
                "The map exceeded the {seconds}-second limit after visiting {visited} paths. Close it and retry with different limits.",
                "El mapa superó el límite de {seconds} segundos tras visitar {visited} rutas. Puedes cerrarlo e intentarlo con otros límites.",
                "La carte a dépassé la limite de {seconds} secondes après {visited} chemins. Fermez-la et réessayez avec d’autres limites.",
                "{visited} पथ देखने के बाद मानचित्र ने {seconds} सेकंड की सीमा पार कर दी। बंद करके अलग सीमाओं के साथ फिर प्रयास करें।",
                "La mappa ha superato il limite di {seconds} secondi dopo {visited} percorsi. Chiudila e riprova con limiti diversi.",
                "マップは {visited} 個のパスを確認した後、{seconds} 秒の制限を超えました。閉じて別の上限で再試行してください。",
                "경로 {visited}개를 확인한 후 지도가 {seconds}초 제한을 초과했습니다. 닫은 다음 다른 제한으로 다시 시도하세요.",
                "Mapa przekroczyła limit {seconds} s po odwiedzeniu {visited} ścieżek. Zamknij ją i ponów próbę z innymi limitami.",
                "O mapa excedeu o limite de {seconds} segundos após visitar {visited} caminhos. Feche-a e tente novamente com outros limites.",
                "Harta a depășit limita de {seconds} secunde după parcurgerea a {visited} căi. Închide-o și reîncearcă folosind alte limite.",
                "Карта превысила лимит в {seconds} с после просмотра путей: {visited}. Закройте её и повторите попытку с другими ограничениями.",
                "Карта перевищила ліміт у {seconds} с після перегляду шляхів: {visited}. Закрийте її та повторіть спробу з іншими обмеженнями.",
                "地图访问 {visited} 个路径后超过了 {seconds} 秒限制。可以关闭并使用其他限制重试。",
            ],
        ),
        (
            "scan_ready",
            [
                "اكتملت الخريطة: تمت زيارة {visited} مسارًا دون عوائق أذونات. تعرض كل قيمة جذر الإجمالي والمستخدم والحر والمتاح. استخدم الأسهم للاستكشاف.",
                "Karte bereit: {visited} Pfade besucht, keine Berechtigungsprobleme. Jede Wurzel zeigt Gesamt-, belegten, freien und verfügbaren Speicher. Mit den Pfeilen erkunden.",
                "Map ready: {visited} paths visited with no permission blocks. Each root shows total, used, free and available space. Use the arrows to explore.",
                "Mapa listo: {visited} rutas visitadas sin bloqueos de permisos. Cada raíz muestra el total, ocupado, libre y disponible. Usa las flechas para explorar.",
                "Carte prête : {visited} chemins parcourus sans blocage d’accès. Chaque racine affiche les espaces total, utilisé, libre et disponible. Utilisez les flèches pour explorer.",
                "मानचित्र तैयार: {visited} पथ देखे गए, अनुमति संबंधी कोई रुकावट नहीं। हर रूट कुल, उपयोग किया गया, खाली और उपलब्ध स्थान दिखाता है। तीरों से देखें।",
                "Mappa pronta: visitati {visited} percorsi senza blocchi di autorizzazione. Ogni radice mostra spazio totale, usato, libero e disponibile. Usa le frecce per esplorare.",
                "マップの準備完了: {visited} 個のパスを確認し、権限による停止はありません。各ルートに合計、使用済み、空き、利用可能容量を表示します。矢印で探索できます。",
                "지도 준비 완료: 경로 {visited}개를 확인했으며 권한 차단이 없습니다. 각 루트에 전체, 사용, 여유, 사용 가능 공간이 표시됩니다. 화살표로 탐색하세요.",
                "Mapa gotowa: odwiedzono {visited} ścieżek, bez blokad uprawnień. Każdy katalog główny pokazuje pojemność całkowitą, używaną, wolną i dostępną. Użyj strzałek, aby eksplorować.",
                "Mapa pronta: {visited} caminhos visitados sem bloqueios de permissões. Cada raiz mostra o espaço total, usado, livre e disponível. Use as setas para explorar.",
                "Harta este gata: au fost parcurse {visited} căi fără blocaje de permisiuni. Fiecare rădăcină afișează spațiul total, utilizat, liber și disponibil. Folosește săgețile pentru explorare.",
                "Карта готова: просмотрено путей: {visited}, ошибок доступа нет. Для каждого корня показаны общий, занятый, свободный и доступный объёмы. Используйте стрелки для навигации.",
                "Карту готово: переглянуто шляхів: {visited}, помилок доступу немає. Для кожного кореня показано загальний, зайнятий, вільний і доступний обсяги. Використовуйте стрілки для навігації.",
                "地图已就绪：已访问 {visited} 个路径，没有权限阻塞。每个根目录显示总计、已用、空闲和可用空间。使用箭头浏览。",
            ],
        ),
        (
            "scan_permissions",
            [
                "اكتملت الخريطة: تمت زيارة {visited} مسارًا؛ {inaccessible} منها تتطلب أذونات إضافية. تعرض كل قيمة جذر الإجمالي والمستخدم والحر والمتاح. يمكنك إعادة المحاولة كمسؤول.",
                "Karte bereit: {visited} Pfade besucht; für {inaccessible} sind zusätzliche Rechte nötig. Jede Wurzel zeigt Gesamt-, belegten, freien und verfügbaren Speicher. Erneuter Versuch als Administrator möglich.",
                "Map ready: {visited} paths visited; {inaccessible} require additional permissions. Each root shows total, used, free and available space. You can retry as administrator.",
                "Mapa listo: {visited} rutas visitadas; {inaccessible} requieren permisos adicionales. Cada raíz muestra el total, ocupado, libre y disponible. Puedes reintentar como administrador.",
                "Carte prête : {visited} chemins parcourus ; {inaccessible} nécessitent des permissions supplémentaires. Chaque racine affiche les espaces total, utilisé, libre et disponible. Vous pouvez réessayer en administrateur.",
                "मानचित्र तैयार: {visited} पथ देखे गए; {inaccessible} के लिए अतिरिक्त अनुमतियाँ चाहिए। हर रूट कुल, उपयोग किया गया, खाली और उपलब्ध स्थान दिखाता है। व्यवस्थापक के रूप में फिर प्रयास करें।",
                "Mappa pronta: visitati {visited} percorsi; {inaccessible} richiedono autorizzazioni aggiuntive. Ogni radice mostra spazio totale, usato, libero e disponibile. Puoi riprovare come amministratore.",
                "マップの準備完了: {visited} 個のパスを確認し、{inaccessible} 個には追加の権限が必要です。各ルートに合計、使用済み、空き、利用可能容量を表示します。管理者として再試行できます。",
                "지도 준비 완료: 경로 {visited}개를 확인했으며 {inaccessible}개에는 추가 권한이 필요합니다. 각 루트에 전체, 사용, 여유, 사용 가능 공간이 표시됩니다. 관리자로 다시 시도할 수 있습니다.",
                "Mapa gotowa: odwiedzono {visited} ścieżek; {inaccessible} wymaga dodatkowych uprawnień. Każdy katalog główny pokazuje pojemność całkowitą, używaną, wolną i dostępną. Możesz ponowić próbę jako administrator.",
                "Mapa pronta: {visited} caminhos visitados; {inaccessible} requerem permissões adicionais. Cada raiz mostra o espaço total, usado, livre e disponível. Pode tentar novamente como administrador.",
                "Harta este gata: au fost parcurse {visited} căi; {inaccessible} necesită permisiuni suplimentare. Fiecare rădăcină afișează spațiul total, utilizat, liber și disponibil. Poți reîncerca drept administrator.",
                "Карта готова: просмотрено путей: {visited}; для {inaccessible} нужны дополнительные права. Для каждого корня показаны общий, занятый, свободный и доступный объёмы. Можно повторить с правами администратора.",
                "Карту готово: переглянуто шляхів: {visited}; для {inaccessible} потрібні додаткові права. Для кожного кореня показано загальний, зайнятий, вільний і доступний обсяги. Можна повторити з правами адміністратора.",
                "地图已就绪：已访问 {visited} 个路径；其中 {inaccessible} 个需要额外权限。每个根目录显示总计、已用、空闲和可用空间。可以尝试以管理员身份重试。",
            ],
        ),
        (
            "scan_request_admin",
            [
                "جارٍ طلب صلاحيات المسؤول… أدخل كلمة المرور إذا طُلبت.", "Administratorrechte werden angefordert… Gib bei Aufforderung dein Passwort ein.", "Requesting administrator privileges… Enter your password if prompted.", "Solicitando permisos de administrador… Introduce la contraseña si se solicita.", "Demande des privilèges administrateur… Saisissez le mot de passe si demandé.", "व्यवस्थापक अनुमतियाँ माँगी जा रही हैं… पूछे जाने पर पासवर्ड दर्ज करें।", "Richiesta dei privilegi di amministratore… Inserisci la password se richiesta.", "管理者権限を要求中…求められた場合はパスワードを入力してください。", "관리자 권한 요청 중… 암호를 묻는 경우 입력하세요.", "Trwa żądanie uprawnień administratora… W razie potrzeby wpisz hasło.", "A pedir privilégios de administrador… Introduza a palavra-passe se for solicitada.", "Se solicită privilegii de administrator… Introdu parola dacă ți se cere.", "Запрос прав администратора… При появлении запроса введите пароль.", "Запит прав адміністратора… Якщо з’явиться запит, введіть пароль.", "正在请求管理员权限…如有提示，请输入密码。",
            ],
        ),
        (
            "scan_error",
            [
                "تعذّر إكمال الخريطة: {error}", "Karte konnte nicht abgeschlossen werden: {error}", "Could not complete the map: {error}", "No se pudo completar el mapa: {error}", "Impossible de terminer la carte : {error}", "मानचित्र पूरा नहीं हो सका: {error}", "Impossibile completare la mappa: {error}", "マップを完了できませんでした: {error}", "지도를 완료할 수 없습니다: {error}", "Nie udało się ukończyć mapy: {error}", "Não foi possível concluir o mapa: {error}", "Harta nu a putut fi finalizată: {error}", "Не удалось завершить карту: {error}", "Не вдалося завершити карту: {error}", "无法完成地图：{error}",
            ],
        ),
        (
            "scan_empty",
            [
                "انتهى الفحص دون العثور على مسارات. تحقق من الأذونات وحاول مرة أخرى.", "Der Scan lieferte keine Pfade. Prüfe die Berechtigungen und versuche es erneut.", "The scan finished without returning any paths. Check permissions and retry.", "El escaneo terminó sin devolver rutas. Comprueba los permisos y vuelve a intentarlo.", "L’analyse n’a renvoyé aucun chemin. Vérifiez les permissions et réessayez.", "स्कैन में कोई पथ नहीं मिला। अनुमतियाँ जाँचें और फिर प्रयास करें।", "La scansione non ha restituito percorsi. Controlla le autorizzazioni e riprova.", "スキャンでパスが見つかりませんでした。権限を確認して再試行してください。", "검사에서 경로를 반환하지 않았습니다. 권한을 확인하고 다시 시도하세요.", "Skanowanie nie zwróciło żadnych ścieżek. Sprawdź uprawnienia i ponów próbę.", "A análise não devolveu caminhos. Verifique as permissões e tente novamente.", "Scanarea nu a returnat căi. Verifică permisiunile și încearcă din nou.", "Сканирование не вернуло пути. Проверьте права и повторите попытку.", "Сканування не повернуло шляхів. Перевірте права та повторіть спробу.", "扫描未返回任何路径。请检查权限并重试。",
            ],
        ),
        (
            "scan_panic",
            ["انتهى الفاحص بشكل غير متوقع.", "Der Scanner wurde unerwartet beendet.", "The scanner stopped unexpectedly.", "El escáner terminó de forma inesperada.", "Le scanner s’est arrêté de manière inattendue.", "स्कैनर अप्रत्याशित रूप से रुक गया।", "Lo scanner si è arrestato in modo imprevisto.", "スキャナーが予期せず停止しました。", "스캐너가 예기치 않게 종료되었습니다.", "Skaner nieoczekiwanie się zatrzymał.", "O analisador terminou inesperadamente.", "Scanerul s-a oprit neașteptat.", "Сканер завершился неожиданно.", "Сканер завершився неочікувано.", "扫描器意外停止。"],
        ),
        (
            "selected_required",
            [
                "حدد ملفًا أو مجلدًا من الخريطة أولًا.", "Wähle zuerst eine Datei oder einen Ordner in der Karte aus.", "Select a file or folder in the map first.", "Selecciona primero un archivo o una carpeta del mapa.", "Sélectionnez d’abord un fichier ou un dossier dans la carte.", "पहले मानचित्र में कोई फ़ाइल या फ़ोल्डर चुनें।", "Seleziona prima un file o una cartella nella mappa.", "最初にマップでファイルまたはフォルダーを選択してください。", "먼저 지도에서 파일이나 폴더를 선택하세요.", "Najpierw wybierz plik lub folder na mapie.", "Selecione primeiro um ficheiro ou pasta no mapa.", "Selectează mai întâi un fișier sau un dosar din hartă.", "Сначала выберите файл или папку на карте.", "Спочатку виберіть файл або папку на карті.", "请先在地图中选择文件或文件夹。",
            ],
        ),
        (
            "copy_title",
            ["نسخ المسار المحدد", "Ausgewählten Pfad kopieren", "Copy selected path", "Copiar ruta seleccionada", "Copier le chemin sélectionné", "चुना हुआ पथ कॉपी करें", "Copia il percorso selezionato", "選択したパスをコピー", "선택한 경로 복사", "Kopiuj wybraną ścieżkę", "Copiar caminho selecionado", "Copiază calea selectată", "Копировать выбранный путь", "Копіювати вибраний шлях", "复制所选路径"],
        ),
        (
            "move_title",
            ["نقل المسار المحدد", "Ausgewählten Pfad verschieben", "Move selected path", "Mover ruta seleccionada", "Déplacer le chemin sélectionné", "चुना हुआ पथ स्थानांतरित करें", "Sposta il percorso selezionato", "選択したパスを移動", "선택한 경로 이동", "Przenieś wybraną ścieżkę", "Mover caminho selecionado", "Mută calea selectată", "Переместить выбранный путь", "Перемістити вибраний шлях", "移动所选路径"],
        ),
        (
            "delete_title",
            ["إرسال المسار المحدد إلى سلة المهملات", "Ausgewählten Pfad in den Papierkorb verschieben", "Send selected path to trash", "Enviar ruta seleccionada a la papelera", "Mettre le chemin sélectionné à la corbeille", "चुने हुए पथ को ट्रैश में भेजें", "Sposta il percorso selezionato nel cestino", "選択したパスをゴミ箱へ移動", "선택한 경로를 휴지통으로 보내기", "Przenieś wybraną ścieżkę do kosza", "Enviar caminho selecionado para o lixo", "Trimite calea selectată în coș", "Отправить выбранный путь в корзину", "Надіслати вибраний шлях до кошика", "将所选路径移至回收站"],
        ),
        (
            "state_inaccessible",
            ["متعذّر الوصول", "nicht zugänglich", "inaccessible", "inaccesible", "inaccessible", "पहुंच योग्य नहीं", "inaccessibile", "アクセス不可", "접근 불가", "niedostępny", "inacessível", "inaccesibil", "недоступно", "недоступно", "无法访问"],
        ),
        (
            "state_protected",
            ["محمي", "geschützt", "protected", "protegida", "protégé", "सुरक्षित", "protetto", "保護", "보호됨", "chroniony", "protegido", "protejat", "защищено", "захищено", "受保护"],
        ),
        (
            "state_writable",
            ["قابل للكتابة", "beschreibbar", "writable", "escribible", "inscriptible", "लिखने योग्य", "scrivibile", "書き込み可能", "쓰기 가능", "zapisywalny", "gravável", "inscriptibil", "доступно для записи", "доступний для запису", "可写"],
        ),
        (
            "state_read_only",
            ["للقراءة فقط", "schreibgeschützt", "read-only", "solo lectura", "lecture seule", "केवल-पठन", "sola lettura", "読み取り専用", "읽기 전용", "tylko do odczytu", "só de leitura", "doar în citire", "только чтение", "лише читання", "只读"],
        ),
        (
            "kind_file",
            ["ملف", "Datei", "file", "archivo", "fichier", "फ़ाइल", "file", "ファイル", "파일", "plik", "ficheiro", "fișier", "файл", "файл", "文件"],
        ),
        (
            "kind_directory",
            ["مجلد", "Ordner", "directory", "carpeta", "dossier", "फ़ोल्डर", "cartella", "フォルダー", "폴더", "folder", "pasta", "dosar", "каталог", "каталог", "目录"],
        ),
        (
            "kind_symlink",
            ["رابط رمزي", "symbolischer Link", "symlink", "enlace simbólico", "lien symbolique", "सिमलिंक", "collegamento simbolico", "シンボリックリンク", "심볼릭 링크", "dowiązanie symboliczne", "ligação simbólica", "legătură simbolică", "символическая ссылка", "символічне посилання", "符号链接"],
        ),
        (
            "kind_other",
            ["عنصر آخر", "sonstiges", "other", "otro", "autre", "अन्य", "altro", "その他", "기타", "inny", "outro", "altul", "другое", "інше", "其他"],
        ),
        (
            "kind_inaccessible",
            ["غير قابل للوصول", "nicht zugänglich", "inaccessible", "inaccesible", "inaccessible", "पहुंच योग्य नहीं", "inaccessibile", "アクセス不可", "접근 불가", "niedostępny", "inacessível", "inaccesibil", "недоступно", "недоступно", "无法访问"],
        ),
        (
            "kind_missing",
            ["مفقود", "fehlt", "missing", "ausente", "manquant", "अनुपस्थित", "mancante", "見つかりません", "누락됨", "brak", "em falta", "lipsește", "отсутствует", "відсутній", "缺失"],
        ),
        (
            "filesystem_usage",
            ["الإجمالي {total} · المستخدم {used} · الحر {free} · المتاح {available}", "Gesamt {total} · belegt {used} · frei {free} · verfügbar {available}", "total {total} · used {used} · free {free} · available {available}", "total {total} · ocupado {used} · libre {free} · disponible {available}", "total {total} · utilisé {used} · libre {free} · disponible {available}", "कुल {total} · उपयोग किया {used} · खाली {free} · उपलब्ध {available}", "totale {total} · usato {used} · libero {free} · disponibile {available}", "合計 {total} · 使用済み {used} · 空き {free} · 利用可能 {available}", "전체 {total} · 사용 {used} · 여유 {free} · 사용 가능 {available}", "całość {total} · użyte {used} · wolne {free} · dostępne {available}", "total {total} · usado {used} · livre {free} · disponível {available}", "total {total} · utilizat {used} · liber {free} · disponibil {available}", "всего {total} · занято {used} · свободно {free} · доступно {available}", "усього {total} · зайнято {used} · вільно {free} · доступно {available}", "总计 {total} · 已用 {used} · 空闲 {free} · 可用 {available}"],
        ),
        (
            "explain_system_root",
            ["جذر النظام: يضم شجرة POSIX كاملة؛ لا تحذف الملفات أو تنقلها.", "Systemwurzel: umfasst den gesamten POSIX-Baum; nichts löschen oder verschieben.", "System root: contains the entire POSIX tree; do not delete or move it.", "Raíz del sistema: contiene todo el árbol POSIX; no borrar ni mover.", "Racine du système : contient toute l’arborescence POSIX ; ne pas supprimer ni déplacer.", "सिस्टम रूट: पूरा POSIX ट्री रखता है; इसे हटाएँ या स्थानांतरित न करें।", "Radice del sistema: contiene l’intero albero POSIX; non eliminare né spostare.", "システムルート：POSIX ツリー全体を含みます。削除・移動しないでください。", "시스템 루트: 전체 POSIX 트리를 포함합니다. 삭제하거나 이동하지 마세요.", "Główny katalog systemu: obejmuje całe drzewo POSIX; nie usuwaj ani nie przenoś.", "Raiz do sistema: contém toda a árvore POSIX; não elimine nem mova.", "Rădăcina sistemului: conține întregul arbore POSIX; nu o șterge și nu o muta.", "Корень системы: содержит всё дерево POSIX; не удаляйте и не перемещайте.", "Корінь системи: містить усе дерево POSIX; не видаляйте й не переміщуйте.", "系统根目录：包含整个 POSIX 树；请勿删除或移动。"],
        ),
        (
            "explain_boot",
            ["ملفات الإقلاع: النواة وinitramfs ومحمل الإقلاع؛ لا تعدّلها دون خطة استعادة.", "Startdateien: Kernel, initramfs und Bootloader; Änderungen nur mit Wiederherstellungsplan.", "Boot files: kernel, initramfs and bootloaders; do not change without a recovery plan.", "Arranque: kernel, initramfs y cargadores; no modificar sin un plan de recuperación.", "Démarrage : noyau, initramfs et chargeurs ; ne pas modifier sans plan de récupération.", "बूट फ़ाइलें: कर्नेल, initramfs और बूटलोडर; पुनर्प्राप्ति योजना के बिना न बदलें।", "Avvio: kernel, initramfs e bootloader; non modificare senza un piano di ripristino.", "起動ファイル：カーネル、initramfs、ブートローダー。復旧計画なしに変更しないでください。", "부팅 파일: 커널, initramfs, 부트로더입니다. 복구 계획 없이 변경하지 마세요.", "Pliki rozruchowe: jądro, initramfs i programy rozruchowe; nie zmieniaj bez planu odzyskiwania.", "Arranque: kernel, initramfs e carregadores; não altere sem um plano de recuperação.", "Fișiere de pornire: kernel, initramfs și încărcătoare; nu modifica fără un plan de recuperare.", "Загрузка: ядро, initramfs и загрузчики; не изменяйте без плана восстановления.", "Завантаження: ядро, initramfs і завантажувачі; не змінюйте без плану відновлення.", "启动文件：内核、initramfs 和引导加载程序；没有恢复计划请勿修改。"],
        ),
        (
            "explain_system_config",
            ["إعدادات النظام والخدمات؛ عدّل ملفات محددة فقط واحتفظ بنسخة احتياطية.", "System- und Dienstkonfiguration; nur gezielte Dateien ändern und vorher sichern.", "System and service configuration; edit specific files only and keep a backup.", "Configuración del sistema y servicios; editar solo archivos concretos y conservar copia.", "Configuration du système et des services ; modifier uniquement des fichiers précis et garder une sauvegarde.", "सिस्टम और सेवा कॉन्फ़िगरेशन; केवल चुनिंदा फ़ाइलें बदलें और बैकअप रखें।", "Configurazione di sistema e servizi; modifica solo file specifici e conserva un backup.", "システムとサービスの設定です。対象ファイルだけを編集し、バックアップを保存してください。", "시스템 및 서비스 설정입니다. 필요한 파일만 편집하고 백업을 보관하세요.", "Konfiguracja systemu i usług; zmieniaj tylko konkretne pliki i zachowaj kopię.", "Configuração do sistema e serviços; edite apenas ficheiros específicos e mantenha uma cópia.", "Configurația sistemului și a serviciilor; modifică doar fișiere precise și păstrează o copie.", "Настройки системы и служб; изменяйте только конкретные файлы и сохраняйте резервную копию.", "Налаштування системи та служб; змінюйте лише конкретні файли й зберігайте резервну копію.", "系统与服务配置；仅编辑明确的文件并保留备份。"],
        ),
        (
            "explain_managed_programs",
            ["برامج ومكتبات يديرها النظام؛ لا تنقلها يدويًا، واستخدم مدير الحزم أو مزيل التثبيت.", "Vom System verwaltete Programme und Bibliotheken; nicht manuell verschieben, Paketverwaltung oder Deinstallation nutzen.", "System-managed programs and libraries; do not move manually—use the package manager or uninstaller.", "Programas y librerías gestionados por el sistema; no mover manualmente, usa el gestor o desinstalador.", "Programmes et bibliothèques gérés par le système ; ne pas déplacer, utiliser le gestionnaire ou le désinstalleur.", "सिस्टम द्वारा प्रबंधित प्रोग्राम और लाइब्रेरी; मैन्युअल रूप से न हटाएँ, पैकेज प्रबंधक या अनइंस्टॉलर उपयोग करें।", "Programmi e librerie gestiti dal sistema; non spostare manualmente, usa il gestore pacchetti o la disinstallazione.", "システム管理のプログラムとライブラリです。手動で移動せず、パッケージ管理またはアンインストーラーを使用してください。", "시스템이 관리하는 프로그램과 라이브러리입니다. 직접 이동하지 말고 패키지 관리자나 제거 프로그램을 사용하세요.", "Programy i biblioteki zarządzane przez system; nie przenoś ręcznie, użyj menedżera pakietów lub deinstalatora.", "Programas e bibliotecas geridos pelo sistema; não mova manualmente, use o gestor de pacotes ou desinstalador.", "Programe și biblioteci gestionate de sistem; nu le muta manual, folosește managerul de pachete sau dezinstalatorul.", "Программы и библиотеки под управлением системы; не перемещайте вручную, используйте пакетный менеджер или удаление.", "Програми й бібліотеки під керуванням системи; не переміщуйте вручну, скористайтеся менеджером пакунків або видаленням.", "由系统管理的程序和库；请勿手动移动，使用包管理器或卸载程序。"],
        ),
        (
            "explain_variable_data",
            ["بيانات متغيرة وسجلات وذاكرة مؤقتة وقواعد خدمات؛ نظّفها بالأداة المناسبة فقط.", "Veränderliche Daten, Protokolle, Caches und Dienstdatenbanken; nur mit dem passenden Werkzeug bereinigen.", "Variable data, logs, caches and service databases; clean only with the appropriate tool.", "Datos variables, registros, cachés y bases de servicios; limpiar solo con la herramienta adecuada.", "Données variables, journaux, caches et bases de services ; nettoyer uniquement avec l’outil approprié.", "परिवर्तनीय डेटा, लॉग, कैश और सेवा डेटाबेस; केवल उपयुक्त टूल से साफ़ करें।", "Dati variabili, log, cache e database dei servizi; pulisci solo con lo strumento adatto.", "可変データ、ログ、キャッシュ、サービスのデータベースです。適切なツールでのみクリーンアップしてください。", "가변 데이터, 로그, 캐시 및 서비스 데이터베이스입니다. 적절한 도구로만 정리하세요.", "Dane zmienne, dzienniki, pamięci podręczne i bazy usług; czyść tylko odpowiednim narzędziem.", "Dados variáveis, registos, caches e bases de dados de serviços; limpe apenas com a ferramenta adequada.", "Date variabile, jurnale, cache și baze de date ale serviciilor; curăță doar cu instrumentul potrivit.", "Изменяемые данные, журналы, кэши и базы служб; очищайте только подходящим инструментом.", "Змінні дані, журнали, кеші та бази служб; очищуйте лише відповідним інструментом.", "可变数据、日志、缓存和服务数据库；仅使用相应工具清理。"],
        ),
        (
            "explain_user_data",
            ["بيانات المستخدمين؛ قد يحتوي كل مجلد على إعدادات ووثائق شخصية.", "Benutzerdaten; jeder Ordner kann persönliche Einstellungen und Dokumente enthalten.", "User data; each folder may contain personal settings and documents.", "Datos de usuarios; cada carpeta puede contener configuración y documentos personales.", "Données utilisateur ; chaque dossier peut contenir des réglages et documents personnels.", "उपयोगकर्ता डेटा; हर फ़ोल्डर में निजी सेटिंग और दस्तावेज़ हो सकते हैं।", "Dati utente; ogni cartella può contenere impostazioni e documenti personali.", "ユーザーデータです。各フォルダーには個人設定や文書が含まれる場合があります。", "사용자 데이터입니다. 각 폴더에 개인 설정과 문서가 있을 수 있습니다.", "Dane użytkowników; każdy folder może zawierać ustawienia i dokumenty osobiste.", "Dados dos utilizadores; cada pasta pode conter configurações e documentos pessoais.", "Datele utilizatorilor; fiecare dosar poate conține setări și documente personale.", "Данные пользователей; в каждой папке могут быть личные настройки и документы.", "Дані користувачів; у кожній папці можуть бути особисті налаштування й документи.", "用户数据；每个文件夹可能包含个人设置和文档。"],
        ),
        (
            "explain_temporary",
            ["ملفات مؤقتة؛ يمكن تنظيفها وفق سياسة النظام، لكن لا تفترض أن كل محتواها غير مهم.", "Temporäre Dateien; nach Systemrichtlinie bereinigen, aber nicht alles als entbehrlich ansehen.", "Temporary files; clean according to system policy, but do not assume everything is disposable.", "Temporales; se pueden purgar según la política del sistema, pero no todo es prescindible.", "Fichiers temporaires ; nettoyer selon la politique du système sans supposer que tout est inutile.", "अस्थायी फ़ाइलें; सिस्टम नीति के अनुसार साफ़ करें, पर सबको बेकार न मानें।", "File temporanei; pulisci secondo la politica di sistema, senza presumere che tutto sia sacrificabile.", "一時ファイルです。システム方針に従って削除し、すべて不要とは限らない点に注意してください。", "임시 파일입니다. 시스템 정책에 따라 정리하되 모두 불필요하다고 가정하지 마세요.", "Pliki tymczasowe; czyść zgodnie z zasadami systemu, nie zakładając, że wszystko jest zbędne.", "Ficheiros temporários; limpe conforme a política do sistema, sem presumir que tudo é descartável.", "Fișiere temporare; curăță conform politicii sistemului, fără să presupui că toate sunt inutile.", "Временные файлы; очищайте согласно политике системы и не считайте всё содержимое ненужным.", "Тимчасові файли; очищуйте за правилами системи й не вважайте весь вміст непотрібним.", "临时文件；请按系统策略清理，不要假设其中所有内容都可丢弃。"],
        ),
        (
            "explain_virtual_proc",
            ["عرض افتراضي للنواة؛ لا يحتوي ملفات عادية للنسخ أو الحذف.", "Virtuelle Kernelansicht; enthält keine normalen Dateien zum Kopieren oder Löschen.", "Virtual view of the kernel; it does not contain ordinary files to copy or delete.", "Vista virtual del kernel; no contiene archivos normales que se puedan copiar o borrar.", "Vue virtuelle du noyau ; ne contient pas de fichiers ordinaires à copier ou supprimer.", "कर्नेल का वर्चुअल दृश्य; इसमें कॉपी या हटाने योग्य सामान्य फ़ाइलें नहीं हैं।", "Vista virtuale del kernel; non contiene file normali da copiare o eliminare.", "カーネルの仮想ビューです。コピーや削除の対象となる通常ファイルはありません。", "커널의 가상 보기입니다. 복사하거나 삭제할 일반 파일이 없습니다.", "Wirtualny widok jądra; nie zawiera zwykłych plików do kopiowania ani usuwania.", "Vista virtual do kernel; não contém ficheiros normais para copiar ou eliminar.", "Vizualizare virtuală a kernelului; nu conține fișiere obișnuite de copiat sau șters.", "Виртуальное представление ядра; обычных файлов для копирования или удаления здесь нет.", "Віртуальне подання ядра; звичайних файлів для копіювання чи видалення тут немає.", "内核的虚拟视图；不包含可复制或删除的普通文件。"],
        ),
        (
            "explain_virtual_sys",
            ["عرض افتراضي للأجهزة والنواة؛ لا تعدّله من هذا المدير.", "Virtuelle Ansicht von Hardware und Kernel; nicht mit diesem Manager ändern.", "Virtual view of hardware and kernel; do not modify it with this manager.", "Vista virtual del hardware y kernel; no modificar desde este gestor.", "Vue virtuelle du matériel et du noyau ; ne pas modifier depuis ce gestionnaire.", "हार्डवेयर और कर्नेल का वर्चुअल दृश्य; इस प्रबंधक से न बदलें।", "Vista virtuale di hardware e kernel; non modificarla da questo gestore.", "ハードウェアとカーネルの仮想ビューです。この管理ツールから変更しないでください。", "하드웨어와 커널의 가상 보기입니다. 이 관리 도구에서 수정하지 마세요.", "Wirtualny widok sprzętu i jądra; nie zmieniaj go w tym menedżerze.", "Vista virtual do hardware e do kernel; não altere neste gestor.", "Vizualizare virtuală a hardware-ului și kernelului; nu o modifica din acest manager.", "Виртуальное представление оборудования и ядра; не изменяйте его этим менеджером.", "Віртуальне подання обладнання та ядра; не змінюйте його в цьому менеджері.", "硬件与内核的虚拟视图；请勿通过此管理器修改。"],
        ),
        (
            "explain_device_nodes",
            ["عُقد أجهزة خاصة؛ لا تتعامل معها كملفات عادية ولا تحذفها.", "Spezielle Gerätedateien; nicht wie normale Dateien behandeln oder löschen.", "Special device nodes; do not treat them as ordinary files or delete them.", "Dispositivos especiales; no tratarlos como archivos comunes ni borrar nodos.", "Nœuds de périphériques spéciaux ; ne pas les traiter comme des fichiers ordinaires ni les supprimer.", "विशेष डिवाइस नोड; सामान्य फ़ाइल न समझें और न हटाएँ।", "Nodi di dispositivi speciali; non trattarli come file comuni né eliminarli.", "特殊なデバイスノードです。通常ファイルとして扱ったり削除したりしないでください。", "특수 장치 노드입니다. 일반 파일처럼 취급하거나 삭제하지 마세요.", "Specjalne węzły urządzeń; nie traktuj ich jak zwykłych plików ani nie usuwaj.", "Nós de dispositivos especiais; não os trate como ficheiros comuns nem os elimine.", "Noduri speciale de dispozitiv; nu le trata ca fișiere obișnuite și nu le șterge.", "Специальные узлы устройств; не считайте их обычными файлами и не удаляйте.", "Спеціальні вузли пристроїв; не вважайте їх звичайними файлами й не видаляйте.", "特殊设备节点；请勿将其当作普通文件或删除。"],
        ),
        (
            "explain_runtime",
            ["حالة مؤقتة للخدمات والجلسات؛ لا تنقلها أو تحذفها يدويًا.", "Flüchtiger Status von Diensten und Sitzungen; nicht manuell verschieben oder löschen.", "Ephemeral service and session state; do not move or delete manually.", "Estado efímero de servicios y sesiones; no mover ni borrar manualmente.", "État éphémère des services et sessions ; ne pas déplacer ni supprimer manuellement.", "सेवाओं और सत्रों की अस्थायी स्थिति; मैन्युअल रूप से न हटाएँ या स्थानांतरित करें।", "Stato temporaneo di servizi e sessioni; non spostare o eliminare manualmente.", "サービスやセッションの一時状態です。手動で移動・削除しないでください。", "서비스와 세션의 임시 상태입니다. 직접 이동하거나 삭제하지 마세요.", "Nietrwały stan usług i sesji; nie przenoś ani nie usuwaj ręcznie.", "Estado temporário de serviços e sessões; não mova nem elimine manualmente.", "Stare temporară a serviciilor și sesiunilor; nu o muta și nu o șterge manual.", "Временное состояние служб и сеансов; не перемещайте и не удаляйте вручную.", "Тимчасовий стан служб і сеансів; не переміщуйте й не видаляйте вручну.", "服务和会话的临时状态；请勿手动移动或删除。"],
        ),
        (
            "explain_user_config",
            ["إعدادات المستخدم؛ تحقق من أن المسار ضمن ملفه الشخصي قبل تعديله.", "Benutzereinstellungen; vor Änderungen sicherstellen, dass der Pfad zum Benutzerprofil gehört.", "User configuration; confirm the path belongs to the user profile before changing it.", "Configuración de usuario; confirma que la ruta pertenece al perfil antes de modificarla.", "Configuration utilisateur ; vérifier que le chemin appartient au profil avant toute modification.", "उपयोगकर्ता कॉन्फ़िगरेशन; बदलाव से पहले पुष्टि करें कि पथ प्रोफ़ाइल का है।", "Configurazione utente; verifica che il percorso appartenga al profilo prima di modificarlo.", "ユーザー設定です。変更前にユーザープロファイル内のパスであることを確認してください。", "사용자 설정입니다. 변경 전에 사용자 프로필 경로인지 확인하세요.", "Konfiguracja użytkownika; przed zmianą upewnij się, że ścieżka należy do profilu.", "Configuração do utilizador; confirme que o caminho pertence ao perfil antes de alterar.", "Configurația utilizatorului; verifică dacă această cale aparține profilului înainte de modificare.", "Настройки пользователя; перед изменением убедитесь, что путь относится к профилю.", "Налаштування користувача; перед зміною переконайтеся, що шлях належить профілю.", "用户配置；修改前请确认该路径属于用户配置文件。"],
        ),
        (
            "explain_user_cache",
            ["ذاكرة مؤقتة للمستخدم؛ غالبًا يمكن تنظيفها، لكن إعادة بنائها قد تبطئ التشغيل التالي.", "Benutzer-Cache; meist bereinigbar, kann den nächsten Start danach aber verlangsamen.", "User cache; usually safe to clear, but rebuilding it may slow the next startup.", "Caché de usuario; suele poder limpiarse, pero regenerarla puede ralentizar el siguiente inicio.", "Cache utilisateur ; généralement nettoyable, mais sa reconstruction peut ralentir le prochain démarrage.", "उपयोगकर्ता कैश; आमतौर पर साफ़ किया जा सकता है, पर दोबारा बनने में अगला स्टार्ट धीमा हो सकता है।", "Cache utente; di solito eliminabile, ma la ricostruzione può rallentare il prossimo avvio.", "ユーザーキャッシュです。通常は削除できますが、再生成により次回起動が遅くなる場合があります。", "사용자 캐시입니다. 대개 삭제할 수 있지만 다시 만드는 동안 다음 시작이 느려질 수 있습니다.", "Pamięć podręczna użytkownika; zwykle można ją wyczyścić, ale odbudowa może spowolnić kolejny start.", "Cache do utilizador; geralmente pode ser limpa, mas a reconstrução pode atrasar o próximo arranque.", "Cache-ul utilizatorului; de obicei poate fi golit, dar reconstruirea poate încetini următoarea pornire.", "Кэш пользователя обычно можно очистить, но его восстановление может замедлить следующий запуск.", "Кеш користувача зазвичай можна очистити, але його відновлення може сповільнити наступний запуск.", "用户缓存通常可以清理，但重建缓存可能会让下次启动变慢。"],
        ),
        (
            "explain_windows_os",
            ["نظام Windows: يحتوي ملفات نظام التشغيل؛ لا تنقلها أو تحذفها.", "Windows-System: enthält Betriebssystemdateien; nicht verschieben oder löschen.", "Windows system: contains operating-system files; do not move or delete.", "Sistema Windows: contiene archivos del sistema operativo; no mover ni borrar.", "Système Windows : contient les fichiers du système d’exploitation ; ne pas déplacer ni supprimer.", "Windows सिस्टम: ऑपरेटिंग सिस्टम फ़ाइलें हैं; इन्हें न हटाएँ या स्थानांतरित करें।", "Sistema Windows: contiene file del sistema operativo; non spostare né eliminare.", "Windows システム：OS ファイルが含まれています。移動・削除しないでください。", "Windows 시스템: 운영 체제 파일이 있습니다. 이동하거나 삭제하지 마세요.", "System Windows: zawiera pliki systemu operacyjnego; nie przenoś ani nie usuwaj.", "Sistema Windows: contém ficheiros do sistema operativo; não mova nem elimine.", "Sistemul Windows: conține fișiere ale sistemului de operare; nu le muta și nu le șterge.", "Система Windows: содержит файлы ОС; не перемещайте и не удаляйте их.", "Система Windows: містить файли ОС; не переміщуйте й не видаляйте їх.", "Windows 系统目录：包含操作系统文件；请勿移动或删除。"],
        ),
        (
            "explain_shared_app_data",
            ["بيانات مشتركة للتطبيقات؛ عدّلها فقط إذا عرفت التطبيق الذي يملكها.", "Gemeinsame Anwendungsdaten; nur ändern, wenn die zuständige Anwendung bekannt ist.", "Shared application data; modify only when you know which application owns it.", "Datos compartidos de aplicaciones; modificar solo si conoces la aplicación propietaria.", "Données partagées des applications ; modifier uniquement si l’application propriétaire est connue.", "साझा ऐप डेटा; तभी बदलें जब पता हो कि इसका स्वामी कौन-सा ऐप है।", "Dati condivisi delle applicazioni; modifica solo se conosci l’applicazione proprietaria.", "アプリ共有データです。所有するアプリが分かる場合にのみ変更してください。", "공유 애플리케이션 데이터입니다. 소유 앱을 알고 있을 때만 수정하세요.", "Wspólne dane aplikacji; zmieniaj tylko wtedy, gdy znasz aplikację będącą ich właścicielem.", "Dados partilhados das aplicações; altere apenas se souber qual aplicação é proprietária.", "Date partajate ale aplicațiilor; modifică doar dacă știi ce aplicație le deține.", "Общие данные приложений; изменяйте только если знаете, какому приложению они принадлежат.", "Спільні дані програм; змінюйте, лише якщо знаєте програму-власника.", "应用程序共享数据；只有确认所属应用后才修改。"],
        ),
        (
            "explain_user_profiles",
            ["ملفات المستخدمين الشخصية؛ تحتوي بيانات وإعدادات خاصة بكل حساب.", "Benutzerprofile; enthalten persönliche Daten und Einstellungen jedes Kontos.", "User profiles; contain personal data and settings for each account.", "Perfiles de usuario; contienen datos y configuración personales de cada cuenta.", "Profils utilisateur ; contiennent les données et réglages personnels de chaque compte.", "उपयोगकर्ता प्रोफ़ाइल; हर खाते का निजी डेटा और सेटिंग रखती है।", "Profili utente; contengono dati e impostazioni personali di ogni account.", "ユーザープロファイル：各アカウントの個人データと設定が含まれます。", "사용자 프로필: 각 계정의 개인 데이터와 설정이 들어 있습니다.", "Profile użytkowników; zawierają osobiste dane i ustawienia każdego konta.", "Perfis de utilizador; contêm dados e definições pessoais de cada conta.", "Profilurile utilizatorilor conțin date și setări personale pentru fiecare cont.", "Профили пользователей содержат личные данные и настройки каждой учётной записи.", "Профілі користувачів містять особисті дані й налаштування кожного облікового запису.", "用户配置文件：包含每个账户的个人数据和设置。"],
        ),
        (
            "explain_user_appdata",
            ["إعدادات وذاكرات مؤقتة لكل مستخدم؛ بعضها قابل للتنظيف وبعضها ضروري للتطبيقات.", "Benutzerspezifische Einstellungen und Caches; manche sind bereinigbar, andere für Apps nötig.", "Per-user settings and caches; some can be cleared, others are required by applications.", "Configuración y cachés por usuario; algunas se pueden limpiar y otras son necesarias.", "Réglages et caches par utilisateur ; certains sont nettoyables, d’autres indispensables aux applications.", "प्रति-उपयोगकर्ता सेटिंग और कैश; कुछ साफ़ किए जा सकते हैं, कुछ ऐप के लिए ज़रूरी हैं।", "Impostazioni e cache per utente; alcune si possono pulire, altre servono alle applicazioni.", "ユーザーごとの設定とキャッシュです。削除できるものも、アプリに必須のものもあります。", "사용자별 설정 및 캐시입니다. 일부는 정리할 수 있지만 일부는 앱에 필요합니다.", "Ustawienia i pamięci podręczne użytkownika; część można czyścić, inne są potrzebne aplikacjom.", "Definições e caches por utilizador; algumas podem ser limpas, outras são necessárias às aplicações.", "Setări și cache-uri per utilizator; unele pot fi curățate, altele sunt necesare aplicațiilor.", "Пользовательские настройки и кэш: часть можно очистить, часть необходима приложениям.", "Налаштування й кеш користувача: деякі можна очистити, інші потрібні програмам.", "每用户设置和缓存；部分可清理，部分是应用程序必需的。"],
        ),
        (
            "explain_windows_metadata",
            ["بيانات وصفية محمية في Windows؛ لا تعدّلها عبر مدير ملفات عادي.", "Geschützte Windows-Metadaten; nicht mit einem normalen Dateimanager ändern.", "Protected Windows metadata; do not modify through a regular file manager.", "Metadatos protegidos de Windows; no modificar desde un gestor de archivos normal.", "Métadonnées Windows protégées ; ne pas modifier avec un gestionnaire de fichiers ordinaire.", "Windows का संरक्षित मेटाडेटा; सामान्य फ़ाइल प्रबंधक से न बदलें।", "Metadati Windows protetti; non modificarli con un normale file manager.", "Windows の保護されたメタデータです。通常のファイルマネージャーから変更しないでください。", "Windows 보호 메타데이터입니다. 일반 파일 관리자에서 수정하지 마세요.", "Chronione metadane systemu Windows; nie zmieniaj ich w zwykłym menedżerze plików.", "Metadados protegidos do Windows; não altere num gestor de ficheiros comum.", "Metadate Windows protejate; nu le modifica într-un manager obișnuit de fișiere.", "Защищённые метаданные Windows; не изменяйте их обычным файловым менеджером.", "Захищені метадані Windows; не змінюйте їх звичайним файловим менеджером.", "Windows 受保护的元数据；请勿通过普通文件管理器修改。"],
        ),
    ];
    let index = SUPPORTED
        .iter()
        .position(|language| *language == current())
        .unwrap_or(3);
    MAP_TEXT
        .iter()
        .find(|(entry, _)| *entry == key)
        .map(|(_, translations)| translations[index])
        .unwrap_or("")
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
        ("en", "network_connections") => "Network connections",
        ("en", "dns_flush") => "Flush DNS cache",
        ("en", "tools_status") => "Dependencies and versions",
        ("en", "tools_install") => "Install a dependency",
        ("en", "adb_devices") => "ADB: connected devices",
        ("en", "container_list") => "Docker/Podman: containers",
        ("en", "kubernetes_contexts") => "Kubernetes: contexts",
        ("de", "network_status") => "Netzwerk, Routen, DNS und offene Ports",
        ("de", "network_connections") => "Netzwerkverbindungen",
        ("de", "dns_flush") => "DNS-Cache leeren",
        ("de", "tools_status") => "Abhängigkeiten und Versionen",
        ("de", "tools_install") => "Abhängigkeit installieren",
        ("de", "adb_devices") => "ADB: verbundene Geräte",
        ("de", "container_list") => "Docker/Podman: Container",
        ("de", "kubernetes_contexts") => "Kubernetes: Kontexte",
        ("fr", "network_status") => "Réseau, routes, DNS et ports en écoute",
        ("fr", "network_connections") => "Connexions réseau",
        ("fr", "dns_flush") => "Vider le cache DNS",
        ("fr", "tools_status") => "Dépendances et versions",
        ("fr", "tools_install") => "Installer une dépendance",
        ("fr", "adb_devices") => "ADB : appareils connectés",
        ("fr", "container_list") => "Docker/Podman : conteneurs",
        ("fr", "kubernetes_contexts") => "Kubernetes : contextes",
        ("pt", "network_status") => "Rede, rotas, DNS e portas de escuta",
        ("pt", "network_connections") => "Ligações de rede",
        ("pt", "dns_flush") => "Limpar a cache DNS",
        ("pt", "tools_status") => "Dependências e versões",
        ("pt", "tools_install") => "Instalar uma dependência",
        ("pt", "adb_devices") => "ADB: dispositivos ligados",
        ("pt", "container_list") => "Docker/Podman: contentores",
        ("pt", "kubernetes_contexts") => "Kubernetes: contextos",
        ("it", "network_status") => "Rete, rotte, DNS e porte in ascolto",
        ("it", "network_connections") => "Connessioni di rete",
        ("it", "dns_flush") => "Svuota cache DNS",
        ("it", "tools_status") => "Dipendenze e versioni",
        ("it", "tools_install") => "Installa una dipendenza",
        ("it", "adb_devices") => "ADB: dispositivi collegati",
        ("it", "container_list") => "Docker/Podman: container",
        ("it", "kubernetes_contexts") => "Kubernetes: contesti",
        ("ca", "network_status") => "Xarxa, rutes, DNS i ports en escolta",
        ("ca", "network_connections") => "Connexions de xarxa",
        ("ca", "dns_flush") => "Buida la memòria cau DNS",
        ("ca", "tools_status") => "Dependències i versions",
        ("ca", "tools_install") => "Instal·lar una dependència",
        ("ca", "adb_devices") => "ADB: dispositius connectats",
        ("ca", "container_list") => "Docker/Podman: contenidors",
        ("ca", "kubernetes_contexts") => "Kubernetes: contextos",
        ("nl", "network_status") => "Netwerk, routes, DNS en luisterpoorten",
        ("nl", "network_connections") => "Netwerkverbindingen",
        ("nl", "dns_flush") => "DNS-cache wissen",
        ("nl", "tools_status") => "Afhankelijkheden en versies",
        ("nl", "tools_install") => "Afhankelijkheid installeren",
        ("nl", "adb_devices") => "ADB: verbonden apparaten",
        ("nl", "container_list") => "Docker/Podman: containers",
        ("nl", "kubernetes_contexts") => "Kubernetes: contexten",
        ("pl", "network_status") => "Sieć, trasy, DNS i nasłuchujące porty",
        ("pl", "network_connections") => "Połączenia sieciowe",
        ("pl", "dns_flush") => "Wyczyść pamięć DNS",
        ("pl", "tools_status") => "Zależności i wersje",
        ("pl", "tools_install") => "Zainstaluj zależność",
        ("pl", "adb_devices") => "ADB: podłączone urządzenia",
        ("pl", "container_list") => "Docker/Podman: kontenery",
        ("pl", "kubernetes_contexts") => "Kubernetes: konteksty",
        ("ar", "network_status") => "الشبكة والمسارات وDNS والمنافذ المستمعة",
        ("ar", "network_connections") => "اتصالات الشبكة",
        ("ar", "dns_flush") => "مسح ذاكرة DNS",
        ("ar", "tools_status") => "التبعيات والإصدارات",
        ("ar", "tools_install") => "تثبيت تبعية",
        ("ar", "adb_devices") => "ADB: الأجهزة المتصلة",
        ("ar", "container_list") => "Docker/Podman: الحاويات",
        ("ar", "kubernetes_contexts") => "Kubernetes: السياقات",
        ("hi", "network_status") => "नेटवर्क, रूट, DNS और सुनने वाले पोर्ट",
        ("hi", "network_connections") => "नेटवर्क कनेक्शन",
        ("hi", "dns_flush") => "DNS कैश साफ़ करें",
        ("hi", "tools_status") => "निर्भरताएँ और संस्करण",
        ("hi", "tools_install") => "निर्भरता स्थापित करें",
        ("hi", "adb_devices") => "ADB: जुड़े उपकरण",
        ("hi", "container_list") => "Docker/Podman: कंटेनर",
        ("hi", "kubernetes_contexts") => "Kubernetes: संदर्भ",
        ("ja", "network_status") => "ネットワーク、ルート、DNS、待受ポート",
        ("ja", "network_connections") => "ネットワーク接続",
        ("ja", "dns_flush") => "DNS キャッシュを消去",
        ("ja", "tools_status") => "依存関係とバージョン",
        ("ja", "tools_install") => "依存関係をインストール",
        ("ja", "adb_devices") => "ADB: 接続デバイス",
        ("ja", "container_list") => "Docker/Podman: コンテナ",
        ("ja", "kubernetes_contexts") => "Kubernetes: コンテキスト",
        ("ko", "network_status") => "네트워크, 경로, DNS 및 수신 포트",
        ("ko", "network_connections") => "네트워크 연결",
        ("ko", "dns_flush") => "DNS 캐시 비우기",
        ("ko", "tools_status") => "종속성 및 버전",
        ("ko", "tools_install") => "종속성 설치",
        ("ko", "adb_devices") => "ADB: 연결된 장치",
        ("ko", "container_list") => "Docker/Podman: 컨테이너",
        ("ko", "kubernetes_contexts") => "Kubernetes: 컨텍스트",
        ("ro", "network_status") => "Rețea, rute, DNS și porturi de ascultare",
        ("ro", "network_connections") => "Conexiuni de rețea",
        ("ro", "dns_flush") => "Golește memoria cache DNS",
        ("ro", "tools_status") => "Dependențe și versiuni",
        ("ro", "tools_install") => "Instalează o dependență",
        ("ro", "adb_devices") => "ADB: dispozitive conectate",
        ("ro", "container_list") => "Docker/Podman: containere",
        ("ro", "kubernetes_contexts") => "Kubernetes: contexte",
        ("ru", "network_status") => "Сеть, маршруты, DNS и прослушиваемые порты",
        ("ru", "network_connections") => "Сетевые подключения",
        ("ru", "dns_flush") => "Очистить кэш DNS",
        ("ru", "tools_status") => "Зависимости и версии",
        ("ru", "tools_install") => "Установить зависимость",
        ("ru", "adb_devices") => "ADB: подключённые устройства",
        ("ru", "container_list") => "Docker/Podman: контейнеры",
        ("ru", "kubernetes_contexts") => "Kubernetes: контексты",
        ("uk", "network_status") => "Мережа, маршрути, DNS і порти прослуховування",
        ("uk", "network_connections") => "Мережеві підключення",
        ("uk", "dns_flush") => "Очистити кеш DNS",
        ("uk", "tools_status") => "Залежності та версії",
        ("uk", "tools_install") => "Встановити залежність",
        ("uk", "adb_devices") => "ADB: підключені пристрої",
        ("uk", "container_list") => "Docker/Podman: контейнери",
        ("uk", "kubernetes_contexts") => "Kubernetes: контексти",
        ("zh", "network_status") => "网络、路由、DNS 和监听端口",
        ("zh", "network_connections") => "网络连接",
        ("zh", "dns_flush") => "清除 DNS 缓存",
        ("zh", "tools_status") => "依赖项和版本",
        ("zh", "tools_install") => "安装依赖项",
        ("zh", "adb_devices") => "ADB：已连接设备",
        ("zh", "container_list") => "Docker/Podman：容器",
        ("zh", "kubernetes_contexts") => "Kubernetes：上下文",
        (_, "network_status") => "Red, rutas, DNS y puertos escuchando",
        (_, "network_connections") => "Conexiones de red",
        (_, "dns_flush") => "Vaciar caché DNS",
        (_, "tools_status") => "Dependencias y versiones",
        (_, "tools_install") => "Instalar una dependencia",
        (_, "adb_devices") => "ADB: dispositivos conectados",
        (_, "container_list") => "Docker/Podman: contenedores",
        (_, "kubernetes_contexts") => "Kubernetes: contextos",
        _ => "",
    }
}

/// Etiquetas operativas de las páginas de Red, Arranque y Servicios. El
/// catálogo es compartido para que la GUI Win32 y la GUI Linux no diverjan
/// por idioma aunque cada plataforma ejecute comandos nativos distintos.
/// Son claves separadas de los argumentos de CLI: los identificadores y
/// valores que se pasan al backend nunca dependen del idioma de la interfaz.
pub fn system_page_text(key: &str) -> &'static str {
    const TEXT: &[(&str, [&str; 15])] = &[
        (
            "native_hardware_status",
            [
                "حالة العتاد",
                "Hardwarestatus",
                "Hardware status",
                "Estado del hardware",
                "État du matériel",
                "हार्डवेयर स्थिति",
                "Stato hardware",
                "ハードウェア状態",
                "하드웨어 상태",
                "Stan sprzętu",
                "Estado do hardware",
                "Starea hardware-ului",
                "Состояние оборудования",
                "Стан обладнання",
                "硬件状态",
            ],
        ),
        (
            "native_power_status",
            [
                "حالة الطاقة وخططها",
                "Energie- und Energiesparpläne",
                "Power status and plans",
                "Estado y planes de energía",
                "État et plans d’alimentation",
                "पावर स्थिति और योजनाएँ",
                "Stato e piani di alimentazione",
                "電源状態とプラン",
                "전원 상태 및 계획",
                "Stan zasilania i plany",
                "Estado e planos de energia",
                "Starea și planurile de alimentare",
                "Состояние питания и планы",
                "Стан живлення та плани",
                "电源状态和计划",
            ],
        ),
        (
            "native_security_status",
            [
                "حالة جدار الحماية والأمان",
                "Firewall- und Sicherheitsstatus",
                "Firewall and security status",
                "Estado del firewall y seguridad",
                "État du pare-feu et de la sécurité",
                "फ़ायरवॉल और सुरक्षा स्थिति",
                "Stato di firewall e sicurezza",
                "ファイアウォールとセキュリティ状態",
                "방화벽 및 보안 상태",
                "Stan zapory i zabezpieczeń",
                "Estado da firewall e segurança",
                "Starea firewallului și securității",
                "Состояние брандмауэра и безопасности",
                "Стан брандмауера та безпеки",
                "防火墙和安全状态",
            ],
        ),
        (
            "native_security_scanners",
            [
                "محللات التعليمات البرمجية وCI",
                "Code- und CI-Scanner",
                "Code and CI scanners",
                "Analizadores de código y CI",
                "Analyseurs de code et CI",
                "कोड और CI स्कैनर",
                "Scanner di codice e CI",
                "コードとCIスキャナー",
                "코드 및 CI 스캐너",
                "Skanery kodu i CI",
                "Scanners de código e CI",
                "Scanere de cod și CI",
                "Сканеры кода и CI",
                "Сканери коду та CI",
                "代码和 CI 扫描器",
            ],
        ),
        (
            "network_title",
            [
                "الشبكة والمسارات وDNS والمنافذ المستمعة",
                "Netzwerk, Routen, DNS und offene Ports",
                "Network, routes, DNS and listening ports",
                "Red, rutas, DNS y puertos escuchando",
                "Réseau, routes, DNS et ports en écoute",
                "नेटवर्क, रूट, DNS और सुनने वाले पोर्ट",
                "Rete, rotte, DNS e porte in ascolto",
                "ネットワーク、ルート、DNS、待受ポート",
                "네트워크, 경로, DNS 및 수신 포트",
                "Sieć, trasy, DNS i nasłuchujące porty",
                "Rede, rotas, DNS e portas de escuta",
                "Rețea, rute, DNS și porturi de ascultare",
                "Сеть, маршруты, DNS и прослушиваемые порты",
                "Мережа, маршрути, DNS і порти прослуховування",
                "网络、路由、DNS 和监听端口",
            ],
        ),
        (
            "network_inspection",
            [
                "استعلام الشبكة",
                "Netzwerkabfrage",
                "Network inspection",
                "Consulta de red",
                "Consultation du réseau",
                "नेटवर्क जानकारी",
                "Consultazione della rete",
                "ネットワークの確認",
                "네트워크 조회",
                "Informacje o sieci",
                "Consulta da rede",
                "Interogare rețea",
                "Сведения о сети",
                "Перегляд мережі",
                "网络查询",
            ],
        ),
        (
            "network_status",
            [
                "الحالة العامة للشبكة",
                "Allgemeiner Netzwerkstatus",
                "Network overview",
                "Estado general de red",
                "État général du réseau",
                "नेटवर्क की सामान्य स्थिति",
                "Stato generale della rete",
                "ネットワークの全体状態",
                "네트워크 전체 상태",
                "Ogólny stan sieci",
                "Estado geral da rede",
                "Starea generală a rețelei",
                "Общее состояние сети",
                "Загальний стан мережі",
                "网络总体状态",
            ],
        ),
        (
            "network_interfaces",
            [
                "الواجهات والعناوين",
                "Schnittstellen und Adressen",
                "Interfaces and addresses",
                "Interfaces y direcciones",
                "Interfaces et adresses",
                "इंटरफ़ेस और पते",
                "Interfacce e indirizzi",
                "インターフェースとアドレス",
                "인터페이스 및 주소",
                "Interfejsy i adresy",
                "Interfaces e endereços",
                "Interfețe și adrese",
                "Интерфейсы и адреса",
                "Інтерфейси й адреси",
                "接口和地址",
            ],
        ),
        (
            "network_routes",
            [
                "جدول التوجيه",
                "Routingtabelle",
                "Routing table",
                "Tabla de rutas",
                "Table de routage",
                "रूट तालिका",
                "Tabella di routing",
                "ルーティングテーブル",
                "라우팅 테이블",
                "Tabela routingu",
                "Tabela de rotas",
                "Tabel de rutare",
                "Таблица маршрутизации",
                "Таблиця маршрутизації",
                "路由表",
            ],
        ),
        (
            "network_dns",
            [
                "DNS ومحللات الأسماء",
                "DNS und Resolver",
                "DNS and resolvers",
                "DNS y resolutores",
                "DNS et résolveurs",
                "DNS और रिज़ॉल्वर",
                "DNS e resolver",
                "DNS とリゾルバー",
                "DNS 및 리졸버",
                "DNS i serwery rozpoznawania nazw",
                "DNS e resolvedores",
                "DNS și rezolvoare",
                "DNS и резолверы",
                "DNS і резолвери",
                "DNS 和解析器",
            ],
        ),
        (
            "network_listening",
            [
                "المنافذ قيد الاستماع",
                "Lauschende Ports",
                "Listening ports",
                "Puertos escuchando",
                "Ports en écoute",
                "सुनने वाले पोर्ट",
                "Porte in ascolto",
                "待受ポート",
                "수신 대기 포트",
                "Porty nasłuchujące",
                "Portas em escuta",
                "Porturi în ascultare",
                "Прослушиваемые порты",
                "Порти, що прослуховуються",
                "监听端口",
            ],
        ),
        (
            "network_connections",
            [
                "اتصالات NetworkManager",
                "NetworkManager-Verbindungen",
                "NetworkManager connections",
                "Conexiones NetworkManager",
                "Connexions NetworkManager",
                "NetworkManager कनेक्शन",
                "Connessioni NetworkManager",
                "NetworkManager 接続",
                "NetworkManager 연결",
                "Połączenia NetworkManager",
                "Ligações do NetworkManager",
                "Conexiuni NetworkManager",
                "Подключения NetworkManager",
                "З’єднання NetworkManager",
                "NetworkManager 连接",
            ],
        ),
        (
            "network_flush_dns",
            [
                "مسح ذاكرة DNS المؤقتة",
                "DNS-Cache leeren",
                "Flush DNS cache",
                "Vaciar caché DNS",
                "Vider le cache DNS",
                "DNS कैश साफ़ करें",
                "Svuota cache DNS",
                "DNS キャッシュを消去",
                "DNS 캐시 비우기",
                "Wyczyść pamięć DNS",
                "Limpar a cache DNS",
                "Golește memoria cache DNS",
                "Очистить кэш DNS",
                "Очистити кеш DNS",
                "清除 DNS 缓存",
            ],
        ),
        (
            "network_management",
            [
                "إدارة الاتصالات",
                "Verbindungsverwaltung",
                "Connection management",
                "Gestión de conexiones",
                "Gestion des connexions",
                "कनेक्शन प्रबंधन",
                "Gestione delle connessioni",
                "接続の管理",
                "연결 관리",
                "Zarządzanie połączeniami",
                "Gestão de ligações",
                "Gestionarea conexiunilor",
                "Управление подключениями",
                "Керування з’єднаннями",
                "连接管理",
            ],
        ),
        (
            "network_interface_manage",
            [
                "تفعيل / تعطيل الواجهة",
                "Schnittstelle aktivieren/deaktivieren",
                "Enable / disable interface",
                "Activar / desactivar interfaz",
                "Activer / désactiver l’interface",
                "इंटरफ़ेस चालू / बंद करें",
                "Attiva / disattiva interfaccia",
                "インターフェースを有効化／無効化",
                "인터페이스 활성화 / 비활성화",
                "Włącz / wyłącz interfejs",
                "Ativar / desativar interface",
                "Activează / dezactivează interfața",
                "Включить / отключить интерфейс",
                "Увімкнути / вимкнути інтерфейс",
                "启用 / 禁用接口",
            ],
        ),
        (
            "network_connect",
            [
                "الاتصال عبر NetworkManager",
                "Mit NetworkManager verbinden",
                "Connect with NetworkManager",
                "Conectar NetworkManager",
                "Se connecter avec NetworkManager",
                "NetworkManager से कनेक्ट करें",
                "Connetti con NetworkManager",
                "NetworkManager で接続",
                "NetworkManager로 연결",
                "Połącz przez NetworkManager",
                "Ligar através do NetworkManager",
                "Conectează prin NetworkManager",
                "Подключиться через NetworkManager",
                "Підключитися через NetworkManager",
                "通过 NetworkManager 连接",
            ],
        ),
        (
            "network_disconnect",
            [
                "قطع الاتصال عبر NetworkManager",
                "Mit NetworkManager trennen",
                "Disconnect with NetworkManager",
                "Desconectar NetworkManager",
                "Se déconnecter avec NetworkManager",
                "NetworkManager से डिस्कनेक्ट करें",
                "Disconnetti con NetworkManager",
                "NetworkManager を切断",
                "NetworkManager 연결 해제",
                "Rozłącz przez NetworkManager",
                "Desligar através do NetworkManager",
                "Deconectează prin NetworkManager",
                "Отключиться через NetworkManager",
                "Від’єднатися через NetworkManager",
                "通过 NetworkManager 断开",
            ],
        ),
        (
            "network_guide",
            [
                "دليل الشبكة",
                "Netzwerkleitfaden",
                "Network guide",
                "Guía de red",
                "Guide réseau",
                "नेटवर्क मार्गदर्शिका",
                "Guida alla rete",
                "ネットワークガイド",
                "네트워크 안내",
                "Przewodnik po sieci",
                "Guia de rede",
                "Ghid de rețea",
                "Руководство по сети",
                "Посібник із мережі",
                "网络指南",
            ],
        ),
        (
            "boot_title",
            [
                "الإقلاع وEFI ومحمل النظام",
                "Start, EFI und Bootloader",
                "Boot, EFI and system loader",
                "Arranque, EFI y cargador del sistema",
                "Démarrage, EFI et chargeur du système",
                "बूट, EFI और सिस्टम लोडर",
                "Avvio, EFI e bootloader di sistema",
                "起動、EFI、システムローダー",
                "부팅, EFI 및 시스템 로더",
                "Rozruch, EFI i program rozruchowy systemu",
                "Arranque, EFI e carregador do sistema",
                "Pornire, EFI și încărcătorul sistemului",
                "Загрузка, EFI и системный загрузчик",
                "Завантаження, EFI та системний завантажувач",
                "启动、EFI 和系统引导程序",
            ],
        ),
        (
            "boot_inspection",
            [
                "فحص الإقلاع",
                "Startprüfung",
                "Boot inspection",
                "Inspección del arranque",
                "Inspection du démarrage",
                "बूट की जाँच",
                "Controllo dell’avvio",
                "起動の確認",
                "부팅 검사",
                "Kontrola rozruchu",
                "Inspeção do arranque",
                "Inspectare pornire",
                "Проверка загрузки",
                "Перевірка завантаження",
                "启动检查",
            ],
        ),
        (
            "boot_status",
            [
                "الحالة العامة للإقلاع",
                "Allgemeiner Startstatus",
                "Boot overview",
                "Estado general del arranque",
                "État général du démarrage",
                "बूट की सामान्य स्थिति",
                "Stato generale dell’avvio",
                "起動の全体状態",
                "부팅 전체 상태",
                "Ogólny stan rozruchu",
                "Estado geral do arranque",
                "Starea generală a pornirii",
                "Общее состояние загрузки",
                "Загальний стан завантаження",
                "启动总体状态",
            ],
        ),
        (
            "efi_entries",
            [
                "إدخالات EFI / NVRAM",
                "EFI-/NVRAM-Einträge",
                "EFI / NVRAM entries",
                "Entradas EFI / NVRAM",
                "Entrées EFI / NVRAM",
                "EFI / NVRAM प्रविष्टियाँ",
                "Voci EFI / NVRAM",
                "EFI / NVRAM エントリー",
                "EFI / NVRAM 항목",
                "Wpisy EFI / NVRAM",
                "Registos EFI / NVRAM",
                "Intrări EFI / NVRAM",
                "Записи EFI / NVRAM",
                "Записи EFI / NVRAM",
                "EFI / NVRAM 条目",
            ],
        ),
        (
            "grub_entries",
            [
                "إدخالات GRUB",
                "GRUB-Einträge",
                "GRUB entries",
                "Entradas GRUB",
                "Entrées GRUB",
                "GRUB प्रविष्टियाँ",
                "Voci GRUB",
                "GRUB エントリー",
                "GRUB 항목",
                "Wpisy GRUB",
                "Entradas GRUB",
                "Intrări GRUB",
                "Записи GRUB",
                "Записи GRUB",
                "GRUB 条目",
            ],
        ),
        (
            "systemd_boot",
            [
                "حالة systemd-boot",
                "systemd-boot-Status",
                "systemd-boot status",
                "Estado de systemd-boot",
                "État de systemd-boot",
                "systemd-boot की स्थिति",
                "Stato di systemd-boot",
                "systemd-boot の状態",
                "systemd-boot 상태",
                "Stan systemd-boot",
                "Estado do systemd-boot",
                "Starea systemd-boot",
                "Состояние systemd-boot",
                "Стан systemd-boot",
                "systemd-boot 状态",
            ],
        ),
        (
            "secure_boot",
            [
                "حالة Secure Boot",
                "Secure-Boot-Status",
                "Secure Boot status",
                "Estado de Secure Boot",
                "État de Secure Boot",
                "Secure Boot की स्थिति",
                "Stato di Secure Boot",
                "Secure Boot の状態",
                "Secure Boot 상태",
                "Stan Secure Boot",
                "Estado do Secure Boot",
                "Starea Secure Boot",
                "Состояние Secure Boot",
                "Стан Secure Boot",
                "Secure Boot 状态",
            ],
        ),
        (
            "boot_plan",
            [
                "إنشاء خطة آمنة",
                "Sicheren Plan erstellen",
                "Generate safe plan",
                "Generar plan seguro",
                "Générer un plan sûr",
                "सुरक्षित योजना बनाएँ",
                "Genera un piano sicuro",
                "安全なプランを作成",
                "안전한 계획 만들기",
                "Utwórz bezpieczny plan",
                "Gerar plano seguro",
                "Generează un plan sigur",
                "Создать безопасный план",
                "Створити безпечний план",
                "生成安全计划",
            ],
        ),
        (
            "boot_next_changes",
            [
                "تغييرات الإقلاع التالي",
                "Änderungen beim nächsten Start",
                "Next-boot changes",
                "Cambios del siguiente arranque",
                "Modifications au prochain démarrage",
                "अगले बूट के बदलाव",
                "Modifiche al prossimo avvio",
                "次回起動時の変更",
                "다음 부팅 변경 사항",
                "Zmiany przy następnym rozruchu",
                "Alterações no próximo arranque",
                "Modificări la următoarea pornire",
                "Изменения при следующей загрузке",
                "Зміни під час наступного завантаження",
                "下次启动时的更改",
            ],
        ),
        (
            "grub_schedule",
            [
                "جدولة إدخال GRUB التالي",
                "Nächsten GRUB-Eintrag planen",
                "Schedule next GRUB entry",
                "Programar siguiente entrada GRUB",
                "Programmer la prochaine entrée GRUB",
                "अगली GRUB प्रविष्टि निर्धारित करें",
                "Programma la prossima voce GRUB",
                "次回の GRUB エントリーを予約",
                "다음 GRUB 항목 예약",
                "Zaplanuj następny wpis GRUB",
                "Agendar a próxima entrada GRUB",
                "Programează următoarea intrare GRUB",
                "Назначить следующую запись GRUB",
                "Запланувати наступний запис GRUB",
                "安排下一个 GRUB 条目",
            ],
        ),
        (
            "grub_cancel",
            [
                "إلغاء إدخال GRUB التالي",
                "Nächsten GRUB-Eintrag abbrechen",
                "Cancel next GRUB entry",
                "Cancelar siguiente entrada GRUB",
                "Annuler la prochaine entrée GRUB",
                "अगली GRUB प्रविष्टि रद्द करें",
                "Annulla la prossima voce GRUB",
                "次回の GRUB エントリーを取り消す",
                "다음 GRUB 항목 취소",
                "Anuluj następny wpis GRUB",
                "Cancelar a próxima entrada GRUB",
                "Anulează următoarea intrare GRUB",
                "Отменить следующую запись GRUB",
                "Скасувати наступний запис GRUB",
                "取消下一个 GRUB 条目",
            ],
        ),
        (
            "boot_guide",
            [
                "دليل الإقلاع وEFI وGRUB",
                "Leitfaden zu Start, EFI und GRUB",
                "Boot, EFI and GRUB guide",
                "Guía de arranque, EFI y GRUB",
                "Guide du démarrage, EFI et GRUB",
                "बूट, EFI और GRUB मार्गदर्शिका",
                "Guida ad avvio, EFI e GRUB",
                "起動、EFI、GRUB のガイド",
                "부팅, EFI 및 GRUB 안내",
                "Przewodnik po rozruchu, EFI i GRUB",
                "Guia de arranque, EFI e GRUB",
                "Ghid pentru pornire, EFI și GRUB",
                "Руководство по загрузке, EFI и GRUB",
                "Посібник із завантаження, EFI та GRUB",
                "启动、EFI 和 GRUB 指南",
            ],
        ),
        (
            "services_title",
            [
                "خدمات النظام",
                "Systemdienste",
                "System services",
                "Servicios del sistema",
                "Services système",
                "सिस्टम सेवाएँ",
                "Servizi di sistema",
                "システムサービス",
                "시스템 서비스",
                "Usługi systemowe",
                "Serviços do sistema",
                "Servicii de sistem",
                "Системные службы",
                "Системні служби",
                "系统服务",
            ],
        ),
        (
            "services_inventory",
            [
                "الجرد حسب النطاق وبدء التشغيل",
                "Inventar nach Bereich und Starttyp",
                "Inventory by scope and startup type",
                "Inventario por ámbito y arranque",
                "Inventaire par portée et type de démarrage",
                "दायरे और स्टार्टअप प्रकार के अनुसार सूची",
                "Inventario per ambito e avvio",
                "範囲と起動方式別の一覧",
                "범위 및 시작 유형별 목록",
                "Spis według zakresu i typu uruchamiania",
                "Inventário por âmbito e tipo de arranque",
                "Inventar după domeniu și tip de pornire",
                "Инвентаризация по области и типу запуска",
                "Перелік за областю та типом запуску",
                "按范围和启动类型查看",
            ],
        ),
        (
            "services_automatic",
            [
                "تلقائية وثابتة (النظام)",
                "Automatisch und statisch (System)",
                "Automatic and static (system)",
                "Automáticos y estáticos (system)",
                "Automatiques et statiques (système)",
                "स्वचालित और स्थिर (सिस्टम)",
                "Automatici e statici (sistema)",
                "自動・静的（システム）",
                "자동 및 정적 (시스템)",
                "Automatyczne i statyczne (system)",
                "Automáticos e estáticos (sistema)",
                "Automatice și statice (sistem)",
                "Автоматические и статические (система)",
                "Автоматичні та статичні (система)",
                "自动和静态（系统）",
            ],
        ),
        (
            "services_manual",
            [
                "يدوية / معطلة (النظام)",
                "Manuell / deaktiviert (System)",
                "Manual / disabled (system)",
                "Manuales / desactivados (system)",
                "Manuels / désactivés (système)",
                "मैन्युअल / अक्षम (सिस्टम)",
                "Manuali / disabilitati (sistema)",
                "手動／無効（システム）",
                "수동 / 비활성화 (시스템)",
                "Ręczne / wyłączone (system)",
                "Manuais / desativados (sistema)",
                "Manuale / dezactivate (sistem)",
                "Вручную / отключённые (система)",
                "Вручну / вимкнені (система)",
                "手动 / 已禁用（系统）",
            ],
        ),
        (
            "services_user",
            [
                "خدمات المستخدم",
                "Benutzerdienste",
                "User services",
                "Servicios del usuario",
                "Services utilisateur",
                "उपयोगकर्ता सेवाएँ",
                "Servizi utente",
                "ユーザーサービス",
                "사용자 서비스",
                "Usługi użytkownika",
                "Serviços do utilizador",
                "Servicii de utilizator",
                "Службы пользователя",
                "Служби користувача",
                "用户服务",
            ],
        ),
        (
            "services_both",
            [
                "الكل: النظام والمستخدم",
                "Alle: System und Benutzer",
                "All: system and user",
                "Todos: sistema y usuario",
                "Tout : système et utilisateur",
                "सभी: सिस्टम और उपयोगकर्ता",
                "Tutti: sistema e utente",
                "すべて：システムとユーザー",
                "전체: 시스템 및 사용자",
                "Wszystkie: system i użytkownik",
                "Todos: sistema e utilizador",
                "Toate: sistem și utilizator",
                "Все: система и пользователь",
                "Усі: система та користувач",
                "全部：系统和用户",
            ],
        ),
        (
            "services_failed",
            [
                "الخدمات المتوقفة والسجل",
                "Fehlgeschlagene Dienste und Journal",
                "Failed services and journal",
                "Servicios fallidos y journal",
                "Services en échec et journal",
                "विफल सेवाएँ और जर्नल",
                "Servizi non riusciti e journal",
                "失敗したサービスとジャーナル",
                "실패한 서비스 및 저널",
                "Usługi zakończone błędem i dziennik",
                "Serviços falhados e journal",
                "Servicii eșuate și jurnal",
                "Сбойные службы и журнал",
                "Несправні служби та журнал",
                "失败的服务和日志",
            ],
        ),
        (
            "services_management",
            [
                "إدارة وتصدير",
                "Verwaltung und Export",
                "Management and export",
                "Gestión y exportación",
                "Gestion et exportation",
                "प्रबंधन और निर्यात",
                "Gestione ed esportazione",
                "管理とエクスポート",
                "관리 및 내보내기",
                "Zarządzanie i eksport",
                "Gestão e exportação",
                "Gestionare și export",
                "Управление и экспорт",
                "Керування та експорт",
                "管理和导出",
            ],
        ),
        (
            "services_manage",
            [
                "إدارة خدمة",
                "Dienst verwalten",
                "Manage service",
                "Gestionar servicio",
                "Gérer un service",
                "सेवा प्रबंधित करें",
                "Gestisci servizio",
                "サービスを管理",
                "서비스 관리",
                "Zarządzaj usługą",
                "Gerir serviço",
                "Gestionează serviciul",
                "Управление службой",
                "Керувати службою",
                "管理服务",
            ],
        ),
        (
            "services_export",
            [
                "تصدير التقرير الكامل",
                "Vollständigen Bericht exportieren",
                "Export full report",
                "Exportar informe completo",
                "Exporter le rapport complet",
                "पूरी रिपोर्ट निर्यात करें",
                "Esporta rapporto completo",
                "完全なレポートをエクスポート",
                "전체 보고서 내보내기",
                "Eksportuj pełny raport",
                "Exportar relatório completo",
                "Exportă raportul complet",
                "Экспортировать полный отчёт",
                "Експортувати повний звіт",
                "导出完整报告",
            ],
        ),
        (
            "services_guide",
            [
                "دليل الخدمات",
                "Dienstleitfaden",
                "Services guide",
                "Guía de servicios",
                "Guide des services",
                "सेवा मार्गदर्शिका",
                "Guida ai servizi",
                "サービスガイド",
                "서비스 안내",
                "Przewodnik po usługach",
                "Guia de serviços",
                "Ghid pentru servicii",
                "Руководство по службам",
                "Посібник зі служб",
                "服务指南",
            ],
        ),
    ];

    let language_index = SUPPORTED
        .iter()
        .position(|language| *language == current())
        .unwrap_or(3);
    TEXT.iter()
        .find(|(text_key, _)| *text_key == key)
        .map(|(_, values)| values[language_index])
        .unwrap_or("")
}

/// Localiza las etiquetas de campos nativos compartidas por las páginas de
/// Red, Arranque y Servicios y por sus guías contextuales.
#[cfg(not(windows))]
pub fn gui_native_prompt(prompt: &str) -> &str {
    const PROMPTS: &[(&str, [&str; 15])] = &[
        (
            "Interfaz de red (ej. eth0)",
            [
                "واجهة الشبكة (مثال: eth0)",
                "Netzwerkschnittstelle (z. B. eth0)",
                "Network interface (e.g. eth0)",
                "Interfaz de red (ej. eth0)",
                "Interface réseau (ex. eth0)",
                "नेटवर्क इंटरफ़ेस (जैसे eth0)",
                "Interfaccia di rete (es. eth0)",
                "ネットワークインターフェース（例: eth0）",
                "네트워크 인터페이스 (예: eth0)",
                "Interfejs sieciowy (np. eth0)",
                "Interface de rede (ex.: eth0)",
                "Interfață de rețea (ex. eth0)",
                "Сетевой интерфейс (например, eth0)",
                "Мережевий інтерфейс (наприклад, eth0)",
                "网络接口（例如 eth0）",
            ],
        ),
        (
            "Estado: up o down",
            [
                "الحالة: up أو down",
                "Status: up oder down",
                "State: up or down",
                "Estado: up o down",
                "État : up ou down",
                "स्थिति: up या down",
                "Stato: up o down",
                "状態: up または down",
                "상태: up 또는 down",
                "Stan: up lub down",
                "Estado: up ou down",
                "Stare: up sau down",
                "Состояние: up или down",
                "Стан: up або down",
                "状态：up 或 down",
            ],
        ),
        (
            "Nombre exacto de la conexión NetworkManager",
            [
                "الاسم الدقيق لاتصال NetworkManager",
                "Exakter Name der NetworkManager-Verbindung",
                "Exact NetworkManager connection name",
                "Nombre exacto de la conexión NetworkManager",
                "Nom exact de la connexion NetworkManager",
                "NetworkManager कनेक्शन का सटीक नाम",
                "Nome esatto della connessione NetworkManager",
                "NetworkManager 接続の正確な名前",
                "NetworkManager 연결의 정확한 이름",
                "Dokładna nazwa połączenia NetworkManager",
                "Nome exato da ligação NetworkManager",
                "Numele exact al conexiunii NetworkManager",
                "Точное имя подключения NetworkManager",
                "Точна назва з’єднання NetworkManager",
                "NetworkManager 连接的准确名称",
            ],
        ),
        (
            "Título exacto de la entrada GRUB (ej. Ubuntu)",
            [
                "العنوان الدقيق لإدخال GRUB (مثال: Ubuntu)",
                "Exakter Titel des GRUB-Eintrags (z. B. Ubuntu)",
                "Exact GRUB entry title (e.g. Ubuntu)",
                "Título exacto de la entrada GRUB (ej. Ubuntu)",
                "Titre exact de l’entrée GRUB (ex. Ubuntu)",
                "GRUB प्रविष्टि का सटीक शीर्षक (जैसे Ubuntu)",
                "Titolo esatto della voce GRUB (es. Ubuntu)",
                "GRUB エントリーの正確なタイトル（例: Ubuntu）",
                "GRUB 항목의 정확한 제목 (예: Ubuntu)",
                "Dokładny tytuł wpisu GRUB (np. Ubuntu)",
                "Título exato da entrada GRUB (ex.: Ubuntu)",
                "Titlul exact al intrării GRUB (ex. Ubuntu)",
                "Точное название записи GRUB (например, Ubuntu)",
                "Точна назва запису GRUB (наприклад, Ubuntu)",
                "GRUB 条目的准确标题（例如 Ubuntu）",
            ],
        ),
        (
            "Ámbito: system o user",
            [
                "النطاق: system أو user",
                "Bereich: system oder user",
                "Scope: system or user",
                "Ámbito: system o user",
                "Portée : system ou user",
                "दायरा: system या user",
                "Ambito: system o user",
                "範囲: system または user",
                "범위: system 또는 user",
                "Zakres: system lub user",
                "Âmbito: system ou user",
                "Domeniu: system sau user",
                "Область: system или user",
                "Область: system або user",
                "范围：system 或 user",
            ],
        ),
        (
            "Unidad (ej. sshd.service)",
            [
                "الوحدة (مثال: sshd.service)",
                "Unit (z. B. sshd.service)",
                "Unit (e.g. sshd.service)",
                "Unidad (ej. sshd.service)",
                "Unité (ex. sshd.service)",
                "यूनिट (जैसे sshd.service)",
                "Unità (es. sshd.service)",
                "ユニット（例: sshd.service）",
                "유닛 (예: sshd.service)",
                "Jednostka (np. sshd.service)",
                "Unidade (ex.: sshd.service)",
                "Unitate (ex. sshd.service)",
                "Юнит (например, sshd.service)",
                "Юніт (наприклад, sshd.service)",
                "单元（例如 sshd.service）",
            ],
        ),
        (
            "Acción: status, start, stop, restart, enable, disable, mask o unmask",
            [
                "الإجراء: status أو start أو stop أو restart أو enable أو disable أو mask أو unmask",
                "Aktion: status, start, stop, restart, enable, disable, mask oder unmask",
                "Action: status, start, stop, restart, enable, disable, mask or unmask",
                "Acción: status, start, stop, restart, enable, disable, mask o unmask",
                "Action : status, start, stop, restart, enable, disable, mask ou unmask",
                "कार्रवाई: status, start, stop, restart, enable, disable, mask या unmask",
                "Azione: status, start, stop, restart, enable, disable, mask o unmask",
                "操作: status、start、stop、restart、enable、disable、mask、unmask",
                "작업: status, start, stop, restart, enable, disable, mask 또는 unmask",
                "Działanie: status, start, stop, restart, enable, disable, mask lub unmask",
                "Ação: status, start, stop, restart, enable, disable, mask ou unmask",
                "Acțiune: status, start, stop, restart, enable, disable, mask sau unmask",
                "Действие: status, start, stop, restart, enable, disable, mask или unmask",
                "Дія: status, start, stop, restart, enable, disable, mask або unmask",
                "操作：status、start、stop、restart、enable、disable、mask 或 unmask",
            ],
        ),
    ];
    let language_index = SUPPORTED
        .iter()
        .position(|language| *language == current())
        .unwrap_or(3);
    PROMPTS
        .iter()
        .find(|(source, _)| *source == prompt)
        .map(|(_, values)| values[language_index])
        .unwrap_or(prompt)
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
    if matches!(key, "update_check" | "update_download") {
        return update_text(key);
    }
    if matches!(
        key,
        "settings_apply"
            | "settings_guide"
            | "elevation_default"
            | "elevation_enabled"
            | "elevation_disabled"
            | "elevation_scope"
            | "elevation_prompt"
            | "elevation_save_error"
    ) {
        return match (current(), key) {
            ("en", "settings_apply") => "Apply settings",
            ("en", "settings_guide") => "Settings and visibility guide",
            ("en", "elevation_default") => "Elevate modifying actions by default (sudo/UAC; not queries or user-data actions)",
            ("en", "elevation_enabled") => "enabled",
            ("en", "elevation_disabled") => "disabled",
            ("en", "elevation_scope") => "Queries, Git/Wine and the user's trash are never elevated automatically.",
            ("en", "elevation_prompt") => "Elevate modifying actions by default with sudo/UAC? (y/n; queries, Git, Wine and the user's trash are never elevated automatically)",
            ("en", "elevation_save_error") => "Could not save the default elevation:",
            ("de", "settings_apply") => "Einstellungen übernehmen",
            ("de", "settings_guide") => "Hilfe zu Einstellungen und Sichtbarkeit",
            ("de", "elevation_default") => "Ändernde Aktionen standardmäßig erhöht ausführen (sudo/UAC; keine Abfragen oder Benutzerdateiaktionen)",
            ("de", "elevation_enabled") => "aktiviert",
            ("de", "elevation_disabled") => "deaktiviert",
            ("de", "elevation_scope") => "Abfragen, Git/Wine und der Papierkorb des Benutzers werden nie automatisch erhöht.",
            ("de", "elevation_prompt") => "Ändernde Aktionen standardmäßig mit sudo/UAC erhöhen? (j/n; Abfragen, Git, Wine und der Papierkorb des Benutzers werden nie automatisch erhöht)",
            ("de", "elevation_save_error") => "Die Standarderhöhung konnte nicht gespeichert werden:",
            ("fr", "settings_apply") => "Appliquer les réglages",
            ("fr", "settings_guide") => "Guide des réglages et de la visibilité",
            ("fr", "elevation_default") => "Élever les actions modificatrices par défaut (sudo/UAC ; pas les consultations ni les actions sur les données utilisateur)",
            ("fr", "elevation_enabled") => "activée",
            ("fr", "elevation_disabled") => "désactivée",
            ("fr", "elevation_scope") => "Les consultations, Git/Wine et la corbeille de l’utilisateur ne sont jamais élevés automatiquement.",
            ("fr", "elevation_prompt") => "Élever les actions modificatrices par défaut avec sudo/UAC ? (o/n ; les consultations, Git, Wine et la corbeille de l’utilisateur ne sont jamais élevés automatiquement)",
            ("fr", "elevation_save_error") => "Impossible d’enregistrer l’élévation par défaut :",
            ("pt", "settings_apply") => "Aplicar definições",
            ("pt", "settings_guide") => "Guia de definições e visibilidade",
            ("pt", "elevation_default") => "Elevar ações modificadoras por predefinição (sudo/UAC; não consultas nem ações sobre dados do utilizador)",
            ("pt", "elevation_enabled") => "ativada",
            ("pt", "elevation_disabled") => "desativada",
            ("pt", "elevation_scope") => "As consultas, Git/Wine e a reciclagem do utilizador nunca são elevadas automaticamente.",
            ("pt", "elevation_prompt") => "Elevar ações modificadoras por predefinição com sudo/UAC? (s/n; consultas, Git, Wine e a reciclagem do utilizador nunca são elevadas automaticamente)",
            ("pt", "elevation_save_error") => "Não foi possível guardar a elevação predefinida:",
            ("it", "settings_apply") => "Applica impostazioni",
            ("it", "settings_guide") => "Guida a impostazioni e visibilità",
            ("it", "elevation_default") => "Eleva per impostazione predefinita le azioni che modificano il sistema (sudo/UAC; non consultazioni o dati utente)",
            ("it", "elevation_enabled") => "attivata",
            ("it", "elevation_disabled") => "disattivata",
            ("it", "elevation_scope") => "Le consultazioni, Git/Wine e il cestino dell’utente non vengono mai elevati automaticamente.",
            ("it", "elevation_prompt") => "Elevare per impostazione predefinita le azioni modificative con sudo/UAC? (s/n; consultazioni, Git, Wine e cestino dell’utente non vengono mai elevati automaticamente)",
            ("it", "elevation_save_error") => "Impossibile salvare l’elevazione predefinita:",
            ("pl", "settings_apply") => "Zastosuj ustawienia",
            ("pl", "settings_guide") => "Przewodnik po ustawieniach i widoczności",
            ("pl", "elevation_default") => "Domyślnie uruchamiaj działania modyfikujące z podwyższonymi uprawnieniami (sudo/UAC; nie zapytania ani działania na danych użytkownika)",
            ("pl", "elevation_enabled") => "włączona",
            ("pl", "elevation_disabled") => "wyłączona",
            ("pl", "elevation_scope") => "Zapytania, Git/Wine i kosz użytkownika nigdy nie są automatycznie uruchamiane z podwyższonymi uprawnieniami.",
            ("pl", "elevation_prompt") => "Uruchamiać domyślnie działania modyfikujące z sudo/UAC? (t/n; zapytania, Git, Wine i kosz użytkownika nigdy nie są automatycznie podwyższane)",
            ("pl", "elevation_save_error") => "Nie można zapisać domyślnego podwyższenia uprawnień:",
            ("ar", "settings_apply") => "تطبيق الإعدادات",
            ("ar", "settings_guide") => "دليل الإعدادات والظهور",
            ("ar", "elevation_default") => "رفع صلاحيات الإجراءات المعدِّلة افتراضيًا (sudo/UAC؛ وليس الاستعلامات أو إجراءات بيانات المستخدم)",
            ("ar", "elevation_enabled") => "مفعّلة",
            ("ar", "elevation_disabled") => "معطّلة",
            ("ar", "elevation_scope") => "لا تُرفع صلاحيات الاستعلامات وGit/Wine وسلة مهملات المستخدم تلقائيًا أبدًا.",
            ("ar", "elevation_prompt") => "رفع صلاحيات الإجراءات المعدِّلة افتراضيًا باستخدام sudo/UAC؟ (ن/ل؛ لا تُرفع صلاحيات الاستعلامات وGit وWine وسلة مهملات المستخدم تلقائيًا أبدًا)",
            ("ar", "elevation_save_error") => "تعذّر حفظ الرفع الافتراضي للصلاحيات:",
            ("hi", "settings_apply") => "सेटिंग लागू करें",
            ("hi", "settings_guide") => "सेटिंग और दृश्यता मार्गदर्शिका",
            ("hi", "elevation_default") => "संशोधन करने वाली कार्रवाइयों को डिफ़ॉल्ट रूप से उन्नत करें (sudo/UAC; क्वेरी या उपयोगकर्ता-डेटा कार्रवाइयाँ नहीं)",
            ("hi", "elevation_enabled") => "सक्रिय",
            ("hi", "elevation_disabled") => "निष्क्रिय",
            ("hi", "elevation_scope") => "क्वेरी, Git/Wine और उपयोगकर्ता की ट्रैश अपने-आप उन्नत नहीं होती।",
            ("hi", "elevation_prompt") => "संशोधन करने वाली कार्रवाइयों को sudo/UAC के साथ डिफ़ॉल्ट रूप से उन्नत करें? (y/n; क्वेरी, Git, Wine और उपयोगकर्ता की ट्रैश अपने-आप उन्नत नहीं होती)",
            ("hi", "elevation_save_error") => "डिफ़ॉल्ट उन्नयन सहेजा नहीं जा सका:",
            ("ja", "settings_apply") => "設定を適用",
            ("ja", "settings_guide") => "設定と表示のガイド",
            ("ja", "elevation_default") => "変更操作を既定で昇格して実行（sudo/UAC。照会やユーザーデータ操作は除く）",
            ("ja", "elevation_enabled") => "有効",
            ("ja", "elevation_disabled") => "無効",
            ("ja", "elevation_scope") => "照会、Git/Wine、ユーザーのごみ箱は自動的に昇格されません。",
            ("ja", "elevation_prompt") => "変更操作を sudo/UAC で既定の昇格実行にしますか？（y/n。照会、Git、Wine、ユーザーのごみ箱は自動昇格されません）",
            ("ja", "elevation_save_error") => "既定の昇格設定を保存できませんでした：",
            ("ko", "settings_apply") => "설정 적용",
            ("ko", "settings_guide") => "설정 및 표시 안내",
            ("ko", "elevation_default") => "수정 작업을 기본적으로 관리자 권한으로 실행 (sudo/UAC; 조회 및 사용자 데이터 작업 제외)",
            ("ko", "elevation_enabled") => "활성화됨",
            ("ko", "elevation_disabled") => "비활성화됨",
            ("ko", "elevation_scope") => "조회, Git/Wine 및 사용자 휴지통은 자동으로 권한 상승되지 않습니다.",
            ("ko", "elevation_prompt") => "수정 작업을 기본적으로 sudo/UAC로 권한 상승하여 실행할까요? (y/n; 조회, Git, Wine 및 사용자 휴지통은 자동 상승하지 않음)",
            ("ko", "elevation_save_error") => "기본 권한 상승을 저장할 수 없습니다:",
            ("ro", "settings_apply") => "Aplică setările",
            ("ro", "settings_guide") => "Ghid pentru setări și vizibilitate",
            ("ro", "elevation_default") => "Rulează implicit cu privilegii ridicate acțiunile modificatoare (sudo/UAC; nu și interogările sau acțiunile asupra datelor utilizatorului)",
            ("ro", "elevation_enabled") => "activată",
            ("ro", "elevation_disabled") => "dezactivată",
            ("ro", "elevation_scope") => "Interogările, Git/Wine și coșul de gunoi al utilizatorului nu sunt ridicate automat.",
            ("ro", "elevation_prompt") => "Rulezi implicit acțiunile modificatoare cu sudo/UAC? (d/n; interogările, Git, Wine și coșul de gunoi al utilizatorului nu sunt ridicate automat)",
            ("ro", "elevation_save_error") => "Ridicarea implicită nu a putut fi salvată:",
            ("ru", "settings_apply") => "Применить настройки",
            ("ru", "settings_guide") => "Руководство по настройкам и видимости",
            ("ru", "elevation_default") => "По умолчанию запускать изменяющие действия с повышенными правами (sudo/UAC; не запросы и не действия с данными пользователя)",
            ("ru", "elevation_enabled") => "включена",
            ("ru", "elevation_disabled") => "отключена",
            ("ru", "elevation_scope") => "Запросы, Git/Wine и корзина пользователя никогда не получают права автоматически.",
            ("ru", "elevation_prompt") => "Запускать изменяющие действия по умолчанию с sudo/UAC? (д/н; запросы, Git, Wine и корзина пользователя автоматически не повышаются)",
            ("ru", "elevation_save_error") => "Не удалось сохранить повышение по умолчанию:",
            ("uk", "settings_apply") => "Застосувати налаштування",
            ("uk", "settings_guide") => "Посібник із налаштувань і видимості",
            ("uk", "elevation_default") => "Типово запускати дії зі зміною системи з підвищеними правами (sudo/UAC; не запити й не дії з даними користувача)",
            ("uk", "elevation_enabled") => "увімкнено",
            ("uk", "elevation_disabled") => "вимкнено",
            ("uk", "elevation_scope") => "Запити, Git/Wine і кошик користувача ніколи не підвищуються автоматично.",
            ("uk", "elevation_prompt") => "Типово запускати дії зі зміною системи через sudo/UAC? (т/н; запити, Git, Wine і кошик користувача автоматично не підвищуються)",
            ("uk", "elevation_save_error") => "Не вдалося зберегти типове підвищення:",
            ("zh", "settings_apply") => "应用设置",
            ("zh", "settings_guide") => "设置与可见性指南",
            ("zh", "elevation_default") => "默认提升修改操作的权限（sudo/UAC；查询和用户数据操作除外）",
            ("zh", "elevation_enabled") => "已启用",
            ("zh", "elevation_disabled") => "已禁用",
            ("zh", "elevation_scope") => "查询、Git/Wine 和用户回收站不会自动提升权限。",
            ("zh", "elevation_prompt") => "是否默认使用 sudo/UAC 提升修改操作？（y/n；查询、Git、Wine 和用户回收站不会自动提升）",
            ("zh", "elevation_save_error") => "无法保存默认提升设置：",
            (_, "settings_apply") => "Aplicar ajustes",
            (_, "settings_guide") => "Guía de ajustes y visibilidad",
            (_, "elevation_default") => "Elevar acciones modificadoras por defecto (sudo/UAC; no consultas ni acciones de datos de usuario)",
            (_, "elevation_enabled") => "activada",
            (_, "elevation_disabled") => "desactivada",
            (_, "elevation_scope") => "Las consultas, Git/Wine y la papelera del usuario nunca se elevan automáticamente.",
            (_, "elevation_prompt") => "¿Elevar acciones modificadoras por defecto con sudo/UAC? (s/n; las consultas, Git, Wine y la papelera del usuario nunca se elevan automáticamente)",
            (_, "elevation_save_error") => "No se pudo guardar la elevación por defecto:",
            (_, _) => "",
        };
    }
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
        ("en", "failed") => "Failed",
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
        ("de", "failed") => "Fehlgeschlagen",
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
        ("fr", "failed") => "Échec",
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
        ("pt", "failed") => "Falhou",
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
        ("it", "failed") => "Non riuscito",
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
        ("ca", "failed") => "Ha fallat",
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
        ("nl", "failed") => "Mislukt",
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
        ("pl", "failed") => "Niepowodzenie",
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
        ("ar", "failed") => "فشل",
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
        ("hi", "failed") => "विफल",
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
        ("ja", "failed") => "失敗",
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
        ("ko", "failed") => "실패",
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
        ("ro", "failed") => "Eșuat",
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
        ("ru", "failed") => "Не удалось",
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
        ("uk", "failed") => "Не вдалося",
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
        ("zh", "failed") => "失败",
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
        (_, "failed") => "Falló",
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
        (_, "automation_new_name") => "Nuevo nombre (solo al editar)",
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
        ("en", "yes") => "Yes",
        ("en", "no") => "No",
        ("en", "cancel") => "Cancel",
        ("en", "cancelling") => "Cancelling…",
        ("de", "yes") => "Ja",
        ("de", "no") => "Nein",
        ("de", "cancel") => "Abbrechen",
        ("de", "cancelling") => "Wird abgebrochen…",
        ("fr", "yes") => "Oui",
        ("fr", "no") => "Non",
        ("fr", "cancel") => "Annuler",
        ("fr", "cancelling") => "Annulation…",
        ("pt", "yes") => "Sim",
        ("pt", "no") => "Não",
        ("pt", "cancel") => "Cancelar",
        ("pt", "cancelling") => "A cancelar…",
        ("it", "yes") => "Sì",
        ("it", "no") => "No",
        ("it", "cancel") => "Annulla",
        ("it", "cancelling") => "Annullamento…",
        ("ca", "yes") => "Sí",
        ("ca", "no") => "No",
        ("ca", "cancel") => "Cancel·lar",
        ("ca", "cancelling") => "Cancel·lant…",
        ("nl", "yes") => "Ja",
        ("nl", "no") => "Nee",
        ("nl", "cancel") => "Annuleren",
        ("nl", "cancelling") => "Annuleren…",
        ("pl", "yes") => "Tak",
        ("pl", "no") => "Nie",
        ("pl", "cancel") => "Anuluj",
        ("pl", "cancelling") => "Anulowanie…",
        ("ar", "yes") => "نعم",
        ("ar", "no") => "لا",
        ("ar", "cancel") => "إلغاء",
        ("ar", "cancelling") => "جارٍ الإلغاء…",
        ("hi", "yes") => "हाँ",
        ("hi", "no") => "नहीं",
        ("hi", "cancel") => "रद्द करें",
        ("hi", "cancelling") => "रद्द किया जा रहा है…",
        ("ja", "yes") => "はい",
        ("ja", "no") => "いいえ",
        ("ja", "cancel") => "キャンセル",
        ("ja", "cancelling") => "キャンセル中…",
        ("ko", "yes") => "예",
        ("ko", "no") => "아니요",
        ("ko", "cancel") => "취소",
        ("ko", "cancelling") => "취소 중…",
        ("ro", "yes") => "Da",
        ("ro", "no") => "Nu",
        ("ro", "cancel") => "Anulează",
        ("ro", "cancelling") => "Se anulează…",
        ("ru", "yes") => "Да",
        ("ru", "no") => "Нет",
        ("ru", "cancel") => "Отмена",
        ("ru", "cancelling") => "Отмена…",
        ("uk", "yes") => "Так",
        ("uk", "no") => "Ні",
        ("uk", "cancel") => "Скасувати",
        ("uk", "cancelling") => "Скасування…",
        ("zh", "yes") => "是",
        ("zh", "no") => "否",
        ("zh", "cancel") => "取消",
        ("zh", "cancelling") => "正在取消…",
        (_, "yes") => "Sí",
        (_, "no") => "No",
        (_, "cancel") => "Cancelar",
        (_, "cancelling") => "Cancelando…",
        _ => "",
    }
}

/// Avisos de confirmación de la GUI. No dependen del locale del sistema:
/// siguen el idioma seleccionado dentro de LTools.
pub fn gui_confirmation_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "warning_data") => "This action may modify or destroy data.",
        ("en", "warning_system") => "This action may change system settings or data.",
        ("en", "review") => "Review the details and confirm to continue.",
        ("en", "details") => "Details",
        ("en", "boot_no_reboot") => "The computer will not restart automatically.",
        ("en", "cleanup_warning") => "The cleaner will inspect caches, temporary files and optional personal paths. Downloads, Documents, applications and Trash will not be deleted without explicit consent; items can be reviewed individually.",
        ("de", "warning_data") => "Diese Aktion kann Daten ändern oder zerstören.",
        ("de", "warning_system") => "Diese Aktion kann Systemeinstellungen oder Daten ändern.",
        ("de", "review") => "Prüfe die Details und bestätige, um fortzufahren.",
        ("de", "details") => "Details",
        ("de", "boot_no_reboot") => "Der Computer wird nicht automatisch neu gestartet.",
        ("de", "cleanup_warning") => "Der Cleaner prüft Caches, temporäre Dateien und optionale persönliche Pfade. Downloads, Dokumente, Anwendungen und Papierkorb werden nicht ohne ausdrückliche Zustimmung gelöscht; Elemente können einzeln geprüft werden.",
        ("fr", "warning_data") => "Cette action peut modifier ou détruire des données.",
        ("fr", "warning_system") => "Cette action peut modifier les paramètres système ou les données.",
        ("fr", "review") => "Vérifiez les détails et confirmez pour continuer.",
        ("fr", "details") => "Détails",
        ("fr", "boot_no_reboot") => "L’ordinateur ne redémarrera pas automatiquement.",
        ("fr", "cleanup_warning") => "Le nettoyeur examine les caches, fichiers temporaires et chemins personnels facultatifs. Les téléchargements, documents, applications et la corbeille ne seront pas supprimés sans accord explicite ; les éléments peuvent être vérifiés un par un.",
        ("pt", "warning_data") => "Esta ação pode alterar ou destruir dados.",
        ("pt", "warning_system") => "Esta ação pode alterar definições do sistema ou dados.",
        ("pt", "review") => "Revise os detalhes e confirme para continuar.",
        ("pt", "details") => "Detalhes",
        ("pt", "boot_no_reboot") => "O computador não será reiniciado automaticamente.",
        ("pt", "cleanup_warning") => "O limpador analisará caches, ficheiros temporários e caminhos pessoais opcionais. Transferências, documentos, aplicações e lixo não serão eliminados sem consentimento explícito; os itens podem ser revistos individualmente.",
        ("it", "warning_data") => "Questa azione può modificare o distruggere i dati.",
        ("it", "warning_system") => "Questa azione può modificare le impostazioni di sistema o i dati.",
        ("it", "review") => "Controlla i dettagli e conferma per continuare.",
        ("it", "details") => "Dettagli",
        ("it", "boot_no_reboot") => "Il computer non verrà riavviato automaticamente.",
        ("it", "cleanup_warning") => "Il pulitore analizzerà cache, file temporanei e percorsi personali facoltativi. Download, documenti, applicazioni e cestino non verranno eliminati senza consenso esplicito; gli elementi si possono esaminare singolarmente.",
        ("ca", "warning_data") => "Aquesta acció pot modificar o destruir dades.",
        ("ca", "warning_system") => "Aquesta acció pot modificar la configuració del sistema o les dades.",
        ("ca", "review") => "Revisa els detalls i confirma per continuar.",
        ("ca", "details") => "Detalls",
        ("ca", "boot_no_reboot") => "L’ordinador no es reiniciarà automàticament.",
        ("ca", "cleanup_warning") => "El netejador revisarà la memòria cau, els fitxers temporals i les rutes personals opcionals. Les baixades, els documents, les aplicacions i la paperera no s’eliminaran sense consentiment explícit; es poden revisar els elements un per un.",
        ("nl", "warning_data") => "Deze actie kan gegevens wijzigen of vernietigen.",
        ("nl", "warning_system") => "Deze actie kan systeeminstellingen of gegevens wijzigen.",
        ("nl", "review") => "Controleer de details en bevestig om door te gaan.",
        ("nl", "details") => "Details",
        ("nl", "boot_no_reboot") => "De computer wordt niet automatisch opnieuw opgestart.",
        ("nl", "cleanup_warning") => "De opschoner controleert caches, tijdelijke bestanden en optionele persoonlijke paden. Downloads, documenten, toepassingen en prullenbak worden niet zonder uitdrukkelijke toestemming verwijderd; items kunnen afzonderlijk worden beoordeeld.",
        ("pl", "warning_data") => "Ta czynność może zmienić lub zniszczyć dane.",
        ("pl", "warning_system") => "Ta czynność może zmienić ustawienia systemowe lub dane.",
        ("pl", "review") => "Sprawdź szczegóły i potwierdź, aby kontynuować.",
        ("pl", "details") => "Szczegóły",
        ("pl", "boot_no_reboot") => "Komputer nie zostanie automatycznie uruchomiony ponownie.",
        ("pl", "cleanup_warning") => "Oczyszczanie sprawdzi pamięci podręczne, pliki tymczasowe i opcjonalne ścieżki osobiste. Pobrane pliki, dokumenty, aplikacje i kosz nie zostaną usunięte bez wyraźnej zgody; elementy można sprawdzać pojedynczo.",
        ("ar", "warning_data") => "قد يؤدي هذا الإجراء إلى تعديل البيانات أو إتلافها.",
        ("ar", "warning_system") => "قد يؤدي هذا الإجراء إلى تغيير إعدادات النظام أو البيانات.",
        ("ar", "review") => "راجع التفاصيل وأكّد للمتابعة.",
        ("ar", "details") => "التفاصيل",
        ("ar", "boot_no_reboot") => "لن يُعاد تشغيل الكمبيوتر تلقائيًا.",
        ("ar", "cleanup_warning") => "سيفحص المنظّف ذاكرات التخزين المؤقت والملفات المؤقتة والمسارات الشخصية الاختيارية. لن تُحذف التنزيلات والمستندات والتطبيقات وسلة المهملات دون موافقة صريحة؛ ويمكن مراجعة العناصر واحدًا تلو الآخر.",
        ("hi", "warning_data") => "यह कार्रवाई डेटा बदल या नष्ट कर सकती है।",
        ("hi", "warning_system") => "यह कार्रवाई सिस्टम सेटिंग या डेटा बदल सकती है।",
        ("hi", "review") => "आगे बढ़ने से पहले विवरण जाँचें और पुष्टि करें।",
        ("hi", "details") => "विवरण",
        ("hi", "boot_no_reboot") => "कंप्यूटर अपने-आप रीस्टार्ट नहीं होगा।",
        ("hi", "cleanup_warning") => "क्लीनर कैश, अस्थायी फ़ाइलें और वैकल्पिक निजी पथ जाँचेगा। स्पष्ट सहमति के बिना डाउनलोड, दस्तावेज़, ऐप्लिकेशन और ट्रैश नहीं हटेंगे; आइटम एक-एक करके देखे जा सकते हैं।",
        ("ja", "warning_data") => "この操作によりデータが変更または破損する可能性があります。",
        ("ja", "warning_system") => "この操作によりシステム設定やデータが変更される可能性があります。",
        ("ja", "review") => "詳細を確認してから続行を確定してください。",
        ("ja", "details") => "詳細",
        ("ja", "boot_no_reboot") => "コンピューターは自動的に再起動しません。",
        ("ja", "cleanup_warning") => "クリーナーはキャッシュ、一時ファイル、任意の個人用パスを調べます。明示的な同意なしにダウンロード、ドキュメント、アプリ、ゴミ箱を削除することはありません。項目を個別に確認できます。",
        ("ko", "warning_data") => "이 작업은 데이터를 변경하거나 파괴할 수 있습니다.",
        ("ko", "warning_system") => "이 작업은 시스템 설정이나 데이터를 변경할 수 있습니다.",
        ("ko", "review") => "세부 정보를 확인한 뒤 계속하려면 확인하세요.",
        ("ko", "details") => "세부 정보",
        ("ko", "boot_no_reboot") => "컴퓨터가 자동으로 다시 시작되지는 않습니다.",
        ("ko", "cleanup_warning") => "정리 도구는 캐시, 임시 파일 및 선택적 개인 경로를 검사합니다. 명시적인 동의 없이는 다운로드, 문서, 앱, 휴지통을 삭제하지 않으며 항목별로 검토할 수 있습니다.",
        ("ro", "warning_data") => "Această acțiune poate modifica sau distruge date.",
        ("ro", "warning_system") => "Această acțiune poate modifica setările sistemului sau datele.",
        ("ro", "review") => "Verifică detaliile și confirmă pentru a continua.",
        ("ro", "details") => "Detalii",
        ("ro", "boot_no_reboot") => "Computerul nu va reporni automat.",
        ("ro", "cleanup_warning") => "Curățătorul va verifica memoria cache, fișierele temporare și căile personale opționale. Descărcările, documentele, aplicațiile și coșul de gunoi nu vor fi șterse fără acord explicit; elementele pot fi verificate individual.",
        ("ru", "warning_data") => "Это действие может изменить или уничтожить данные.",
        ("ru", "warning_system") => "Это действие может изменить системные настройки или данные.",
        ("ru", "review") => "Проверьте сведения и подтвердите продолжение.",
        ("ru", "details") => "Подробности",
        ("ru", "boot_no_reboot") => "Компьютер не будет перезагружен автоматически.",
        ("ru", "cleanup_warning") => "Очистка проверит кэши, временные файлы и необязательные личные пути. Загрузки, документы, приложения и корзина не будут удалены без явного согласия; элементы можно проверять по одному.",
        ("uk", "warning_data") => "Ця дія може змінити або знищити дані.",
        ("uk", "warning_system") => "Ця дія може змінити системні налаштування або дані.",
        ("uk", "review") => "Перевірте подробиці та підтвердьте продовження.",
        ("uk", "details") => "Подробиці",
        ("uk", "boot_no_reboot") => "Комп’ютер не перезавантажиться автоматично.",
        ("uk", "cleanup_warning") => "Засіб очищення перевірить кеші, тимчасові файли та необов’язкові особисті шляхи. Завантаження, документи, програми й кошик не буде видалено без явної згоди; елементи можна перевіряти окремо.",
        ("zh", "warning_data") => "此操作可能修改或销毁数据。",
        ("zh", "warning_system") => "此操作可能更改系统设置或数据。",
        ("zh", "review") => "请检查详细信息并确认后继续。",
        ("zh", "details") => "详细信息",
        ("zh", "boot_no_reboot") => "计算机不会自动重启。",
        ("zh", "cleanup_warning") => "清理器将检查缓存、临时文件和可选的个人路径。未经明确同意，不会删除下载、文档、应用或回收站内容；可逐项检查。",
        (_, "warning_data") => "Esta acción puede modificar o destruir datos.",
        (_, "warning_system") => "Esta acción puede cambiar la configuración del sistema o los datos.",
        (_, "review") => "Revisa los detalles y confirma para continuar.",
        (_, "details") => "Detalles",
        (_, "boot_no_reboot") => "El equipo no se reiniciará automáticamente.",
        (_, "cleanup_warning") => "El limpiador revisará cachés, temporales y rutas personales opcionales. Descargas, Documentos, aplicaciones y papelera no se borrarán sin consentimiento explícito; puedes revisar los elementos uno por uno.",
        _ => "",
    }
}

/// Textos del módulo de paquetes/Git. Las operaciones y sus argumentos son
/// estables para automatización; solo se traduce la interfaz visible.
#[cfg(any(unix, test))]
pub fn git_action_text(key: &str) -> &'static str {
    if key == "guide" {
        return match current() {
            "en" => "Git and GitHub guide (gh)",
            "de" => "Git- und GitHub-Handbuch (gh)",
            "fr" => "Guide Git et GitHub (gh)",
            "pt" => "Guia de Git e GitHub (gh)",
            "it" => "Guida Git e GitHub (gh)",
            "pl" => "Przewodnik Git i GitHub (gh)",
            "ar" => "دليل Git وGitHub (gh)",
            "hi" => "Git और GitHub मार्गदर्शिका (gh)",
            "ja" => "Git と GitHub のガイド (gh)",
            "ko" => "Git 및 GitHub 안내 (gh)",
            "ro" => "Ghid Git și GitHub (gh)",
            "ru" => "Руководство по Git и GitHub (gh)",
            "uk" => "Посібник із Git і GitHub (gh)",
            "zh" => "Git 与 GitHub 指南 (gh)",
            _ => "Guía completa de Git y GitHub (gh)",
        };
    }
    let text: [&'static str; 13] = match current() {
        "en" => [
            "GitHub CLI version",
            "GitHub CLI help",
            "Native gh command…",
            "Diagnose repository",
            "Rebuild .git index…",
            "Recover .git from remote…",
            "Local diagnosis and recovery",
            "Native GitHub CLI command (gh)",
            "gh command (issue, pr, run, workflow, api, extension…)",
            "gh arguments (quotes supported; no shell is run)",
            "HTTPS/SSH remote repository URL",
            "Remote branch (optional; use its default)",
            "Run",
        ],
        "de" => [
            "Version der GitHub CLI",
            "Hilfe der GitHub CLI",
            "Nativer gh-Befehl…",
            "Repository diagnostizieren",
            ".git-Index neu erstellen…",
            ".git aus Remote wiederherstellen…",
            "Lokale Diagnose und Wiederherstellung",
            "Nativer GitHub-CLI-Befehl (gh)",
            "gh-Befehl (issue, pr, run, workflow, api, extension…)",
            "gh-Argumente (Anführungszeichen; keine Shell-Ausführung)",
            "HTTPS/SSH-URL des Remote-Repositorys",
            "Remote-Branch (optional; Standard verwenden)",
            "Ausführen",
        ],
        "fr" => [
            "Version de GitHub CLI",
            "Aide de GitHub CLI",
            "Commande gh native…",
            "Diagnostiquer le dépôt",
            "Reconstruire l’index .git…",
            "Récupérer .git depuis un dépôt distant…",
            "Diagnostic et récupération locaux",
            "Commande native GitHub CLI (gh)",
            "Commande gh (issue, pr, run, workflow, api, extension…)",
            "Arguments gh (guillemets acceptés ; aucun shell exécuté)",
            "URL HTTPS/SSH du dépôt distant",
            "Branche distante (facultative ; défaut du dépôt)",
            "Exécuter",
        ],
        "pt" => [
            "Versão do GitHub CLI",
            "Ajuda do GitHub CLI",
            "Comando gh nativo…",
            "Diagnosticar repositório",
            "Reconstruir índice .git…",
            "Recuperar .git do remoto…",
            "Diagnóstico e recuperação local",
            "Comando nativo do GitHub CLI (gh)",
            "Comando gh (issue, pr, run, workflow, api, extension…)",
            "Argumentos gh (aceita aspas; não executa shell)",
            "URL HTTPS/SSH do repositório remoto",
            "Ramo remoto (opcional; usa o predefinido)",
            "Executar",
        ],
        "it" => [
            "Versione di GitHub CLI",
            "Guida di GitHub CLI",
            "Comando gh nativo…",
            "Diagnostica repository",
            "Ricostruisci indice .git…",
            "Recupera .git dal remoto…",
            "Diagnosi e recupero locali",
            "Comando nativo GitHub CLI (gh)",
            "Comando gh (issue, pr, run, workflow, api, extension…)",
            "Argomenti gh (virgolette supportate; nessuna shell)",
            "URL HTTPS/SSH del repository remoto",
            "Ramo remoto (facoltativo; usa quello predefinito)",
            "Esegui",
        ],
        "pl" => [
            "Wersja GitHub CLI",
            "Pomoc GitHub CLI",
            "Natywne polecenie gh…",
            "Diagnozuj repozytorium",
            "Odbuduj indeks .git…",
            "Odzyskaj .git ze zdalnego repozytorium…",
            "Lokalna diagnostyka i odzyskiwanie",
            "Natywne polecenie GitHub CLI (gh)",
            "Polecenie gh (issue, pr, run, workflow, api, extension…)",
            "Argumenty gh (obsługa cudzysłowów; bez powłoki)",
            "Adres HTTPS/SSH zdalnego repozytorium",
            "Zdalna gałąź (opcjonalnie; domyślna)",
            "Uruchom",
        ],
        "ar" => [
            "إصدار GitHub CLI",
            "مساعدة GitHub CLI",
            "أمر gh أصلي…",
            "تشخيص المستودع",
            "إعادة إنشاء فهرس .git…",
            "استعادة .git من مستودع بعيد…",
            "التشخيص والاستعادة محليًا",
            "أمر GitHub CLI أصلي (gh)",
            "أمر gh (issue وpr وrun وworkflow وapi وextension…)",
            "وسائط gh (تُقبل علامات الاقتباس؛ بلا تشغيل shell)",
            "عنوان HTTPS/SSH للمستودع البعيد",
            "فرع بعيد (اختياري؛ الافتراضي)",
            "تنفيذ",
        ],
        "hi" => [
            "GitHub CLI संस्करण",
            "GitHub CLI सहायता",
            "मूल gh कमांड…",
            "रिपॉज़िटरी जाँचें",
            ".git इंडेक्स फिर बनाएँ…",
            "रिमोट से .git पुनर्प्राप्त करें…",
            "स्थानीय जाँच और पुनर्प्राप्ति",
            "मूल GitHub CLI कमांड (gh)",
            "gh कमांड (issue, pr, run, workflow, api, extension…)",
            "gh तर्क (उद्धरण समर्थित; shell नहीं चलता)",
            "HTTPS/SSH रिमोट रिपॉज़िटरी URL",
            "रिमोट शाखा (वैकल्पिक; डिफ़ॉल्ट)",
            "चलाएँ",
        ],
        "ja" => [
            "GitHub CLI のバージョン",
            "GitHub CLI のヘルプ",
            "gh ネイティブコマンド…",
            "リポジトリを診断",
            ".git インデックスを再構築…",
            "リモートから .git を復元…",
            "ローカル診断と復元",
            "GitHub CLI ネイティブコマンド (gh)",
            "gh コマンド (issue、pr、run、workflow、api、extension…)",
            "gh 引数（引用符対応。シェルは実行しません）",
            "HTTPS/SSH リモートリポジトリ URL",
            "リモートブランチ（任意。既定を使用）",
            "実行",
        ],
        "ko" => [
            "GitHub CLI 버전",
            "GitHub CLI 도움말",
            "네이티브 gh 명령…",
            "저장소 진단",
            ".git 인덱스 재구성…",
            "원격에서 .git 복구…",
            "로컬 진단 및 복구",
            "네이티브 GitHub CLI 명령 (gh)",
            "gh 명령 (issue, pr, run, workflow, api, extension…)",
            "gh 인수 (따옴표 지원; 셸은 실행하지 않음)",
            "HTTPS/SSH 원격 저장소 URL",
            "원격 브랜치 (선택; 기본값 사용)",
            "실행",
        ],
        "ro" => [
            "Versiunea GitHub CLI",
            "Ajutor GitHub CLI",
            "Comandă gh nativă…",
            "Diagnostichează depozitul",
            "Recreează indexul .git…",
            "Recuperează .git din depozitul distant…",
            "Diagnostic și recuperare locală",
            "Comandă GitHub CLI nativă (gh)",
            "Comandă gh (issue, pr, run, workflow, api, extension…)",
            "Argumente gh (cu ghilimele; fără shell)",
            "URL HTTPS/SSH al depozitului distant",
            "Ramură distantă (opțional; cea implicită)",
            "Execută",
        ],
        "ru" => [
            "Версия GitHub CLI",
            "Справка GitHub CLI",
            "Нативная команда gh…",
            "Диагностика репозитория",
            "Пересоздать индекс .git…",
            "Восстановить .git из удалённого репозитория…",
            "Локальная диагностика и восстановление",
            "Нативная команда GitHub CLI (gh)",
            "Команда gh (issue, pr, run, workflow, api, extension…)",
            "Аргументы gh (кавычки поддерживаются; shell не запускается)",
            "HTTPS/SSH-адрес удалённого репозитория",
            "Удалённая ветка (необязательно; по умолчанию)",
            "Запустить",
        ],
        "uk" => [
            "Версія GitHub CLI",
            "Довідка GitHub CLI",
            "Власна команда gh…",
            "Діагностувати репозиторій",
            "Перебудувати індекс .git…",
            "Відновити .git із віддаленого репозиторію…",
            "Локальна діагностика й відновлення",
            "Власна команда GitHub CLI (gh)",
            "Команда gh (issue, pr, run, workflow, api, extension…)",
            "Аргументи gh (лапки підтримуються; shell не запускається)",
            "HTTPS/SSH-адреса віддаленого репозиторію",
            "Віддалена гілка (необов’язково; типова)",
            "Запустити",
        ],
        "zh" => [
            "GitHub CLI 版本",
            "GitHub CLI 帮助",
            "原生 gh 命令…",
            "诊断仓库",
            "重建 .git 索引…",
            "从远程恢复 .git…",
            "本地诊断与恢复",
            "原生 GitHub CLI 命令 (gh)",
            "gh 命令（issue、pr、run、workflow、api、extension…）",
            "gh 参数（支持引号；不会运行 shell）",
            "HTTPS/SSH 远程仓库 URL",
            "远程分支（可选；使用默认分支）",
            "运行",
        ],
        _ => [
            "Versión de GitHub CLI",
            "Ayuda nativa de GitHub CLI",
            "Comando nativo de gh…",
            "Diagnosticar repositorio",
            "Reconstruir índice .git…",
            "Recuperar .git desde remoto…",
            "Diagnóstico y recuperación local",
            "Comando nativo de GitHub CLI (gh)",
            "Comando gh (issue, pr, run, workflow, api, extension…)",
            "Argumentos de gh (admite comillas; no ejecuta shell)",
            "URL HTTPS/SSH del repositorio remoto",
            "Rama remota (opcional; usa la predeterminada)",
            "Ejecutar",
        ],
    };
    let index = match key {
        "version" => 0,
        "help" => 1,
        "native" => 2,
        "diagnose" => 3,
        "repair_index" => 4,
        "repair_remote" => 5,
        "recovery_heading" => 6,
        "native_title" => 7,
        "native_command" => 8,
        "native_arguments" => 9,
        "remote_url" => 10,
        "branch" => 11,
        "execute" => 12,
        _ => return "",
    };
    text[index]
}

pub fn tools_text(key: &str) -> &'static str {
    match (current(), key) {
        ("en", "menu") => "Packages, stores and Git",
        ("en", "help") => "Search/install from detected stores and perform guarded Git operations",
        ("en", "title") => "=== Packages, stores and Git ===",
        ("en", "software_menu") => "Software, packages and stores",
        ("en", "software_title") => "=== Software, packages and stores ===",
        ("en", "stores") => "Detected package stores",
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
        ("en", "git_lfs") => "Git LFS status and native command",
        ("de", "git_lfs") => "Git-LFS-Status und nativer Befehl",
        ("fr", "git_lfs") => "État Git LFS et commande native",
        ("pt", "git_lfs") => "Estado do Git LFS e comando nativo",
        ("it", "git_lfs") => "Stato Git LFS e comando nativo",
        ("ca", "git_lfs") => "Estat de Git LFS i ordre nativa",
        ("nl", "git_lfs") => "Git LFS-status en native opdracht",
        ("pl", "git_lfs") => "Stan Git LFS i polecenie natywne",
        ("ar", "git_lfs") => "حالة Git LFS والأمر الأصلي",
        ("hi", "git_lfs") => "Git LFS स्थिति और मूल कमांड",
        ("ja", "git_lfs") => "Git LFS の状態とネイティブコマンド",
        ("ko", "git_lfs") => "Git LFS 상태 및 네이티브 명령",
        ("ro", "git_lfs") => "Stare Git LFS și comandă nativă",
        ("ru", "git_lfs") => "Состояние Git LFS и нативная команда",
        ("uk", "git_lfs") => "Стан Git LFS і нативна команда",
        ("pl", "menu") => "Pakiety, źródła i Git",
        ("pl", "help") => "Szukaj/instaluj z wykrytych źródeł i wykonuj chronione operacje Git",
        ("pl", "title") => "=== Pakiety, źródła i Git ===",
        ("pl", "search") => "Szukaj pakietu w dostępnych źródłach",
        ("pl", "install") => "Wybierz i zainstaluj pakiet",
        (_, "menu") => "Paquetes, almacenes y Git",
        (_, "help") => "Buscar/instalar en stores detectadas y ejecutar operaciones Git protegidas",
        (_, "title") => "=== Paquetes, almacenes y Git ===",
        (_, "software_menu") => "Software, paquetes y almacenes",
        (_, "software_title") => "=== Software, paquetes y almacenes ===",
        (_, "stores") => "Almacenes de paquetes detectados",
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
        (_, "git_lfs") => "Estado de Git LFS y comando nativo",
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

/// Textos breves de la ayuda global que no pertenecen a una familia concreta.
/// Mantenerlos aquí evita que `--lang` deje fragmentos de español entre
/// descripciones traducidas. Los nombres de comandos y opciones se conservan
/// literalmente porque forman parte de la interfaz CLI.
pub fn help_extra(key: &str) -> &'static str {
    const TEXT: &[(&str, [&str; 15])] = &[
        (
            "report",
            [
                "Lire un rapport avec sortie directe, pagination ou éditeur",
                "Bericht direkt, mit Pager oder Editor lesen",
                "Read a report directly, through a pager or in an editor",
                "Leer un informe con salida directa, paginador o editor",
                "Lire un rapport directement, avec un pager ou un éditeur",
                "रिपोर्ट को सीधे, पेजर या संपादक के माध्यम से पढ़ें",
                "Leggi un rapporto direttamente, con pager o editor",
                "レポートを直接、ページャー、またはエディターで読む",
                "보고서를 직접, 페이저 또는 편집기로 읽기",
                "Czytaj raport bezpośrednio, przez pager lub w edytorze",
                "Ler um relatório diretamente, através de um pager ou editor",
                "Citiți un raport direct, prin pager sau editor",
                "Читать отчет напрямую, через pager или редактор",
                "Читати звіт напряму, через pager або редактор",
                "直接、通过分页器或编辑器读取报告",
            ],
        ),
        (
            "privileges",
            [
                "Politique d’élévation par action et plateforme",
                "Erhöhungsrichtlinie nach Aktion und Plattform",
                "Elevation policy by action and platform",
                "Política de elevación por acción y plataforma",
                "Politique d’élévation selon l’action et la plateforme",
                "कार्रवाई और प्लेटफ़ॉर्म के अनुसार उन्नयन नीति",
                "Politica di elevazione per azione e piattaforma",
                "操作とプラットフォームごとの昇格ポリシー",
                "작업 및 플랫폼별 권한 상승 정책",
                "Zasady podnoszenia uprawnień według działania i platformy",
                "Política de elevação por ação e plataforma",
                "Politica de elevare în funcție de acțiune și platformă",
                "Политика повышения прав по действию и платформе",
                "Політика підвищення прав за дією та платформою",
                "按操作和平台划分的提权策略",
            ],
        ),
        (
            "privilege_policy",
            [
                "Les actions obligatoires demandent sudo/UAC ; les consultations, Git/Wine et la corbeille utilisateur ne sont pas élevées.",
                "Erforderliche Aktionen fragen sudo/UAC an; Abfragen, Git/Wine und der Papierkorb des Benutzers werden nicht erhöht.",
                "Required actions ask for sudo/UAC; queries, Git/Wine and the user’s trash are never elevated.",
                "Las acciones obligatorias piden sudo/UAC; consultas, Git/Wine y la papelera del usuario no se elevan.",
                "Les actions obligatoires demandent sudo/UAC ; les consultations, Git/Wine et la corbeille utilisateur ne sont pas élevées.",
                "आवश्यक कार्रवाइयाँ sudo/UAC माँगती हैं; क्वेरी, Git/Wine और उपयोगकर्ता का ट्रैश उन्नत नहीं किया जाता।",
                "Le azioni obbligatorie richiedono sudo/UAC; consultazioni, Git/Wine e cestino dell’utente non vengono elevate.",
                "必須操作では sudo/UAC を要求します。照会、Git/Wine、ユーザーのゴミ箱は昇格しません。",
                "필수 작업은 sudo/UAC를 요청하며 조회, Git/Wine 및 사용자 휴지통은 권한 상승하지 않습니다.",
                "Wymagane działania proszą o sudo/UAC; zapytania, Git/Wine i kosz użytkownika nie są uruchamiane z podwyższonymi uprawnieniami.",
                "As ações obrigatórias pedem sudo/UAC; consultas, Git/Wine e o lixo do utilizador não são elevados.",
                "Acțiunile obligatorii cer sudo/UAC; interogările, Git/Wine și coșul utilizatorului nu sunt elevate.",
                "Обязательные действия запрашивают sudo/UAC; запросы, Git/Wine и корзина пользователя не запускаются с повышенными правами.",
                "Обов’язкові дії запитують sudo/UAC; запити, Git/Wine і кошик користувача не виконуються з підвищенням прав.",
                "必需操作会请求 sudo/UAC；查询、Git/Wine 和用户回收站不会提权。",
            ],
        ),
        (
            "doctor_install",
            [
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
                "doctor --install TOOL",
            ],
        ),
        (
            "elevation_hint",
            [
                "--elevate | --no-elevate ; la préférence persistante se change dans Réglages.",
                "--elevate | --no-elevate; die dauerhafte Einstellung wird unter Einstellungen geändert.",
                "--elevate | --no-elevate; change the persistent preference in Settings.",
                "--elevate | --no-elevate; la preferencia persistente se cambia en Ajustes.",
                "--elevate | --no-elevate ; la préférence persistante se modifie dans Paramètres.",
                "--elevate | --no-elevate; स्थायी प्राथमिकता सेटिंग्स में बदलें।",
                "--elevate | --no-elevate; cambia la preferenza persistente in Impostazioni.",
                "--elevate | --no-elevate。永続的な設定は設定画面で変更します。",
                "--elevate | --no-elevate; 영구 기본 설정은 설정에서 변경합니다.",
                "--elevate | --no-elevate; trwałe ustawienie zmienisz w Ustawieniach.",
                "--elevate | --no-elevate; altere a preferência persistente em Definições.",
                "--elevate | --no-elevate; modificați preferința persistentă în Setări.",
                "--elevate | --no-elevate; изменяйте постоянную настройку в разделе «Настройки».",
                "--elevate | --no-elevate; змінюйте постійне налаштування в розділі «Параметри».",
                "--elevate | --no-elevate；在设置中更改持久偏好。",
            ],
        ),
        (
            "git",
            [
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [options sûres]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [sichere Optionen]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [safe options]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [opciones seguras]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [options sûres]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [सुरक्षित विकल्प]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [opzioni sicure]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [安全なオプション]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [안전한 옵션]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [bezpieczne opcje]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [opções seguras]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [opțiuni sigure]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [безопасные параметры]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [безпечні параметри]",
                "git status|log|clone|fetch|pull|add|commit|push|branch|tag|release|diagnose|repair|lfs|gh|login [安全选项]",
            ],
        ),
        (
            "winslim",
            [
                "Détecter NSudo et lancer des processus Windows avec une identité explicite",
                "NSudo erkennen und Windows-Prozesse mit einer expliziten Identität starten",
                "Detect NSudo and launch Windows processes with an explicit identity",
                "Detectar NSudo y lanzar procesos Windows con identidad explícita",
                "Détecter NSudo et lancer des processus Windows avec une identité explicite",
                "NSudo का पता लगाएँ और स्पष्ट पहचान के साथ Windows प्रक्रियाएँ चलाएँ",
                "Rileva NSudo e avvia processi Windows con un’identità esplicita",
                "NSudo を検出し、明示した ID で Windows プロセスを起動",
                "NSudo를 감지하고 명시적 ID로 Windows 프로세스 실행",
                "Wykrywaj NSudo i uruchamiaj procesy Windows z jawną tożsamością",
                "Detetar o NSudo e iniciar processos Windows com uma identidade explícita",
                "Detectați NSudo și lansați procese Windows cu o identitate explicită",
                "Обнаруживать NSudo и запускать процессы Windows с указанным идентификатором",
                "Виявляти NSudo й запускати процеси Windows із явно вказаною особою",
                "检测 NSudo 并使用明确身份启动 Windows 进程",
            ],
        ),
        (
            "release_manifest",
            [
                "Générer le manifeste vérifiable d’une release GitHub",
                "Überprüfbares GitHub-Release-Manifest erzeugen",
                "Generate a verifiable GitHub release manifest",
                "Generar el manifiesto verificable de una release de GitHub",
                "Générer le manifeste vérifiable d’une release GitHub",
                "सत्यापनीय GitHub रिलीज़ मेनिफेस्ट बनाएँ",
                "Genera il manifesto verificabile di una release GitHub",
                "検証可能な GitHub リリースマニフェストを生成",
                "검증 가능한 GitHub 릴리스 매니페스트 생성",
                "Generuj weryfikowalny manifest wydania GitHub",
                "Gerar o manifesto verificável de uma release GitHub",
                "Generați manifestul verificabil al unei versiuni GitHub",
                "Создать проверяемый манифест релиза GitHub",
                "Створити перевірюваний маніфест релізу GitHub",
                "生成可验证的 GitHub 发布清单",
            ],
        ),
        (
            "release_checksums",
            [
                "Générer SHA256SUMS.txt pour les artefacts publiables",
                "SHA256SUMS.txt für veröffentlichbare Artefakte erzeugen",
                "Generate SHA256SUMS.txt for publishable artifacts",
                "Generar SHA256SUMS.txt para los artefactos publicables",
                "Générer SHA256SUMS.txt pour les artefacts publiables",
                "प्रकाशित किए जा सकने वाले आर्टिफ़ैक्ट के लिए SHA256SUMS.txt बनाएँ",
                "Genera SHA256SUMS.txt per gli artefatti pubblicabili",
                "公開可能な成果物用の SHA256SUMS.txt を生成",
                "게시 가능한 아티팩트용 SHA256SUMS.txt 생성",
                "Generuj SHA256SUMS.txt dla artefaktów do publikacji",
                "Gerar SHA256SUMS.txt para os artefactos publicáveis",
                "Generați SHA256SUMS.txt pentru artefactele publicabile",
                "Создать SHA256SUMS.txt для публикуемых артефактов",
                "Створити SHA256SUMS.txt для артефактів, готових до публікації",
                "为可发布构件生成 SHA256SUMS.txt",
            ],
        ),
        (
            "release_signature",
            [
                "Signer ou vérifier SHA256SUMS.txt avec Ed25519",
                "SHA256SUMS.txt mit Ed25519 signieren oder prüfen",
                "Sign or verify SHA256SUMS.txt with Ed25519",
                "Firmar o verificar SHA256SUMS.txt con Ed25519",
                "Signer ou vérifier SHA256SUMS.txt avec Ed25519",
                "Ed25519 से SHA256SUMS.txt पर हस्ताक्षर या सत्यापन करें",
                "Firma o verifica SHA256SUMS.txt con Ed25519",
                "Ed25519 で SHA256SUMS.txt に署名または検証",
                "Ed25519로 SHA256SUMS.txt 서명 또는 검증",
                "Podpisuj lub weryfikuj SHA256SUMS.txt za pomocą Ed25519",
                "Assinar ou verificar SHA256SUMS.txt com Ed25519",
                "Semnați sau verificați SHA256SUMS.txt cu Ed25519",
                "Подписать или проверить SHA256SUMS.txt с помощью Ed25519",
                "Підписати або перевірити SHA256SUMS.txt за допомогою Ed25519",
                "使用 Ed25519 签名或验证 SHA256SUMS.txt",
            ],
        ),
        (
            "update",
            [
                "update [check|download] [--repository OWNER/REPO] [--pause] — vérifie une release GitHub, la signature et le hash, puis prépare le paquet sans privilèges",
                "update [check|download] [--repository OWNER/REPO] [--pause] — prüft ein GitHub-Release, Signatur und Hash und bereitet das Paket ohne erhöhte Rechte vor",
                "update [check|download] [--repository OWNER/REPO] [--pause] — checks a GitHub release, signature and hash, then prepares the package without privileges",
                "update [check|download] [--repository OWNER/REPO] [--pause] — comprueba una release GitHub, verifica firma/hash y prepara el paquete sin privilegios",
                "update [check|download] [--repository OWNER/REPO] [--pause] — vérifie une release GitHub, la signature et le hash, puis prépare le paquet sans privilèges",
                "update [check|download] [--repository OWNER/REPO] [--pause] — GitHub रिलीज़, हस्ताक्षर और हैश जाँचकर बिना उन्नत अधिकार के पैकेज तैयार करता है",
                "update [check|download] [--repository OWNER/REPO] [--pause] — verifica una release GitHub, firma e hash e prepara il pacchetto senza privilegi",
                "update [check|download] [--repository OWNER/REPO] [--pause] — GitHub リリース、署名、ハッシュを確認し、権限なしでパッケージを準備",
                "update [check|download] [--repository OWNER/REPO] [--pause] — GitHub 릴리스, 서명 및 해시를 확인하고 권한 없이 패키지 준비",
                "update [check|download] [--repository OWNER/REPO] [--pause] — sprawdza wydanie GitHub, podpis i hash, a następnie przygotowuje pakiet bez podwyższonych uprawnień",
                "update [check|download] [--repository OWNER/REPO] [--pause] — verifica uma release GitHub, assinatura e hash e prepara o pacote sem privilégios",
                "update [check|download] [--repository OWNER/REPO] [--pause] — verifică o versiune GitHub, semnătura și hashul, apoi pregătește pachetul fără privilegii",
                "update [check|download] [--repository OWNER/REPO] [--pause] — проверяет релиз GitHub, подпись и хеш, затем готовит пакет без повышения прав",
                "update [check|download] [--repository OWNER/REPO] [--pause] — перевіряє реліз GitHub, підпис і хеш, а потім готує пакет без підвищення прав",
                "update [check|download] [--repository OWNER/REPO] [--pause] — 检查 GitHub 发布、签名和哈希，然后在不提权的情况下准备包",
            ],
        ),
        (
            "scanner_note",
            [
                "Consultation en lecture seule : aucun analyseur n’est exécuté et le dépôt n’est pas modifié.",
                "Nur-Lese-Abfrage: Es wird kein Scanner ausgeführt und das Repository nicht verändert.",
                "Read-only inventory: no scanner is run and the repository is not modified.",
                "Consulta informativa: no se ejecuta ningún análisis ni se modifica el repositorio.",
                "Inventaire en lecture seule : aucun analyseur n’est exécuté et le dépôt n’est pas modifié.",
                "केवल-पठन सूची: कोई स्कैनर नहीं चलाया जाता और रिपॉज़िटरी बदली नहीं जाती।",
                "Inventario in sola lettura: nessuno scanner viene eseguito e il repository non viene modificato.",
                "読み取り専用の一覧です。スキャナーは実行せず、リポジトリも変更しません。",
                "읽기 전용 목록입니다. 스캐너를 실행하지 않으며 저장소를 수정하지 않습니다.",
                "Inwentaryzacja tylko do odczytu: żaden skaner nie jest uruchamiany, a repozytorium nie jest modyfikowane.",
                "Inventário somente leitura: nenhum scanner é executado e o repositório não é alterado.",
                "Inventar doar în citire: nu rulează niciun scaner și depozitul nu este modificat.",
                "Инвентаризация только для чтения: сканеры не запускаются, репозиторий не изменяется.",
                "Інвентаризація лише для читання: сканери не запускаються, репозиторій не змінюється.",
                "只读清单：不会运行扫描器，也不会修改仓库。",
            ],
        ),
    ];
    let index = SUPPORTED
        .iter()
        .position(|candidate| *candidate == current())
        .unwrap_or(3);
    TEXT.iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, values)| values[index])
        .unwrap_or("")
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

/// Etiquetas de acciones que conectan una categoría con su guía contextual.
/// El catálogo de la guía y el constructor de la GUI comparten estas mismas
/// cadenas para que el índice no se quede desfasado al cambiar de idioma.
#[allow(dead_code)]
pub fn gui_catalog_text(key: &str) -> &'static str {
    const TEXT: &[(&str, [&str; 15])] = &[
        (
            "native_guide",
            [
                "دليل الأدوات الأصلية",
                "Handbuch für native Werkzeuge",
                "Native tools guide",
                "Guía de herramientas nativas",
                "Guide des outils natifs",
                "मूल उपकरण मार्गदर्शिका",
                "Guida agli strumenti nativi",
                "ネイティブツールガイド",
                "네이티브 도구 안내",
                "Przewodnik po narzędziach natywnych",
                "Guia de ferramentas nativas",
                "Ghid pentru instrumente native",
                "Руководство по системным инструментам",
                "Посібник із вбудованих інструментів",
                "原生工具指南",
            ],
        ),
        (
            "dependencies_guide",
            [
                "دليل التبعيات",
                "Handbuch für Abhängigkeiten",
                "Dependencies guide",
                "Guía de dependencias",
                "Guide des dépendances",
                "निर्भरताएँ मार्गदर्शिका",
                "Guida alle dipendenze",
                "依存関係ガイド",
                "종속성 안내",
                "Przewodnik po zależnościach",
                "Guia de dependências",
                "Ghid pentru dependențe",
                "Руководство по зависимостям",
                "Посібник із залежностей",
                "依赖项指南",
            ],
        ),
        (
            "installable_guide",
            [
                "دليل الأدوات القابلة للتثبيت",
                "Handbuch für installierbare Werkzeuge",
                "Installable tools guide",
                "Guía de herramientas instalables",
                "Guide des outils installables",
                "इंस्टॉल करने योग्य उपकरण मार्गदर्शिका",
                "Guida agli strumenti installabili",
                "インストール可能なツールガイド",
                "설치 가능한 도구 안내",
                "Przewodnik po narzędziach instalowalnych",
                "Guia de ferramentas instaláveis",
                "Ghid pentru instrumente instalabile",
                "Руководство по устанавливаемым инструментам",
                "Посібник із інструментів для встановлення",
                "可安装工具指南",
            ],
        ),
        (
            "automation_guide",
            [
                "دليل الأتمتة",
                "Handbuch für Automatisierung",
                "Automation guide",
                "Guía de automatización",
                "Guide de l’automatisation",
                "स्वचालन मार्गदर्शिका",
                "Guida all’automazione",
                "自動化ガイド",
                "자동화 안내",
                "Przewodnik po automatyzacji",
                "Guia de automação",
                "Ghid pentru automatizare",
                "Руководство по автоматизации",
                "Посібник з автоматизації",
                "自动化指南",
            ],
        ),
        (
            "registered_scripts",
            [
                "البرامج النصية المسجلة",
                "Registrierte Skripte",
                "Registered scripts",
                "Scripts registrados",
                "Scripts enregistrés",
                "पंजीकृत स्क्रिप्ट",
                "Script registrati",
                "登録済みスクリプト",
                "등록된 스크립트",
                "Zarejestrowane skrypty",
                "Scripts registados",
                "Scripturi înregistrate",
                "Зарегистрированные скрипты",
                "Зареєстровані скрипти",
                "已注册脚本",
            ],
        ),
        (
            "register_script",
            [
                "تسجيل برنامج نصي جديد",
                "Neues Skript registrieren",
                "Register new script",
                "Registrar nuevo script",
                "Enregistrer un nouveau script",
                "नई स्क्रिप्ट पंजीकृत करें",
                "Registra nuovo script",
                "新しいスクリプトを登録",
                "새 스크립트 등록",
                "Zarejestruj nowy skrypt",
                "Registar novo script",
                "Înregistrează un script nou",
                "Зарегистрировать новый скрипт",
                "Зареєструвати новий скрипт",
                "注册新脚本",
            ],
        ),
        (
            "scripts_guide",
            [
                "دليل البرامج النصية والأتمتة",
                "Handbuch für Skripte und Automatisierung",
                "Scripts and automation guide",
                "Guía de scripts y automatización",
                "Guide des scripts et de l’automatisation",
                "स्क्रिप्ट और स्वचालन मार्गदर्शिका",
                "Guida a script e automazione",
                "スクリプトと自動化ガイド",
                "스크립트 및 자동화 안내",
                "Przewodnik po skryptach i automatyzacji",
                "Guia de scripts e automação",
                "Ghid pentru scripturi și automatizare",
                "Руководство по скриптам и автоматизации",
                "Посібник зі скриптів та автоматизації",
                "脚本和自动化指南",
            ],
        ),
    ];
    let index = SUPPORTED
        .iter()
        .position(|language| *language == current())
        .unwrap_or(3);
    TEXT.iter()
        .find(|(entry_key, _)| *entry_key == key)
        .map(|(_, values)| values[index])
        .unwrap_or("")
}

/// Etiquetas de los ajustes de la CLI. Se mantienen fuera de `gui_text` para
/// que el binario de consola conserve portabilidad incluso en plataformas
/// donde no se compila una GUI nativa.
pub fn settings_text(key: &str) -> &'static str {
    if matches!(key, "update_check" | "update_download") {
        return update_text(key);
    }
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

/// Etiquetas de actualización, compartidas por la GUI y la navegación CLI.
pub fn update_text(key: &str) -> &'static str {
    match (current(), key) {
        ("ar", "update_check") => "التحقق من وجود تحديثات",
        ("ar", "update_download") => "تنزيل التحديث بعد التحقق",
        ("de", "update_check") => "Nach Updates suchen",
        ("de", "update_download") => "Verifiziertes Update herunterladen",
        ("en", "update_check") => "Check for updates",
        ("en", "update_download") => "Download verified update",
        ("es", "update_check") => "Comprobar actualizaciones",
        ("es", "update_download") => "Descargar actualización verificada",
        ("fr", "update_check") => "Rechercher des mises à jour",
        ("fr", "update_download") => "Télécharger la mise à jour vérifiée",
        ("hi", "update_check") => "अपडेट जाँचें",
        ("hi", "update_download") => "सत्यापित अपडेट डाउनलोड करें",
        ("it", "update_check") => "Controlla aggiornamenti",
        ("it", "update_download") => "Scarica l’aggiornamento verificato",
        ("ja", "update_check") => "更新を確認",
        ("ja", "update_download") => "検証済み更新をダウンロード",
        ("ko", "update_check") => "업데이트 확인",
        ("ko", "update_download") => "검증된 업데이트 다운로드",
        ("nl", "update_check") => "Controleren op updates",
        ("nl", "update_download") => "Geverifieerde update downloaden",
        ("pl", "update_check") => "Sprawdź aktualizacje",
        ("pl", "update_download") => "Pobierz zweryfikowaną aktualizację",
        ("pt", "update_check") => "Procurar atualizações",
        ("pt", "update_download") => "Transferir atualização verificada",
        ("ro", "update_check") => "Verifică actualizările",
        ("ro", "update_download") => "Descarcă actualizarea verificată",
        ("ru", "update_check") => "Проверить обновления",
        ("ru", "update_download") => "Скачать проверенное обновление",
        ("uk", "update_check") => "Перевірити оновлення",
        ("uk", "update_download") => "Завантажити перевірене оновлення",
        ("zh", "update_check") => "检查更新",
        ("zh", "update_download") => "下载已验证的更新",
        (_, "update_check") => "Comprobar actualizaciones",
        (_, "update_download") => "Descargar actualización verificada",
        (_, _) => "",
    }
}

/// Textos del registro de automatizaciones. Las automatizaciones son datos
/// del usuario, no plugins con código embebido; este catálogo solo traduce la
/// navegación y sus mensajes básicos.
fn automation_extended_text(key: &str) -> Option<&'static str> {
    const TEXT: &[(&str, [&str; 15])] = &[
        (
            "field_name",
            [
                "الاسم", "Name", "Name", "Nombre", "Nom", "नाम", "Nome", "名前",
                "이름", "Nazwa", "Nome", "Nume", "Имя", "Назва", "名称",
            ],
        ),
        (
            "field_program",
            [
                "البرنامج أو المسار",
                "Programm oder Pfad",
                "Program or path",
                "Programa o ruta",
                "Programme ou chemin",
                "कार्यक्रम या पथ",
                "Programma o percorso",
                "プログラムまたはパス",
                "프로그램 또는 경로",
                "Program lub ścieżka",
                "Programa ou caminho",
                "Program sau cale",
                "Программа или путь",
                "Програма або шлях",
                "程序或路径",
            ],
        ),
        (
            "field_working_directory",
            [
                "دليل العمل (فارغ للإبقاء؛ - للحذف)",
                "Arbeitsverzeichnis (leer beibehalten; - löschen)",
                "Working directory (blank keeps; - removes)",
                "Directorio de trabajo (vacío conserva; - elimina)",
                "Répertoire de travail (vide conserve ; - supprime)",
                "कार्य निर्देशिका (खाली रखने के लिए; - हटाने के लिए)",
                "Directory di lavoro (vuota mantiene; - rimuove)",
                "作業ディレクトリ（空欄は保持、- は削除）",
                "작업 디렉터리 (비우면 유지, - 는 삭제)",
                "Katalog roboczy (puste zachowuje; - usuwa)",
                "Diretório de trabalho (vazio mantém; - remove)",
                "Director de lucru (gol păstrează; - elimină)",
                "Рабочий каталог (пусто — сохранить; - — удалить)",
                "Робочий каталог (порожньо — зберегти; - — вилучити)",
                "工作目录（留空保留；- 删除）",
            ],
        ),
        (
            "field_arguments",
            [
                "المعاملات (فارغ للإبقاء؛ - للحذف)",
                "Argumente (leer beibehalten; - löschen)",
                "Arguments (blank keeps; - removes)",
                "Argumentos (vacío conserva; - elimina)",
                "Arguments (vide conserve ; - supprime)",
                "तर्क (खाली रखने के लिए; - हटाने के लिए)",
                "Argomenti (vuoto mantiene; - rimuove)",
                "引数（空欄は保持、- は削除）",
                "인수 (비우면 유지, - 는 삭제)",
                "Argumenty (puste zachowuje; - usuwa)",
                "Argumentos (vazio mantém; - remove)",
                "Argumente (gol păstrează; - elimină)",
                "Аргументы (пусто — сохранить; - — удалить)",
                "Аргументи (порожньо — зберегти; - — вилучити)",
                "参数（留空保留；- 删除）",
            ],
        ),
        (
            "save",
            [
                "حفظ التغييرات", "Änderungen speichern", "Save changes", "Guardar cambios",
                "Enregistrer les modifications", "परिवर्तन सहेजें", "Salva modifiche", "変更を保存",
                "변경 사항 저장", "Zapisz zmiany", "Guardar alterações", "Salvează modificările",
                "Сохранить изменения", "Зберегти зміни", "保存更改",
            ],
        ),
        (
            "refresh",
            [
                "إعادة تحميل القائمة", "Liste neu laden", "Refresh list", "Recargar listado",
                "Actualiser la liste", "सूची रीफ़्रेश करें", "Ricarica elenco", "一覧を再読み込み",
                "목록 새로 고침", "Odśwież listę", "Recarregar lista", "Reîncarcă lista",
                "Обновить список", "Оновити список", "刷新列表",
            ],
        ),
        (
            "none",
            [
                "لا توجد نصوص برمجية مسجلة.", "Keine Skripte registriert.", "No scripts registered.",
                "No hay scripts registrados.", "Aucun script enregistré.", "कोई स्क्रिप्ट पंजीकृत नहीं है।",
                "Nessuno script registrato.", "登録されたスクリプトはありません。", "등록된 스크립트가 없습니다.",
                "Nie zarejestrowano skryptów.", "Nenhum script registado.", "Nu există scripturi înregistrate.",
                "Зарегистрированных скриптов нет.", "Зареєстрованих скриптів немає.", "没有已注册的脚本。",
            ],
        ),
        (
            "action_run",
            [
                "تشغيل", "Ausführen", "Run", "Ejecutar", "Exécuter", "चलाएँ", "Esegui", "実行",
                "실행", "Uruchom", "Executar", "Rulează", "Запустить", "Запустити", "运行",
            ],
        ),
        (
            "action_edit",
            [
                "تحرير", "Bearbeiten", "Edit", "Editar", "Modifier", "संपादित करें", "Modifica", "編集",
                "편집", "Edytuj", "Editar", "Editează", "Изменить", "Редагувати", "编辑",
            ],
        ),
        (
            "action_remove",
            [
                "إزالة", "Entfernen", "Remove", "Borrar", "Supprimer", "हटाएँ", "Rimuovi", "削除",
                "제거", "Usuń", "Remover", "Elimină", "Удалить", "Вилучити", "移除",
            ],
        ),
        (
            "winslim_opened",
            [
                "تم فتح مساعد WinSlim / NSudo في وحدة تحكم مستقلة.",
                "Der WinSlim-/NSudo-Assistent wurde in einer separaten Konsole geöffnet.",
                "The WinSlim / NSudo assistant opened in a separate console.",
                "Se abrió el asistente WinSlim / NSudo en una consola independiente.",
                "L’assistant WinSlim / NSudo a été ouvert dans une console séparée.",
                "WinSlim / NSudo सहायक अलग कंसोल में खुल गया।",
                "L’assistente WinSlim / NSudo è stato aperto in una console separata.",
                "WinSlim / NSudo アシスタントを別のコンソールで開きました。",
                "WinSlim / NSudo 도우미가 별도 콘솔에서 열렸습니다.",
                "Asystent WinSlim / NSudo został otwarty w osobnej konsoli.",
                "O assistente WinSlim / NSudo foi aberto numa consola separada.",
                "Asistentul WinSlim / NSudo a fost deschis într-o consolă separată.",
                "Мастер WinSlim / NSudo открыт в отдельной консоли.",
                "Помічник WinSlim / NSudo відкрито в окремій консолі.",
                "WinSlim / NSudo 助手已在独立控制台中打开。",
            ],
        ),
        (
            "update_opened",
            [
                "تم فتح وحدة تحكم مستقلة للتحقق من تحديث أو تنزيله. لا تتم ترقية العملية ولا استبدال التثبيت تلقائيًا.",
                "Eine separate Konsole zur Prüfung oder zum Download eines Updates wurde geöffnet. Der Prozess wird nicht erhöht und die Installation nicht automatisch ersetzt.",
                "A separate console opened to check for or download an update. The process is not elevated and the installation is not replaced automatically.",
                "Se abrió una consola independiente para comprobar o descargar una actualización. No se eleva el proceso ni se sustituye automáticamente la instalación.",
                "Une console séparée a été ouverte pour rechercher ou télécharger une mise à jour. Le processus n’est pas élevé et l’installation n’est pas remplacée automatiquement.",
                "अपडेट जाँचने या डाउनलोड करने के लिए अलग कंसोल खुल गया। प्रक्रिया को उन्नत नहीं किया जाता और इंस्टॉलेशन अपने-आप बदला नहीं जाता।",
                "È stata aperta una console separata per verificare o scaricare un aggiornamento. Il processo non viene elevato e l’installazione non viene sostituita automaticamente.",
                "更新の確認またはダウンロード用に別のコンソールを開きました。プロセスは昇格されず、インストールも自動置換されません。",
                "업데이트 확인 또는 다운로드를 위해 별도 콘솔이 열렸습니다. 프로세스는 승격되지 않으며 설치도 자동으로 교체되지 않습니다.",
                "Otwarto osobną konsolę, aby sprawdzić lub pobrać aktualizację. Proces nie jest podnoszony, a instalacja nie jest automatycznie zastępowana.",
                "Foi aberta uma consola separada para verificar ou descarregar uma atualização. O processo não é elevado e a instalação não é substituída automaticamente.",
                "A fost deschisă o consolă separată pentru verificarea sau descărcarea unei actualizări. Procesul nu este elevat, iar instalarea nu este înlocuită automat.",
                "Открыта отдельная консоль для проверки или загрузки обновления. Процесс не получает повышенных прав, а установка не заменяется автоматически.",
                "Відкрито окрему консоль для перевірки або завантаження оновлення. Процес не отримує підвищених прав, а встановлення не замінюється автоматично.",
                "已打开独立控制台以检查或下载更新。进程不会提升权限，安装不会自动替换。",
            ],
        ),
    ];
    let index = SUPPORTED
        .iter()
        .position(|language| *language == current())
        .unwrap_or(3);
    TEXT.iter()
        .find(|(entry_key, _)| *entry_key == key)
        .map(|(_, values)| values[index])
}

pub fn automation_text(key: &str) -> &'static str {
    if let Some(text) = automation_extended_text(key) {
        return text;
    }
    match (current(), key) {
        ("ar", "winslim_status") => "حالة WSCore وNSudo",
        ("ar", "winslim_launch") => "فتح مساعد تشغيل NSudo…",
        ("ar", "winslim_guide") => "دليل استخدام NSudo وأمانه",
        ("de", "winslim_status") => "WSCore- und NSudo-Status",
        ("de", "winslim_launch") => "NSudo-Startassistent öffnen…",
        ("de", "winslim_guide") => "NSudo: Nutzung und Sicherheit",
        ("en", "winslim_status") => "WSCore and NSudo status",
        ("en", "winslim_launch") => "Open the NSudo launch assistant…",
        ("en", "winslim_guide") => "NSudo usage and safety guide",
        ("es", "winslim_status") => "Estado de WSCore y NSudo",
        ("es", "winslim_launch") => "Abrir el asistente de lanzamiento NSudo…",
        ("es", "winslim_guide") => "Guía de uso y seguridad NSudo",
        ("fr", "winslim_status") => "État de WSCore et NSudo",
        ("fr", "winslim_launch") => "Ouvrir l’assistant de lancement NSudo…",
        ("fr", "winslim_guide") => "Guide d’utilisation et de sécurité NSudo",
        ("hi", "winslim_status") => "WSCore और NSudo की स्थिति",
        ("hi", "winslim_launch") => "NSudo लॉन्च सहायक खोलें…",
        ("hi", "winslim_guide") => "NSudo उपयोग और सुरक्षा मार्गदर्शिका",
        ("it", "winslim_status") => "Stato di WSCore e NSudo",
        ("it", "winslim_launch") => "Apri l’assistente di avvio NSudo…",
        ("it", "winslim_guide") => "Guida all’uso e alla sicurezza di NSudo",
        ("ja", "winslim_status") => "WSCore と NSudo の状態",
        ("ja", "winslim_launch") => "NSudo 起動アシスタントを開く…",
        ("ja", "winslim_guide") => "NSudo の使用方法と安全ガイド",
        ("ko", "winslim_status") => "WSCore 및 NSudo 상태",
        ("ko", "winslim_launch") => "NSudo 실행 도우미 열기…",
        ("ko", "winslim_guide") => "NSudo 사용 및 안전 가이드",
        ("pl", "winslim_status") => "Stan WSCore i NSudo",
        ("pl", "winslim_launch") => "Otwórz asystenta uruchamiania NSudo…",
        ("pl", "winslim_guide") => "Instrukcja użycia i bezpieczeństwa NSudo",
        ("pt", "winslim_status") => "Estado do WSCore e do NSudo",
        ("pt", "winslim_launch") => "Abrir o assistente de inicialização do NSudo…",
        ("pt", "winslim_guide") => "Guia de uso e segurança do NSudo",
        ("ro", "winslim_status") => "Starea WSCore și NSudo",
        ("ro", "winslim_launch") => "Deschide asistentul de lansare NSudo…",
        ("ro", "winslim_guide") => "Ghid de utilizare și siguranță NSudo",
        ("ru", "winslim_status") => "Состояние WSCore и NSudo",
        ("ru", "winslim_launch") => "Открыть мастер запуска NSudo…",
        ("ru", "winslim_guide") => "Руководство по NSudo и безопасности",
        ("uk", "winslim_status") => "Стан WSCore і NSudo",
        ("uk", "winslim_launch") => "Відкрити помічник запуску NSudo…",
        ("uk", "winslim_guide") => "Посібник із використання NSudo та безпеки",
        ("zh", "winslim_status") => "WSCore 和 NSudo 状态",
        ("zh", "winslim_launch") => "打开 NSudo 启动助手…",
        ("zh", "winslim_guide") => "NSudo 使用与安全指南",
        ("en", "title") => "=== Automation / imported scripts ===",
        ("en", "help") => "register and run user scripts without a shell",
        ("en", "menu") => "Imported scripts and automations",
        ("en", "list") => "List registered scripts",
        ("en", "list_title") => "Registered automations",
        ("en", "add") => "Register a script",
        ("en", "run") => "Run a registered script",
        ("en", "remove") => "Remove a registration",
        ("en", "edit") => "Edit a registered script",
        ("en", "name") => "Name",
        ("en", "program") => "Program or script path",
        ("en", "working_directory") => "Working directory",
        ("en", "arguments") => "Arguments",
        ("en", "current") => "current directory",
        ("en", "registry") => "Registry",
        ("en", "none") => "No scripts registered.",
        ("en", "saved") => "Automation registered.",
        ("en", "removed") => "Registration removed.",
        ("en", "updated") => "Automation updated.",
        ("en", "command") => "Command",
        ("en", "winslim_ready") => "WinSlim integration surface detected at:",
        ("en", "winslim_placeholder") => "NSudo launches are available only through the explicit launch assistant; normal actions continue to use UAC.",
        ("en", "winslim_wscore_missing") => "WSCore: not detected",
        ("en", "winslim_nsudo_missing") => "NSudo: not detected; standard elevation continues to use UAC.",
        ("en", "winslim_nsudo_found") => "NSudo detected at:",
        ("en", "winslim_default") => "NSudo is used only by `winslim launch` or when LTOOLS_USE_NSUDO=1 explicitly selects it for a supported elevation; ordinary actions are not elevated automatically.",
        ("en", "winslim_unavailable") => {
            "The WinSlim/NSudo surface appears only on Windows when C:\\WSCore or a supported NSudo launcher is detected."
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
        (_, "edit") => "Editar un script registrado",
        (_, "name") => "Nombre",
        (_, "program") => "Ruta del programa o script",
        (_, "working_directory") => "Directorio de trabajo",
        (_, "arguments") => "Argumentos",
        (_, "current") => "directorio actual",
        (_, "registry") => "Registro",
        (_, "none") => "No hay scripts registrados.",
        (_, "saved") => "Automatización registrada.",
        (_, "removed") => "Registro eliminado.",
        (_, "updated") => "Automatización actualizada.",
        (_, "command") => "Comando",
        (_, "winslim_ready") => "Superficie de integración WinSlim detectada en:",
        (_, "winslim_placeholder") => {
            "El asistente NSudo solo se usa cuando se solicita expresamente; las acciones normales conservan UAC."
        }
        (_, "winslim_wscore_missing") => "WSCore: no detectado",
        (_, "winslim_nsudo_missing") => "NSudo: no detectado; la elevación normal continúa usando UAC.",
        (_, "winslim_nsudo_found") => "NSudo detectado en:",
        (_, "winslim_default") => "NSudo solo se usa con `winslim launch` o si LTOOLS_USE_NSUDO=1 lo selecciona expresamente para una elevación compatible; las acciones normales no se elevan automáticamente.",
        (_, "winslim_unavailable") => {
            "La superficie WinSlim/NSudo solo aparece en Windows cuando se detecta C:\\WSCore o un lanzador NSudo compatible."
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
            "storage_partitions",
            [
                "الأقسام والجداول",
                "Partitionen und Tabellen",
                "Partitions and tables",
                "Particionado y tablas",
                "Partitions et tables",
                "पार्टीशन और तालिकाएँ",
                "Partizioni e tabelle",
                "パーティションとテーブル",
                "파티션 및 테이블",
                "Partycje i tablice",
                "Partições e tabelas",
                "Partiții și tabele",
                "Разделы и таблицы",
                "Розділи та таблиці",
                "分区和分区表",
            ],
        ),
        (
            "storage_filesystems",
            [
                "أنظمة الملفات",
                "Dateisysteme",
                "Filesystems",
                "Sistemas de archivos",
                "Systèmes de fichiers",
                "फाइल सिस्टम",
                "Sistemi di file",
                "ファイルシステム",
                "파일 시스템",
                "Systemy plików",
                "Sistemas de ficheiros",
                "Sisteme de fișiere",
                "Файловые системы",
                "Файлові системи",
                "文件系统",
            ],
        ),
        (
            "storage_volumes",
            [
                "التشفير ووحدات التخزين",
                "Verschlüsselung und Volumes",
                "Encryption and volumes",
                "Cifrado y volúmenes",
                "Chiffrement et volumes",
                "एन्क्रिप्शन और वॉल्यूम",
                "Crittografia e volumi",
                "暗号化とボリューム",
                "암호화 및 볼륨",
                "Szyfrowanie i woluminy",
                "Encriptação e volumes",
                "Criptare și volume",
                "Шифрование и тома",
                "Шифрування та томи",
                "加密和卷",
            ],
        ),
        (
            "wine_prefixes",
            [
                "إدارة بادئات Wine وProton",
                "Wine- und Proton-Präfixe verwalten",
                "Manage Wine and Proton prefixes",
                "Gestión de prefijos Wine y Proton",
                "Gérer les préfixes Wine et Proton",
                "Wine और Proton प्रीफ़िक्स प्रबंधित करें",
                "Gestisci prefissi Wine e Proton",
                "Wine と Proton のプレフィックス管理",
                "Wine 및 Proton 프리픽스 관리",
                "Zarządzanie prefiksami Wine i Proton",
                "Gerir prefixos Wine e Proton",
                "Gestionează prefixele Wine și Proton",
                "Управление префиксами Wine и Proton",
                "Керування префіксами Wine та Proton",
                "管理 Wine 和 Proton 前缀",
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
            "installable_ssh",
            [
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
                "SSH / SCP / SFTP",
            ],
        ),
        (
            "installable_android",
            [
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
                "Android (ADB)",
            ],
        ),
        (
            "installable_utilities",
            [
                "أدوات النظام",
                "System utilities",
                "System utilities",
                "Utilidades del sistema",
                "Utilitaires système",
                "सिस्टम उपयोगिताएँ",
                "Utilità di sistema",
                "システムユーティリティ",
                "시스템 유틸리티",
                "Narzędzia systemowe",
                "Utilitários do sistema",
                "Utilitare de sistem",
                "Системные утилиты",
                "Системні утиліти",
                "系统工具",
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
            "kubernetes",
            [
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
                "Kubernetes",
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
        boot_label, category_text, gui_account_text, gui_action_text, gui_catalog_text,
        gui_confirmation_text, gui_family_text, gui_text, help_extra, language_test_guard,
        native_tools_label, normalize, set, settings_text, storage_action_text,
        storage_section_text, SUPPORTED,
    };
    #[cfg(not(windows))]
    use super::{gui_native_prompt, system_page_text};

    #[test]
    fn normalizes_language_variants() {
        let _guard = language_test_guard();
        assert_eq!(normalize("en_US.UTF-8"), "en");
        assert_eq!(normalize("pt-BR"), "pt");
        assert_eq!(normalize("unknown"), "es");
    }

    #[test]
    fn exposes_the_supported_catalog_languages() {
        let _guard = language_test_guard();
        assert_eq!(
            SUPPORTED,
            &[
                "ar", "de", "en", "es", "fr", "hi", "it", "ja", "ko", "pl", "pt", "ro", "ru", "uk",
                "zh",
            ]
        );
    }

    #[test]
    fn exposes_storage_map_labels_in_every_language() {
        let _guard = language_test_guard();
        for language in SUPPORTED {
            set(language);
            for key in [
                "hint",
                "column",
                "expand",
                "collapse",
                "elevate",
                "copy",
                "move",
                "delete",
                "close",
                "scan_initial",
                "scan_progress",
                "scan_timeout",
                "scan_ready",
                "scan_permissions",
                "scan_request_admin",
                "scan_error",
                "scan_empty",
                "scan_panic",
                "selected_required",
                "copy_title",
                "move_title",
                "delete_title",
                "state_inaccessible",
                "state_protected",
                "state_writable",
                "state_read_only",
                "kind_file",
                "kind_directory",
                "kind_symlink",
                "kind_other",
                "kind_inaccessible",
                "kind_missing",
                "filesystem_usage",
                "explain_system_root",
                "explain_boot",
                "explain_system_config",
                "explain_managed_programs",
                "explain_variable_data",
                "explain_user_data",
                "explain_temporary",
                "explain_virtual_proc",
                "explain_virtual_sys",
                "explain_device_nodes",
                "explain_runtime",
                "explain_user_config",
                "explain_user_cache",
                "explain_windows_os",
                "explain_shared_app_data",
                "explain_user_profiles",
                "explain_user_appdata",
                "explain_windows_metadata",
            ] {
                assert!(!super::storage_map_text(key).is_empty());
            }
            for (key, markers) in [
                ("scan_progress", &["{visited}"][..]),
                ("scan_timeout", &["{seconds}", "{visited}"][..]),
                ("scan_ready", &["{visited}"][..]),
                ("scan_permissions", &["{visited}", "{inaccessible}"][..]),
                ("scan_error", &["{error}"][..]),
                (
                    "filesystem_usage",
                    &["{total}", "{used}", "{free}", "{available}"][..],
                ),
            ] {
                let message = super::storage_map_text(key);
                for marker in markers {
                    assert!(
                        message.contains(marker),
                        "missing format marker {marker} in {language}/{key}"
                    );
                }
            }
            if *language != "es" {
                for key in [
                    "scan_initial",
                    "scan_ready",
                    "scan_permissions",
                    "scan_request_admin",
                    "scan_empty",
                ] {
                    let message = super::storage_map_text(key);
                    assert!(
                        !message.contains("Mapa listo"),
                        "Spanish map status in {language}/{key}"
                    );
                    assert!(
                        !message.contains("Analizando discos"),
                        "Spanish map status in {language}/{key}"
                    );
                    assert!(
                        !message.contains("Solicitando permisos"),
                        "Spanish map status in {language}/{key}"
                    );
                    assert!(
                        !message.contains("El escaneo terminó"),
                        "Spanish map status in {language}/{key}"
                    );
                }
            }
        }
        set("es");
    }

    #[test]
    fn exposes_all_main_menu_categories_in_every_language() {
        let _guard = language_test_guard();
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
        let _guard = language_test_guard();
        for language in SUPPORTED {
            set(language);
            for key in [
                "theme",
                "language",
                "color",
                "current",
                "update_check",
                "update_download",
            ] {
                assert!(!settings_text(key).is_empty());
            }
        }
        set("es");
    }

    #[test]
    fn exposes_automation_navigation_text() {
        let _guard = language_test_guard();
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
                "field_name",
                "field_program",
                "field_working_directory",
                "field_arguments",
                "save",
                "refresh",
                "none",
                "action_run",
                "action_edit",
                "action_remove",
                "winslim_opened",
                "update_opened",
            ] {
                assert!(!super::automation_text(key).is_empty());
            }
        }
        set("es");
    }

    #[test]
    fn gui_catalog_bridge_labels_exist_in_every_terminal_language() {
        let _guard = language_test_guard();
        for language in SUPPORTED {
            set(language);
            for key in [
                "native_guide",
                "dependencies_guide",
                "installable_guide",
                "automation_guide",
                "registered_scripts",
                "register_script",
                "scripts_guide",
            ] {
                assert!(
                    !gui_catalog_text(key).is_empty(),
                    "{language} missing {key}"
                );
            }
        }
        set("es");
    }

    #[test]
    fn core_ui_does_not_fall_back_to_spanish_for_supported_languages() {
        let _guard = language_test_guard();
        let spanish = [
            "Herramientas seguras del sistema y acciones rápidas",
            "Auditar discos y aplicaciones",
            "Discos y particiones",
            "Guía de particionado y protecciones",
            "Herramientas SSH, Android, Docker y Kubernetes",
        ];
        for language in SUPPORTED {
            set(language);
            assert!(
                !gui_text("failed").is_empty(),
                "{language} missing failed status"
            );
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
        let _guard = language_test_guard();
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
                "settings_apply",
                "settings_guide",
                "elevation_default",
                "elevation_enabled",
                "elevation_disabled",
                "elevation_scope",
                "elevation_prompt",
                "elevation_save_error",
                "update_check",
                "update_download",
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
    fn elevation_cli_feedback_does_not_fall_back_to_spanish() {
        let _guard = language_test_guard();
        let spanish = [
            gui_text("elevation_default"),
            gui_text("elevation_scope"),
            gui_text("elevation_prompt"),
            gui_text("elevation_save_error"),
        ];
        for language in SUPPORTED {
            set(language);
            if *language != "es" {
                for (key, spanish_value) in [
                    ("elevation_default", spanish[0]),
                    ("elevation_scope", spanish[1]),
                    ("elevation_prompt", spanish[2]),
                    ("elevation_save_error", spanish[3]),
                ] {
                    assert_ne!(
                        gui_text(key),
                        spanish_value,
                        "Spanish elevation feedback in {language}/{key}"
                    );
                }
            }
        }
        set("es");
    }

    #[test]
    fn gui_family_labels_are_available_in_every_terminal_language() {
        let _guard = language_test_guard();
        let families = [
            "native_storage",
            "storage_partitions",
            "storage_filesystems",
            "storage_volumes",
            "wine_prefixes",
            "native_system",
            "installable_connectivity",
            "installable_ssh",
            "installable_android",
            "installable_utilities",
            "installable_docker",
            "kubernetes",
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
    fn storage_section_labels_are_available_in_every_terminal_language() {
        let _guard = language_test_guard();
        for language in SUPPORTED {
            set(language);
            for key in [
                "query_map",
                "file_paths",
                "disk_volumes",
                "tools_cleanup",
                "partition_inspection",
                "partition_tables",
                "partition_recovery",
                "partition_destruction",
            ] {
                assert!(
                    !storage_section_text(key).is_empty(),
                    "{language} missing {key}"
                );
            }
        }
        set("es");
    }

    #[test]
    #[cfg(not(windows))]
    fn network_boot_and_services_pages_are_translated_in_every_language() {
        let _guard = language_test_guard();
        let keys = [
            "native_hardware_status",
            "native_power_status",
            "native_security_status",
            "native_security_scanners",
            "network_title",
            "network_inspection",
            "network_status",
            "network_interfaces",
            "network_routes",
            "network_dns",
            "network_listening",
            "network_connections",
            "network_flush_dns",
            "network_management",
            "network_interface_manage",
            "network_connect",
            "network_disconnect",
            "network_guide",
            "boot_title",
            "boot_inspection",
            "boot_status",
            "efi_entries",
            "grub_entries",
            "systemd_boot",
            "secure_boot",
            "boot_plan",
            "boot_next_changes",
            "grub_schedule",
            "grub_cancel",
            "boot_guide",
            "services_title",
            "services_inventory",
            "services_automatic",
            "services_manual",
            "services_user",
            "services_both",
            "services_failed",
            "services_management",
            "services_manage",
            "services_export",
            "services_guide",
        ];
        set("es");
        let spanish_page_titles = [
            system_page_text("network_title"),
            system_page_text("boot_title"),
            system_page_text("services_title"),
            system_page_text("native_security_scanners"),
        ];
        assert_eq!(
            system_page_text("network_title"),
            "Red, rutas, DNS y puertos escuchando"
        );
        assert_eq!(system_page_text("efi_entries"), "Entradas EFI / NVRAM");
        assert_eq!(
            system_page_text("services_both"),
            "Todos: sistema y usuario"
        );

        for language in SUPPORTED {
            set(language);
            for key in keys {
                let translated = system_page_text(key);
                assert!(!translated.trim().is_empty(), "{language} missing {key}");
            }
            if *language != "es" {
                for (index, key) in [
                    "network_title",
                    "boot_title",
                    "services_title",
                    "native_security_scanners",
                ]
                .iter()
                .enumerate()
                {
                    assert_ne!(
                        system_page_text(key),
                        spanish_page_titles[index],
                        "{language} fell back to Spanish for {key}"
                    );
                }
            }
        }
        set("es");
    }

    #[test]
    #[cfg(not(windows))]
    fn native_network_boot_and_service_prompts_are_localized() {
        let _guard = language_test_guard();
        let prompts = [
            "Interfaz de red (ej. eth0)",
            "Estado: up o down",
            "Nombre exacto de la conexión NetworkManager",
            "Título exacto de la entrada GRUB (ej. Ubuntu)",
            "Ámbito: system o user",
            "Unidad (ej. sshd.service)",
            "Acción: status, start, stop, restart, enable, disable, mask o unmask",
        ];
        set("es");
        for prompt in prompts {
            assert_eq!(gui_native_prompt(prompt), prompt);
        }
        let spanish_interface = gui_native_prompt(prompts[0]);
        for language in SUPPORTED {
            set(language);
            for prompt in prompts {
                let translated = gui_native_prompt(prompt);
                assert!(!translated.trim().is_empty(), "{language}: {prompt}");
            }
            if *language != "es" {
                assert_ne!(
                    gui_native_prompt(prompts[0]),
                    spanish_interface,
                    "{language} fell back to Spanish"
                );
            }
        }
        set("es");
    }

    #[test]
    fn account_gui_labels_exist_in_every_terminal_language() {
        let _guard = language_test_guard();
        let keys = [
            "title",
            "manage_accounts",
            "manage_groups",
            "team_admin",
            "list",
            "groups",
            "identity",
            "sessions",
            "inspect",
            "create",
            "modify",
            "password",
            "lock",
            "unlock",
            "delete",
            "expire",
            "group_create",
            "group_delete",
            "group_primary",
            "group_add",
            "group_remove",
            "admin_add",
            "admin_groups",
            "guide",
            "field_user_group",
            "field_details",
            "field_confirm",
            "field_optional",
            "local_user",
            "new_password",
            "repeat_password",
            "guide_fields",
            "guide_simple",
            "guide_complex",
            "guide_fields_linux",
            "guide_complex_linux",
        ];
        for language in SUPPORTED {
            set(language);
            for key in keys {
                assert!(
                    !gui_account_text(key).trim().is_empty(),
                    "{language} missing account GUI string {key}"
                );
            }
            if *language != "es" {
                assert_ne!(gui_account_text("title"), "Usuarios, grupos y sesiones");
                assert_ne!(gui_account_text("list"), "Listar cuentas locales");
                assert_ne!(gui_account_text("password"), "Cambiar contraseña");
            }
        }
        set("es");
    }

    #[test]
    fn modal_actions_are_translated_in_every_terminal_language() {
        let _guard = language_test_guard();
        for language in SUPPORTED {
            set(language);
            assert!(!gui_action_text("yes").is_empty());
            assert!(!gui_action_text("no").is_empty());
            assert!(!gui_action_text("cancel").is_empty());
            assert!(!gui_action_text("cancelling").is_empty());
            for key in [
                "warning_data",
                "warning_system",
                "review",
                "details",
                "boot_no_reboot",
                "cleanup_warning",
            ] {
                assert!(
                    !gui_confirmation_text(key).is_empty(),
                    "{language} missing confirmation text {key}"
                );
            }
        }
        set("es");
    }

    #[test]
    fn global_help_extras_are_present_and_localized() {
        let _guard = language_test_guard();
        let keys = [
            "report",
            "privileges",
            "privilege_policy",
            "elevation_hint",
            "doctor_install",
            "winslim",
            "release_manifest",
            "release_checksums",
            "release_signature",
            "update",
            "git",
            "scanner_note",
        ];
        set("es");
        let spanish_report = help_extra("report");
        for language in SUPPORTED {
            set(language);
            for key in keys {
                assert!(!help_extra(key).trim().is_empty(), "{language}: {key}");
            }
            if *language != "es" {
                assert_ne!(help_extra("report"), spanish_report, "{language}: report");
            }
        }
        set("es");
    }
}
