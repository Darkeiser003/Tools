//! Registro y ejecución explícita de automatizaciones del usuario.
//!
//! Las entradas se guardan como TSV en la configuración de LTools. Cada
//! campo se escapa y cada argumento conserva su propia frontera; al ejecutar
//! se usa `Command` directamente, nunca una shell. Así se pueden registrar
//! scripts propios sin convertir LTools en un lanzador arbitrario de cadenas.

use crate::common::{self, Context};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Automation {
    name: String,
    program: String,
    working_directory: Option<String>,
    args: Vec<String>,
}

fn registry_path() -> PathBuf {
    let base = if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(path)
    } else if cfg!(windows) {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| common::home_dir().join("AppData/Roaming"))
    } else {
        common::home_dir().join(".config")
    };
    base.join("ltools").join("automations.tsv")
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape(value: &str) -> String {
    let mut output = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            output.push(match character {
                't' => '\t',
                'n' => '\n',
                'r' => '\r',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    if escaped {
        output.push('\\');
    }
    output
}

fn parse_line(line: &str) -> Option<Automation> {
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let fields = line.split('\t').map(unescape).collect::<Vec<_>>();
    if fields.len() != 4 || fields[0].is_empty() || fields[1].is_empty() {
        return None;
    }
    let args = if fields[3].is_empty() {
        Vec::new()
    } else {
        fields[3].split('\u{1f}').map(str::to_owned).collect()
    };
    Some(Automation {
        name: fields[0].clone(),
        program: fields[1].clone(),
        working_directory: (!fields[2].is_empty()).then_some(fields[2].clone()),
        args,
    })
}

fn load() -> Result<Vec<Automation>, String> {
    let path = registry_path();
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("no se pudo leer {}: {error}", path.display())),
    };
    Ok(content.lines().filter_map(parse_line).collect())
}

fn save(entries: &[Automation]) -> Result<(), String> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("no se pudo crear la configuración: {error}"))?;
    }
    let mut content =
        String::from("# ltools-automation-v1\n# name\tprogram\tworking_directory\targs\n");
    for entry in entries {
        let args = entry
            .args
            .iter()
            .map(|arg| escape(arg))
            .collect::<Vec<_>>()
            .join("\u{1f}");
        content.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            escape(&entry.name),
            escape(&entry.program),
            escape(entry.working_directory.as_deref().unwrap_or("")),
            args
        ));
    }
    fs::write(&path, content)
        .map_err(|error| format!("no se pudo guardar {}: {error}", path.display()))
}

fn split_arguments(input: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in input.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if quote == Some(character) {
            quote = None;
        } else if quote.is_none() && (character == '\'' || character == '"') {
            quote = Some(character);
        } else if quote.is_none() && character.is_whitespace() {
            if !current.is_empty() {
                values.push(std::mem::take(&mut current));
            }
        } else {
            current.push(character);
        }
    }
    if escaped {
        current.push('\\');
    }
    if quote.is_some() {
        return Err("los argumentos tienen comillas sin cerrar".into());
    }
    if !current.is_empty() {
        values.push(current);
    }
    Ok(values)
}

fn input(prompt: &str) -> Option<String> {
    use std::io::{self, Write};
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut value = String::new();
    match io::stdin().read_line(&mut value) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(value.trim().to_owned()),
    }
}

fn validate(entry: &Automation) -> Result<(), String> {
    if entry.name.is_empty() || entry.name.contains(['\t', '\n', '\r']) {
        return Err("el nombre no puede estar vacío ni contener saltos de línea".into());
    }
    if entry.program.trim().is_empty() {
        return Err("el programa o script no puede estar vacío".into());
    }
    if let Some(directory) = &entry.working_directory {
        if !Path::new(directory).is_dir() {
            return Err(format!("el directorio de trabajo no existe: {directory}"));
        }
    }
    Ok(())
}

fn list() -> Result<(), String> {
    let entries = load()?;
    println!("{}", crate::i18n::automation_text("list_title"));
    if entries.is_empty() {
        println!("{}", crate::i18n::automation_text("none"));
        return Ok(());
    }
    for (index, entry) in entries.iter().enumerate() {
        println!("  {:>2}) {} → {}", index + 1, entry.name, entry.program);
        if let Some(directory) = &entry.working_directory {
            println!(
                "      {}: {}",
                crate::i18n::automation_text("working_directory"),
                directory
            );
        }
        if !entry.args.is_empty() {
            println!(
                "      {}: {}",
                crate::i18n::automation_text("arguments"),
                entry.args.join(" ")
            );
        }
    }
    println!(
        "{}: {}",
        crate::i18n::automation_text("registry"),
        registry_path().display()
    );
    Ok(())
}

fn add(entry: Automation, ctx: &Context) -> Result<(), String> {
    validate(&entry)?;
    let mut entries = load()?;
    if entries
        .iter()
        .any(|old| old.name.eq_ignore_ascii_case(&entry.name))
    {
        return Err(format!(
            "ya existe una automatización llamada {}",
            entry.name
        ));
    }
    if ctx.dry_run {
        println!(
            "Simulación: se registraría {} → {}",
            entry.name, entry.program
        );
        return Ok(());
    }
    entries.push(entry.clone());
    save(&entries)?;
    if let Some(plan) = &ctx.plan {
        let _ = plan.record(
            "automation-register",
            Path::new(&entry.program),
            "executed",
            true,
            &entry.name,
            &registry_path().display().to_string(),
        );
    }
    println!("{}", crate::i18n::automation_text("saved"));
    Ok(())
}

fn remove(name: &str, ctx: &Context) -> Result<(), String> {
    let mut entries = load()?;
    let old_len = entries.len();
    entries.retain(|entry| !entry.name.eq_ignore_ascii_case(name));
    if entries.len() == old_len {
        return Err(format!("no existe una automatización llamada {name}"));
    }
    if ctx.dry_run {
        println!("Simulación: se eliminaría la entrada {name}.");
        return Ok(());
    }
    save(&entries)?;
    println!("{}", crate::i18n::automation_text("removed"));
    Ok(())
}

fn command_for(entry: &Automation) -> (String, Vec<String>) {
    let path = Path::new(&entry.program);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut args = Vec::new();
    let program = if cfg!(windows) && extension == "ps1" {
        args.extend([
            "-NoProfile".into(),
            "-ExecutionPolicy".into(),
            "Bypass".into(),
            "-File".into(),
            entry.program.clone(),
        ]);
        "powershell".into()
    } else if !cfg!(windows) && extension == "sh" {
        args.extend([entry.program.clone()]);
        "bash".into()
    } else if !cfg!(windows) && extension == "py" {
        args.extend([entry.program.clone()]);
        "python3".into()
    } else if cfg!(windows) && matches!(extension.as_str(), "cmd" | "bat") {
        args.extend(["/d".into(), "/c".into(), entry.program.clone()]);
        "cmd".into()
    } else {
        entry.program.clone()
    };
    args.extend(entry.args.clone());
    (program, args)
}

fn run_named(ctx: &Context, name: &str, extra: &[String]) -> Result<(), String> {
    let entry = load()?
        .into_iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("no existe una automatización llamada {name}"))?;
    validate(&entry)?;
    let (program, mut args) = command_for(&entry);
    args.extend_from_slice(extra);
    println!(
        "{}: {} {}",
        crate::i18n::automation_text("command"),
        program,
        args.iter()
            .map(|arg| common::shell_display(arg))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if ctx.dry_run {
        println!("Simulación: no se ejecutó la automatización.");
        return Ok(());
    }
    let mut command = Command::new(&program);
    command.args(&args);
    if let Some(directory) = &entry.working_directory {
        command.current_dir(directory);
    }
    let status = command
        .status()
        .map_err(|error| format!("no se pudo ejecutar {}: {error}", entry.name))?;
    if let Some(plan) = &ctx.plan {
        let _ = plan.record(
            "automation-run",
            Path::new(&entry.program),
            if status.success() {
                "executed"
            } else {
                "failed"
            },
            false,
            &entry.name,
            &status.to_string(),
        );
    }
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("la automatización terminó con {status}"))
}

fn interactive_menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("{}", crate::i18n::automation_text("title"));
        println!("  1) {}", crate::i18n::automation_text("list"));
        println!("  2) {}", crate::i18n::automation_text("add"));
        println!("  3) {}", crate::i18n::automation_text("run"));
        println!("  4) {}", crate::i18n::automation_text("remove"));
        println!("  q) {}", crate::i18n::text("menu.back"));
        let Some(answer) = input(crate::i18n::text("menu.prompt")) else {
            return Ok(());
        };
        match answer.to_ascii_lowercase().as_str() {
            "" | "q" | "quit" | "salir" => return Ok(()),
            "1" => {
                list()?;
                let _ = input(crate::i18n::tools_text("pause"));
            }
            "2" => {
                let Some(name) = input(&format!("{}: ", crate::i18n::automation_text("name")))
                else {
                    continue;
                };
                let Some(program) =
                    input(&format!("{}: ", crate::i18n::automation_text("program")))
                else {
                    continue;
                };
                let directory = input(&format!(
                    "{} (Enter = {}): ",
                    crate::i18n::automation_text("working_directory"),
                    crate::i18n::automation_text("current")
                ))
                .filter(|value| !value.is_empty());
                let raw_args = input(&format!(
                    "{} (comillas para espacios): ",
                    crate::i18n::automation_text("arguments")
                ))
                .unwrap_or_default();
                match split_arguments(&raw_args).and_then(|args| {
                    add(
                        Automation {
                            name,
                            program,
                            working_directory: directory,
                            args,
                        },
                        ctx,
                    )
                }) {
                    Ok(()) => {}
                    Err(error) => eprintln!("Error: {error}"),
                }
                let _ = input(crate::i18n::tools_text("pause"));
            }
            "3" => {
                let Some(name) = input(&format!("{}: ", crate::i18n::automation_text("name")))
                else {
                    continue;
                };
                if let Err(error) = run_named(ctx, &name, &[]) {
                    eprintln!("Error: {error}");
                }
                let _ = input(crate::i18n::tools_text("pause"));
            }
            "4" => {
                let Some(name) = input(&format!("{}: ", crate::i18n::automation_text("name")))
                else {
                    continue;
                };
                if let Err(error) = remove(&name, ctx) {
                    eprintln!("Error: {error}");
                }
                let _ = input(crate::i18n::tools_text("pause"));
            }
            _ => {
                println!("{}", crate::i18n::text("menu.invalid"));
                let _ = input(crate::i18n::tools_text("pause"));
            }
        }
    }
}

pub fn run(ctx: &Context, args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str).unwrap_or("menu") {
        "menu" => interactive_menu(ctx),
        "list" => list(),
        "add" | "register" => {
            let mut name = None;
            let mut program = None;
            let mut directory = None;
            let mut script_args = Vec::new();
            let mut index = 1;
            while index < args.len() {
                match args[index].as_str() {
                    "--name" => {
                        index += 1;
                        name = args.get(index).cloned();
                    }
                    "--program" | "--script" => {
                        index += 1;
                        program = args.get(index).cloned();
                    }
                    "--working-directory" | "--cwd" => {
                        index += 1;
                        directory = args.get(index).cloned();
                    }
                    "--arg" => {
                        index += 1;
                        if let Some(value) = args.get(index) {
                            script_args.push(value.clone());
                        }
                    }
                    "--args" => {
                        index += 1;
                        if let Some(value) = args.get(index) {
                            script_args.extend(split_arguments(value)?);
                        }
                    }
                    other => {
                        return Err(format!("opción desconocida para automation add: {other}"))
                    }
                }
                index += 1;
            }
            add(
                Automation {
                    name: name.ok_or("falta --name")?,
                    program: program.ok_or("falta --program")?,
                    working_directory: directory,
                    args: script_args,
                },
                ctx,
            )
        }
        "run" | "execute" => {
            let name = args.get(1).ok_or("falta el nombre de la automatización")?;
            run_named(ctx, name, &args[2..])
        }
        "remove" | "delete" => remove(
            args.get(1).ok_or("falta el nombre de la automatización")?,
            ctx,
        ),
        other => Err(format!("operación automation desconocida: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_line, split_arguments, Automation};

    #[test]
    fn parses_arguments_with_quotes() {
        assert_eq!(
            split_arguments("--name \"mi script\" --flag 'dos palabras'").unwrap(),
            ["--name", "mi script", "--flag", "dos palabras"]
        );
    }

    #[test]
    fn parses_registry_lines_without_losing_paths() {
        let entry =
            parse_line("demo\t/home/user/script.sh\t/home/user/project\t--name\u{1f}dos").unwrap();
        assert_eq!(
            entry,
            Automation {
                name: "demo".into(),
                program: "/home/user/script.sh".into(),
                working_directory: Some("/home/user/project".into()),
                args: vec!["--name".into(), "dos".into()]
            }
        );
    }
}
