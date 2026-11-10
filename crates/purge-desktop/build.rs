fn main() {
    let assets =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../assets");
    println!(
        "cargo:rerun-if-changed={}",
        assets.join("purge.ico").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        assets.join("windows.rc").display()
    );
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile(assets.join("windows.rc"), embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
}
