use purge_core::*;
use purge_engine::scanner::{self, Rule, ScanEvent, ScanScope};
use std::{
    fs,
    path::Path,
    sync::{Arc, atomic::AtomicBool, mpsc},
    time::{Duration, SystemTime},
};
use tempfile::TempDir;
fn rule(path: &Path, children: bool) -> Rule {
    Rule {
        path: path.into(),
        name: "fixture cache".into(),
        description: "cache".into(),
        category: Category::Caches,
        safety: Safety::Safe,
        children,
        minimum_age: Duration::from_secs(86400),
    }
}
fn scan(scope: ScanScope, settings: ScanOptions, categories: Vec<Category>) -> Vec<ScanEvent> {
    let (sender, receiver) = mpsc::channel();
    scanner::scan_in(
        scope,
        settings,
        categories,
        Arc::new(AtomicBool::new(false)),
        sender,
    );
    receiver.into_iter().collect()
}
#[test]
fn an_explicit_scope_scans_only_eligible_cache_project_and_personal_candidates() {
    let root = TempDir::new().unwrap();
    let cache = root.path().join("cache");
    fs::create_dir(&cache).unwrap();
    fs::write(cache.join("entry"), b"cache").unwrap();
    let temporary = root.path().join("temporary");
    fs::create_dir(&temporary).unwrap();
    let old = temporary.join("old.tmp");
    fs::write(&old, b"old").unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&old)
        .unwrap()
        .set_times(
            fs::FileTimes::new().set_modified(SystemTime::now() - Duration::from_secs(48 * 3600)),
        )
        .unwrap();
    fs::write(temporary.join("recent.tmp"), b"recent").unwrap();
    let project = root.path().join("projects/app");
    fs::create_dir_all(project.join("target")).unwrap();
    fs::write(project.join("Cargo.toml"), "[package]").unwrap();
    fs::write(project.join("target/output"), b"output").unwrap();
    let personal = root.path().join("personal");
    fs::create_dir(&personal).unwrap();
    fs::File::create(personal.join("movie.MP4"))
        .unwrap()
        .set_len(2 * 1024 * 1024)
        .unwrap();
    fs::write(personal.join("small.txt"), b"small").unwrap();
    fs::File::create(personal.join(".private"))
        .unwrap()
        .set_len(2 * 1024 * 1024)
        .unwrap();
    let scope = ScanScope {
        rules: vec![rule(&cache, false), rule(&temporary, true)],
        personal_roots: vec![personal],
        ..Default::default()
    };
    let settings = ScanOptions {
        minimum_file_mb: 1,
        stale_project_days: 0,
        project_roots: vec![root.path().join("projects")],
        ..Default::default()
    };
    let events = scan(
        scope,
        settings,
        vec![Category::Caches, Category::Developer, Category::LargeFiles],
    );
    let items: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            ScanEvent::Item(item) => Some(item),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 4);
    assert!(items.iter().any(|item| item.path == old));
    assert!(
        items
            .iter()
            .any(|item| item.kind == ItemKind::ProjectArtifact && item.project_guard.is_some())
    );
    assert!(
        items
            .iter()
            .any(|item| item.kind == ItemKind::PersonalFile(FileKind::Video)
                && item.safety == Safety::Review)
    );
    assert!(!items.iter().any(|item| item.name.contains("recent")));
    assert!(!items.iter().any(|item| item.name == ".private"));
    assert!(matches!(
        events.last(),
        Some(ScanEvent::Finished { cancelled: false })
    ));
}
#[test]
fn excluded_paths_and_locked_projects_are_not_published() {
    let root = TempDir::new().unwrap();
    let cache = root.path().join("cache");
    fs::create_dir(&cache).unwrap();
    fs::write(cache.join("entry"), b"cache").unwrap();
    let project = root.path().join("project");
    fs::create_dir_all(project.join("target")).unwrap();
    fs::write(project.join("Cargo.toml"), "[package]").unwrap();
    fs::write(project.join("target/output"), b"output").unwrap();
    fs::write(project.join(".purge-keep"), "").unwrap();
    let scope = ScanScope {
        rules: vec![rule(&cache, false)],
        ..Default::default()
    };
    let settings = ScanOptions {
        exclusions: vec![cache],
        project_roots: vec![root.path().into()],
        stale_project_days: 0,
        ..Default::default()
    };
    let events = scan(scope, settings, vec![Category::Caches, Category::Developer]);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, ScanEvent::Item(_)))
    );
}

#[test]
fn discovery_lists_only_cache_leaves_in_real_user_browser_profiles() {
    let root = TempDir::new().unwrap();
    let local = root.path().join("local");
    let roaming = root.path().join("roaming");
    for name in ["Default", "Profile 1", "System Profile"] {
        fs::create_dir_all(local.join("Google/Chrome/User Data").join(name)).unwrap();
    }
    let rules = scanner::discover_cache_rules(root.path(), &local, &roaming);
    let chrome: Vec<_> = rules
        .iter()
        .filter(|rule| rule.name.starts_with("Google Chrome"))
        .collect();
    assert_eq!(chrome.len(), 6);
    assert!(chrome.iter().all(|rule| rule.safety == Safety::Safe));
    assert!(
        !chrome
            .iter()
            .any(|rule| rule.name.contains("System Profile"))
    );
    assert!(!rules.iter().any(|rule| rule.path.ends_with("Cookies")));
    assert!(
        rules
            .iter()
            .any(|rule| rule.name == "NuGet" && rule.safety == Safety::Review)
    );
}

#[test]
fn each_supported_project_type_requires_its_matching_manifest() {
    let root = TempDir::new().unwrap();
    for (index, (artifact, manifest)) in [
        ("node_modules", "package.json"),
        ("target", "Cargo.toml"),
        ("build", "pubspec.yaml"),
        (".gradle", "build.gradle"),
        (".gradle", "build.gradle.kts"),
        (".gradle", "settings.gradle"),
    ]
    .into_iter()
    .enumerate()
    {
        let project = root.path().join(index.to_string());
        fs::create_dir_all(project.join(artifact)).unwrap();
        assert!(!scanner::project_artifact(&project.join(artifact)));
        fs::write(project.join(manifest), "").unwrap();
        assert!(scanner::project_artifact(&project.join(artifact)));
    }
    for name in [".venv", "venv"] {
        let path = root.path().join(name);
        fs::create_dir(&path).unwrap();
        assert!(!scanner::project_artifact(&path));
        fs::write(path.join("pyvenv.cfg"), "").unwrap();
        assert!(scanner::project_artifact(&path));
    }
    assert!(!scanner::project_artifact(&root.path().join("unknown")));
}

#[test]
fn application_snapshots_keep_registration_warnings_and_cannot_be_cleaned_as_files() {
    let application = Item {
        id: 42,
        name: "Fixture Editor".into(),
        path: "fixture-registration".into(),
        safety: Safety::Review,
        description: "Fixture publisher".into(),
        kind: ItemKind::InstalledApplication,
        footprint: Footprint::default(),
        project_guard: None,
        modified: None,
        duplicate_group: None,
    };
    let events = scan(
        ScanScope {
            applications: vec![application],
            application_warning: Some("fixture registration unavailable".into()),
            ..Default::default()
        },
        ScanOptions::default(),
        vec![Category::Applications],
    );
    assert!(events.iter().any(|event| matches!(event,
        ScanEvent::Item(item) if item.name == "Fixture Editor" && !item.cleanable()
    )));
    assert!(events.iter().any(|event| matches!(event,
        ScanEvent::Warning(message) if message == "fixture registration unavailable"
    )));
    assert!(matches!(
        events.last(),
        Some(ScanEvent::Finished { cancelled: false })
    ));
}
