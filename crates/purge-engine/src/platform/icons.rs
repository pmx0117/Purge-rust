use std::collections::HashMap;

/// Straight-alpha BGRA pixels, independent of the desktop renderer.
pub struct ApplicationIcon {
    pub size: u32,
    pub bgra: Vec<u8>,
}

#[cfg(any(windows, test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IconSource {
    pub path: std::path::PathBuf,
    pub index: i32,
}

#[cfg(any(windows, test))]
impl IconSource {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        let (path, index) = if let Some(quoted) = value.strip_prefix('"') {
            let end = quoted.find('"')?;
            let suffix = quoted[end + 1..].trim();
            let index = if suffix.is_empty() {
                0
            } else {
                suffix.strip_prefix(',')?.trim().parse().ok()?
            };
            (&quoted[..end], index)
        } else {
            value
                .rsplit_once(',')
                .and_then(|(path, index)| index.trim().parse().ok().map(|index| (path, index)))
                .unwrap_or((value, 0))
        };
        let path = path.trim();
        if path.is_empty() || path.starts_with(r"\\") {
            return None;
        }
        Some(Self {
            path: path.into(),
            index,
        })
    }
}

#[cfg(windows)]
pub fn application_icons() -> HashMap<u64, ApplicationIcon> {
    super::applications::icon_sources()
        .into_iter()
        .filter_map(|(id, source)| read_icon(&source).map(|icon| (id, icon)))
        .collect()
}

#[cfg(not(windows))]
pub fn application_icons() -> HashMap<u64, ApplicationIcon> {
    HashMap::new()
}

#[cfg(windows)]
fn read_icon(source: &IconSource) -> Option<ApplicationIcon> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::UI::{Shell::SHDefExtractIconW, WindowsAndMessaging::*},
        core::PCWSTR,
    };
    const SIDE: u32 = 96;
    if !source.path.is_absolute() || !source.path.is_file() {
        return None;
    }
    let name: Vec<_> = source
        .path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let mut handle = HICON::default();
    unsafe {
        SHDefExtractIconW(
            PCWSTR(name.as_ptr()),
            source.index,
            0,
            Some(&mut handle),
            None,
            SIDE,
        )
        .ok()
        .ok()?;
    }
    if handle.is_invalid() {
        return None;
    }
    let icon = OwnedIcon(handle);
    let mut raster = IconRaster::new(SIDE)?;
    unsafe {
        DrawIconEx(
            raster.dc,
            0,
            0,
            icon.0,
            SIDE as i32,
            SIDE as i32,
            0,
            None,
            DI_NORMAL,
        )
        .ok()?;
    }
    let mut pixels = raster.pixels().to_vec();
    if !pixels.chunks_exact(4).any(|pixel| pixel[3] != 0) {
        raster.clear();
        unsafe {
            DrawIconEx(
                raster.dc,
                0,
                0,
                icon.0,
                SIDE as i32,
                SIDE as i32,
                0,
                None,
                DI_MASK,
            )
            .ok()?;
        }
        for (pixel, mask) in pixels
            .chunks_exact_mut(4)
            .zip(raster.pixels().chunks_exact(4))
        {
            pixel[3] = 255 - mask[0];
        }
    }
    unpremultiply(&mut pixels);
    Some(ApplicationIcon {
        size: SIDE,
        bgra: pixels,
    })
}

#[cfg(any(windows, test))]
fn unpremultiply(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 {
            pixel[..3].fill(0);
            continue;
        }
        for channel in &mut pixel[..3] {
            *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
        }
    }
}

#[cfg(windows)]
struct OwnedIcon(windows::Win32::UI::WindowsAndMessaging::HICON);
#[cfg(windows)]
impl Drop for OwnedIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyIcon(self.0);
        }
    }
}

#[cfg(windows)]
struct IconRaster {
    dc: windows::Win32::Graphics::Gdi::HDC,
    bitmap: windows::Win32::Graphics::Gdi::HBITMAP,
    previous: windows::Win32::Graphics::Gdi::HGDIOBJ,
    bits: *mut u8,
    length: usize,
}
#[cfg(windows)]
impl IconRaster {
    fn new(side: u32) -> Option<Self> {
        use windows::Win32::Graphics::Gdi::*;
        let dc = unsafe { CreateCompatibleDC(None) };
        if dc.is_invalid() {
            return None;
        }
        let header = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: side as i32,
                biHeight: -(side as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap =
            unsafe { CreateDIBSection(Some(dc), &header, DIB_RGB_COLORS, &mut bits, None, 0) };
        let Ok(bitmap) = bitmap else {
            unsafe {
                let _ = DeleteDC(dc);
            }
            return None;
        };
        let previous = unsafe { SelectObject(dc, HGDIOBJ(bitmap.0)) };
        let mut raster = Self {
            dc,
            bitmap,
            previous,
            bits: bits.cast(),
            length: side as usize * side as usize * 4,
        };
        if raster.bits.is_null() {
            return None;
        }
        raster.clear();
        Some(raster)
    }
    fn clear(&mut self) {
        unsafe {
            std::ptr::write_bytes(self.bits, 0, self.length);
        }
    }
    fn pixels(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.bits, self.length) }
    }
}
#[cfg(windows)]
impl Drop for IconRaster {
    fn drop(&mut self) {
        use windows::Win32::Graphics::Gdi::*;
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.dc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icon_locations_handle_quotes_commas_and_resource_indices() {
        let source = IconSource::parse(r#""C:\Program Files\Acme, Inc\app.exe", -12"#).unwrap();
        assert_eq!(
            source.path,
            std::path::PathBuf::from(r"C:\Program Files\Acme, Inc\app.exe")
        );
        assert_eq!(source.index, -12);
        assert_eq!(IconSource::parse(r"C:\Tools\app.ico").unwrap().index, 0);
        assert!(IconSource::parse("").is_none());
        assert!(IconSource::parse(r"\\server\icons\app.ico").is_none());
        assert!(IconSource::parse(r#""C:\Tools\app.exe" /run-command"#).is_none());
    }
    #[test]
    fn translucent_icon_edges_keep_their_original_color() {
        let mut pixels = [25, 50, 100, 128, 255, 255, 255, 0];
        unpremultiply(&mut pixels);
        assert_eq!(pixels, [50, 100, 199, 128, 0, 0, 0, 0]);
    }
    #[cfg(windows)]
    #[test]
    fn native_icon_is_read_without_a_window_or_disk_output() {
        let system = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap());
        let icon = read_icon(&IconSource {
            path: system.join("System32/shell32.dll"),
            index: 0,
        })
        .unwrap();
        assert_eq!(icon.bgra.len(), (icon.size * icon.size * 4) as usize);
        assert!(icon.bgra.chunks_exact(4).any(|pixel| pixel[3] != 0));
    }
}
