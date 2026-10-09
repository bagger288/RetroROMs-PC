use crate::models::CatalogCategory;
use crate::theme::ThemePreset;
use crate::ui::{SortOption, ViewMode};
use egui::{RichText, ScrollArea, Ui};

pub fn render_topbar(
    ui: &mut Ui,
    search_query: &mut String,
    categories: &[CatalogCategory],
    selected_category: &mut String,
    view_mode: &mut ViewMode,
    sort_option: &mut SortOption,
    is_loading: bool,
    theme: ThemePreset,
    on_category_changed: &mut bool,
    on_refresh_clicked: &mut bool,
) {
    ui.vertical(|ui| {
        // Line 1: Search, View Mode, Sort, Refresh
        ui.horizontal(|ui| {
            ui.label("🔍");
            let _search_edit = ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("Поиск игры по названию...")
                    .desired_width(280.0),
            );
            if !search_query.is_empty() && ui.small_button("✖").clicked() {
                search_query.clear();
            }

            ui.add_space(16.0);

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

            ui.add_space(16.0);

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

        // Line 2: Horizontal categories list (Emu-Land #pagelist_top)
        if !categories.is_empty() {
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
