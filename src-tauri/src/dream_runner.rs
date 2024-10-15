use crate::app_settings::SETTINGS;
use anyhow::{Ok, Result};
/// Displays dreams. Loads settings, creates windows, chooses a dream and runs it.

#[derive(Debug)]
pub struct DreamRunner {}

impl DreamRunner {
    pub fn new() -> Result<Self> {
        let s = Self {};
        Ok(s)
    }

    pub fn initialise(&mut self) -> Result<()> {
        use std::result::Result::Ok; // Block anyhow's Result for the `context` macro
        const FULLSCREEN: bool = true;

        let need_multiscreen = SETTINGS.read().unwrap().attempt_multiscreen;

        tauri::Builder::default()
            .setup(move |app| {
                // Build the primary window
                let primary_window = tauri::WebviewWindowBuilder::new(
                    app,
                    "primary",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .fullscreen(FULLSCREEN)
                .visible(false)
                .build()?;

                if need_multiscreen {
                    let monitors = primary_window.available_monitors()?;
                    let primary_monitor = primary_window.current_monitor()?.unwrap();
                    for (i, monitor) in monitors.iter().enumerate() {
                        if !compare_monitors(&primary_monitor, monitor) {
                            let label = format!("extra{}", i);
                            let pos = calculate_window_position(monitor);
                            //let size = calculate_window_size(monitor);
                            let secondary_window = tauri::WebviewWindowBuilder::new(
                                app,
                                label,
                                tauri::WebviewUrl::App("index.html".into()),
                            )
                            .position(pos.0, pos.1)
                            //.inner_size(size.0, size.1)
                            //.fullscreen(true)
                            .visible(false)
                            .build()?;
                            secondary_window.set_fullscreen(FULLSCREEN)?;
                        }
                    }
                }
                std::result::Result::Ok(())
            })
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_process::init())
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
        Ok(())
    }
}

#[tauri::command]
fn window_finished_loading() {}

fn compare_monitors(a: &tauri::window::Monitor, b: &tauri::window::Monitor) -> bool {
    a.name() == b.name() && a.position() == b.position() && a.size() == b.size()
}

fn calculate_window_position(monitor: &tauri::window::Monitor) -> (f64, f64) {
    let (x, y) = (monitor.position().x as f64, monitor.position().y as f64);
    let scale = monitor.scale_factor();
    (x / scale, y / scale)
}
