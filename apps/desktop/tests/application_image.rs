use purge::application_image::icns_png;

#[test]
fn icns_selects_png_and_rejects_truncated_or_looping_entries() {
    let png = b"\x89PNG\r\n\x1a\nfixture";
    let mut file = b"icns".to_vec();
    file.extend_from_slice(&((png.len() + 16) as u32).to_be_bytes());
    file.extend_from_slice(b"ic08");
    file.extend_from_slice(&((png.len() + 8) as u32).to_be_bytes());
    file.extend_from_slice(png);
    assert_eq!(icns_png(&file), Some(png.as_slice()));
    assert!(icns_png(&file[..file.len() - 1]).is_none());
    let mut invalid = file.clone();
    invalid[12..16].copy_from_slice(&0_u32.to_be_bytes());
    assert!(icns_png(&invalid).is_none());
    assert!(icns_png(b"not an icon").is_none());
}
