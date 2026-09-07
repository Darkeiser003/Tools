//! Menú pequeño que agrupa las operaciones interactivas de software y Git.

use crate::common::Context;
use std::io::{self, Write};

pub fn menu(ctx: &Context) -> Result<(), String> {
    loop {
        crate::clear_screen();
        println!("{}", crate::i18n::tools_text("title"));
        println!("  1) {}", crate::i18n::tools_text("search"));
        println!("  2) {}", crate::i18n::tools_text("install"));
        println!("  3) {}", crate::i18n::tools_text("git_menu"));
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
            "3" => git_menu(ctx),
            _ => {
                println!("{}", crate::i18n::text("menu.invalid"));
                pause();
            }
        }
    }
}

fn git_menu(ctx: &Context) {
    loop {
        crate::clear_screen();
        println!("{}", crate::i18n::tools_text("git_title"));
        println!("  1) {}", crate::i18n::tools_text("git_status"));
        println!("  2) {}", crate::i18n::tools_text("git_log"));
        println!("  3) {}", crate::i18n::tools_text("git_clone"));
        println!("  4) {}", crate::i18n::tools_text("git_fetch"));
        println!("  5) {}", crate::i18n::tools_text("git_pull"));
        println!("  6) {}", crate::i18n::tools_text("git_add"));
        println!("  7) {}", crate::i18n::tools_text("git_commit"));
        println!("  8) {}", crate::i18n::tools_text("git_push"));
        println!("  9) {}", crate::i18n::tools_text("git_branch"));
        println!(" 10) {}", crate::i18n::tools_text("git_tag"));
        println!(" 11) {}", crate::i18n::tools_text("git_release"));
        println!(" 12) {}", crate::i18n::tools_text("git_login"));
        println!(" 13) {}", crate::i18n::tools_text("gh_repo"));
        println!(" 14) {}", crate::i18n::tools_text("gh_prs"));
        println!(" 15) {}", crate::i18n::tools_text("gh_releases"));
        println!(" 16) {}", crate::i18n::tools_text("gh_auth_status"));
        println!("  q) {}", crate::i18n::text("menu.back"));
        let answer = input(crate::i18n::text("menu.prompt"));
        let result = match answer.as_str() {
            "" | "q" | "quit" | "salir" => return,
            "1" => crate::git::run(ctx, &["status".into()]),
            "2" => crate::git::run(ctx, &["log".into()]),
            "3" => run_clone_result(ctx),
            "4" => crate::git::run(ctx, &["fetch".into(), "--prune".into()]),
            "5" => crate::git::run(ctx, &["pull".into(), "--rebase".into()]),
            "6" => crate::git::run(ctx, &["add".into(), "--all".into()]),
            "7" => {
                let message = input("Mensaje del commit (Enter para volver): ");
                if message.is_empty() {
                    continue;
                }
                crate::git::run(ctx, &["commit".into(), "--message".into(), message])
            }
            "8" => crate::git::run(ctx, &["push".into()]),
            "9" => crate::git::run(ctx, &["branch".into(), "--list".into()]),
            "10" => crate::git::run(ctx, &["tag".into(), "--list".into()]),
            "11" => {
                let tag = input("Tag de release (Enter para volver): ");
                if tag.is_empty() {
                    continue;
                }
                crate::git::run(ctx, &["release".into(), "--tag".into(), tag])
            }
            "12" => crate::git::run(ctx, &["login".into()]),
            "13" => crate::git::run(ctx, &["gh".into(), "repo".into()]),
            "14" => crate::git::run(ctx, &["gh".into(), "prs".into()]),
            "15" => crate::git::run(ctx, &["gh".into(), "releases".into()]),
            "16" => crate::git::run(ctx, &["gh".into(), "auth-status".into()]),
            _ => {
                println!("{}", crate::i18n::text("menu.invalid"));
                pause();
                continue;
            }
        };
        show_result(result);
        pause();
    }
}

fn run_clone_result(ctx: &Context) -> Result<(), String> {
    let url = input("URL Git (Enter para volver): ");
    if url.is_empty() {
        return Err("operación cancelada".into());
    }
    let destination = input("Destino (vacío para el nombre automático): ");
    let mut args = vec!["clone".into(), url];
    if !destination.is_empty() {
        args.push(destination);
    }
    crate::git::run(ctx, &args)
}

fn run_and_pause(ctx: &Context, args: &[String]) {
    let result = crate::software::run(ctx, args);
    show_result(result);
    pause();
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
