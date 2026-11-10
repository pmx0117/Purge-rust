#[derive(Debug, Clone, Default)]
pub struct DiskInfo {
    pub name: String,
    pub total: u64,
    pub free: u64,
}
impl DiskInfo {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.free)
    }
}
