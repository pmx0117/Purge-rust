use std::path::{Path, PathBuf};
pub fn home_dir() -> PathBuf {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}
pub fn local_dir() -> PathBuf {
    user_directory(UserDirectory::Cache)
}
pub fn roaming_dir() -> PathBuf {
    user_directory(UserDirectory::Config)
}
pub fn user_data_dir() -> PathBuf {
    user_directory(UserDirectory::Data)
}

#[derive(Clone, Copy)]
enum UserDirectory {
    Cache,
    Config,
    Data,
}

fn user_directory(kind: UserDirectory) -> PathBuf {
    let (windows, linux, macos) = match kind {
        UserDirectory::Cache => (
            ("LOCALAPPDATA", "AppData/Local"),
            ("XDG_CACHE_HOME", ".cache"),
            "Library/Caches",
        ),
        UserDirectory::Config => (
            ("APPDATA", "AppData/Roaming"),
            ("XDG_CONFIG_HOME", ".config"),
            "Library/Application Support",
        ),
        UserDirectory::Data => (
            ("LOCALAPPDATA", "AppData/Local"),
            ("XDG_DATA_HOME", ".local/share"),
            "Library/Application Support",
        ),
    };
    match super::OperatingSystem::current() {
        super::OperatingSystem::Windows => application_data(windows.0, windows.1),
        super::OperatingSystem::Linux => application_data(linux.0, linux.1),
        super::OperatingSystem::MacOs => home_dir().join(macos),
    }
}
fn application_data(variable: &str, fallback: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home_dir().join(fallback))
}

pub use purge_core::path_within as within;
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
        let names = [
            "Downloads",
            "Desktop",
            "Documents",
            if cfg!(target_os = "macos") {
                "Movies"
            } else {
                "Videos"
            },
            "Music",
            "Pictures",
        ];
        let home = home_dir();
        let mut roots: Vec<_> = names.into_iter().map(|name| home.join(name)).collect();
        #[cfg(target_os = "linux")]
        if let Ok(settings) = std::fs::read_to_string(roaming_dir().join("user-dirs.dirs")) {
            for (index, key) in [
                "DOWNLOAD",
                "DESKTOP",
                "DOCUMENTS",
                "VIDEOS",
                "MUSIC",
                "PICTURES",
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(path) = xdg_user_directory(&settings, key, &home) {
                    roots[index] = path;
                }
            }
        }
        roots.retain(|path| path != &home);
        roots.sort();
        roots.dedup();
        roots
    }
}

#[cfg(any(target_os = "linux", test))]
pub fn xdg_user_directory(settings: &str, key: &str, home: &Path) -> Option<PathBuf> {
    let prefix = format!("XDG_{key}_DIR=");
    let value = settings
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(&prefix))?;
    let value = value.trim().strip_prefix('"')?.strip_suffix('"')?;
    let path = if value == "$HOME" {
        home.to_owned()
    } else if let Some(child) = value.strip_prefix("$HOME/") {
        home.join(child)
    } else {
        PathBuf::from(value)
    };
    (path.is_absolute()
        && !path
            .components()
            .any(|part| part == std::path::Component::ParentDir))
    .then_some(path)
}
