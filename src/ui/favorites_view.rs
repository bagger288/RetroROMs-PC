use crate::models::GameCard;
use crate::theme::ThemePreset;
use crate::ui::catalog_grid::render_catalog_grid;
use crate::ui::catalog_table::render_catalog_table;
use crate::ui::ViewMode;
use egui::{RichText, Ui};

pub fn render_favorites_view(
    ui: &mut Ui,
    favorites: &[GameCard],
    view_mode: ViewMode,
    theme: ThemePreset,
    on_game_clicked: &mut Option<GameCard>,
    on_download_clicked: &mut Option<GameCard>,
    on_favorite_toggled: &mut Option<(GameCard, bool)>,
    on_play_clicked: &mut Option<GameCard>,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(
            RichText::new("⭐ Избранные игры")
                .color(theme.primary_color())
                .strong(),
        );
        ui.label(RichText::new(format!("({} игр)", favorites.len())).weak());
    });
    ui.add_space(8.0);

    if favorites.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("Список избранного пуст").size(16.0).color(theme.text_color()));
            ui.label("Нажимайте на звездочку (☆) на любой игре в каталоге, чтобы добавить её сюда.");
        });
        return;
    }

    let mut dummy_page = None;
    match view_mode {
        ViewMode::Grid => {
            render_catalog_grid(
                ui,
                favorites,
                1,
                1,
                theme,
                on_game_clicked,
                on_download_clicked,
                on_favorite_toggled,
                on_play_clicked,
                &mut dummy_page,
            );
        }
        ViewMode::Table => {
            render_catalog_table(
                ui,
                favorites,
                1,
                1,
                theme,
                on_game_clicked,
                on_download_clicked,
                on_favorite_toggled,
                on_play_clicked,
                &mut dummy_page,
            );
        }
    }
}
