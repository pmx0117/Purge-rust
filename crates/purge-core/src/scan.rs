use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScanOptions {
    pub minimum_file_mb: u64,
    pub stale_project_days: u64,
    pub project_roots: Vec<PathBuf>,
    pub exclusions: Vec<PathBuf>,
}

impl ScanOptions {
    /// Both discovery and cleanup require readable, non-future project activity.
    pub fn project_is_stale(&self, state: &crate::Footprint, now: SystemTime) -> bool {
        state.is_complete()
            && state.older_than(
                now,
                Duration::from_secs(self.stale_project_days.saturating_mul(86400)),
            )
    }
}
impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            minimum_file_mb: 100,
            stale_project_days: 30,
            project_roots: vec![],
            exclusions: vec![],
        }
    }
}
