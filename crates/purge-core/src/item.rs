use crate::ItemKind;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::SystemTime};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Caches,
    Developer,
    LargeFiles,
    Applications,
}

impl Category {
    pub const ALL: [Self; 4] = [
        Self::Caches,
        Self::Developer,
        Self::LargeFiles,
        Self::Applications,
    ];
    pub fn label(self) -> &'static str {
        CATEGORY_NAMES
            .iter()
            .find(|entry| entry.0 == self)
            .map(|entry| entry.1)
            .unwrap_or("未知分类")
    }
}

const CATEGORY_NAMES: &[(Category, &str)] = &[
    (Category::Caches, "应用缓存"),
    (Category::Developer, "开发工具"),
    (Category::LargeFiles, "大文件"),
    (Category::Applications, "已安装应用"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Safety {
    Safe,
    Review,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Footprint {
    pub bytes: u64,
    pub files: u64,
    pub fingerprint: u64,
    pub incomplete: bool,
    pub has_links: bool,
    pub newest: Option<SystemTime>,
}

impl Footprint {
    pub fn older_than(&self, now: SystemTime, minimum: std::time::Duration) -> bool {
        self.newest
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age >= minimum)
    }

    pub fn is_complete(&self) -> bool {
        !self.incomplete && !self.has_links
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,
    pub safety: Safety,
    pub description: String,
    pub kind: ItemKind,
    pub footprint: Footprint,
    /// Project source and git activity must also remain unchanged before cleanup.
    pub project_guard: Option<Footprint>,
    pub modified: Option<SystemTime>,
    pub duplicate_group: Option<u64>,
}

impl Item {
    pub fn category(&self) -> Category {
        self.kind.category()
    }
    pub fn is_application(&self) -> bool {
        self.kind == ItemKind::InstalledApplication
    }
    pub fn safe_cleanable(&self) -> bool {
        self.kind.allows_safe_clean() && self.safety == Safety::Safe && self.cleanable()
    }
    pub fn bytes(&self) -> u64 {
        self.footprint.bytes
    }
    pub fn cleanable(&self) -> bool {
        !self.is_application() && self.footprint.is_complete()
    }
}
