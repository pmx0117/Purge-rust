#![cfg(unix)]
use purge_engine::{platform, storage};
use std::path::PathBuf;

#[cfg(target_os = "linux")]
#[test]
fn linux_uses_xdg_directories_instead_of_windows_or_temporary_paths() {
    let home = platform::home_dir();
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home.join(".cache"));
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home.join(".config"));
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home.join(".local/share"));
    assert_eq!(platform::local_dir(), cache);
    assert_eq!(platform::roaming_dir(), config);
    assert_eq!(storage::data_dir(), data.join("Purge"));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_uses_user_library_locations() {
    let home = platform::home_dir();
    assert_eq!(platform::local_dir(), home.join("Library/Caches"));
    assert_eq!(
        platform::roaming_dir(),
        home.join("Library/Application Support")
    );
    assert_eq!(
        storage::data_dir(),
        home.join("Library/Application Support/Purge")
    );
}

#[cfg(unix)]
#[test]
fn unix_disk_capacity_reports_the_actual_home_volume() {
    let disk = platform::disk_info().unwrap();
    assert!(disk.total > 0);
    assert!(disk.free <= disk.total);
    assert!(PathBuf::from(&disk.name).is_absolute());
    assert!(platform::within(
        &platform::home_dir().canonicalize().unwrap(),
        &PathBuf::from(&disk.name)
    ));
}

#[test]
fn unix_cache_catalogs_never_include_windows_or_shared_roots() {
    use purge_engine::{platform::OperatingSystem, scanner};
    let fixture = tempfile::TempDir::new().unwrap();
    let home = fixture.path();
    for (system, cache, config, browser) in [
        (
            OperatingSystem::Linux,
            home.join(".cache"),
            home.join(".config"),
            "google-chrome",
        ),
        (
            OperatingSystem::MacOs,
            home.join("Library/Caches"),
            home.join("Library/Application Support"),
            "Google/Chrome",
        ),
    ] {
        std::fs::create_dir_all(cache.join(browser).join("Default")).unwrap();
        let rules = scanner::discover_cache_rules_for(system, home, &cache, &config);
        assert!(
            rules
                .iter()
                .any(|rule| rule.path == cache.join(browser).join("Default/Cache"))
        );
        assert!(
            rules
                .iter()
                .any(|rule| rule.path == home.join(".npm/_cacache"))
        );
        assert!(
            rules
                .iter()
                .all(|rule| !rule.path.to_string_lossy().contains("AppData"))
        );
        assert!(!rules.iter().any(|rule| rule.path == cache
            || rule.path == std::path::Path::new("/tmp")
            || rule.path == home));
    }
}

#[test]
fn desktop_entry_is_metadata_and_never_an_executable_command() {
    let fixture = tempfile::TempDir::new().unwrap();
    let marker = fixture.path().join("should-not-exist");
    let path = fixture.path().join("editor.desktop");
    let contents = format!(
        "[Desktop Entry]\nType=Application\nName=编辑器\nIcon=editor\nExec=touch {}\n[Desktop Action Delete]\nName=Other\n",
        marker.display()
    );
    let (item, icon) = platform::desktop_entry(&path, &contents).unwrap();
    assert_eq!(item.name, "编辑器");
    assert_eq!(icon.as_deref(), Some("editor"));
    assert!(item.is_application());
    assert!(!item.cleanable());
    assert!(!marker.exists());
    assert!(
        platform::desktop_entry(&path, &contents.replace("Type=Application", "Type=Link"))
            .is_none()
    );
    assert!(
        platform::desktop_entry(
            &path,
            &contents.replace("Name=编辑器", "Name=编辑器\nHidden=true")
        )
        .is_none()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn xdg_known_folders_expand_home_without_evaluating_shell() {
    let home = PathBuf::from("/home/example");
    assert_eq!(
        platform::xdg_user_directory("XDG_DOWNLOAD_DIR=\"$HOME/下载\"", "DOWNLOAD", &home),
        Some(home.join("下载"))
    );
    assert!(
        platform::xdg_user_directory("XDG_DOWNLOAD_DIR=\"$(touch bad)\"", "DOWNLOAD", &home)
            .is_none()
    );
    assert!(
        platform::xdg_user_directory("XDG_DOWNLOAD_DIR=\"$HOME/../other\"", "DOWNLOAD", &home)
            .is_none()
    );
}

#[test]
fn personal_scan_roots_are_specific_absolute_user_folders() {
    let home = platform::home_dir();
    let roots = platform::personal_roots();
    assert!(!roots.is_empty());
    assert!(roots.iter().all(|root| root.is_absolute() && root != &home));
    let mut unique = roots.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), roots.len());
}
