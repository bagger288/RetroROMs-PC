use crate::models::GameCard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    Catalog = 0,
    Downloads = 1,
    Favorites = 2,
    Settings = 3,
}

impl NavTab {
    pub fn from_idx(idx: i32) -> Self {
        match idx {
            1 => NavTab::Downloads,
            2 => NavTab::Favorites,
            3 => NavTab::Settings,
            _ => NavTab::Catalog,
        }
    }

    pub fn to_idx(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid = 0,
    Table = 1,
}

impl ViewMode {
    pub fn from_idx(idx: i32) -> Self {
        match idx {
            1 => ViewMode::Table,
            _ => ViewMode::Grid,
        }
    }

    pub fn to_idx(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    Default = 0,
    RatingDesc = 1,
    TitleAsc = 2,
    TitleDesc = 3,
}

impl SortOption {
    pub fn from_idx(idx: i32) -> Self {
        match idx {
            1 => SortOption::RatingDesc,
            2 => SortOption::TitleAsc,
            3 => SortOption::TitleDesc,
            _ => SortOption::Default,
        }
    }

    pub fn to_idx(&self) -> i32 {
        *self as i32
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortOption::Default => "По умолчанию",
            SortOption::RatingDesc => "★ По рейтингу",
            SortOption::TitleAsc => "А-Я По названию",
            SortOption::TitleDesc => "Я-А По названию",
        }
    }
}

pub fn get_console_badge_info(slug: &str) -> (&'static str, &'static str) {
    match slug {
        "dendy" => ("NES", "/ Денди / Famicom"),
        "genesis" => ("GENESIS", "/ MegaDrive"),
        "snes" => ("SNES", "/ Super Famicom"),
        "psx" => ("PS1", "/ PlayStation"),
        "n64" => ("N64", "/ Nintendo 64"),
        "gba" => ("GBA", "/ Game Boy Advance"),
        "gb" => ("GB", "/ Game Boy"),
        "gbc" => ("GBC", "/ Game Boy Color"),
        "32x" => ("32X", "/ Mega Drive 32X"),
        "segacd" => ("SEGA CD", "/ Mega-CD"),
        "sms" => ("SMS", "/ Master System"),
        "sg-1000" => ("SG-1000", "/ Sega SG-1000"),
        "pce" => ("PCE", "/ TurboGrafx-16"),
        "pcecd" => ("PCE CD", "/ TurboGrafx-CD"),
        "3do" => ("3DO", "/ 3DO Interactive"),
        "famicom_disk_system" => ("FDS", "/ Famicom Disk"),
        "neogeocd" => ("NEOGEO", "/ Neo Geo CD"),
        "jaguar" => ("JAGUAR", "/ Atari Jaguar"),
        "gg" => ("GAME GEAR", "/ Sega Game Gear"),
        "lynx" => ("LYNX", "/ Atari Lynx"),
        "ngp" => ("NGP", "/ Neo Geo Pocket"),
        "ws" => ("WS", "/ WonderSwan"),
        "vboy" => ("VIRTUAL BOY", "/ Nintendo"),
        "pmini" => ("POKEMINI", "/ Pokémon Mini"),
        "sv" => ("SV", "/ Supervision"),
        "2600" => ("ATARI 2600", "/ Atari"),
        "5200" => ("ATARI 5200", "/ Atari"),
        "7800" => ("ATARI 7800", "/ Atari"),
        "coleco" => ("COLECO", "/ ColecoVision"),
        "vectrex" => ("VECTREX", "/ Vectrex"),
        "intellivision" => ("INTV", "/ Intellivision"),
        "arcadia" => ("ARCADIA", "/ Arcadia 2001"),
        "chaf" => ("CHANNEL F", "/ Fairchild"),
        _ => ("ROM", ""),
    }
}

pub fn get_format_label(slug: &str) -> &'static str {
    match slug {
        "psx" | "3do" | "segacd" | "pcecd" | "neogeocd" => "ISO",
        _ => "ROM",
    }
}

pub fn get_region_label(game: &GameCard) -> String {
    if !game.regions.is_empty() {
        game.regions.join("/")
    } else if game.title.to_lowercase().contains("rus") || game.title.contains("Перевод") {
        "RUS".to_string()
    } else if game.title.to_lowercase().contains("(e)") || game.title.contains("(eur") {
        "EUR".to_string()
    } else if game.title.to_lowercase().contains("(j)") || game.title.contains("(jap") {
        "JAP".to_string()
    } else {
        "ROM".to_string()
    }
}
