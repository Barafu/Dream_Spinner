use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use log;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    sync::{LazyLock, RwLock},
};

pub static SETTINGS: LazyLock<RwLock<SettingsRaw>> = LazyLock::new(|| {
    RwLock::new(SettingsRaw::read_from_file_default().unwrap())
});

use crate::dreams::DreamId;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
/// Contains all persistant settings of the application
pub struct SettingsRaw {
    /// Contains unique settings of particular dreams
    pub dream_settings: HashMap<String, String>,

    /// Try to detect and cover additional monitors.
    pub attempt_multiscreen: bool,

    /// Show FPS statistics on primary screen.
    pub show_fps: bool,

    pub selected_dreams: HashSet<DreamId>,

    pub viewport_mode: ViewportMode,
}

impl Default for SettingsRaw {
    fn default() -> Self {
        Self {
            dream_settings: HashMap::new(),
            attempt_multiscreen: false,
            show_fps: false,
            selected_dreams: HashSet::from(["fractal_clock".to_string()]),
            viewport_mode: ViewportMode::Immediate,
        }
    }
}

impl SettingsRaw {
    pub fn read_from_file_default() -> Result<Self> {
        let path = Self::determine_settings_path()?;
        log::info!("Reading settings from {}", path.display());
        Self::read_from_file(&path)
    }

    pub fn write_to_file_default(&self) -> Result<()> {
        let path = Self::determine_settings_path()?;
        log::info!("Writing settings to {}", path.display());
        self.write_to_file(&path)
    }

    fn read_from_file(path: &Path) -> Result<Self> {
        let toml = std::fs::read_to_string(path)?;
        let s: Self = toml::from_str(&toml)?;
        Ok(s)
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        let toml = toml::to_string(&self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    /// Finds the path to the settrings file. First tries if dream_settings.toml
    /// exists in the same directory as the executable. If not, tries if it exists
    /// in the user's settings directory. If not, creates it in the user
    ///  settings directory. If creation fails, returns error.
    fn determine_settings_path() -> Result<PathBuf> {
        const SETTINGS_FILE_NAME: &str = "dream_settings.toml";
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        let settings_file = exe_dir.join(SETTINGS_FILE_NAME);
        if settings_file.is_file() {
            return Ok(settings_file);
        }

        let user_dirs =
            ProjectDirs::from("goo", "Barafu Albino", "Dream Spinner").ok_or(
                anyhow!("Can not detect settings directory in user folder"),
            )?;
        let settings_dir = user_dirs.config_dir();
        let settings_file = settings_dir.join(SETTINGS_FILE_NAME);
        if settings_file.is_file() {
            return Ok(settings_file);
        }
        std::fs::create_dir_all(&settings_dir)?;
        File::create(&settings_file)?;
        return Ok(settings_file);
    }
}

#[derive(
    PartialEq, Debug, serde::Deserialize, serde::Serialize, Default, Clone, Copy,
)]
pub enum ViewportMode {
    #[default]
    Immediate,
    Deferred,
}

impl Display for ViewportMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewportMode::Immediate => write!(f, "Immediate"),
            ViewportMode::Deferred => write!(f, "Deferred"),
        }
    }
}
