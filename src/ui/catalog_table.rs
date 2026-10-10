use crate::models::GameCard;
use crate::theme::ThemePreset;
use egui::{Color32, Frame, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};

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
    on_view_image: &mut Option<(String, String)>,
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
        .id_salt("catalog_table_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let col_spacing = 10.0;
            let cover_w = 64.0;
            let plat_w = 135.0;
            let year_w = 55.0;
            let region_w = 85.0;
            let size_w = 70.0;
            let actions_w = 140.0;
            let fixed_total = cover_w + plat_w + year_w + region_w + size_w + actions_w + (col_spacing * 6.0);

            let row_h = 44.0;
            let margin_x = 12.0;
            let available_w = (ui.available_width() - 16.0).max(fixed_total + 180.0 + margin_x);
            let title_w = (available_w - margin_x - fixed_total).max(180.0);

            // Table Header Row
            Frame::none()
                .fill(theme.card_bg_color().gamma_multiply(0.8))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = col_spacing;

                        ui.add_sized([cover_w, 20.0], egui::Label::new(RichText::new("Обложка").strong().size(12.0)));
                        ui.add_sized([plat_w, 20.0], egui::Label::new(RichText::new("Платформа").strong().size(12.0)));
                        ui.add_sized([year_w, 20.0], egui::Label::new(RichText::new("Год").strong().size(12.0)));
                        ui.add_sized([title_w, 20.0], egui::Label::new(RichText::new("Название").strong().size(12.0)));
                        ui.add_sized([region_w, 20.0], egui::Label::new(RichText::new("Язык / Регион").strong().size(12.0)));
                        ui.add_sized([size_w, 20.0], egui::Label::new(RichText::new("Размер").strong().size(12.0)));
                        ui.add_sized([actions_w, 20.0], egui::Label::new(RichText::new("Действия").strong().size(12.0)));
                    });
                });

            ui.add_space(4.0);

            // Table Data Rows
            for (idx, game) in games.iter().enumerate() {
                let bg_color = if idx % 2 == 0 {
                    theme.card_bg_color()
                } else {
                    theme.card_bg_color().gamma_multiply(0.55)
                };

                let card_frame = Frame::none()
                    .fill(bg_color)
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0, theme.primary_color().gamma_multiply(0.12)))
                    .inner_margin(egui::Margin::symmetric(6.0, 3.0));

                card_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = col_spacing;

                        // 1. Cover (64x38 px container, clickable for full image viewer)
                        let thumb_size = Vec2::new(56.0, 38.0);
                        ui.allocate_ui_with_layout(
                            Vec2::new(cover_w, row_h),
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                let img_frame = Frame::none()
                                    .fill(Color32::from_rgb(18, 22, 34))
                                    .rounding(Rounding::same(4.0))
                                    .show(ui, |ui| {
                                        ui.set_min_size(thumb_size);
                                        ui.set_max_size(thumb_size);
                                        ui.centered_and_justified(|ui| {
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
                                        });
                                    });

                                let img_resp = ui.interact(
                                    img_frame.response.rect,
                                    ui.id().with("table_cover").with(&game.id),
                                    egui::Sense::click(),
                                );
                                if img_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if img_resp.on_hover_text("🔍 Нажмите, чтобы рассмотреть обложку").clicked() {
                                    if let Some(cover_url) = &game.cover_url {
                                        if !cover_url.is_empty() {
                                            *on_view_image = Some((format!("Обложка: {}", game.title), cover_url.clone()));
                                        }
                                    }
                                }
                            },
                        );

                        // 2. Platform (Strictly fixed width, colored label)
                        ui.add_sized(
                            [plat_w, row_h],
                            egui::Label::new(
                                RichText::new(&game.console_name)
                                    .color(theme.primary_color())
                                    .size(11.5)
                                    .strong(),
                            ).truncate(),
                        );

                        // 3. Year of Release (Strictly fixed width)
                        let year_text = if game.year.is_empty() { "—".to_string() } else { game.year.clone() };
                        ui.add_sized(
                            [year_w, row_h],
                            egui::Label::new(
                                RichText::new(year_text)
                                    .size(11.0)
                                    .color(Color32::from_white_alpha(160)),
                            ),
                        );

                        // 4. Game Title (Flexible width, bold, truncated, opens details on click)
                        let title_resp = ui.add_sized(
                            [title_w, row_h],
                            egui::Button::new(
                                RichText::new(&game.title)
                                    .size(12.5)
                                    .strong()
                                    .color(theme.text_color()),
                            )
                            .truncate()
                            .frame(false),
                        );
                        title_resp.on_hover_text(&game.title);
                        if title_resp.clicked() {
                            *on_game_clicked = Some(game.clone());
                        }

                        // 5. Language / Region Badge (Strictly fixed width)
                        ui.allocate_ui_with_layout(
                            Vec2::new(region_w, row_h),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let (region_label, bg_col, fg_col) = if !game.regions.is_empty() {
                                    let first = game.regions.join("/");
                                    if first.contains("RUS") || first.contains("Пиратки") {
                                        (first, Color32::from_rgb(0, 100, 160), Color32::from_rgb(180, 235, 255))
                                    } else if first.contains("USA") {
                                        (first, Color32::from_rgb(30, 120, 60), Color32::from_rgb(200, 255, 210))
                                    } else if first.contains("EUR") {
                                        (first, Color32::from_rgb(150, 110, 20), Color32::from_rgb(255, 240, 190))
                                    } else if first.contains("JAP") {
                                        (first, Color32::from_rgb(160, 50, 30), Color32::from_rgb(255, 210, 200))
                                    } else {
                                        (first, Color32::from_rgb(50, 55, 70), Color32::from_white_alpha(200))
                                    }
                                } else if game.title.to_lowercase().contains("rus") || game.title.contains("Перевод") {
                                    ("RUS".into(), Color32::from_rgb(0, 100, 160), Color32::from_rgb(180, 235, 255))
                                } else if game.title.to_lowercase().contains("(e)") || game.title.contains("(eur") {
                                    ("EUR".into(), Color32::from_rgb(150, 110, 20), Color32::from_rgb(255, 240, 190))
                                } else if game.title.to_lowercase().contains("(j)") || game.title.contains("(jap") {
                                    ("JAP".into(), Color32::from_rgb(160, 50, 30), Color32::from_rgb(255, 210, 200))
                                } else {
                                    ("ROM".into(), Color32::from_rgb(50, 55, 70), Color32::from_white_alpha(190))
                                };

                                Frame::none()
                                    .fill(bg_col)
                                    .rounding(Rounding::same(4.0))
                                    .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.add_sized(
                                            [64.0, 16.0],
                                            egui::Label::new(
                                                RichText::new(region_label)
                                                    .color(fg_col)
                                                    .size(10.0)
                                                    .strong(),
                                            ).truncate(),
                                        );
                                    });
                            },
                        );

                        // 6. Size (Strictly fixed width)
                        let size_text = if game.file_size.is_empty() { "—".to_string() } else { game.file_size.clone() };
                        ui.add_sized(
                            [size_w, row_h],
                            egui::Label::new(
                                RichText::new(size_text)
                                    .size(11.0)
                                    .color(Color32::from_white_alpha(150)),
                            ),
                        );

                        // 7. Action Buttons (Strictly fixed width: Fav star + Download/Play)
                        ui.allocate_ui_with_layout(
                            Vec2::new(actions_w, row_h),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;

                                // Favorite Star
                                let fav_text = if game.is_favorite { "★" } else { "☆" };
                                let fav_color = if game.is_favorite {
                                    Color32::from_rgb(255, 215, 0)
                                } else {
                                    Color32::from_white_alpha(160)
                                };
                                if ui
                                    .add_sized(
                                        [30.0, 28.0],
                                        egui::Button::new(RichText::new(fav_text).color(fav_color).size(13.0)),
                                    )
                                    .on_hover_text(if game.is_favorite { "Удалить из избранного" } else { "Добавить в избранное" })
                                    .clicked()
                                {
                                    *on_favorite_toggled = Some((game.clone(), !game.is_favorite));
                                }

                                // Main Action button: Download or Play
                                let btn_w = 98.0;
                                if game.is_downloaded {
                                    if ui
                                        .add_sized(
                                            [btn_w, 28.0],
                                            egui::Button::new(
                                                RichText::new("▶ Играть")
                                                    .color(Color32::from_rgb(0, 230, 118))
                                                    .strong()
                                                    .size(11.5),
                                            ),
                                        )
                                        .on_hover_text("Запустить в эмуляторе")
                                        .clicked()
                                    {
                                        *on_play_clicked = Some(game.clone());
                                    }
                                } else {
                                    let dl_btn = ui.add_sized(
                                        [btn_w, 28.0],
                                        egui::Button::new(
                                            RichText::new("⬇ Скачать")
                                                .color(theme.primary_color())
                                                .strong()
                                                .size(11.5),
                                        ),
                                    );
                                    if dl_btn.on_hover_text("Выбрать версию ROM для скачивания").clicked() {
                                        *on_download_clicked = Some(game.clone());
                                    }
                                }
                            },
                        );
                    });
                });

                ui.add_space(2.0);
            }

            // Pagination Controls
            if total_pages > 1 {
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
            }
        });
}
