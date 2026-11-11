use purge_core::{Category, ScanOptions};
use purge_engine::settings::StoredSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppLayout {
    #[default]
    Cards,
    List,
}

impl AppLayout {
    pub fn from_key(key: &str) -> Self {
        match key {
            "list" => Self::List,
            _ => Self::Cards,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Cards => "cards",
            Self::List => "list",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Safe,
    Review,
    Duplicates,
}
impl Filter {
    pub fn from_key(key: &str) -> Self {
        match key {
            "all" => Self::All,
            "review" => Self::Review,
            _ => Self::Safe,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Safe => "safe",
            Self::Review | Self::Duplicates => "review",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Size,
    Name,
    Oldest,
    Smallest,
    Newest,
}
const SORTS: &[(Sort, &str, &str)] = &[
    (Sort::Size, "size", "最大优先"),
    (Sort::Smallest, "smallest", "最小优先"),
    (Sort::Newest, "newest", "最新优先"),
    (Sort::Oldest, "oldest", "最旧优先"),
    (Sort::Name, "name", "按名称"),
];
impl Sort {
    fn definition(self) -> &'static (Self, &'static str, &'static str) {
        SORTS
            .iter()
            .find(|entry| entry.0 == self)
            .expect("every sort has metadata")
    }
    pub fn key(self) -> &'static str {
        self.definition().1
    }
    pub fn label(self) -> &'static str {
        self.definition().2
    }
    pub fn from_key(key: &str) -> Self {
        SORTS
            .iter()
            .find(|entry| entry.1 == key)
            .map(|entry| entry.0)
            .unwrap_or(Self::Size)
    }
    pub fn all() -> impl Iterator<Item = Self> {
        SORTS.iter().map(|entry| entry.0)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct CategoryPreferences {
    pub filter: Filter,
    pub sort: Sort,
}
#[derive(Debug, Clone)]
pub struct Preferences {
    pub scan: ScanOptions,
    pub dark: bool,
    pub scan_on_start: bool,
    pub app_layout: AppLayout,
    pub minimize_to_tray: bool,
    caches: CategoryPreferences,
    developer: CategoryPreferences,
}
impl Default for Preferences {
    fn default() -> Self {
        StoredSettings::default().into()
    }
}
impl From<StoredSettings> for Preferences {
    fn from(stored: StoredSettings) -> Self {
        Self {
            scan: stored.scan,
            dark: stored.dark,
            scan_on_start: stored.scan_on_start,
            app_layout: AppLayout::from_key(&stored.app_layout),
            minimize_to_tray: stored.minimize_to_tray,
            caches: CategoryPreferences {
                filter: Filter::from_key(&stored.cache_filter),
                sort: Sort::from_key(&stored.cache_sort),
            },
            developer: CategoryPreferences {
                filter: Filter::from_key(&stored.developer_filter),
                sort: Sort::from_key(&stored.developer_sort),
            },
        }
    }
}
impl Preferences {
    pub fn to_stored(&self) -> StoredSettings {
        StoredSettings {
            scan: self.scan.clone(),
            dark: self.dark,
            scan_on_start: self.scan_on_start,
            app_layout: self.app_layout.key().into(),
            minimize_to_tray: self.minimize_to_tray,
            cache_filter: self.caches.filter.key().into(),
            developer_filter: self.developer.filter.key().into(),
            cache_sort: self.caches.sort.key().into(),
            developer_sort: self.developer.sort.key().into(),
        }
    }
    pub fn category(&self, category: Category) -> Option<&CategoryPreferences> {
        match category {
            Category::Caches => Some(&self.caches),
            Category::Developer => Some(&self.developer),
            _ => None,
        }
    }
    pub fn category_mut(&mut self, category: Category) -> Option<&mut CategoryPreferences> {
        match category {
            Category::Caches => Some(&mut self.caches),
            Category::Developer => Some(&mut self.developer),
            _ => None,
        }
    }
}
