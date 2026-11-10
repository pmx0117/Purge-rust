use purge_core::{ScanOptions as Settings, *};
use purge_engine::{cleaner, scanner};
use std::{
    fs,
    path::Path,
    sync::{Arc, atomic::AtomicBool, mpsc},
};
use tempfile::TempDir;

fn item(path: &Path, category: Category) -> Item {
    Item {
        id: path_id(path),
        name: "fixture".into(),
        path: path.into(),
        safety: Safety::Review,
        description: String::new(),
        kind: match category {
            Category::Caches => ItemKind::ApplicationCache,
            Category::Developer => ItemKind::DeveloperCache,
            Category::LargeFiles => ItemKind::PersonalFile(FileKind::Other),
            Category::Applications => ItemKind::InstalledApplication,
        },
        footprint: scanner::measure(path, &AtomicBool::new(false)),
        project_guard: None,
        modified: None,
        duplicate_group: None,
    }
}
fn project() -> (TempDir, Item, Settings) {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Cargo.toml"), "[package]\nname='fixture'\n").unwrap();
    let artifact = dir.path().join("target");
    fs::create_dir(&artifact).unwrap();
    fs::write(artifact.join("build-output"), b"regenerable").unwrap();
    let mut entry = item(&artifact, Category::Developer);
    entry.kind = ItemKind::ProjectArtifact;
    entry.project_guard = Some(scanner::measure(dir.path(), &AtomicBool::new(false)));
    let settings = Settings {
        project_roots: vec![dir.path().into()],
        stale_project_days: 0,
        ..Default::default()
    };
    (dir, entry, settings)
}
#[test]
fn only_allowlisted_artifacts_reach_recycler() {
    let (_dir, entry, settings) = project();
    let mut calls = 0;
    let report = cleaner::clean_with(&[entry], &settings, |_| {
        calls += 1;
        Ok(())
    });
    assert_eq!(calls, 1);
    assert_eq!(report.moved_ids.len(), 1);
    assert!(report.failures.is_empty());
}
#[test]
fn changed_artifact_never_reaches_recycler() {
    let (_dir, entry, settings) = project();
    fs::write(entry.path.join("new-output"), b"changed").unwrap();
    let report = cleaner::clean_with(&[entry], &settings, |_| panic!("changed path recycled"));
    assert_eq!(report.failures.len(), 1);
}
#[test]
fn source_changes_protect_unchanged_build_output() {
    let (dir, entry, settings) = project();
    fs::write(dir.path().join("main.rs"), b"new source").unwrap();
    assert!(
        cleaner::validate(&entry, &settings)
            .unwrap_err()
            .to_string()
            .contains("新活动")
    );
}

#[test]
fn relabelling_a_project_cannot_bypass_its_activity_proof() {
    let (_directory, mut entry, settings) = project();
    entry.kind = ItemKind::DeveloperCache;
    entry.project_guard = None;
    assert!(
        cleaner::validate(&entry, &settings).is_err(),
        "the real project path, not its display label, determines required proof"
    );
}
#[test]
fn arbitrary_directory_cannot_claim_to_be_cache() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("personal.txt"), b"keep").unwrap();
    let entry = item(dir.path(), Category::Caches);
    assert!(cleaner::validate(&entry, &Settings::default()).is_err());
}
#[test]
fn excluded_child_protects_entire_parent() {
    let (_dir, entry, mut settings) = project();
    settings.exclusions.push(entry.path.join("build-output"));
    assert!(cleaner::validate(&entry, &settings).is_err());
}
#[test]
fn artifact_name_requires_project_manifest() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("keep"), "personal").unwrap();
    let entry = item(&target, Category::Developer);
    let settings = Settings {
        project_roots: vec![dir.path().into()],
        ..Default::default()
    };
    assert!(!scanner::project_artifact(&target));
    assert!(cleaner::validate(&entry, &settings).is_err());
}
#[test]
fn project_keep_marker_and_git_lock_block_cleanup() {
    let (dir, entry, settings) = project();
    fs::write(dir.path().join(".purge-keep"), "").unwrap();
    assert!(cleaner::validate(&entry, &settings).is_err());
    fs::remove_file(dir.path().join(".purge-keep")).unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".git/index.lock"), "").unwrap();
    assert!(cleaner::validate(&entry, &settings).is_err());
}
#[test]
fn recycler_failures_are_reported_without_claiming_success() {
    let (_dir, entry, settings) = project();
    let report = cleaner::clean_with(&[entry], &settings, |_| anyhow::bail!("file locked"));
    assert!(report.moved_ids.is_empty());
    assert_eq!(report.bytes, 0);
    assert!(report.failures[0].contains("file locked"));
}
#[test]
fn duplicate_selection_is_only_recycled_once() {
    let (_dir, entry, settings) = project();
    let mut calls = 0;
    let report = cleaner::clean_with(&[entry.clone(), entry], &settings, |_| {
        calls += 1;
        Ok(())
    });
    assert_eq!(calls, 1);
    assert_eq!(report.moved_ids.len(), 1);
}
#[test]
fn duplicate_detection_compares_contents_and_honors_cancellation() {
    let dir = TempDir::new().unwrap();
    let mut entries = vec![];
    for (name, bytes) in [("a", b"same"), ("b", b"same"), ("c", b"diff")] {
        let p = dir.path().join(name);
        fs::write(&p, bytes).unwrap();
        entries.push(item(&p, Category::LargeFiles));
    }
    let groups = scanner::find_duplicates(&entries, &AtomicBool::new(false));
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[&entries[0].id], groups[&entries[1].id]);
    assert!(!groups.contains_key(&entries[2].id));
    assert!(scanner::find_duplicates(&entries, &AtomicBool::new(true)).is_empty());
}
#[test]
fn cancelled_scan_always_signals_completion() {
    let (tx, rx) = mpsc::channel();
    scanner::scan(
        Settings::default(),
        Category::ALL.to_vec(),
        Arc::new(AtomicBool::new(true)),
        tx,
    );
    assert!(matches!(
        rx.recv().unwrap(),
        scanner::ScanEvent::Finished { cancelled: true }
    ));
}
#[test]
fn installed_apps_can_never_be_recycled() {
    let (_dir, mut entry, settings) = project();
    entry.kind = ItemKind::InstalledApplication;
    assert!(cleaner::validate(&entry, &settings).is_err());
}
#[test]
fn measurement_changes_when_same_size_contents_are_replaced() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("file");
    fs::write(&p, b"1234").unwrap();
    let first = scanner::measure(&p, &AtomicBool::new(false));
    std::thread::sleep(std::time::Duration::from_millis(30));
    fs::write(&p, b"5678").unwrap();
    assert_ne!(first, scanner::measure(&p, &AtomicBool::new(false)));
}
#[cfg(unix)]
#[test]
fn symlinks_are_never_followed() {
    use std::os::unix::fs::symlink;
    let (dir, entry, settings) = project();
    let outside = TempDir::new().unwrap();
    fs::write(outside.path().join("keep"), b"keep").unwrap();
    symlink(outside.path(), entry.path.join("linked")).unwrap();
    let footprint = scanner::measure(&entry.path, &AtomicBool::new(false));
    assert!(footprint.has_links);
    assert!(cleaner::validate(&entry, &settings).is_err());
    assert!(dir.path().exists());
}
#[cfg(windows)]
#[test]
fn junctions_are_never_followed() {
    use std::process::Command;
    let (_dir, entry, settings) = project();
    let outside = TempDir::new().unwrap();
    fs::write(outside.path().join("keep"), b"keep").unwrap();
    let link = entry.path.join("linked");
    assert!(
        Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(outside.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    let footprint = scanner::measure(&entry.path, &AtomicBool::new(false));
    assert!(footprint.has_links);
    assert!(cleaner::validate(&entry, &settings).is_err());
    fs::remove_dir(link).unwrap();
}

#[cfg(windows)]
#[test]
#[ignore = "Explicit Windows integration check; recycles and restores only a generated fixture"]
fn windows_recycle_bin_round_trip() {
    let (directory, entry, settings) = project();
    let report = cleaner::clean(std::slice::from_ref(&entry), &settings);
    assert!(report.failures.is_empty(), "{:?}", report.failures);
    assert_eq!(report.moved_ids, vec![entry.id]);
    assert!(!entry.path.exists());
    let recycled = trash::os_limited::list()
        .unwrap()
        .into_iter()
        .find(|item| {
            purge_engine::platform::same_path(&item.original_parent, directory.path())
                && item.name == entry.path.file_name().unwrap()
        })
        .expect("fixture must be present in Windows Recycle Bin");
    trash::os_limited::restore_all([recycled]).unwrap();
    assert_eq!(
        fs::read(entry.path.join("build-output")).unwrap(),
        b"regenerable"
    );
}

#[test]
fn bulk_select_all_requires_separate_review_selection() {
    let (_dir, mut safe, _settings) = project();
    safe.kind = ItemKind::DeveloperCache;
    safe.safety = Safety::Safe;
    let mut review = safe.clone();
    review.id += 1;
    review.safety = Safety::Review;
    let items = [safe.clone(), review.clone()];
    let mut selected = std::collections::HashSet::new();
    let (add, remove) = selection_toggle(&items, &selected, true);
    assert_eq!(add, vec![safe.id]);
    assert!(remove.is_empty());
    selected.extend(add);
    let (add, remove) = selection_toggle(&items, &selected, true);
    assert!(add.is_empty());
    assert_eq!(remove, vec![safe.id]);
    let (add, _) = selection_toggle(&items, &selected, false);
    assert_eq!(add, vec![review.id]);
}

#[test]
fn merged_cache_rows_keep_every_real_cleanup_candidate() {
    let (_directory, mut first, _settings) = project();
    first.kind = ItemKind::ApplicationCache;
    first.name = "Chrome · Default · Cache".into();
    let mut second = first.clone();
    second.id += 1;
    second.name = "Chrome · Profile 1 · GPUCache".into();
    let expected = first.bytes() + second.bytes();
    let original_ids = vec![first.id, second.id];
    let grouped = group_items(vec![first, second]);
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped[0].name, "Chrome");
    assert_eq!(grouped[0].bytes(), expected);
    assert_eq!(
        grouped[0].items.iter().map(|i| i.id).collect::<Vec<_>>(),
        original_ids
    );
}
