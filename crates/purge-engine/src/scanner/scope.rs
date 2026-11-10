use super::Rule;
use crate::platform;
use purge_core::{Category, DiskInfo, Item};
use std::path::PathBuf;
#[derive(Default)]
pub struct ScanScope {
    pub rules: Vec<Rule>,
    pub personal_roots: Vec<PathBuf>,
    pub applications: Vec<Item>,
    pub application_warning: Option<String>,
    pub disk: DiskInfo,
    pub trash_bytes: Option<u64>,
}
impl ScanScope {
    pub fn current(categories: &[Category]) -> Self {
        let (applications, application_warning) = application_snapshot(categories);
        Self {
            rules: super::cache_rules(),
            personal_roots: categories
                .contains(&Category::LargeFiles)
                .then(platform::personal_roots)
                .unwrap_or_default(),
            applications,
            application_warning,
            disk: platform::disk_info().unwrap_or_default(),
            trash_bytes: platform::trash_bytes().unwrap_or(None),
        }
    }
}
fn application_snapshot(categories: &[Category]) -> (Vec<Item>, Option<String>) {
    categories
        .contains(&Category::Applications)
        .then(platform::installed_apps)
        .unwrap_or_else(|| Ok(vec![]))
        .map(|items| (items, None))
        .unwrap_or_else(|error| (vec![], Some(format!("无法读取应用列表：{error}"))))
}
