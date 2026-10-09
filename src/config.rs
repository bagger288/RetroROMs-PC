use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_directory: String,
    pub auto_unpack_zip: bool,
    pub delete_zip_after_unpack: bool,
    pub active_theme: String,
    pub view_mode: String, // "Grid" or "Table"
    pub emulator_paths: HashMap<String, String>,
    pub emulator_args: HashMap<String, String>,
    pub show_only_enabled_consoles: bool,
    #[serde(default)]
    pub last_console_slug: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let default_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("RetroROMs")
            .to_string_lossy()
            .to_string();

        let mut default_emus = HashMap::new();
        default_emus.insert("dendy".to_string(), "mesen".to_string());
        default_emus.insert("genesis".to_string(), "fusion".to_string());
        default_emus.insert("snes".to_string(), "snes9x".to_string());
        default_emus.insert("psx".to_string(), "duckstation".to_string());
        default_emus.insert("gba".to_string(), "mgba".to_string());

        Self {
            download_directory: default_dir,
            auto_unpack_zip: true,
            delete_zip_after_unpack: true,
            active_theme: "ArcadeNeon".to_string(),
            view_mode: "Grid".to_string(),
            emulator_paths: default_emus,
            emulator_args: HashMap::new(),
            show_only_enabled_consoles: true,
            last_console_slug: Some("dendy".to_string()),
        }
    }
}

impl AppSettings {
    pub fn config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("RetroROMs");
            let _ = fs::create_dir_all(&app_dir);
            app_dir.join("config.json")
        } else {
            PathBuf::from("retroms_config.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    return settings;
                }
            }
        }
        let settings = Self::default();
        let _ = settings.save();
        settings
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_console_folder(&self, console_folder_name: &str) -> PathBuf {
        Path::new(&self.download_directory).join(console_folder_name)
    }
}
