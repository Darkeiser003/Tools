//! Lectura integrada de informes generados por LTools.
//!
//! La lectura es de solo lectura. El editor es una acción explícita del
//! usuario y se ejecuta sin shell para no reinterpretar rutas ni variables.

use crate::common::{command_exists, Context};
use crate::i18n;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(_ctx: &Context, args: &[String]) -> Result<(), String> {
    let action = args
        .iter()
        .find(|value| !value.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("view");
    let path = args
        .iter()
        .position(|value| value == "--path" || value == "--file")
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .or_else(|| {
            args.iter()
                .find(|value| !value.starts_with('-') && value.as_str() != action)
                .map(PathBuf::from)
        })
        .ok_or("report requiere una ruta de fichero o directorio")?;
    match action {
        "view" | "read" | "cat" => view(&resolve_primary(&path), ViewMode::Cat),
        "pager" | "less" | "more" => view(&resolve_primary(&path), ViewMode::Pager),
        "edit" | "editor" => view(&resolve_primary(&path), ViewMode::Editor),
        "menu" | "open" => interactive(&path),
        _ => Err("report admite menu, view, pager o editor".into()),
    }
}

#[derive(Clone, Copy)]
enum ViewMode {
    Cat,
    Pager,
    Editor,
}

pub fn interactive(report: &Path) -> Result<(), String> {
    let files = report_files(report)?;
    if files.is_empty() {
        return Err(format!("no hay informes legibles en {}", report.display()));
    }
    loop {
        println!("\n{}: {}", t("report"), report.display());
        for (index, path) in files.iter().enumerate() {
            println!(
                "  {}) {}",
                index + 1,
                path.file_name().unwrap_or_default().to_string_lossy()
            );
        }
        println!("  c) {}", t("summary"));
        println!("  e) {}", t("editor"));
        println!("  p) {}", t("pager"));
        println!("  q) {}", t("back"));
        print!("{} ", t("prompt"));
        let _ = io::stdout().flush();
        let mut answer = String::new();
        if io::stdin()
            .read_line(&mut answer)
            .map_err(|e| e.to_string())?
            == 0
        {
            return Ok(());
        }
        let answer = answer.trim().to_lowercase();
        if answer.is_empty() || matches!(answer.as_str(), "q" | "quit" | "salir") {
            return Ok(());
        }
        let (path, mode) = match answer.as_str() {
            "c" => (resolve_primary(report), ViewMode::Cat),
            "e" => (resolve_primary(report), ViewMode::Editor),
            "p" => (resolve_primary(report), ViewMode::Pager),
            value => match value.parse::<usize>() {
                Ok(index) if (1..=files.len()).contains(&index) => {
                    (files[index - 1].clone(), ViewMode::Cat)
                }
                _ => {
                    println!("{}", t("invalid"));
                    continue;
                }
            },
        };
        if let Err(error) = view(&path, mode) {
            eprintln!("Error al abrir el informe: {error}");
        }
    }
}

fn t(key: &str) -> &'static str {
    match (i18n::current(), key) {
        ("en", "report") => "Report",
        ("en", "summary") => "Read the main summary",
        ("en", "editor") => "Open a report in the editor",
        ("en", "pager") => "Read with pager",
        ("en", "back") => "Back",
        ("en", "prompt") => "Choose an option (Enter to go back):",
        ("en", "invalid") => "Invalid option.",
        ("de", "report") => "Bericht",
        ("de", "summary") => "Hauptzusammenfassung lesen",
        ("de", "editor") => "Bericht im Editor öffnen",
        ("de", "pager") => "Mit Pager lesen",
        ("de", "back") => "Zurück",
        ("de", "prompt") => "Option wählen (Enter zum Zurückgehen):",
        ("de", "invalid") => "Ungültige Option.",
        ("fr", "report") => "Rapport",
        ("fr", "summary") => "Lire le résumé principal",
        ("fr", "editor") => "Ouvrir un rapport dans l’éditeur",
        ("fr", "pager") => "Lire avec le pager",
        ("fr", "back") => "Retour",
        ("fr", "prompt") => "Choisissez une option (Entrée pour revenir) :",
        ("fr", "invalid") => "Option invalide.",
        ("pt", "report") => "Relatório",
        ("pt", "summary") => "Ler o resumo principal",
        ("pt", "editor") => "Abrir um relatório no editor",
        ("pt", "pager") => "Ler com paginador",
        ("pt", "back") => "Voltar",
        ("pt", "prompt") => "Escolha uma opção (Enter para voltar):",
        ("pt", "invalid") => "Opção inválida.",
        ("it", "report") => "Rapporto",
        ("it", "summary") => "Leggi il riepilogo principale",
        ("it", "editor") => "Apri un rapporto nell’editor",
        ("it", "pager") => "Leggi con il paginatore",
        ("it", "back") => "Indietro",
        ("it", "prompt") => "Scegli un’opzione (Invio per tornare):",
        ("it", "invalid") => "Opzione non valida.",
        ("ca", "report") => "Informe",
        ("ca", "summary") => "Llegir el resum principal",
        ("ca", "editor") => "Obrir un informe a l’editor",
        ("ca", "pager") => "Llegir amb paginador",
        ("ca", "back") => "Tornar",
        ("ca", "prompt") => "Tria una opció (Enter per tornar):",
        ("ca", "invalid") => "Opció no vàlida.",
        ("nl", "report") => "Rapport",
        ("nl", "summary") => "Hoofdsamenvatting lezen",
        ("nl", "editor") => "Een rapport in de editor openen",
        ("nl", "pager") => "Met pager lezen",
        ("nl", "back") => "Terug",
        ("nl", "prompt") => "Kies een optie (Enter om terug te gaan):",
        ("nl", "invalid") => "Ongeldige optie.",
        ("pl", "report") => "Raport",
        ("pl", "summary") => "Czytaj główne podsumowanie",
        ("pl", "editor") => "Otwórz raport w edytorze",
        ("pl", "pager") => "Czytaj w pagerze",
        ("pl", "back") => "Wstecz",
        ("pl", "prompt") => "Wybierz opcję (Enter, aby wrócić):",
        ("pl", "invalid") => "Nieprawidłowa opcja.",
        (_, "report") => "Informe",
        (_, "summary") => "Leer el resumen principal",
        (_, "editor") => "Abrir un informe en el editor",
        (_, "pager") => "Leer con paginador",
        (_, "back") => "Volver",
        (_, "prompt") => "Elige una opción (Enter para volver):",
        (_, "invalid") => "Opción no válida.",
        (_, _) => "",
    }
}

fn view(path: &Path, mode: ViewMode) -> Result<(), String> {
    let metadata =
        fs::metadata(path).map_err(|e| format!("no se puede leer {}: {e}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("no es un fichero legible: {}", path.display()));
    }
    match mode {
        ViewMode::Cat => {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("no se puede abrir {}: {e}", path.display()))?;
            print!("\n===== {} =====\n{}", path.display(), content);
            if !content.ends_with('\n') {
                println!();
            }
            Ok(())
        }
        ViewMode::Pager => {
            for pager in pager_candidates() {
                if !command_exists(&pager) {
                    continue;
                }
                let status = Command::new(pager)
                    .arg(path)
                    .status()
                    .map_err(|e| e.to_string())?;
                if status.success() {
                    return Ok(());
                }
            }
            view(path, ViewMode::Cat)
        }
        ViewMode::Editor => {
            let editor = editor_command()
                .ok_or("no se encontró editor; define EDITOR o VISUAL, o instala nano/vi")?;
            let status = Command::new(&editor[0])
                .args(&editor[1..])
                .arg(path)
                .status()
                .map_err(|e| format!("no se pudo iniciar el editor: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("el editor terminó con código {}", status))
            }
        }
    }
}

fn resolve_primary(path: &Path) -> PathBuf {
    if path.is_file() {
        return path.to_path_buf();
    }
    for name in ["summary.txt", "inventory.tsv", "packages.tsv", "report.txt"] {
        let candidate = path.join(name);
        if candidate.is_file() {
            return candidate;
        }
    }
    path.to_path_buf()
}

fn report_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    let mut files = fs::read_dir(path)
        .map_err(|e| format!("no se puede listar {}: {e}", path.display()))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|candidate| {
            candidate.is_file()
                && candidate
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| matches!(extension, "txt" | "tsv" | "json"))
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|candidate| {
        let name = candidate.file_name().unwrap_or_default().to_string_lossy();
        (
            !name.eq_ignore_ascii_case("summary.txt"),
            name.to_lowercase(),
        )
    });
    Ok(files)
}

fn pager_candidates() -> Vec<String> {
    let configured = std::env::var("PAGER").ok();
    configured
        .as_deref()
        .and_then(|value| value.split_whitespace().next())
        .map(ToOwned::to_owned)
        .into_iter()
        .chain(["less".to_owned(), "more".to_owned()])
        .collect()
}

fn editor_command() -> Option<Vec<String>> {
    let configured = std::env::var("VISUAL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("EDITOR")
                .ok()
                .filter(|value| !value.trim().is_empty())
        });
    if let Some(value) = configured {
        let command = value
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if command
            .first()
            .is_some_and(|program| command_exists(program))
        {
            return Some(command);
        }
    }
    ["nano", "vi", "vim"]
        .iter()
        .find_map(|program| command_exists(program).then(|| vec![(*program).to_owned()]))
}
