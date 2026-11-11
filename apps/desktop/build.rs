#[path = "src/brand.rs"]
mod brand;

use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree},
};
use std::{fs, path::PathBuf};

fn main() {
    let assets =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../assets");
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    for file in ["icons/purge-mark.svg", "windows.rc"] {
        println!("cargo:rerun-if-changed={}", assets.join(file).display());
    }
    println!("cargo:rerun-if-changed=src/brand.rs");
    let glyph = fs::read_to_string(assets.join("icons/purge-mark.svg"))
        .unwrap()
        .replace("black", "white");
    let inset = (brand::SIZE - brand::GLYPH_SIZE) / 2.;
    let source = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{side}" height="{side}" viewBox="0 0 {side} {side}"><rect width="{side}" height="{side}" rx="{radius}" fill="#{color:06x}"/><svg x="{inset}" y="{inset}" width="{glyph_size}" height="{glyph_size}" viewBox="0 0 14 14">{glyph}</svg></svg>"##,
        side = brand::SIZE,
        radius = brand::RADIUS,
        color = brand::COLOR,
        glyph_size = brand::GLYPH_SIZE,
    );
    let tree = Tree::from_str(&source, &Options::default()).unwrap();
    fs::write(output.join("purge.png"), rasterize(&tree, 256)).unwrap();
    let sizes = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256];
    let images: Vec<_> = sizes
        .iter()
        .map(|&size| {
            let png = rasterize(&tree, size);
            fs::write(output.join(format!("purge-{size}.png")), &png).unwrap();
            png
        })
        .collect();
    fs::write(output.join("purge.ico"), ico(&sizes, &images)).unwrap();
    fs::write(output.join("purge.icns"), icns(&tree)).unwrap();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let icon = output
            .join("purge.ico")
            .to_string_lossy()
            .replace('\\', "/");
        let resource = fs::read_to_string(assets.join("windows.rc"))
            .unwrap()
            .replace("\"purge.ico\"", &format!("\"{icon}\""));
        let resource_path = output.join("windows.rc");
        fs::write(&resource_path, resource).unwrap();
        embed_resource::compile(resource_path, embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
}

fn rasterize(tree: &Tree, size: u32) -> Vec<u8> {
    let mut bitmap = Pixmap::new(size, size).unwrap();
    resvg::render(
        tree,
        Transform::from_scale(size as f32 / brand::SIZE, size as f32 / brand::SIZE),
        &mut bitmap.as_mut(),
    );
    bitmap.encode_png().unwrap()
}

fn ico(sizes: &[u32], images: &[Vec<u8>]) -> Vec<u8> {
    let mut data = vec![0, 0, 1, 0];
    data.extend_from_slice(&(images.len() as u16).to_le_bytes());
    let mut offset = 6 + images.len() * 16;
    for (&size, image) in sizes.iter().zip(images) {
        data.extend_from_slice(&[(size % 256) as u8, (size % 256) as u8, 0, 0]);
        data.extend_from_slice(&1_u16.to_le_bytes());
        data.extend_from_slice(&32_u16.to_le_bytes());
        data.extend_from_slice(&(image.len() as u32).to_le_bytes());
        data.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += image.len();
    }
    for image in images {
        data.extend_from_slice(image);
    }
    data
}

fn icns(tree: &Tree) -> Vec<u8> {
    let mut images = Vec::new();
    for (kind, side) in [
        (b"icp4", 16),
        (b"icp5", 32),
        (b"icp6", 64),
        (b"ic07", 128),
        (b"ic08", 256),
        (b"ic09", 512),
        (b"ic10", 1024),
    ] {
        let png = rasterize(tree, side);
        images.extend_from_slice(kind);
        images.extend_from_slice(&((png.len() + 8) as u32).to_be_bytes());
        images.extend_from_slice(&png);
    }
    let mut result = b"icns".to_vec();
    result.extend_from_slice(&((images.len() + 8) as u32).to_be_bytes());
    result.extend(images);
    result
}
