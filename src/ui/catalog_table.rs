use crate::models::GameCard;
use crate::theme::ThemePreset;
use egui::{Color32, RichText, Rounding, ScrollArea, Ui, Vec2};

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
            let available_width = ui.available_width().max(600.0);
            // Reserve fixed sizes for other columns, remaining for Title
            // Cover: 44, Platform: 150, Year: 55, Rating: 65, Size: 65, Actions: 130, Spacings: ~100
            let title_col_width = (available_width - 520.0).max(220.0);

            egui::Grid::new("catalog_table_grid")
                .num_columns(7)
                .spacing([14.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    // Header Row
                    ui.label(RichText::new("Обложка").strong().size(12.0));
                    ui.add_sized([title_col_width, 24.0], egui::Label::new(RichText::new("Название").strong().size(12.0)));
                    ui.add_sized([150.0, 24.0], egui::Label::new(RichText::new("Платформа").strong().size(12.0)));
                    ui.add_sized([55.0, 24.0], egui::Label::new(RichText::new("Год").strong().size(12.0)));
                    ui.add_sized([65.0, 24.0], egui::Label::new(RichText::new("Рейтинг").strong().size(12.0)));
                    ui.add_sized([65.0, 24.0], egui::Label::new(RichText::new("Размер").strong().size(12.0)));
                    ui.add_sized([130.0, 24.0], egui::Label::new(RichText::new("Действия").strong().size(12.0)));
                    ui.end_row();

                    for game in games {
                        // Col 0: Thumbnail (centered fixed size)
                        let thumb_size = Vec2::new(36.0, 36.0);
                        ui.allocate_ui_with_layout(
                            thumb_size,
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                if let Some(cover_url) = &game.cover_url {
                                    if !cover_url.is_empty() {
                                        ui.add(
                                            egui::Image::new(cover_url)
                                                .max_size(thumb_size)
                                                .rounding(Rounding::same(4.0)),
                                        );
                                    } else {
                                        ui.label(RichText::new("🎮").size(18.0));
                                    }
                                } else {
                                    ui.label(RichText::new("🎮").size(18.0));
                                }
                            },
                        );

                        // Col 1: Title (Strictly truncated button/label so long titles never push columns)
                        let title_text = RichText::new(&game.title)
                            .color(theme.text_color())
                            .strong();
                        let title_resp = ui.add_sized(
                            [title_col_width, 36.0],
                            egui::Button::new(title_text)
                                .truncate()
                                .frame(false),
                        );
                        if title_resp.clicked() {
                            *on_game_clicked = Some(game.clone());
                        }

                        // Col 2: Platform (Strictly fixed width)
                        ui.add_sized(
                            [150.0, 36.0],
                            egui::Label::new(
                                RichText::new(&game.console_name)
                                    .color(theme.primary_color())
                                    .size(11.0),
                            )
                            .truncate(),
                        );

                        // Col 3: Year (Strictly fixed width)
                        ui.add_sized(
                            [55.0, 36.0],
                            egui::Label::new(RichText::new(&game.year).size(11.0).weak()),
                        );

                        // Col 4: Rating (Strictly fixed width)
                        ui.add_sized(
                            [65.0, 36.0],
                            egui::Label::new(
                                RichText::new(format!("★ {:.1}", game.rating))
                                    .size(11.0)
                                    .color(Color32::from_rgb(255, 204, 0)),
                            ),
                        );

                        // Col 5: Size (Strictly fixed width)
                        ui.add_sized(
                            [65.0, 36.0],
                            egui::Label::new(RichText::new(&game.file_size).size(11.0).weak()),
                        );

                        // Col 6: Actions (Strictly fixed width)
                        ui.allocate_ui_with_layout(
                            Vec2::new(130.0, 36.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                // Fav
                                let fav_text = if game.is_favorite { "★" } else { "☆" };
                                let fav_color = if game.is_favorite {
                                    Color32::from_rgb(255, 215, 0)
                                } else {
                                    Color32::from_white_alpha(160)
                                };
                                if ui
                                    .button(RichText::new(fav_text).color(fav_color))
                                    .on_hover_text("Избранное")
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
                                    .on_hover_text("Скачать")
                                    .clicked()
                                {
                                    *on_download_clicked = Some(game.clone());
                                }

                                // Details
                                if ui
                                    .button("ℹ")
                                    .on_hover_text("Подробнее")
                                    .clicked()
                                {
                                    *on_game_clicked = Some(game.clone());
                                }
                            },
                        );

                        ui.end_row();
                    }
                });

            // Pagination Controls (always in view, never overflow right)
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Страница {} из {}", current_page, total_pages)).strong());
                ui.add_space(20.0);

                let prev_enabled = current_page > 1;
                if ui
                    .add_enabled(prev_enabled, egui::Button::new("◀ Назад"))
                    .clicked()
                {
                    *on_page_changed = Some(current_page - 1);
                }

                ui.add_space(8.0);

                let next_enabled = current_page < total_pages;
                if ui
                    .add_enabled(next_enabled, egui::Button::new("Вперёд ▶"))
                    .clicked()
                {
                    *on_page_changed = Some(current_page + 1);
                }
            });
            ui.add_space(24.0);
        });
}
