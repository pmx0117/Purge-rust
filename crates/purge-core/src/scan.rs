use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScanOptions {
    pub minimum_file_mb: u64,
    pub stale_project_days: u64,
    pub project_roots: Vec<PathBuf>,
    pub exclusions: Vec<PathBuf>,
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
