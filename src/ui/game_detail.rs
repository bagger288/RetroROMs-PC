use crate::models::{GameCard, RomFileVersion};
use crate::theme::ThemePreset;
use egui::{Align2, Color32, Frame, RichText, Rounding, ScrollArea, Stroke, Vec2, Window};

pub fn render_game_detail_window(
    ctx: &egui::Context,
    game: &mut Option<GameCard>,
    rom_versions: &[RomFileVersion],
    is_loading_versions: bool,
    theme: ThemePreset,
    on_download_version: &mut Option<(GameCard, RomFileVersion)>,
    on_play: &mut Option<GameCard>,
    on_reveal: &mut Option<String>,
    on_view_image: &mut Option<(String, String)>,
) {
    if game.is_none() {
        return;
    }

    let g = game.as_ref().unwrap().clone();
    let mut is_open = true;

    // Window with bounded width and height to strictly prevent auto-expansion bug
    Window::new(format!("📖 Информация об игре: {}", g.title))
        .open(&mut is_open)
        .resizable(true)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size(Vec2::new(760.0, 580.0))
        .min_width(580.0)
        .max_width(820.0)
        .min_height(400.0)
        .max_height(720.0)
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .id_salt("game_detail_modal_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let content_w = ui.available_width().clamp(300.0, 780.0);

                    // 1. Top Section: Cover + Core Metadata
                    ui.horizontal(|ui| {
                        // Cover container (120x150 px)
                        let cover_size = Vec2::new(120.0, 150.0);
                        let cover_frame = Frame::none()
                            .fill(Color32::from_rgb(15, 20, 30))
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0_f32, theme.card_border_color()))
                            .show(ui, |ui| {
                                ui.set_min_size(cover_size);
                                ui.set_max_size(cover_size);
                                ui.centered_and_justified(|ui| {
                                    if let Some(cover_url) = &g.cover_url {
                                        if !cover_url.is_empty() {
                                            ui.add(
                                                egui::Image::new(cover_url)
                                                    .max_size(cover_size)
                                                    .rounding(Rounding::same(8.0)),
                                            );
                                        } else {
                                            ui.label(RichText::new("🎮").size(36.0));
                                        }
                                    } else {
                                        ui.label(RichText::new("🎮").size(36.0));
                                    }
                                });
                            });

                        let cover_resp = ui.interact(
                            cover_frame.response.rect,
                            ui.id().with("detail_cover_click"),
                            egui::Sense::click(),
                        );
                        if cover_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if cover_resp.on_hover_text("🔍 Нажмите, чтобы открыть в полном размере").clicked() {
                            if let Some(cover_url) = &g.cover_url {
                                if !cover_url.is_empty() {
                                    *on_view_image = Some((format!("Обложка: {}", g.title), cover_url.clone()));
                                }
                            }
                        }

                        ui.add_space(12.0);

                        // Metadata column
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&g.title)
                                    .size(17.0)
                                    .strong()
                                    .color(theme.text_color()),
                            );

                            if let Some(orig) = &g.original_title {
                                if !orig.is_empty() && orig != &g.title {
                                    ui.label(
                                        RichText::new(orig)
                                            .size(11.5)
                                            .color(theme.text_secondary()),
                                    );
                                }
                            }

                            ui.add_space(4.0);

                            // Metadata Badges (Console, Genre, Year, Rating)
                            ui.horizontal_wrapped(|ui| {
                                // Console badge
                                Frame::none()
                                    .fill(theme.primary_color().gamma_multiply(0.25))
                                    .rounding(Rounding::same(4.0))
                                    .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(&g.console_name)
                                                .color(theme.primary_color())
                                                .strong()
                                                .size(11.0),
                                        );
                                    });

                                if !g.genre.is_empty() {
                                    Frame::none()
                                        .fill(theme.surface_container())
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.label(RichText::new(&g.genre).size(11.0).color(theme.text_color()));
                                        });
                                }

                                if !g.year.is_empty() {
                                    Frame::none()
                                        .fill(theme.surface_container())
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("📅 {}", g.year))
                                                    .size(11.0)
                                                    .color(theme.text_secondary()),
                                            );
                                        });
                                }

                                if g.rating > 0.0 {
                                    Frame::none()
                                        .fill(theme.secondary_color().gamma_multiply(0.25))
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("★ {:.1}", g.rating))
                                                    .color(theme.secondary_color())
                                                    .strong()
                                                    .size(11.0),
                                            );
                                        });
                                }

                                if !g.file_size.is_empty() {
                                    Frame::none()
                                        .fill(theme.surface_container())
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("📦 {}", g.file_size))
                                                    .size(11.0)
                                                    .color(theme.text_secondary()),
                                            );
                                        });
                                }
                            });

                            ui.add_space(6.0);

                            if !g.developer.is_empty() || !g.publisher.is_empty() {
                                ui.horizontal(|ui| {
                                    if !g.developer.is_empty() {
                                        ui.label(RichText::new(format!("Разработчик: {}", g.developer)).size(11.0).weak());
                                    }
                                    if !g.publisher.is_empty() {
                                        ui.label(RichText::new(format!("• Издатель: {}", g.publisher)).size(11.0).weak());
                                    }
                                });
                            }

                            ui.add_space(8.0);

                            // Action buttons if downloaded
                            if g.is_downloaded {
                                ui.horizontal(|ui| {
                                    if ui
                                        .button(
                                            RichText::new("▶ Запустить игру")
                                                .color(Color32::BLACK)
                                                .strong()
                                                .size(13.0),
                                        )
                                        .clicked()
                                    {
                                        *on_play = Some(g.clone());
                                    }

                                    if let Some(path) = &g.local_file_path {
                                        if ui.button(RichText::new("📁 Открыть папку").size(12.0)).clicked() {
                                            *on_reveal = Some(path.clone());
                                        }
                                    }
                                });
                            }
                        });
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // 2. Screenshots Gallery (Horizontal scroll)
                    if !g.screenshot_urls.is_empty() {
                        ui.label(RichText::new("Скриншоты:").strong().size(12.0));
                        ui.add_space(4.0);

                        ScrollArea::horizontal()
                            .id_salt("game_detail_screens_scroll")
                            .max_height(140.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let ss_size = Vec2::new(180.0, 120.0);
                                    for (ss_idx, ss_url) in g.screenshot_urls.iter().enumerate() {
                                        let ss_frame = Frame::none()
                                            .fill(Color32::from_rgb(18, 22, 34))
                                            .rounding(Rounding::same(6.0))
                                            .stroke(Stroke::new(1.0_f32, theme.card_border_color()))
                                            .show(ui, |ui| {
                                                ui.set_min_size(ss_size);
                                                ui.set_max_size(ss_size);
                                                ui.centered_and_justified(|ui| {
                                                    ui.add(
                                                        egui::Image::new(ss_url)
                                                            .max_size(ss_size)
                                                            .rounding(Rounding::same(6.0)),
                                                    );
                                                });
                                            });

                                        let ss_resp = ui.interact(
                                            ss_frame.response.rect,
                                            ui.id().with("detail_ss").with(ss_idx),
                                            egui::Sense::click(),
                                        );
                                        if ss_resp.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if ss_resp.on_hover_text("🔍 Нажмите для просмотра в полном окне").clicked() {
                                            *on_view_image = Some((format!("Скриншот #{}: {}", ss_idx + 1, g.title), ss_url.clone()));
                                        }
                                        ui.add_space(6.0);
                                    }
                                });
                            });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);
                    }

                    // 3. Description
                    if !g.description.is_empty() {
                        ui.label(RichText::new("Описание игры:").strong().size(12.0));
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(&g.description)
                                .size(12.0)
                                .color(theme.text_color()),
                        );
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);
                    }

                    // 4. ROM Versions / Downloads
                    ui.label(RichText::new("Доступные версии ROM файла:").strong().size(13.0));
                    ui.add_space(6.0);

                    if is_loading_versions {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.add_space(8.0);
                            ui.label(RichText::new("Загрузка доступных ревизий игры...").color(theme.primary_color()));
                        });
                    } else if rom_versions.is_empty() {
                        // Fallback: Default download button
                        ui.horizontal(|ui| {
                            let dl_name = if g.title.ends_with(".zip") {
                                g.title.clone()
                            } else {
                                format!("{}.zip", g.title)
                            };

                            if ui
                                .button(
                                    RichText::new(format!("⬇ Скачать основной ROM ({})", dl_name))
                                        .color(theme.primary_color())
                                        .strong()
                                        .size(12.5),
                                )
                                .clicked()
                            {
                                *on_download_version = Some((
                                    g.clone(),
                                    RomFileVersion {
                                        fid: "main".into(),
                                        name: dl_name,
                                        size: g.file_size.clone(),
                                        category: "Основные".into(),
                                        download_url: g.download_url.clone(),
                                        region_or_type: "ROM".into(),
                                    },
                                ));
                            }
                        });
                    } else {
                        // Render versions table in bounded container
                        for ver in rom_versions {
                            let card_frame = Frame::none()
                                .fill(theme.surface_container())
                                .rounding(Rounding::same(6.0))
                                .stroke(Stroke::new(1.0_f32, theme.primary_color().gamma_multiply(0.2)))
                                .inner_margin(egui::Margin::symmetric(10.0, 7.0));

                            card_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Region badge
                                    let (badge_bg, badge_fg) = match ver.region_or_type.as_str() {
                                        "RUS" => (Color32::from_rgb(0, 100, 160), Color32::from_rgb(180, 235, 255)),
                                        "USA" => (Color32::from_rgb(30, 120, 60), Color32::from_rgb(200, 255, 210)),
                                        "EUR" => (Color32::from_rgb(150, 110, 20), Color32::from_rgb(255, 240, 190)),
                                        "JAP" => (Color32::from_rgb(160, 50, 30), Color32::from_rgb(255, 210, 200)),
                                        "HACK" => (Color32::from_rgb(110, 40, 140), Color32::from_rgb(240, 200, 255)),
                                        _ => (Color32::from_rgb(50, 55, 70), Color32::from_white_alpha(200)),
                                    };

                                    Frame::none()
                                        .fill(badge_bg)
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                Vec2::new(54.0, 14.0),
                                                egui::Label::new(
                                                    RichText::new(&ver.region_or_type)
                                                        .color(badge_fg)
                                                        .size(10.5)
                                                        .strong(),
                                                ),
                                            );
                                        });

                                    ui.add_space(4.0);

                                    // Filename + Category/Size
                                    let middle_w = (content_w - 200.0).max(150.0);
                                    ui.vertical(|ui| {
                                        ui.add_sized(
                                            Vec2::new(middle_w, 16.0),
                                            egui::Label::new(
                                                RichText::new(&ver.name)
                                                    .size(12.0)
                                                    .strong()
                                                    .color(theme.text_color()),
                                            )
                                            .truncate(),
                                        );
                                        ui.horizontal(|ui| {
                                            if !ver.category.is_empty() {
                                                ui.label(RichText::new(&ver.category).size(10.0).weak());
                                                ui.label(RichText::new("•").size(10.0).weak());
                                            }
                                            ui.label(RichText::new(format!("Размер: {}", ver.size)).size(10.0).weak());
                                        });
                                    });

                                    // Download Button on the right
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui
                                            .button(
                                                RichText::new("⬇ Скачать")
                                                    .color(theme.primary_color())
                                                    .strong()
                                                    .size(12.0),
                                            )
                                            .clicked()
                                        {
                                            *on_download_version = Some((g.clone(), ver.clone()));
                                        }
                                    });
                                });
                            });
                            ui.add_space(3.0);
                        }
                    }

                    ui.add_space(12.0);
                });
        });

    if !is_open {
        *game = None;
    }
}
