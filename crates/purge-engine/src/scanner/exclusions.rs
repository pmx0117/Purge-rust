use crate::{platform, storage};
use purge_core::ScanOptions;
use std::path::{Path, PathBuf};
pub fn excluded(path: &Path, settings: &ScanOptions) -> bool {
    matches_exclusion(path, settings, false)
}
pub fn contains_exclusion(path: &Path, settings: &ScanOptions) -> bool {
    matches_exclusion(path, settings, true)
}
fn protected_roots(settings: &ScanOptions) -> impl Iterator<Item = PathBuf> + '_ {
    settings
        .exclusions
        .iter()
        .cloned()
        .chain(std::iter::once(storage::data_dir()))
}
fn matches_exclusion(path: &Path, settings: &ScanOptions, include_children: bool) -> bool {
    protected_roots(settings).any(|root| {
        platform::within(path, &root) || (include_children && platform::within(&root, path))
    })
}
