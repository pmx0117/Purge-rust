use crate::{
    launch::LaunchOptions,
    model::{AppModel, Page},
    preferences::{AppLayout, Filter, Preferences, Sort},
};
use purge_core::*;
use purge_engine::{
    cleaner,
    jobs::{JobEvent, Jobs, Operation},
    platform,
    scanner::{self, ScanEvent},
    storage::LocalRepository,
};
use std::{path::PathBuf, thread};

/// The controller is the sole writer of application data and persistence.
/// Filesystem workers communicate only through Jobs; GPUI is an adapter.
pub struct AppController {
    pub model: AppModel,
    jobs: Jobs,
    repository: LocalRepository,
    theme_override: Option<bool>,
}
impl AppController {
    pub fn new(repository: LocalRepository, launch: LaunchOptions) -> Self {
        let (settings, notice) = load_preferences(&repository);
        let mut model = AppModel::new(settings);
        model.notice = notice;
        load_history(&repository, &mut model);
        model.disk = platform::disk_info().unwrap_or_default();
        if let Some(page) = launch.page {
            model.navigate(page);
        }
        Self {
            model,
            jobs: Jobs::default(),
            repository,
            theme_override: launch.theme,
        }
    }
    pub fn appearance(&self) -> bool {
        self.theme_override.unwrap_or(self.model.settings.dark)
    }
    pub fn busy(&self) -> bool {
        self.jobs.busy()
    }
    pub fn operation(&self) -> Option<Operation> {
        self.jobs.operation()
    }
    pub fn can_close(&self) -> bool {
        self.jobs.can_close()
    }
    pub fn cancel(&self) -> bool {
        self.jobs.cancel()
    }
    pub fn save(&mut self) {
        if let Err(error) = self
            .repository
            .save_settings(&self.model.settings.to_stored())
        {
            self.model.notice = Some(format!("设置保存失败：{error}"));
        }
    }
    pub fn set_appearance(&mut self, dark: bool) {
        self.theme_override = None;
        self.model.settings.dark = dark;
        self.save();
    }

    pub fn set_app_layout(&mut self, layout: AppLayout) {
        self.model.settings.app_layout = layout;
        self.save();
    }
    pub fn navigate(&mut self, page: Page) -> bool {
        self.model.navigate(page)
    }
    pub fn set_filter(&mut self, filter: Filter) -> bool {
        if !self.model.set_filter(filter) {
            return false;
        }
        self.save_scoped_choices();
        true
    }
    pub fn set_sort(&mut self, sort: Sort) {
        self.model.set_sort(sort);
        self.save_scoped_choices();
    }
    fn save_scoped_choices(&mut self) {
        if self.model.choice_is_persistent() {
            self.save();
        }
    }
    pub fn start_scan(&mut self, categories: Vec<Category>) -> bool {
        if self.model.review.is_some() {
            return false;
        }
        let Ok((sender, cancel)) = self.jobs.begin_scan() else {
            return false;
        };
        self.model.begin_scan(categories.clone());
        let settings = self.model.settings.scan.clone();
        self.spawn("purge-scan", move || {
            scanner::scan(settings, categories, cancel, sender)
        });
        true
    }
    pub fn review_items(&mut self, items: Vec<Item>) -> bool {
        if self.busy() {
            return false;
        }
        self.model.show_review(items)
    }
    pub fn review_safe(&mut self) -> bool {
        let items = self
            .model
            .items
            .iter()
            .filter(|item| item.safe_cleanable())
            .cloned()
            .collect();
        self.review_items(items)
    }
    pub fn confirm_cleanup(&mut self) -> bool {
        if self.busy() {
            return false;
        }
        let Some(items) = self.model.cleanup_plan() else {
            return false;
        };
        let Ok(sender) = self.jobs.begin_cleanup() else {
            return false;
        };
        let settings = self.model.settings.scan.clone();
        self.model.status = "正在移入系统回收站…".into();
        self.spawn("purge-cleanup", move || {
            let _ = sender.send(cleaner::clean(&items, &settings));
        });
        true
    }
    pub fn start_duplicates(&mut self) -> bool {
        if self.model.review.is_some() || !self.model.can_find_duplicates() {
            return false;
        }
        let Ok((sender, cancel)) = self.jobs.begin_duplicates() else {
            return false;
        };
        let items = self.model.items.clone();
        self.model.status = "正在比较重复文件内容…".into();
        self.spawn("purge-duplicates", move || {
            let _ = sender.send(scanner::find_duplicates(&items, &cancel));
        });
        true
    }
    fn spawn(&mut self, name: &str, work: impl FnOnce() + Send + 'static) {
        if let Err(error) = thread::Builder::new().name(name.into()).spawn(work) {
            self.model.notice = Some(format!("无法启动任务：{error}"));
        }
    }
    pub fn poll(&mut self) -> bool {
        let cancelled = self.jobs.cancellation_requested();
        let events = self.jobs.poll();
        let changed = !events.is_empty();
        for event in events {
            self.apply_event(event, cancelled);
        }
        changed
    }
    fn apply_event(&mut self, event: JobEvent, cancelled: bool) {
        match event {
            JobEvent::Scan(event) => self.apply_scan(event),
            JobEvent::Cleanup(report) => self.apply_cleanup(report),
            JobEvent::Duplicates(groups) => {
                self.model.apply_duplicates(groups, cancelled);
                self.model.status = "准备就绪".into();
            }
            JobEvent::Interrupted(operation) => self.apply_interruption(operation),
        }
    }
    fn apply_scan(&mut self, event: ScanEvent) {
        let finished = matches!(event, ScanEvent::Finished { .. });
        self.model
            .apply_scan_event(event, &chrono::Local::now().format("%H:%M").to_string());
        if finished {
            self.model.disk = platform::disk_info().unwrap_or_default();
        }
    }
    fn apply_cleanup(&mut self, report: CleanupReport) {
        self.model.record_cleanup(
            report,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        if let Err(error) = self.repository.save_history(&self.model.history) {
            self.model.warnings.push(format!("历史保存失败：{error}"));
        }
        self.model.disk = platform::disk_info().unwrap_or_default();
        self.model.status = "准备就绪".into();
    }
    fn apply_interruption(&mut self, operation: Operation) {
        self.model
            .notice
            .get_or_insert_with(|| interruption_message(operation).into());
        self.model.status = "任务已中断".into();
        if operation == Operation::Scan {
            self.model.scan_cancelled = true;
        }
    }
    pub fn select_items(&mut self, ids: &[u64]) {
        if self.busy() {
            return;
        }
        for id in ids {
            if self.model.can_select(*id) {
                self.model.selected.insert(*id);
            }
        }
    }
    pub fn toggle_group(&mut self, ids: &[u64]) {
        if self.busy() {
            return;
        }
        let clear = ids.iter().all(|id| self.model.selected.contains(id));
        for id in ids {
            if clear {
                self.model.selected.remove(id);
            } else if self.model.can_select(*id) {
                self.model.selected.insert(*id);
            } else {
                self.model.notice =
                    Some("请至少保留一份重复文件；可以使用“保留此份”选择其他副本。".into());
            }
        }
    }
    pub fn toggle_visible(&mut self, query: &str) {
        if self.busy() {
            return;
        }
        let items = self.model.visible(query);
        if self.model.filter == Filter::Duplicates {
            self.model.toggle_duplicate_extras(&items);
            return;
        }
        let safe_first = matches!(
            self.model.page,
            Page::Category(Category::Caches | Category::Developer)
        ) && self.model.filter == Filter::All;
        let (add, remove) = selection_toggle(&items, &self.model.selected, safe_first);
        self.model.selected.extend(add);
        for id in remove {
            self.model.selected.remove(&id);
        }
    }
    pub fn exclude_group(&mut self, group: &ItemGroup) {
        if self.busy() {
            return;
        }
        let paths = excluded_paths(group);
        self.model.settings.scan.exclusions.extend(paths);
        for item in &group.items {
            self.model.items.retain(|current| current.id != item.id);
            self.model.selected.remove(&item.id);
        }
        self.model.refresh_duplicate_copies();
        self.save();
    }
    pub fn add_directories(&mut self, paths: Vec<PathBuf>, exclusion: bool) {
        if self.busy() {
            return;
        }
        let target = if exclusion {
            &mut self.model.settings.scan.exclusions
        } else {
            &mut self.model.settings.scan.project_roots
        };
        for path in paths {
            if !target.contains(&path) {
                target.push(path);
            }
        }
        self.save();
    }
    pub fn data_directory(&self) -> anyhow::Result<PathBuf> {
        Ok(self.repository.ensure_directory()?.to_path_buf())
    }
}
fn load_preferences(repository: &LocalRepository) -> (Preferences, Option<String>) {
    match repository.load_settings() {
        Ok(settings) => (settings.into(), None),
        Err(error) => (Preferences::default(), Some(error.to_string())),
    }
}
fn load_history(repository: &LocalRepository, model: &mut AppModel) {
    match repository.load_history() {
        Ok(history) => model.history = history,
        Err(error) => model.notice = Some(error.to_string()),
    }
}
const INTERRUPTION_MESSAGES: &[(Operation, &str)] = &[
    (Operation::Scan, "扫描意外中断，请重新扫描。"),
    (
        Operation::Cleanup,
        "本次清理未正常完成，请重新扫描并确认文件状态。",
    ),
    (Operation::Duplicates, "重复文件检查意外中断，请重试。"),
];
fn interruption_message(operation: Operation) -> &'static str {
    INTERRUPTION_MESSAGES
        .iter()
        .find(|entry| entry.0 == operation)
        .map(|entry| entry.1)
        .unwrap_or("任务已中断")
}

fn excluded_paths(group: &ItemGroup) -> Vec<PathBuf> {
    let paths = if group
        .items
        .iter()
        .all(|item| item.kind == ItemKind::Temporary)
    {
        group
            .items
            .iter()
            .filter_map(|item| item.path.parent().map(|path| path.to_path_buf()))
            .collect::<Vec<_>>()
    } else {
        group.items.iter().map(|item| item.path.clone()).collect()
    };
    paths
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backend_events_are_projected_without_gui_or_localized_task_guards() {
        let directory = tempfile::TempDir::new().unwrap();
        let mut app = AppController::new(
            LocalRepository::new(directory.path()),
            LaunchOptions::default(),
        );
        app.apply_event(
            JobEvent::Scan(ScanEvent::Progress("checking".into())),
            false,
        );
        assert_eq!(app.model.status, "checking");
        app.model.navigate(Page::Category(Category::LargeFiles));
        app.apply_event(JobEvent::Duplicates(Default::default()), true);
        assert_eq!(app.model.filter, Filter::Duplicates);
        app.model.notice = None;
        app.apply_event(JobEvent::Interrupted(Operation::Scan), false);
        assert!(app.model.scan_cancelled);
        assert!(app.model.notice.unwrap().contains("重新扫描"));
    }
}
