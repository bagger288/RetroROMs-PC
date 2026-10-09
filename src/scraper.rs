use crate::models::{CatalogCategory, GameCard, GamesPageResult, RomFileVersion};
use regex::Regex;
use scraper::{Html, Selector};

pub const BASE_URL: &str = "https://www.emu-land.net";
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

pub fn get_game_subpath(slug: &str) -> &'static str {
    match slug.to_lowercase().as_str() {
        "psx" => "iso",
        "3do" | "segacd" | "pcecd" | "neogeocd" | "famicom_disk_system" | "sg-1000" | "gb" | "gbc" => "games",
        _ => "roms",
    }
}

pub fn is_valid_cover_image(url: &str) -> bool {
    let lower = url.to_lowercase();
    // Exclude country flags (jp.png, us.png, eu.png), transparent pixels, UI icons, and skin assets
    if lower.contains("flag")
        || lower.contains("pixel.gif")
        || lower.contains("spacer")
        || lower.contains("rating")
        || lower.contains("star")
        || lower.contains("/skin/")
        || lower.contains("icon")
    {
        return false;
    }
    true
}

pub fn normalize_image_url(src: &str) -> Option<String> {
    let clean = src.trim();
    if clean.is_empty() || !is_valid_cover_image(clean) {
        return None;
    }
    if clean.starts_with("//") {
        Some(format!("https:{}", clean))
    } else if clean.starts_with('/') {
        Some(format!("{}{}", BASE_URL, clean))
    } else if clean.starts_with("http://") || clean.starts_with("https://") {
        Some(clean.to_string())
    } else {
        Some(format!("{}/{}", BASE_URL, clean))
    }
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
            .timeout(std::time::Duration::from_secs(25))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let no_redirect_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(25))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            no_redirect_client,
        }
    }

    pub fn get_initial_categories(slug: &str) -> Vec<CatalogCategory> {
        let s = slug.to_lowercase();
        // Consoles with only 1 page on Emu-Land
        if s == "psx" || s == "2600" || s == "5200" || s == "7800" || s == "coleco" || s == "vectrex" || s == "intellivision" || s == "arcadia" || s == "chaf" {
            return vec![CatalogCategory {
                key: "all".to_string(),
                label: "Все игры".to_string(),
            }];
        }

        let mut cats = Vec::new();
        if s != "neogeocd" && s != "jaguar" {
            cats.push(CatalogCategory {
                key: "top".to_string(),
                label: "🔥 Популярные".to_string(),
            });
        }
        if s != "jaguar" {
            cats.push(CatalogCategory {
                key: "best".to_string(),
                label: "★ Лучшие".to_string(),
            });
        }
        cats.push(CatalogCategory {
            key: "0-9".to_string(),
            label: "0-9".to_string(),
        });
        for c in b'a'..=b'z' {
            let ch = (c as char).to_string();
            cats.push(CatalogCategory {
                key: ch.clone(),
                label: ch.to_uppercase(),
            });
        }
        cats
    }

    pub async fn fetch_games_page(
        &self,
        slug: &str,
        console_name: &str,
        section: &str,
        category: &str,
        page: usize,
    ) -> Result<GamesPageResult, Box<dyn std::error::Error + Send + Sync>> {
        let subpath = get_game_subpath(slug);
        let url = if category == "all" || category.is_empty() {
            if page <= 1 {
                format!("{}/{}/{}/{}", BASE_URL, section, slug, subpath)
            } else {
                format!("{}/{}/{}/{}/{}", BASE_URL, section, slug, subpath, page)
            }
        } else if page <= 1 {
            format!("{}/{}/{}/{}/{}", BASE_URL, section, slug, subpath, category)
        } else {
            format!("{}/{}/{}/{}/{}/{}", BASE_URL, section, slug, subpath, category, page)
        };

        let resp = self.client.get(&url).send().await?;
        let html_text = resp.text().await?;
        let document = Html::parse_document(&html_text);

        // 1. Parse categories from #pagelist_top
        let mut available_categories = Vec::new();
        let pagelist_sel = Selector::parse("#pagelist_top, .pagelist").unwrap();
        let a_sel = Selector::parse("a").unwrap();
        if let Some(pl) = document.select(&pagelist_sel).next() {
            for a in pl.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    let parts: Vec<&str> = href.trim_matches('/').split('/').collect();
                    if let Some(last_part) = parts.last() {
                        let key = last_part.to_string();
                        // Ignore pure page numbers
                        if key.parse::<usize>().is_err() && key != subpath && key != slug {
                            let raw_text = a.text().collect::<Vec<_>>().join("").trim().to_string();
                            let label = match key.as_str() {
                                "top" => "🔥 Популярные".to_string(),
                                "best" => "★ Лучшие".to_string(),
                                "rating" => "★ Рейтинг".to_string(),
                                "homebrew" => "🛠 Homebrew".to_string(),
                                "misc" => "📁 Прочее".to_string(),
                                _ => raw_text,
                            };
                            if !available_categories.iter().any(|c: &CatalogCategory| c.key == key) {
                                available_categories.push(CatalogCategory { key, label });
                            }
                        }
                    }
                }
            }
        }

        if available_categories.is_empty() {
            available_categories = Self::get_initial_categories(slug);
        }

        // 2. Parse pagination (.num a)
        let num_a_sel = Selector::parse(".num a, .pagelist .num a").unwrap();
        let mut max_page = page;
        let mut has_next = false;
        for a in document.select(&num_a_sel) {
            let t = a.text().collect::<Vec<_>>().join("").trim().to_string();
            if let Ok(p_num) = t.parse::<usize>() {
                if p_num > max_page {
                    max_page = p_num;
                }
            }
            if t.contains("Далее") || t.contains('»') {
                has_next = true;
            }
        }

        // 3. Parse games (.fcontainer, .glist-item, .game-item, table rows)
        let mut games = Vec::new();
        let fcontainer_sel = Selector::parse(".fcontainer").unwrap();
        let rheader_a_sel = Selector::parse(".rheader a, h4 a, .title a").unwrap();
        let img_sel = Selector::parse(".picture img, .ss-area img, img.game-screens, a.highslide img, .screen img, img[src*='ss.emu-land.net'], img[src*='_pict'], img").unwrap();
        let btn_sdl_sel = Selector::parse("[onclick*='getmfl'], .btn-sdl, a[href*='act=getmfl']").unwrap();
        let mfile_regex = Regex::new(r"id=([0-9]+)").unwrap();
        let finfo_sel = Selector::parse(".finfo li, .finfo").unwrap();

        for container in document.select(&fcontainer_sel) {
            let title_el = container.select(&rheader_a_sel).next();
            let title = match title_el {
                Some(el) => el.text().collect::<Vec<_>>().join("").trim().to_string(),
                None => continue,
            };

            let game_page_slug = title_el
                .and_then(|el| el.value().attr("href"))
                .map(|href| href.trim_matches('/').split('/').last().unwrap_or("").to_string());

            let cover_url = container
                .select(&img_sel)
                .filter_map(|img| img.value().attr("src"))
                .find_map(normalize_image_url);

            let mut mfile_id = None;
            if let Some(sdl) = container.select(&btn_sdl_sel).next() {
                if let Some(onclick) = sdl.value().attr("onclick") {
                    if let Some(cap) = mfile_regex.captures(onclick) {
                        mfile_id = Some(cap[1].to_string());
                    }
                }
                if mfile_id.is_none() {
                    if let Some(href) = sdl.value().attr("href") {
                        if let Some(cap) = mfile_regex.captures(href) {
                            mfile_id = Some(cap[1].to_string());
                        }
                    }
                }
            }

            let mut genre = "Action".to_string();
            let mut year = "".to_string();
            let mut developer = "".to_string();
            for li in container.select(&finfo_sel) {
                let text = li.text().collect::<Vec<_>>().join(" ");
                if text.contains("Жанр:") {
                    genre = text.replace("•", "").replace("Жанр:", "").trim().to_string();
                } else if text.contains("Год") {
                    let re_yr = Regex::new(r"(19\d\d|20\d\d)").unwrap();
                    if let Some(cap) = re_yr.captures(&text) {
                        year = cap[1].to_string();
                    }
                } else if text.contains("Разработчик:") {
                    developer = text.replace("•", "").replace("Разработчик:", "").trim().to_string();
                }
            }

            let id = mfile_id.clone().unwrap_or_else(|| {
                game_page_slug.clone().unwrap_or_else(|| title.to_lowercase().replace(' ', "_"))
            });

            games.push(GameCard {
                id,
                console_slug: slug.to_string(),
                console_name: console_name.to_string(),
                section: section.to_string(),
                title,
                original_title: None,
                genre,
                year,
                publisher: "Unknown".to_string(),
                developer,
                rating: 4.8,
                file_size: "ROM".to_string(),
                cover_url,
                screenshot_urls: Vec::new(),
                description: String::new(),
                download_url: String::new(),
                mfile_id,
                game_page_slug,
                regions: vec!["US".to_string()],
                is_favorite: false,
                is_downloaded: false,
                local_file_path: None,
            });
        }

        Ok(GamesPageResult {
            games,
            current_page: page,
            total_pages: max_page.max(1),
            has_next_page: has_next || (page < max_page),
            available_categories,
        })
    }

    pub async fn fetch_rom_versions(
        &self,
        section: &str,
        slug: &str,
        mfile_id: &str,
    ) -> Result<Vec<RomFileVersion>, Box<dyn std::error::Error + Send + Sync>> {
        let subpath = get_game_subpath(slug);
        let url = format!(
            "{}/{}/{}/{}?act=getmfl&id={}",
            BASE_URL, section, slug, subpath, mfile_id
        );
        let referer = format!("{}/{}/{}/{}", BASE_URL, section, slug, subpath);

        let resp = self
            .client
            .get(&url)
            .header("Referer", &referer)
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await?;

        let html = resp.text().await?;
        let document = Html::parse_document(&html);

        let collaps_sel = Selector::parse(".collaps-item, .romlist-area, body").unwrap();
        let title_sel = Selector::parse(".title, .head").unwrap();
        let item_sel = Selector::parse(".item, li, tr").unwrap();
        let a_sel = Selector::parse("a[href*='fid=']").unwrap();
        let size_sel = Selector::parse(".size, span").unwrap();

        let fid_regex = Regex::new(r"fid=([0-9]+)").unwrap();
        let mut versions = Vec::new();

        for block in document.select(&collaps_sel) {
            let group_name = block
                .select(&title_sel)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join("").trim().to_string())
                .unwrap_or_else(|| "Основные".to_string());

            for item in block.select(&item_sel) {
                if let Some(a) = item.select(&a_sel).next() {
                    let href = a.value().attr("href").unwrap_or("");
                    let fid = match fid_regex.captures(href) {
                        Some(c) => c[1].to_string(),
                        None => continue,
                    };
                    let name = a.text().collect::<Vec<_>>().join("").trim().to_string();
                    let size = item
                        .select(&size_sel)
                        .next()
                        .map(|s| s.text().collect::<Vec<_>>().join("").trim().to_string())
                        .unwrap_or_else(|| "ROM".to_string());

                    let download_url = format!(
                        "{}/{}/{}/{}?act=getmfl&id={}&fid={}",
                        BASE_URL, section, slug, subpath, mfile_id, fid
                    );

                    let name_upper = name.to_uppercase();
                    let region_or_type = if name_upper.contains("(R)") || name_upper.contains("(RUS)") || name_upper.contains("RUSSIAN") {
                        "RUS".to_string()
                    } else if name_upper.contains("(U)") || name_upper.contains("(USA)") {
                        "USA".to_string()
                    } else if name_upper.contains("(E)") || name_upper.contains("(EUROPE)") {
                        "EUR".to_string()
                    } else if name_upper.contains("(J)") || name_upper.contains("(JAPAN)") {
                        "JAP".to_string()
                    } else if name_upper.contains("[H") || name_upper.contains("HACK") {
                        "HACK".to_string()
                    } else {
                        "ROM".to_string()
                    };

                    versions.push(RomFileVersion {
                        fid,
                        name,
                        size,
                        category: group_name.clone(),
                        download_url,
                        region_or_type,
                    });
                }
            }
        }

        Ok(versions)
    }

    pub async fn get_direct_download_url(
        &self,
        getmfl_url: &str,
        referer: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self
            .no_redirect_client
            .get(getmfl_url)
            .header("Referer", referer)
            .send()
            .await?;

        if let Some(loc) = resp.headers().get(reqwest::header::LOCATION) {
            let location_str = loc.to_str()?;
            return Ok(location_str.to_string());
        }

        // If no 302 Location header, check if final url changed
        Ok(resp.url().to_string())
    }

    pub async fn fetch_game_page_details(
        &self,
        game: &mut GameCard,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let slug = match &game.game_page_slug {
            Some(s) if !s.is_empty() => s.as_str(),
            _ => return Ok(()),
        };
        let subpath = get_game_subpath(&game.console_slug);
        let url = if slug.starts_with("http") {
            slug.to_string()
        } else if slug.starts_with('/') {
            format!("{}{}", BASE_URL, slug)
        } else {
            format!("{}/{}/{}/{}/{}", BASE_URL, game.section, game.console_slug, subpath, slug)
        };

        let resp = self.client.get(&url).send().await?;
        let html = resp.text().await?;
        let doc = Html::parse_document(&html);

        // Parse full description from .ftext or .description
        let desc_sel = Selector::parse(".ftext p, .ftext, .description p, .description, #description").unwrap();
        let mut desc_parts = Vec::new();
        for el in doc.select(&desc_sel) {
            let t = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
            if !t.is_empty()
                && !t.contains("Kaillera server")
                && !t.contains("Mednafen server")
                && !t.contains("Подробнее...")
                && !t.contains("Случайный скриншот")
                && !desc_parts.contains(&t)
            {
                desc_parts.push(t);
            }
        }
        if !desc_parts.is_empty() {
            game.description = desc_parts.join("\n\n");
        }

        // Parse game screenshots
        let img_sel = Selector::parse(".picture img, .ss-area img, img.game-screens, .screen img, a.highslide img, img[src*='ss.emu-land.net'], img[src*='_pict'], img[src*='screens']").unwrap();
        let mut screens = Vec::new();
        for img in doc.select(&img_sel) {
            if let Some(src) = img.value().attr("src") {
                if let Some(norm) = normalize_image_url(src) {
                    if !screens.contains(&norm) {
                        screens.push(norm);
                    }
                }
            }
        }
        if !screens.is_empty() {
            if game.cover_url.is_none() {
                game.cover_url = Some(screens[0].clone());
            }
            game.screenshot_urls = screens;
        }

        Ok(())
    }
}
