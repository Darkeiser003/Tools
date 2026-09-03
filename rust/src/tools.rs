//! Menú pequeño que agrupa las operaciones interactivas de software y Git.

use crate::common::Context;
use std::io::{self, Write};

pub fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("{}", crate::i18n::tools_text("title"));
        println!("  1) {}", crate::i18n::tools_text("search"));
        println!("  2) {}", crate::i18n::tools_text("install"));
        println!("  3) {}", crate::i18n::tools_text("git_status"));
        println!("  4) {}", crate::i18n::tools_text("git_clone"));
        println!("  5) {}", crate::i18n::tools_text("git_fetch"));
        println!("  6) {}", crate::i18n::tools_text("git_pull"));
        println!("  7) {}", crate::i18n::tools_text("git_login"));
        println!("  q) {}", crate::i18n::text("menu.back"));
        print!("{}", crate::i18n::text("menu.prompt"));
        let _ = io::stdout().flush();
        let mut answer = String::new();
        if io::stdin().read_line(&mut answer).is_err() {
            return Ok(());
        }
        match answer.trim().to_lowercase().as_str() {
            "" | "q" | "quit" | "salir" => return Ok(()),
            "1" => run_and_pause(ctx, &["search".into()]),
            "2" => run_and_pause(ctx, &["install".into()]),
            "3" => run_and_pause_git(ctx, &["status".into()]),
            "4" => run_clone(ctx),
            "5" => run_and_pause_git(ctx, &["fetch".into()]),
            "6" => run_and_pause_git(ctx, &["pull".into()]),
            "7" => run_and_pause_git(ctx, &["login".into()]),
            _ => {
                println!("{}", crate::i18n::text("menu.invalid"));
                pause();
            }
        }
    }
}

fn run_and_pause(ctx: &Context, args: &[String]) {
    let result = crate::software::run(ctx, args);
    show_result(result);
    pause();
}
fn run_and_pause_git(ctx: &Context, args: &[String]) {
    let result = crate::git::run(ctx, args);
    show_result(result);
    pause();
}
fn run_clone(ctx: &Context) {
    let url = input("URL Git (Enter para volver): ");
    if url.is_empty() {
        return;
    }
    let destination = input("Destino (vacío para el nombre automático): ");
    let mut args = vec!["clone".into(), url];
    if !destination.is_empty() {
        args.push(destination);
    }
    run_and_pause_git(ctx, &args);
}
fn input(prompt: &str) -> String {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut value = String::new();
    if io::stdin().read_line(&mut value).is_ok() {
        value.trim().to_string()
    } else {
        String::new()
    }
}
fn show_result(result: Result<(), String>) {
    if let Err(error) = result {
        eprintln!("Error: {error}");
    } else {
        println!("{}", crate::i18n::tools_text("done"));
    }
}
fn pause() {
    print!("{}", crate::i18n::tools_text("pause"));
    let _ = io::stdout().flush();
    let mut value = String::new();
    let _ = io::stdin().read_line(&mut value);
}
