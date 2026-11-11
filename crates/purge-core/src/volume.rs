#[derive(Debug, Clone, Default)]
pub struct DiskInfo {
    pub name: String,
    pub total: u64,
    pub free: u64,
    pub volume_id: Option<u64>,
}
impl DiskInfo {
    pub fn contains(&self, item: &crate::Item) -> bool {
        if let (Some(volume), Some(candidate)) = (self.volume_id, item.footprint.volume_id) {
            return volume == candidate;
        }
        path_within(&item.path, std::path::Path::new(&self.name))
    }
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.free)
    }
}

/// Compare complete path components with native Windows case handling.
pub fn path_within(path: &std::path::Path, root: &std::path::Path) -> bool {
    #[cfg(windows)]
    {
        let normalized = |path: &std::path::Path| {
            path.components()
                .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
                .collect::<Vec<_>>()
        };
        normalized(path).starts_with(&normalized(root))
    }
    #[cfg(not(windows))]
    {
        path.starts_with(root)
    }
}
