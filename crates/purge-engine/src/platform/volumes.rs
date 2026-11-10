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
    })
}
#[cfg(not(windows))]
pub fn disk_info() -> Result<DiskInfo> {
    Ok(DiskInfo::default())
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
    Ok(None)
}
