use eframe::egui;
use egui::{CentralPanel, RichText, SidePanel, TopBottomPanel};
use retroms_desktop::config::AppSettings;
use retroms_desktop::db::Database;
use retroms_desktop::downloader::{self, DownloadEvent, DownloadManager};
use retroms_desktop::integrations::{launch_emulator, launch_retroarch, reveal_in_file_explorer};
use retroms_desktop::models::{
    ConsoleInfo, DownloadRecord, DownloadStatus, GameCard, RomFileVersion, ZipExtractionRequest,
};
use retroms_desktop::scraper::EmuLandClient;
use retroms_desktop::theme::ThemePreset;
use retroms_desktop::ui::catalog_grid::render_catalog_grid;
use retroms_desktop::ui::catalog_table::render_catalog_table;
use retroms_desktop::ui::downloads_view::render_downloads_view;
use retroms_desktop::ui::favorites_view::render_favorites_view;
use retroms_desktop::ui::game_detail::render_game_detail_window;
use retroms_desktop::ui::image_viewer_modal::{render_image_viewer_modal, ImageViewerState};
use retroms_desktop::ui::rom_versions_modal::render_rom_versions_modal;
use retroms_desktop::ui::settings_view::render_settings_view;
use retroms_desktop::ui::sidebar::render_sidebar;
use retroms_desktop::ui::topbar::render_topbar;
use retroms_desktop::ui::zip_modal::render_zip_modal;
use retroms_desktop::ui::{NavTab, SortOption, ViewMode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    println!("============================================================");
    println!("  RetroROMs Desktop v1.0.0 (Emu-Land Catalog & ROM Manager)");
    println!("============================================================");
    println!("[LOG] Консоль запущена. Все логи приложения выводятся сюда.");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([880.0, 560.0])
            .with_title("RetroROMs — Emu-Land Catalog & ROM Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "RetroROMs Desktop",
        native_options,
        Box::new(|cc| {
            // Install egui image loaders so network and file images load seamlessly
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(RetroRomsApp::new(cc)))
        }),
    )
}

pub struct RetroRomsApp {
    // Database & Settings
    db: Database,
    settings: AppSettings,
    theme: ThemePreset,

    // Navigation & View
    current_tab: NavTab,
    view_mode: ViewMode,
    sort_option: SortOption,

    // Consoles
    consoles: Vec<ConsoleInfo>,
    selected_console_idx: usize,

    // Catalog state
    catalog_games: Vec<GameCard>,
    categories: Vec<retroms_desktop::models::CatalogCategory>,
    selected_category: String,
    current_page: usize,
    total_pages: usize,
    is_loading_games: bool,
    status_message: String,

    // Search
    search_query: String,
    search_results: Vec<GameCard>,
    is_searching: bool,
    last_executed_search: String,
    search_platform_filter: Option<String>,
    search_tx: tokio::sync::mpsc::UnboundedSender<(String, Result<Vec<GameCard>, String>)>,
    search_rx: UnboundedReceiver<(String, Result<Vec<GameCard>, String>)>,
    last_query_typed_at: Option<std::time::Instant>,
    last_query_seen: String,

    // Favorites
    favorite_games: Vec<GameCard>,

    // Downloads
    download_manager: DownloadManager,
    download_rx: UnboundedReceiver<DownloadEvent>,
    active_downloads: HashMap<i64, DownloadRecord>,
    download_tokens: HashMap<i64, Arc<AtomicBool>>,
    download_history: Vec<DownloadRecord>,

    // Overlays / Modals
    selected_game_for_detail: Option<GameCard>,
    selected_game_for_versions: Option<GameCard>,
    rom_versions: Vec<RomFileVersion>,
    is_loading_versions: bool,
    image_viewer_state: Option<ImageViewerState>,
    zip_extraction_request: Option<ZipExtractionRequest>,

    // Network Client
    scraper: EmuLandClient,
}

fn strip_rom_tags(stem: &str) -> String {
    let mut result = String::new();
    let mut depth_paren = 0;
    let mut depth_bracket = 0;
    for c in stem.chars() {
        match c {
            '(' => depth_paren += 1,
            ')' => {
                if depth_paren > 0 {
                    depth_paren -= 1;
                }
            }
            '[' => depth_bracket += 1,
            ']' => {
                if depth_bracket > 0 {
                    depth_bracket -= 1;
                }
            }
            _ => {
                if depth_paren == 0 && depth_bracket == 0 {
                    result.push(c);
                }
            }
        }
    }
    result.trim().to_string()
}

fn normalize_title_for_rom_match(title: &str) -> String {
    let t = title.trim();
    let t = if let Some(stripped) = t.strip_suffix(", The") {
        format!("The {}", stripped)
    } else if let Some(stripped) = t.strip_suffix(", the") {
        format!("The {}", stripped)
    } else if let Some(stripped) = t.strip_suffix(", A") {
        format!("A {}", stripped)
    } else if let Some(stripped) = t.strip_suffix(", An") {
        format!("An {}", stripped)
    } else {
        t.to_string()
    };
    t.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

impl RetroRomsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let db = Database::open().expect("Failed to initialize SQLite database");
        let settings = AppSettings::load();
        let theme = ThemePreset::from_str(&settings.active_theme);
        let view_mode = if settings.view_mode == "Table" {
            ViewMode::Table
        } else {
            ViewMode::Grid
        };

        let consoles = db.get_consoles().unwrap_or_else(|_| retroms_desktop::models::get_default_consoles());
        let (tx, rx) = unbounded_channel();
        let (search_tx, search_rx) = unbounded_channel();
        let download_manager = DownloadManager::new(tx);
        let scraper = EmuLandClient::new();

        let selected_console_idx = if let Some(slug) = &settings.last_console_slug {
            consoles
                .iter()
                .position(|c| c.is_enabled && c.slug == *slug)
                .or_else(|| consoles.iter().position(|c| c.is_enabled))
                .unwrap_or(0)
        } else {
            consoles.iter().position(|c| c.is_enabled).unwrap_or(0)
        };

        let mut app = Self {
            db,
            settings,
            theme,
            current_tab: NavTab::Catalog,
            view_mode,
            sort_option: SortOption::Default,
            consoles,
            selected_console_idx,
            catalog_games: Vec::new(),
            categories: Vec::new(),
            selected_category: "top".to_string(),
            current_page: 1,
            total_pages: 1,
            is_loading_games: false,
            status_message: "Готово к работе".to_string(),
            search_query: String::new(),
            search_results: Vec::new(),
            is_searching: false,
            last_executed_search: String::new(),
            search_platform_filter: None,
            search_tx,
            search_rx,
            last_query_typed_at: None,
            last_query_seen: String::new(),
            favorite_games: Vec::new(),
            download_manager,
            download_rx: rx,
            active_downloads: HashMap::new(),
            download_tokens: HashMap::new(),
            download_history: Vec::new(),
            selected_game_for_detail: None,
            selected_game_for_versions: None,
            rom_versions: Vec::new(),
            is_loading_versions: false,
            image_viewer_state: None,
            zip_extraction_request: None,
            scraper,
        };

        app.reload_favorites();
        app.reload_downloads_history();
        app.trigger_load_games();
        app.sync_download_status();

        app
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    pub fn reload_favorites(&mut self) {
        if let Ok(favs) = self.db.get_favorite_games() {
            self.favorite_games = favs;
        }
        self.sync_download_status();
    }

    pub fn reload_downloads_history(&mut self) {
        if let Ok(history) = self.db.get_download_history() {
            self.download_history = history;
        }
    }

    pub fn sync_download_status(&mut self) {
        if let Ok(history) = self.db.get_download_history() {
            self.download_history = history;
        }

        let download_dir = std::path::PathBuf::from(&self.settings.download_directory);

        // Map console slug -> folder_name (e.g. "dendy" -> "NES")
        let console_folder_map: std::collections::HashMap<String, String> = self
            .consoles
            .iter()
            .map(|c| (c.slug.clone(), c.folder_name.clone()))
            .collect();

        // 1. Map of completed downloads from DB
        // Prioritize records whose file actually exists AND is inside current download_dir!
        let mut completed_in_dir: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut completed_anywhere: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for r in &self.download_history {
            if r.status == DownloadStatus::Completed {
                if let Some(path_str) = &r.local_path {
                    let p = std::path::Path::new(path_str);
                    if p.exists() {
                        let console_title_key = format!("{}:{}", r.console_slug, r.game_title.to_lowercase());
                        if p.starts_with(&download_dir) {
                            completed_in_dir.entry(r.game_id.clone()).or_insert_with(|| path_str.clone());
                            completed_in_dir.entry(console_title_key.clone()).or_insert_with(|| path_str.clone());
                        }
                        completed_anywhere.entry(r.game_id.clone()).or_insert_with(|| path_str.clone());
                        completed_anywhere.entry(console_title_key).or_insert_with(|| path_str.clone());
                    }
                }
            }
        }

        const VALID_EXTS: &[&str] = &[
            "nes", "sfc", "smc", "bin", "gen", "md", "smd", "gba", "gb", "gbc",
            "n64", "z64", "v64", "nds", "pce", "iso", "cue", "chd", "zip", "7z", "rom",
        ];

        let update_card = |g: &mut GameCard| {
            let g_console_title_key = format!("{}:{}", g.console_slug, g.title.to_lowercase());

            // 1. Priority: Check DB completed downloads that are inside current download_dir (exact game ID or console + title)
            if let Some(path) = completed_in_dir.get(&g.id).or_else(|| completed_in_dir.get(&g_console_title_key)) {
                g.is_downloaded = true;
                g.local_file_path = Some(path.clone());
                return;
            }

            // 2. Check DB completed downloads anywhere on disk
            if let Some(path) = completed_anywhere.get(&g.id).or_else(|| completed_anywhere.get(&g_console_title_key)) {
                g.is_downloaded = true;
                g.local_file_path = Some(path.clone());
                return;
            }

            // 3. Fallback: Search directly on disk in configured download directories
            let folder_name = console_folder_map.get(&g.console_slug).cloned().unwrap_or_else(|| "ROMs".to_string());
            let console_folder = g.console_name.replace(['/', '\\'], "_");
            let mut target_subfolders = Vec::new();
            if !folder_name.is_empty() {
                target_subfolders.push(download_dir.join(&folder_name));
            }
            if !console_folder.is_empty() {
                target_subfolders.push(download_dir.join(&console_folder));
            }
            if !g.console_slug.is_empty() {
                target_subfolders.push(download_dir.join(&g.console_slug));
            }
            target_subfolders.push(download_dir.clone());

            let raw_title = g.title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
            let safe_title = raw_title.trim_end_matches('.').trim().to_string();
            let title_clean = normalize_title_for_rom_match(&g.title);

            for target_subfolder in &target_subfolders {
                if target_subfolder.exists() {
                    for ext in VALID_EXTS {
                        let candidate1 = target_subfolder.join(format!("{}.{}", safe_title, ext));
                        if candidate1.exists() {
                            g.is_downloaded = true;
                            g.local_file_path = Some(candidate1.to_string_lossy().to_string());
                            return;
                        }
                    }
                    if let Ok(entries) = std::fs::read_dir(target_subfolder) {
                        for entry in entries.flatten() {
                            let entry_path = entry.path();
                            if entry_path.is_file() {
                                let ext = entry_path
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("")
                                    .to_lowercase();
                                if !VALID_EXTS.contains(&ext.as_str()) {
                                    continue;
                                }

                                if let Some(stem) = entry_path.file_stem().and_then(|n| n.to_str()) {
                                    let stem_without_tags = strip_rom_tags(stem);
                                    let stem_clean = normalize_title_for_rom_match(&stem_without_tags);
                                    let raw_stem_clean = normalize_title_for_rom_match(stem);

                                    // Exact equality check prevents false positives from prefix matching
                                    if !title_clean.is_empty() && (stem_clean == title_clean || raw_stem_clean == title_clean) {
                                        g.is_downloaded = true;
                                        g.local_file_path = Some(entry_path.to_string_lossy().to_string());
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            g.is_downloaded = false;
            g.local_file_path = None;
        };

        for g in &mut self.catalog_games {
            update_card(g);
        }
        for g in &mut self.search_results {
            update_card(g);
        }
        for g in &mut self.favorite_games {
            update_card(g);
        }
        if let Some(g) = &mut self.selected_game_for_detail {
            update_card(g);
        }
    }

    pub fn current_console(&self) -> Option<&ConsoleInfo> {
        self.consoles.get(self.selected_console_idx)
    }

    pub fn trigger_load_games(&mut self) {
        self.trigger_load_games_sync();
    }

    pub fn execute_site_search(&mut self, query: &str) {
        let clean = query.trim().to_string();
        if clean.is_empty() {
            self.search_results.clear();
            self.is_searching = false;
            self.search_platform_filter = None;
            self.last_executed_search.clear();
            self.set_status("Поиск очищен");
            return;
        }

        self.last_executed_search = clean.clone();
        self.is_searching = true;
        self.search_platform_filter = None;
        self.set_status(format!("Поиск на Emu-Land.net по запросу «{}»...", clean));

        let scraper = self.scraper.clone();
        let tx = self.search_tx.clone();
        let q = clean.clone();

        tokio::spawn(async move {
            let res = scraper.search_games_site_wide(&q).await.map_err(|e| e.to_string());
            let _ = tx.send((q, res));
        });
    }

    pub fn open_game_detail(&mut self, game: &GameCard) {
        let mut full_game = game.clone();
        self.is_loading_versions = true;
        self.rom_versions.clear();
        self.selected_game_for_detail = Some(full_game.clone());

        let scraper = self.scraper.clone();
        let section = full_game.section.clone();
        let slug = full_game.console_slug.clone();
        let mfile_id_opt = full_game.mfile_id.clone();
        let game_for_details = full_game.clone();

        let (details_res, versions_res) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut gm = game_for_details;
                let d_res = scraper.fetch_game_page_details(&mut gm).await;
                let mid = gm.mfile_id.clone().or(mfile_id_opt);
                let v_res = if let Some(m) = &mid {
                    scraper.fetch_rom_versions(&section, &slug, m).await
                } else {
                    Ok(Vec::new())
                };
                ((d_res, gm), v_res)
            })
        });

        self.is_loading_versions = false;
        if let (Ok(_), updated_game) = details_res {
            full_game = updated_game;
            // Also keep description & cover cached in catalog games list or search results
            if let Some(cg) = self.catalog_games.iter_mut().find(|cg| cg.id == full_game.id) {
                cg.description = full_game.description.clone();
                cg.mfile_id = full_game.mfile_id.clone();
                if cg.cover_url.is_none() && full_game.cover_url.is_some() {
                    cg.cover_url = full_game.cover_url.clone();
                }
            }
            if let Some(sg) = self.search_results.iter_mut().find(|sg| sg.id == full_game.id) {
                sg.description = full_game.description.clone();
                sg.mfile_id = full_game.mfile_id.clone();
                if sg.cover_url.is_none() && full_game.cover_url.is_some() {
                    sg.cover_url = full_game.cover_url.clone();
                }
            }
        }
        if let Ok(versions) = versions_res {
            self.rom_versions = versions;
        }

        self.selected_game_for_detail = Some(full_game);
    }

    pub fn open_rom_versions_modal(&mut self, game: &GameCard) {
        self.is_loading_versions = true;
        self.rom_versions.clear();
        self.selected_game_for_versions = Some(game.clone());

        let scraper = self.scraper.clone();
        let section = game.section.clone();
        let slug = game.console_slug.clone();
        let mut mfile_id_opt = game.mfile_id.clone();
        let mut gm = game.clone();

        let versions_res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if mfile_id_opt.is_none() {
                    let _ = scraper.fetch_game_page_details(&mut gm).await;
                    mfile_id_opt = gm.mfile_id.clone();
                }

                if let Some(mid) = &mfile_id_opt {
                    scraper.fetch_rom_versions(&section, &slug, mid).await
                } else {
                    Ok(Vec::new())
                }
            })
        });

        self.is_loading_versions = false;
        if let Ok(mut versions) = versions_res {
            if versions.is_empty() {
                let subpath = retroms_desktop::scraper::get_game_subpath(&slug);
                let ext = if subpath == "iso" { "7z" } else { "zip" };
                let fallback_dl = if let Some(mid) = &mfile_id_opt {
                    format!("{}/{}/{}/{}?act=getmfl&id={}", retroms_desktop::scraper::BASE_URL, section, slug, subpath, mid)
                } else {
                    game.download_url.clone()
                };
                versions.push(RomFileVersion {
                    fid: mfile_id_opt.unwrap_or_else(|| "default".to_string()),
                    name: if game.title.ends_with(ext) { game.title.clone() } else { format!("{}.{}", game.title, ext) },
                    size: game.file_size.clone(),
                    category: "Основные".to_string(),
                    download_url: fallback_dl,
                    region_or_type: "USA".to_string(),
                });
            }
            self.rom_versions = versions;
        }
    }

    pub fn trigger_load_rom_versions(&mut self, game: &GameCard) {
        self.open_rom_versions_modal(game);
    }

    pub fn start_game_download(&mut self, game: GameCard, version: Option<RomFileVersion>) {
        let console_folder = self
            .consoles
            .iter()
            .find(|c| c.slug == game.console_slug)
            .map(|c| c.folder_name.clone())
            .unwrap_or_else(|| "ROMs".to_string());

        let target_dir = PathBuf::from(&self.settings.download_directory);
        let auto_unpack = self.settings.auto_unpack_zip;
        let delete_zip = self.settings.delete_zip_after_unpack;

        let game_clone = game.clone();
        let scraper = self.scraper.clone();

        // 1. Determine download link
        let download_url = if let Some(v) = &version {
            v.download_url.clone()
        } else if let Some(mfile_id) = &game.mfile_id {
            // Fetch first version or direct
            let subpath = retroms_desktop::scraper::get_game_subpath(&game.console_slug);
            let versions = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    scraper.fetch_rom_versions(&game.section, &game.console_slug, mfile_id).await
                })
            }).unwrap_or_default();

            if let Some(first_ver) = versions.first() {
                first_ver.download_url.clone()
            } else {
                format!(
                    "https://www.emu-land.net/{}/{}/{}?act=getmfl&id={}",
                    game.section, game.console_slug, subpath, mfile_id
                )
            }
        } else {
            return;
        };

        let referer = format!(
            "https://www.emu-land.net/{}/{}/{}",
            game.section, game.console_slug,
            retroms_desktop::scraper::get_game_subpath(&game.console_slug)
        );

        // 2. Resolve direct URL through 302 redirect
        let direct_url = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                scraper.get_direct_download_url(&download_url, &referer).await
            })
        }).unwrap_or(download_url);

        // 3. Insert record in SQLite
        let raw_title = game.title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let safe_title = raw_title.trim_end_matches('.').trim().to_string();
        let version_name = version.as_ref().map(|v| v.name.clone());
        let file_name = if let Some(v_name) = &version_name {
            v_name.clone()
        } else {
            format!("{}.zip", safe_title)
        };
        let record_id = match self.db.add_download_record(
            &game.id,
            &game.title,
            &game.console_slug,
            &game.console_name,
            &file_name,
            &target_dir.to_string_lossy(),
            &direct_url,
        ) {
            Ok(id) => id,
            Err(e) => {
                self.set_status(format!("Ошибка базы данных: {}", e));
                return;
            }
        };

        let record = DownloadRecord {
            id: record_id,
            game_id: game.id.clone(),
            game_title: game.title.clone(),
            console_slug: game.console_slug.clone(),
            console_name: game.console_name.clone(),
            file_name,
            target_directory: target_dir.to_string_lossy().to_string(),
            local_path: None,
            total_bytes: 0,
            downloaded_bytes: 0,
            speed_bytes_sec: 0,
            status: DownloadStatus::Downloading,
            download_url: direct_url.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            error_message: None,
        };

        self.active_downloads.insert(record_id, record);
        let cancel_token = Arc::new(AtomicBool::new(false));
        self.download_tokens.insert(record_id, cancel_token.clone());

        self.download_manager.download_game(
            record_id,
            game_clone,
            version_name,
            direct_url,
            target_dir,
            console_folder,
            auto_unpack,
            delete_zip,
            cancel_token,
        );

        self.set_status(format!("Начата загрузка {}", game.title));
    }

    pub fn cancel_download(&mut self, record_id: i64) {
        if let Some(token) = self.download_tokens.get(&record_id) {
            token.store(true, Ordering::Relaxed);
        }
        self.active_downloads.remove(&record_id);
        let _ = self.db.update_download_status(record_id, DownloadStatus::Cancelled, None, None);
        self.reload_downloads_history();
        self.set_status("Загрузка отменена");
    }

    pub fn locate_rom_file(&self, game: &GameCard) -> Option<PathBuf> {
        let download_dir = PathBuf::from(&self.settings.download_directory);
        let console_folder = self
            .consoles
            .iter()
            .find(|c| c.slug == game.console_slug)
            .map(|c| c.folder_name.clone())
            .unwrap_or_else(|| "ROMs".to_string());

        let candidate_dirs = [
            download_dir.join(&console_folder),
            download_dir.join(&game.console_name),
            download_dir.join(&game.console_slug),
            download_dir.clone(),
        ];

        let raw_title = game.title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let safe_title_trimmed = raw_title.trim_end_matches('.').trim().to_string();
        let safe_title = raw_title.clone();
        let title_clean = normalize_title_for_rom_match(&game.title);

        let console_ext = retroms_desktop::downloader::get_console_default_rom_extension(&game.console_slug);
        let mut extensions = vec![console_ext, "nes", "sfc", "smc", "bin", "gen", "md", "smd", "gba", "gb", "gbc", "n64", "z64", "v64", "nds", "pce", "iso", "cue", "chd", "zip", "7z", "rom"];
        extensions.dedup();

        // Helper to fix any obsolete .rom extension or double dots on disk
        let fix_rom_extension = |p: PathBuf| -> PathBuf {
            let file_name = match p.file_name().and_then(|s| s.to_str()) {
                Some(n) => n,
                None => return p,
            };
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            let target_ext = retroms_desktop::downloader::get_console_default_rom_extension(&game.console_slug);

            if ext.eq_ignore_ascii_case("rom") || file_name.contains("..") {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let clean_stem = stem.trim_end_matches('.').trim();
                let use_ext = if ext.eq_ignore_ascii_case("rom") { target_ext } else { ext };
                let corrected = p.with_file_name(format!("{}.{}", clean_stem, use_ext));
                if corrected != p {
                    if let Ok(_) = std::fs::rename(&p, &corrected) {
                        println!("[PLAY] Автоматически исправлено имя файла ROM: {} -> {}", p.display(), corrected.display());
                        return corrected;
                    }
                }
            }
            p
        };

        // 1. FIRST & PRIMARY: Search in the user's CURRENT configured download directory!
        for dir in &candidate_dirs {
            if !dir.exists() {
                continue;
            }

            for ext in &extensions {
                let p1 = dir.join(format!("{}.{}", safe_title_trimmed, ext));
                if p1.exists() {
                    return Some(fix_rom_extension(p1));
                }
                let p2 = dir.join(format!("{}.{}", safe_title, ext));
                if p2.exists() {
                    return Some(fix_rom_extension(p2));
                }
            }

            // Alphanumeric match (e.g. "Flintstones, The (USA).sfc", "Super_Mario_Bros.zip", etc.)
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        let ext = entry_path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        if !extensions.iter().any(|&e| e.eq_ignore_ascii_case(&ext)) {
                            continue;
                        }

                        if let Some(stem) = entry_path.file_stem().and_then(|n| n.to_str()) {
                            let stem_without_tags = strip_rom_tags(stem);
                            let stem_clean = normalize_title_for_rom_match(&stem_without_tags);
                            let raw_stem_clean = normalize_title_for_rom_match(stem);

                            if !title_clean.is_empty() && (stem_clean == title_clean || raw_stem_clean == title_clean) {
                                return Some(fix_rom_extension(entry_path));
                            }
                        }
                    }
                }
            }
        }

        // 2. Check if game already has local_file_path that actually exists AND is inside current download directory
        if let Some(p) = &game.local_file_path {
            let path = PathBuf::from(p);
            if path.exists() && path.starts_with(&download_dir) {
                return Some(fix_rom_extension(path));
            }
        }

        // 3. Check download history in SQLite for this game (preferring records inside current download directory)
        for r in &self.download_history {
            if r.status == DownloadStatus::Completed
                && (r.game_id == game.id || r.game_title.eq_ignore_ascii_case(&game.title))
            {
                if let Some(p) = &r.local_path {
                    let path = PathBuf::from(p);
                    if path.exists() && path.starts_with(&download_dir) {
                        return Some(fix_rom_extension(path));
                    }
                }
            }
        }

        // 4. Fallback: if game has local_file_path anywhere else on disk that exists
        if let Some(p) = &game.local_file_path {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(fix_rom_extension(path));
            }
        }

        // 5. Fallback: any completed record from DB anywhere on disk
        for r in &self.download_history {
            if r.status == DownloadStatus::Completed
                && (r.game_id == game.id || r.game_title.eq_ignore_ascii_case(&game.title))
            {
                if let Some(p) = &r.local_path {
                    let path = PathBuf::from(p);
                    if path.exists() {
                        return Some(fix_rom_extension(path));
                    }
                }
            }
        }

        None
    }

    pub fn play_game(&mut self, game: &GameCard) {
        println!("--------------------------------------------------");
        println!("[PLAY] Запрос на запуск игры: '{}' (консоль: {})", game.title, game.console_slug);

        let local_path = match self.locate_rom_file(game) {
            Some(p) => {
                println!("[PLAY] Найден ROM файл на диске: {}", p.display());
                p
            }
            None => {
                let download_dir = Path::new(&self.settings.download_directory);
                let console_folder = self
                    .consoles
                    .iter()
                    .find(|c| c.slug == game.console_slug)
                    .map(|c| c.folder_name.as_str())
                    .unwrap_or("ROMs");
                let expected_dir = download_dir.join(console_folder);

                let err_msg = format!(
                    "Файл игры '{}' не найден в папке: {}. Убедитесь, что игра скачана.",
                    game.title, expected_dir.display()
                );
                println!("[PLAY ОШИБКА] {}", err_msg);
                println!("[PLAY] Текущая папка загрузок в настройках: {}", download_dir.display());
                self.set_status(err_msg);
                return;
            }
        };

        // Cache the found path on the card
        if let Some(cg) = self.catalog_games.iter_mut().find(|cg| cg.id == game.id) {
            cg.local_file_path = Some(local_path.to_string_lossy().to_string());
            cg.is_downloaded = true;
        }

        println!("[PLAY] Итоговый путь к файлу: {}", local_path.display());
        println!("[PLAY] Режим RetroArch: {}, путь: {}", self.settings.use_retroarch, self.settings.retroarch_path);

        let emulator = self.settings.emulator_paths.get(&game.console_slug).cloned().unwrap_or_default();
        let args = self.settings.emulator_args.get(&game.console_slug).cloned().unwrap_or_default();

        if self.settings.use_retroarch {
            if self.settings.retroarch_path.trim().is_empty() {
                let msg = "ОШИБКА: В настройках включён RetroArch, но путь к retroarch.exe не указан! Укажите его в Настройках.";
                println!("[PLAY ОШИБКА] {}", msg);
                self.set_status(msg);
                return;
            }

            let core = self.settings.get_retroarch_core_for_console(&game.console_slug);
            println!("[PLAY] Выбранное ядро RetroArch для {}: {:?}", game.console_slug, core);

            match launch_retroarch(
                &self.settings.retroarch_path,
                &local_path,
                core,
                &self.settings.retroarch_args,
            ) {
                Ok(_) => {
                    let core_display = core.unwrap_or("авто");
                    let msg = format!("Запущен RetroArch [{}] для {}", core_display, game.title);
                    println!("[PLAY УСПЕХ] {}", msg);
                    self.set_status(msg);
                }
                Err(e) => {
                    let msg = format!("Не удалось запустить RetroArch: {}", e);
                    println!("[PLAY ОШИБКА] {}", msg);
                    self.set_status(msg);
                }
            }
        } else if !emulator.is_empty() {
            println!("[PLAY] Запуск через отдельный эмулятор: {} {:?}", emulator, args);
            match launch_emulator(&emulator, &local_path, &args) {
                Ok(_) => {
                    let msg = format!("Запущен эмулятор: {}", emulator);
                    println!("[PLAY УСПЕХ] {}", msg);
                    self.set_status(msg);
                }
                Err(e) => {
                    let msg = format!("Не удалось запустить эмулятор: {}", e);
                    println!("[PLAY ОШИБКА] {}", msg);
                    self.set_status(msg);
                }
            }
        } else {
            println!("[PLAY] Эмулятор не настроен. Открытие через системную ассоциацию: {}", local_path.display());
            let _ = open::that(&local_path);
            self.set_status(format!("Открыт файл: {}", local_path.display()));
        }
    }
}

impl eframe::App for RetroRomsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Process asynchronous download events
        while let Ok(event) = self.download_rx.try_recv() {
            match event {
                DownloadEvent::Started { record_id, game_id: _ } => {
                    let _ = self.db.update_download_status(record_id, DownloadStatus::Downloading, None, None);
                }
                DownloadEvent::Progress {
                    record_id,
                    downloaded_bytes,
                    total_bytes,
                    speed_bytes_sec,
                    percent: _,
                } => {
                    if let Some(record) = self.active_downloads.get_mut(&record_id) {
                        record.downloaded_bytes = downloaded_bytes;
                        record.total_bytes = total_bytes;
                        record.speed_bytes_sec = speed_bytes_sec;
                    }
                    let _ = self.db.update_download_progress(record_id, downloaded_bytes, total_bytes);
                }
                DownloadEvent::Completed { record_id, local_path, message } => {
                    self.active_downloads.remove(&record_id);
                    self.download_tokens.remove(&record_id);
                    let _ = self.db.update_download_status(record_id, DownloadStatus::Completed, Some(&local_path), None);
                    self.reload_downloads_history();
                    self.sync_download_status();
                    self.set_status(message);
                }
                DownloadEvent::ZipNeedsSelection { request } => {
                    self.active_downloads.remove(&request.record_id);
                    self.download_tokens.remove(&request.record_id);
                    self.zip_extraction_request = Some(request);
                }
                DownloadEvent::Failed { record_id, game_id: _, error } => {
                    self.active_downloads.remove(&record_id);
                    self.download_tokens.remove(&record_id);
                    let _ = self.db.update_download_status(record_id, DownloadStatus::Failed, None, Some(&error));
                    self.reload_downloads_history();
                    self.set_status(format!("Ошибка загрузки: {}", error));
                }
            }
        }

        // Process search results from background task
        while let Ok((query, res)) = self.search_rx.try_recv() {
            if query == self.search_query.trim() {
                self.is_searching = false;
                match res {
                    Ok(mut games) => {
                        let fav_ids: std::collections::HashSet<String> = self.favorite_games.iter().map(|g| g.id.clone()).collect();
                        for g in &mut games {
                            if fav_ids.contains(&g.id) {
                                g.is_favorite = true;
                            }
                        }
                        self.search_results = games.clone();
                        self.sync_download_status();
                        let _ = self.db.save_games(&games);
                        self.set_status(format!("Найдено {} игр по запросу «{}»", games.len(), query));
                    }
                    Err(e) => {
                        // Offline / error fallback: search local SQLite DB
                        if let Ok(mut local_games) = self.db.search_games(&query) {
                            let fav_ids: std::collections::HashSet<String> = self.favorite_games.iter().map(|g| g.id.clone()).collect();
                            for g in &mut local_games {
                                if fav_ids.contains(&g.id) {
                                    g.is_favorite = true;
                                }
                            }
                            self.search_results = local_games;
                            self.sync_download_status();
                            self.set_status(format!("Офлайн-поиск: найдено {} игр (сеть: {})", self.search_results.len(), e));
                        } else {
                            self.set_status(format!("Ошибка поиска на Emu-Land: {}", e));
                        }
                    }
                }
            }
        }

        // Automatic typing debounce trigger
        let current_trimmed_query = self.search_query.trim().to_string();
        if current_trimmed_query != self.last_query_seen {
            self.last_query_seen = current_trimmed_query.clone();
            self.last_query_typed_at = Some(std::time::Instant::now());
        }

        if let Some(typed_at) = self.last_query_typed_at {
            if typed_at.elapsed() >= std::time::Duration::from_millis(450) {
                self.last_query_typed_at = None;
                if current_trimmed_query != self.last_executed_search {
                    self.execute_site_search(&current_trimmed_query);
                }
            }
        }

        // Apply visual theme
        self.theme.apply_to_ctx(ctx);

        // Handle modals
        let mut download_version_to_start = None;
        let mut play_from_detail = None;
        let mut reveal_from_detail = None;
        let mut image_to_view_from_detail = None;

        render_game_detail_window(
            ctx,
            &mut self.selected_game_for_detail,
            &self.rom_versions,
            self.is_loading_versions,
            self.theme,
            &mut download_version_to_start,
            &mut play_from_detail,
            &mut reveal_from_detail,
            &mut image_to_view_from_detail,
        );

        if let Some((title, url)) = image_to_view_from_detail {
            self.image_viewer_state = Some(ImageViewerState::new(title, url));
        }

        if let Some((game, ver)) = download_version_to_start {
            let ver_name = ver.name.clone();
            self.start_game_download(game, Some(ver));
            self.selected_game_for_detail = None;
            self.set_status(format!("Начата загрузка версии: {}", ver_name));
        }
        if let Some(game) = play_from_detail {
            self.play_game(&game);
        }
        if let Some(path) = reveal_from_detail {
            let _ = reveal_in_file_explorer(Path::new(&path));
        }

        // ROM Versions Modal (opened specifically via "⬇ Скачать" button)
        let mut download_version_from_modal = None;
        render_rom_versions_modal(
            ctx,
            &mut self.selected_game_for_versions,
            &self.rom_versions,
            self.is_loading_versions,
            self.theme,
            &mut download_version_from_modal,
        );

        if let Some((game, ver)) = download_version_from_modal {
            let ver_name = ver.name.clone();
            self.start_game_download(game, Some(ver));
            self.selected_game_for_versions = None;
            self.set_status(format!("Начата загрузка версии: {}", ver_name));
        }

        // Image Viewer Modal (opened via clicking game cover / screenshots)
        render_image_viewer_modal(
            ctx,
            &mut self.image_viewer_state,
            self.theme,
        );

        let mut zip_selection_confirmed = None;
        render_zip_modal(
            ctx,
            &mut self.zip_extraction_request,
            self.theme,
            &mut zip_selection_confirmed,
        );

        if let Some((request, selected_names)) = zip_selection_confirmed {
            self.download_manager.extract_multi_rom_selection(
                request,
                selected_names,
                self.settings.delete_zip_after_unpack,
            );
            self.zip_extraction_request = None;
        }

        // 2. Top Bar (Search, view mode, sort, categories)
        TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(self.theme.bg_color()).inner_margin(8.0))
            .show(ctx, |ui| {
                let mut category_changed = false;
                let mut refresh_clicked = false;
                let mut search_triggered = false;
                let mut clear_search = false;

                // Group search results by console slug to form platform filter chips
                let mut platform_counts: HashMap<String, (String, usize)> = HashMap::new();
                for g in &self.search_results {
                    let entry = platform_counts.entry(g.console_slug.clone()).or_insert_with(|| {
                        let short = g.console_name.split('/').next().unwrap_or(&g.console_name).trim().to_string();
                        (short, 0)
                    });
                    entry.1 += 1;
                }
                let mut search_platforms: Vec<(String, String, usize)> = platform_counts
                    .into_iter()
                    .map(|(slug, (name, count))| (slug, name, count))
                    .collect();
                search_platforms.sort_by(|a, b| b.2.cmp(&a.2));

                render_topbar(
                    ui,
                    &mut self.search_query,
                    self.is_searching,
                    self.search_results.len(),
                    &search_platforms,
                    &mut self.search_platform_filter,
                    &self.categories,
                    &mut self.selected_category,
                    &mut self.view_mode,
                    &mut self.sort_option,
                    self.is_loading_games,
                    self.theme,
                    &mut search_triggered,
                    &mut clear_search,
                    &mut category_changed,
                    &mut refresh_clicked,
                );

                if search_triggered {
                    self.last_query_typed_at = None;
                    let q = self.search_query.clone();
                    self.execute_site_search(&q);
                }
                if clear_search {
                    self.last_query_typed_at = None;
                    self.execute_site_search("");
                }
                if category_changed {
                    self.current_page = 1;
                    self.trigger_load_games_sync();
                }
                if refresh_clicked {
                    if !self.search_query.trim().is_empty() {
                        let q = self.search_query.clone();
                        self.execute_site_search(&q);
                    } else {
                        self.trigger_load_games_sync();
                    }
                }
            });

        // 3. Bottom Status Bar
        TopBottomPanel::bottom("bottom_status_panel")
            .frame(egui::Frame::none().fill(self.theme.card_bg_color()).inner_margin(6.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status_message).size(11.0).color(self.theme.text_color()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let active_count = self.active_downloads.len();
                        if active_count > 0 {
                            ui.label(
                                RichText::new(format!("⬇ Активных загрузок: {}", active_count))
                                    .size(11.0)
                                    .color(self.theme.primary_color()),
                            );
                        }
                    });
                });
            });

        // 4. Side Panel (Sidebar)
        SidePanel::left("left_sidebar_panel")
            .resizable(true)
            .default_width(220.0)
            .width_range(180.0..=300.0)
            .frame(egui::Frame::none().fill(self.theme.bg_color()).inner_margin(8.0))
            .show(ctx, |ui| {
                let mut console_changed = false;
                let mut manage_clicked = false;

                let active_dl_count = self.active_downloads.len();
                let fav_count = self.favorite_games.len();

                render_sidebar(
                    ui,
                    &mut self.current_tab,
                    &mut self.consoles,
                    &mut self.selected_console_idx,
                    active_dl_count,
                    fav_count,
                    self.theme,
                    &mut console_changed,
                    &mut manage_clicked,
                );

                if console_changed {
                    if let Some(c) = self.consoles.get(self.selected_console_idx) {
                        self.settings.last_console_slug = Some(c.slug.clone());
                        let _ = self.settings.save();
                    }
                    self.selected_category = "top".to_string();
                    self.current_page = 1;
                    self.trigger_load_games_sync();
                }

                if manage_clicked {
                    self.current_tab = NavTab::Settings;
                }
            });

        // 5. Central Panel (Main Content)
        CentralPanel::default()
            .frame(egui::Frame::none().fill(self.theme.bg_color()).inner_margin(8.0))
            .show(ctx, |ui| {
                match self.current_tab {
                    NavTab::Catalog => {
                        let mut game_to_open = None;
                        let mut game_to_download = None;
                        let mut favorite_toggle = None;
                        let mut play_clicked = None;
                        let mut page_changed = None;
                        let mut image_to_view = None;

                        let is_search_mode = !self.search_query.trim().is_empty();

                        // Filter and sort displayed games
                        let mut displayed: Vec<GameCard> = if is_search_mode {
                            let mut res = self.search_results.clone();
                            if let Some(ref slug_filter) = self.search_platform_filter {
                                res.retain(|g| g.console_slug.eq_ignore_ascii_case(slug_filter));
                            }
                            res
                        } else {
                            self.catalog_games.clone()
                        };

                        // Apply sort
                        match self.sort_option {
                            SortOption::RatingDesc => displayed.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal)),
                            SortOption::TitleAsc => displayed.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
                            SortOption::TitleDesc => displayed.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase())),
                            SortOption::Default => {}
                        }

                        if is_search_mode && displayed.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(50.0);
                                if self.is_searching {
                                    ui.spinner();
                                    ui.add_space(14.0);
                                    ui.label(
                                        RichText::new("Поиск по всей библиотеке Emu-Land.net…")
                                            .size(16.0)
                                            .color(self.theme.accent_color())
                                            .strong(),
                                    );
                                } else {
                                    ui.label(RichText::new("🔍").size(48.0));
                                    ui.add_space(10.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "Ничего не найдено по запросу «{}»",
                                            self.search_query.trim()
                                        ))
                                        .size(16.0)
                                        .color(self.theme.text_color())
                                        .strong(),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(
                                            "Попробуйте ввести название на английском (например: Mario, Sonic, Mortal Kombat, Zelda, Contra).",
                                        )
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(140, 150, 175)),
                                    );
                                    ui.add_space(16.0);
                                    if ui.button("Очистить поиск").clicked() {
                                        self.search_query.clear();
                                        self.execute_site_search("");
                                    }
                                }
                            });
                        } else {
                            let cur_page = if is_search_mode { 1 } else { self.current_page };
                            let tot_pages = if is_search_mode { 1 } else { self.total_pages };

                            match self.view_mode {
                                ViewMode::Grid => {
                                    render_catalog_grid(
                                        ui,
                                        &displayed,
                                        cur_page,
                                        tot_pages,
                                        self.theme,
                                        &mut game_to_open,
                                        &mut game_to_download,
                                        &mut favorite_toggle,
                                        &mut play_clicked,
                                        &mut page_changed,
                                        &mut image_to_view,
                                    );
                                }
                                ViewMode::Table => {
                                    render_catalog_table(
                                        ui,
                                        &displayed,
                                        cur_page,
                                        tot_pages,
                                        self.theme,
                                        &mut game_to_open,
                                        &mut game_to_download,
                                        &mut favorite_toggle,
                                        &mut play_clicked,
                                        &mut page_changed,
                                        &mut image_to_view,
                                    );
                                }
                            }
                        }

                        if let Some(p) = page_changed {
                            if !is_search_mode {
                                self.current_page = p;
                                self.trigger_load_games_sync();
                            }
                        }
                        if let Some(game) = game_to_open {
                            self.open_game_detail(&game);
                        }
                        if let Some(game) = game_to_download {
                            self.open_rom_versions_modal(&game);
                        }
                        if let Some((title, url)) = image_to_view {
                            self.image_viewer_state = Some(ImageViewerState::new(title, url));
                        }
                        if let Some((game, new_fav)) = favorite_toggle {
                            let _ = self.db.set_game_favorite(&game, new_fav);
                            self.reload_favorites();
                            // Update local copy
                            if let Some(g) = self.catalog_games.iter_mut().find(|g| g.id == game.id) {
                                g.is_favorite = new_fav;
                            }
                            if let Some(g) = self.search_results.iter_mut().find(|g| g.id == game.id) {
                                g.is_favorite = new_fav;
                            }
                        }
                        if let Some(game) = play_clicked {
                            self.play_game(&game);
                        }
                    }
                    NavTab::Downloads => {
                        let mut cancel_dl = None;
                        let mut play_file = None;
                        let mut reveal_file = None;
                        let mut delete_hist = None;
                        let mut clear_all_hist = false;

                        let active_vec: Vec<DownloadRecord> = self.active_downloads.values().cloned().collect();

                        render_downloads_view(
                            ui,
                            &active_vec,
                            &self.download_history,
                            self.theme,
                            &mut cancel_dl,
                            &mut play_file,
                            &mut reveal_file,
                            &mut delete_hist,
                            &mut clear_all_hist,
                        );

                        if let Some(id) = cancel_dl {
                            self.cancel_download(id);
                        }
                        if let Some(fpath) = play_file {
                            let _ = open::that(&fpath);
                        }
                        if let Some(fpath) = reveal_file {
                            let _ = reveal_in_file_explorer(Path::new(&fpath));
                        }
                        if let Some(id) = delete_hist {
                            let _ = self.db.delete_download_record(id);
                            self.reload_downloads_history();
                        }
                    }
                    NavTab::Favorites => {
                        let mut game_to_open = None;
                        let mut game_to_download = None;
                        let mut favorite_toggle = None;
                        let mut play_clicked = None;
                        let mut image_to_view = None;

                        render_favorites_view(
                            ui,
                            &self.favorite_games,
                            self.view_mode,
                            self.theme,
                            &mut game_to_open,
                            &mut game_to_download,
                            &mut favorite_toggle,
                            &mut play_clicked,
                            &mut image_to_view,
                        );

                        if let Some(game) = game_to_open {
                            self.open_game_detail(&game);
                        }
                        if let Some(game) = game_to_download {
                            self.open_rom_versions_modal(&game);
                        }
                        if let Some((title, url)) = image_to_view {
                            self.image_viewer_state = Some(ImageViewerState::new(title, url));
                        }
                        if let Some((game, new_fav)) = favorite_toggle {
                            let _ = self.db.set_game_favorite(&game, new_fav);
                            self.reload_favorites();
                        }
                        if let Some(game) = play_clicked {
                            self.play_game(&game);
                        }
                    }
                    NavTab::Settings => {
                        let mut consoles_updated = false;
                        let mut theme_changed = None;
                        let prev_dl_dir = self.settings.download_directory.clone();

                        render_settings_view(
                            ui,
                            &mut self.settings,
                            &mut self.consoles,
                            self.theme,
                            &mut consoles_updated,
                            &mut theme_changed,
                        );

                        if prev_dl_dir != self.settings.download_directory {
                            self.sync_download_status();
                        }

                        if consoles_updated {
                            let _ = self.db.update_console_order_and_enabled(&self.consoles);
                        }
                        if let Some(new_theme_str) = theme_changed {
                            self.settings.active_theme = new_theme_str.clone();
                            self.theme = ThemePreset::from_str(&new_theme_str);
                            let _ = self.settings.save();
                        }
                    }
                }
            });

        // Request continuous repaint if active downloads are running
        if !self.active_downloads.is_empty() {
            ctx.request_repaint();
        }
    }
}

impl RetroRomsApp {
    pub fn trigger_load_games_sync(&mut self) {
        let console = match self.current_console() {
            Some(c) => c.clone(),
            None => return,
        };

        self.is_loading_games = true;
        self.set_status(format!("Загрузка каталога: {}...", console.short_name));

        let slug = console.slug.clone();
        let name = console.name.clone();
        let section = console.section.clone();
        let category = self.selected_category.clone();
        let page = self.current_page;
        let scraper = self.scraper.clone();

        let fav_ids: std::collections::HashSet<String> = self.favorite_games.iter().map(|g| g.id.clone()).collect();

        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                scraper.fetch_games_page(&slug, &name, &section, &category, page).await
            })
        });

        self.is_loading_games = false;
        match result {
            Ok(page_result) => {
                self.current_page = page_result.current_page;
                self.total_pages = page_result.total_pages;
                if !page_result.available_categories.is_empty() {
                    self.categories = page_result.available_categories;
                }

                let mut games = page_result.games;
                for g in &mut games {
                    if fav_ids.contains(&g.id) {
                        g.is_favorite = true;
                    }
                }

                let count = games.len();
                self.catalog_games = games;
                self.sync_download_status();
                self.set_status(format!("Загружено {} игр ({})", count, console.short_name));
            }
            Err(e) => {
                self.set_status(format!("Ошибка загрузки: {}", e));
            }
        }
    }
}
