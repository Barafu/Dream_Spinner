// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_settings;
mod dream_runner;
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
    let mut dr = dream_runner::DreamRunner::new()?;
    dr.initialise()?;
    Ok(())
}

fn show_config() -> Result<()> {
    todo!();
}

fn show_preview(_handle: usize) -> Result<()> {
    Ok(())
}
