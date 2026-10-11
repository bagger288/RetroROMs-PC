use crate::models::CatalogCategory;
use crate::theme::ThemePreset;
use crate::ui::{SortOption, ViewMode};
use egui::{Color32, Frame, RichText, Rounding, ScrollArea, Stroke, Ui};

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

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        // 1. Search Bar (Rounded pill shape)
        let search_frame = Frame::none()
            .fill(theme.surface_container())
            .rounding(Rounding::same(16.0))
            .stroke(Stroke::new(1.0_f32, theme.chip_border_color()))
            .inner_margin(egui::Margin::symmetric(10.0, 5.0));

        search_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(13.0).color(theme.text_secondary()));
                let search_edit = ui.add(
                    egui::TextEdit::singleline(search_query)
                        .hint_text("Поиск по всей библиотеке EmuLand (Mario, Sonic...)")
                        .desired_width(320.0)
                        .frame(false),
                );

                if search_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    *on_search_triggered = true;
                }

                if !search_query.trim().is_empty() {
                    if ui
                        .small_button("✖")
                        .on_hover_text("Очистить поиск")
                        .clicked()
                    {
                        search_query.clear();
                        *on_clear_search = true;
                    }
                }
            });
        });

        ui.add_space(4.0);

        // 2. View Mode Switcher ("Вид: [⊞ Сетка] [☰ Таблица]")
        ui.label(RichText::new("Вид:").color(theme.text_secondary()).size(12.5));

        // Grid Button
        let grid_active = *view_mode == ViewMode::Grid;
        let grid_bg = if grid_active { theme.primary_color() } else { theme.surface_container() };
        let grid_fg = if grid_active { Color32::WHITE } else { theme.text_secondary() };
        let grid_stroke = if grid_active { Stroke::NONE } else { Stroke::new(1.0_f32, theme.chip_border_color()) };

        let grid_resp = Frame::none()
            .fill(grid_bg)
            .rounding(Rounding::same(14.0))
            .stroke(grid_stroke)
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.label(RichText::new("⊞ Сетка").color(grid_fg).strong().size(12.0));
            });
        if ui.interact(grid_resp.response.rect, ui.id().with("btn_view_grid"), egui::Sense::click()).clicked() {
            *view_mode = ViewMode::Grid;
        }

        // Table Button
        let table_active = *view_mode == ViewMode::Table;
        let table_bg = if table_active { theme.primary_color() } else { theme.surface_container() };
        let table_fg = if table_active { Color32::WHITE } else { theme.text_secondary() };
        let table_stroke = if table_active { Stroke::NONE } else { Stroke::new(1.0_f32, theme.chip_border_color()) };

        let table_resp = Frame::none()
            .fill(table_bg)
            .rounding(Rounding::same(14.0))
            .stroke(table_stroke)
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.label(RichText::new("☰ Таблица").color(table_fg).strong().size(12.0));
            });
        if ui.interact(table_resp.response.rect, ui.id().with("btn_view_table"), egui::Sense::click()).clicked() {
            *view_mode = ViewMode::Table;
        }

        ui.add_space(4.0);

        // 3. Sort ComboBox (Rounded pill)
        let sort_label = match sort_option {
            SortOption::Default => "По умолчанию",
            SortOption::RatingDesc => "По рейтингу",
            SortOption::TitleAsc => "А-Я По названию",
            SortOption::TitleDesc => "Я-А По названию",
        };

        egui::ComboBox::from_id_salt("sort_combo")
            .selected_text(format!("Сортировка: {}", sort_label))
            .show_ui(ui, |ui| {
                ui.selectable_value(sort_option, SortOption::Default, "По умолчанию");
                ui.selectable_value(sort_option, SortOption::RatingDesc, "★ По рейтингу");
                ui.selectable_value(sort_option, SortOption::TitleAsc, "А-Я По названию");
                ui.selectable_value(sort_option, SortOption::TitleDesc, "Я-А По названию");
            });

        ui.add_space(4.0);

        // 4. Middle Section: Quick Categories or Search Platforms
        if is_search_mode {
            if is_searching {
                ui.label(RichText::new("⏳ Поиск…").color(theme.secondary_color()).size(12.0));
            } else {
                ui.label(RichText::new(format!("Найдено: {} игр", search_results_count)).color(theme.text_secondary()).size(12.0));
            }

            if search_results_count > 0 && !search_platforms.is_empty() {
                ScrollArea::horizontal()
                    .id_salt("search_platform_chips_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let all_sel = search_platform_filter.is_none();
                            let all_bg = if all_sel { theme.primary_color() } else { theme.surface_container() };
                            let all_fg = if all_sel { Color32::WHITE } else { theme.text_secondary() };
                            let all_chip = Frame::none()
                                .fill(all_bg)
                                .rounding(Rounding::same(14.0))
                                .stroke(Stroke::new(1.0_f32, theme.chip_border_color()))
                                .inner_margin(egui::Margin::symmetric(9.0, 5.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(format!("Все ({})", search_results_count)).color(all_fg).size(11.5));
                                });
                            if ui.interact(all_chip.response.rect, ui.id().with("all_plat_filter"), egui::Sense::click()).clicked() {
                                *search_platform_filter = None;
                            }

                            for (slug, display_name, count) in search_platforms {
                                let is_sel = search_platform_filter.as_ref().map(|s| s == slug).unwrap_or(false);
                                let p_bg = if is_sel { theme.primary_color() } else { theme.surface_container() };
                                let p_fg = if is_sel { Color32::WHITE } else { theme.text_secondary() };
                                let p_chip = Frame::none()
                                    .fill(p_bg)
                                    .rounding(Rounding::same(14.0))
                                    .stroke(Stroke::new(1.0_f32, theme.chip_border_color()))
                                    .inner_margin(egui::Margin::symmetric(9.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(format!("{} ({})", display_name, count)).color(p_fg).size(11.5));
                                    });
                                if ui.interact(p_chip.response.rect, ui.id().with("p_chip").with(slug), egui::Sense::click()).clicked() {
                                    if is_sel {
                                        *search_platform_filter = None;
                                    } else {
                                        *search_platform_filter = Some(slug.clone());
                                    }
                                }
                            }
                        });
                    });
            }
        } else {
            // Standard category chips matching screenshot:
            // "📁 Прочее", "🎮 Homebrew", "🔥 Популярные", "★ Лучшие"
            let quick_cats = [
                ("misc", "📁 Прочее"),
                ("homebrew", "🎮 Homebrew"),
                ("top", "🔥 Популярные"),
                ("best", "★ Лучшие"),
            ];

            for (cat_key, cat_title) in quick_cats {
                let is_active = selected_category == cat_key;
                let chip_bg = if is_active { theme.primary_color() } else { theme.surface_container() };
                let chip_fg = if is_active { Color32::WHITE } else { theme.text_secondary() };
                let chip_stroke = if is_active { Stroke::NONE } else { Stroke::new(1.0_f32, theme.chip_border_color()) };

                let chip_resp = Frame::none()
                    .fill(chip_bg)
                    .rounding(Rounding::same(14.0))
                    .stroke(chip_stroke)
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(cat_title).color(chip_fg).strong().size(12.0));
                    });

                if ui.interact(chip_resp.response.rect, ui.id().with("chip").with(cat_key), egui::Sense::click()).clicked() {
                    if *selected_category != cat_key {
                        *selected_category = cat_key.to_string();
                        *on_category_changed = true;
                    }
                }
            }

            // Scrollable remaining categories (e.g. alphabet letters A-Z) if any
            let remaining_cats: Vec<&CatalogCategory> = categories
                .iter()
                .filter(|c| c.key != "misc" && c.key != "homebrew" && c.key != "top" && c.key != "best")
                .collect();

            if !remaining_cats.is_empty() {
                ScrollArea::horizontal()
                    .id_salt("extra_cats_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for c in remaining_cats {
                                let is_active = selected_category == &c.key;
                                let chip_bg = if is_active { theme.primary_color() } else { theme.surface_container() };
                                let chip_fg = if is_active { Color32::WHITE } else { theme.text_secondary() };
                                let chip_stroke = if is_active { Stroke::NONE } else { Stroke::new(1.0_f32, theme.chip_border_color()) };

                                let chip_resp = Frame::none()
                                    .fill(chip_bg)
                                    .rounding(Rounding::same(14.0))
                                    .stroke(chip_stroke)
                                    .inner_margin(egui::Margin::symmetric(9.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(&c.label).color(chip_fg).size(11.5));
                                    });

                                if ui.interact(chip_resp.response.rect, ui.id().with("chip_extra").with(&c.key), egui::Sense::click()).clicked() {
                                    if selected_category != &c.key {
                                        *selected_category = c.key.clone();
                                        *on_category_changed = true;
                                    }
                                }
                            }
                        });
                    });
            }
        }

        // 5. Far Right: Refresh Button
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let ref_bg = theme.surface_container();
            let ref_fg = theme.text_color();

            let ref_resp = Frame::none()
                .fill(ref_bg)
                .rounding(Rounding::same(14.0))
                .stroke(Stroke::new(1.0_f32, theme.chip_border_color()))
                .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                .show(ui, |ui| {
                    let text = if is_loading { "⏳ Загрузка..." } else { "⟳ Обновить" };
                    ui.label(RichText::new(text).color(ref_fg).size(12.0));
                });

            if ui.interact(ref_resp.response.rect, ui.id().with("btn_refresh"), egui::Sense::click()).clicked() && !is_loading {
                *on_refresh_clicked = true;
            }
        });
    });
}
