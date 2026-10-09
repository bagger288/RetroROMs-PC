use retroms_desktop::archive::format_bytes;
use retroms_desktop::config::AppSettings;
use retroms_desktop::db::Database;
use retroms_desktop::models::{get_default_consoles, GameCard};
use retroms_desktop::scraper::{get_game_subpath, normalize_image_url, EmuLandClient};

#[test]
fn test_default_consoles_count_and_data() {
    let consoles = get_default_consoles();
    assert_eq!(consoles.len(), 33, "Must have exactly 33 supported consoles");

    let dendy = consoles.iter().find(|c| c.slug == "dendy").expect("dendy found");
    assert_eq!(dendy.folder_name, "NES");
    assert_eq!(dendy.section, "consoles");

    let psx = consoles.iter().find(|c| c.slug == "psx").expect("psx found");
    assert_eq!(psx.folder_name, "PS1");

    let gba = consoles.iter().find(|c| c.slug == "gba").expect("gba found");
    assert_eq!(gba.section, "portable");
}

#[test]
fn test_game_subpaths() {
    assert_eq!(get_game_subpath("psx"), "iso");
    assert_eq!(get_game_subpath("3do"), "games");
    assert_eq!(get_game_subpath("segacd"), "games");
    assert_eq!(get_game_subpath("dendy"), "roms");
    assert_eq!(get_game_subpath("genesis"), "roms");
    assert_eq!(get_game_subpath("snes"), "roms");
}

#[test]
fn test_normalize_image_url() {
    assert_eq!(
        normalize_image_url("//files.emu-land.net/screens/nes/mario.png"),
        Some("https://files.emu-land.net/screens/nes/mario.png".to_string())
    );
    assert_eq!(
        normalize_image_url("/pictures/ss.jpg"),
        Some("https://www.emu-land.net/pictures/ss.jpg".to_string())
    );
    assert_eq!(normalize_image_url(""), None);
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512.0 B");
    assert_eq!(format_bytes(1024), "1.0 KiB");
    assert_eq!(format_bytes(1024 * 1024 * 5), "5.0 MiB");
}

#[test]
fn test_initial_categories() {
    let dendy_cats = EmuLandClient::get_initial_categories("dendy");
    assert!(dendy_cats.iter().any(|c| c.key == "top"));
    assert!(dendy_cats.iter().any(|c| c.key == "best"));
    assert!(dendy_cats.iter().any(|c| c.key == "0-9"));
    assert!(dendy_cats.iter().any(|c| c.key == "a"));
}

#[test]
fn test_sqlite_database_operations() {
    let temp_dir = std::env::temp_dir().join(format!("test_db_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
    let _ = std::fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test.db");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS consoles (
            slug TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            short_name TEXT NOT NULL,
            category TEXT NOT NULL,
            folder_name TEXT NOT NULL,
            section TEXT NOT NULL DEFAULT 'consoles',
            is_enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            release_year TEXT NOT NULL DEFAULT '',
            icon_key TEXT NOT NULL DEFAULT 'gamepad',
            roms_count_estimate TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            console_slug TEXT NOT NULL,
            console_name TEXT NOT NULL,
            section TEXT NOT NULL DEFAULT 'consoles',
            title TEXT NOT NULL,
            original_title TEXT,
            genre TEXT NOT NULL DEFAULT 'Action',
            year TEXT NOT NULL DEFAULT 'N/A',
            publisher TEXT NOT NULL DEFAULT 'Unknown',
            developer TEXT NOT NULL DEFAULT 'Unknown',
            rating REAL NOT NULL DEFAULT 4.8,
            file_size TEXT NOT NULL DEFAULT 'ROM',
            cover_url TEXT,
            screenshot_urls_json TEXT NOT NULL DEFAULT '[]',
            description TEXT NOT NULL DEFAULT '',
            download_url TEXT NOT NULL DEFAULT '',
            mfile_id TEXT,
            game_page_slug TEXT,
            regions_json TEXT NOT NULL DEFAULT '["USA"]',
            is_favorite INTEGER NOT NULL DEFAULT 0,
            is_downloaded INTEGER NOT NULL DEFAULT 0,
            local_file_path TEXT,
            cached_timestamp INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id TEXT NOT NULL,
            game_title TEXT NOT NULL,
            console_slug TEXT NOT NULL,
            console_name TEXT NOT NULL,
            file_name TEXT NOT NULL,
            target_directory TEXT NOT NULL,
            local_path TEXT,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            downloaded_bytes INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'Pending',
            download_url TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            error_message TEXT
        );
        "#,
    ).unwrap();

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM consoles", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
