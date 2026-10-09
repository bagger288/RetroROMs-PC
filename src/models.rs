use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadStatus::Pending => "В очереди",
            DownloadStatus::Downloading => "Загрузка",
            DownloadStatus::Completed => "Готово",
            DownloadStatus::Failed => "Ошибка",
            DownloadStatus::Cancelled => "Отменено",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleInfo {
    pub slug: String,
    pub name: String,
    pub short_name: String,
    pub category: String, // "home", "portable", "classic"
    pub folder_name: String,
    pub section: String, // "consoles" or "portable"
    pub is_enabled: bool,
    pub order: i32,
    pub release_year: String,
    pub icon_key: String,
    pub roms_count_estimate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogCategory {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCard {
    pub id: String,
    pub console_slug: String,
    pub console_name: String,
    pub section: String,
    pub title: String,
    pub original_title: Option<String>,
    pub genre: String,
    pub year: String,
    pub publisher: String,
    pub developer: String,
    pub rating: f32,
    pub file_size: String,
    pub cover_url: Option<String>,
    pub screenshot_urls: Vec<String>,
    pub description: String,
    pub download_url: String,
    pub mfile_id: Option<String>,
    pub game_page_slug: Option<String>,
    pub regions: Vec<String>,
    pub is_favorite: bool,
    pub is_downloaded: bool,
    pub local_file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomFileVersion {
    pub fid: String,
    pub name: String,
    pub size: String,
    pub category: String, // "Основные", "Переводы", "Хаки", "GoodSet"
    pub download_url: String,
    pub region_or_type: String, // "RUS", "USA", "EUR", "JAP", "HACK", "GOODSET"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipRomEntry {
    pub entry_name: String,
    pub display_name: String,
    pub size_bytes: u64,
    pub formatted_size: String,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: i64,
    pub game_id: String,
    pub game_title: String,
    pub console_slug: String,
    pub console_name: String,
    pub file_name: String,
    pub target_directory: String,
    pub local_path: Option<String>,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bytes_sec: u64,
    pub status: DownloadStatus,
    pub download_url: String,
    pub timestamp: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamesPageResult {
    pub games: Vec<GameCard>,
    pub current_page: usize,
    pub total_pages: usize,
    pub has_next_page: bool,
    pub available_categories: Vec<CatalogCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipExtractionRequest {
    pub game: GameCard,
    pub temp_zip_path: String,
    pub console_folder_name: String,
    pub target_directory: String,
    pub record_id: i64,
    pub entries: Vec<ZipRomEntry>,
}

/// The 33 consoles supported by Emu-Land with downloadable games
pub fn get_default_consoles() -> Vec<ConsoleInfo> {
    let raw = vec![
        // Домашние консоли
        ("dendy", "NES / Dendy / Famicom", "NES", "home", "NES", "consoles", "1983", "🎮", "~2,500"),
        ("genesis", "Sega Mega Drive / Genesis", "Genesis", "home", "MegaDrive", "consoles", "1988", "🎮", "~1,900"),
        ("snes", "Super Nintendo (SNES)", "SNES", "home", "SNES", "consoles", "1990", "🎮", "~3,400"),
        ("psx", "Sony PlayStation 1", "PS1", "home", "PS1", "consoles", "1994", "💿", "~1,800"),
        ("n64", "Nintendo 64", "N64", "home", "N64", "consoles", "1996", "🎮", "~390"),
        ("32x", "Sega 32X", "32X", "home", "Sega32X", "consoles", "1994", "🎮", "~40"),
        ("segacd", "Sega CD / Mega CD", "Sega CD", "home", "SegaCD", "consoles", "1991", "💿", "~210"),
        ("sms", "Sega Master System", "SMS", "home", "SMS", "consoles", "1985", "🎮", "~540"),
        ("sg-1000", "Sega SG-1000", "SG-1000", "home", "SG1000", "consoles", "1983", "🎮", "~80"),
        ("pce", "PC Engine / TurboGrafx-16", "PCE", "home", "PCEngine", "consoles", "1987", "🎮", "~650"),
        ("pcecd", "PC Engine CD", "PCE CD", "home", "PCEngineCD", "consoles", "1988", "💿", "~420"),
        ("3do", "3DO Interactive Multiplayer", "3DO", "home", "3DO", "consoles", "1993", "💿", "~280"),
        ("famicom_disk_system", "Famicom Disk System", "FDS", "home", "FDS", "consoles", "1986", "💾", "~200"),
        ("neogeocd", "Neo Geo CD", "NeoGeo CD", "home", "NeoGeoCD", "consoles", "1994", "💿", "~100"),
        ("jaguar", "Atari Jaguar", "Jaguar", "home", "Jaguar", "consoles", "1993", "🎮", "~60"),

        // Портативные системы
        ("gba", "Game Boy Advance", "GBA", "portable", "GBA", "portable", "2001", "📱", "~2,800"),
        ("gb", "Game Boy", "GB", "portable", "GameBoy", "portable", "1989", "📱", "~1,600"),
        ("gbc", "Game Boy Color", "GBC", "portable", "GBC", "portable", "1998", "📱", "~1,400"),
        ("gg", "Sega Game Gear", "Game Gear", "portable", "GameGear", "portable", "1990", "📱", "~390"),
        ("lynx", "Atari Lynx", "Lynx", "portable", "Lynx", "portable", "1989", "📱", "~75"),
        ("ngp", "Neo Geo Pocket", "NGP", "portable", "NGP", "portable", "1998", "📱", "~80"),
        ("ws", "Bandai WonderSwan", "WonderSwan", "portable", "WonderSwan", "portable", "1999", "📱", "~200"),
        ("vboy", "Nintendo Virtual Boy", "Virtual Boy", "portable", "VirtualBoy", "portable", "1995", "🥽", "~22"),
        ("pmini", "Pokémon Mini", "PokeMini", "portable", "PokeMini", "portable", "2001", "📱", "~10"),
        ("sv", "Watara Supervision", "Supervision", "portable", "Supervision", "portable", "1992", "📱", "~65"),

        // Классические системы
        ("2600", "Atari 2600", "Atari 2600", "classic", "Atari2600", "consoles", "1977", "🕹", "~500"),
        ("5200", "Atari 5200", "Atari 5200", "classic", "Atari5200", "consoles", "1982", "🕹", "~100"),
        ("7800", "Atari 7800", "Atari 7800", "classic", "Atari7800", "consoles", "1986", "🕹", "~80"),
        ("coleco", "ColecoVision", "Coleco", "classic", "ColecoVision", "consoles", "1982", "🕹", "~140"),
        ("vectrex", "Vectrex", "Vectrex", "classic", "Vectrex", "consoles", "1982", "📺", "~30"),
        ("intellivision", "Intellivision", "Intellivision", "classic", "Intellivision", "consoles", "1979", "🕹", "~125"),
        ("arcadia", "Emerson Arcadia 2001", "Arcadia", "classic", "Arcadia", "consoles", "1982", "🕹", "~35"),
        ("chaf", "Fairchild Channel F", "Channel F", "classic", "ChannelF", "consoles", "1976", "🕹", "~30"),
    ];

    raw.into_iter()
        .enumerate()
        .map(|(i, (slug, name, short_name, category, folder_name, section, year, icon, count))| {
            ConsoleInfo {
                slug: slug.to_string(),
                name: name.to_string(),
                short_name: short_name.to_string(),
                category: category.to_string(),
                folder_name: folder_name.to_string(),
                section: section.to_string(),
                is_enabled: true,
                order: i as i32,
                release_year: year.to_string(),
                icon_key: icon.to_string(),
                roms_count_estimate: count.to_string(),
            }
        })
        .collect()
}
