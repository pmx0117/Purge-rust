use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyAction {
    Scan,
    ResetFilters,
    Review,
    Settings,
}

pub struct EmptyState {
    pub title: &'static str,
    pub hint: String,
    pub action: Option<EmptyAction>,
}

impl AppModel {
    pub fn can_select(&self, id: u64) -> bool {
        let Some(item) = self
            .items
            .iter()
            .find(|item| item.id == id && item.cleanable())
        else {
            return false;
        };
        item.duplicate_group.is_none_or(|group| {
            self.items.iter().any(|other| {
                other.duplicate_group == Some(group)
                    && other.id != id
                    && !self.selected.contains(&other.id)
            })
        })
    }

    pub fn toggle_duplicate_extras(&mut self, visible: &[Item]) {
        let mut groups = HashSet::new();
        let mut extras = Vec::new();
        for item in visible {
            let Some(group) = item.duplicate_group else {
                continue;
            };
            if !groups.insert(group) {
                continue;
            }
            let keeper = self
                .items
                .iter()
                .find(|item| {
                    item.duplicate_group == Some(group) && !self.selected.contains(&item.id)
                })
                .map(|item| item.id)
                .unwrap_or(item.id);
            self.selected.remove(&keeper);
            extras.extend(
                visible
                    .iter()
                    .filter(|item| {
                        item.duplicate_group == Some(group) && item.id != keeper && item.cleanable()
                    })
                    .map(|item| item.id),
            );
        }
        let clear = !extras.is_empty() && extras.iter().all(|id| self.selected.contains(id));
        for id in extras {
            if clear {
                self.selected.remove(&id);
            } else {
                self.selected.insert(id);
            }
        }
    }

    pub fn can_find_duplicates(&self) -> bool {
        self.items
            .iter()
            .filter(|item| item.category() == Category::LargeFiles && item.cleanable())
            .take(2)
            .count()
            == 2
    }

    pub fn reset_result_filters(&mut self) {
        self.filter = Filter::All;
        self.kind_filter = None;
        self.only_selected = false;
    }

    pub fn selected_items(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|item| {
                self.page.scan_category() == Some(item.category())
                    && self.selected.contains(&item.id)
            })
            .cloned()
            .collect()
    }

    pub fn hidden_selection_count(&self, visible: &[Item]) -> usize {
        let visible: HashSet<_> = visible.iter().map(|item| item.id).collect();
        self.selected_items()
            .iter()
            .filter(|item| !visible.contains(&item.id))
            .count()
    }

    pub fn select_duplicate_keeper(&mut self, keeper: u64) {
        let Some(group) = self
            .items
            .iter()
            .find(|item| item.id == keeper)
            .and_then(|item| item.duplicate_group)
        else {
            return;
        };
        for item in &self.items {
            if item.duplicate_group == Some(group) {
                if item.id != keeper && item.cleanable() {
                    self.selected.insert(item.id);
                } else {
                    self.selected.remove(&item.id);
                }
            }
        }
    }

    pub fn results_stale(&self, category: Category) -> bool {
        let Some(previous) = self.scan_options.get(&category) else {
            return false;
        };
        let current = &self.settings.scan;
        match category {
            Category::Applications => false,
            Category::Caches => previous.exclusions != current.exclusions,
            Category::LargeFiles => {
                previous.minimum_file_mb != current.minimum_file_mb
                    || previous.exclusions != current.exclusions
            }
            Category::Developer => {
                previous.stale_project_days != current.stale_project_days
                    || previous.project_roots != current.project_roots
                    || previous.exclusions != current.exclusions
            }
        }
    }

    pub fn page_results_stale(&self) -> bool {
        match self.page {
            Page::Category(category) => self.results_stale(category),
            Page::Overview => self
                .scan_options
                .keys()
                .any(|category| self.results_stale(*category)),
            _ => false,
        }
    }

    pub fn scanned_file_threshold(&self) -> u64 {
        self.scan_options
            .get(&Category::LargeFiles)
            .unwrap_or(&self.settings.scan)
            .minimum_file_mb
    }

    pub fn empty_state(&self, query: &str, busy: bool) -> EmptyState {
        let category = self.page.scan_category();
        let total = self
            .items
            .iter()
            .filter(|item| Some(item.category()) == category)
            .count();
        let (title, hint, action) = if busy {
            (
                "正在检查文件",
                "任务仍在进行，完成后会更新结果。".into(),
                None,
            )
        } else if !query.is_empty() {
            (
                "没有匹配的文件",
                "试试其他名称或路径，或清除搜索和筛选。".into(),
                Some(EmptyAction::ResetFilters),
            )
        } else if self.only_selected && self.selected_items().is_empty() {
            (
                "当前分类没有已选项目",
                "可以返回全部结果继续选择。".into(),
                Some(EmptyAction::ResetFilters),
            )
        } else if total > 0 && self.filter == Filter::Safe {
            let review = self
                .items
                .iter()
                .filter(|item| Some(item.category()) == category && item.safety == Safety::Review)
                .count();
            (
                "没有可安全清理的项目",
                format!("另有 {review} 项需要你确认，请先检查文件及清理影响。"),
                Some(EmptyAction::Review),
            )
        } else if total > 0 && self.filter == Filter::Duplicates {
            (
                "没有发现重复文件",
                "可以返回全部文件，或重新比较文件内容。".into(),
                Some(EmptyAction::ResetFilters),
            )
        } else if total > 0 {
            (
                "当前筛选没有结果",
                "其他项目仍在，请调整类型或筛选条件。".into(),
                Some(EmptyAction::ResetFilters),
            )
        } else if category.is_some_and(|category| !self.scanned.contains(&category)) {
            (
                "尚未扫描此分类",
                "扫描后会在这里显示结果。".into(),
                Some(EmptyAction::Scan),
            )
        } else if category == Some(Category::Developer)
            && self.settings.scan.project_roots.is_empty()
        {
            (
                "本次未发现项目",
                "可以在设置中添加开发项目目录，再重新扫描。".into(),
                Some(EmptyAction::Settings),
            )
        } else {
            (
                "本次未发现项目",
                "检查扫描范围或调整设置后，可以重新扫描。".into(),
                Some(EmptyAction::Scan),
            )
        };
        EmptyState {
            title,
            hint,
            action,
        }
    }
}
