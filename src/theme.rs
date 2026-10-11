use slint::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    ArcadeNeon,     // Default theme matching the screenshot
    NintendoRetro,  // Nintendo Retro
    SegaMega,       // Sega Mega
    Retrowave,      // Synthwave 80s
    OledBlack,      // OLED Pure Black
    GameBoy,        // Game Boy Lime
    ClassicDark,    // Classic Dark
    LightClassic,   // Light Classic
    Cyberpunk,      // Cyberpunk
}

impl ThemePreset {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Cyberpunk" | "Киберпанк" => ThemePreset::Cyberpunk,
            "LightClassic" | "Светлая классика" | "Light" => ThemePreset::LightClassic,
            "NintendoRetro" | "Nintendo Retro" => ThemePreset::NintendoRetro,
            "SegaMega" | "Sega Mega" => ThemePreset::SegaMega,
            "Retrowave" | "Synthwave 80s" => ThemePreset::Retrowave,
            "ClassicDark" | "Темная классика" => ThemePreset::ClassicDark,
            "GameBoy" | "Game Boy Lime" => ThemePreset::GameBoy,
            "OledBlack" | "OLED Pure Black" => ThemePreset::OledBlack,
            _ => ThemePreset::ArcadeNeon,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemePreset::ArcadeNeon => "ArcadeNeon",
            ThemePreset::Cyberpunk => "Cyberpunk",
            ThemePreset::Retrowave => "Retrowave",
            ThemePreset::ClassicDark => "ClassicDark",
            ThemePreset::LightClassic => "LightClassic",
            ThemePreset::NintendoRetro => "NintendoRetro",
            ThemePreset::SegaMega => "SegaMega",
            ThemePreset::GameBoy => "GameBoy",
            ThemePreset::OledBlack => "OledBlack",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ThemePreset::ArcadeNeon => "🎮 RetroROMs Blue (Default)",
            ThemePreset::NintendoRetro => "🎮 Nintendo Retro",
            ThemePreset::SegaMega => "🌀 Sega Mega",
            ThemePreset::Retrowave => "🌴 Synthwave 80s",
            ThemePreset::OledBlack => "🖤 OLED Pure Black",
            ThemePreset::GameBoy => "👾 Game Boy Lime",
            ThemePreset::ClassicDark => "🌑 Темная классика",
            ThemePreset::LightClassic => "☀️ Светлая классика",
            ThemePreset::Cyberpunk => "⚡ Cyberpunk 2077",
        }
    }

    pub fn is_light(&self) -> bool {
        matches!(self, ThemePreset::LightClassic)
    }

    /// Primary accent color: Vibrant Arcade Blue (#0078D7)
    pub fn primary_color(&self) -> Color {
        match self {
            ThemePreset::ArcadeNeon => Color::from_argb_u8(255, 0, 120, 215),    // #0078D7
            ThemePreset::NintendoRetro => Color::from_argb_u8(255, 230, 0, 18),   // #E60012
            ThemePreset::SegaMega => Color::from_argb_u8(255, 0, 137, 207),       // #0089CF
            ThemePreset::Retrowave => Color::from_argb_u8(255, 192, 38, 211),     // #C026D3
            ThemePreset::OledBlack => Color::from_argb_u8(255, 0, 230, 118),      // #00E676
            ThemePreset::GameBoy => Color::from_argb_u8(255, 139, 172, 15),       // #8BAC0F
            ThemePreset::ClassicDark => Color::from_argb_u8(255, 100, 181, 246),  // Soft Blue
            ThemePreset::LightClassic => Color::from_argb_u8(255, 2, 132, 199),   // #0284C7
            ThemePreset::Cyberpunk => Color::from_argb_u8(255, 255, 230, 0),      // Neon Yellow
        }
    }

    /// Secondary accent color: Golden Amber (#FFB300) for rating stars
    pub fn secondary_color(&self) -> Color {
        match self {
            ThemePreset::ArcadeNeon => Color::from_argb_u8(255, 255, 179, 0),     // #FFB300
            ThemePreset::NintendoRetro => Color::from_argb_u8(255, 255, 204, 0),  // #FFCC00
            ThemePreset::SegaMega => Color::from_argb_u8(255, 255, 160, 0),       // #FFA000
            ThemePreset::Retrowave => Color::from_argb_u8(255, 6, 182, 212),      // #06B6D4
            ThemePreset::OledBlack => Color::from_argb_u8(255, 255, 179, 0),      // #FFB300
            ThemePreset::GameBoy => Color::from_argb_u8(255, 155, 188, 15),       // #9BBC0F
            ThemePreset::ClassicDark => Color::from_argb_u8(255, 176, 190, 197),  // Silver Gray
            ThemePreset::LightClassic => Color::from_argb_u8(255, 217, 119, 6),   // #D97706
            ThemePreset::Cyberpunk => Color::from_argb_u8(255, 0, 255, 204),      // Cyan Teal
        }
    }

    /// Background color
    pub fn window_bg_color(&self) -> Color {
        match self {
            ThemePreset::OledBlack => Color::from_argb_u8(255, 0, 0, 0),
            ThemePreset::LightClassic => Color::from_argb_u8(255, 245, 247, 250),
            ThemePreset::GameBoy => Color::from_argb_u8(255, 15, 22, 15),
            ThemePreset::ClassicDark => Color::from_argb_u8(255, 18, 18, 18),
            ThemePreset::Retrowave => Color::from_argb_u8(255, 15, 10, 25),
            _ => Color::from_argb_u8(255, 9, 14, 23), // #090e17
        }
    }

    pub fn card_bg_color(&self) -> Color {
        match self {
            ThemePreset::OledBlack => Color::from_argb_u8(255, 8, 8, 8),
            ThemePreset::LightClassic => Color::from_argb_u8(255, 255, 255, 255),
            _ => Color::from_argb_u8(255, 17, 24, 39), // #111827
        }
    }

    pub fn card_border_color(&self) -> Color {
        match self {
            ThemePreset::OledBlack => Color::from_argb_u8(255, 20, 20, 20),
            ThemePreset::LightClassic => Color::from_argb_u8(255, 220, 225, 235),
            _ => Color::from_argb_u8(255, 28, 40, 60), // #1c283c
        }
    }
}
