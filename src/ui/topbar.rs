use crate::models::CatalogCategory;
use crate::theme::ThemePreset;
use crate::ui::{SortOption, ViewMode};
use egui::{RichText, ScrollArea, Ui};

pub fn render_topbar(
    ui: &mut Ui,
    search_query: &mut String,
    is_searching: bool,
    search_results_count: usize,
    search_platforms: &[(String, String, usize)],
    search_platform_filter: &mut Option<String>,
    categories: &[CatalogCategory],
    selected_category: &mut String,
    view_mode: &mut ViewMode,
    sort_option: &mut SortOption,
    is_loading: bool,
    theme: ThemePreset,
    on_search_triggered: &mut bool,
    on_clear_search: &mut bool,
    on_category_changed: &mut bool,
    on_refresh_clicked: &mut bool,
) {
    let is_search_mode = !search_query.trim().is_empty();

    ui.vertical(|ui| {
        // Line 1: Search input, Search buttons, View Mode, Sort, Refresh
        ui.horizontal(|ui| {
            ui.label("🔍");
            let search_edit = ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("Поиск по всей библиотеке Emu-Land (Mario, Sonic…)")
                    .desired_width(320.0),
            );

            // Press Enter to search
            if search_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                *on_search_triggered = true;
            }

            if !search_query.trim().is_empty() {
                if ui.button("Найти").clicked() {
                    *on_search_triggered = true;
                }
                if ui.small_button("✖").on_hover_text("Очистить поиск").clicked() {
                    search_query.clear();
                    *on_clear_search = true;
                }
            }

            ui.add_space(12.0);

            // View Mode buttons
            ui.label("Вид:");
            let grid_active = *view_mode == ViewMode::Grid;
            let table_active = *view_mode == ViewMode::Table;

            if ui
                .selectable_label(
                    grid_active,
                    RichText::new("⊞ Сетка")
                        .color(if grid_active { theme.primary_color() } else { theme.text_color() }),
                )
                .clicked()
            {
                *view_mode = ViewMode::Grid;
            }

            if ui
                .selectable_label(
                    table_active,
                    RichText::new("☰ Таблица")
                        .color(if table_active { theme.primary_color() } else { theme.text_color() }),
                )
                .clicked()
            {
                *view_mode = ViewMode::Table;
            }

            ui.add_space(12.0);

            // Sort Dropdown
            egui::ComboBox::from_id_salt("sort_combo")
                .selected_text(format!("Сортировка: {}", sort_option.label()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(sort_option, SortOption::Default, SortOption::Default.label());
                    ui.selectable_value(sort_option, SortOption::RatingDesc, SortOption::RatingDesc.label());
                    ui.selectable_value(sort_option, SortOption::TitleAsc, SortOption::TitleAsc.label());
                    ui.selectable_value(sort_option, SortOption::TitleDesc, SortOption::TitleDesc.label());
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(if is_loading { "⏳ Загрузка..." } else { "⟳ Обновить" })
                    .clicked()
                    && !is_loading
                {
                    *on_refresh_clicked = true;
                }
            });
        });

        ui.add_space(6.0);

        // Line 2: Either Global Search Results & Platform Filter Chips, or Normal Console Categories
        if is_search_mode {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // 1. Status / count label
                if is_searching {
                    ui.label(
                        RichText::new("⏳ Поиск по всей библиотеке Emu-Land.net…")
                            .color(theme.accent_color())
                            .strong(),
                    );
                } else {
                    ui.label(
                        RichText::new(format!("Найдено: {} игр", search_results_count))
                            .color(theme.primary_color())
                            .strong(),
                    );
                }

                // 2. Reserve width for reset button, give middle space to scroll area
                let reset_btn_w = 120.0;
                let scroll_w = (ui.available_width() - reset_btn_w - 12.0).max(60.0);

                if search_results_count > 0 && !search_platforms.is_empty() {
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(scroll_w, 24.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ScrollArea::horizontal()
                                .id_salt("search_platforms_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 6.0;
                                        let all_selected = search_platform_filter.is_none();
                                        let mut all_text = RichText::new(format!("Все платформы ({})", search_results_count));
                                        if all_selected {
                                            all_text = all_text.color(theme.primary_color()).strong();
                                        }
                                        if ui.selectable_label(all_selected, all_text).clicked() {
                                            *search_platform_filter = None;
                                        }

                                        for (slug, display_name, count) in search_platforms {
                                            let is_selected = search_platform_filter
                                                .as_ref()
                                                .map(|s| s == slug)
                                                .unwrap_or(false);
                                            let mut chip_text = RichText::new(format!("{} ({})", display_name, count));
                                            if is_selected {
                                                chip_text = chip_text.color(theme.primary_color()).strong();
                                            }
                                            if ui.selectable_label(is_selected, chip_text).clicked() {
                                                if is_selected {
                                                    *search_platform_filter = None;
                                                } else {
                                                    *search_platform_filter = Some(slug.clone());
                                                }
                                            }
                                        }
                                    });
                                });
                        },
                    );
                }

                // 3. Reset button on the right
                if ui.button("Сбросить поиск").clicked() {
                    search_query.clear();
                    *on_clear_search = true;
                }
            });
        } else if !categories.is_empty() {
            ScrollArea::horizontal()
                .id_salt("topbar_categories_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for cat in categories {
                            let is_selected = *selected_category == cat.key;
                            let mut text = RichText::new(&cat.label);
                            if is_selected {
                                text = text.color(theme.primary_color()).strong();
                            }

                            let btn = ui.selectable_label(is_selected, text);
                            if btn.clicked() && !is_selected {
                                *selected_category = cat.key.clone();
                                *on_category_changed = true;
                            }
                        }
                    });
                });
        }
    });
}
