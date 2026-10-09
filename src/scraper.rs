use crate::models::{CatalogCategory, GameCard, GamesPageResult, RomFileVersion};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use std::sync::LazyLock;
use std::time::Duration;

pub const BASE_URL: &str = "https://www.emu-land.net";
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

static RE_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"id=([0-9]+)").expect("Invalid regex")
});

static RE_FID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"fid=([0-9]+)").expect("Invalid regex")
});

static RE_YEAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([12][0-9]{3})").expect("Invalid regex")
});

pub fn get_game_subpath(slug: &str) -> &'static str {
    match slug.to_lowercase().as_str() {
        "psx" => "iso",
        "3do" | "segacd" | "pcecd" | "neogeocd" | "famicom_disk_system" | "sg-1000" | "gb" | "gbc" => "games",
        _ => "roms",
    }
}

pub fn normalize_image_url(raw: &str) -> Option<String> {
    if raw.trim().is_empty() {
        return None;
    }
    let mut url = raw.trim().replace("&amp;", "&").replace("\\/", "/");
    if url.starts_with("//") {
        url = format!("https:{}", url);
    } else if url.starts_with('/') {
        url = format!("{}{}", BASE_URL, url);
    }
    Some(url.replace(' ', "%20"))
}

#[derive(Clone)]
pub struct EmuLandClient {
    client: reqwest::Client,
    no_redirect_client: reqwest::Client,
}

impl Default for EmuLandClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EmuLandClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let no_redirect_client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_default();

        Self {
            client,
            no_redirect_client,
        }
    }

    pub fn get_initial_categories(slug: &str) -> Vec<CatalogCategory> {
        let s = slug.to_lowercase();
        match s.as_str() {
            "dendy" | "genesis" | "snes" | "gb" | "gbc" => {
                let mut list = vec![
                    CatalogCategory { key: "top".into(), label: "🔥 Популярные".into() },
                    CatalogCategory { key: "best".into(), label: "★ Лучшие".into() },
                    CatalogCategory { key: "homebrew".into(), label: "🛠 Homebrew".into() },
                    CatalogCategory { key: "misc".into(), label: "📁 Прочее".into() },
                    CatalogCategory { key: "0-9".into(), label: "0-9".into() },
                ];
                for c in 'a'..='z' {
                    list.push(CatalogCategory { key: c.to_string(), label: c.to_uppercase().to_string() });
                }
                list
            }
            "gba" => {
                let mut list = vec![
                    CatalogCategory { key: "top".into(), label: "🔥 Популярные".into() },
                    CatalogCategory { key: "best".into(), label: "★ Лучшие".into() },
                    CatalogCategory { key: "misc".into(), label: "📁 Прочее".into() },
                    CatalogCategory { key: "0-9".into(), label: "0-9".into() },
                ];
                for c in 'a'..='z' {
                    list.push(CatalogCategory { key: c.to_string(), label: c.to_uppercase().to_string() });
                }
                list
            }
            "n64" => {
                let mut list = vec![
                    CatalogCategory { key: "top".into(), label: "🔥 Популярные".into() },
                    CatalogCategory { key: "rating".into(), label: "★ Рейтинг".into() },
                    CatalogCategory { key: "misc".into(), label: "📁 Прочее".into() },
                    CatalogCategory { key: "0-9".into(), label: "0-9".into() },
                ];
                for c in 'a'..='z' {
                    list.push(CatalogCategory { key: c.to_string(), label: c.to_uppercase().to_string() });
                }
                list
            }
            "segacd" | "sms" | "pce" => {
                let mut list = vec![
                    CatalogCategory { key: "top".into(), label: "🔥 Популярные".into() },
                    CatalogCategory { key: "best".into(), label: "★ Лучшие".into() },
                    CatalogCategory { key: "0-9".into(), label: "0-9".into() },
                ];
                for c in 'a'..='z' {
                    list.push(CatalogCategory { key: c.to_string(), label: c.to_uppercase().to_string() });
                }
                list
            }
            "3do" => {
                let mut list = vec![
                    CatalogCategory { key: "top".into(), label: "🔥 Популярные".into() },
                ];
                for c in 'a'..='z' {
                    list.push(CatalogCategory { key: c.to_string(), label: c.to_uppercase().to_string() });
                }
                list
            }
            _ => vec![CatalogCategory { key: "all".into(), label: "Все игры".into() }],
        }
    }

    pub async fn fetch_categories(&self, console_slug: &str, section: &str) -> Vec<CatalogCategory> {
        let subpath = get_game_subpath(console_slug);
        let url = format!("{}/{}/{}/{}", BASE_URL, section, console_slug, subpath);

        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(_) => return Self::get_initial_categories(console_slug),
        };

        let html_text = match resp.text().await {
            Ok(t) => t,
            Err(_) => return Self::get_initial_categories(console_slug),
        };

        let document = Html::parse_document(&html_text);
        let pagelist_sel = Selector::parse("#pagelist_top, .pagelist").unwrap();
        let add_sel = Selector::parse(".add a").unwrap();
        let abc_sel = Selector::parse(".abc a").unwrap();

        if let Some(pagelist) = document.select(&pagelist_sel).next() {
            let mut categories = Vec::new();
            let mut seen_keys = std::collections::HashSet::new();

            for a in pagelist.select(&add_sel) {
                if let Some(href) = a.value().attr("href") {
                    let trimmed = href.trim_end_matches('/');
                    let key = trimmed.split('/').last().unwrap_or("").to_lowercase();
                    let text = a.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !key.is_empty() && seen_keys.insert(key.clone()) {
                        let label = match key.as_str() {
                            "top" => "🔥 Популярные".to_string(),
                            "best" => "★ Лучшие".to_string(),
                            "rating" => "★ Рейтинг".to_string(),
                            "homebrew" => "🛠 Homebrew".to_string(),
                            "misc" => "📁 Прочее".to_string(),
                            _ => text,
                        };
                        categories.push(CatalogCategory { key, label });
                    }
                }
            }

            for a in pagelist.select(&abc_sel) {
                if let Some(href) = a.value().attr("href") {
                    let trimmed = href.trim_end_matches('/');
                    let key = trimmed.split('/').last().unwrap_or("").to_lowercase();
                    let text = a.text().collect::<Vec<_>>().join(" ").trim().to_uppercase();
                    if !key.is_empty() && seen_keys.insert(key.clone()) {
                        categories.push(CatalogCategory { key, label: text });
                    }
                }
            }

            if !categories.is_empty() {
                return categories;
            }
        }

        Self::get_initial_categories(console_slug)
    }

    pub async fn fetch_games_page(
        &self,
        console_slug: &str,
        console_name: &str,
        section: &str,
        category: &str,
        page: usize,
    ) -> Result<GamesPageResult, Box<dyn std::error::Error + Send + Sync>> {
        let subpath = get_game_subpath(console_slug);

        let url = if category == "all" || category.is_empty() {
            if page <= 1 {
                format!("{}/{}/{}/{}", BASE_URL, section, console_slug, subpath)
            } else {
                format!("{}/{}/{}/{}/{}", BASE_URL, section, console_slug, subpath, page)
            }
        } else if page <= 1 {
            format!("{}/{}/{}/{}/{}", BASE_URL, section, console_slug, subpath, category)
        } else {
            format!(
                "{}/{}/{}/{}/{}/{}",
                BASE_URL, section, console_slug, subpath, category, page
            )
        };

        let resp = self
            .client
            .get(&url)
            .header("Referer", format!("{}/{}/{}", BASE_URL, section, console_slug))
            .send()
            .await?;

        let html = resp.text().await?;
        let document = Html::parse_document(&html);

        let mut games = Vec::new();

        // Emu-Land uses .fcontainer for game cards
        let container_sel = Selector::parse(".fcontainer").unwrap();
        let title_sel = Selector::parse(".rheader a, h4.rheader a, .rheader span.hand, h4 a").unwrap();
        let img_sel = Selector::parse(".picture img, img.pixelated, img.game-screens").unwrap();
        let finfo_sel = Selector::parse(".finfo li").unwrap();
        let fbottom_sel = Selector::parse(".fbottom li").unwrap();
        let dl_btn_sel = Selector::parse(".btn-sdl, .btn-dl, [onclick*='getmfl'], [onclick*='getfile']").unwrap();

        for container in document.select(&container_sel) {
            let title_elem = container.select(&title_sel).next();
            let title = match title_elem {
                Some(el) => el.text().collect::<Vec<_>>().join(" ").trim().to_string(),
                None => continue,
            };
            if title.is_empty() {
                continue;
            }

            let game_page_href = title_elem
                .and_then(|el| el.value().attr("href"))
                .unwrap_or("");

            let game_page_slug = if !game_page_href.is_empty() {
                Some(game_page_href.trim_start_matches('/').to_string())
            } else {
                None
            };

            // Cover and screenshots
            let mut cover_url = None;
            let mut screenshot_urls = Vec::new();

            if let Some(img_elem) = container.select(&img_sel).next() {
                if let Some(src) = img_elem.value().attr("src") {
                    cover_url = normalize_image_url(src);
                    if let Some(c) = &cover_url {
                        screenshot_urls.push(c.clone());
                    }
                }

                // Check for data-gallery JSON
                if let Some(gallery_attr) = img_elem.value().attr("data-gallery") {
                    if let Ok(val) = serde_json::from_str::<Value>(gallery_attr) {
                        if let Some(list) = val.get("list").and_then(|l| l.as_array()) {
                            for item in list {
                                if let Some(src_str) = item.get("src").and_then(|s| s.as_str()) {
                                    if let Some(norm) = normalize_image_url(src_str) {
                                        if !screenshot_urls.contains(&norm) {
                                            screenshot_urls.push(norm);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Metadata from .finfo
            let mut genre = "Action".to_string();
            let mut developer = "Unknown".to_string();
            let mut publisher = "Unknown".to_string();
            let mut year = "N/A".to_string();

            for li in container.select(&finfo_sel) {
                let text = li.text().collect::<Vec<_>>().join(" ").trim().to_string();
                let lower = text.to_lowercase();
                if lower.contains("жанр:") {
                    genre = text.split("Жанр:").nth(1).unwrap_or("Action").trim().to_string();
                } else if lower.contains("разработчик:") {
                    developer = text.split("Разработчик:").nth(1).unwrap_or("Unknown").trim().to_string();
                } else if lower.contains("издатель:") {
                    publisher = text.split("Издатель:").nth(1).unwrap_or("Unknown").trim().to_string();
                } else if lower.contains("год выпуска:") {
                    if let Some(cap) = RE_YEAR.captures(&text) {
                        year = cap[1].to_string();
                    }
                }
            }

            // File size
            let mut file_size = "ROM".to_string();
            for li in container.select(&fbottom_sel) {
                let text = li.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if text.to_lowercase().contains("размер:") {
                    file_size = text.split("Размер:").nth(1).unwrap_or("ROM").trim().to_string();
                }
            }

            // Mfile ID detection from download button onclick
            let mut mfile_id = None;
            for btn in container.select(&dl_btn_sel) {
                if let Some(onclick) = btn.value().attr("onclick") {
                    if let Some(cap) = RE_ID.captures(onclick) {
                        mfile_id = Some(cap[1].to_string());
                        break;
                    }
                }
            }

            // Fallback for mfile_id from pict_ or mfile_ element IDs
            if mfile_id.is_none() {
                let inner_html = container.html();
                if let Some(cap) = RE_ID.captures(&inner_html) {
                    mfile_id = Some(cap[1].to_string());
                }
            }

            let id = format!("{}_{}", console_slug, game_page_slug.as_deref().unwrap_or(&title));

            games.push(GameCard {
                id,
                console_slug: console_slug.to_string(),
                console_name: console_name.to_string(),
                section: section.to_string(),
                title,
                original_title: None,
                genre,
                year,
                publisher,
                developer,
                rating: 4.8,
                file_size,
                cover_url,
                screenshot_urls,
                description: String::new(),
                download_url: url.clone(),
                mfile_id,
                game_page_slug,
                regions: vec!["USA".to_string()],
                is_favorite: false,
                is_downloaded: false,
                local_file_path: None,
            });
        }

        // Pagination calculation from .num a
        let num_sel = Selector::parse(".num a, div.num a, .pagination a").unwrap();
        let mut max_page = page;
        for a in document.select(&num_sel) {
            let t = a.text().collect::<Vec<_>>().join(" ").trim().to_string();
            if let Ok(p) = t.parse::<usize>() {
                if p > max_page {
                    max_page = p;
                }
            }
            if let Some(href) = a.value().attr("href") {
                if let Some(cap) = RE_ID.captures(href) {
                    if let Ok(p) = cap[1].parse::<usize>() {
                        if p > max_page {
                            max_page = p;
                        }
                    }
                }
            }
        }

        let has_next_page = max_page > page;

        Ok(GamesPageResult {
            games,
            current_page: page,
            total_pages: max_page.max(1),
            has_next_page,
            available_categories: Vec::new(),
        })
    }

    pub async fn fetch_rom_versions(
        &self,
        console_slug: &str,
        section: &str,
        mfile_id: &str,
    ) -> Result<Vec<RomFileVersion>, Box<dyn std::error::Error + Send + Sync>> {
        let subpath = get_game_subpath(console_slug);
        let url = format!(
            "{}/{}/{}/{}?act=getmfl&id={}",
            BASE_URL, section, console_slug, subpath, mfile_id
        );
        let referer = format!("{}/{}/{}/{}", BASE_URL, section, console_slug, subpath);

        let resp = self
            .client
            .get(&url)
            .header("Referer", &referer)
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await?;

        let html = resp.text().await?;
        let document = Html::parse_fragment(&html);

        let mut versions = Vec::new();
        let group_sel = Selector::parse(".collaps-item, .collaps, .mfile-block, .romlist-group").unwrap();
        let group_title_sel = Selector::parse(".title span, .title, .group-header").unwrap();
        let item_sel = Selector::parse(".item, .file-item, li").unwrap();
        let link_sel = Selector::parse("a[href*='fid='], a[href*='act=']").unwrap();
        let size_sel = Selector::parse(".size, small, .fsize").unwrap();

        for group in document.select(&group_sel) {
            let category_name = group
                .select(&group_title_sel)
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .unwrap_or_else(|| "Основные".to_string());

            for item in group.select(&item_sel) {
                if let Some(link) = item.select(&link_sel).next() {
                    let href = link.value().attr("href").unwrap_or("");
                    let name = link.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if name.is_empty() {
                        continue;
                    }

                    let fid = RE_FID
                        .captures(href)
                        .map(|c| c[1].to_string())
                        .unwrap_or_else(|| mfile_id.to_string());

                    let size = item
                        .select(&size_sel)
                        .next()
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .unwrap_or_else(|| "ROM".to_string());

                    let full_download_url = if href.starts_with("http") {
                        href.to_string()
                    } else if href.starts_with('/') {
                        format!("{}{}", BASE_URL, href)
                    } else {
                        format!(
                            "{}/{}/{}/{}?act=getmfl&id={}&fid={}",
                            BASE_URL, section, console_slug, subpath, mfile_id, fid
                        )
                    };

                    // Detect region or type
                    let region_or_type = if name.contains("(R)") || name.to_lowercase().contains("rus") || name.contains("Russian") {
                        "RUS".to_string()
                    } else if name.contains("(U)") || name.contains("(USA)") {
                        "USA".to_string()
                    } else if name.contains("(E)") || name.contains("(Europe)") {
                        "EUR".to_string()
                    } else if name.contains("(J)") || name.contains("(Japan)") {
                        "JAP".to_string()
                    } else if name.contains("[h") || name.to_lowercase().contains("hack") {
                        "HACK".to_string()
                    } else if category_name.to_lowercase().contains("goodset") || name.to_lowercase().contains("goodset") {
                        "GOODSET".to_string()
                    } else {
                        "OTHER".to_string()
                    };

                    versions.push(RomFileVersion {
                        fid,
                        name,
                        size,
                        category: category_name.clone(),
                        download_url: full_download_url,
                        region_or_type,
                    });
                }
            }
        }

        // Fallback: search any links with fid if no .collaps-item found
        if versions.is_empty() {
            for link in document.select(&link_sel) {
                let href = link.value().attr("href").unwrap_or("");
                let name = link.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if name.is_empty() {
                    continue;
                }
                let fid = RE_FID
                    .captures(href)
                    .map(|c| c[1].to_string())
                    .unwrap_or_else(|| mfile_id.to_string());

                let full_download_url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!(
                        "{}/{}/{}/{}?act=getmfl&id={}&fid={}",
                        BASE_URL, section, console_slug, subpath, mfile_id, fid
                    )
                };

                versions.push(RomFileVersion {
                    fid,
                    name,
                    size: "ROM".to_string(),
                    category: "Основные".to_string(),
                    download_url: full_download_url,
                    region_or_type: "USA".to_string(),
                });
            }
        }

        Ok(versions)
    }

    pub async fn resolve_direct_download_url(
        &self,
        target_url: &str,
        referer: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self
            .no_redirect_client
            .get(target_url)
            .header("Referer", referer)
            .send()
            .await?;

        if let Some(loc) = resp.headers().get(reqwest::header::LOCATION) {
            let loc_str = loc.to_str()?;
            if loc_str.starts_with("http") {
                return Ok(loc_str.to_string());
            } else if loc_str.starts_with('/') {
                return Ok(format!("{}{}", BASE_URL, loc_str));
            }
        }

        Ok(target_url.to_string())
    }

    pub async fn fetch_game_page_details(
        &self,
        game_page_slug: &str,
    ) -> Result<(String, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
        let url = if game_page_slug.starts_with("http") {
            game_page_slug.to_string()
        } else {
            format!("{}/{}", BASE_URL, game_page_slug)
        };

        let resp = self.client.get(&url).send().await?;
        let html = resp.text().await?;
        let document = Html::parse_document(&html);

        let desc_sel = Selector::parse(".ftext p, .description p, #description, .desc").unwrap();
        let mut desc_parts = Vec::new();
        for p in document.select(&desc_sel) {
            let text = p.text().collect::<Vec<_>>().join(" ").trim().to_string();
            if !text.is_empty() {
                desc_parts.push(text);
            }
        }
        let description = desc_parts.join("\n\n");

        let screenshot_sel = Selector::parse(".ss-area a.ss, .picture img, .ss-area img, a[href*='screenshots'] img").unwrap();
        let mut screenshots = Vec::new();
        for el in document.select(&screenshot_sel) {
            let src = el.value().attr("href").or_else(|| el.value().attr("src"));
            if let Some(src_str) = src {
                if let Some(normalized) = normalize_image_url(src_str) {
                    if !screenshots.contains(&normalized) {
                        screenshots.push(normalized);
                    }
                }
            }
        }

        Ok((description, screenshots))
    }
}
