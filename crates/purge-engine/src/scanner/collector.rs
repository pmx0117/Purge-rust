use super::{ScanEvent, contains_exclusion, has_link_component, measure};
use purge_core::{Footprint, Item, ItemKind, Safety, ScanOptions, path_id};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
};
#[derive(Default)]
struct SeenPaths(BTreeSet<PathBuf>);
impl SeenPaths {
    fn overlaps(&self, path: &Path) -> bool {
        path.ancestors().any(|parent| self.0.contains(parent))
            || self
                .0
                .range(path.to_path_buf()..)
                .next()
                .is_some_and(|child| child.starts_with(path))
    }
}
pub(super) struct Candidate {
    pub path: PathBuf,
    pub name: String,
    pub safety: Safety,
    pub description: String,
    pub kind: ItemKind,
    pub measured: Option<Footprint>,
}
pub(super) struct ScanCollector<'a> {
    pub settings: &'a ScanOptions,
    pub cancel: &'a AtomicBool,
    pub sender: &'a Sender<ScanEvent>,
    seen: SeenPaths,
}
impl<'a> ScanCollector<'a> {
    pub fn new(
        settings: &'a ScanOptions,
        cancel: &'a AtomicBool,
        sender: &'a Sender<ScanEvent>,
    ) -> Self {
        Self {
            settings,
            cancel,
            sender,
            seen: SeenPaths::default(),
        }
    }
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
    pub fn emit(&self, event: ScanEvent) {
        let _ = self.sender.send(event);
    }
    pub fn progress(&self, message: String) {
        self.emit(ScanEvent::Progress(message));
    }
    pub fn add(&mut self, candidate: Candidate) {
        if self.skip(&candidate.path) {
            return;
        }
        let Some(item) = candidate.into_item(self.cancel) else {
            return;
        };
        self.seen.0.insert(item.path.clone());
        self.emit(ScanEvent::Item(Box::new(item)));
    }
    fn skip(&self, path: &Path) -> bool {
        contains_exclusion(path, self.settings)
            || !path.exists()
            || has_link_component(path)
            || self.seen.overlaps(path)
    }
}
impl Candidate {
    fn into_item(self, cancel: &AtomicBool) -> Option<Item> {
        let footprint = self.measured.unwrap_or_else(|| measure(&self.path, cancel));
        if !usable(&footprint) {
            return None;
        }
        let project_guard = project_proof(&self.path, self.kind, cancel).ok()?;
        let modified = fs::metadata(&self.path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        Some(Item {
            id: path_id(&self.path),
            name: self.name,
            path: self.path,
            safety: self.safety,
            description: self.description,
            kind: self.kind,
            footprint,
            project_guard,
            modified,
            duplicate_group: None,
            duplicate_copies: Default::default(),
        })
    }
}
fn usable(state: &Footprint) -> bool {
    state.bytes > 0 && state.is_complete()
}
fn project_proof(
    path: &Path,
    kind: ItemKind,
    cancel: &AtomicBool,
) -> Result<Option<Footprint>, ()> {
    if kind != ItemKind::ProjectArtifact {
        return Ok(None);
    }
    let project = path.parent().ok_or(())?;
    let state = measure(project, cancel);
    if state.incomplete || state.has_links {
        return Err(());
    }
    Ok(Some(state))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prevents_parent_child_double_count_without_matching_similar_names() {
        let mut paths = SeenPaths::default();
        paths.0.insert(PathBuf::from("root/cache/app"));
        assert!(paths.overlaps(Path::new("root/cache/app")));
        assert!(paths.overlaps(Path::new("root/cache/app/nested")));
        assert!(paths.overlaps(Path::new("root/cache")));
        assert!(!paths.overlaps(Path::new("root/cache/app-other")));
        assert!(!paths.overlaps(Path::new("root/cache2")));
    }
}
