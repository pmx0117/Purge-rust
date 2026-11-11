use anyhow::{Result, bail};
pub use purge_engine::platform::{disk_info, display_path, within};
use std::{path::Path, process::Command};
pub fn reveal(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("路径已不存在：{}", path.display());
    }
    #[cfg(windows)]
    {
        Command::new("explorer.exe")
            .arg("/select,")
            .arg(display_path(path))
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").args(["-R", "--"]).arg(path).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path.parent().unwrap_or(path))
            .spawn()?;
    }
    Ok(())
}

pub fn open_folder(path: &Path) -> Result<()> {
    open_target(path.as_os_str())
}

fn open_target(target: &std::ffi::OsStr) -> Result<()> {
    let command = if cfg!(windows) {
        "explorer.exe"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(command).arg(target).spawn()?;
    Ok(())
}

pub fn open_apps_settings() -> Result<()> {
    #[cfg(windows)]
    {
        open_uri("ms-settings:appsfeatures")
    }
    #[cfg(target_os = "macos")]
    {
        open_folder(Path::new("/Applications"))
    }
    #[cfg(target_os = "linux")]
    {
        open_uri("appstream://")
    }
}
pub fn open_storage_settings() -> Result<()> {
    #[cfg(windows)]
    {
        open_uri("ms-settings:storagesense")
    }
    #[cfg(target_os = "macos")]
    {
        open_uri("x-apple.systempreferences:com.apple.settings.Storage")
    }
    #[cfg(target_os = "linux")]
    {
        open_folder(&purge_engine::platform::home_dir())
    }
}
pub fn open_recycle_bin() -> Result<()> {
    #[cfg(windows)]
    {
        Command::new("explorer.exe")
            .arg("shell:RecycleBinFolder")
            .spawn()?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        open_folder(&purge_engine::platform::home_dir().join(".Trash"))
    }
    #[cfg(target_os = "linux")]
    {
        open_uri("trash:///")
    }
}

/// Launch through Explorer so ShellExecute cannot pump messages while GPUI
/// holds its application borrow (a reentrant COM callback can otherwise panic).
pub fn open_uri(uri: &str) -> Result<()> {
    open_target(std::ffi::OsStr::new(uri))
}

#[cfg(windows)]
pub fn apply_caption_theme(handle: isize, dark: bool) {
    use windows::Win32::{
        Foundation::HWND,
        Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute},
    };
    let value = i32::from(dark);
    // Unsupported Windows versions retain their normal caption styling.
    let _ = unsafe {
        DwmSetWindowAttribute(
            HWND(handle as *mut _),
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&value as *const i32).cast(),
            std::mem::size_of_val(&value) as u32,
        )
    };
}

#[cfg(windows)]
pub fn apply_window_icons(handle: isize) -> Result<()> {
    use windows::{
        Win32::{
            Foundation::{HWND, LPARAM, WPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::{
                HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
                WindowsAndMessaging::*,
            },
        },
        core::PCWSTR,
    };
    let hwnd = HWND(handle as *mut _);
    let module = unsafe { GetModuleHandleW(None)? };
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    for (kind, metric) in [(ICON_SMALL, SM_CXSMICON), (ICON_BIG, SM_CXICON)] {
        let side = unsafe { GetSystemMetricsForDpi(metric, dpi) };
        // LR_SHARED leaves the resource handle with Windows; repeated DPI
        // changes neither leak handles nor destroy the window class's icon.
        let icon = unsafe {
            LoadImageW(
                Some(module.into()),
                PCWSTR(1 as _),
                IMAGE_ICON,
                side,
                side,
                LR_SHARED,
            )?
        };
        unsafe {
            SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(kind as usize)),
                Some(LPARAM(icon.0 as isize)),
            );
        }
    }
    Ok(())
}

pub fn is_minimized(window: &gpui::Window) -> bool {
    #[cfg(any(windows, target_os = "macos"))]
    if let Ok(handle) = raw_window_handle::HasWindowHandle::window_handle(window) {
        match handle.as_raw() {
            #[cfg(windows)]
            raw_window_handle::RawWindowHandle::Win32(handle) => {
                return unsafe {
                    windows::Win32::UI::WindowsAndMessaging::IsIconic(
                        windows::Win32::Foundation::HWND(handle.hwnd.get() as *mut _),
                    )
                    .as_bool()
                };
            }
            #[cfg(target_os = "macos")]
            raw_window_handle::RawWindowHandle::AppKit(handle) => {
                return unsafe { crate::macos_window::is_minimized(handle.ns_view) };
            }
            _ => {}
        }
    }
    let _ = window;
    false
}
