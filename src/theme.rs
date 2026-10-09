use egui::{Color32, Stroke, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    ArcadeNeon,
    Cyberpunk,
    Retrowave,
    ClassicDark,
    GameBoy,
}

impl ThemePreset {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Cyberpunk" => ThemePreset::Cyberpunk,
            "Retrowave" => ThemePreset::Retrowave,
            "ClassicDark" => ThemePreset::ClassicDark,
            "GameBoy" => ThemePreset::GameBoy,
            _ => ThemePreset::ArcadeNeon,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemePreset::ArcadeNeon => "ArcadeNeon",
            ThemePreset::Cyberpunk => "Cyberpunk",
            ThemePreset::Retrowave => "Retrowave",
            ThemePreset::ClassicDark => "ClassicDark",
            ThemePreset::GameBoy => "GameBoy",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ThemePreset::ArcadeNeon => "🕹 Arcade Neon",
            ThemePreset::Cyberpunk => "⚡ Cyberpunk 2077",
            ThemePreset::Retrowave => "🌴 Retrowave Sunset",
            ThemePreset::ClassicDark => "🌑 Classic Dark",
            ThemePreset::GameBoy => "👾 GameBoy Matrix",
        }
    }

    pub fn primary_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(0, 229, 255),    // Vibrant Cyan
            ThemePreset::Cyberpunk => Color32::from_rgb(255, 230, 0),     // Neon Yellow
            ThemePreset::Retrowave => Color32::from_rgb(255, 110, 180),   // Hot Pink
            ThemePreset::ClassicDark => Color32::from_rgb(100, 181, 246), // Soft Blue
            ThemePreset::GameBoy => Color32::from_rgb(139, 172, 15),      // DMG Bright Green
        }
    }

    pub fn secondary_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(255, 0, 128),    // Neon Magenta
            ThemePreset::Cyberpunk => Color32::from_rgb(0, 255, 204),     // Cyan Teal
            ThemePreset::Retrowave => Color32::from_rgb(255, 140, 0),     // Sunset Orange
            ThemePreset::ClassicDark => Color32::from_rgb(176, 190, 197), // Silver Gray
            ThemePreset::GameBoy => Color32::from_rgb(155, 188, 15),      // DMG Olive Green
        }
    }

    pub fn bg_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(14, 15, 24),
            ThemePreset::Cyberpunk => Color32::from_rgb(13, 14, 20),
            ThemePreset::Retrowave => Color32::from_rgb(20, 12, 30),
            ThemePreset::ClassicDark => Color32::from_rgb(24, 25, 28),
            ThemePreset::GameBoy => Color32::from_rgb(15, 56, 15),
        }
    }

    pub fn card_bg_color(&self) -> Color32 {
        match self {
            ThemePreset::ArcadeNeon => Color32::from_rgb(24, 26, 42),
            ThemePreset::Cyberpunk => Color32::from_rgb(22, 24, 34),
            ThemePreset::Retrowave => Color32::from_rgb(34, 20, 50),
            ThemePreset::ClassicDark => Color32::from_rgb(35, 37, 42),
            ThemePreset::GameBoy => Color32::from_rgb(48, 98, 48),
        }
    }

    pub fn text_color(&self) -> Color32 {
        match self {
            ThemePreset::GameBoy => Color32::from_rgb(202, 220, 159),
            _ => Color32::from_rgb(240, 242, 248),
        }
    }

    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        visuals.panel_fill = self.bg_color();
        visuals.window_fill = self.card_bg_color();
        visuals.extreme_bg_color = Color32::from_black_alpha(120);

        visuals.widgets.noninteractive.bg_fill = self.card_bg_color();
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_color());

        visuals.widgets.inactive.bg_fill = self.card_bg_color();
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_color());
        visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);

        visuals.widgets.hovered.bg_fill = self.secondary_color().gamma_multiply(0.25);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, self.primary_color());
        visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);

        visuals.widgets.active.bg_fill = self.primary_color().gamma_multiply(0.35);
        visuals.widgets.active.fg_stroke = Stroke::new(2.0_f32, self.primary_color());
        visuals.widgets.active.rounding = egui::Rounding::same(6.0);

        visuals.selection.bg_fill = self.primary_color().gamma_multiply(0.35);
        visuals.selection.stroke = Stroke::new(1.0_f32, self.primary_color());

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        ctx.set_style(style);
    }

    pub fn apply_to_context(&self, ctx: &egui::Context) {
        self.apply_to_ctx(ctx);
    }
}
