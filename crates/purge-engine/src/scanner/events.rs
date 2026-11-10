use purge_core::{DiskInfo, Item};
#[derive(Debug)]
pub enum ScanEvent {
    Progress(String),
    Item(Box<Item>),
    Warning(String),
    Storage {
        disk: DiskInfo,
        trash_bytes: Option<u64>,
    },
    Finished {
        cancelled: bool,
    },
}
