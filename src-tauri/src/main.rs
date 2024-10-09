// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_settings;
mod parse_cli;

use anyhow::{Context, Ok, Result};

use app_settings::SETTINGS;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// #[tauri::command]
// fn greet(name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

fn main() -> Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let args: Vec<String> = std::env::args().collect();
    let parsed = parse_cli::parse_args(&args)?;

    match parsed.command {
        parse_cli::MainCommand::Show => show_dream()?,
        parse_cli::MainCommand::Config => show_config()?,
        parse_cli::MainCommand::Preview => {
            show_preview(parsed.handle.unwrap())?
        }
    }

    Ok(())
}

fn show_dream() -> Result<()> {
    use std::result::Result::Ok; // Block anyhow's Result for the `context` macro
    let tauri_context = tauri::generate_context!();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri_context)
        .context("error while running tauri application");
    Ok(())
}

fn show_config() -> Result<()> {
    todo!();
}

fn show_preview(_handle: usize) -> Result<()> {
    Ok(())
}
