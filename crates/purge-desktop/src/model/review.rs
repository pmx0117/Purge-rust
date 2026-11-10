use super::*;
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct ReviewPlan {
    pub items: Vec<Item>,
    pub acknowledged: bool,
}
impl AppModel {
    pub fn show_review(&mut self, items: Vec<Item>) -> bool {
        let items: Vec<_> = items.into_iter().filter(Item::cleanable).collect();
        if items.is_empty() {
            return false;
        }
        self.review = Some(ReviewPlan {
            items,
            acknowledged: false,
        });
        true
    }
    pub fn acknowledged(&self) -> bool {
        self.review.as_ref().is_some_and(|plan| plan.acknowledged)
    }
    pub fn toggle_acknowledgement(&mut self) {
        if let Some(plan) = &mut self.review {
            plan.acknowledged = !plan.acknowledged;
        }
    }
    pub fn cleanup_plan(&mut self) -> Option<Vec<Item>> {
        if !self.acknowledged() {
            return None;
        }
        let items = self.review.take()?.items;
        if self.consumes_duplicate_group(&items) {
            self.notice =
                Some("该选择包含某组重复文件的全部副本。请至少保留一份，再确认清理。".into());
            return None;
        }
        Some(items)
    }
    fn consumes_duplicate_group(&self, items: &[Item]) -> bool {
        let selected: HashSet<_> = items.iter().map(|item| item.id).collect();
        items
            .iter()
            .filter_map(|item| item.duplicate_group)
            .any(|group| self.all_group_members_selected(group, &selected))
    }
    fn all_group_members_selected(&self, group: u64, selected: &HashSet<u64>) -> bool {
        self.items
            .iter()
            .filter(|item| item.duplicate_group == Some(group))
            .all(|item| selected.contains(&item.id))
    }
    pub fn apply_duplicates(&mut self, groups: HashMap<u64, u64>, cancelled: bool) {
        for item in &mut self.items {
            item.duplicate_group = groups.get(&item.id).copied();
        }
        if self.page == Page::Category(Category::LargeFiles) {
            self.filter = Filter::Duplicates;
        }
        self.notice = Some(
            if cancelled {
                "检查已停止，显示已完成的结果。"
            } else {
                "重复文件已逐字节核对。请选择不需要的副本，至少保留一份。"
            }
            .into(),
        );
    }
    pub fn record_cleanup(&mut self, report: CleanupReport, time: String) {
        self.trash_bytes = report.trash_bytes;
        self.items
            .retain(|item| !report.moved_ids.contains(&item.id));
        self.selected.clear();
        let entry = HistoryEntry {
            time,
            bytes: report.bytes,
            moved: report.moved_ids.len(),
            failed: report.failures.len(),
        };
        self.notice = Some(format!(
            "已将 {} 项（{}）移入回收站，{} 项跳过。清空回收站后才会释放空间。",
            entry.moved,
            crate::format::format_bytes(entry.bytes),
            entry.failed
        ));
        self.warnings = report.failures;
        self.history.insert(0, entry);
        self.history.truncate(200);
    }
}
