#[cfg(windows)]
use super::icons::IconSource;
use anyhow::Result;
use purge_core::Item;
#[cfg(windows)]
use {
    purge_core::{Footprint, ItemKind, Safety, path_id},
    std::{
        collections::HashSet,
        path::{Path, PathBuf},
    },
    winreg::{RegKey, enums::*},
};

#[cfg(windows)]
pub fn installed_apps() -> Result<Vec<Item>> {
    Ok(all_registrations().map(|entry| entry.item).collect())
}
#[cfg(windows)]
fn all_registrations() -> impl Iterator<Item = Registration> {
    let mut seen = HashSet::new();
    uninstall_keys()
        .into_iter()
        .flat_map(registrations)
        .filter(move |entry| seen.insert(entry.identity.clone()))
}
#[cfg(windows)]
struct Registration {
    identity: (String, String),
    item: Item,
    icon: Option<IconSource>,
}
#[cfg(windows)]
fn uninstall_keys() -> Vec<RegKey> {
    [
        (HKEY_CURRENT_USER, KEY_READ | KEY_WOW64_64KEY),
        (HKEY_CURRENT_USER, KEY_READ | KEY_WOW64_32KEY),
        (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_32KEY),
    ]
    .into_iter()
    .filter_map(|(hive, flags)| {
        RegKey::predef(hive)
            .open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
                flags,
            )
            .ok()
    })
    .collect()
}
#[cfg(windows)]
fn registrations(key: RegKey) -> Vec<Registration> {
    key.enum_keys()
        .filter_map(std::result::Result::ok)
        .filter_map(|name| key.open_subkey_with_flags(&name, KEY_READ).ok())
        .filter_map(registration)
        .collect()
}
#[cfg(windows)]
fn registration(app: RegKey) -> Option<Registration> {
    let display: String = app
        .get_value::<String, _>("DisplayName")
        .ok()
        .filter(|name| !name.trim().is_empty())?;
    visible_registration(&app).then(|| registration_details(&app, display))
}
#[cfg(windows)]
fn visible_registration(app: &RegKey) -> bool {
    app.get_value::<u32, _>("SystemComponent").unwrap_or(0) != 1
}
#[cfg(windows)]
fn registration_details(app: &RegKey, display: String) -> Registration {
    let version: String = app.get_value("DisplayVersion").unwrap_or_default();
    let publisher: String = app.get_value("Publisher").unwrap_or_default();
    let location: String = app.get_value("InstallLocation").unwrap_or_default();
    let path = PathBuf::from(location.trim_matches('"'));
    let item = Item {
        id: path_id(Path::new(&format!("app:{display}:{version}"))),
        name: display.clone(),
        path: path.clone(),
        safety: Safety::Review,
        description: [publisher.as_str(), version.as_str()]
            .into_iter()
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" · "),
        kind: ItemKind::InstalledApplication,
        footprint: Footprint {
            bytes: u64::from(app.get_value::<u32, _>("EstimatedSize").unwrap_or(0)) * 1024,
            ..Default::default()
        },
        project_guard: None,
        modified: None,
        duplicate_group: None,
        duplicate_copies: Default::default(),
    };
    let icon = app
        .get_value::<String, _>("DisplayIcon")
        .ok()
        .and_then(|value| IconSource::parse(&value))
        .or_else(|| install_directory_icon(&path, &display));
    Registration {
        identity: (display, version),
        item,
        icon,
    }
}
#[cfg(not(windows))]
pub fn installed_apps() -> Result<Vec<Item>> {
    Ok(super::applications_unix::registrations()
        .into_iter()
        .map(|entry| entry.item)
        .collect())
}

#[cfg(windows)]
pub(super) fn icon_sources() -> Vec<(u64, IconSource)> {
    all_registrations()
        .filter_map(|entry| entry.icon.map(|icon| (entry.item.id, icon)))
        .collect()
}

#[cfg(windows)]
fn install_directory_icon(path: &Path, display: &str) -> Option<IconSource> {
    if !path.is_absolute() || path.to_string_lossy().starts_with(r"\\") {
        return None;
    }
    let display = display.to_lowercase();
    let mut candidates: Vec<_> = std::fs::read_dir(path)
        .ok()?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
        })
        .filter(|path| {
            !path.file_stem().is_some_and(|name| {
                let name = name.to_string_lossy().to_lowercase();
                ["unins", "setup", "update"]
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
            })
        })
        .collect();
    candidates.sort_by_key(|path| {
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        (!display.contains(&stem), path.clone())
    });
    candidates
        .into_iter()
        .next()
        .map(|path| IconSource { path, index: 0 })
}

#[cfg(unix)]
pub fn application_icon_paths() -> std::collections::HashMap<u64, std::path::PathBuf> {
    super::applications_unix::registrations()
        .into_iter()
        .filter_map(|entry| entry.icon.map(|icon| (entry.item.id, icon)))
        .collect()
}
#[cfg(windows)]
pub fn application_icon_paths() -> std::collections::HashMap<u64, std::path::PathBuf> {
    Default::default()
}
