#!/usr/bin/env bash
# Catálogo pequeño y sin dependencias para la interfaz común de LTools.
# Los módulos pueden crecer hacia este catálogo sin acoplarse al idioma del SO.

set -u

ltools_normalize_lang() {
    local raw="${1:-}" code
    raw="${raw,,}"
    code="${raw%%[_-]*}"
    case "$code" in
        es|en|de|fr|pt|it|ca|nl|pl) printf '%s' "$code" ;;
        *) printf 'es' ;;
    esac
}

ltools_i18n_set() {
    LTOOLS_LANG_CODE="$(ltools_normalize_lang "${1:-}")"
    export LTOOLS_LANG_CODE
}

ltools_i18n_init() {
    local selected="${LTOOLS_LANG:-${LC_ALL:-${LC_MESSAGES:-${LANG:-es}}}}"
    ltools_i18n_set "$selected"
}

ltools_t() {
    local key="$1"
    case "${LTOOLS_LANG_CODE:-es}:$key" in
        es:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        es:menu.audit) printf 'Auditar discos, paquetes y aplicaciones' ;;
        es:menu.games) printf 'Auditar juegos, Wine y Proton' ;;
        es:menu.clean) printf 'Limpiar paquetes, cachés y rutas' ;;
        es:menu.prefix) printf 'Crear o migrar un prefijo Wine' ;;
        es:menu.defaults) printf 'Ver rutas y defaults de Wine/Proton' ;;
        es:menu.packages) printf 'Inventario de paquetes y almacenes' ;;
        es:menu.system) printf 'Servicios, procesos y daemons del sistema' ;;
        es:menu.doctor) printf 'Diagnóstico de dependencias y del sistema' ;;
        es:menu.help) printf 'Ayuda' ;;
        es:menu.quit) printf 'Salir' ;;
        es:menu.prompt) printf 'Elige una opción: ' ;;
        es:menu.invalid) printf 'Opción no válida.' ;;
        en:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        en:menu.audit) printf 'Audit disks, packages and applications' ;;
        en:menu.games) printf 'Audit games, Wine and Proton' ;;
        en:menu.clean) printf 'Clean packages, caches and paths' ;;
        en:menu.prefix) printf 'Create or migrate a Wine prefix' ;;
        en:menu.defaults) printf 'Show Wine/Proton paths and defaults' ;;
        en:menu.packages) printf 'Package and store inventory' ;;
        en:menu.system) printf 'System services, processes and daemons' ;;
        en:menu.doctor) printf 'Dependency and system diagnostics' ;;
        en:menu.help) printf 'Help' ;;
        en:menu.quit) printf 'Quit' ;;
        en:menu.prompt) printf 'Choose an option: ' ;;
        en:menu.invalid) printf 'Invalid option.' ;;
        de:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        de:menu.audit) printf 'Datenträger, Pakete und Anwendungen prüfen' ;;
        de:menu.games) printf 'Spiele, Wine und Proton prüfen' ;;
        de:menu.clean) printf 'Pakete, Caches und Pfade bereinigen' ;;
        de:menu.prefix) printf 'Wine-Präfix erstellen oder migrieren' ;;
        de:menu.defaults) printf 'Wine-/Proton-Pfade und Standards anzeigen' ;;
        de:menu.packages) printf 'Paket- und Store-Inventar' ;;
        de:menu.system) printf 'Systemdienste, Prozesse und Daemons' ;;
        de:menu.doctor) printf 'Abhängigkeiten und System diagnostizieren' ;;
        de:menu.help) printf 'Hilfe' ;;
        de:menu.quit) printf 'Beenden' ;;
        de:menu.prompt) printf 'Option wählen: ' ;;
        de:menu.invalid) printf 'Ungültige Option.' ;;
        fr:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        fr:menu.audit) printf 'Auditer les disques, paquets et applications' ;;
        fr:menu.games) printf 'Auditer les jeux, Wine et Proton' ;;
        fr:menu.clean) printf 'Nettoyer les paquets, caches et chemins' ;;
        fr:menu.prefix) printf 'Créer ou migrer un préfixe Wine' ;;
        fr:menu.defaults) printf 'Afficher les chemins et valeurs par défaut Wine/Proton' ;;
        fr:menu.packages) printf 'Inventaire des paquets et magasins' ;;
        fr:menu.system) printf 'Services, processus et démons du système' ;;
        fr:menu.doctor) printf 'Diagnostiquer les dépendances et le système' ;;
        fr:menu.help) printf 'Aide' ;;
        fr:menu.quit) printf 'Quitter' ;;
        fr:menu.prompt) printf 'Choisissez une option : ' ;;
        fr:menu.invalid) printf 'Option invalide.' ;;
        pt:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        pt:menu.audit) printf 'Auditar discos, pacotes e aplicações' ;;
        pt:menu.games) printf 'Auditar jogos, Wine e Proton' ;;
        pt:menu.clean) printf 'Limpar pacotes, caches e caminhos' ;;
        pt:menu.prefix) printf 'Criar ou migrar um prefixo Wine' ;;
        pt:menu.defaults) printf 'Ver caminhos e padrões do Wine/Proton' ;;
        pt:menu.packages) printf 'Inventário de pacotes e lojas' ;;
        pt:menu.system) printf 'Serviços, processos e daemons do sistema' ;;
        pt:menu.doctor) printf 'Diagnóstico de dependências e do sistema' ;;
        pt:menu.help) printf 'Ajuda' ;;
        pt:menu.quit) printf 'Sair' ;;
        pt:menu.prompt) printf 'Escolha uma opção: ' ;;
        pt:menu.invalid) printf 'Opção inválida.' ;;
        it:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        it:menu.audit) printf 'Verifica dischi, pacchetti e applicazioni' ;;
        it:menu.games) printf 'Verifica giochi, Wine e Proton' ;;
        it:menu.clean) printf 'Pulisci pacchetti, cache e percorsi' ;;
        it:menu.prefix) printf 'Crea o migra un prefisso Wine' ;;
        it:menu.defaults) printf 'Mostra percorsi e predefiniti Wine/Proton' ;;
        it:menu.packages) printf 'Inventario di pacchetti e store' ;;
        it:menu.system) printf 'Servizi, processi e demoni del sistema' ;;
        it:menu.doctor) printf 'Diagnostica dipendenze e sistema' ;;
        it:menu.help) printf 'Aiuto' ;;
        it:menu.quit) printf 'Esci' ;;
        it:menu.prompt) printf 'Scegli un’opzione: ' ;;
        it:menu.invalid) printf 'Opzione non valida.' ;;
        ca:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        ca:menu.audit) printf 'Auditar discs, paquets i aplicacions' ;;
        ca:menu.games) printf 'Auditar jocs, Wine i Proton' ;;
        ca:menu.clean) printf 'Netejar paquets, cau i rutes' ;;
        ca:menu.prefix) printf 'Crear o migrar un prefix Wine' ;;
        ca:menu.defaults) printf 'Mostrar rutes i valors predeterminats de Wine/Proton' ;;
        ca:menu.packages) printf 'Inventari de paquets i botigues' ;;
        ca:menu.system) printf 'Serveis, processos i dimonis del sistema' ;;
        ca:menu.doctor) printf 'Diagnòstic de dependències i del sistema' ;;
        ca:menu.help) printf 'Ajuda' ;;
        ca:menu.quit) printf 'Sortir' ;;
        ca:menu.prompt) printf 'Tria una opció: ' ;;
        ca:menu.invalid) printf 'Opció no vàlida.' ;;
        nl:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        nl:menu.audit) printf 'Schijven, pakketten en toepassingen controleren' ;;
        nl:menu.games) printf 'Games, Wine en Proton controleren' ;;
        nl:menu.clean) printf 'Pakketten, caches en paden opruimen' ;;
        nl:menu.prefix) printf 'Wine-prefix maken of migreren' ;;
        nl:menu.defaults) printf 'Standaardpaden van Wine/Proton tonen' ;;
        nl:menu.packages) printf 'Pakketten- en store-inventaris' ;;
        nl:menu.system) printf 'Systeemdiensten, processen en daemons' ;;
        nl:menu.doctor) printf 'Afhankelijkheden en systeem diagnosticeren' ;;
        nl:menu.help) printf 'Help' ;;
        nl:menu.quit) printf 'Afsluiten' ;;
        nl:menu.prompt) printf 'Kies een optie: ' ;;
        nl:menu.invalid) printf 'Ongeldige optie.' ;;
        pl:menu.title) printf '=== LTools %s ===' "${2:-}" ;;
        pl:menu.audit) printf 'Audyt dysków, pakietów i aplikacji' ;;
        pl:menu.games) printf 'Audyt gier, Wine i Proton' ;;
        pl:menu.clean) printf 'Czyszczenie pakietów, pamięci podręcznych i ścieżek' ;;
        pl:menu.prefix) printf 'Tworzenie lub migracja prefiksu Wine' ;;
        pl:menu.defaults) printf 'Pokaż domyślne ścieżki Wine/Proton' ;;
        pl:menu.packages) printf 'Inwentaryzacja pakietów i sklepów' ;;
        pl:menu.system) printf 'Usługi systemowe, procesy i demony' ;;
        pl:menu.doctor) printf 'Diagnostyka zależności i systemu' ;;
        pl:menu.help) printf 'Pomoc' ;;
        pl:menu.quit) printf 'Wyjście' ;;
        pl:menu.prompt) printf 'Wybierz opcję: ' ;;
        pl:menu.invalid) printf 'Nieprawidłowa opcja.' ;;
        *) ltools_i18n_set es; ltools_t "$key" "$@" ;;
    esac
}

ltools_i18n_init
