use crate::models::GameCard;
use crate::theme::ThemePreset;
use egui::{Color32, Frame, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};

pub fn render_catalog_grid(
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
            ui.label(RichText::new("Попробуйте сменить категорию или поисковый запрос").weak());
        });
        return;
    }

    ScrollArea::vertical()
        .id_salt("catalog_grid_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let available_width = ui.available_width().max(320.0);
            let min_card_width = 190.0;
            let spacing = 12.0;

            // Compute columns based on min width, and stretch cards evenly to fill width
            let cols = ((available_width + spacing) / (min_card_width + spacing))
                .floor()
                .max(1.0) as usize;
            let card_width = ((available_width - (cols as f32 - 1.0) * spacing) / cols as f32)
                .max(min_card_width);
            let card_height = 290.0;

            let mut row_idx = 0;
            while row_idx < games.len() {
                ui.horizontal(|ui| {
                    for col in 0..cols {
                        let idx = row_idx + col;
                        if idx < games.len() {
                            let game = &games[idx];
                            render_game_card(
                                ui,
                                game,
                                card_width,
                                card_height,
                                theme,
                                on_game_clicked,
                                on_download_clicked,
                                on_favorite_toggled,
                                on_play_clicked,
                            );
                            if col + 1 < cols {
                                ui.add_space(spacing);
                            }
                        }
                    }
                });
                ui.add_space(spacing);
                row_idx += cols;
            }

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

fn render_game_card(
    ui: &mut Ui,
    game: &GameCard,
    width: f32,
    height: f32,
    theme: ThemePreset,
    on_game_clicked: &mut Option<GameCard>,
    on_download_clicked: &mut Option<GameCard>,
    on_favorite_toggled: &mut Option<(GameCard, bool)>,
    on_play_clicked: &mut Option<GameCard>,
) {
    let frame = Frame::none()
        .fill(theme.card_bg_color())
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0_f32, theme.primary_color().gamma_multiply(0.3)))
        .inner_margin(8.0);

    frame.show(ui, |ui| {
        ui.set_width(width);
        ui.set_height(height);

        ui.vertical(|ui| {
            let inner_width = (width - 16.0).max(120.0);
            let img_height = 140.0;
            let img_box_size = Vec2::new(inner_width, img_height);

            // 1. Centered Image Container with dark background and true centering
            Frame::none()
                .fill(Color32::from_rgb(18, 22, 34))
                .rounding(Rounding::same(6.0))
                .show(ui, |ui| {
                    ui.set_min_size(img_box_size);
                    ui.set_max_size(img_box_size);
                    ui.centered_and_justified(|ui| {
                        let mut image_rendered = false;
                        if let Some(cover_url) = &game.cover_url {
                            if !cover_url.is_empty() {
                                let resp = ui.add(
                                    egui::Image::new(cover_url)
                                        .max_size(img_box_size)
                                        .rounding(Rounding::same(6.0)),
                                );
                                image_rendered = true;
                                if resp.clicked() {
                                    *on_game_clicked = Some(game.clone());
                                }
                            }
                        }

                        if !image_rendered {
                            ui.label(RichText::new("🎮").size(34.0));
                        }
                    });
                });

            ui.add_space(8.0);

            // 2. Title with truncate
            let title_text = RichText::new(&game.title)
                .color(theme.text_color())
                .strong()
                .size(13.0);
            if ui.add(egui::Label::new(title_text).truncate()).clicked() {
                *on_game_clicked = Some(game.clone());
            }

            ui.add_space(4.0);

            // 3. Info row: Console & Year
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&game.console_name)
                        .size(10.0)
                        .color(theme.primary_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(&game.year)
                            .size(10.0)
                            .color(Color32::from_white_alpha(120)),
                    );
                });
            });

            ui.add_space(2.0);

            // 4. Rating & File size row
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("★ {:.1}", game.rating))
                        .size(11.0)
                        .color(Color32::from_rgb(255, 204, 0)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(&game.file_size)
                            .size(10.0)
                            .color(Color32::from_white_alpha(140)),
                    );
                });
            });

            ui.add_space(6.0);

            // 5. Action Buttons centered horizontally across card
            let btn_row_size = Vec2::new(inner_width, 28.0);
            ui.allocate_ui_with_layout(
                btn_row_size,
                egui::Layout::left_to_right(egui::Align::Center).with_main_align(egui::Align::Center),
                |ui| {
                    ui.set_min_size(btn_row_size);
                    ui.set_max_size(btn_row_size);

                    // Favorite Star
                    let fav_text = if game.is_favorite { "★" } else { "☆" };
                    let fav_color = if game.is_favorite {
                        Color32::from_rgb(255, 215, 0)
                    } else {
                        Color32::from_white_alpha(160)
                    };
                    if ui
                        .add_sized([30.0, 26.0], egui::Button::new(RichText::new(fav_text).color(fav_color).size(13.0)))
                        .on_hover_text("Избранное")
                        .clicked()
                    {
                        *on_favorite_toggled = Some((game.clone(), !game.is_favorite));
                    }

                    // Main Action: Download or Play (symmetrically centered)
                    let main_btn_width = (inner_width - 76.0).max(75.0);
                    if game.is_downloaded {
                        if ui
                            .add_sized(
                                [main_btn_width, 26.0],
                                egui::Button::new(
                                    RichText::new("▶ Играть")
                                        .color(Color32::from_rgb(0, 230, 118))
                                        .strong()
                                        .size(11.0),
                                ),
                            )
                            .on_hover_text("Запустить в эмуляторе")
                            .clicked()
                        {
                            *on_play_clicked = Some(game.clone());
                        }
                    } else {
                        let dl_btn = ui.add_sized(
                            [main_btn_width, 26.0],
                            egui::Button::new(
                                RichText::new("⬇ Скачать")
                                    .color(theme.primary_color())
                                    .size(11.0),
                            ),
                        );
                        if dl_btn.clicked() {
                            *on_download_clicked = Some(game.clone());
                        }
                    }

                    // Details Button
                    if ui
                        .add_sized([30.0, 26.0], egui::Button::new(RichText::new("ℹ").size(12.0)))
                        .on_hover_text("Подробнее об игре")
                        .clicked()
                    {
                        *on_game_clicked = Some(game.clone());
                    }
                },
            );
        });
    });
}
