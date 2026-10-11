slint::include_modules!();

use retroms_desktop::config::AppSettings;
use retroms_desktop::db::Database;
use retroms_desktop::downloader::{DownloadEvent, DownloadManager};
use retroms_desktop::image_cache::ImageCacheManager;
use retroms_desktop::integrations::{
    launch_emulator, launch_retroarch, pick_download_folder, pick_executable_file,
    reveal_in_file_explorer,
};
use retroms_desktop::models::{
    CatalogCategory, ConsoleInfo, DownloadRecord, DownloadStatus, GameCard, RomFileVersion,
    ZipExtractionRequest, ZipRomEntry,
};
use retroms_desktop::scraper::EmuLandClient;
use retroms_desktop::theme::ThemePreset;
use retroms_desktop::ui::{
    get_console_badge_info, get_format_label, get_region_label, NavTab, SortOption, ViewMode,
};

use slint::{ComponentHandle, ModelRc, VecModel};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::unbounded_channel;

struct AppState {
    db: Database,
    settings: AppSettings,
    consoles: Vec<ConsoleInfo>,
    selected_console_idx: usize,
    catalog_games: Vec<GameCard>,
    categories: Vec<CatalogCategory>,
    selected_category: String,
    current_page: usize,
    total_pages: usize,
    is_loading_games: bool,
    search_query: String,
    is_search_mode: bool,
    search_results: Vec<GameCard>,
    search_platform_filter: Option<String>,
    is_searching: bool,
    favorite_games: Vec<GameCard>,
    download_manager: DownloadManager,
    active_downloads: HashMap<i64, DownloadRecord>,
    download_tokens: HashMap<i64, Arc<AtomicBool>>,
    download_history: Vec<DownloadRecord>,
    selected_game_for_detail: Option<GameCard>,
    selected_game_for_versions: Option<GameCard>,
    rom_versions: Vec<RomFileVersion>,
    current_zip_request: Option<ZipExtractionRequest>,
    scraper: EmuLandClient,
    image_cache: ImageCacheManager,
    current_tab: NavTab,
    view_mode: ViewMode,
    sort_option: SortOption,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  RetroROMs Desktop (Slint Modern PC Edition)");
    println!("  Emu-Land Catalog & ROM Manager");
    println!("============================================================");

    let db = Database::open().expect("Failed to initialize SQLite database");
    let settings = AppSettings::load();
    let consoles = db
        .get_consoles()
        .unwrap_or_else(|_| retroms_desktop::models::get_default_consoles());

    let (dl_tx, mut dl_rx) = unbounded_channel();
    let download_manager = DownloadManager::new(dl_tx);
    let scraper = EmuLandClient::new();
    let (image_cache, mut img_completed_rx) = ImageCacheManager::new();

    let view_mode = if settings.view_mode == "Table" {
        ViewMode::Table
    } else {
        ViewMode::Grid
    };

    let selected_console_idx = if let Some(slug) = &settings.last_console_slug {
        consoles
            .iter()
            .position(|c| c.is_enabled && c.slug == *slug)
            .or_else(|| consoles.iter().position(|c| c.is_enabled))
            .unwrap_or(0)
    } else {
        consoles.iter().position(|c| c.is_enabled).unwrap_or(0)
    };

    let main_window = MainWindow::new()?;

    let app_state = Arc::new(Mutex::new(AppState {
        db,
        settings,
        consoles,
        selected_console_idx,
        catalog_games: Vec::new(),
        categories: Vec::new(),
        selected_category: "top".to_string(),
        current_page: 1,
        total_pages: 1,
        is_loading_games: false,
        search_query: String::new(),
        is_search_mode: false,
        search_results: Vec::new(),
        search_platform_filter: None,
        is_searching: false,
        favorite_games: Vec::new(),
        download_manager,
        active_downloads: HashMap::new(),
        download_tokens: HashMap::new(),
        download_history: Vec::new(),
        selected_game_for_detail: None,
        selected_game_for_versions: None,
        rom_versions: Vec::new(),
        current_zip_request: None,
        scraper,
        image_cache,
        current_tab: NavTab::Catalog,
        view_mode,
        sort_option: SortOption::Default,
    }));

    // Initial state loading
    {
        let mut state = app_state.lock().unwrap();
        reload_favorites(&mut state);
        reload_downloads_history(&mut state);
        apply_theme_to_ui(&main_window, &state.settings.active_theme);
        sync_consoles_to_ui(&main_window, &state);
        sync_settings_to_ui(&main_window, &state);
        main_window.set_view_mode(state.view_mode.to_idx());
    }

    // Trigger initial catalog load
    trigger_load_games(app_state.clone(), main_window.as_weak(), 1);

    // 1. Download Events Listener Task
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        tokio::spawn(async move {
            while let Some(event) = dl_rx.recv().await {
                let mut state = state.lock().unwrap();
                match event {
                    DownloadEvent::Started {
                        record_id,
                        game_id,
                    } => {
                        let record = DownloadRecord {
                            id: record_id,
                            game_id,
                            game_title: "".to_string(),
                            console_slug: "".to_string(),
                            console_name: "".to_string(),
                            file_name: "".to_string(),
                            target_directory: "".to_string(),
                            local_path: None,
                            total_bytes: 0,
                            downloaded_bytes: 0,
                            speed_bytes_sec: 0,
                            status: DownloadStatus::Downloading,
                            download_url: "".to_string(),
                            timestamp: chrono::Utc::now().timestamp(),
                            error_message: None,
                        };
                        state.active_downloads.insert(record_id, record);
                    }
                    DownloadEvent::Progress {
                        record_id,
                        downloaded_bytes,
                        total_bytes,
                        speed_bytes_sec,
                        percent: _,
                    } => {
                        if let Some(dl) = state.active_downloads.get_mut(&record_id) {
                            dl.downloaded_bytes = downloaded_bytes;
                            dl.total_bytes = total_bytes;
                            dl.speed_bytes_sec = speed_bytes_sec;
                            dl.status = DownloadStatus::Downloading;
                        }
                    }
                    DownloadEvent::Completed {
                        record_id,
                        local_path: _,
                        message: _,
                    } => {
                        state.active_downloads.remove(&record_id);
                        reload_downloads_history(&mut state);
                        sync_download_status_on_games(&mut state);
                    }
                    DownloadEvent::ZipNeedsSelection { request } => {
                        let title = request.game.title.clone();
                        let entries: Vec<ZipRomEntry> = request.entries.clone();
                        let selected_count = entries.iter().filter(|e| e.is_selected).count() as i32;
                        state.current_zip_request = Some(request);

                        let ui_weak = ui_weak.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_zip_game_title(title.into());
                                ui.set_zip_selected_count(selected_count);
                                let slint_entries: Vec<ZipEntryData> = entries
                                    .iter()
                                    .map(|e| ZipEntryData {
                                        entry_name: e.entry_name.clone().into(),
                                        display_name: e.display_name.clone().into(),
                                        formatted_size: e.formatted_size.clone().into(),
                                        is_selected: e.is_selected,
                                    })
                                    .collect();
                                ui.set_zip_entries(ModelRc::new(VecModel::from(slint_entries)));
                                ui.set_show_zip_modal(true);
                            }
                        });
                    }
                    DownloadEvent::Failed {
                        record_id,
                        game_id: _,
                        error: _,
                    } => {
                        state.active_downloads.remove(&record_id);
                        reload_downloads_history(&mut state);
                    }
                }

                // Update UI active downloads and history
                sync_downloads_to_ui(&ui_weak, &state);
            }
        });
    }

    // 2. Image Cache Notifications Listener
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        tokio::spawn(async move {
            while let Some(_game_id) = img_completed_rx.recv().await {
                let state_ref = state.clone();
                let ui_weak = ui_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let state = state_ref.lock().unwrap();
                        let games_slint = build_slint_games(&state);
                        let favs_slint = build_slint_favorites(&state);
                        ui.set_catalog_games(ModelRc::new(VecModel::from(games_slint)));
                        ui.set_favorite_games(ModelRc::new(VecModel::from(favs_slint)));
                    }
                });
            }
        });
    }

    // 3. Setup Callbacks between Slint UI and Rust Backend

    // Navigation Tab Selected
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_nav_tab_selected(move |tab_idx| {
            let mut state = state.lock().unwrap();
            state.current_tab = NavTab::from_idx(tab_idx);
            sync_displayed_games_to_ui(&ui_weak, &state);
        });
    }

    // Console Selected in Sidebar
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_console_selected(move |idx| {
            {
                let mut state = state.lock().unwrap();
                state.selected_console_idx = idx as usize;
                state.selected_category = "top".to_string();
                state.current_page = 1;
                state.current_tab = NavTab::Catalog;
                if let Some(c) = state.consoles.get(state.selected_console_idx) {
                    state.settings.last_console_slug = Some(c.slug.clone());
                    let _ = state.settings.save();
                }
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_tab(0);
                    ui.set_active_category("top".into());
                    sync_consoles_to_ui(&ui, &state);
                }
            }
            trigger_load_games(state.clone(), ui_weak.clone(), 1);
        });
    }

    // Manage Consoles
    {
        let ui_weak = main_window.as_weak();
        main_window.on_manage_consoles(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_current_tab(3); // Navigate to Settings
            }
        });
    }

    // View Mode Changed (Grid / Table)
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_view_mode_changed(move |mode_idx| {
            let mut state = state.lock().unwrap();
            state.view_mode = ViewMode::from_idx(mode_idx);
            state.settings.view_mode = if state.view_mode == ViewMode::Table {
                "Table".to_string()
            } else {
                "Grid".to_string()
            };
            let _ = state.settings.save();
            sync_displayed_games_to_ui(&ui_weak, &state);
        });
    }

    // Sort Changed
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_sort_changed(move |sort_idx| {
            let mut state = state.lock().unwrap();
            state.sort_option = SortOption::from_idx(sort_idx);
            sync_displayed_games_to_ui(&ui_weak, &state);
        });
    }

    // Category Selected
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_category_selected(move |cat_key| {
            {
                let mut state = state.lock().unwrap();
                state.selected_category = cat_key.to_string();
                state.current_page = 1;
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_active_category(cat_key.clone());
                }
            }
            trigger_load_games(state.clone(), ui_weak.clone(), 1);
        });
    }

    // Platform Filter in Search
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_platform_filter_selected(move |slug| {
            let mut state = state.lock().unwrap();
            if let Some(cur) = &state.search_platform_filter {
                if cur == slug.as_str() {
                    state.search_platform_filter = None;
                } else {
                    state.search_platform_filter = Some(slug.to_string());
                }
            } else {
                state.search_platform_filter = Some(slug.to_string());
            }
            sync_displayed_games_to_ui(&ui_weak, &state);
        });
    }

    // Search Triggered
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_search_triggered(move |query| {
            let q = query.trim().to_string();
            let state_clone = state.clone();
            let ui_weak_clone = ui_weak.clone();

            {
                let mut state = state.lock().unwrap();
                state.search_query = q.clone();
                if q.is_empty() {
                    state.is_search_mode = false;
                    state.search_results.clear();
                    sync_displayed_games_to_ui(&ui_weak_clone, &state);
                    return;
                }
                state.is_search_mode = true;
                state.is_searching = true;
                if let Some(ui) = ui_weak_clone.upgrade() {
                    ui.set_is_search_mode(true);
                    ui.set_is_searching(true);
                    ui.set_status_text(format!("Поиск на Emu-Land.net по запросу «{}»…", q).into());
                }
            }

            tokio::spawn(async move {
                let scraper = { state_clone.lock().unwrap().scraper.clone() };
                let search_res = scraper.search_games_site_wide(&q).await;

                let mut state = state_clone.lock().unwrap();
                state.is_searching = false;
                match search_res {
                    Ok(games) => {
                        let _ = state.db.save_games(&games);
                        state.search_results = games;
                        sync_download_status_on_games(&mut state);
                        let count = state.search_results.len();
                        let status_msg = format!("Найдено {} игр по запросу «{}»", count, q);
                        state.set_status(&status_msg);

                        let ui_weak = ui_weak_clone.clone();
                        let state_ref = state_clone.clone();

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                let state = state_ref.lock().unwrap();
                                let games_slint = build_slint_games(&state);
                                let platforms = build_search_platforms(&state);
                                ui.set_is_searching(false);
                                ui.set_search_results_count(count as i32);
                                ui.set_search_platforms(ModelRc::new(VecModel::from(platforms)));
                                ui.set_catalog_games(ModelRc::new(VecModel::from(games_slint)));
                                ui.set_status_text(status_msg.into());
                            }
                        });
                    }
                    Err(e) => {
                        // Offline search fallback
                        if let Ok(local_games) = state.db.search_games(&q) {
                            state.search_results = local_games;
                            sync_download_status_on_games(&mut state);
                        }
                        let count = state.search_results.len();
                        let status_msg = format!("Офлайн-поиск: найдено {} игр (сеть: {})", count, e);

                        let ui_weak = ui_weak_clone.clone();
                        let state_ref = state_clone.clone();

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                let state = state_ref.lock().unwrap();
                                let games_slint = build_slint_games(&state);
                                let platforms = build_search_platforms(&state);
                                ui.set_is_searching(false);
                                ui.set_search_results_count(count as i32);
                                ui.set_search_platforms(ModelRc::new(VecModel::from(platforms)));
                                ui.set_catalog_games(ModelRc::new(VecModel::from(games_slint)));
                                ui.set_status_text(status_msg.into());
                            }
                        });
                    }
                }
            });
        });
    }

    // Search Cleared
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_search_cleared(move || {
            let mut state = state.lock().unwrap();
            state.search_query.clear();
            state.is_search_mode = false;
            state.search_results.clear();
            state.search_platform_filter = None;
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_is_search_mode(false);
                ui.set_is_searching(false);
                ui.set_search_results_count(0);
            }
            sync_displayed_games_to_ui(&ui_weak, &state);
        });
    }

    // Refresh Clicked
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_refresh_clicked(move || {
            let page = { state.lock().unwrap().current_page };
            trigger_load_games(state.clone(), ui_weak.clone(), page);
        });
    }

    // Page Changed
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_page_changed(move |new_page| {
            if new_page >= 1 {
                trigger_load_games(state.clone(), ui_weak.clone(), new_page as usize);
            }
        });
    }

    // Game Detail Opened
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_game_detail_opened(move |game_id| {
            let target_game = {
                let state = state.lock().unwrap();
                state
                    .catalog_games
                    .iter()
                    .chain(state.search_results.iter())
                    .chain(state.favorite_games.iter())
                    .find(|g| g.id == game_id.as_str())
                    .cloned()
            };

            if let Some(mut game) = target_game {
                let state_clone = state.clone();
                let ui_weak_clone = ui_weak.clone();

                // Open modal immediately with current info
                if let Some(ui) = ui_weak.upgrade() {
                    let slint_game = {
                        let st = state.lock().unwrap();
                        game_card_to_slint(&game, &st.image_cache)
                    };
                    ui.set_detail_game(slint_game);
                    ui.set_detail_is_loading_versions(true);
                    ui.set_show_game_detail(true);
                }

                // Fetch details and versions in background
                tokio::spawn(async move {
                    let scraper = { state_clone.lock().unwrap().scraper.clone() };
                    let _ = scraper.fetch_game_page_details(&mut game).await;

                    let versions = if let Some(mid) = &game.mfile_id {
                        scraper
                            .fetch_rom_versions(&game.section, &game.console_slug, mid)
                            .await
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };

                    {
                        let mut st = state_clone.lock().unwrap();
                        st.selected_game_for_detail = Some(game.clone());
                        st.rom_versions = versions.clone();
                    }

                    let ui_weak = ui_weak_clone.clone();
                    let slint_versions: Vec<RomVersionData> = versions
                        .iter()
                        .map(|v| RomVersionData {
                            fid: v.fid.clone().into(),
                            name: v.name.clone().into(),
                            size: v.size.clone().into(),
                            category: v.category.clone().into(),
                            region_or_type: v.region_or_type.clone().into(),
                            download_url: v.download_url.clone().into(),
                        })
                        .collect();

                    let screenshot_urls = game.screenshot_urls.clone();

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            let screenshots: Vec<ScreenshotData> = screenshot_urls
                                .iter()
                                .map(|url| ScreenshotData {
                                    url: url.clone().into(),
                                    img: slint::Image::default(),
                                    has_img: false,
                                })
                                .collect();
                            ui.set_detail_is_loading_versions(false);
                            ui.set_detail_rom_versions(ModelRc::new(VecModel::from(slint_versions)));
                            ui.set_detail_screenshots(ModelRc::new(VecModel::from(screenshots)));
                        }
                    });
                });
            }
        });
    }

    // Game Download Clicked (Opens ROM Versions Modal)
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_game_download_clicked(move |game_id| {
            let target_game = {
                let state = state.lock().unwrap();
                state
                    .catalog_games
                    .iter()
                    .chain(state.search_results.iter())
                    .chain(state.favorite_games.iter())
                    .find(|g| g.id == game_id.as_str())
                    .cloned()
            };

            if let Some(mut game) = target_game {
                let state_clone = state.clone();
                let ui_weak_clone = ui_weak.clone();

                if let Some(ui) = ui_weak.upgrade() {
                    let slint_game = {
                        let st = state.lock().unwrap();
                        game_card_to_slint(&game, &st.image_cache)
                    };
                    ui.set_versions_game(slint_game);
                    ui.set_versions_is_loading(true);
                    ui.set_show_rom_versions(true);
                }

                tokio::spawn(async move {
                    let scraper = { state_clone.lock().unwrap().scraper.clone() };
                    if game.mfile_id.is_none() {
                        let _ = scraper.fetch_game_page_details(&mut game).await;
                    }

                    let versions = if let Some(mid) = &game.mfile_id {
                        scraper
                            .fetch_rom_versions(&game.section, &game.console_slug, mid)
                            .await
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };

                    {
                        let mut st = state_clone.lock().unwrap();
                        st.selected_game_for_versions = Some(game.clone());
                        st.rom_versions = versions.clone();
                    }

                    let ui_weak = ui_weak_clone.clone();
                    let slint_versions: Vec<RomVersionData> = versions
                        .iter()
                        .map(|v| RomVersionData {
                            fid: v.fid.clone().into(),
                            name: v.name.clone().into(),
                            size: v.size.clone().into(),
                            category: v.category.clone().into(),
                            region_or_type: v.region_or_type.clone().into(),
                            download_url: v.download_url.clone().into(),
                        })
                        .collect();

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_versions_is_loading(false);
                            ui.set_modal_rom_versions(ModelRc::new(VecModel::from(slint_versions)));
                        }
                    });
                });
            }
        });
    }

    // Download Version Selected
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_download_version_selected(move |game_id, fid| {
            let (target_game, version) = {
                let state = state.lock().unwrap();
                let g = state
                    .catalog_games
                    .iter()
                    .chain(state.search_results.iter())
                    .chain(state.favorite_games.iter())
                    .find(|g| g.id == game_id.as_str())
                    .cloned();
                let v = state.rom_versions.iter().find(|v| v.fid == fid.as_str()).cloned();
                (g, v)
            };

            if let Some(game) = target_game {
                let state_clone = state.clone();
                let ui_weak_clone = ui_weak.clone();

                tokio::spawn(async move {
                    start_download_async(state_clone, ui_weak_clone, game, version).await;
                });
            }
        });
    }

    // Favorite Toggled
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_game_favorite_toggled(move |game_id, is_fav| {
            let mut state = state.lock().unwrap();
            let mut target_game = None;
            for g in &mut state.catalog_games {
                if g.id == game_id.as_str() {
                    g.is_favorite = is_fav;
                    target_game = Some(g.clone());
                }
            }
            for g in &mut state.search_results {
                if g.id == game_id.as_str() {
                    g.is_favorite = is_fav;
                    if target_game.is_none() {
                        target_game = Some(g.clone());
                    }
                }
            }
            for g in &mut state.favorite_games {
                if g.id == game_id.as_str() {
                    g.is_favorite = is_fav;
                    if target_game.is_none() {
                        target_game = Some(g.clone());
                    }
                }
            }
            if let Some(g) = target_game {
                let _ = state.db.set_game_favorite(&g, is_fav);
                reload_favorites(&mut state);
                sync_displayed_games_to_ui(&ui_weak, &state);
            }
        });
    }

    // Play Game Clicked
    {
        let state = app_state.clone();
        main_window.on_game_play_clicked(move |game_id| {
            let target_game = {
                let state = state.lock().unwrap();
                state
                    .catalog_games
                    .iter()
                    .chain(state.search_results.iter())
                    .chain(state.favorite_games.iter())
                    .find(|g| g.id == game_id.as_str())
                    .cloned()
            };

            if let Some(game) = target_game {
                if let Some(path) = &game.local_file_path {
                    let state = state.lock().unwrap();
                    play_rom_path(&state.settings, &game.console_slug, Path::new(path));
                }
            }
        });
    }

    // Reveal File Clicked
    {
        let state = app_state.clone();
        main_window.on_game_reveal_clicked(move |game_id| {
            let target_game = {
                let state = state.lock().unwrap();
                state
                    .catalog_games
                    .iter()
                    .chain(state.search_results.iter())
                    .chain(state.favorite_games.iter())
                    .find(|g| g.id == game_id.as_str())
                    .cloned()
            };

            if let Some(game) = target_game {
                if let Some(path) = &game.local_file_path {
                    let _ = reveal_in_file_explorer(Path::new(path));
                }
            }
        });
    }

    // Cancel Download
    {
        let state = app_state.clone();
        main_window.on_cancel_download(move |dl_id| {
            let mut state = state.lock().unwrap();
            if let Some(token) = state.download_tokens.remove(&(dl_id as i64)) {
                token.store(true, Ordering::SeqCst);
            }
            state.active_downloads.remove(&(dl_id as i64));
            let _ = state.db.update_download_status(dl_id as i64, DownloadStatus::Cancelled, None, None);
            reload_downloads_history(&mut state);
        });
    }

    // Play Downloaded File from History
    {
        let state = app_state.clone();
        main_window.on_play_downloaded_file(move |file_path| {
            let p = PathBuf::from(file_path.as_str());
            let state = state.lock().unwrap();
            let console_slug = state
                .download_history
                .iter()
                .find(|r| r.local_path.as_deref() == Some(file_path.as_str()))
                .map(|r| r.console_slug.clone())
                .unwrap_or_default();
            play_rom_path(&state.settings, &console_slug, &p);
        });
    }

    // Reveal Downloaded File in Explorer
    {
        main_window.on_reveal_downloaded_file(move |file_path| {
            let _ = reveal_in_file_explorer(Path::new(file_path.as_str()));
        });
    }

    // Delete History Item
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_delete_history_item(move |item_id| {
            let mut state = state.lock().unwrap();
            let _ = state.db.delete_download_record(item_id as i64);
            reload_downloads_history(&mut state);
            sync_downloads_to_ui(&ui_weak, &state);
        });
    }

    // Clear All Download History
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_clear_download_history(move || {
            let mut state = state.lock().unwrap();
            for item in &state.download_history {
                let _ = state.db.delete_download_record(item.id);
            }
            reload_downloads_history(&mut state);
            sync_downloads_to_ui(&ui_weak, &state);
        });
    }

    // Zip Modal Toggle Entry
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_zip_toggle_entry(move |idx| {
            let mut state = state.lock().unwrap();
            if let Some(req) = &mut state.current_zip_request {
                if let Some(entry) = req.entries.get_mut(idx as usize) {
                    entry.is_selected = !entry.is_selected;
                }
                let selected_count = req.entries.iter().filter(|e| e.is_selected).count() as i32;
                let entries_slint: Vec<ZipEntryData> = req
                    .entries
                    .iter()
                    .map(|e| ZipEntryData {
                        entry_name: e.entry_name.clone().into(),
                        display_name: e.display_name.clone().into(),
                        formatted_size: e.formatted_size.clone().into(),
                        is_selected: e.is_selected,
                    })
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_zip_selected_count(selected_count);
                    ui.set_zip_entries(ModelRc::new(VecModel::from(entries_slint)));
                }
            }
        });
    }

    // Zip Modal Select All
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_zip_select_all(move || {
            let mut state = state.lock().unwrap();
            if let Some(req) = &mut state.current_zip_request {
                for e in &mut req.entries {
                    e.is_selected = true;
                }
                let count = req.entries.len() as i32;
                let entries_slint: Vec<ZipEntryData> = req
                    .entries
                    .iter()
                    .map(|e| ZipEntryData {
                        entry_name: e.entry_name.clone().into(),
                        display_name: e.display_name.clone().into(),
                        formatted_size: e.formatted_size.clone().into(),
                        is_selected: true,
                    })
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_zip_selected_count(count);
                    ui.set_zip_entries(ModelRc::new(VecModel::from(entries_slint)));
                }
            }
        });
    }

    // Zip Modal Deselect All
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_zip_deselect_all(move || {
            let mut state = state.lock().unwrap();
            if let Some(req) = &mut state.current_zip_request {
                for e in &mut req.entries {
                    e.is_selected = false;
                }
                let entries_slint: Vec<ZipEntryData> = req
                    .entries
                    .iter()
                    .map(|e| ZipEntryData {
                        entry_name: e.entry_name.clone().into(),
                        display_name: e.display_name.clone().into(),
                        formatted_size: e.formatted_size.clone().into(),
                        is_selected: false,
                    })
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_zip_selected_count(0);
                    ui.set_zip_entries(ModelRc::new(VecModel::from(entries_slint)));
                }
            }
        });
    }

    // Zip Modal Select Only RUS/USA
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_zip_select_rus_usa(move || {
            let mut state = state.lock().unwrap();
            if let Some(req) = &mut state.current_zip_request {
                for e in &mut req.entries {
                    let n = e.display_name.to_lowercase();
                    e.is_selected = n.contains("(r)")
                        || n.contains("rus")
                        || n.contains("(u)")
                        || n.contains("usa");
                }
                let count = req.entries.iter().filter(|e| e.is_selected).count() as i32;
                let entries_slint: Vec<ZipEntryData> = req
                    .entries
                    .iter()
                    .map(|e| ZipEntryData {
                        entry_name: e.entry_name.clone().into(),
                        display_name: e.display_name.clone().into(),
                        formatted_size: e.formatted_size.clone().into(),
                        is_selected: e.is_selected,
                    })
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_zip_selected_count(count);
                    ui.set_zip_entries(ModelRc::new(VecModel::from(entries_slint)));
                }
            }
        });
    }

    // Zip Modal Confirm Extract
    {
        let state = app_state.clone();
        main_window.on_zip_confirm(move || {
            let mut state = state.lock().unwrap();
            if let Some(req) = state.current_zip_request.take() {
                let selected: Vec<String> = req
                    .entries
                    .iter()
                    .filter(|e| e.is_selected)
                    .map(|e| e.entry_name.clone())
                    .collect();
                let del = state.settings.delete_zip_after_unpack;
                state
                    .download_manager
                    .extract_multi_rom_selection(req, selected, del);
            }
        });
    }

    // Settings: Browse Download Directory
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_settings_browse_download_dir(move || {
            if let Some(folder) = pick_download_folder() {
                let mut state = state.lock().unwrap();
                state.settings.download_directory = folder.to_string_lossy().to_string();
                let _ = state.settings.save();
                sync_settings_to_ui(&ui_weak.upgrade().unwrap(), &state);
            }
        });
    }

    // Settings: Auto Unpack Toggled
    {
        let state = app_state.clone();
        main_window.on_settings_auto_unpack_toggled(move |val| {
            let mut state = state.lock().unwrap();
            state.settings.auto_unpack_zip = val;
            let _ = state.settings.save();
        });
    }

    // Settings: Delete Zip Toggled
    {
        let state = app_state.clone();
        main_window.on_settings_delete_zip_toggled(move |val| {
            let mut state = state.lock().unwrap();
            state.settings.delete_zip_after_unpack = val;
            let _ = state.settings.save();
        });
    }

    // Settings: Theme Selected
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_settings_theme_selected(move |theme_name| {
            let mut state = state.lock().unwrap();
            state.settings.active_theme = theme_name.to_string();
            let _ = state.settings.save();
            if let Some(ui) = ui_weak.upgrade() {
                apply_theme_to_ui(&ui, &theme_name);
                ui.set_settings_active_theme(theme_name);
            }
        });
    }

    // Settings: RetroArch Toggled
    {
        let state = app_state.clone();
        main_window.on_settings_use_retroarch_toggled(move |val| {
            let mut state = state.lock().unwrap();
            state.settings.use_retroarch = val;
            let _ = state.settings.save();
        });
    }

    // Settings: Browse RetroArch Exe
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_settings_browse_retroarch_exe(move || {
            if let Some(exe) = pick_executable_file("Выберите исполняемый файл RetroArch") {
                let mut state = state.lock().unwrap();
                state.settings.retroarch_path = exe.to_string_lossy().to_string();
                let _ = state.settings.save();
                sync_settings_to_ui(&ui_weak.upgrade().unwrap(), &state);
            }
        });
    }

    // Settings: Browse Console Emulator Exe
    {
        let state = app_state.clone();
        let ui_weak = main_window.as_weak();
        main_window.on_settings_browse_emu(move |slug| {
            if let Some(exe) = pick_executable_file(&format!("Выберите эмулятор для {}", slug)) {
                let mut state = state.lock().unwrap();
                state
                    .settings
                    .emulator_paths
                    .insert(slug.to_string(), exe.to_string_lossy().to_string());
                let _ = state.settings.save();
                sync_settings_to_ui(&ui_weak.upgrade().unwrap(), &state);
            }
        });
    }

    // Image Viewer Controls
    {
        let ui_weak = main_window.as_weak();
        main_window.on_image_viewer_opened(move |title, img| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_viewer_title(title);
                ui.set_viewer_img(img);
                ui.set_viewer_zoom(1.0);
                ui.set_show_image_viewer(true);
            }
        });
    }

    {
        let ui_weak = main_window.as_weak();
        main_window.on_image_zoom_in(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let z = (ui.get_viewer_zoom() * 1.25).min(5.0);
                ui.set_viewer_zoom(z);
            }
        });
    }

    {
        let ui_weak = main_window.as_weak();
        main_window.on_image_zoom_out(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let z = (ui.get_viewer_zoom() * 0.8).max(0.2);
                ui.set_viewer_zoom(z);
            }
        });
    }

    {
        let ui_weak = main_window.as_weak();
        main_window.on_image_zoom_reset(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_viewer_zoom(1.0);
            }
        });
    }

    // Run Slint Event Loop
    main_window.run()?;
    Ok(())
}

// Helper Functions

impl AppState {
    pub fn set_status(&mut self, text: &str) {
        println!("[STATUS] {}", text);
    }
}

fn trigger_load_games(
    state: Arc<Mutex<AppState>>,
    ui_weak: slint::Weak<MainWindow>,
    page: usize,
) {
    let (console, category) = {
        let mut st = state.lock().unwrap();
        st.is_loading_games = true;
        st.current_page = page;
        let c = st
            .consoles
            .get(st.selected_console_idx)
            .cloned()
            .unwrap_or_else(|| st.consoles[0].clone());
        let cat = st.selected_category.clone();
        (c, cat)
    };

    if let Some(ui) = ui_weak.upgrade() {
        ui.set_is_loading_games(true);
        ui.set_status_text(format!("Загрузка каталога: {}…", console.short_name).into());
    }

    tokio::spawn(async move {
        let scraper = { state.lock().unwrap().scraper.clone() };
        let result = scraper
            .fetch_games_page(&console.slug, &console.name, &console.section, &category, page)
            .await;

        let mut st = state.lock().unwrap();
        st.is_loading_games = false;

        match result {
            Ok(page_result) => {
                st.current_page = page_result.current_page;
                st.total_pages = page_result.total_pages;
                if !page_result.available_categories.is_empty() {
                    st.categories = page_result.available_categories;
                }

                let fav_ids: std::collections::HashSet<String> =
                    st.favorite_games.iter().map(|g| g.id.clone()).collect();
                let mut games = page_result.games;
                for g in &mut games {
                    if fav_ids.contains(&g.id) {
                        g.is_favorite = true;
                    }
                }
                st.catalog_games = games;
                sync_download_status_on_games(&mut st);

                let count = st.catalog_games.len();
                let status_msg = format!("Загружено {} игр ({})", count, console.short_name);
                st.set_status(&status_msg);

                let cur_p = st.current_page as i32;
                let tot_p = st.total_pages as i32;
                let ui_weak = ui_weak.clone();
                let state_ref = state.clone();
                drop(st);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let st = state_ref.lock().unwrap();
                        let games_slint = build_slint_games(&st);
                        let cats_slint: Vec<CategoryData> = st
                            .categories
                            .iter()
                            .map(|c| CategoryData {
                                key: c.key.clone().into(),
                                label: c.label.clone().into(),
                                is_selected: c.key == st.selected_category,
                            })
                            .collect();
                        ui.set_is_loading_games(false);
                        ui.set_current_page(cur_p);
                        ui.set_total_pages(tot_p);
                        ui.set_catalog_games(ModelRc::new(VecModel::from(games_slint)));
                        ui.set_extra_categories(ModelRc::new(VecModel::from(cats_slint)));
                        ui.set_status_text(status_msg.into());
                    }
                });
            }
            Err(e) => {
                let err_msg = format!("Ошибка загрузки: {}", e);
                let ui_weak = ui_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_is_loading_games(false);
                        ui.set_status_text(err_msg.into());
                    }
                });
            }
        }
    });
}

async fn start_download_async(
    state: Arc<Mutex<AppState>>,
    ui_weak: slint::Weak<MainWindow>,
    game: GameCard,
    version: Option<RomFileVersion>,
) {
    let (target_dir, auto_unpack, delete_zip, scraper, console_folder) = {
        let st = state.lock().unwrap();
        let c_folder = st
            .consoles
            .iter()
            .find(|c| c.slug == game.console_slug)
            .map(|c| c.folder_name.clone())
            .unwrap_or_else(|| "ROMs".to_string());
        (
            PathBuf::from(&st.settings.download_directory),
            st.settings.auto_unpack_zip,
            st.settings.delete_zip_after_unpack,
            st.scraper.clone(),
            c_folder,
        )
    };

    let download_url = if let Some(v) = &version {
        v.download_url.clone()
    } else if let Some(mfile_id) = &game.mfile_id {
        let subpath = retroms_desktop::scraper::get_game_subpath(&game.console_slug);
        let versions = scraper
            .fetch_rom_versions(&game.section, &game.console_slug, mfile_id)
            .await
            .unwrap_or_default();
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
        game.section,
        game.console_slug,
        retroms_desktop::scraper::get_game_subpath(&game.console_slug)
    );

    let direct_url = scraper
        .get_direct_download_url(&download_url, &referer)
        .await
        .unwrap_or(download_url);

    let raw_title = game
        .title
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let safe_title = raw_title.trim_end_matches('.').trim().to_string();
    let file_name = if let Some(v_name) = version.as_ref().map(|v| v.name.clone()) {
        v_name
    } else {
        format!("{}.zip", safe_title)
    };

    let record_id = {
        let st = state.lock().unwrap();
        st.db
            .add_download_record(
                &game.id,
                &game.title,
                &game.console_slug,
                &game.console_name,
                &file_name,
                &target_dir.to_string_lossy(),
                &direct_url,
            )
            .unwrap_or(0)
    };

    let cancel_token = Arc::new(AtomicBool::new(false));
    {
        let mut st = state.lock().unwrap();
        st.download_tokens.insert(record_id, cancel_token.clone());
        let dl_record = DownloadRecord {
            id: record_id,
            game_id: game.id.clone(),
            game_title: game.title.clone(),
            console_slug: game.console_slug.clone(),
            console_name: game.console_name.clone(),
            file_name: file_name.clone(),
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
        st.active_downloads.insert(record_id, dl_record);
        sync_downloads_to_ui(&ui_weak, &st);
    }

    let download_mgr = { state.lock().unwrap().download_manager.clone() };
    download_mgr.download_game(
        record_id,
        game,
        Some(file_name),
        direct_url,
        target_dir,
        console_folder,
        auto_unpack,
        delete_zip,
        cancel_token,
    );
}

fn reload_favorites(state: &mut AppState) {
    if let Ok(favs) = state.db.get_favorite_games() {
        state.favorite_games = favs;
    }
}

fn reload_downloads_history(state: &mut AppState) {
    if let Ok(history) = state.db.get_download_history() {
        state.download_history = history;
    }
}

fn sync_download_status_on_games(state: &mut AppState) {
    let download_dir = PathBuf::from(&state.settings.download_directory);

    let mut completed_map: HashMap<String, String> = HashMap::new();
    for r in &state.download_history {
        if r.status == DownloadStatus::Completed {
            if let Some(path_str) = &r.local_path {
                let p = Path::new(path_str);
                if p.exists() {
                    completed_map.insert(r.game_id.clone(), path_str.clone());
                    let key = format!("{}:{}", r.console_slug, r.game_title.to_lowercase());
                    completed_map.insert(key, path_str.clone());
                }
            }
        }
    }

    let update = |g: &mut GameCard| {
        let key = format!("{}:{}", g.console_slug, g.title.to_lowercase());
        if let Some(path) = completed_map.get(&g.id).or_else(|| completed_map.get(&key)) {
            g.is_downloaded = true;
            g.local_file_path = Some(path.clone());
            return;
        }

        // Direct file search in download dir
        let folder = &g.console_slug;
        let cand = download_dir.join(folder).join(format!("{}.zip", g.title));
        if cand.exists() {
            g.is_downloaded = true;
            g.local_file_path = Some(cand.to_string_lossy().to_string());
            return;
        }

        g.is_downloaded = false;
        g.local_file_path = None;
    };

    for g in &mut state.catalog_games {
        update(g);
    }
    for g in &mut state.search_results {
        update(g);
    }
    for g in &mut state.favorite_games {
        update(g);
    }
}

fn apply_theme_to_ui(ui: &MainWindow, theme_str: &str) {
    let preset = ThemePreset::from_str(theme_str);
    Theme::get(ui).set_primary(preset.primary_color());
    Theme::get(ui).set_secondary(preset.secondary_color());
    Theme::get(ui).set_window_bg(preset.window_bg_color());
    Theme::get(ui).set_card_bg(preset.card_bg_color());
    Theme::get(ui).set_card_border(preset.card_border_color());
}

fn sync_consoles_to_ui(ui: &MainWindow, state: &AppState) {
    let consoles_slint: Vec<ConsoleData> = state
        .consoles
        .iter()
        .enumerate()
        .map(|(idx, c)| ConsoleData {
            slug: c.slug.clone().into(),
            name: c.name.clone().into(),
            short_name: c.short_name.clone().into(),
            category: c.category.clone().into(),
            folder_name: c.folder_name.clone().into(),
            year: c.release_year.clone().into(),
            icon_key: c.icon_key.clone().into(),
            roms_count: c.roms_count_estimate.clone().into(),
            is_enabled: c.is_enabled,
            is_selected: idx == state.selected_console_idx,
        })
        .collect();

    ui.set_consoles(ModelRc::new(VecModel::from(consoles_slint)));
    ui.set_selected_console_idx(state.selected_console_idx as i32);
}

fn sync_settings_to_ui(ui: &MainWindow, state: &AppState) {
    ui.set_settings_download_dir(state.settings.download_directory.clone().into());
    ui.set_settings_auto_unpack(state.settings.auto_unpack_zip);
    ui.set_settings_delete_zip(state.settings.delete_zip_after_unpack);
    ui.set_settings_active_theme(state.settings.active_theme.clone().into());
    ui.set_settings_use_retroarch(state.settings.use_retroarch);
    ui.set_settings_retroarch_path(state.settings.retroarch_path.clone().into());
    ui.set_settings_retroarch_args(state.settings.retroarch_args.clone().into());

    let get_emu = |slug: &str| {
        state
            .settings
            .emulator_paths
            .get(slug)
            .cloned()
            .unwrap_or_default()
            .into()
    };
    ui.set_settings_emu_nes(get_emu("dendy"));
    ui.set_settings_emu_genesis(get_emu("genesis"));
    ui.set_settings_emu_snes(get_emu("snes"));
    ui.set_settings_emu_ps1(get_emu("psx"));
    ui.set_settings_emu_gba(get_emu("gba"));
    ui.set_settings_emu_n64(get_emu("n64"));
}

fn sync_downloads_to_ui(ui_weak: &slint::Weak<MainWindow>, state: &AppState) {
    let active_slint: Vec<DownloadItemData> = state
        .active_downloads
        .values()
        .map(download_record_to_slint)
        .collect();

    let history_slint: Vec<DownloadItemData> = state
        .download_history
        .iter()
        .map(download_record_to_slint)
        .collect();

    let active_count = active_slint.len() as i32;
    let ui_weak = ui_weak.clone();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_active_downloads(ModelRc::new(VecModel::from(active_slint)));
            ui.set_download_history(ModelRc::new(VecModel::from(history_slint)));
            ui.set_active_downloads_count(active_count);
        }
    });
}

fn sync_displayed_games_to_ui(ui_weak: &slint::Weak<MainWindow>, state: &AppState) {
    if let Some(ui) = ui_weak.upgrade() {
        let games_slint = build_slint_games(state);
        let favs_slint = build_slint_favorites(state);
        let fav_count = favs_slint.len() as i32;
        ui.set_catalog_games(ModelRc::new(VecModel::from(games_slint)));
        ui.set_favorite_games(ModelRc::new(VecModel::from(favs_slint)));
        ui.set_favorites_count(fav_count);
    }
}

fn build_slint_games(state: &AppState) -> Vec<GameCardData> {
    let source_games = if state.is_search_mode {
        let mut list = state.search_results.clone();
        if let Some(filter) = &state.search_platform_filter {
            list.retain(|g| g.console_slug.eq_ignore_ascii_case(filter));
        }
        list
    } else {
        state.catalog_games.clone()
    };

    let mut sorted = source_games;
    match state.sort_option {
        SortOption::RatingDesc => sorted.sort_by(|a, b| {
            b.rating
                .partial_cmp(&a.rating)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        SortOption::TitleAsc => {
            sorted.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        }
        SortOption::TitleDesc => {
            sorted.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase()))
        }
        SortOption::Default => {}
    }

    sorted
        .iter()
        .map(|g| game_card_to_slint(g, &state.image_cache))
        .collect()
}

fn build_slint_favorites(state: &AppState) -> Vec<GameCardData> {
    state
        .favorite_games
        .iter()
        .map(|g| game_card_to_slint(g, &state.image_cache))
        .collect()
}

fn build_search_platforms(state: &AppState) -> Vec<PlatformFilterData> {
    let mut map: BTreeMap<String, (String, usize)> = BTreeMap::new();
    for g in &state.search_results {
        let entry = map.entry(g.console_slug.clone()).or_insert_with(|| {
            let short = g
                .console_name
                .split('/')
                .next()
                .unwrap_or(&g.console_name)
                .trim()
                .to_string();
            (short, 0)
        });
        entry.1 += 1;
    }

    map.into_iter()
        .map(|(slug, (label, count))| {
            let is_selected = state
                .search_platform_filter
                .as_ref()
                .map(|s| s == &slug)
                .unwrap_or(false);
            PlatformFilterData {
                slug: slug.into(),
                label: label.into(),
                count: count as i32,
                is_selected,
            }
        })
        .collect()
}

fn game_card_to_slint(card: &GameCard, image_cache: &ImageCacheManager) -> GameCardData {
    let (badge_text, subname) = get_console_badge_info(&card.console_slug);
    let format_label = get_format_label(&card.console_slug);
    let region_label = get_region_label(card);

    let (cover_image, has_cover) = if let Some(img) = image_cache.get_cached_image(&card.id) {
        (img, true)
    } else {
        if let Some(cover_url) = &card.cover_url {
            image_cache.queue_download(&card.id, cover_url);
        }
        (slint::Image::default(), false)
    };

    GameCardData {
        id: card.id.clone().into(),
        title: card.title.clone().into(),
        original_title: card.original_title.clone().unwrap_or_default().into(),
        console_slug: card.console_slug.clone().into(),
        console_name: card.console_name.clone().into(),
        genre: card.genre.clone().into(),
        year: card.year.clone().into(),
        publisher: card.publisher.clone().into(),
        developer: card.developer.clone().into(),
        rating: card.rating,
        rating_str: if card.rating > 0.0 {
            format!("{:.1}", card.rating).into()
        } else {
            "4.8".into()
        },
        file_size: card.file_size.clone().into(),
        badge_text: badge_text.into(),
        subname: subname.into(),
        format_label: format_label.into(),
        region_label: region_label.into(),
        description: card.description.clone().into(),
        cover_image,
        has_cover,
        is_favorite: card.is_favorite,
        is_downloaded: card.is_downloaded,
        local_path: card.local_file_path.clone().unwrap_or_default().into(),
    }
}

fn download_record_to_slint(record: &DownloadRecord) -> DownloadItemData {
    let progress = if record.total_bytes > 0 {
        (record.downloaded_bytes as f32 / record.total_bytes as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let progress_text = format!(
        "{:.1}% • {} / {}",
        progress * 100.0,
        retroms_desktop::archive::format_bytes(record.downloaded_bytes),
        retroms_desktop::archive::format_bytes(record.total_bytes)
    );
    let speed_text = format!(
        "{}/s",
        retroms_desktop::archive::format_bytes(record.speed_bytes_sec)
    );

    let (status_text, status_code) = match record.status {
        DownloadStatus::Pending => ("В очереди", 0),
        DownloadStatus::Downloading => ("Загрузка", 1),
        DownloadStatus::Completed => ("Готово", 2),
        DownloadStatus::Failed => ("Ошибка", 3),
        DownloadStatus::Cancelled => ("Отменено", 4),
    };

    DownloadItemData {
        id: record.id as i32,
        game_id: record.game_id.clone().into(),
        game_title: record.game_title.clone().into(),
        console_name: record.console_name.clone().into(),
        file_name: record.file_name.clone().into(),
        progress,
        progress_text: progress_text.into(),
        speed_text: speed_text.into(),
        status_text: status_text.into(),
        status_code,
        is_completed: record.status == DownloadStatus::Completed,
        local_path: record.local_path.clone().unwrap_or_default().into(),
    }
}

fn play_rom_path(settings: &AppSettings, console_slug: &str, path: &Path) {
    if settings.use_retroarch && !settings.retroarch_path.trim().is_empty() {
        let core = settings.get_retroarch_core_for_console(console_slug);
        let core_opt = if core.is_empty() { None } else { Some(core.as_str()) };
        let _ = launch_retroarch(
            &settings.retroarch_path,
            path,
            core_opt,
            &settings.retroarch_args,
        );
    } else if let Some(emu_path) = settings.emulator_paths.get(console_slug) {
        if !emu_path.trim().is_empty() {
            let emu_args = settings
                .emulator_args
                .get(console_slug)
                .cloned()
                .unwrap_or_default();
            let _ = launch_emulator(emu_path, path, &emu_args);
            return;
        }
        let _ = open::that(path);
    } else {
        let _ = open::that(path);
    }
}
