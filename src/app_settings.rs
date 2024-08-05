use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use std::{collections::HashMap, fs::File, path::{Path, PathBuf}};

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
/// Contains all persistant settings of the application
pub struct Settings {
    /// Contains unique settings of particular dreams
    pub dream_settings: HashMap<String, String>,
}

impl Settings {
    pub fn read_from_file_default() -> Result<Self> {
        let path = Self::determine_settings_path()?;
        Self::read_from_file(&path)
    }

    pub fn write_to_file_default(&self) -> Result<()> {
        let path = Self::determine_settings_path()?;
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

        let user_dirs = ProjectDirs::from("goo", "Barafu Albino", "Dream Spinner")
            .ok_or(anyhow!("Can not detect settings directory in user folder"))?;
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
