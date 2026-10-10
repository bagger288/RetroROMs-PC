pub mod catalog_grid;
pub mod catalog_table;
pub mod downloads_view;
pub mod favorites_view;
pub mod game_detail;
pub mod image_viewer_modal;
pub mod rom_versions_modal;
pub mod settings_view;
pub mod sidebar;
pub mod topbar;
pub mod zip_modal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    Catalog,
    Downloads,
    Favorites,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    Default,
    RatingDesc,
    TitleAsc,
    TitleDesc,
}

impl SortOption {
    pub fn label(&self) -> &'static str {
        match self {
            SortOption::Default => "По умолчанию",
            SortOption::RatingDesc => "★ По рейтингу",
            SortOption::TitleAsc => "А-Я По названию",
            SortOption::TitleDesc => "Я-А По названию",
        }
    }
}
