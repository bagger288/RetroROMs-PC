use crate::archive::{extract_single_rom, scan_zip_rom_entries};
use crate::models::{GameCard, RomFileVersion, ZipExtractionRequest};
use crate::scraper::{EmuLandClient, USER_AGENT};
use futures_util::StreamExt;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started {
        record_id: i64,
        game_id: String,
        file_name: String,
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
        game_id: String,
        file_path: String,
        message: String,
    },
    RequiresSelection(ZipExtractionRequest),
    Failed {
        record_id: i64,
        game_id: String,
        error: String,
    },
}

#[derive(Clone)]
pub struct DownloadManager {
    client: reqwest::Client,
    scraper: EmuLandClient,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadManager {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_default();

        Self {
            client,
            scraper: EmuLandClient::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start_download(
        &self,
        record_id: i64,
        game: GameCard,
        version: Option<RomFileVersion>,
        target_dir: PathBuf,
        auto_unpack_zip: bool,
        delete_zip_after_unpack: bool,
        tx: mpsc::UnboundedSender<DownloadEvent>,
        cancelled: Arc<AtomicBool>,
    ) {
        let scraper = self.scraper.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            let initial_download_url = version
                .as_ref()
                .map(|v| v.download_url.clone())
                .unwrap_or_else(|| game.download_url.clone());

            let display_name = version
                .as_ref()
                .map(|v| v.name.clone())
                .unwrap_or_else(|| format!("{}.zip", game.title));

            let _ = tx.send(DownloadEvent::Started {
                record_id,
                game_id: game.id.clone(),
                file_name: display_name.clone(),
            });

            // 1. Resolve direct download URL if needed (e.g. from Emu-Land 302 redirect)
            let direct_url = match scraper
                .resolve_direct_download_url(&initial_download_url, &game.download_url)
                .await
            {
                Ok(url) => url,
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Failed {
                        record_id,
                        game_id: game.id.clone(),
                        error: format!("Ошибка разрешения ссылки: {}", e),
                    });
                    return;
                }
            };

            // 2. Prepare temp file path
            let temp_dir = std::env::temp_dir();
            let timestamp = chrono::Utc::now().timestamp_millis();
            let temp_file_path = temp_dir.join(format!("retroms_{}_{}.tmp", record_id, timestamp));

            // 3. Initiate stream request
            let resp = match client
                .get(&direct_url)
                .header("Referer", &game.download_url)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Failed {
                        record_id,
                        game_id: game.id.clone(),
                        error: format!("Ошибка сетевого запроса: {}", e),
                    });
                    return;
                }
            };

            if !resp.status().is_success() {
                let _ = tx.send(DownloadEvent::Failed {
                    record_id,
                    game_id: game.id.clone(),
                    error: format!("Сервер вернул статус HTTP {}", resp.status()),
                });
                return;
            }

            // Verify content type isn't HTML
            if let Some(ct) = resp.headers().get(reqwest::header::CONTENT_TYPE) {
                if let Ok(ct_str) = ct.to_str() {
                    if ct_str.contains("text/html") {
                        let _ = tx.send(DownloadEvent::Failed {
                            record_id,
                            game_id: game.id.clone(),
                            error: "Сервер вернул HTML страницу вместо файла (защита от скачивания)".into(),
                        });
                        return;
                    }
                }
            }

            let total_size = resp.content_length().unwrap_or(0);
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

            let mut stream = resp.bytes_stream();
            let mut downloaded: u64 = 0;
            let mut last_emit = Instant::now();
            let mut bytes_since_last_emit: u64 = 0;
            let mut current_speed: u64 = 0;

            while let Some(chunk_res) = stream.next().await {
                if cancelled.load(Ordering::Relaxed) {
                    let _ = fs::remove_file(&temp_file_path);
                    let _ = tx.send(DownloadEvent::Failed {
                        record_id,
                        game_id: game.id.clone(),
                        error: "Загрузка отменена пользователем".into(),
                    });
                    return;
                }

                let chunk = match chunk_res {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = fs::remove_file(&temp_file_path);
                        let _ = tx.send(DownloadEvent::Failed {
                            record_id,
                            game_id: game.id.clone(),
                            error: format!("Ошибка при чтении потока: {}", e),
                        });
                        return;
                    }
                };

                if let Err(e) = file.write_all(&chunk) {
                    let _ = fs::remove_file(&temp_file_path);
                    let _ = tx.send(DownloadEvent::Failed {
                        record_id,
                        game_id: game.id.clone(),
                        error: format!("Ошибка записи на диск: {}", e),
                    });
                    return;
                }

                let chunk_len = chunk.len() as u64;
                downloaded += chunk_len;
                bytes_since_last_emit += chunk_len;

                // Throttle progress updates to ~10 times per second
                if last_emit.elapsed().as_millis() >= 100 {
                    let elapsed_sec = last_emit.elapsed().as_secs_f64();
                    if elapsed_sec > 0.0 {
                        current_speed = (bytes_since_last_emit as f64 / elapsed_sec) as u64;
                    }
                    bytes_since_last_emit = 0;
                    last_emit = Instant::now();

                    let percent = if total_size > 0 {
                        (downloaded as f32 / total_size as f32) * 100.0
                    } else {
                        0.0
                    };

                    let _ = tx.send(DownloadEvent::Progress {
                        record_id,
                        downloaded_bytes: downloaded,
                        total_bytes: total_size,
                        speed_bytes_sec: current_speed,
                        percent,
                    });
                }
            }

            drop(file);

            // 4. Archive inspection & unpacking
            let console_folder = &game.console_slug;
            let console_dest_dir = target_dir.join(console_folder);
            let _ = fs::create_dir_all(&console_dest_dir);

            // Check if temp file is a ZIP archive
            let is_zip = {
                if let Ok(mut f) = File::open(&temp_file_path) {
                    use std::io::Read;
                    let mut magic = [0u8; 4];
                    f.read_exact(&mut magic).is_ok() && magic == [0x50, 0x4B, 0x03, 0x04]
                } else {
                    false
                }
            };

            if is_zip {
                match scan_zip_rom_entries(&temp_file_path) {
                    Ok(entries) if entries.len() == 1 && auto_unpack_zip => {
                        // Exactly 1 ROM entry & auto-unpack enabled
                        let single_entry = &entries[0];
                        match extract_single_rom(
                            &temp_file_path,
                            &console_dest_dir,
                            &single_entry.entry_name,
                            delete_zip_after_unpack,
                        ) {
                            Ok(extracted_path) => {
                                let _ = tx.send(DownloadEvent::Completed {
                                    record_id,
                                    game_id: game.id.clone(),
                                    file_path: extracted_path.to_string_lossy().to_string(),
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
                    }
                    Ok(entries) if entries.len() > 1 => {
                        // Multi-file archive (e.g. GoodSet / translations / hacks) -> request user selection!
                        let _ = tx.send(DownloadEvent::RequiresSelection(ZipExtractionRequest {
                            game: game.clone(),
                            temp_zip_path: temp_file_path.to_string_lossy().to_string(),
                            console_folder_name: console_folder.to_string(),
                            target_directory: console_dest_dir.to_string_lossy().to_string(),
                            record_id,
                            entries,
                        }));
                    }
                    _ => {
                        // If auto-unpack is disabled or no valid ROM entries, move the ZIP directly
                        let clean_title = sanitize_filename(&display_name);
                        let final_path = console_dest_dir.join(clean_title);
                        if let Err(_e) = fs::rename(&temp_file_path, &final_path) {
                            // Try copy if across filesystems
                            if let Err(copy_err) = fs::copy(&temp_file_path, &final_path) {
                                let _ = tx.send(DownloadEvent::Failed {
                                    record_id,
                                    game_id: game.id.clone(),
                                    error: format!("Не удалось сохранить файл: {}", copy_err),
                                });
                                return;
                            }
                            let _ = fs::remove_file(&temp_file_path);
                        }

                        let _ = tx.send(DownloadEvent::Completed {
                            record_id,
                            game_id: game.id.clone(),
                            file_path: final_path.to_string_lossy().to_string(),
                            message: format!("Сохранено в {}", final_path.display()),
                        });
                    }
                }
            } else {
                // Not a ZIP archive, move directly to console folder
                let clean_name = sanitize_filename(&display_name);
                let final_path = console_dest_dir.join(clean_name);
                if let Err(_) = fs::rename(&temp_file_path, &final_path) {
                    let _ = fs::copy(&temp_file_path, &final_path);
                    let _ = fs::remove_file(&temp_file_path);
                }

                let _ = tx.send(DownloadEvent::Completed {
                    record_id,
                    game_id: game.id.clone(),
                    file_path: final_path.to_string_lossy().to_string(),
                    message: format!("Сохранено в {}", final_path.display()),
                });
            }
        });
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect()
}
