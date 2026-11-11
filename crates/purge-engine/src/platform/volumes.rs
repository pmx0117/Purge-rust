#[cfg(windows)]
use super::paths::home_dir;
use anyhow::Result;
use purge_core::DiskInfo;
#[cfg(windows)]
pub fn disk_info() -> Result<DiskInfo> {
    let root = volume_root()?;
    volume_space(&root)
}
#[cfg(windows)]
fn volume_root() -> Result<Vec<u16>> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{Win32::Storage::FileSystem::GetVolumePathNameW, core::PCWSTR};
    let home: Vec<u16> = home_dir()
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let mut root = vec![0_u16; 260];
    unsafe {
        GetVolumePathNameW(PCWSTR(home.as_ptr()), &mut root)?;
    }
    Ok(root)
}
#[cfg(windows)]
fn volume_space(root: &[u16]) -> Result<DiskInfo> {
    use windows::{Win32::Storage::FileSystem::GetDiskFreeSpaceExW, core::PCWSTR};
    let (mut available, mut total, mut free) = (0, 0, 0);
    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(root.as_ptr()),
            Some(&mut available),
            Some(&mut total),
            Some(&mut free),
        )?;
    }
    let end = root
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(root.len());
    Ok(DiskInfo {
        name: String::from_utf16_lossy(&root[..end]),
        total,
        free,
        volume_id: None,
    })
}
#[cfg(not(windows))]
pub fn disk_info() -> Result<DiskInfo> {
    use std::{
        ffi::CString,
        os::unix::{ffi::OsStrExt, fs::MetadataExt},
    };
    let home = super::home_dir().canonicalize()?;
    let device = std::fs::metadata(&home)?.dev();
    let mut root = home.clone();
    while let Some(parent) = root.parent() {
        if std::fs::metadata(parent)?.dev() != device {
            break;
        }
        root = parent.to_owned();
    }
    let name = CString::new(home.as_os_str().as_bytes())?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    if unsafe { libc::statvfs(name.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let stats = unsafe { stats.assume_init() };
    #[cfg(target_os = "macos")]
    let (blocks, available, size) = (
        u64::from(stats.f_blocks),
        u64::from(stats.f_bavail),
        u64::from(stats.f_frsize),
    );
    #[cfg(not(target_os = "macos"))]
    let (blocks, available, size) = (stats.f_blocks, stats.f_bavail, stats.f_frsize);
    Ok(DiskInfo {
        name: root.to_string_lossy().into_owned(),
        total: blocks.saturating_mul(size),
        free: available.saturating_mul(size),
        volume_id: Some(device),
    })
}

#[cfg(windows)]
pub fn trash_bytes() -> Result<Option<u64>> {
    use windows::{
        Win32::UI::Shell::{SHQUERYRBINFO, SHQueryRecycleBinW},
        core::PCWSTR,
    };
    let mut info = SHQUERYRBINFO {
        cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
        ..Default::default()
    };
    unsafe {
        SHQueryRecycleBinW(PCWSTR::null(), &mut info)?;
    }
    Ok(Some(info.i64Size.max(0) as u64))
}
#[cfg(not(windows))]
pub fn trash_bytes() -> Result<Option<u64>> {
    let root = if cfg!(target_os = "macos") {
        super::home_dir().join(".Trash")
    } else {
        super::user_data_dir().join("Trash/files")
    };
    if !root.exists() {
        return Ok(Some(0));
    }
    let state = crate::scanner::measure(&root, &std::sync::atomic::AtomicBool::new(false));
    Ok(state.is_complete().then_some(state.bytes))
}
