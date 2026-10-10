use crate::config::AppSettings;
use crate::integrations::{pick_download_folder, pick_executable_file};
use crate::models::ConsoleInfo;
use crate::theme::ThemePreset;
use egui::{RichText, ScrollArea, Ui};

pub fn render_settings_view(
    ui: &mut Ui,
    settings: &mut AppSettings,
    consoles: &mut [ConsoleInfo],
    theme: ThemePreset,
    on_consoles_updated: &mut bool,
    on_theme_changed: &mut Option<String>,
) {
    ScrollArea::vertical()
        .id_salt("settings_view_scroll")
        .show(ui, |ui| {
            ui.add_space(8.0);
            ui.heading(
                RichText::new("⚙ Настройки приложения")
                    .color(theme.primary_color())
                    .strong(),
            );
            ui.add_space(12.0);

            // 1. Download Path Section
            ui.group(|ui| {
                ui.label(RichText::new("📁 ПАПКА СОХРАНЕНИЯ ROMs").strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.add(
                        egui::TextEdit::singleline(&mut settings.download_directory)
                            .desired_width(450.0),
                    ).changed() {
                        let _ = settings.save();
                    }
                    if ui.button("Обзор...").clicked() {
                        if let Some(folder) = pick_download_folder() {
                            settings.download_directory = folder.to_string_lossy().to_string();
                            let _ = settings.save();
                            println!("[SETTINGS] Папка сохранения ROMs обновлена: {}", settings.download_directory);
                        }
                    }
                });
                ui.label(
                    RichText::new("В этой папке автоматически создаются подкаталоги по именам консолей (NES, MegaDrive, SNES...)")
                        .weak()
                        .size(11.0),
                );
            });

            ui.add_space(12.0);

            // 2. Unpack Settings
            ui.group(|ui| {
                ui.label(RichText::new("📦 РАСПАКОВКА АРХИВОВ (SMART UNPACK)").strong());
                ui.add_space(4.0);
                if ui.checkbox(
                    &mut settings.auto_unpack_zip,
                    "Автоматически распаковывать однофайловые ZIP-архивы",
                ).changed() {
                    let _ = settings.save();
                }
                if ui.checkbox(
                    &mut settings.delete_zip_after_unpack,
                    "Удалять исходный .zip файл после успешного извлечения РОМа",
                ).changed() {
                    let _ = settings.save();
                }
                ui.label(
                    RichText::new("Многофайловые архивы (GoodSet, сборники переводов) всегда открывают диалог выбора файлов")
                        .weak()
                        .size(11.0),
                );
            });

            ui.add_space(12.0);

            // 3. Theme Preset
            ui.group(|ui| {
                ui.label(RichText::new("🎨 ТЕМА ОФОРМЛЕНИЯ").strong());
                ui.add_space(4.0);
                let themes = [
                    ThemePreset::ArcadeNeon,
                    ThemePreset::Cyberpunk,
                    ThemePreset::Retrowave,
                    ThemePreset::ClassicDark,
                    ThemePreset::GameBoy,
                ];

                ui.horizontal(|ui| {
                    for t in themes {
                        let is_active = settings.active_theme == t.as_str();
                        if ui.selectable_label(is_active, t.display_name()).clicked() {
                            settings.active_theme = t.as_str().to_string();
                            *on_theme_changed = Some(t.as_str().to_string());
                        }
                    }
                });
            });

            ui.add_space(12.0);

            // 4. RetroArch Universal Frontend Integration
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🕹 ЕДИНЫЙ ЭМУЛЯТОР RETROARCH").strong().size(13.0));
                    ui.label(RichText::new("(Multi-System Frontend)").weak().size(11.0));
                });
                ui.add_space(4.0);

                if ui.checkbox(
                    &mut settings.use_retroarch,
                    RichText::new("Использовать RetroArch для запуска всех игр (единый эмулятор)").strong(),
                ).changed() {
                    let _ = settings.save();
                    println!("[SETTINGS] Режим RetroArch: {}", settings.use_retroarch);
                }
                ui.label(
                    RichText::new("Позволяет запускать игры любых платформ через один эмулятор с автозагрузкой ядер libretro.")
                        .weak()
                        .size(11.0),
                );
                ui.add_space(6.0);

                ui.label(RichText::new("Путь к исполняемому файлу RetroArch (retroarch.exe):").size(12.0));
                ui.horizontal(|ui| {
                    if ui.add(
                        egui::TextEdit::singleline(&mut settings.retroarch_path)
                            .desired_width(450.0)
                            .hint_text("C:\\RetroArch-Win64\\retroarch.exe"),
                    ).changed() {
                        let _ = settings.save();
                    }
                    if ui.button("Обзор...").clicked() {
                        if let Some(exe_file) = pick_executable_file("Выберите исполняемый файл RetroArch") {
                            settings.retroarch_path = exe_file.to_string_lossy().to_string();
                            let _ = settings.save();
                            println!("[SETTINGS] Путь к RetroArch установлен: {}", settings.retroarch_path);
                        }
                    }
                });

                ui.add_space(4.0);
                ui.label(RichText::new("Дополнительные параметры запуска RetroArch:").size(12.0));
                if ui.add(
                    egui::TextEdit::singleline(&mut settings.retroarch_args)
                        .desired_width(450.0)
                        .hint_text("-f (для полноэкранного режима)"),
                ).changed() {
                    let _ = settings.save();
                }

                ui.add_space(6.0);
                ui.collapsing("⚙ Настройка ядер libretro для всех систем (Cores)", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("По умолчанию заданы стабильные ядра. RetroArch ищет ядра в подкаталоге cores:")
                                .weak()
                                .size(11.0),
                        );
                        if ui.button("↺ Сбросить все ядра на стандартные").clicked() {
                            settings.retroarch_cores = crate::config::get_default_retroarch_cores();
                            let _ = settings.save();
                        }
                    });
                    ui.add_space(6.0);

                    if settings.retroarch_cores.is_empty() {
                        settings.retroarch_cores = crate::config::get_default_retroarch_cores();
                    }

                    let mut picked_core = None;

                    for console in consoles.iter() {
                        let slug = &console.slug;
                        let default_core = crate::config::get_default_retroarch_cores()
                            .get(slug)
                            .cloned()
                            .unwrap_or_else(|| "core_libretro.dll".to_string());

                        let core_entry = settings
                            .retroarch_cores
                            .entry(slug.clone())
                            .or_insert_with(|| default_core.clone());

                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{} {}", console.icon_key, console.name)).strong().size(12.0));
                            ui.add(egui::TextEdit::singleline(core_entry).desired_width(260.0));
                            if ui.button("Обзор .dll...").clicked() {
                                if let Some(dll_file) = pick_executable_file(&format!("Выберите ядро RetroArch для {}", console.name)) {
                                    let filename = dll_file.file_name().unwrap_or_default().to_string_lossy().to_string();
                                    picked_core = Some((slug.clone(), filename));
                                }
                            }
                        });
                        ui.add_space(2.0);
                    }

                    if let Some((slug, filename)) = picked_core {
                        settings.retroarch_cores.insert(slug, filename);
                        let _ = settings.save();
                    }
                });
            });

            ui.add_space(12.0);

            // 5. Individual Emulators Configuration
            ui.group(|ui| {
                ui.label(RichText::new("🎮 ОТДЕЛЬНЫЕ ЭМУЛЯТОРЫ ДЛЯ КОНСОЛЕЙ (ПК)").strong());
                if settings.use_retroarch {
                    ui.label(
                        RichText::new("ℹ Сейчас активен режим RetroArch. Отдельные эмуляторы используются, если RetroArch отключён.")
                            .color(egui::Color32::from_rgb(255, 204, 0))
                            .size(11.0),
                    );
                } else {
                    ui.label(
                        RichText::new("Укажите пути к отдельным эмуляторам на компьютере:")
                            .weak()
                            .size(11.0),
                    );
                }
                ui.add_space(6.0);

                let key_consoles = [
                    ("dendy", "NES / Dendy (например Mesen.exe / fceux)"),
                    ("genesis", "Sega Genesis (Kega Fusion / blastem)"),
                    ("snes", "Super Nintendo (snes9x / bsnes)"),
                    ("psx", "PlayStation 1 (DuckStation / ePSXe)"),
                    ("gba", "Game Boy Advance (mGBA / VisualBoyAdvance)"),
                    ("n64", "Nintendo 64 (Project64 / Simple64)"),
                ];

                for (slug, desc) in key_consoles {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(slug.to_uppercase()).strong());
                        ui.label(RichText::new(format!("({})", desc)).weak().size(11.0));
                    });

                    ui.horizontal(|ui| {
                        let current_path = settings.emulator_paths.entry(slug.to_string()).or_default();
                        ui.add(egui::TextEdit::singleline(current_path).desired_width(450.0));
                        if ui.button("Обзор...").clicked() {
                            if let Some(exe_file) = pick_executable_file(&format!("Выберите эмулятор для {}", slug)) {
                                *current_path = exe_file.to_string_lossy().to_string();
                            }
                        }
                    });
                    ui.add_space(4.0);
                }
            });

            ui.add_space(12.0);

            // 5. Consoles Visibility and Order
            ui.group(|ui| {
                ui.label(RichText::new("📋 УПРАВЛЕНИЕ ПЛАТФОРМАМИ (33 СИСТЕМЫ)").strong());
                ui.label(
                    RichText::new("Включайте и отключайте системы, меняйте их порядок стрелочками:")
                        .weak()
                        .size(11.0),
                );
                ui.add_space(6.0);

                let mut swap_up = None;
                let mut swap_down = None;

                for (idx, console) in consoles.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let changed = ui.checkbox(&mut console.is_enabled, "").changed();
                        if changed {
                            *on_consoles_updated = true;
                        }

                        if ui.small_button("▲").clicked() && idx > 0 {
                            swap_up = Some(idx);
                        }
                        if ui.small_button("▼").clicked() && idx + 1 < 33 {
                            swap_down = Some(idx);
                        }

                        ui.label(format!("{} {}", console.icon_key, console.name));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(&console.release_year).weak().size(11.0));
                        });
                    });
                }

                if let Some(i) = swap_up {
                    consoles.swap(i, i - 1);
                    *on_consoles_updated = true;
                }
                if let Some(i) = swap_down {
                    consoles.swap(i, i + 1);
                    *on_consoles_updated = true;
                }
            });

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui
                    .button(
                        RichText::new("💾 Сохранить все настройки")
                            .color(theme.primary_color())
                            .strong()
                            .size(13.0),
                    )
                    .clicked()
                {
                    let _ = settings.save();
                    println!("[SETTINGS] Настройки сохранены вручную: папка='{}', RetroArch={}", settings.download_directory, settings.use_retroarch);
                }
            });
            ui.add_space(20.0);
        });
}
