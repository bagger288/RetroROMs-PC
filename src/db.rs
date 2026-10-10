use crate::models::{
    get_default_consoles, ConsoleInfo, DownloadRecord, DownloadStatus, GameCard,
};
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn db_path() -> PathBuf {
        if let Some(config_dir) = dirs::data_local_dir().or_else(dirs::config_dir) {
            let app_dir = config_dir.join("RetroROMs");
            let _ = fs::create_dir_all(&app_dir);
            app_dir.join("retroms.db")
        } else {
            PathBuf::from("retroms.db")
        }
    }

    pub fn open() -> Result<Self> {
        let path = Self::db_path();
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        db.seed_default_consoles_if_needed()?;
        Ok(db)
    }

    pub fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
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

            CREATE INDEX IF NOT EXISTS idx_games_console_slug ON games (console_slug);
            CREATE INDEX IF NOT EXISTS idx_games_is_fav ON games (is_favorite);
            CREATE INDEX IF NOT EXISTS idx_games_title ON games (title COLLATE NOCASE);

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
        )?;
        Ok(())
    }

    fn seed_default_consoles_if_needed(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM consoles", [], |row| row.get(0))?;

        if count == 0 {
            let defaults = get_default_consoles();
            let mut stmt = conn.prepare(
                r#"
                INSERT INTO consoles (
                    slug, name, short_name, category, folder_name, section,
                    is_enabled, sort_order, release_year, icon_key, roms_count_estimate
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
            )?;

            for c in defaults {
                stmt.execute(params![
                    c.slug,
                    c.name,
                    c.short_name,
                    c.category,
                    c.folder_name,
                    c.section,
                    c.is_enabled as i32,
                    c.order,
                    c.release_year,
                    c.icon_key,
                    c.roms_count_estimate,
                ])?;
            }
        }
        Ok(())
    }

    pub fn get_consoles(&self) -> Result<Vec<ConsoleInfo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT slug, name, short_name, category, folder_name, section,
                   is_enabled, sort_order, release_year, icon_key, roms_count_estimate
            FROM consoles
            ORDER BY sort_order ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ConsoleInfo {
                slug: row.get(0)?,
                name: row.get(1)?,
                short_name: row.get(2)?,
                category: row.get(3)?,
                folder_name: row.get(4)?,
                section: row.get(5)?,
                is_enabled: row.get::<_, i32>(6)? != 0,
                order: row.get(7)?,
                release_year: row.get(8)?,
                icon_key: row.get(9)?,
                roms_count_estimate: row.get(10)?,
            })
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn update_console_order_and_enabled(&self, consoles: &[ConsoleInfo]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "UPDATE consoles SET sort_order = ?1, is_enabled = ?2 WHERE slug = ?3",
        )?;
        for (i, c) in consoles.iter().enumerate() {
            stmt.execute(params![i as i32, c.is_enabled as i32, c.slug])?;
        }
        Ok(())
    }

    pub fn get_favorite_games(&self) -> Result<Vec<GameCard>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, console_slug, console_name, section, title, original_title,
                   genre, year, publisher, developer, rating, file_size, cover_url,
                   screenshot_urls_json, description, download_url, mfile_id,
                   game_page_slug, regions_json, is_favorite, is_downloaded, local_file_path
            FROM games
            WHERE is_favorite = 1
            ORDER BY title COLLATE NOCASE ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let screenshots_json: String = row.get(13)?;
            let screenshots: Vec<String> =
                serde_json::from_str(&screenshots_json).unwrap_or_default();
            let regions_json: String = row.get(18)?;
            let regions: Vec<String> =
                serde_json::from_str(&regions_json).unwrap_or_else(|_| vec!["USA".to_string()]);

            Ok(GameCard {
                id: row.get(0)?,
                console_slug: row.get(1)?,
                console_name: row.get(2)?,
                section: row.get(3)?,
                title: row.get(4)?,
                original_title: row.get(5)?,
                genre: row.get(6)?,
                year: row.get(7)?,
                publisher: row.get(8)?,
                developer: row.get(9)?,
                rating: row.get(10)?,
                file_size: row.get(11)?,
                cover_url: row.get(12)?,
                screenshot_urls: screenshots,
                description: row.get(14)?,
                download_url: row.get(15)?,
                mfile_id: row.get(16)?,
                game_page_slug: row.get(17)?,
                regions,
                is_favorite: row.get::<_, i32>(19)? != 0,
                is_downloaded: row.get::<_, i32>(20)? != 0,
                local_file_path: row.get(21)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn is_game_favorite(&self, game_id: &str) -> bool {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT is_favorite FROM games WHERE id = ?1",
            params![game_id],
            |row| row.get::<_, i32>(0),
        )
        .map(|val| val != 0)
        .unwrap_or(false)
    }

    pub fn set_game_favorite(&self, game: &GameCard, favorite: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let screens_json = serde_json::to_string(&game.screenshot_urls).unwrap_or_default();
        let regions_json = serde_json::to_string(&game.regions).unwrap_or_default();

        conn.execute(
            r#"
            INSERT INTO games (
                id, console_slug, console_name, section, title, original_title,
                genre, year, publisher, developer, rating, file_size, cover_url,
                screenshot_urls_json, description, download_url, mfile_id,
                game_page_slug, regions_json, is_favorite, is_downloaded, local_file_path,
                cached_timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)
            ON CONFLICT(id) DO UPDATE SET
                is_favorite = excluded.is_favorite
            "#,
            params![
                game.id,
                game.console_slug,
                game.console_name,
                game.section,
                game.title,
                game.original_title,
                game.genre,
                game.year,
                game.publisher,
                game.developer,
                game.rating,
                game.file_size,
                game.cover_url,
                screens_json,
                game.description,
                game.download_url,
                game.mfile_id,
                game.game_page_slug,
                regions_json,
                favorite as i32,
                game.is_downloaded as i32,
                game.local_file_path,
                chrono::Utc::now().timestamp(),
            ],
        )?;
        Ok(())
    }

    pub fn save_games(&self, games: &[GameCard]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        for game in games {
            let screens_json = serde_json::to_string(&game.screenshot_urls).unwrap_or_default();
            let regions_json = serde_json::to_string(&game.regions).unwrap_or_default();
            let _ = conn.execute(
                r#"
                INSERT INTO games (
                    id, console_slug, console_name, section, title, original_title,
                    genre, year, publisher, developer, rating, file_size, cover_url,
                    screenshot_urls_json, description, download_url, mfile_id,
                    game_page_slug, regions_json, is_favorite, is_downloaded, local_file_path,
                    cached_timestamp
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)
                ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    cover_url = COALESCE(excluded.cover_url, games.cover_url),
                    genre = excluded.genre,
                    cached_timestamp = excluded.cached_timestamp
                "#,
                params![
                    game.id,
                    game.console_slug,
                    game.console_name,
                    game.section,
                    game.title,
                    game.original_title,
                    game.genre,
                    game.year,
                    game.publisher,
                    game.developer,
                    game.rating,
                    game.file_size,
                    game.cover_url,
                    screens_json,
                    game.description,
                    game.download_url,
                    game.mfile_id,
                    game.game_page_slug,
                    regions_json,
                    game.is_favorite as i32,
                    game.is_downloaded as i32,
                    game.local_file_path,
                    now,
                ],
            );
        }
        Ok(())
    }

    pub fn search_games(&self, query: &str) -> Result<Vec<GameCard>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query.trim());
        let mut stmt = conn.prepare(
            r#"
            SELECT id, console_slug, console_name, section, title, original_title,
                   genre, year, publisher, developer, rating, file_size, cover_url,
                   screenshot_urls_json, description, download_url, mfile_id,
                   game_page_slug, regions_json, is_favorite, is_downloaded, local_file_path
            FROM games
            WHERE title LIKE ?1 OR original_title LIKE ?1
            ORDER BY rating DESC, title COLLATE NOCASE ASC
            LIMIT 100
            "#,
        )?;

        let rows = stmt.query_map([&pattern], |row| {
            let screenshots_json: String = row.get(13)?;
            let screenshots: Vec<String> =
                serde_json::from_str(&screenshots_json).unwrap_or_default();
            let regions_json: String = row.get(18)?;
            let regions: Vec<String> =
                serde_json::from_str(&regions_json).unwrap_or_else(|_| vec!["USA".to_string()]);

            Ok(GameCard {
                id: row.get(0)?,
                console_slug: row.get(1)?,
                console_name: row.get(2)?,
                section: row.get(3)?,
                title: row.get(4)?,
                original_title: row.get(5)?,
                genre: row.get(6)?,
                year: row.get(7)?,
                publisher: row.get(8)?,
                developer: row.get(9)?,
                rating: row.get(10)?,
                file_size: row.get(11)?,
                cover_url: row.get(12)?,
                screenshot_urls: screenshots,
                description: row.get(14)?,
                download_url: row.get(15)?,
                mfile_id: row.get(16)?,
                game_page_slug: row.get(17)?,
                regions,
                is_favorite: row.get::<_, i32>(19)? != 0,
                is_downloaded: row.get::<_, i32>(20)? != 0,
                local_file_path: row.get(21)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn mark_game_downloaded(
        &self,
        game_id: &str,
        downloaded: bool,
        local_path: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE games SET is_downloaded = ?1, local_file_path = ?2 WHERE id = ?3",
            params![downloaded as i32, local_path, game_id],
        )?;
        Ok(())
    }

    pub fn add_download_record(
        &self,
        game_id: &str,
        game_title: &str,
        console_slug: &str,
        console_name: &str,
        file_name: &str,
        target_directory: &str,
        download_url: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            r#"
            INSERT INTO downloads (
                game_id, game_title, console_slug, console_name,
                file_name, target_directory, status, download_url, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'Downloading', ?7, ?8)
            "#,
            params![
                game_id,
                game_title,
                console_slug,
                console_name,
                file_name,
                target_directory,
                download_url,
                now
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_download_progress(
        &self,
        record_id: i64,
        downloaded: u64,
        total: u64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE downloads SET downloaded_bytes = ?1, total_bytes = ?2 WHERE id = ?3",
            params![downloaded as i64, total as i64, record_id],
        )?;
        Ok(())
    }

    pub fn update_download_status(
        &self,
        record_id: i64,
        status: DownloadStatus,
        local_path: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let status_str = match status {
            DownloadStatus::Pending => "Pending",
            DownloadStatus::Downloading => "Downloading",
            DownloadStatus::Completed => "Completed",
            DownloadStatus::Failed => "Failed",
            DownloadStatus::Cancelled => "Cancelled",
        };

        conn.execute(
            "UPDATE downloads SET status = ?1, local_path = ?2, error_message = ?3 WHERE id = ?4",
            params![status_str, local_path, error_message, record_id],
        )?;
        Ok(())
    }

    pub fn get_download_history(&self) -> Result<Vec<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, game_id, game_title, console_slug, console_name,
                   file_name, target_directory, local_path, total_bytes,
                   downloaded_bytes, status, download_url, timestamp, error_message
            FROM downloads
            ORDER BY timestamp DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let status_str: String = row.get(10)?;
            let status = match status_str.as_str() {
                "Downloading" => DownloadStatus::Downloading,
                "Completed" => DownloadStatus::Completed,
                "Failed" => DownloadStatus::Failed,
                "Cancelled" => DownloadStatus::Cancelled,
                _ => DownloadStatus::Pending,
            };

            Ok(DownloadRecord {
                id: row.get(0)?,
                game_id: row.get(1)?,
                game_title: row.get(2)?,
                console_slug: row.get(3)?,
                console_name: row.get(4)?,
                file_name: row.get(5)?,
                target_directory: row.get(6)?,
                local_path: row.get(7)?,
                total_bytes: row.get::<_, i64>(8)? as u64,
                downloaded_bytes: row.get::<_, i64>(9)? as u64,
                speed_bytes_sec: 0,
                status,
                download_url: row.get(11)?,
                timestamp: row.get(12)?,
                error_message: row.get(13)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_download_record(&self, record_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM downloads WHERE id = ?1", params![record_id])?;
        Ok(())
    }
}
