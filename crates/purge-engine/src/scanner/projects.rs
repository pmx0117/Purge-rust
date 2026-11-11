use super::{
    collector::{Candidate, ScanCollector},
    excluded, has_link_component, measure,
};
use crate::platform;
use purge_core::{ItemKind, Safety, ScanOptions};
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::SystemTime,
};
use walkdir::{DirEntry, WalkDir};
pub fn project_artifact(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    match path.file_name().and_then(|s| s.to_str()).unwrap_or("") {
        "node_modules" => parent.join("package.json").is_file(),
        "target" => parent.join("Cargo.toml").is_file(),
        ".venv" | "venv" => path.join("pyvenv.cfg").is_file(),
        ".gradle" => {
            parent.join("build.gradle").is_file()
                || parent.join("build.gradle.kts").is_file()
                || parent.join("settings.gradle").is_file()
        }
        "build" => parent.join("pubspec.yaml").is_file(),
        _ => false,
    }
}

pub(super) fn scan_projects(collector: &mut ScanCollector<'_>) {
    let roots = collector.settings.project_roots.clone();
    for root in roots {
        if collector.cancelled() {
            return;
        }
        scan_project_root(&root, collector);
    }
}
fn scan_project_root(root: &Path, collector: &mut ScanCollector<'_>) {
    if has_link_component(root) {
        return;
    }
    let settings = collector.settings;
    let cancel = collector.cancel;
    let walker = WalkDir::new(root)
        .follow_links(false)
        .max_depth(10)
        .into_iter()
        .filter_entry(|entry| descend(entry, root, settings, cancel));
    for entry in walker.flatten() {
        if collector.cancelled() {
            return;
        }
        scan_project_entry(entry, collector);
    }
}
fn descend(entry: &DirEntry, root: &Path, settings: &ScanOptions, cancel: &AtomicBool) -> bool {
    if ignored(entry, settings, cancel) {
        return false;
    }
    if fs::symlink_metadata(entry.path()).is_ok_and(|metadata| platform::is_link(&metadata)) {
        return false;
    }
    !entry
        .path()
        .ancestors()
        .skip(1)
        .take_while(|parent| *parent != root)
        .any(project_artifact)
}
fn ignored(entry: &DirEntry, settings: &ScanOptions, cancel: &AtomicBool) -> bool {
    cancel.load(Ordering::Relaxed)
        || excluded(entry.path(), settings)
        || entry.file_name() == ".git"
}
fn scan_project_entry(entry: DirEntry, collector: &mut ScanCollector<'_>) {
    if !entry.file_type().is_dir() || !project_artifact(entry.path()) {
        return;
    }
    let Some(project) = entry.path().parent() else {
        return;
    };
    if project_locked(project) {
        return;
    }
    collector.progress(format!("正在检查项目 {}", project.display()));
    let state = measure(project, collector.cancel);
    if !collector
        .settings
        .project_is_stale(&state, SystemTime::now())
    {
        return;
    }
    let name = format!(
        "{} / {}",
        project.file_name().unwrap_or_default().to_string_lossy(),
        entry.file_name().to_string_lossy()
    );
    collector.add(Candidate {
        path: entry.path().to_path_buf(),
        name,
        safety: Safety::Review,
        description:
            "项目的依赖或构建产物。请停止开发服务器并确认项目已闲置；之后需要重新安装依赖或构建。"
                .into(),
        kind: ItemKind::ProjectArtifact,
        measured: None,
    });
}
fn project_locked(project: &Path) -> bool {
    project.join(".git/index.lock").exists() || project.join(".purge-keep").exists()
}
