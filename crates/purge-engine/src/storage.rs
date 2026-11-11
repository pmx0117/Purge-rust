use crate::settings::StoredSettings;
use anyhow::{Context, Result};
use purge_core::HistoryEntry;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// One repository instance belongs to one application controller. Workers return
/// immutable results; they never write settings or history themselves.
pub struct LocalRepository {
    directory: PathBuf,
}

impl Default for LocalRepository {
    fn default() -> Self {
        Self::new(data_dir())
    }
}
impl LocalRepository {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn ensure_directory(&self) -> Result<&Path> {
        fs::create_dir_all(&self.directory)?;
        Ok(&self.directory)
    }
    pub fn load_settings(&self) -> Result<StoredSettings> {
        read_json(&self.directory.join("settings.json"))
    }
    pub fn save_settings(&self, value: &StoredSettings) -> Result<()> {
        write_json(&self.directory, "settings.json", value)
    }
    pub fn load_history(&self) -> Result<Vec<HistoryEntry>> {
        read_json(&self.directory.join("history.json"))
    }
    pub fn save_history(&self, value: &[HistoryEntry]) -> Result<()> {
        write_json(&self.directory, "history.json", value)
    }
}

pub fn data_dir() -> PathBuf {
    crate::platform::user_data_dir().join("Purge")
}
fn read_json<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    match fs::read(path) {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).with_context(|| format!("无法读取 {}", path.display()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(error) => Err(error).with_context(|| format!("无法读取 {}", path.display())),
    }
}
fn write_json<T: Serialize + ?Sized>(directory: &Path, name: &str, value: &T) -> Result<()> {
    // Serialize before touching the previous file. Each writer owns a distinct
    // temporary file, so interrupted or concurrent writes cannot share a name.
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::create_dir_all(directory)?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(directory.join(name))
        .map_err(|error| error.error)?;
    Ok(())
}

// Compatibility entry points delegate to the same repository implementation.
pub fn load_settings() -> Result<StoredSettings> {
    LocalRepository::default().load_settings()
}
pub fn save_settings(value: &StoredSettings) -> Result<()> {
    LocalRepository::default().save_settings(value)
}
pub fn load_history() -> Result<Vec<HistoryEntry>> {
    LocalRepository::default().load_history()
}
pub fn save_history(value: &[HistoryEntry]) -> Result<()> {
    LocalRepository::default().save_history(value)
}
