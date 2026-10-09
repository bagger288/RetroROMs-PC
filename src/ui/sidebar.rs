use crate::models::ConsoleInfo;
use crate::theme::ThemePreset;
use crate::ui::NavTab;
use egui::{Color32, RichText, ScrollArea, Ui};

pub fn render_sidebar(
    ui: &mut Ui,
    current_tab: &mut NavTab,
    consoles: &mut [ConsoleInfo],
    selected_console_idx: &mut usize,
    active_downloads_count: usize,
    favorites_count: usize,
    theme: ThemePreset,
    on_console_changed: &mut bool,
    on_manage_consoles_clicked: &mut bool,
) {
    ui.vertical(|ui| {
        // App Header / Branding
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🕹 RetroROMs").color(theme.primary_color()).strong());
            ui.label(RichText::new("PC").small().color(theme.secondary_color()));
        });
        ui.label(RichText::new("Emu-Land Catalog & ROM Manager").weak().size(11.0));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Navigation Tabs
        ui.label(RichText::new("НАВИГАЦИЯ").small().weak().strong());
        ui.add_space(4.0);

        let nav_buttons = [
            (NavTab::Catalog, "📚 Каталог игр", 0),
            (NavTab::Downloads, "⬇ Загрузки", active_downloads_count),
            (NavTab::Favorites, "⭐ Избранное", favorites_count),
            (NavTab::Settings, "⚙ Настройки", 0),
        ];

        for (tab, label, badge) in nav_buttons {
            let is_selected = *current_tab == tab;
            let mut text = RichText::new(label);
            if is_selected {
                text = text.color(theme.primary_color()).strong();
            }

            ui.horizontal(|ui| {
                let btn = ui.selectable_label(is_selected, text);
                if btn.clicked() {
                    *current_tab = tab;
                }
                if badge > 0 {
                    ui.label(
                        RichText::new(format!("({})", badge))
                            .small()
                            .color(theme.secondary_color()),
                    );
                }
            });
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        // Consoles Section
        ui.horizontal(|ui| {
            ui.label(RichText::new("🎮 ПЛАТФОРМЫ").small().weak().strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("⚙ Платформы").clicked() {
                    *on_manage_consoles_clicked = true;
                }
            });
        });
        ui.add_space(4.0);

        ScrollArea::vertical()
            .id_salt("sidebar_consoles_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut move_up_idx = None;
                let mut move_down_idx = None;

                for (idx, console) in consoles.iter().enumerate() {
                    if !console.is_enabled {
                        continue;
                    }

                    let is_active = *selected_console_idx == idx;
                    ui.horizontal(|ui| {
                        // Quick reorder buttons on hover / compact
                        if ui.small_button("▲").on_hover_text("Поднять выше").clicked() && idx > 0 {
                            move_up_idx = Some(idx);
                        }
                        if ui.small_button("▼").on_hover_text("Опустить ниже").clicked()
                            && idx + 1 < consoles.len()
                        {
                            move_down_idx = Some(idx);
                        }

                        let icon = &console.icon_key;
                        let label_text = format!("{} {}", icon, console.short_name);
                        let mut text = RichText::new(label_text);

                        if is_active {
                            text = text.color(theme.primary_color()).strong();
                        }

                        let resp = ui.selectable_label(is_active, text);
                        if resp.clicked() {
                            if *selected_console_idx != idx {
                                *selected_console_idx = idx;
                                *current_tab = NavTab::Catalog;
                                *on_console_changed = true;
                            }
                        }

                        // ROMs estimate badge
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(&console.roms_count_estimate)
                                    .size(10.0)
                                    .color(Color32::from_white_alpha(90)),
                            );
                        });
                    });
                }

                if let Some(i) = move_up_idx {
                    consoles.swap(i, i - 1);
                    if *selected_console_idx == i {
                        *selected_console_idx = i - 1;
                    } else if *selected_console_idx == i - 1 {
                        *selected_console_idx = i;
                    }
                }
                if let Some(i) = move_down_idx {
                    consoles.swap(i, i + 1);
                    if *selected_console_idx == i {
                        *selected_console_idx = i + 1;
                    } else if *selected_console_idx == i + 1 {
                        *selected_console_idx = i;
                    }
                }
            });
    });
}
