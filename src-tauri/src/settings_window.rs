use anyhow::Result;

#[derive(Debug)]
pub struct SettingsWindow {}

impl SettingsWindow {
    pub fn new() -> Self {
        SettingsWindow {}
    }

    pub fn launch(&mut self) -> Result<()> {
        const SETTINGS_PAGE: &str = "/src/settings.html";
        use std::result::Result::Ok; // Block anyhow's Result for the `context` macro
        tauri::Builder::default()
            .setup(move |app| {
                // Build the settings window
                let _window = tauri::WebviewWindowBuilder::new(
                    app,
                    "primary",
                    tauri::WebviewUrl::App(SETTINGS_PAGE.into()),
                )
                .visible(true)
                .build()?;
                std::result::Result::Ok(())
            })
            .plugin(tauri_plugin_process::init())
            .run(tauri::generate_context!())
            .expect("error while running tauri application");

        Ok(())
    }
}
