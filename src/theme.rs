use egui::{Color32, Stroke, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    ArcadeNeon,     // Киберпанк / Неон (Android default)
    Cyberpunk,      // Киберпанк желтый / бирюзовый
    Retrowave,      // Synthwave 80s
    ClassicDark,    // Темная классика
    LightClassic,   // Светлая классика (Android Light)
    NintendoRetro,  // Nintendo Retro (Красный / Золотой)
    SegaMega,       // Sega Mega (Синий / Оранжевый)
    GameBoy,        // Game Boy Lime (Оливковый)
    OledBlack,      // OLED Pure Black (Изумруд / Черный)
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
            ThemePreset::ArcadeNeon => "⚡ Киберпанк (Неон)",
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

    /// Primary accent color matching Android AppThemeManager
    pub fn primary_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(0, 229, 255),    // #00E5FF Neon Cyan
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

    /// Secondary accent color matching Android AppThemeManager
    pub fn secondary_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(255, 179, 0),    // #FFB300 Golden Amber
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

    /// Tertiary pink/red accent
    pub fn tertiary_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(255, 51, 102),   // #FF3366 Retro Magenta
            ThemePreset::LightClassic => Color32::from_rgb(225, 29, 72),  // Rose
            _ => Color32::from_rgb(255, 64, 129),
        }
    }

    /// Background color matching Android palette
    pub fn bg_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(11, 14, 20),     // #0B0E14 Deep Navy Black
            ThemePreset::NintendoRetro => Color32::from_rgb(20, 20, 30),  // #14141E
            ThemePreset::SegaMega => Color32::from_rgb(13, 19, 31),       // #0D131F
            ThemePreset::Retrowave => Color32::from_rgb(18, 11, 30),      // #120B1E
            ThemePreset::OledBlack => Color32::from_rgb(0, 0, 0),         // #000000 Pure Black
            ThemePreset::GameBoy => Color32::from_rgb(18, 27, 16),        // #121B10
            ThemePreset::ClassicDark => Color32::from_rgb(22, 24, 29),    // Classic Dark
            ThemePreset::LightClassic => Color32::from_rgb(246, 248, 252),// #F6F8FC Light background
            ThemePreset::Cyberpunk => Color32::from_rgb(13, 14, 20),
        }
    }

    /// Surface / Card background matching Android surface
    pub fn card_bg_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(20, 25, 35),     // #141923 Android Surface
            ThemePreset::NintendoRetro => Color32::from_rgb(32, 32, 46),  // #20202E
            ThemePreset::SegaMega => Color32::from_rgb(24, 34, 54),       // #182236
            ThemePreset::Retrowave => Color32::from_rgb(30, 19, 51),      // #1E1333
            ThemePreset::OledBlack => Color32::from_rgb(18, 18, 18),      // #121212
            ThemePreset::GameBoy => Color32::from_rgb(28, 42, 24),        // #1C2A18
            ThemePreset::ClassicDark => Color32::from_rgb(33, 36, 44),
            ThemePreset::LightClassic => Color32::from_rgb(255, 255, 255),// #FFFFFF
            ThemePreset::Cyberpunk => Color32::from_rgb(22, 24, 34),
        }
    }

    /// Surface container for nested groups, headers, modal toolbars
    pub fn surface_container(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(35, 43, 62),     // #232B3E
            ThemePreset::NintendoRetro => Color32::from_rgb(44, 44, 64),
            ThemePreset::SegaMega => Color32::from_rgb(32, 46, 74),
            ThemePreset::Retrowave => Color32::from_rgb(45, 29, 77),
            ThemePreset::OledBlack => Color32::from_rgb(28, 28, 28),
            ThemePreset::GameBoy => Color32::from_rgb(38, 58, 33),
            ThemePreset::ClassicDark => Color32::from_rgb(44, 48, 58),
            ThemePreset::LightClassic => Color32::from_rgb(223, 230, 240),// #DFE6F0
            ThemePreset::Cyberpunk => Color32::from_rgb(32, 36, 48),
        }
    }

    /// Card border stroke color matching Android #2F3B52
    pub fn card_border_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(47, 59, 82),     // #2F3B52
            ThemePreset::NintendoRetro => Color32::from_rgb(60, 50, 70),
            ThemePreset::SegaMega => Color32::from_rgb(40, 60, 90),
            ThemePreset::Retrowave => Color32::from_rgb(70, 40, 100),
            ThemePreset::OledBlack => Color32::from_rgb(38, 38, 38),
            ThemePreset::GameBoy => Color32::from_rgb(48, 70, 42),
            ThemePreset::ClassicDark => Color32::from_rgb(55, 60, 72),
            ThemePreset::LightClassic => Color32::from_rgb(203, 213, 225),// #CBD5E1
            ThemePreset::Cyberpunk => Color32::from_rgb(50, 55, 75),
        }
    }

    /// Primary text color
    pub fn text_color(&self) -> Color32 {
        if self.is_light() {
            Color32::from_rgb(15, 23, 42) // #0F172A
        } else {
            match self {
                ThemePreset::GameBoy => Color32::from_rgb(202, 220, 159),
                _ => Color32::from_rgb(241, 245, 249), // #F1F5F9 Slate 100
            }
        }
    }

    /// Secondary text color (muted)
    pub fn text_secondary(&self) -> Color32 {
        if self.is_light() {
            Color32::from_rgb(51, 65, 85) // #334155
        } else {
            Color32::from_rgb(148, 163, 184) // #94A3B8
        }
    }

    /// Muted caption color
    pub fn text_muted(&self) -> Color32 {
        Color32::from_rgb(100, 116, 139) // #64748B
    }

    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let is_light = self.is_light();
        let mut visuals = if is_light { Visuals::light() } else { Visuals::dark() };

        visuals.panel_fill = self.bg_color();
        visuals.window_fill = self.card_bg_color();
        visuals.extreme_bg_color = if is_light {
            Color32::from_rgb(235, 240, 248)
        } else {
            Color32::from_black_alpha(150)
        };

        // Window styling (modals)
        visuals.window_rounding = egui::Rounding::same(10.0);
        visuals.window_stroke = Stroke::new(1.0_f32, self.card_border_color());
        visuals.window_shadow = egui::epaint::Shadow {
            extrusion: 16.0,
            color: Color32::from_black_alpha(if is_light { 40 } else { 120 }),
        };

        // Non-interactive items
        visuals.widgets.noninteractive.bg_fill = self.card_bg_color();
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_color());
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);

        // Inactive buttons and inputs
        visuals.widgets.inactive.bg_fill = self.surface_container();
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_color());
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.card_border_color());

        // Hovered states
        visuals.widgets.hovered.bg_fill = self.primary_color().gamma_multiply(0.20);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, self.primary_color());
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, self.primary_color().gamma_multiply(0.8));

        // Active / clicked states
        visuals.widgets.active.bg_fill = self.primary_color().gamma_multiply(0.35);
        visuals.widgets.active.fg_stroke = Stroke::new(2.0_f32, self.primary_color());
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);

        // Selection & highlights
        visuals.selection.bg_fill = self.primary_color().gamma_multiply(0.30);
        visuals.selection.stroke = Stroke::new(1.0_f32, self.primary_color());

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.window_margin = egui::Margin::same(12.0);
        ctx.set_style(style);
    }

    pub fn apply_to_context(&self, ctx: &egui::Context) {
        self.apply_to_ctx(ctx);
    }
}
