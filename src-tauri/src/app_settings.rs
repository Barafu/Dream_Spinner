use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use log::{self, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    path::{Path, PathBuf},
    sync::{LazyLock, RwLock},
};

/// The one and only global settings object. Don't lock it for long.
pub static SETTINGS: LazyLock<RwLock<SettingsRaw>> =
    LazyLock::new(|| RwLock::new(SettingsRaw::read_from_file_default().unwrap()));

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default)]
/// Contains all persistant settings of the application
pub struct SettingsRaw {
    /// Contains unique settings of particular dreams
    pub dream_settings: BTreeMap<String, String>,

    /// Try to detect and cover additional monitors.
    pub attempt_multiscreen: bool,

    /// Show FPS statistics on primary screen.
    pub show_fps: bool,

    /// The dreams user selected to display.
    pub selected_dreams: BTreeSet<String>,

    /// Display dreams that are still in development. This setting has
    /// no UI toggle and must be set by editing settings file.
    pub allow_dev_dreams: bool,

    /// Selected color scheme that dreams should use.
    pub color_scheme: ColorScheme,
}

impl Default for SettingsRaw {
    fn default() -> Self {
        Self {
            dream_settings: BTreeMap::new(),
            attempt_multiscreen: false,
            show_fps: false,
            selected_dreams: BTreeSet::from(["fractal_clock".to_string()]),
            allow_dev_dreams: false,
            color_scheme: ColorScheme::default(),
        }
    }
}

impl SettingsRaw {
    /// Read settings from default file location.
    pub fn read_from_file_default() -> Result<Self> {
        let path = Self::determine_settings_path()?;
        log::info!("Reading settings from {}", path.display());
        Self::read_from_file(&path)
    }

    /// Write settings to default file location.
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

    /// Finds the path to the settings file. First tries if dream_settings.toml
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

//============ Color ==============
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub fn new_with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 && hex.len() != 8 {
            return Err("Invalid hex color format".to_string());
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red value")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green value")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue value")?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid alpha value")?
        } else {
            255
        };

        Ok(Color::new_with_alpha(r, g, b, a))
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RGBA({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

//============= ColorScheme ==============

// Define the structure that matches the JSON schema
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ColorScheme {
    pub colors: [Color; 16],
    pub name: String,
    pub foreground: Color,
    pub background: Color,
    pub cursor: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            colors: [Color::new(0, 0, 0); 16],
            name: "None".to_string(),
            foreground: Color::new(200, 200, 200),
            background: Color::new(0, 0, 0),
            cursor: Color::new(0, 200, 200),
        }
    }
}

#[derive(Deserialize, Debug)]
struct ColorSchemeText {
    color_01: String,
    color_02: String,
    color_03: String,
    color_04: String,
    color_05: String,
    color_06: String,
    color_07: String,
    color_08: String,
    color_09: String,
    color_10: String,
    color_11: String,
    color_12: String,
    color_13: String,
    color_14: String,
    color_15: String,
    color_16: String,
    name: String,
    foreground: String,
    background: String,
    cursor: String,
}

impl ColorScheme {
    fn from_color_scheme_text(cs: ColorSchemeText) -> Self {
        Self {
            colors: [
                Color::from_hex(&cs.color_01).unwrap(),
                Color::from_hex(&cs.color_02).unwrap(),
                Color::from_hex(&cs.color_03).unwrap(),
                Color::from_hex(&cs.color_04).unwrap(),
                Color::from_hex(&cs.color_05).unwrap(),
                Color::from_hex(&cs.color_06).unwrap(),
                Color::from_hex(&cs.color_07).unwrap(),
                Color::from_hex(&cs.color_08).unwrap(),
                Color::from_hex(&cs.color_09).unwrap(),
                Color::from_hex(&cs.color_10).unwrap(),
                Color::from_hex(&cs.color_11).unwrap(),
                Color::from_hex(&cs.color_12).unwrap(),
                Color::from_hex(&cs.color_13).unwrap(),
                Color::from_hex(&cs.color_14).unwrap(),
                Color::from_hex(&cs.color_15).unwrap(),
                Color::from_hex(&cs.color_16).unwrap(),
            ],
            name: cs.name,
            foreground: Color::from_hex(&cs.foreground).unwrap(),
            background: Color::from_hex(&cs.background).unwrap(),
            cursor: Color::from_hex(&cs.cursor).unwrap(),
        }
    }
    pub fn read_default_schemes() -> BTreeMap<String, ColorScheme> {
        let color_scheme_json = include_str!("../assets/color_scheme_data.json");
        let color_schemes: Vec<ColorSchemeText> = serde_json::from_str(&color_scheme_json).unwrap();
        let mut color_schemes_map: BTreeMap<String, ColorScheme> = BTreeMap::new();
        for cs_t in color_schemes.into_iter() {
            let color_scheme = ColorScheme::from_color_scheme_text(cs_t);
            color_schemes_map.insert(color_scheme.name.clone(), color_scheme);
        }
        info!("Read {} color schemes", color_schemes_map.len());
        color_schemes_map
    }
}
