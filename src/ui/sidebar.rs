use crate::models::ConsoleInfo;
use crate::theme::ThemePreset;
use crate::ui::NavTab;
use egui::{Color32, Frame, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};

pub fn render_sidebar(
    ui: &mut Ui,
    current_tab: &mut NavTab,
    consoles: &mut [ConsoleInfo],
    selected_console_idx: &mut usize,
    active_dl_count: usize,
    fav_count: usize,
    theme: ThemePreset,
    console_changed: &mut bool,
    manage_clicked: &mut bool,
) {
    ui.vertical(|ui| {
        // App Title Banner
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("🕹").size(24.0));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("RetroROMs")
                            .color(theme.primary_color())
                            .strong()
                            .size(16.0),
                    );
                    ui.label(
                        RichText::new("v1.0")
                            .color(theme.secondary_color())
                            .size(10.0)
                            .strong(),
                    );
                });
                ui.label(
                    RichText::new("Emu-Land Catalog & ROM Manager")
                        .color(theme.text_secondary())
                        .size(10.0),
                );
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        // 1. Primary Navigation Tabs (Android M3 style navigation)
        let tabs = [
            (NavTab::Catalog, "🎮", "Каталог игр", None),
            (
                NavTab::Downloads,
                "⬇",
                "Загрузки",
                if active_dl_count > 0 { Some(active_dl_count) } else { None },
            ),
            (
                NavTab::Favorites,
                "⭐",
                "Избранное",
                if fav_count > 0 { Some(fav_count) } else { None },
            ),
            (NavTab::Settings, "⚙", "Настройки", None),
        ];

        for (tab, icon, label, badge) in tabs {
            let is_selected = *current_tab == tab;
            let bg_color = if is_selected {
                theme.primary_color().gamma_multiply(0.25)
            } else {
                Color32::TRANSPARENT
            };
            let stroke = if is_selected {
                Stroke::new(1.0_f32, theme.primary_color().gamma_multiply(0.7))
            } else {
                Stroke::NONE
            };

            let item_frame = Frame::none()
                .fill(bg_color)
                .rounding(Rounding::same(8.0))
                .stroke(stroke)
                .inner_margin(egui::Margin::symmetric(10.0, 7.0));

            let resp = item_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).size(14.0));
                    ui.add_space(4.0);
                    let text_color = if is_selected {
                        theme.primary_color()
                    } else {
                        theme.text_color()
                    };
                    ui.label(
                        RichText::new(label)
                            .color(text_color)
                            .strong()
                            .size(13.0),
                    );

                    if let Some(count) = badge {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            Frame::none()
                                .fill(theme.secondary_color())
                                .rounding(Rounding::same(10.0))
                                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(count.to_string())
                                            .color(Color32::BLACK)
                                            .strong()
                                            .size(10.0),
                                    );
                                });
                        });
                    }
                });
            });

            let interact_resp = ui.interact(resp.response.rect, ui.id().with(label), egui::Sense::click());
            if interact_resp.clicked() {
                *current_tab = tab;
            }
            if interact_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            ui.add_space(2.0);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        // 2. Consoles List Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("ПЛАТФОРМЫ")
                    .color(theme.text_secondary())
                    .strong()
                    .size(11.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(RichText::new("⚙ Список").size(10.5))
                    .on_hover_text("Настройка порядка и видимости консолей")
                    .clicked()
                {
                    *manage_clicked = true;
                }
            });
        });
        ui.add_space(4.0);

        // 3. Scrollable List of Consoles
        ScrollArea::vertical()
            .id_salt("sidebar_consoles_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let categories = [
                    ("home", "ДОМАШНИЕ КОНСОЛИ"),
                    ("portable", "ПОРТАТИВНЫЕ"),
                    ("classic", "КЛАССИКА"),
                ];

                for (cat_key, cat_title) in categories {
                    let cat_consoles: Vec<(usize, &ConsoleInfo)> = consoles
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_enabled && c.category == cat_key)
                        .collect();

                    if cat_consoles.is_empty() {
                        continue;
                    }

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(cat_title)
                            .color(theme.primary_color().gamma_multiply(0.7))
                            .size(10.0)
                            .strong(),
                    );
                    ui.add_space(2.0);

                    for (original_idx, c) in cat_consoles {
                        let is_selected = *current_tab == NavTab::Catalog && *selected_console_idx == original_idx;
                        let bg_color = if is_selected {
                            theme.primary_color().gamma_multiply(0.22)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let stroke = if is_selected {
                            Stroke::new(1.0_f32, theme.primary_color().gamma_multiply(0.6))
                        } else {
                            Stroke::NONE
                        };

                        let item_frame = Frame::none()
                            .fill(bg_color)
                            .rounding(Rounding::same(6.0))
                            .stroke(stroke)
                            .inner_margin(egui::Margin::symmetric(8.0, 5.0));

                        let resp = item_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&c.icon_key).size(13.0));
                                ui.add_space(2.0);

                                let name_color = if is_selected {
                                    theme.primary_color()
                                } else {
                                    theme.text_color()
                                };
                                ui.add_sized(
                                    Vec2::new(110.0, 16.0),
                                    egui::Label::new(
                                        RichText::new(&c.short_name)
                                            .color(name_color)
                                            .strong()
                                            .size(12.0),
                                    )
                                    .truncate(),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(&c.release_year)
                                            .color(theme.text_secondary())
                                            .size(10.0),
                                    );
                                });
                            });
                        });

                        let interact = ui.interact(
                            resp.response.rect,
                            ui.id().with("console_btn").with(&c.slug),
                            egui::Sense::click(),
                        );
                        if interact.clicked() {
                            if *selected_console_idx != original_idx || *current_tab != NavTab::Catalog {
                                *selected_console_idx = original_idx;
                                *current_tab = NavTab::Catalog;
                                *console_changed = true;
                            }
                        }
                        if interact.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        ui.add_space(1.0);
                    }
                }
            });
    });
}
