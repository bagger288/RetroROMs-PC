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
        .id_salt("catalog_grid_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let spacing = 12.0;
            let min_card_width = 180.0;
            let scrollbar_reserve = 18.0;
            let available_width = (ui.available_width() - scrollbar_reserve).max(min_card_width);

            // Compute columns dynamically based on true available space
            let cols = (((available_width + spacing) / (min_card_width + spacing))
                .floor()
                .max(1.0)) as usize;

            let total_spacing = (cols as f32 - 1.0) * spacing;
            let card_width = ((available_width - total_spacing) / cols as f32).floor();
            let card_height = 275.0;

            let mut row_idx = 0;
            while row_idx < games.len() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = spacing;
                    for col in 0..cols {
                        let idx = row_idx + col;
                        if idx < games.len() {
                            let game = &games[idx];
                            ui.allocate_ui_with_layout(
                                Vec2::new(card_width, card_height),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(card_width);
                                    ui.set_height(card_height);
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
                                        on_view_image,
                                    );
                                },
                            );
                        }
                    }
                });
                ui.add_space(spacing);
                row_idx += cols;
            }

            // Pagination Controls
            if total_pages > 1 {
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("Страница {} из {}", current_page, total_pages))
                            .strong()
                            .color(theme.primary_color()),
                    );
                    ui.add_space(20.0);

                    let prev_enabled = current_page > 1;
                    if ui.add_enabled(prev_enabled, egui::Button::new("◀ Назад")).clicked() {
                        *on_page_changed = Some(current_page - 1);
                    }

                    ui.add_space(8.0);

                    let next_enabled = current_page < total_pages;
                    if ui.add_enabled(next_enabled, egui::Button::new("Вперёд ▶")).clicked() {
                        *on_page_changed = Some(current_page + 1);
                    }
                });
                ui.add_space(24.0);
            }
        });
}

fn get_console_badge_info(slug: &str, console_name: &str) -> (&'static str, &'static str) {
    match slug {
        "dendy" => ("NES", "/ Денди / Famicom"),
        "genesis" => ("GENESIS", "/ MegaDrive"),
        "snes" => ("SNES", "/ Super Famicom"),
        "psx" => ("PS1", "/ PlayStation"),
        "n64" => ("N64", "/ Nintendo 64"),
        "gba" => ("GBA", "/ Game Boy Advance"),
        "gb" => ("GB", "/ Game Boy"),
        "gbc" => ("GBC", "/ Game Boy Color"),
        "32x" => ("32X", "/ Mega Drive 32X"),
        "segacd" => ("SEGA CD", "/ Mega-CD"),
        "sms" => ("SMS", "/ Master System"),
        "sg-1000" => ("SG-1000", "/ Sega SG-1000"),
        "pce" => ("PCE", "/ TurboGrafx-16"),
        "pcecd" => ("PCE CD", "/ TurboGrafx-CD"),
        "3do" => ("3DO", "/ 3DO Interactive"),
        "famicom_disk_system" => ("FDS", "/ Famicom Disk"),
        "neogeocd" => ("NEOGEO", "/ Neo Geo CD"),
        "jaguar" => ("JAGUAR", "/ Atari Jaguar"),
        "gg" => ("GAME GEAR", "/ Sega Game Gear"),
        "lynx" => ("LYNX", "/ Atari Lynx"),
        "ngp" => ("NGP", "/ Neo Geo Pocket"),
        "ws" => ("WS", "/ WonderSwan"),
        "vboy" => ("VIRTUAL BOY", "/ Nintendo"),
        "pmini" => ("POKEMINI", "/ Pokémon Mini"),
        "sv" => ("SV", "/ Supervision"),
        "2600" => ("ATARI 2600", "/ Atari"),
        "5200" => ("ATARI 5200", "/ Atari"),
        "7800" => ("ATARI 7800", "/ Atari"),
        "coleco" => ("COLECO", "/ ColecoVision"),
        "vectrex" => ("VECTREX", "/ Vectrex"),
        "intellivision" => ("INTV", "/ Intellivision"),
        "arcadia" => ("ARCADIA", "/ Arcadia 2001"),
        "chaf" => ("CHANNEL F", "/ Fairchild"),
        _ => ("ROM", ""),
    }
}

fn get_format_label(slug: &str) -> &'static str {
    match slug {
        "psx" | "3do" | "segacd" | "pcecd" | "neogeocd" => "ISO",
        _ => "ROM",
    }
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
    on_view_image: &mut Option<(String, String)>,
) {
    let margin = 8.0_f32;
    let stroke_width: f32 = 1.0_f32;
    let inner_width = (width - (margin + stroke_width) * 2.0).max(80.0);
    let inner_height = (height - (margin + stroke_width) * 2.0).max(80.0);

    let frame = Frame::none()
        .fill(theme.card_bg_color())
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(stroke_width, theme.card_border_color()))
        .inner_margin(margin);

    frame.show(ui, |ui| {
        ui.set_min_size(Vec2::new(inner_width, inner_height));
        ui.set_max_size(Vec2::new(inner_width, inner_height));

        ui.vertical(|ui| {
            // 1. Cover Image (Large, 125px tall)
            let img_height = 125.0;
            let img_box_size = Vec2::new(inner_width, img_height);

            let img_frame_resp = Frame::none()
                .fill(Color32::from_rgb(10, 15, 23))
                .rounding(Rounding::same(6.0))
                .show(ui, |ui| {
                    ui.set_min_size(img_box_size);
                    ui.set_max_size(img_box_size);
                    ui.centered_and_justified(|ui| {
                        let mut image_rendered = false;
                        if let Some(cover_url) = &game.cover_url {
                            if !cover_url.is_empty() {
                                ui.add(
                                    egui::Image::new(cover_url)
                                        .max_size(img_box_size)
                                        .rounding(Rounding::same(6.0)),
                                );
                                image_rendered = true;
                            }
                        }

                        if !image_rendered {
                            ui.label(RichText::new("🎮").size(34.0));
                        }
                    });
                });

            let cover_resp = ui.interact(
                img_frame_resp.response.rect,
                ui.id().with("cover_click").with(&game.id),
                egui::Sense::click(),
            );
            if cover_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if cover_resp.on_hover_text("🔍 Нажмите для просмотра обложки").clicked() {
                if let Some(cover_url) = &game.cover_url {
                    if !cover_url.is_empty() {
                        *on_view_image = Some((format!("Обложка: {}", game.title), cover_url.clone()));
                    }
                }
            }

            ui.add_space(6.0);

            // 2. Title & Metadata (Clickable to open details)
            let info_block_resp = ui.vertical(|ui| {
                // Game Title
                let title_text = RichText::new(&game.title)
                    .color(Color32::WHITE)
                    .strong()
                    .size(13.0);
                ui.add_sized(
                    [inner_width, 18.0],
                    egui::Label::new(title_text).truncate(),
                );

                ui.add_space(3.0);

                // Platform Badge + Subname + Year row
                let (badge_text, subname) = get_console_badge_info(&game.console_slug, &game.console_name);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    // Blue Pill Badge (e.g. [ NES ])
                    Frame::none()
                        .fill(theme.primary_color())
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(4.0, 1.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(badge_text)
                                    .color(Color32::WHITE)
                                    .strong()
                                    .size(9.5),
                            );
                        });

                    // Sub-name text (e.g. "/ Денди / Famicom")
                    let sub_w = (inner_width - 80.0).max(20.0);
                    ui.add_sized(
                        [sub_w, 14.0],
                        egui::Label::new(
                            RichText::new(subname)
                                .size(9.5)
                                .color(Color32::from_rgb(78, 159, 229)),
                        ).truncate(),
                    );

                    // Year on the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let y = if game.year.is_empty() { "—".to_string() } else { game.year.clone() };
                        ui.label(RichText::new(y).size(10.0).color(theme.text_muted()));
                    });
                });

                ui.add_space(2.0);

                // Rating & Format row
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    ui.label(RichText::new("★").color(theme.secondary_color()).size(11.5));
                    let r = if game.rating > 0.0 { format!("{:.1}", game.rating) } else { "4.8".to_string() };
                    ui.label(RichText::new(r).color(Color32::WHITE).strong().size(11.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let fmt = get_format_label(&game.console_slug);
                        ui.label(RichText::new(fmt).size(10.0).color(theme.text_muted()));
                    });
                });
            });

            let info_interact = ui.interact(
                info_block_resp.response.rect,
                ui.id().with("info_block_click").with(&game.id),
                egui::Sense::click(),
            );
            if info_interact.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if info_interact.on_hover_text("📖 Нажмите, чтобы открыть информацию об игре").clicked() {
                *on_game_clicked = Some(game.clone());
            }

            ui.add_space(6.0);

            // 3. Action Buttons Row: [ ⬇ Скачать ] + [ ♡ Heart ]
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                let heart_w = 28.0;
                let dl_btn_w = (inner_width - heart_w - 6.0).max(60.0);

                // Main Download / Play button
                if game.is_downloaded {
                    let play_resp = Frame::none()
                        .fill(Color32::from_rgb(0, 230, 118))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                        .show(ui, |ui| {
                            ui.set_min_size(Vec2::new(dl_btn_w, 24.0));
                            ui.centered_and_justified(|ui| {
                                ui.label(RichText::new("▶ Играть").color(Color32::BLACK).strong().size(11.5));
                            });
                        });
                    if ui.interact(play_resp.response.rect, ui.id().with("btn_play").with(&game.id), egui::Sense::click()).clicked() {
                        *on_play_clicked = Some(game.clone());
                    }
                } else {
                    let dl_resp = Frame::none()
                        .fill(theme.primary_color())
                        .rounding(Rounding::same(6.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                        .show(ui, |ui| {
                            ui.set_min_size(Vec2::new(dl_btn_w, 24.0));
                            ui.centered_and_justified(|ui| {
                                ui.label(RichText::new("⬇ Скачать").color(Color32::WHITE).strong().size(11.5));
                            });
                        });
                    let dl_interact = ui.interact(dl_resp.response.rect, ui.id().with("btn_dl").with(&game.id), egui::Sense::click());
                    if dl_interact.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if dl_interact.on_hover_text("Выбрать версию ROM для скачивания").clicked() {
                        *on_download_clicked = Some(game.clone());
                    }
                }

                // Heart Favorite button
                let (heart_char, heart_col) = if game.is_favorite {
                    ("♥", theme.tertiary_color())
                } else {
                    ("♡", theme.text_muted())
                };

                let heart_frame = Frame::none()
                    .fill(theme.surface_container())
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0_f32, theme.chip_border_color()))
                    .inner_margin(egui::Margin::symmetric(6.0, 5.0));

                let heart_resp = heart_frame.show(ui, |ui| {
                    ui.set_min_size(Vec2::new(heart_w, 24.0));
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new(heart_char).color(heart_col).size(13.0));
                    });
                });
                let heart_interact = ui.interact(heart_resp.response.rect, ui.id().with("fav_heart").with(&game.id), egui::Sense::click());
                if heart_interact.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let tooltip = if game.is_favorite { "Удалить из избранного" } else { "Добавить в избранное" };
                if heart_interact.on_hover_text(tooltip).clicked() {
                    *on_favorite_toggled = Some((game.clone(), !game.is_favorite));
                }
            });
        });
    });
}
