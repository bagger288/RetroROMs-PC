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
    let mut is_open = game.is_some();
    if !is_open {
        return;
    }

    let g = game.as_ref().unwrap().clone();

    Window::new(format!("💾 Выбор версии ROM: {}", g.title))
        .open(&mut is_open)
        .resizable(true)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(ctx.screen_rect().center())
        .default_size([680.0, 480.0])
        .min_width(520.0)
        .min_height(350.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Header info
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&g.console_name)
                            .color(theme.primary_color())
                            .strong(),
                    );
                    ui.label(RichText::new("•").weak());
                    ui.label(RichText::new(&g.genre).weak());
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

                if is_loading_versions {
                    ui.vertical_centered(|ui| {
                        ui.add_space(30.0);
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Запрос доступных ревизий игры с Emu-Land.net (act=getmfl)...")
                                .size(13.0)
                                .color(theme.primary_color()),
                        );
                        ui.label(RichText::new("Обычно это занимает 1-2 секунды").weak().size(11.0));
                        ui.add_space(30.0);
                    });
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
                        ui.add_space(12.0);

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
                            is_open = false;
                        }
                        ui.add_space(20.0);
                    });
                } else {
                    // Versions list
                    ScrollArea::vertical()
                        .id_salt("rom_versions_modal_scroll")
                        .max_height(360.0)
                        .show(ui, |ui| {
                            egui::Grid::new("rom_versions_modal_grid")
                                .num_columns(5)
                                .spacing([12.0, 8.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Файл ROM").strong());
                                    ui.label(RichText::new("Регион / Тип").strong());
                                    ui.label(RichText::new("Категория").strong());
                                    ui.label(RichText::new("Размер").strong());
                                    ui.label(RichText::new("Действие").strong());
                                    ui.end_row();

                                    for ver in rom_versions {
                                        // Filename
                                        ui.label(RichText::new(&ver.name).strong());

                                        // Region Badge
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
                                            .inner_margin(Vec2::new(6.0, 2.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    RichText::new(&ver.region_or_type)
                                                        .color(badge_fg)
                                                        .size(10.0)
                                                        .strong(),
                                                );
                                            });

                                        // Category
                                        ui.label(RichText::new(&ver.category).size(11.0).weak());

                                        // Size
                                        ui.label(RichText::new(&ver.size).size(11.0));

                                        // Download button
                                        if ui
                                            .button(
                                                RichText::new("⬇ Скачать")
                                                    .color(theme.primary_color())
                                                    .strong(),
                                            )
                                            .clicked()
                                        {
                                            *on_download_version = Some((g.clone(), ver.clone()));
                                            is_open = false;
                                        }

                                        ui.end_row();
                                    }
                                });
                        });
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Закрыть").clicked() {
                        is_open = false;
                    }
                });
            });
        });

    if !is_open {
        *game = None;
    }
}
