use super::{
    collector::{Candidate, ScanCollector},
    excluded, has_link_component,
};
use crate::platform;
use purge_core::{FileKind, ItemKind, Safety, ScanOptions};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use walkdir::{DirEntry, WalkDir};
pub fn file_kind(path: &Path) -> FileKind {
    FileKind::from_extension(
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or(""),
    )
}
pub(super) fn scan_large_files(roots: &[PathBuf], collector: &mut ScanCollector<'_>) {
    let mut errors = 0;
    for root in roots {
        if collector.cancelled() {
            return;
        }
        errors += scan_root(root, collector);
    }
    if errors > 0 {
        collector.emit(super::ScanEvent::Warning(format!(
            "{errors} 个位置无法访问，已跳过。"
        )));
    }
}
fn scan_root(root: &Path, collector: &mut ScanCollector<'_>) -> usize {
    if !root.exists() || has_link_component(root) {
        return 0;
    }
    collector.progress(format!("正在扫描 {}", root.display()));
    let settings = collector.settings;
    let cancel = collector.cancel;
    let walker = WalkDir::new(root)
        .follow_links(false)
        .max_depth(32)
        .into_iter()
        .filter_entry(|entry| descend(entry, settings, cancel));
    let mut errors = 0;
    for entry in walker {
        if collector.cancelled() {
            return errors;
        }
        match entry {
            Ok(entry) => scan_file(entry, collector),
            Err(_) => errors += 1,
        }
    }
    errors
}
fn descend(entry: &DirEntry, settings: &ScanOptions, cancel: &AtomicBool) -> bool {
    if cancel.load(Ordering::Relaxed) || excluded(entry.path(), settings) {
        return false;
    }
    if hidden_or_generated(entry) {
        return false;
    }
    fs::symlink_metadata(entry.path()).is_ok_and(|metadata| {
        !platform::is_link(&metadata) && !platform::is_cloud_placeholder(&metadata)
    })
}
fn hidden_or_generated(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    name.starts_with('.')
        || [
            "node_modules",
            "target",
            "venv",
            "Pods",
            "AppData",
            "$RECYCLE.BIN",
        ]
        .contains(&name.as_ref())
}
fn scan_file(entry: DirEntry, collector: &mut ScanCollector<'_>) {
    if !entry.file_type().is_file() {
        return;
    }
    let Ok(metadata) = entry.metadata() else {
        return;
    };
    if metadata.len()
        < collector
            .settings
            .minimum_file_mb
            .saturating_mul(1024 * 1024)
    {
        return;
    }
    collector.add(Candidate {
        path: entry.path().to_path_buf(),
        name: entry.file_name().to_string_lossy().into_owned(),
        safety: Safety::Review,
        description: "这是你的个人文件。请检查内容，确认不再需要后再移入回收站。".into(),
        kind: ItemKind::PersonalFile(file_kind(entry.path())),
        measured: None,
    });
}
