mod archive;
mod config;
mod db;
mod downloader;
mod integrations;
mod models;
mod scraper;
mod theme;
mod ui;

use archive::extract_selected_entries;
use config::AppSettings;
use db::Database;
use downloader::{DownloadEvent, DownloadManager};
use integrations::{launch_emulator, reveal_in_file_explorer};
use models::{
    CatalogCategory, ConsoleInfo, DownloadRecord, DownloadStatus, GameCard, RomFileVersion,
    ZipExtractionRequest,
};
use scraper::EmuLandClient;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use theme::ThemePreset;
use tokio::sync::mpsc;
use ui::catalog_grid::render_catalog_grid;
use ui::catalog_table::render_catalog_table;
use ui::downloads_view::render_downloads_view;
use ui::favorites_view::render_favorites_view;
use ui::game_detail::render_game_detail_window;
use ui::settings_view::render_settings_view;
use ui::sidebar::render_sidebar;
use ui::topbar::render_topbar;
use ui::zip_modal::render_zip_modal;
use ui::{NavTab, SortOption, ViewMode};

enum ScraperResponse {
    GamesLoaded {
        games: Vec<GameCard>,
        categories: Vec<CatalogCategory>,
        page: usize,
        total_pages: usize,
    },
    VersionsLoaded {
        game_id: String,
        versions: Vec<RomFileVersion>,
        description: String,
        screenshots: Vec<String>,
    },
    Error(String),
}

struct RetroRomsApp {
    settings: AppSettings,
    db: Arc<Database>,
    scraper: EmuLandClient,
    downloader: DownloadManager,

    // Navigation and state
    active_tab: NavTab,
    consoles: Vec<ConsoleInfo>,
    selected_console_idx: usize,
    categories: Vec<CatalogCategory>,
    selected_category: String,
    current_page: usize,
    total_pages: usize,
    games: Vec<GameCard>,
    search_query: String,
    sort_option: SortOption,
    view_mode: ViewMode,
    is_loading: bool,
    status_message: Option<(String, std::time::Instant)>,

    // Downloads
    active_downloads: Vec<DownloadRecord>,
    download_cancellations: HashMap<i64, Arc<AtomicBool>>,
    download_history: Vec<DownloadRecord>,
    download_rx: mpsc::UnboundedReceiver<DownloadEvent>,
    download_tx: mpsc::UnboundedSender<DownloadEvent>,

    // Scraper async channel
    scraper_rx: mpsc::UnboundedReceiver<ScraperResponse>,
    scraper_tx: mpsc::UnboundedSender<ScraperResponse>,

    // Modals
    selected_game_for_detail: Option<GameCard>,
    rom_versions: Vec<RomFileVersion>,
    is_loading_versions: bool,
    zip_extraction_request: Option<ZipExtractionRequest>,

    // Favorites
    favorites: Vec<GameCard>,
}

impl RetroRomsApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let settings = AppSettings::load();
        let db = Arc::new(Database::open().expect("Failed to open SQLite database"));
        let theme = ThemePreset::from_str(&settings.active_theme);
        theme.apply_to_ctx(&cc.egui_ctx);

        let consoles = db.get_consoles().unwrap_or_default();
        let download_history = db.get_download_history().unwrap_or_default();
        let favorites = db.get_favorite_games().unwrap_or_default();

        let (download_tx, download_rx) = mpsc::unbounded_channel();
        let (scraper_tx, scraper_rx) = mpsc::unbounded_channel();

        let mut app = Self {
            settings,
            db,
            scraper: EmuLandClient::new(),
            downloader: DownloadManager::new(),
            active_tab: NavTab::Catalog,
            consoles,
            selected_console_idx: 0,
            categories: Vec::new(),
            selected_category: "top".to_string(),
            current_page: 1,
            total_pages: 1,
            games: Vec::new(),
            search_query: String::new(),
            sort_option: SortOption::Default,
            view_mode: ViewMode::Grid,
            is_loading: false,
            status_message: None,
            active_downloads: Vec::new(),
            download_cancellations: HashMap::new(),
            download_history,
            download_rx,
            download_tx,
            scraper_rx,
            scraper_tx,
            selected_game_for_detail: None,
            rom_versions: Vec::new(),
            is_loading_versions: false,
            zip_extraction_request: None,
            favorites,
        };

        app.trigger_load_games(1, cc.egui_ctx.clone());
        app
    }

    fn trigger_load_games(&mut self, page: usize, ctx: egui::Context) {
        if self.consoles.is_empty() {
            return;
        }

        self.is_loading = true;
        self.current_page = page;

        let console = self.consoles[self.selected_console_idx].clone();
        let category = self.selected_category.clone();
        let scraper = self.scraper.clone();
        let tx = self.scraper_tx.clone();
        let db = Arc::clone(&self.db);

        tokio::spawn(async move {
            let categories = scraper
                .fetch_categories(&console.slug, &console.section)
                .await;

            match scraper
                .fetch_games_page(
                    &console.slug,
                    &console.name,
                    &console.section,
                    &category,
                    page,
                )
                .await
            {
                Ok(mut res) => {
                    for g in &mut res.games {
                        g.is_favorite = db.is_game_favorite(&g.id);
                    }
                    let _ = tx.send(ScraperResponse::GamesLoaded {
                        games: res.games,
                        categories,
                        page: res.current_page,
                        total_pages: res.total_pages,
                    });
                }
                Err(e) => {
                    let _ = tx.send(ScraperResponse::Error(format!(
                        "Не удалось загрузить игры: {}",
                        e
                    )));
                }
            }
            ctx.request_repaint();
        });
    }

    fn trigger_load_rom_versions(&mut self, game: GameCard, ctx: egui::Context) {
        self.selected_game_for_detail = Some(game.clone());
        self.rom_versions.clear();
        self.is_loading_versions = true;

        let scraper = self.scraper.clone();
        let tx = self.scraper_tx.clone();

        tokio::spawn(async move {
            let mut versions = Vec::new();
            if let Some(mfile_id) = &game.mfile_id {
                if let Ok(v) = scraper
                    .fetch_rom_versions(&game.console_slug, &game.section, mfile_id)
                    .await
                {
                    versions = v;
                }
            }

            let (desc, screenshots) = if let Some(slug) = &game.game_page_slug {
                scraper
                    .fetch_game_page_details(slug)
                    .await
                    .unwrap_or_default()
            } else {
                (String::new(), Vec::new())
            };

            let _ = tx.send(ScraperResponse::VersionsLoaded {
                game_id: game.id,
                versions,
                description: desc,
                screenshots,
            });
            ctx.request_repaint();
        });
    }

    fn start_game_download(&mut self, game: GameCard, version: Option<RomFileVersion>, ctx: egui::Context) {
        let file_name = version
            .as_ref()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| format!("{}.zip", game.title));

        let target_dir = PathBuf::from(&self.settings.download_directory);

        let record_id = self
            .db
            .add_download_record(
                &game.id,
                &game.title,
                &game.console_slug,
                &game.console_name,
                &file_name,
                &self.settings.download_directory,
                &game.download_url,
            )
            .unwrap_or(0);

        let cancellation = Arc::new(AtomicBool::new(false));
        self.download_cancellations
            .insert(record_id, cancellation.clone());

        self.downloader.start_download(
            record_id,
            game.clone(),
            version,
            target_dir,
            self.settings.auto_unpack_zip,
            self.settings.delete_zip_after_unpack,
            self.download_tx.clone(),
            cancellation,
            ctx,
        );

        self.set_status(format!("Начало загрузки: {}", file_name));
    }

    fn cancel_download(&mut self, record_id: i64) {
        if let Some(c) = self.download_cancellations.remove(&record_id) {
            c.store(true, Ordering::Relaxed);
        }
        let _ = self.db.update_download_status(
            record_id,
            DownloadStatus::Cancelled,
            None,
            Some("Отменено пользователем"),
        );
        self.active_downloads.retain(|d| d.id != record_id);
        self.reload_downloads_history();
    }

    fn reload_downloads_history(&mut self) {
        if let Ok(h) = self.db.get_download_history() {
            self.download_history = h;
        }
    }

    fn reload_favorites(&mut self) {
        if let Ok(favs) = self.db.get_favorite_games() {
            self.favorites = favs;
        }
    }

    fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, std::time::Instant::now()));
    }

    fn play_game(&mut self, game: &GameCard) {
        let path_to_play = game
            .local_file_path
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| {
                let folder = self.settings.get_console_folder(&game.console_slug);
                let candidates = [
                    folder.join(format!("{}.nes", game.title)),
                    folder.join(format!("{}.bin", game.title)),
                    folder.join(format!("{}.smd", game.title)),
                    folder.join(format!("{}.sfc", game.title)),
                    folder.join(format!("{}.zip", game.title)),
                ];
                candidates.into_iter().find(|p| p.exists())
            });

        if let Some(rom_path) = path_to_play {
            if let Some(emu_exe) = self.settings.emulator_paths.get(&game.console_slug) {
                if !emu_exe.is_empty() {
                    let custom_args = self.settings.emulator_args.get(&game.console_slug).map(|s| s.as_str());
                    if let Err(e) = launch_emulator(emu_exe, &rom_path, custom_args) {
                        self.set_status(format!("Ошибка запуска эмулятора: {}", e));
                    } else {
                        self.set_status(format!("Запущен {}", emu_exe));
                    }
                    return;
                }
            }
            let _ = reveal_in_file_explorer(&rom_path);
            self.set_status("Эмулятор не настроен. Файл показан в проводнике.".into());
        } else {
            self.set_status("Файл РОМа не найден на диске.".into());
        }
    }
}

impl eframe::App for RetroRomsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle Hotkeys
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
            self.active_tab = NavTab::Catalog;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.trigger_load_games(self.current_page, ctx.clone());
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_game_for_detail = None;
            self.zip_extraction_request = None;
        }

        // Drain background scraper events
        while let Ok(msg) = self.scraper_rx.try_recv() {
            self.is_loading = false;
            match msg {
                ScraperResponse::GamesLoaded {
                    games,
                    categories,
                    page,
                    total_pages,
                } => {
                    let count = games.len();
                    self.games = games;
                    if !categories.is_empty() {
                        self.categories = categories;
                    }
                    self.current_page = page;
                    self.total_pages = total_pages;
                    self.set_status(format!("✅ Загружено игр: {} (стр. {}/{})", count, page, total_pages));
                }
                ScraperResponse::VersionsLoaded {
                    game_id,
                    versions,
                    description,
                    screenshots,
                } => {
                    self.is_loading_versions = false;
                    self.rom_versions = versions;
                    if let Some(detail) = &mut self.selected_game_for_detail {
                        if detail.id == game_id {
                            if !description.is_empty() {
                                detail.description = description;
                            }
                            if !screenshots.is_empty() {
                                detail.screenshot_urls = screenshots;
                            }
                        }
                    }
                }
                ScraperResponse::Error(err) => {
                    self.set_status(format!("❌ {}", err));
                }
            }
        }

        // Drain download manager events
        while let Ok(event) = self.download_rx.try_recv() {
            match event {
                DownloadEvent::Started {
                    record_id,
                    game_id,
                    file_name,
                } => {
                    let console = &self.consoles[self.selected_console_idx];
                    self.active_downloads.push(DownloadRecord {
                        id: record_id,
                        game_id,
                        game_title: file_name.clone(),
                        console_slug: console.slug.clone(),
                        console_name: console.name.clone(),
                        file_name,
                        target_directory: self.settings.download_directory.clone(),
                        local_path: None,
                        total_bytes: 0,
                        downloaded_bytes: 0,
                        speed_bytes_sec: 0,
                        status: DownloadStatus::Downloading,
                        download_url: "".into(),
                        timestamp: chrono::Utc::now().timestamp(),
                        error_message: None,
                    });
                }
                DownloadEvent::Progress {
                    record_id,
                    downloaded_bytes,
                    total_bytes,
                    speed_bytes_sec,
                    _percent: _,
                } => {
                    if let Some(item) = self.active_downloads.iter_mut().find(|d| d.id == record_id) {
                        item.downloaded_bytes = downloaded_bytes;
                        item.total_bytes = total_bytes;
                        item.speed_bytes_sec = speed_bytes_sec;
                    }
                    let _ = self.db.update_download_progress(record_id, downloaded_bytes, total_bytes);
                }
                DownloadEvent::Completed {
                    record_id,
                    game_id,
                    file_path,
                    message,
                } => {
                    self.active_downloads.retain(|d| d.id != record_id);
                    self.download_cancellations.remove(&record_id);
                    let _ = self.db.update_download_status(
                        record_id,
                        DownloadStatus::Completed,
                        Some(&file_path),
                        None,
                    );
                    let _ = self.db.mark_game_downloaded(&game_id, true, Some(&file_path));

                    for g in &mut self.games {
                        if g.id == game_id {
                            g.is_downloaded = true;
                            g.local_file_path = Some(file_path.clone());
                        }
                    }
                    self.reload_downloads_history();
                    self.set_status(format!("✅ {}", message));
                }
                DownloadEvent::RequiresSelection(request) => {
                    self.active_downloads.retain(|d| d.id != request.record_id);
                    self.zip_extraction_request = Some(request);
                }
                DownloadEvent::Failed {
                    record_id,
                    _game_id: _,
                    error,
                } => {
                    self.active_downloads.retain(|d| d.id != record_id);
                    self.download_cancellations.remove(&record_id);
                    let _ = self.db.update_download_status(
                        record_id,
                        DownloadStatus::Failed,
                        None,
                        Some(&error),
                    );
                    self.reload_downloads_history();
                    self.set_status(format!("❌ {}", error));
                }
            }
        }

        let current_theme = ThemePreset::from_str(&self.settings.active_theme);

        // Top Status Bar (if any message)
        egui::TopBottomPanel::bottom("bottom_status_panel")
            .min_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some((msg, time)) = &self.status_message {
                        if time.elapsed().as_secs() < 8 {
                            ui.label(egui::RichText::new(msg).small().color(current_theme.primary_color()));
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Каталог: {} | Загружено игр: {}",
                                self.consoles.get(self.selected_console_idx).map(|c| c.name.as_str()).unwrap_or(""),
                                self.download_history.iter().filter(|d| d.status == DownloadStatus::Completed).count()
                            ))
                            .small()
                            .weak(),
                        );
                    });
                });
            });

        // Left Sidebar
        let mut console_changed = false;
        let mut manage_consoles_clicked = false;

        egui::SidePanel::left("left_sidebar_panel")
            .min_width(220.0)
            .max_width(280.0)
            .show(ctx, |ui| {
                render_sidebar(
                    ui,
                    &mut self.active_tab,
                    &mut self.consoles,
                    &mut self.selected_console_idx,
                    self.active_downloads.len(),
                    self.favorites.len(),
                    current_theme,
                    &mut console_changed,
                    &mut manage_consoles_clicked,
                );
            });

        if console_changed {
            self.selected_category = "top".to_string();
            self.trigger_load_games(1, ctx.clone());
            let _ = self.db.update_console_order_and_enabled(&self.consoles);
        }

        if manage_consoles_clicked {
            self.active_tab = NavTab::Settings;
        }

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                NavTab::Catalog => {
                    let mut category_changed = false;
                    let mut refresh_clicked = false;

                    render_topbar(
                        ui,
                        &mut self.search_query,
                        &self.categories,
                        &mut self.selected_category,
                        &mut self.view_mode,
                        &mut self.sort_option,
                        self.is_loading,
                        current_theme,
                        &mut category_changed,
                        &mut refresh_clicked,
                    );

                    if category_changed || refresh_clicked {
                        self.trigger_load_games(1, ctx.clone());
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if self.is_loading && self.games.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(60.0);
                            ui.spinner();
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("Загрузка игр с Emu-Land.net...")
                                    .color(current_theme.primary_color())
                                    .size(16.0),
                            );
                        });
                        return;
                    }

                    // Filter and Sort Games
                    let mut display_games = self.games.clone();
                    if !self.search_query.trim().is_empty() {
                        let query = self.search_query.to_lowercase();
                        display_games.retain(|g| {
                            g.title.to_lowercase().contains(&query)
                                || g.console_name.to_lowercase().contains(&query)
                        });
                    }

                    match self.sort_option {
                        SortOption::RatingDesc => {
                            display_games.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
                        }
                        SortOption::TitleAsc => {
                            display_games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
                        }
                        SortOption::TitleDesc => {
                            display_games.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase()));
                        }
                        SortOption::Default => {}
                    }

                    let mut game_clicked = None;
                    let mut download_clicked = None;
                    let mut favorite_toggled = None;
                    let mut play_clicked = None;
                    let mut page_changed = None;

                    match self.view_mode {
                        ViewMode::Grid => {
                            render_catalog_grid(
                                ui,
                                &display_games,
                                self.current_page,
                                self.total_pages,
                                current_theme,
                                &mut game_clicked,
                                &mut download_clicked,
                                &mut favorite_toggled,
                                &mut play_clicked,
                                &mut page_changed,
                            );
                        }
                        ViewMode::Table => {
                            render_catalog_table(
                                ui,
                                &display_games,
                                self.current_page,
                                self.total_pages,
                                current_theme,
                                &mut game_clicked,
                                &mut download_clicked,
                                &mut favorite_toggled,
                                &mut play_clicked,
                                &mut page_changed,
                            );
                        }
                    }

                    if let Some(g) = game_clicked {
                        self.trigger_load_rom_versions(g, ctx.clone());
                    }
                    if let Some(g) = download_clicked {
                        self.trigger_load_rom_versions(g, ctx.clone());
                    }
                    if let Some((g, is_fav)) = favorite_toggled {
                        let _ = self.db.set_game_favorite(&g, is_fav);
                        for item in &mut self.games {
                            if item.id == g.id {
                                item.is_favorite = is_fav;
                            }
                        }
                        self.reload_favorites();
                    }
                    if let Some(g) = play_clicked {
                        self.play_game(&g);
                    }
                    if let Some(p) = page_changed {
                        self.trigger_load_games(p, ctx.clone());
                    }
                }
                NavTab::Downloads => {
                    let mut cancel_id = None;
                    let mut play_file = None;
                    let mut reveal_file = None;
                    let mut delete_id = None;
                    let mut clear_all = false;

                    render_downloads_view(
                        ui,
                        &self.active_downloads,
                        &self.download_history,
                        current_theme,
                        &mut cancel_id,
                        &mut play_file,
                        &mut reveal_file,
                        &mut delete_id,
                        &mut clear_all,
                    );

                    if let Some(id) = cancel_id {
                        self.cancel_download(id);
                    }
                    if let Some(path) = play_file {
                        let path_obj = Path::new(&path);
                        let _ = reveal_in_file_explorer(path_obj);
                    }
                    if let Some(path) = reveal_file {
                        let path_obj = Path::new(&path);
                        let _ = reveal_in_file_explorer(path_obj);
                    }
                    if let Some(id) = delete_id {
                        let _ = self.db.delete_download_record(id);
                        self.reload_downloads_history();
                    }
                    if clear_all {
                        for h in &self.download_history {
                            let _ = self.db.delete_download_record(h.id);
                        }
                        self.reload_downloads_history();
                    }
                }
                NavTab::Favorites => {
                    let mut game_clicked = None;
                    let mut download_clicked = None;
                    let mut favorite_toggled = None;
                    let mut play_clicked = None;

                    render_favorites_view(
                        ui,
                        &self.favorites,
                        self.view_mode,
                        current_theme,
                        &mut game_clicked,
                        &mut download_clicked,
                        &mut favorite_toggled,
                        &mut play_clicked,
                    );

                    if let Some(g) = game_clicked {
                        self.trigger_load_rom_versions(g, ctx.clone());
                    }
                    if let Some(g) = download_clicked {
                        self.trigger_load_rom_versions(g, ctx.clone());
                    }
                    if let Some((g, is_fav)) = favorite_toggled {
                        let _ = self.db.set_game_favorite(&g, is_fav);
                        self.reload_favorites();
                    }
                    if let Some(g) = play_clicked {
                        self.play_game(&g);
                    }
                }
                NavTab::Settings => {
                    let mut consoles_updated = false;
                    let mut theme_changed = None;

                    render_settings_view(
                        ui,
                        &mut self.settings,
                        &mut self.consoles,
                        current_theme,
                        &mut consoles_updated,
                        &mut theme_changed,
                    );

                    if consoles_updated {
                        let _ = self.db.update_console_order_and_enabled(&self.consoles);
                    }
                    if let Some(t_str) = theme_changed {
                        let t = ThemePreset::from_str(&t_str);
                        t.apply_to_ctx(ctx);
                        let _ = self.settings.save();
                    }
                }
            }
        });

        // Modals
        let mut download_version_selected = None;
        let mut modal_play_clicked = None;
        let mut modal_reveal_clicked = None;

        render_game_detail_window(
            ctx,
            &mut self.selected_game_for_detail,
            &self.rom_versions,
            self.is_loading_versions,
            current_theme,
            &mut download_version_selected,
            &mut modal_play_clicked,
            &mut modal_reveal_clicked,
        );

        if let Some((game, ver)) = download_version_selected {
            self.start_game_download(game, Some(ver), ctx.clone());
        }
        if let Some(g) = modal_play_clicked {
            self.play_game(&g);
        }
        if let Some(path) = modal_reveal_clicked {
            let _ = reveal_in_file_explorer(Path::new(&path));
        }

        // Multi-file ZIP extraction modal
        let mut extract_confirmed = None;
        render_zip_modal(
            ctx,
            &mut self.zip_extraction_request,
            current_theme,
            &mut extract_confirmed,
        );

        if let Some((req, selected_names)) = extract_confirmed {
            let zip_path = PathBuf::from(&req.temp_zip_path);
            let target_folder = PathBuf::from(&req.target_directory);
            let delete_zip = self.settings.delete_zip_after_unpack;

            match extract_selected_entries(&zip_path, &selected_names, &target_folder, delete_zip) {
                Ok(files) => {
                    let first_path = files.first().map(|p| p.to_string_lossy().to_string());
                    let _ = self.db.update_download_status(
                        req.record_id,
                        DownloadStatus::Completed,
                        first_path.as_deref(),
                        None,
                    );
                    let _ = self.db.mark_game_downloaded(
                        &req.game.id,
                        true,
                        first_path.as_deref(),
                    );
                    self.reload_downloads_history();
                    self.set_status(format!(
                        "✅ Извлечено файлов: {} в {}",
                        files.len(),
                        target_folder.display()
                    ));
                }
                Err(e) => {
                    let _ = self.db.update_download_status(
                        req.record_id,
                        DownloadStatus::Failed,
                        None,
                        Some(&e.to_string()),
                    );
                    self.set_status(format!("❌ Ошибка распаковки: {}", e));
                }
            }
            self.zip_extraction_request = None;
        }

        // Request repaint for active downloads
        if !self.active_downloads.is_empty() {
            ctx.request_repaint();
        }
    }
}

fn main() -> eframe::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime");
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RetroROMs PC — Emu-Land Catalog & ROM Manager")
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([850.0, 550.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RetroROMs PC",
        options,
        Box::new(|cc| Ok(Box::new(RetroRomsApp::new(cc)))),
    )
}
