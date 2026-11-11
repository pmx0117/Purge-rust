mod queries;
mod review;
mod scanning;
use crate::preferences::{Filter, Preferences, Sort};
use purge_core::*;
pub use review::ReviewPlan;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Overview,
    Category(Category),
    Settings,
    History,
    About,
}
impl Page {
    pub fn title(self) -> &'static str {
        if let Self::Category(category) = self {
            return category.label();
        }
        PAGE_TITLES
            .iter()
            .find(|entry| entry.0 == self)
            .map(|entry| entry.1)
            .unwrap_or("Purge")
    }
    pub fn scan_category(self) -> Option<Category> {
        match self {
            Self::Category(category) => Some(category),
            _ => None,
        }
    }
    pub fn shows_scan_status(self) -> bool {
        matches!(self, Self::Overview | Self::Category(_))
    }
}
const PAGE_TITLES: &[(Page, &str)] = &[
    (Page::Overview, "概览"),
    (Page::Settings, "设置"),
    (Page::History, "清理历史"),
    (Page::About, "关于 Purge"),
];
#[derive(Debug)]
pub struct AppModel {
    pub page: Page,
    pub settings: Preferences,
    pub history: Vec<HistoryEntry>,
    pub items: Vec<Item>,
    pub selected: HashSet<u64>,
    pub filter: Filter,
    pub kind_filter: Option<FileKind>,
    pub sort: Sort,
    pub disk: DiskInfo,
    pub trash_bytes: Option<u64>,
    pub scanned: HashSet<Category>,
    pub scan_categories: Vec<Category>,
    pub status: String,
    pub last_scan: Option<String>,
    pub scan_cancelled: bool,
    pub notice: Option<String>,
    pub warnings: Vec<String>,
    pub review: Option<ReviewPlan>,
}
impl Default for AppModel {
    fn default() -> Self {
        Self::new(Preferences::default())
    }
}
impl AppModel {
    pub fn new(settings: Preferences) -> Self {
        Self {
            page: Page::Overview,
            settings,
            history: vec![],
            items: vec![],
            selected: HashSet::new(),
            filter: Filter::All,
            kind_filter: None,
            sort: Sort::Size,
            disk: DiskInfo::default(),
            trash_bytes: None,
            scanned: HashSet::new(),
            scan_categories: vec![],
            status: "准备就绪".into(),
            last_scan: None,
            scan_cancelled: false,
            notice: None,
            warnings: vec![],
            review: None,
        }
    }
    fn saved_choices(&self, category: Category) -> crate::preferences::CategoryPreferences {
        self.settings.category(category).copied().unwrap_or(
            crate::preferences::CategoryPreferences {
                filter: Filter::All,
                sort: Sort::Size,
            },
        )
    }
    pub fn saved_filter(&self, category: Category) -> Filter {
        self.saved_choices(category).filter
    }
    pub fn saved_sort(&self, category: Category) -> Sort {
        self.saved_choices(category).sort
    }
    pub fn navigate(&mut self, page: Page) -> bool {
        if self.review.is_some() {
            return false;
        }
        self.page = page;
        self.kind_filter = None;
        self.filter = page
            .scan_category()
            .map(|category| self.saved_filter(category))
            .unwrap_or(Filter::All);
        self.sort = page
            .scan_category()
            .map(|category| self.saved_sort(category))
            .unwrap_or(Sort::Size);
        true
    }
    pub fn set_filter(&mut self, filter: Filter) -> bool {
        if self.review.is_some() {
            return false;
        }
        self.filter = filter;
        self.update_saved_choice(|preferences| preferences.filter = filter);
        true
    }
    fn update_saved_choice(
        &mut self,
        update: impl FnOnce(&mut crate::preferences::CategoryPreferences),
    ) {
        if let Some(value) = self
            .page
            .scan_category()
            .and_then(|category| self.settings.category_mut(category))
        {
            update(value);
        }
    }
    pub fn set_sort(&mut self, sort: Sort) {
        self.sort = sort;
        self.update_saved_choice(|preferences| preferences.sort = sort);
    }
    pub fn choice_is_persistent(&self) -> bool {
        self.page
            .scan_category()
            .and_then(|category| self.settings.category(category))
            .is_some()
    }
    pub fn total(&self, category: Category) -> u64 {
        self.items
            .iter()
            .filter(|item| item.category() == category)
            .map(Item::bytes)
            .sum()
    }
    pub fn safe_bytes(&self) -> u64 {
        self.items
            .iter()
            .filter(|item| item.safe_cleanable())
            .map(Item::bytes)
            .sum()
    }
}

mod results;
pub use results::{FilterTab, SelectionSummary};
