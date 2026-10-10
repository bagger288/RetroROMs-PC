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
    on_play_clicked: &mut Option<GameCard>,
    on_reveal_clicked: &mut Option<String>,
    on_view_image: &mut Option<(String, String)>,
) {
    let mut is_open = game.is_some();
    if !is_open {
        return;
    }

    let g = game.as_ref().unwrap().clone();

    Window::new(format!("🎮 {}", g.title))
        .open(&mut is_open)
        .resizable(true)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([720.0, 600.0])
        .min_width(500.0)
        .max_width(760.0)
        .min_height(400.0)
        .max_height(720.0)
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .id_salt("game_detail_scroll")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Cover / Large Screenshot
                        if let Some(cover) = &g.cover_url {
                            let img_resp = ui.add(
                                egui::Image::new(cover)
                                    .fit_to_exact_size(Vec2::new(180.0, 200.0))
                                    .rounding(Rounding::same(8.0))
                                    .sense(egui::Sense::click()),
                            );
                            if img_resp.on_hover_text("🔍 Нажмите, чтобы увеличить обложку").clicked() {
                                *on_view_image = Some((format!("Обложка: {}", g.title), cover.clone()));
                            }
                        }

                        ui.add_space(16.0);

                        // Metadata Column
                        ui.vertical(|ui| {
                            ui.heading(
                                RichText::new(&g.title)
                                    .color(theme.primary_color())
                                    .strong(),
                            );
                            if let Some(orig) = &g.original_title {
                                ui.label(RichText::new(orig).weak().italics());
                            }

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Платформа:").strong());
                                ui.label(&g.console_name);
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Жанр:").strong());
                                ui.label(&g.genre);
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Год выпуска:").strong());
                                ui.label(&g.year);
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Рейтинг:").strong());
                                ui.label(
                                    RichText::new(format!("★ {:.1} / 5.0", g.rating))
                                        .color(Color32::from_rgb(255, 204, 0))
                                        .strong(),
                                );
                            });

                            ui.add_space(12.0);

                            // Action buttons
                            ui.horizontal(|ui| {
                                if g.is_downloaded {
                                    if ui
                                        .button(
                                            RichText::new("▶ Играть в эмуляторе")
                                                .color(Color32::from_rgb(0, 230, 118))
                                                .strong(),
                                        )
                                        .clicked()
                                    {
                                        *on_play_clicked = Some(g.clone());
                                    }

                                    if let Some(path) = &g.local_file_path {
                                        if ui.button("📁 Показать в папке").clicked() {
                                            *on_reveal_clicked = Some(path.clone());
                                        }
                                    }
                                }
                            });
                        });
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Screenshot Gallery (if multiple)
                    if g.screenshot_urls.len() > 1 {
                        ui.label(RichText::new("Скриншоты").strong());
                        ui.add_space(4.0);
                        ScrollArea::horizontal()
                            .id_salt("game_screenshots_scroll")
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for (ss_idx, ss_url) in g.screenshot_urls.iter().enumerate() {
                                        let ss_resp = ui.add(
                                            egui::Image::new(ss_url)
                                                .fit_to_exact_size(Vec2::new(160.0, 110.0))
                                                .rounding(Rounding::same(4.0))
                                                .sense(egui::Sense::click()),
                                        );
                                        if ss_resp.on_hover_text("🔍 Нажмите, чтобы рассмотреть скриншот").clicked() {
                                            *on_view_image = Some((format!("Скриншот #{}: {}", ss_idx + 1, g.title), ss_url.clone()));
                                        }
                                        ui.add_space(8.0);
                                    }
                                });
                            });
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);
                    }

                    // Description
                    ui.label(RichText::new("📖 Описание игры").strong().size(13.0));
                    ui.add_space(4.0);
                    if !g.description.is_empty() {
                        ui.label(RichText::new(&g.description).size(12.0));
                    } else if is_loading_versions {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(RichText::new("Загрузка описания игры с Emu-Land.net...").weak());
                        });
                    } else {
                        ui.label(
                            RichText::new("Описание игры подгружается с Emu-Land.net или отсутствует. Вы можете выбрать нужную версию ROM ниже для скачивания.")
                                .weak()
                                .italics(),
                        );
                    }
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Available ROM Versions Table
                    ui.label(RichText::new("Доступные версии ROM / Архивы").strong().size(14.0));
                    ui.add_space(4.0);

                    if is_loading_versions {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Запрос доступных ревизий с Emu-Land (act=getmfl)...");
                        });
                    } else if rom_versions.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Версии не найдены. Можно скачать основной архив:");
                            if ui
                                .button(RichText::new("⬇ Скачать основной ROM").color(theme.primary_color()))
                                .clicked()
                            {
                                *on_download_version = Some((
                                    g.clone(),
                                    RomFileVersion {
                                        fid: "main".into(),
                                        name: format!("{}.zip", g.title),
                                        size: g.file_size.clone(),
                                        category: "Основные".into(),
                                        download_url: g.download_url.clone(),
                                        region_or_type: "USA".into(),
                                    },
                                ));
                            }
                        });
                    } else {
                        let dl_btn_w = 100.0;
                        let badge_w = 54.0;

                        for ver in rom_versions {
                            let card_frame = Frame::none()
                                .fill(theme.card_bg_color())
                                .rounding(Rounding::same(6.0))
                                .stroke(Stroke::new(1.0, theme.primary_color().gamma_multiply(0.2)))
                                .inner_margin(egui::Margin::symmetric(10.0, 7.0));

                            card_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 10.0;

                                    let (badge_bg, badge_fg) = match ver.region_or_type.as_str() {
                                        "RUS" => (Color32::from_rgb(0, 100, 160), Color32::from_rgb(180, 235, 255)),
                                        "USA" => (Color32::from_rgb(30, 120, 60), Color32::from_rgb(200, 255, 210)),
                                        "EUR" => (Color32::from_rgb(150, 110, 20), Color32::from_rgb(255, 240, 190)),
                                        "JAP" => (Color32::from_rgb(160, 50, 30), Color32::from_rgb(255, 210, 200)),
                                        "HACK" => (Color32::from_rgb(110, 40, 140), Color32::from_rgb(240, 200, 255)),
                                        "GOODSET" => (Color32::from_rgb(160, 30, 90), Color32::from_rgb(255, 200, 230)),
                                        _ => (Color32::from_rgb(50, 55, 70), Color32::from_white_alpha(200)),
                                    };

                                    Frame::none()
                                        .fill(badge_bg)
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                [badge_w, 16.0],
                                                egui::Label::new(
                                                    RichText::new(&ver.region_or_type)
                                                        .color(badge_fg)
                                                        .size(10.5)
                                                        .strong(),
                                                ),
                                            );
                                        });

                                    let text_width = (ui.available_width() - dl_btn_w - 20.0).max(180.0);
                                    ui.allocate_ui_with_layout(
                                        Vec2::new(text_width, 34.0),
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            let fn_label = ui.add_sized(
                                                [text_width, 18.0],
                                                egui::Label::new(
                                                    RichText::new(&ver.name)
                                                        .size(12.5)
                                                        .strong()
                                                        .color(theme.text_color()),
                                                )
                                                .truncate(),
                                            );
                                            fn_label.on_hover_text(&ver.name);

                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 6.0;
                                                if !ver.category.is_empty() {
                                                    ui.label(
                                                        RichText::new(&ver.category)
                                                            .size(10.5)
                                                            .color(Color32::from_white_alpha(150)),
                                                    );
                                                    ui.label(RichText::new("•").size(10.0).weak());
                                                }
                                                ui.label(
                                                    RichText::new(format!("Размер: {}", ver.size))
                                                        .size(10.5)
                                                        .color(Color32::from_white_alpha(180)),
                                                );
                                            });
                                        },
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let dl_btn = ui.add_sized(
                                            [dl_btn_w, 28.0],
                                            egui::Button::new(
                                                RichText::new("⬇ Скачать")
                                                    .color(theme.primary_color())
                                                    .strong()
                                                    .size(11.5),
                                            ),
                                        );
                                        if dl_btn.on_hover_text("Скачать эту версию ROM").clicked() {
                                            *on_download_version = Some((g.clone(), ver.clone()));
                                        }
                                    });
                                });
                            });
                            ui.add_space(4.0);
                        }
                    }

                    ui.add_space(20.0);
                });
        });

    if !is_open {
        *game = None;
    }
}
