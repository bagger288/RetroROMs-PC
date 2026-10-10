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
            let row_spacing = 6.0;
            let thumb_size = Vec2::new(92.0, 58.0);
            let right_actions_width = 155.0; // Fav star (32) + spacing (8) + Download/Play btn (105) + margins

            for game in games {
                let available_w = (ui.available_width() - 14.0).max(360.0);

                let card_frame = Frame::none()
                    .fill(theme.card_bg_color())
                    .rounding(Rounding::same(8.0))
                    .stroke(Stroke::new(1.0, theme.primary_color().gamma_multiply(0.2)))
                    .inner_margin(egui::Margin {
                        left: 8.0,
                        right: 12.0,
                        top: 6.0,
                        bottom: 6.0,
                    });

                let row_resp = card_frame.show(ui, |ui| {
                    ui.set_width(available_w);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;

                        // 1. Cover Thumbnail (92x58px with dark backdrop, clickable for full zoom)
                        let thumb_frame = Frame::none()
                            .fill(Color32::from_rgb(18, 22, 34))
                            .rounding(Rounding::same(6.0))
                            .show(ui, |ui| {
                                ui.set_min_size(thumb_size);
                                ui.set_max_size(thumb_size);
                                ui.centered_and_justified(|ui| {
                                    if let Some(cover_url) = &game.cover_url {
                                        if !cover_url.is_empty() {
                                            ui.add(
                                                egui::Image::new(cover_url)
                                                    .max_size(thumb_size)
                                                    .rounding(Rounding::same(6.0)),
                                            );
                                        } else {
                                            ui.label(RichText::new("🎮").size(24.0));
                                        }
                                    } else {
                                        ui.label(RichText::new("🎮").size(24.0));
                                    }
                                });
                            });

                        let thumb_interact = ui.interact(
                            thumb_frame.response.rect,
                            ui.id().with("list_cover_click").with(&game.id),
                            egui::Sense::click(),
                        );
                        if thumb_interact.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if thumb_interact
                            .on_hover_text("🔍 Нажмите, чтобы рассмотреть обложку (скроллинг для зума)")
                            .clicked()
                        {
                            if let Some(cover_url) = &game.cover_url {
                                if !cover_url.is_empty() {
                                    *on_view_image = Some((format!("Обложка: {}", game.title), cover_url.clone()));
                                }
                            }
                        }

                        // 2. Middle section: Game Title + Metadata Badges
                        let middle_width = (available_w - thumb_size.x - right_actions_width - 36.0).max(120.0);

                        let info_resp = ui.allocate_ui_with_layout(
                            Vec2::new(middle_width, 58.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.add_space(2.0);

                                // Line 1: Game Title
                                let title_text = RichText::new(&game.title)
                                    .size(14.0)
                                    .strong()
                                    .color(theme.text_color());
                                let title_label = ui.add_sized(
                                    [middle_width, 22.0],
                                    egui::Label::new(title_text).truncate(),
                                );
                                title_label.on_hover_text(&game.title);

                                ui.add_space(6.0);

                                // Line 2: Metadata Badges (Platform, Year, Genre, Rating, Size)
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;

                                    // Platform Pill Badge
                                    Frame::none()
                                        .fill(theme.primary_color().gamma_multiply(0.18))
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(&game.console_name)
                                                    .size(10.5)
                                                    .color(theme.primary_color())
                                                    .strong(),
                                            );
                                        });

                                    // Year
                                    if !game.year.is_empty() {
                                        ui.label(
                                            RichText::new(&game.year)
                                                .size(11.0)
                                                .color(Color32::from_white_alpha(150)),
                                        );
                                        ui.label(RichText::new("•").size(10.0).weak());
                                    }

                                    // Genre
                                    if !game.genre.is_empty() && game.genre != "Не указан" {
                                        ui.label(
                                            RichText::new(&game.genre)
                                                .size(11.0)
                                                .color(Color32::from_white_alpha(150)),
                                        );
                                        ui.label(RichText::new("•").size(10.0).weak());
                                    }

                                    // Rating Badge with Star
                                    ui.label(
                                        RichText::new(format!("★ {:.1}", game.rating))
                                            .size(11.5)
                                            .color(Color32::from_rgb(255, 204, 0))
                                            .strong(),
                                    );

                                    // File Size
                                    if !game.file_size.is_empty() {
                                        ui.label(RichText::new("•").size(10.0).weak());
                                        ui.label(
                                            RichText::new(&game.file_size)
                                                .size(11.0)
                                                .color(Color32::from_white_alpha(140)),
                                        );
                                    }
                                });
                            },
                        );

                        // Clicking anywhere on title/info area opens game details
                        let info_interact = ui.interact(
                            info_resp.response.rect,
                            ui.id().with("list_info_click").with(&game.id),
                            egui::Sense::click(),
                        );
                        if info_interact.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if info_interact
                            .on_hover_text("📖 Нажмите, чтобы открыть подробную информацию об игре")
                            .clicked()
                        {
                            *on_game_clicked = Some(game.clone());
                        }

                        // 3. Right Section: Action Buttons (Always cleanly docked to the right edge)
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;

                            // Main Action: Download or Play button
                            let btn_w = 105.0;
                            let btn_h = 30.0;
                            if game.is_downloaded {
                                if ui
                                    .add_sized(
                                        [btn_w, btn_h],
                                        egui::Button::new(
                                            RichText::new("▶ Играть")
                                                .color(Color32::from_rgb(0, 230, 118))
                                                .strong()
                                                .size(12.0),
                                        ),
                                    )
                                    .on_hover_text("Запустить в эмуляторе")
                                    .clicked()
                                {
                                    *on_play_clicked = Some(game.clone());
                                }
                            } else {
                                let dl_btn = ui.add_sized(
                                    [btn_w, btn_h],
                                    egui::Button::new(
                                        RichText::new("⬇ Скачать")
                                            .color(theme.primary_color())
                                            .strong()
                                            .size(12.0),
                                    ),
                                );
                                if dl_btn.on_hover_text("Выбрать версию ROM для скачивания").clicked() {
                                    *on_download_clicked = Some(game.clone());
                                }
                            }

                            // Favorite Star button
                            let fav_text = if game.is_favorite { "★" } else { "☆" };
                            let fav_color = if game.is_favorite {
                                Color32::from_rgb(255, 215, 0)
                            } else {
                                Color32::from_white_alpha(160)
                            };
                            if ui
                                .add_sized(
                                    [32.0, btn_h],
                                    egui::Button::new(RichText::new(fav_text).color(fav_color).size(14.0)),
                                )
                                .on_hover_text(if game.is_favorite { "Удалить из избранного" } else { "Добавить в избранное" })
                                .clicked()
                            {
                                *on_favorite_toggled = Some((game.clone(), !game.is_favorite));
                            }
                        });
                    });
                });

                if row_resp.response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                ui.add_space(row_spacing);
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
