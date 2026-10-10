use crate::models::{GameCard, RomFileVersion};
use crate::theme::ThemePreset;
use egui::{Align2, Color32, Frame, RichText, Rounding, ScrollArea, Stroke, Vec2, Window};

pub fn render_rom_versions_modal(
    ctx: &egui::Context,
    game: &mut Option<GameCard>,
    rom_versions: &[RomFileVersion],
    is_loading_versions: bool,
    theme: ThemePreset,
    on_download_version: &mut Option<(GameCard, RomFileVersion)>,
) {
    if game.is_none() {
        return;
    }

    let g = game.as_ref().unwrap().clone();
    let mut is_open = true;
    let mut close_requested = false;

    // Constrain window size with strict max_width so it can NEVER auto-expand across the screen
    Window::new(format!("💾 Выбор версии ROM: {}", g.title))
        .open(&mut is_open)
        .resizable(true)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([700.0, 500.0])
        .min_width(540.0)
        .max_width(740.0)
        .min_height(360.0)
        .max_height(600.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // 1. Header info
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&g.console_name)
                            .color(theme.primary_color())
                            .strong(),
                    );
                    if !g.genre.is_empty() {
                        ui.label(RichText::new("•").weak());
                        ui.label(RichText::new(&g.genre).weak());
                    }
                    if !g.year.is_empty() {
                        ui.label(RichText::new("•").weak());
                        ui.label(RichText::new(&g.year).weak());
                    }
                });

                ui.add_space(4.0);
                ui.label(
                    RichText::new("Выберите интересующую ревизию, язык или хак для скачивания:")
                        .size(12.0)
                        .color(Color32::from_white_alpha(180)),
                );
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // 2. Loading state
                if is_loading_versions {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.spinner();
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new("Запрос доступных ревизий игры с Emu-Land.net (act=getmfl)...")
                                .size(13.0)
                                .color(theme.primary_color())
                                .strong(),
                        );
                        ui.label(RichText::new("Обычно это занимает 1-2 секунды").weak().size(11.0));
                        ui.add_space(40.0);
                    });
                // 3. Empty state (fallback to main ROM)
                } else if rom_versions.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("Отдельный список ревизий для этой игры не найден на сервере.")
                                .size(13.0)
                                .color(Color32::from_rgb(255, 180, 50)),
                        );
                        ui.add_space(8.0);
                        ui.label("Вы можете скачать стандартный архив игры напрямую:");
                        ui.add_space(14.0);

                        let main_name = if g.title.ends_with(".zip") {
                            g.title.clone()
                        } else {
                            format!("{}.zip", g.title)
                        };

                        if ui
                            .button(
                                RichText::new(format!("⬇ Скачать основной ROM ({})", main_name))
                                    .color(theme.primary_color())
                                    .strong()
                                    .size(13.0),
                            )
                            .clicked()
                        {
                            *on_download_version = Some((
                                g.clone(),
                                RomFileVersion {
                                    fid: "main".into(),
                                    name: main_name,
                                    size: g.file_size.clone(),
                                    category: "Основные".into(),
                                    download_url: g.download_url.clone(),
                                    region_or_type: "ROM".into(),
                                },
                            ));
                            close_requested = true;
                        }
                        ui.add_space(20.0);
                    });
                // 4. Versions list
                } else {
                    ScrollArea::vertical()
                        .id_salt("rom_versions_modal_scroll")
                        .auto_shrink([false, false])
                        .max_height(360.0)
                        .show(ui, |ui| {
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

                                        // Region Badge Pill (Fixed width, distinct colors)
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

                                        // Middle section: Filename + Category/Size
                                        let middle_w = (ui.available_width() - dl_btn_w - 20.0).max(180.0);
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(middle_w, 34.0),
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| {
                                                let fn_label = ui.add_sized(
                                                    [middle_w, 18.0],
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

                                        // Download button docked to right edge
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
                                                close_requested = true;
                                            }
                                        });
                                    });
                                });

                                ui.add_space(4.0);
                            }
                        });
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Закрыть").clicked() {
                        close_requested = true;
                    }
                });
            });
        });

    if !is_open || close_requested {
        *game = None;
    }
}
