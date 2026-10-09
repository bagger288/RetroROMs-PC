use crate::archive::format_bytes;
use crate::models::{DownloadRecord, DownloadStatus};
use crate::theme::ThemePreset;
use egui::{Color32, ProgressBar, RichText, ScrollArea, Ui};

pub fn render_downloads_view(
    ui: &mut Ui,
    active_downloads: &[DownloadRecord],
    history: &[DownloadRecord],
    theme: ThemePreset,
    on_cancel_download: &mut Option<i64>,
    on_play_file: &mut Option<String>,
    on_reveal_file: &mut Option<String>,
    on_delete_history_item: &mut Option<i64>,
    on_clear_history: &mut bool,
) {
    ScrollArea::vertical()
        .id_salt("downloads_view_scroll")
        .show(ui, |ui| {
            ui.add_space(8.0);
            ui.heading(
                RichText::new("⬇ Загрузки и история")
                    .color(theme.primary_color())
                    .strong(),
            );
            ui.add_space(8.0);

            // Active Downloads Section
            if !active_downloads.is_empty() {
                ui.label(RichText::new("АКТИВНЫЕ ЗАГРУЗКИ").strong().weak());
                ui.add_space(4.0);

                for dl in active_downloads {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&dl.game_title).strong());
                            ui.label(format!("({})", dl.console_name));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("✖ Отменить").clicked() {
                                    *on_cancel_download = Some(dl.id);
                                }
                            });
                        });

                        ui.add_space(4.0);
                        let progress = if dl.total_bytes > 0 {
                            (dl.downloaded_bytes as f32 / dl.total_bytes as f32).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };

                        let speed_str = format_bytes(dl.speed_bytes_sec);
                        let text = format!(
                            "{:.1}% • {} / {} • {}/s",
                            progress * 100.0,
                            format_bytes(dl.downloaded_bytes),
                            format_bytes(dl.total_bytes),
                            speed_str
                        );

                        ui.add(
                            ProgressBar::new(progress)
                                .text(text)
                                .fill(theme.primary_color()),
                        );
                    });
                    ui.add_space(6.0);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);
            }

            // History Section
            ui.horizontal(|ui| {
                ui.label(RichText::new("ИСТОРИЯ ЗАГРУЗОК").strong().weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !history.is_empty() && ui.button("🗑 Очистить историю").clicked() {
                        *on_clear_history = true;
                    }
                });
            });
            ui.add_space(4.0);

            if history.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new("История загрузок пуста").weak());
                });
                return;
            }

            egui::Grid::new("downloads_history_grid")
                .striped(true)
                .min_col_width(80.0)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Игра").strong());
                    ui.label(RichText::new("Платформа").strong());
                    ui.label(RichText::new("Файл").strong());
                    ui.label(RichText::new("Размер").strong());
                    ui.label(RichText::new("Статус").strong());
                    ui.label(RichText::new("Действия").strong());
                    ui.end_row();

                    for item in history {
                        ui.label(&item.game_title);
                        ui.label(&item.console_name);
                        ui.label(&item.file_name);
                        ui.label(format_bytes(item.downloaded_bytes));

                        // Status badge
                        let (status_text, status_color) = match item.status {
                            DownloadStatus::Completed => ("Готово", Color32::from_rgb(0, 230, 118)),
                            DownloadStatus::Failed => ("Ошибка", Color32::from_rgb(255, 82, 82)),
                            DownloadStatus::Downloading => ("Загрузка", Color32::from_rgb(0, 229, 255)),
                            DownloadStatus::Cancelled => ("Отменено", Color32::from_rgb(180, 180, 180)),
                            DownloadStatus::Pending => ("В очереди", Color32::from_rgb(255, 193, 7)),
                        };

                        ui.label(RichText::new(status_text).color(status_color).strong());

                        // Actions
                        ui.horizontal(|ui| {
                            if item.status == DownloadStatus::Completed {
                                if let Some(path) = &item.local_path {
                                    if ui.button("▶ Играть").clicked() {
                                        *on_play_file = Some(path.clone());
                                    }
                                    if ui.button("📁").on_hover_text("Открыть в проводнике").clicked() {
                                        *on_reveal_file = Some(path.clone());
                                    }
                                }
                            }
                            if ui
                                .button("✖")
                                .on_hover_text("Удалить из истории")
                                .clicked()
                            {
                                *on_delete_history_item = Some(item.id);
                            }
                        });
                        ui.end_row();
                    }
                });

            ui.add_space(20.0);
        });
}
