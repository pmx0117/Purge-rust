use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupEntry {
    pub name: String,
    #[serde(with = "history_path")]
    pub path: PathBuf,
    pub bytes: u64,
    pub error: Option<String>,
}

mod history_path {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::path::{Path, PathBuf};

    pub fn serialize<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(path) = path.to_str() {
            return serializer.serialize_str(path);
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            serializer.serialize_bytes(path.as_os_str().as_bytes())
        }
        #[cfg(not(unix))]
        serializer.serialize_str(&path.to_string_lossy())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PathBuf, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            Text(PathBuf),
            Bytes(Vec<u8>),
        }
        match Value::deserialize(deserializer)? {
            Value::Text(path) => Ok(path),
            Value::Bytes(bytes) => {
                #[cfg(unix)]
                {
                    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
                    Ok(PathBuf::from(OsString::from_vec(bytes)))
                }
                #[cfg(not(unix))]
                {
                    let _ = bytes;
                    Err(serde::de::Error::custom(
                        "无法在此系统读取 Unix 原始文件路径",
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub time: String,
    pub bytes: u64,
    pub moved: usize,
    pub failed: usize,
    #[serde(default)]
    pub items: Vec<CleanupEntry>,
    #[serde(default)]
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CleanupReport {
    pub moved_ids: Vec<u64>,
    pub bytes: u64,
    pub failures: Vec<String>,
    pub trash_bytes: Option<u64>,
    pub items: Vec<CleanupEntry>,
}
