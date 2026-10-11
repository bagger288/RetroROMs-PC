use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Clone)]
pub struct ImageCacheManager {
    cache_dir: PathBuf,
    download_tx: UnboundedSender<(String, String)>, // (game_id, url)
}

impl ImageCacheManager {
    pub fn new() -> (Self, UnboundedReceiver<String>) {
        let cache_dir = if let Some(cd) = dirs::cache_dir() {
            cd.join("RetroROMs").join("covers")
        } else {
            PathBuf::from("cache").join("covers")
        };
        let _ = std::fs::create_dir_all(&cache_dir);

        let (download_tx, mut download_rx) = unbounded_channel::<(String, String)>();
        let (completed_tx, completed_rx) = unbounded_channel::<String>();

        let dir_clone = cache_dir.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) RetroROMs-Desktop/1.0")
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default();

            while let Some((game_id, url)) = download_rx.recv().await {
                if url.trim().is_empty() {
                    continue;
                }

                let safe_name = sanitize_filename(&game_id);
                let ext = if url.ends_with(".png") {
                    "png"
                } else if url.ends_with(".webp") {
                    "webp"
                } else {
                    "jpg"
                };
                let dest = dir_clone.join(format!("{}.{}", safe_name, ext));

                if dest.exists() && dest.metadata().map(|m| m.len() > 0).unwrap_or(false) {
                    let _ = completed_tx.send(game_id);
                    continue;
                }

                match client.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(bytes) = resp.bytes().await {
                            if !bytes.is_empty() {
                                let _ = std::fs::write(&dest, &bytes);
                                let _ = completed_tx.send(game_id);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        (
            Self {
                cache_dir,
                download_tx,
            },
            completed_rx,
        )
    }

    pub fn get_cached_image(&self, game_id: &str) -> Option<slint::Image> {
        let safe_name = sanitize_filename(game_id);
        for ext in &["jpg", "png", "webp", "jpeg"] {
            let p = self.cache_dir.join(format!("{}.{}", safe_name, ext));
            if p.exists() && p.metadata().map(|m| m.len() > 0).unwrap_or(false) {
                if let Ok(img) = slint::Image::load_from_path(&p) {
                    return Some(img);
                }
            }
        }
        None
    }

    pub fn queue_download(&self, game_id: &str, url: &str) {
        if self.get_cached_image(game_id).is_none() && !url.trim().is_empty() {
            let _ = self.download_tx.send((game_id.to_string(), url.to_string()));
        }
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
