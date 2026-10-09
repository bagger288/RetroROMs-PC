use crate::models::GameCard;
use crate::theme::ThemePreset;
use egui::{Color32, RichText, ScrollArea, Ui, Vec2};

pub fn render_catalog_table(
    ui: &mut Ui,
    games: &[GameCard],
    current_page: usize,
    total_pages: usize,
    theme: ThemePreset,
    on_game_clicked: &mut Option<GameCard>,
    on_download_clicked: &mut Option<GameCard>,
    on_favorite_toggled: &mut Option<(GameCard, bool)>,
    on_play_clicked: &mut Option<GameCard>,
    on_page_changed: &mut Option<usize>,
) {
    if games.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("Игры не найдены").size(18.0).color(theme.text_color()));
        });
        return;
    }

    ScrollArea::vertical()
        .id_salt("catalog_table_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Header Row
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("Обложка").strong().size(12.0));
                ui.add_space(20.0);
                ui.label(RichText::new("Название").strong().size(12.0));
                ui.add_space(180.0);
                ui.label(RichText::new("Платформа").strong().size(12.0));
                ui.add_space(60.0);
                ui.label(RichText::new("Год").strong().size(12.0));
                ui.add_space(30.0);
                ui.label(RichText::new("Рейтинг").strong().size(12.0));
                ui.add_space(30.0);
                ui.label(RichText::new("Размер").strong().size(12.0));
                ui.add_space(40.0);
                ui.label(RichText::new("Действия").strong().size(12.0));
            });
            ui.separator();

            for game in games {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);

                    // Thumbnail
                    if let Some(cover_url) = &game.cover_url {
                        ui.add(
                            egui::Image::new(cover_url)
                                .fit_to_exact_size(Vec2::new(36.0, 36.0)),
                        );
                    } else {
                        ui.label("🎮");
                    }

                    ui.add_space(12.0);

                    // Title
                    let title_btn = ui.add_sized(
                        [220.0, 36.0],
                        egui::Button::new(
                            RichText::new(&game.title)
                                .color(theme.text_color())
                                .strong(),
                        )
                        .frame(false),
                    );
                    if title_btn.clicked() {
                        *on_game_clicked = Some(game.clone());
                    }

                    // Platform
                    ui.add_sized(
                        [100.0, 36.0],
                        egui::Label::new(
                            RichText::new(&game.console_name)
                                .color(theme.primary_color())
                                .size(11.0),
                        ),
                    );

                    // Year
                    ui.add_sized(
                        [50.0, 36.0],
                        egui::Label::new(RichText::new(&game.year).size(11.0).weak()),
                    );

                    // Rating
                    ui.add_sized(
                        [60.0, 36.0],
                        egui::Label::new(
                            RichText::new(format!("★ {:.1}", game.rating))
                                .size(11.0)
                                .color(Color32::from_rgb(255, 204, 0)),
                        ),
                    );

                    // Size
                    ui.add_sized(
                        [60.0, 36.0],
                        egui::Label::new(RichText::new(&game.file_size).size(11.0).weak()),
                    );

                    // Actions
                    ui.horizontal(|ui| {
                        // Fav
                        let fav_text = if game.is_favorite { "★" } else { "☆" };
                        let fav_color = if game.is_favorite {
                            Color32::from_rgb(255, 215, 0)
                        } else {
                            Color32::from_white_alpha(160)
                        };
                        if ui
                            .button(RichText::new(fav_text).color(fav_color))
                            .clicked()
                        {
                            *on_favorite_toggled = Some((game.clone(), !game.is_favorite));
                        }

                        // Play or Download
                        if game.is_downloaded {
                            if ui
                                .button(RichText::new("▶ Играть").color(Color32::from_rgb(0, 230, 118)))
                                .clicked()
                            {
                                *on_play_clicked = Some(game.clone());
                            }
                        } else if ui
                            .button(RichText::new("⬇").color(theme.primary_color()))
                            .clicked()
                        {
                            *on_download_clicked = Some(game.clone());
                        }

                        // Details
                        if ui.button("ℹ").clicked() {
                            *on_game_clicked = Some(game.clone());
                        }
                    });
                });
                ui.separator();
            }

            // Pagination Controls
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.label(format!("Страница {} из {}", current_page, total_pages));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_enabled(current_page < total_pages, egui::Button::new("Вперёд ▶"))
                        .clicked()
                    {
                        *on_page_changed = Some(current_page + 1);
                    }
                    ui.add_space(8.0);
                    if ui
                        .add_enabled(current_page > 1, egui::Button::new("◀ Назад"))
                        .clicked()
                    {
                        *on_page_changed = Some(current_page - 1);
                    }
                });
            });
            ui.add_space(20.0);
        });
}
