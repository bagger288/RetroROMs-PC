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
    #[serde(default)]
    pub use_retroarch: bool,
    #[serde(default)]
    pub retroarch_path: String,
    #[serde(default)]
    pub retroarch_args: String,
    #[serde(default)]
    pub retroarch_cores: HashMap<String, String>,
}

pub fn get_default_retroarch_cores() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("dendy".to_string(), "fceumm_libretro.dll".to_string());
    m.insert("genesis".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    m.insert("snes".to_string(), "snes9x_libretro.dll".to_string());
    m.insert("psx".to_string(), "swanstation_libretro.dll".to_string());
    m.insert("gba".to_string(), "mgba_libretro.dll".to_string());
    m.insert("gb".to_string(), "gambatte_libretro.dll".to_string());
    m.insert("gbc".to_string(), "gambatte_libretro.dll".to_string());
    m.insert("n64".to_string(), "mupen64plus_next_libretro.dll".to_string());
    m.insert("nds".to_string(), "melonds_libretro.dll".to_string());
    m.insert("sms".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    m.insert("gamegear".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    m.insert("sega_cd".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    m.insert("sega_32x".to_string(), "picodrive_libretro.dll".to_string());
    m.insert("pce".to_string(), "mednafen_pce_fast_libretro.dll".to_string());
    m.insert("pcfx".to_string(), "mednafen_pcfx_libretro.dll".to_string());
    m.insert("2600".to_string(), "stella_libretro.dll".to_string());
    m.insert("5200".to_string(), "a5200_libretro.dll".to_string());
    m.insert("7800".to_string(), "prosystem_libretro.dll".to_string());
    m.insert("lynx".to_string(), "handy_libretro.dll".to_string());
    m.insert("wonderswan".to_string(), "mednafen_wswan_libretro.dll".to_string());
    m.insert("ngp".to_string(), "mednafen_ngp_libretro.dll".to_string());
    m.insert("arcade".to_string(), "fbneo_libretro.dll".to_string());
    m.insert("neogeo".to_string(), "fbneo_libretro.dll".to_string());
    m.insert("cps1".to_string(), "fbneo_libretro.dll".to_string());
    m.insert("cps2".to_string(), "fbneo_libretro.dll".to_string());
    m.insert("cps3".to_string(), "fbneo_libretro.dll".to_string());
    m.insert("saturn".to_string(), "beetle_saturn_libretro.dll".to_string());
    m.insert("dreamcast".to_string(), "flycast_libretro.dll".to_string());
    m.insert("psp".to_string(), "ppsspp_libretro.dll".to_string());
    m.insert("3do".to_string(), "opera_libretro.dll".to_string());
    m.insert("coleco".to_string(), "gearcoleco_libretro.dll".to_string());
    m.insert("vectrex".to_string(), "vecx_libretro.dll".to_string());
    m.insert("intellivision".to_string(), "freeintv_libretro.dll".to_string());
    m
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
            use_retroarch: false,
            retroarch_path: String::new(),
            retroarch_args: String::new(),
            retroarch_cores: get_default_retroarch_cores(),
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

    pub fn get_retroarch_core_for_console(&self, console_slug: &str) -> Option<&str> {
        self.retroarch_cores.get(console_slug).map(|s| s.as_str())
    }
}
