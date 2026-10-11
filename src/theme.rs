use egui::{Color32, Stroke, Visuals};

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
            "Cyberpunk" | "Киберпанк" => ThemePreset::ArcadeNeon,
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

    /// Primary accent color: Vibrant Arcade Blue (#0078D4 / #0084FF) matching the screenshot
    pub fn primary_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(0, 126, 237),    // #007EED Vibrant Blue
            ThemePreset::NintendoRetro => Color32::from_rgb(230, 0, 18),  // #E60012 Nintendo Red
            ThemePreset::SegaMega => Color32::from_rgb(0, 137, 207),      // #0089CF Sega Blue
            ThemePreset::Retrowave => Color32::from_rgb(192, 38, 211),    // #C026D3 Synthwave Magenta
            ThemePreset::OledBlack => Color32::from_rgb(0, 230, 118),     // #00E676 Emerald Green
            ThemePreset::GameBoy => Color32::from_rgb(139, 172, 15),      // #8BAC0F GameBoy Olive
            ThemePreset::ClassicDark => Color32::from_rgb(100, 181, 246), // Soft Blue
            ThemePreset::LightClassic => Color32::from_rgb(2, 132, 199),  // #0284C7 Sky Blue
            ThemePreset::Cyberpunk => Color32::from_rgb(255, 230, 0),     // Neon Yellow
        }
    }

    /// Secondary accent color: Golden Amber (#FFB300) for rating stars
    pub fn secondary_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(255, 179, 0),    // #FFB300 Rating Gold
            ThemePreset::NintendoRetro => Color32::from_rgb(255, 204, 0), // #FFCC00 Gold
            ThemePreset::SegaMega => Color32::from_rgb(255, 160, 0),      // #FFA000 Amber
            ThemePreset::Retrowave => Color32::from_rgb(6, 182, 212),     // #06B6D4 Neon Cyan
            ThemePreset::OledBlack => Color32::from_rgb(255, 179, 0),     // #FFB300 Amber
            ThemePreset::GameBoy => Color32::from_rgb(155, 188, 15),      // #9BBC0F DMG Olive Green
            ThemePreset::ClassicDark => Color32::from_rgb(176, 190, 197), // Silver Gray
            ThemePreset::LightClassic => Color32::from_rgb(217, 119, 6),  // #D97706 Amber
            ThemePreset::Cyberpunk => Color32::from_rgb(0, 255, 204),     // Cyan Teal
        }
    }

    pub fn accent_color(&self) -> Color32 {
        self.secondary_color()
    }

    /// Tertiary pink/red accent (hearts / favorites)
    pub fn tertiary_color(&self) -> Color32 {
        Color32::from_rgb(255, 51, 102) // #FF3366 Pink / Heart Red
    }

    /// Main Canvas background: Deep rich navy (#080E16) matching screenshot
    pub fn bg_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(8, 14, 22),      // #080E16 Deep Navy Black
            ThemePreset::NintendoRetro => Color32::from_rgb(20, 20, 30),  // #14141E
            ThemePreset::SegaMega => Color32::from_rgb(13, 19, 31),       // #0D131F
            ThemePreset::Retrowave => Color32::from_rgb(18, 11, 30),      // #120B1E
            ThemePreset::OledBlack => Color32::from_rgb(0, 0, 0),         // #000000 Pure Black
            ThemePreset::GameBoy => Color32::from_rgb(18, 27, 16),        // #121B10
            ThemePreset::ClassicDark => Color32::from_rgb(22, 24, 29),
            ThemePreset::LightClassic => Color32::from_rgb(246, 248, 252),
            ThemePreset::Cyberpunk => Color32::from_rgb(13, 14, 20),
        }
    }

    /// Sidebar background (#0A1019)
    pub fn sidebar_bg_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(10, 16, 25),     // #0A1019 Sidebar Navy
            ThemePreset::OledBlack => Color32::from_rgb(0, 0, 0),
            ThemePreset::LightClassic => Color32::from_rgb(240, 243, 248),
            _ => self.bg_color(),
        }
    }

    /// Card background (#0E1724) matching screenshot cards
    pub fn card_bg_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(14, 23, 36),     // #0E1724 Card Body
            ThemePreset::NintendoRetro => Color32::from_rgb(32, 32, 46),  // #20202E
            ThemePreset::SegaMega => Color32::from_rgb(24, 34, 54),       // #182236
            ThemePreset::Retrowave => Color32::from_rgb(30, 19, 51),      // #1E1333
            ThemePreset::OledBlack => Color32::from_rgb(18, 18, 18),      // #121212
            ThemePreset::GameBoy => Color32::from_rgb(28, 42, 24),        // #1C2A18
            ThemePreset::ClassicDark => Color32::from_rgb(33, 36, 44),
            ThemePreset::LightClassic => Color32::from_rgb(255, 255, 255),
            ThemePreset::Cyberpunk => Color32::from_rgb(22, 24, 34),
        }
    }

    /// Surface container for chips, inputs, topbar search box (#111A27)
    pub fn surface_container(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(17, 26, 39),     // #111A27 Chips & Search
            ThemePreset::NintendoRetro => Color32::from_rgb(44, 44, 64),
            ThemePreset::SegaMega => Color32::from_rgb(32, 46, 74),
            ThemePreset::Retrowave => Color32::from_rgb(45, 29, 77),
            ThemePreset::OledBlack => Color32::from_rgb(28, 28, 28),
            ThemePreset::GameBoy => Color32::from_rgb(38, 58, 33),
            ThemePreset::ClassicDark => Color32::from_rgb(44, 48, 58),
            ThemePreset::LightClassic => Color32::from_rgb(223, 230, 240),
            ThemePreset::Cyberpunk => Color32::from_rgb(32, 36, 48),
        }
    }

    /// Card & widget stroke border color (#1A283B)
    pub fn card_border_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(26, 40, 59),     // #1A283B Dark Navy Border
            ThemePreset::NintendoRetro => Color32::from_rgb(60, 50, 70),
            ThemePreset::SegaMega => Color32::from_rgb(40, 60, 90),
            ThemePreset::Retrowave => Color32::from_rgb(70, 40, 100),
            ThemePreset::OledBlack => Color32::from_rgb(38, 38, 38),
            ThemePreset::GameBoy => Color32::from_rgb(48, 70, 42),
            ThemePreset::ClassicDark => Color32::from_rgb(55, 60, 72),
            ThemePreset::LightClassic => Color32::from_rgb(203, 213, 225),
            ThemePreset::Cyberpunk => Color32::from_rgb(50, 55, 75),
        }
    }

    /// Chip border stroke (#1D2C40)
    pub fn chip_border_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(29, 44, 64),     // #1D2C40
            _ => self.card_border_color(),
        }
    }

    /// Primary text color (pure white / crisp)
    pub fn text_color(&self) -> Color32 {
        if self.is_light() {
            Color32::from_rgb(15, 23, 42) // #0F172A
        } else {
            match self {
                ThemePreset::GameBoy => Color32::from_rgb(202, 220, 159),
                _ => Color32::from_rgb(245, 248, 252), // #F5F8FC Clean White
            }
        }
    }

    /// Secondary text color (cool slate gray)
    pub fn text_secondary(&self) -> Color32 {
        if self.is_light() {
            Color32::from_rgb(51, 65, 85)
        } else {
            Color32::from_rgb(140, 155, 174) // #8C9BAE Cool Gray
        }
    }

    /// Muted caption color
    pub fn text_muted(&self) -> Color32 {
        Color32::from_rgb(110, 126, 145) // #6E7E91
    }

    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let is_light = self.is_light();
        let mut visuals = if is_light { Visuals::light() } else { Visuals::dark() };

        visuals.panel_fill = self.bg_color();
        visuals.window_fill = self.card_bg_color();
        visuals.extreme_bg_color = if is_light {
            Color32::from_rgb(235, 240, 248)
        } else {
            self.surface_container()
        };

        // Window styling (modals)
        visuals.window_rounding = egui::Rounding::same(10.0);
        visuals.window_stroke = Stroke::new(1.0_f32, self.card_border_color());
        visuals.window_shadow.color = Color32::from_black_alpha(if is_light { 40 } else { 120 });
        visuals.window_shadow.blur = 16.0_f32;

        // Non-interactive items
        visuals.widgets.noninteractive.bg_fill = self.card_bg_color();
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_color());
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);

        // Inactive buttons, chips and inputs
        visuals.widgets.inactive.bg_fill = self.surface_container();
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_color());
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.chip_border_color());

        // Hovered states
        visuals.widgets.hovered.bg_fill = self.primary_color().gamma_multiply(0.35);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, self.primary_color());

        // Active / clicked states
        visuals.widgets.active.bg_fill = self.primary_color();
        visuals.widgets.active.fg_stroke = Stroke::new(2.0_f32, Color32::WHITE);
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);

        // Selection & highlights
        visuals.selection.bg_fill = self.primary_color();
        visuals.selection.stroke = Stroke::new(1.0_f32, self.primary_color());

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(12.0);
        ctx.set_style(style);
    }

    pub fn apply_to_context(&self, ctx: &egui::Context) {
        self.apply_to_ctx(ctx);
    }
}
