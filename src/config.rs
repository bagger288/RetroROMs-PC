use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_directory: String,
    pub auto_unpack_zip: bool,
    pub delete_zip_after_unpack: bool,
    pub active_theme: String,
    pub view_mode: String,
    pub emulator_paths: HashMap<String, String>,
    pub emulator_args: HashMap<String, String>,
    pub use_retroarch: bool,
    pub retroarch_path: String,
    pub retroarch_args: String,
    pub retroarch_cores: HashMap<String, String>,
    pub last_console_slug: Option<String>,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
}

fn default_ui_scale() -> f32 {
    1.0
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = if let Some(user_downloads) = dirs::download_dir() {
            user_downloads.join("RetroROMs").to_string_lossy().to_string()
        } else {
            "RetroROMs_Downloads".to_string()
        };

        Self {
            download_directory: download_dir,
            auto_unpack_zip: true,
            delete_zip_after_unpack: false,
            active_theme: "ArcadeNeon".to_string(),
            view_mode: "Table".to_string(),
            emulator_paths: HashMap::new(),
            emulator_args: HashMap::new(),
            use_retroarch: false,
            retroarch_path: String::new(),
            retroarch_args: String::new(),
            retroarch_cores: get_default_retroarch_cores(),
            last_console_slug: Some("dendy".to_string()),
            ui_scale: 1.0,
        }
    }
}

impl AppSettings {
    fn get_config_file_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("RetroROMs");
            let _ = fs::create_dir_all(&app_dir);
            app_dir.join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::get_config_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut settings) = serde_json::from_str::<AppSettings>(&content) {
                    if settings.retroarch_cores.is_empty() {
                        settings.retroarch_cores = get_default_retroarch_cores();
                    }
                    if settings.ui_scale < 0.5 || settings.ui_scale > 3.0 {
                        settings.ui_scale = 1.0;
                    }
                    return settings;
                }
            }
        }
        let default_settings = Self::default();
        let _ = default_settings.save();
        default_settings
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = Self::get_config_file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn get_retroarch_core_for_console(&self, slug: &str) -> String {
        if let Some(custom_core) = self.retroarch_cores.get(slug) {
            if !custom_core.trim().is_empty() {
                return custom_core.clone();
            }
        }
        get_default_retroarch_cores()
            .get(slug)
            .cloned()
            .unwrap_or_else(|| "core_libretro.dll".to_string())
    }
}

pub fn get_default_retroarch_cores() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("dendy".to_string(), "fceumm_libretro.dll".to_string());
    map.insert("genesis".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    map.insert("snes".to_string(), "snes9x_libretro.dll".to_string());
    map.insert("psx".to_string(), "mednafen_psx_hw_libretro.dll".to_string());
    map.insert("n64".to_string(), "mupen64plus_next_libretro.dll".to_string());
    map.insert("gba".to_string(), "mgba_libretro.dll".to_string());
    map.insert("gb".to_string(), "gambatte_libretro.dll".to_string());
    map.insert("gbc".to_string(), "gambatte_libretro.dll".to_string());
    map.insert("32x".to_string(), "picodrive_libretro.dll".to_string());
    map.insert("segacd".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    map.insert("sms".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    map.insert("sg-1000".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    map.insert("pce".to_string(), "mednafen_pce_fast_libretro.dll".to_string());
    map.insert("pcecd".to_string(), "mednafen_pce_fast_libretro.dll".to_string());
    map.insert("3do".to_string(), "opera_libretro.dll".to_string());
    map.insert("famicom_disk_system".to_string(), "fceumm_libretro.dll".to_string());
    map.insert("neogeocd".to_string(), "neocd_libretro.dll".to_string());
    map.insert("gg".to_string(), "genesis_plus_gx_libretro.dll".to_string());
    map.insert("lynx".to_string(), "handy_libretro.dll".to_string());
    map.insert("ngp".to_string(), "mednafen_ngp_libretro.dll".to_string());
    map.insert("ws".to_string(), "mednafen_wswan_libretro.dll".to_string());
    map.insert("vboy".to_string(), "mednafen_vb_libretro.dll".to_string());
    map.insert("pmini".to_string(), "pokeem_libretro.dll".to_string());
    map.insert("sv".to_string(), "watara_libretro.dll".to_string());
    map.insert("2600".to_string(), "stella_libretro.dll".to_string());
    map.insert("5200".to_string(), "a5200_libretro.dll".to_string());
    map.insert("7800".to_string(), "prosystem_libretro.dll".to_string());
    map.insert("coleco".to_string(), "gearcoleco_libretro.dll".to_string());
    map.insert("vectrex".to_string(), "vecx_libretro.dll".to_string());
    map.insert("intellivision".to_string(), "freeintv_libretro.dll".to_string());
    map.insert("arcadia".to_string(), "mame_libretro.dll".to_string());
    map.insert("chaf".to_string(), "mame_libretro.dll".to_string());
    map.insert("jaguar".to_string(), "virtualjaguar_libretro.dll".to_string());
    map
}
