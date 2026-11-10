use crate::platform;
use purge_core::Footprint;
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

pub fn measure(path: &Path, cancel: &AtomicBool) -> Footprint {
    Measurement {
        result: Footprint::default(),
        signatures: vec![],
        stack: vec![path.to_path_buf()],
        visited: 0,
        cancel,
    }
    .run()
}
struct Measurement<'a> {
    result: Footprint,
    signatures: Vec<u64>,
    stack: Vec<PathBuf>,
    visited: u64,
    cancel: &'a AtomicBool,
}
impl Measurement<'_> {
    fn run(mut self) -> Footprint {
        while let Some(path) = self.stack.pop() {
            self.visited += 1;
            if self.stopped() {
                self.result.incomplete = true;
                break;
            }
            self.visit(&path);
        }
        self.signatures.sort_unstable();
        let mut hash = DefaultHasher::new();
        self.signatures.hash(&mut hash);
        self.result.fingerprint = hash.finish();
        self.result
    }
    fn stopped(&self) -> bool {
        self.cancel.load(Ordering::Relaxed) || self.visited > 500_000
    }
    fn visit(&mut self, path: &Path) {
        match fs::symlink_metadata(path) {
            Ok(metadata) => self.accept(path, &metadata),
            Err(_) => self.result.incomplete = true,
        }
    }
    fn accept(&mut self, path: &Path, metadata: &fs::Metadata) {
        if platform::is_link(metadata) || platform::is_cloud_placeholder(metadata) {
            self.result.has_links = true;
            return;
        }
        let modified = metadata.modified().ok();
        self.result.newest = self.result.newest.max(modified);
        let mut hash = DefaultHasher::new();
        path.hash(&mut hash);
        metadata.len().hash(&mut hash);
        modified.hash(&mut hash);
        self.signatures.push(hash.finish());
        self.accept_contents(path, metadata);
    }
    fn accept_contents(&mut self, path: &Path, metadata: &fs::Metadata) {
        if metadata.is_file() {
            self.result.bytes = self.result.bytes.saturating_add(metadata.len());
            self.result.files += 1;
        }
        if metadata.is_dir() {
            self.read_children(path);
        }
    }
    fn read_children(&mut self, path: &Path) {
        match fs::read_dir(path) {
            Ok(children) => {
                for child in children {
                    match child {
                        Ok(child) => self.stack.push(child.path()),
                        Err(_) => self.result.incomplete = true,
                    }
                }
            }
            Err(_) => self.result.incomplete = true,
        }
    }
}
pub fn has_link_component(path: &Path) -> bool {
    path.ancestors().any(|parent| {
        fs::symlink_metadata(parent)
            .map(|metadata| platform::is_link(&metadata))
            .unwrap_or(true)
    })
}
