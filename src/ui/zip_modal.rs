use crate::models::ZipExtractionRequest;
use crate::theme::ThemePreset;
use egui::{Align2, RichText, ScrollArea, Window};

pub fn render_zip_modal(
    ctx: &egui::Context,
    request: &mut Option<ZipExtractionRequest>,
    theme: ThemePreset,
    on_extract_confirmed: &mut Option<(ZipExtractionRequest, Vec<String>)>,
) {
    if request.is_none() {
        return;
    }

    let req = request.as_mut().unwrap();
    let mut is_open = true;
    let mut close_requested = false;

    Window::new(format!("📦 Распаковка архива: {}", req.game.title))
        .open(&mut is_open)
        .resizable(true)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([580.0, 480.0])
        .min_width(450.0)
        .min_height(350.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Архив содержит несколько файлов РОМов.").strong());
                ui.label("Отметьте файлы, которые вы хотите распаковать в папку консоли:");
                ui.add_space(8.0);

                // Filter Buttons
                ui.horizontal(|ui| {
                    if ui.button("Выбрать все").clicked() {
                        for e in &mut req.entries {
                            e.is_selected = true;
                        }
                    }
                    if ui.button("Снять все").clicked() {
                        for e in &mut req.entries {
                            e.is_selected = false;
                        }
                    }
                    if ui.button("Только RUS / USA").clicked() {
                        for e in &mut req.entries {
                            let n = e.display_name.to_lowercase();
                            e.is_selected = n.contains("(r)")
                                || n.contains("rus")
                                || n.contains("(u)")
                                || n.contains("usa");
                        }
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // File list with checkboxes
                ScrollArea::vertical()
                    .id_salt("zip_entries_scroll")
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for entry in &mut req.entries {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut entry.is_selected, "");
                                ui.label(&entry.display_name);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&entry.formatted_size).weak().size(11.0),
                                        );
                                    },
                                );
                            });
                        }
                    });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // Bottom confirmation actions
                ui.horizontal(|ui| {
                    let selected_count = req.entries.iter().filter(|e| e.is_selected).count();
                    let can_extract = selected_count > 0;

                    let extract_btn = ui.add_enabled(
                        can_extract,
                        egui::Button::new(
                            RichText::new(format!("Распаковать выбранное ({})", selected_count))
                                .color(theme.primary_color())
                                .strong(),
                        ),
                    );

                    if extract_btn.clicked() {
                        let selected_names: Vec<String> = req
                            .entries
                            .iter()
                            .filter(|e| e.is_selected)
                            .map(|e| e.entry_name.clone())
                            .collect();

                        *on_extract_confirmed = Some((req.clone(), selected_names));
                    }

                    if ui.button("Отмена").clicked() {
                        close_requested = true;
                    }
                });
            });
        });

    if !is_open || close_requested {
        *request = None;
    }
}
