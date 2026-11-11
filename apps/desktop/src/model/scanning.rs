use super::*;
use purge_engine::scanner::ScanEvent;
impl AppModel {
    pub fn begin_scan(&mut self, categories: Vec<Category>) {
        self.items
            .retain(|item| !categories.contains(&item.category()));
        self.selected
            .retain(|id| self.items.iter().any(|item| item.id == *id));
        for category in &categories {
            self.scanned.remove(category);
        }
        self.scan_categories = categories;
        self.warnings.clear();
        self.notice = None;
        self.status = "正在准备扫描…".into();
    }
    pub fn apply_scan_event(&mut self, event: ScanEvent, time: &str) {
        match event {
            ScanEvent::Progress(message) => self.status = message,
            ScanEvent::Item(item) => self.insert_item(*item),
            ScanEvent::Warning(message) => self.warnings.push(message),
            ScanEvent::Storage { disk, trash_bytes } => {
                self.disk = disk;
                self.trash_bytes = trash_bytes;
            }
            ScanEvent::Finished { cancelled } => self.finish_scan(cancelled, time),
        }
    }
    fn insert_item(&mut self, item: Item) {
        if !self.items.iter().any(|current| current.id == item.id) {
            self.items.push(item);
        }
    }
    fn finish_scan(&mut self, cancelled: bool, time: &str) {
        self.scan_cancelled = cancelled;
        self.last_scan = Some(time.into());
        self.status = if cancelled {
            "扫描已停止 · 已发现的结果仍可查看"
        } else {
            "扫描完成"
        }
        .into();
        if !cancelled {
            self.scanned.extend(self.scan_categories.iter().copied());
        }
    }
}
