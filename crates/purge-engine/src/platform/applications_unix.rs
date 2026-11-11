use purge_core::{Footprint, Item, ItemKind, Safety, path_id};
#[cfg(target_os = "linux")]
use std::fs;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(super) struct Registration {
    pub item: Item,
    pub icon: Option<PathBuf>,
}

pub(super) fn registrations() -> Vec<Registration> {
    #[cfg(target_os = "macos")]
    {
        mac_registrations(&[
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
            super::home_dir().join("Applications"),
        ])
    }
    #[cfg(not(target_os = "macos"))]
    {
        let mut roots = vec![super::user_data_dir()];
        roots.extend(
            std::env::var_os("XDG_DATA_DIRS")
                .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
                .unwrap_or_else(|| vec!["/usr/local/share".into(), "/usr/share".into()]),
        );
        desktop_registrations(&roots)
    }
}

/// Parse data only: an Exec field is never interpreted or run.
pub fn desktop_entry(path: &Path, contents: &str) -> Option<(Item, Option<String>)> {
    let values = desktop_values(contents);
    if values.get("Type").copied() != Some("Application")
        || ["Hidden", "NoDisplay"].into_iter().any(|key| {
            values
                .get(key)
                .is_some_and(|value| value.eq_ignore_ascii_case("true"))
        })
    {
        return None;
    }
    let name = values.get("Name")?.trim();
    if name.is_empty() {
        return None;
    }
    Some((
        Item {
            id: path_id(path),
            name: name.into(),
            path: path.into(),
            safety: Safety::Review,
            description: values.get("Comment").copied().unwrap_or("桌面应用").into(),
            kind: ItemKind::InstalledApplication,
            footprint: Footprint::default(),
            project_guard: None,
            modified: None,
            duplicate_group: None,
        },
        values.get("Icon").map(|value| value.to_string()),
    ))
}

#[cfg(target_os = "linux")]
pub(super) fn desktop_registrations(data_roots: &[PathBuf]) -> Vec<Registration> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = vec![];
    for root in data_roots.iter().filter(|root| root.is_absolute()) {
        let Ok(entries) = fs::read_dir(root.join("applications")) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|value| value != "desktop")
                || !seen.insert(entry.file_name())
            {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Some((item, icon)) = desktop_entry(&path, &text) else {
                continue;
            };
            let icon = icon.as_deref().and_then(|name| find_icon(name, data_roots));
            result.push(Registration { item, icon });
        }
    }
    result
}

#[cfg(target_os = "linux")]
fn find_icon(name: &str, data_roots: &[PathBuf]) -> Option<PathBuf> {
    let path = PathBuf::from(name);
    if path.is_absolute() {
        return path.is_file().then_some(path);
    }
    if name.contains('/') || name.contains('\\') || name.is_empty() {
        return None;
    }
    for root in data_roots {
        for directory in [
            "icons/hicolor/128x128/apps",
            "icons/hicolor/96x96/apps",
            "icons/hicolor/64x64/apps",
            "icons/hicolor/48x48/apps",
            "icons/hicolor/scalable/apps",
            "pixmaps",
        ] {
            for suffix in ["", ".png", ".svg"] {
                let candidate = root.join(directory).join(format!("{name}{suffix}"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn mac_registrations(roots: &[PathBuf]) -> Vec<Registration> {
    let mut result = vec![];
    for root in roots {
        let walker = walkdir::WalkDir::new(root)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0
                    || entry
                        .path()
                        .parent()
                        .is_none_or(|parent| parent.extension().is_none_or(|ext| ext != "app"))
            });
        for entry in walker.flatten().filter(|entry| {
            entry.file_type().is_dir() && entry.path().extension().is_some_and(|ext| ext == "app")
        }) {
            let path = entry.path();
            let plist = path.join("Contents/Info.plist");
            let output = std::process::Command::new("/usr/bin/plutil")
                .args(["-convert", "json", "-o", "-"])
                .arg(&plist)
                .output();
            let metadata: serde_json::Value = output
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| serde_json::from_slice(&output.stdout).ok())
                .unwrap_or_default();
            let name = metadata
                .get("CFBundleDisplayName")
                .or_else(|| metadata.get("CFBundleName"))
                .and_then(|value| value.as_str())
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    path.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned()
                });
            let version = metadata
                .get("CFBundleShortVersionString")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let icon = metadata
                .get("CFBundleIconFile")
                .and_then(|value| value.as_str())
                .map(|name| {
                    let mut icon = path.join("Contents/Resources").join(name);
                    if icon.extension().is_none() {
                        icon.set_extension("icns");
                    }
                    icon
                })
                .filter(|icon| icon.is_file());
            let footprint =
                crate::scanner::measure(path, &std::sync::atomic::AtomicBool::new(false));
            result.push(Registration {
                item: Item {
                    id: path_id(path),
                    name,
                    path: path.into(),
                    description: version.into(),
                    kind: ItemKind::InstalledApplication,
                    safety: Safety::Review,
                    footprint,
                    project_guard: None,
                    modified: None,
                    duplicate_group: None,
                },
                icon,
            });
        }
    }
    result
}

fn desktop_values(contents: &str) -> BTreeMap<&str, &str> {
    let mut values = BTreeMap::new();
    let mut in_entry = false;
    for line in contents.lines().map(str::trim) {
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            values.insert(key.trim(), value.trim());
        }
    }
    values
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    fn local_desktop_entries_override_system_entries_and_resolve_user_icons() {
        let fixture = tempfile::TempDir::new().unwrap();
        let user = fixture.path().join("user");
        let system = fixture.path().join("system");
        fs::create_dir_all(user.join("applications")).unwrap();
        fs::create_dir_all(system.join("applications")).unwrap();
        fs::create_dir_all(user.join("icons/hicolor/64x64/apps")).unwrap();
        let entry = "[Desktop Entry]\nType=Application\nName=Editor\nIcon=editor\n";
        fs::write(user.join("applications/editor.desktop"), entry).unwrap();
        fs::write(
            system.join("applications/editor.desktop"),
            entry.replace("Editor", "System Editor"),
        )
        .unwrap();
        let icon = user.join("icons/hicolor/64x64/apps/editor.png");
        fs::write(&icon, b"fixture-image-path").unwrap();
        fs::write(
            user.join("applications/hidden.desktop"),
            "[Desktop Entry]\nHidden=true",
        )
        .unwrap();
        fs::write(system.join("applications/hidden.desktop"), entry).unwrap();
        let entries = desktop_registrations(&[user.clone(), system.clone()]);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].item.name, "Editor");
        assert_eq!(entries[0].icon.as_ref(), Some(&icon));
        assert_eq!(
            find_icon(icon.to_str().unwrap(), std::slice::from_ref(&user)),
            Some(icon)
        );
        assert!(find_icon("../elsewhere", std::slice::from_ref(&user)).is_none());
        assert!(find_icon("missing-icon", &[user]).is_none());
    }
}
