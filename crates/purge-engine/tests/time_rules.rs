use purge_core::{Category, Item, ScanOptions};
use purge_engine::{
    cleaner,
    scanner::{self, ScanEvent, ScanScope},
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool, mpsc},
    time::{Duration, SystemTime},
};
use tempfile::TempDir;

const DAY: u64 = 24 * 60 * 60;

fn set_modified(path: &Path, modified: SystemTime) {
    let mut options = fs::OpenOptions::new();
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_WRITE_ATTRIBUTES and FILE_FLAG_BACKUP_SEMANTICS also open directories.
        options.access_mode(0x100).custom_flags(0x02000000);
    }
    #[cfg(not(windows))]
    options.read(true);
    options
        .open(path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
}

fn project(root: &Path, name: &str, modified: SystemTime) -> PathBuf {
    let directory = root.join(name);
    fs::create_dir_all(directory.join("target")).unwrap();
    fs::write(directory.join("Cargo.toml"), "[package]").unwrap();
    fs::write(directory.join("main.rs"), "fn main() {}").unwrap();
    fs::write(directory.join("target/output"), "output").unwrap();
    for entry in walkdir::WalkDir::new(&directory).contents_first(true) {
        set_modified(entry.unwrap().path(), modified);
    }
    directory
}

fn scan(local: &Path, projects: &Path, idle_days: u64) -> Vec<Item> {
    let (sender, receiver) = mpsc::channel();
    scanner::scan_in(
        ScanScope {
            rules: scanner::discover_cache_rules(local, local, local),
            ..Default::default()
        },
        ScanOptions {
            stale_project_days: idle_days,
            project_roots: vec![projects.to_owned()],
            ..Default::default()
        },
        vec![Category::Caches, Category::Developer],
        Arc::new(AtomicBool::new(false)),
        sender,
    );
    receiver
        .into_iter()
        .filter_map(|event| match event {
            ScanEvent::Item(item) => Some(*item),
            _ => None,
        })
        .collect()
}

#[test]
fn temporary_retention_and_cache_rules_do_not_follow_project_idle_days() {
    let directory = TempDir::new().unwrap();
    let local = directory.path().join("local");
    let projects = directory.path().join("projects");
    fs::create_dir_all(local.join("Temp")).unwrap();
    fs::create_dir_all(local.join("CrashDumps")).unwrap();
    let now = SystemTime::now();
    let old = local.join("Temp/old.tmp");
    fs::write(&old, "old").unwrap();
    set_modified(&old, now - Duration::from_secs(2 * DAY));
    let recent = local.join("Temp/recent.tmp");
    fs::write(&recent, "recent").unwrap();
    set_modified(&recent, now - Duration::from_secs(DAY / 2));
    let crash = local.join("CrashDumps/old.dmp");
    fs::write(&crash, "crash").unwrap();
    set_modified(&crash, now - Duration::from_secs(2 * DAY));
    let active = local.join("Temp/old-folder-with-recent-child");
    fs::create_dir(&active).unwrap();
    fs::write(active.join("still-active"), "active").unwrap();
    set_modified(&active, now - Duration::from_secs(2 * DAY));
    let cache = local.join("Google/Chrome/User Data/Default/Cache");
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("new-cache"), "cache").unwrap();
    let forty = project(
        &projects,
        "idle-40-days",
        now - Duration::from_secs(40 * DAY),
    );
    let old_project = project(
        &projects,
        "idle-120-days",
        now - Duration::from_secs(120 * DAY),
    );

    let expected_caches = BTreeSet::from([old, crash, cache]);
    for idle_days in [0, 30, 90, 180] {
        let items = scan(&local, &projects, idle_days);
        let cache_paths: BTreeSet<_> = items
            .iter()
            .filter(|item| item.category() == Category::Caches)
            .map(|item| item.path.clone())
            .collect();
        assert_eq!(
            cache_paths, expected_caches,
            "project idle days = {idle_days}"
        );
        assert_eq!(
            items.iter().any(|item| item.path == forty.join("target")),
            idle_days <= 40
        );
        assert_eq!(
            items
                .iter()
                .any(|item| item.path == old_project.join("target")),
            idle_days <= 120
        );
    }
}

#[test]
fn cleanup_rechecks_the_current_project_setting_and_source_activity() {
    let directory = TempDir::new().unwrap();
    let projects = directory.path().join("projects");
    let local = directory.path().join("local");
    let now = SystemTime::now();
    let forty = project(
        &projects,
        "idle-40-days",
        now - Duration::from_secs(40 * DAY),
    );
    let old = project(
        &projects,
        "idle-120-days",
        now - Duration::from_secs(120 * DAY),
    );
    let items = scan(&local, &projects, 30);
    let young_item = items
        .iter()
        .find(|item| item.path == forty.join("target"))
        .unwrap();
    let old_item = items
        .iter()
        .find(|item| item.path == old.join("target"))
        .unwrap();
    let mut settings = ScanOptions {
        project_roots: vec![projects],
        stale_project_days: 90,
        ..Default::default()
    };
    assert!(
        cleaner::validate(young_item, &settings)
            .unwrap_err()
            .to_string()
            .contains("闲置时间")
    );
    cleaner::validate(old_item, &settings).unwrap();
    settings.stale_project_days = 180;
    assert!(
        cleaner::validate(old_item, &settings)
            .unwrap_err()
            .to_string()
            .contains("闲置时间")
    );
    settings.stale_project_days = 90;
    fs::write(old.join("main.rs"), "fn changed() {}").unwrap();
    assert!(
        cleaner::validate(old_item, &settings)
            .unwrap_err()
            .to_string()
            .contains("新活动")
    );
}

#[test]
fn show_all_projects_does_not_publish_a_future_dated_activity_snapshot() {
    let directory = TempDir::new().unwrap();
    let projects = directory.path().join("projects");
    let local = directory.path().join("local");
    project(
        &projects,
        "clock-ahead",
        SystemTime::now() + Duration::from_secs(DAY),
    );
    assert!(
        scan(&local, &projects, 0).is_empty(),
        "unknown or future project activity must agree with cleanup validation"
    );
}
