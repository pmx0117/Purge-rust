use std::path::{Path, PathBuf};
pub fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}
pub fn local_dir() -> PathBuf {
    application_data("LOCALAPPDATA", "AppData/Local")
}
pub fn roaming_dir() -> PathBuf {
    application_data("APPDATA", "AppData/Roaming")
}
fn application_data(variable: &str, fallback: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(fallback))
}

/// Compare complete path components, including Windows case insensitivity.
pub fn within(path: &Path, root: &Path) -> bool {
    #[cfg(windows)]
    {
        let clean = |p: &Path| {
            p.components()
                .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
                .collect::<Vec<_>>()
        };
        clean(path).starts_with(&clean(root))
    }
    #[cfg(not(windows))]
    {
        path.starts_with(root)
    }
}
pub fn same_path(a: &Path, b: &Path) -> bool {
    within(a, b) && within(b, a)
}

pub fn display_path(path: &Path) -> String {
    #[cfg(windows)]
    {
        path.to_string_lossy().replace('/', "\\")
    }
    #[cfg(not(windows))]
    {
        path.display().to_string()
    }
}

pub fn personal_roots() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        use windows::Win32::{System::Com::CoTaskMemFree, UI::Shell::*};
        let mut roots = vec![];
        for id in [
            FOLDERID_Downloads,
            FOLDERID_Desktop,
            FOLDERID_Documents,
            FOLDERID_Videos,
            FOLDERID_Music,
            FOLDERID_Pictures,
        ] {
            unsafe {
                if let Ok(path) = SHGetKnownFolderPath(&id, KF_FLAG_DEFAULT, None) {
                    if let Ok(text) = path.to_string() {
                        roots.push(PathBuf::from(text));
                    }
                    CoTaskMemFree(Some(path.0.cast()));
                }
            }
        }
        roots.sort();
        roots.dedup();
        roots
    }
    #[cfg(not(windows))]
    {
        [
            "Downloads",
            "Desktop",
            "Documents",
            "Videos",
            "Music",
            "Pictures",
        ]
        .into_iter()
        .map(|s| home_dir().join(s))
        .collect()
    }
}
