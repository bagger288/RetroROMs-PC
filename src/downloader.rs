use crate::archive::{extract_selected_entries, extract_single_rom, scan_zip_rom_entries};
use crate::models::{GameCard, ZipExtractionRequest};
use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started {
        record_id: i64,
        game_id: String,
    },
    Progress {
        record_id: i64,
        downloaded_bytes: u64,
        total_bytes: u64,
        speed_bytes_sec: u64,
        percent: f32,
    },
    Completed {
        record_id: i64,
        local_path: String,
        message: String,
    },
    ZipNeedsSelection {
        request: ZipExtractionRequest,
    },
    Failed {
        record_id: i64,
        game_id: String,
        error: String,
    },
}

#[derive(Clone)]
pub struct DownloadManager {
    client: reqwest::Client,
    tx: UnboundedSender<DownloadEvent>,
}

impl DownloadManager {
    pub fn new(tx: UnboundedSender<DownloadEvent>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client, tx }
    }

    pub fn download_game(
        &self,
        record_id: i64,
        game: GameCard,
        direct_url: String,
        target_dir: PathBuf,
        console_folder_name: String,
        auto_unpack_zip: bool,
        delete_zip: bool,
        cancel_token: Arc<AtomicBool>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let _ = tx.send(DownloadEvent::Started {
                record_id,
                game_id: game.id.clone(),
            });

            let referer = format!(
                "https://www.emu-land.net/{}/{}/{}",
                game.section, game.console_slug,
                crate::scraper::get_game_subpath(&game.console_slug)
            );

            let clean_direct_url = if direct_url.starts_with("//") {
                format!("https:{}", direct_url)
            } else if direct_url.starts_with('/') {
                format!("https://www.emu-land.net{}", direct_url)
            } else {
                direct_url.clone()
            }.replace(' ', "%20");

            let res = client
                .get(&clean_direct_url)
                .header("Referer", &referer)
                .send()
                .await;

            let response = match res {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Failed {
                        record_id,
                        game_id: game.id.clone(),
                        error: format!("Ошибка соединения: {}", e),
                    });
                    return;
                }
            };

            if !response.status().is_success() {
                let _ = tx.send(DownloadEvent::Failed {
                    record_id,
                    game_id: game.id.clone(),
                    error: format!("HTTP статус: {}", response.status()),
                });
                return;
            }

            let total_bytes = response.content_length().unwrap_or(0);
            let temp_dir = std::env::temp_dir();
            let safe_title = game.title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
            let temp_file_path = temp_dir.join(format!("retroms_{}_{}.tmp", record_id, safe_title));

            let mut file = match File::create(&temp_file_path) {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Failed {
                        record_id,
                        game_id: game.id.clone(),
                        error: format!("Не удалось создать временный файл: {}", e),
                    });
                    return;
                }
            };

            let mut stream = response.bytes_stream();
            let mut downloaded_bytes: u64 = 0;
            let mut last_update = Instant::now();
            let mut bytes_since_last_update: u64 = 0;
            let mut speed_bytes_sec: u64 = 0;

            while let Some(chunk_result) = stream.next().await {
                if cancel_token.load(Ordering::Relaxed) {
                    drop(file);
                    let _ = std::fs::remove_file(&temp_file_path);
                    return;
                }

                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        drop(file);
                        let _ = std::fs::remove_file(&temp_file_path);
                        let _ = tx.send(DownloadEvent::Failed {
                            record_id,
                            game_id: game.id.clone(),
                            error: format!("Ошибка потока: {}", e),
                        });
                        return;
                    }
                };

                let len = chunk.len() as u64;
                if let Err(e) = file.write_all(&chunk) {
                    drop(file);
                    let _ = std::fs::remove_file(&temp_file_path);
                    let _ = tx.send(DownloadEvent::Failed {
                        record_id,
                        game_id: game.id.clone(),
                        error: format!("Ошибка записи: {}", e),
                    });
                    return;
                }

                downloaded_bytes += len;
                bytes_since_last_update += len;

                if last_update.elapsed().as_millis() >= 200 {
                    let elapsed_sec = last_update.elapsed().as_secs_f64();
                    if elapsed_sec > 0.0 {
                        speed_bytes_sec = (bytes_since_last_update as f64 / elapsed_sec) as u64;
                    }
                    bytes_since_last_update = 0;
                    last_update = Instant::now();

                    let percent = if total_bytes > 0 {
                        ((downloaded_bytes as f64 / total_bytes as f64) * 100.0) as f32
                    } else {
                        0.0
                    };

                    let _ = tx.send(DownloadEvent::Progress {
                        record_id,
                        downloaded_bytes,
                        total_bytes,
                        speed_bytes_sec,
                        percent,
                    });
                }
            }

            let _ = file.flush();
            drop(file);

            // Final progress update
            let percent = if total_bytes > 0 { 100.0 } else { 0.0 };
            let _ = tx.send(DownloadEvent::Progress {
                record_id,
                downloaded_bytes,
                total_bytes,
                speed_bytes_sec: 0,
                percent,
            });

            // Post-download archive handling
            let final_console_dir = target_dir.join(&console_folder_name);
            let _ = std::fs::create_dir_all(&final_console_dir);

            match scan_zip_rom_entries(&temp_file_path) {
                Ok(entries) => {
                    if entries.is_empty() {
                        // Not a ZIP or contains no ROM entries: move file as is
                        let target_file = final_console_dir.join(format!("{}.zip", safe_title));
                        if let Err(_e) = std::fs::rename(&temp_file_path, &target_file) {
                            let _ = std::fs::copy(&temp_file_path, &target_file);
                            let _ = std::fs::remove_file(&temp_file_path);
                        }
                        let _ = tx.send(DownloadEvent::Completed {
                            record_id,
                            local_path: target_file.to_string_lossy().to_string(),
                            message: format!("Сохранено: {}.zip", safe_title),
                        });
                    } else if entries.len() == 1 {
                        let single_entry = &entries[0];
                        if auto_unpack_zip {
                            match extract_single_rom(&temp_file_path, &final_console_dir, &single_entry.entry_name, delete_zip) {
                                Ok(extracted_path) => {
                                    let _ = tx.send(DownloadEvent::Completed {
                                        record_id,
                                        local_path: extracted_path.to_string_lossy().to_string(),
                                        message: format!("Распаковано: {}", single_entry.display_name),
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(DownloadEvent::Failed {
                                        record_id,
                                        game_id: game.id.clone(),
                                        error: format!("Ошибка распаковки: {}", e),
                                    });
                                }
                            }
                        } else {
                            let target_file = final_console_dir.join(format!("{}.zip", safe_title));
                            let _ = std::fs::rename(&temp_file_path, &target_file);
                            let _ = tx.send(DownloadEvent::Completed {
                                record_id,
                                local_path: target_file.to_string_lossy().to_string(),
                                message: format!("Архив сохранен: {}.zip", safe_title),
                            });
                        }
                    } else {
                        // Multi-rom ZIP archive -> Ask user
                        let _ = tx.send(DownloadEvent::ZipNeedsSelection {
                            request: ZipExtractionRequest {
                                game: game.clone(),
                                temp_zip_path: temp_file_path.to_string_lossy().to_string(),
                                console_folder_name,
                                target_directory: target_dir.to_string_lossy().to_string(),
                                record_id,
                                entries,
                            },
                        });
                    }
                }
                Err(_) => {
                    // Not a zip archive, preserve file
                    let target_file = final_console_dir.join(format!("{}.rom", safe_title));
                    let _ = std::fs::rename(&temp_file_path, &target_file);
                    let _ = tx.send(DownloadEvent::Completed {
                        record_id,
                        local_path: target_file.to_string_lossy().to_string(),
                        message: format!("Сохранено: {}", safe_title),
                    });
                }
            }
        });
    }

    pub fn extract_multi_rom_selection(
        &self,
        request: ZipExtractionRequest,
        selected_entries: Vec<String>,
        delete_zip: bool,
    ) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let zip_path = PathBuf::from(&request.temp_zip_path);
            let target_folder = PathBuf::from(&request.target_directory).join(&request.console_folder_name);
            let _ = std::fs::create_dir_all(&target_folder);

            match extract_selected_entries(&zip_path, &selected_entries, &target_folder, delete_zip) {
                Ok(files) => {
                    let first = files.first().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                    let _ = tx.send(DownloadEvent::Completed {
                        record_id: request.record_id,
                        local_path: first,
                        message: format!("Распаковано файлов: {}", files.len()),
                    });
                }
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Failed {
                        record_id: request.record_id,
                        game_id: request.game.id.clone(),
                        error: format!("Ошибка распаковки: {}", e),
                    });
                }
            }
        });
    }
}
