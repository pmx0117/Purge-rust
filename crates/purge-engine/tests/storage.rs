use std::{fs, process::Command};
use tempfile::TempDir;

#[test]
fn unrelated_temporary_path_does_not_break_preference_save() {
    let directory = TempDir::new().unwrap();
    let data = if cfg!(target_os = "macos") {
        directory.path().join("Library/Application Support/Purge")
    } else {
        directory.path().join("Purge")
    };
    fs::create_dir_all(data.join("settings.json.tmp")).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "save_preferences_probe",
            "--ignored",
            "--nocapture",
        ])
        .env("LOCALAPPDATA", directory.path())
        .env("XDG_DATA_HOME", directory.path())
        .env("HOME", directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    );
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(data.join("settings.json")).unwrap()).unwrap();
    assert_eq!(saved["minimum_file_mb"], 500);
    assert!(data.join("settings.json.tmp").is_dir());
}

#[test]
#[ignore = "Runs in a subprocess with isolated platform data directories"]
fn save_preferences_probe() {
    let settings = purge_engine::settings::StoredSettings {
        scan: purge_core::ScanOptions {
            minimum_file_mb: 500,
            ..Default::default()
        },
        ..Default::default()
    };
    purge_engine::storage::save_settings(&settings).unwrap();
    assert_eq!(
        purge_engine::storage::load_settings()
            .unwrap()
            .scan
            .minimum_file_mb,
        500
    );
}

#[test]
fn repositories_keep_independent_preferences_and_preserve_legacy_json() {
    let first = TempDir::new().unwrap();
    let second = TempDir::new().unwrap();
    let a = purge_engine::storage::LocalRepository::new(first.path());
    let b = purge_engine::storage::LocalRepository::new(second.path());
    let mut settings = a.load_settings().unwrap();
    settings.dark = true;
    settings.scan.minimum_file_mb = 500;
    a.save_settings(&settings).unwrap();
    assert!(a.load_settings().unwrap().dark);
    assert!(!b.load_settings().unwrap().dark);
    let json: serde_json::Value =
        serde_json::from_slice(&fs::read(first.path().join("settings.json")).unwrap()).unwrap();
    assert_eq!(json["minimum_file_mb"], 500);
    assert!(json.get("scan").is_none());
}

#[test]
fn existing_partial_settings_load_with_defaults() {
    let directory = TempDir::new().unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"dark":true,"minimum_file_mb":25}"#,
    )
    .unwrap();
    let repository = purge_engine::storage::LocalRepository::new(directory.path());
    let settings = repository.load_settings().unwrap();
    assert!(settings.dark);
    assert_eq!(settings.scan.minimum_file_mb, 25);
    assert_eq!(settings.scan.stale_project_days, 30);
    assert_eq!(settings.cache_filter, "safe");
}

#[test]
fn malformed_settings_are_reported_without_replacing_the_file() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("settings.json");
    fs::write(&path, b"{broken").unwrap();
    let repository = purge_engine::storage::LocalRepository::new(directory.path());
    assert!(repository.load_settings().is_err());
    assert_eq!(fs::read(path).unwrap(), b"{broken");
}

#[test]
fn cleanup_history_round_trips_through_its_own_file() {
    let directory = TempDir::new().unwrap();
    let repository = purge_engine::storage::LocalRepository::new(directory.path());
    let entry = purge_core::HistoryEntry {
        time: "2026-11-10 14:00:00".into(),
        bytes: 1024,
        moved: 2,
        failed: 1,
    };
    repository.save_history(&[entry]).unwrap();
    let history = repository.load_history().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].moved, 2);
    assert_eq!(history[0].failed, 1);
    assert_eq!(history[0].bytes, 1024);
    assert!(!directory.path().join("settings.json").exists());
}

#[test]
fn application_layout_choice_survives_other_preference_saves() {
    let directory = TempDir::new().unwrap();
    let repository = purge_engine::storage::LocalRepository::new(directory.path());
    std::fs::write(
        directory.path().join("settings.json"),
        r#"{"app_layout":"list","dark":true}"#,
    )
    .unwrap();
    let mut settings = repository.load_settings().unwrap();
    settings.dark = false;
    repository.save_settings(&settings).unwrap();
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("settings.json")).unwrap())
            .unwrap();
    assert_eq!(
        saved.get("app_layout").and_then(|value| value.as_str()),
        Some("list")
    );
}

#[test]
fn tray_preference_survives_unrelated_setting_updates() {
    let directory = TempDir::new().unwrap();
    let repository = purge_engine::storage::LocalRepository::new(directory.path());
    fs::write(
        directory.path().join("settings.json"),
        r#"{"minimize_to_tray":false}"#,
    )
    .unwrap();
    let mut settings = repository.load_settings().unwrap();
    settings.dark = true;
    repository.save_settings(&settings).unwrap();
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.path().join("settings.json")).unwrap()).unwrap();
    assert_eq!(
        saved.get("minimize_to_tray"),
        Some(&serde_json::Value::Bool(false))
    );
}
