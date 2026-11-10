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
    let mut seen = HashSet::new();
    Ok(uninstall_keys()
        .into_iter()
        .flat_map(registrations)
        .filter(|entry| seen.insert(entry.identity.clone()))
        .map(|entry| entry.item)
        .collect())
}
#[cfg(windows)]
struct Registration {
    identity: (String, String),
    item: Item,
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
        path,
        safety: Safety::Review,
        description: format!("{publisher}  {version} · 使用 Windows 官方卸载入口"),
        kind: ItemKind::InstalledApplication,
        footprint: Footprint {
            bytes: u64::from(app.get_value::<u32, _>("EstimatedSize").unwrap_or(0)) * 1024,
            ..Default::default()
        },
        project_guard: None,
        modified: None,
        duplicate_group: None,
    };
    Registration {
        identity: (display, version),
        item,
    }
}
#[cfg(not(windows))]
pub fn installed_apps() -> Result<Vec<Item>> {
    Ok(vec![])
}
