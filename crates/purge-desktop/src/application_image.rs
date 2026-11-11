/// Extract an embedded PNG from ICNS without executing app code or writing files.
pub fn icns_png(data: &[u8]) -> Option<&[u8]> {
    let data = container(data)?;
    let (mut offset, mut selected) = (8_usize, None);
    while offset < data.len() {
        let (end, image) = entry(data, offset)?;
        if image.starts_with(b"\x89PNG\r\n\x1a\n") {
            selected = Some(image);
        }
        offset = end;
    }
    selected
}

fn container(data: &[u8]) -> Option<&[u8]> {
    if data.get(..4)? != b"icns" {
        return None;
    }
    let total = length(data.get(4..8)?)?;
    (total >= 8).then_some(())?;
    data.get(..total)
}

fn entry(data: &[u8], offset: usize) -> Option<(usize, &[u8])> {
    let header = data.get(offset..offset.checked_add(8)?)?;
    let size = length(&header[4..8])?;
    if size < 8 {
        return None;
    }
    let end = offset.checked_add(size)?;
    Some((end, data.get(offset + 8..end)?))
}

fn length(bytes: &[u8]) -> Option<usize> {
    Some(u32::from_be_bytes(bytes.try_into().ok()?) as usize)
}
