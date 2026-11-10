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
    #[cfg(not(windows))]
    {
        Command::new("xdg-open")
            .arg(path.parent().unwrap_or(path))
            .spawn()?;
    }
    Ok(())
}

pub fn open_folder(path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        Command::new("explorer.exe").arg(path).spawn()?;
    }
    #[cfg(not(windows))]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

pub fn open_apps_settings() -> Result<()> {
    open_uri("ms-settings:appsfeatures")
}
pub fn open_storage_settings() -> Result<()> {
    open_uri("ms-settings:storagesense")
}
pub fn open_recycle_bin() -> Result<()> {
    #[cfg(windows)]
    {
        Command::new("explorer.exe")
            .arg("shell:RecycleBinFolder")
            .spawn()?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        bail!("仅在 Windows 上可用")
    }
}

/// Launch through Explorer so ShellExecute cannot pump messages while GPUI
/// holds its application borrow (a reentrant COM callback can otherwise panic).
pub fn open_uri(uri: &str) -> Result<()> {
    #[cfg(windows)]
    {
        Command::new("explorer.exe").arg(uri).spawn()?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Command::new("xdg-open").arg(uri).spawn()?;
        Ok(())
    }
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
