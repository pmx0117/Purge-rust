use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub time: String,
    pub bytes: u64,
    pub moved: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CleanupReport {
    pub moved_ids: Vec<u64>,
    pub bytes: u64,
    pub failures: Vec<String>,
    pub trash_bytes: Option<u64>,
}
