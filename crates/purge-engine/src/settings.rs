use purge_core::ScanOptions;
use serde::{Deserialize, Serialize};
/// Flat serialized shape remains compatible with existing settings.json files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StoredSettings {
    #[serde(flatten)]
    pub scan: ScanOptions,
    pub dark: bool,
    pub scan_on_start: bool,
    pub cache_filter: String,
    pub developer_filter: String,
    pub cache_sort: String,
    pub developer_sort: String,
}
impl Default for StoredSettings {
    fn default() -> Self {
        Self {
            scan: ScanOptions::default(),
            dark: false,
            scan_on_start: true,
            cache_filter: "safe".into(),
            developer_filter: "safe".into(),
            cache_sort: "size".into(),
            developer_sort: "size".into(),
        }
    }
}
