use super::*;
impl AppModel {
    pub fn visible(&self, query: &str) -> Vec<Item> {
        let Some(category) = self.page.scan_category() else {
            return vec![];
        };
        let normalized = query.to_lowercase();
        let mut items: Vec<_> = self
            .items
            .iter()
            .filter(|item| self.matches(item, category, &normalized))
            .cloned()
            .collect();
        sort_items(&mut items, self.sort);
        items
    }
    fn matches(&self, item: &Item, category: Category, query: &str) -> bool {
        item.category() == category
            && matches_filter(item, self.filter)
            && self.matches_kind(item)
            && matches_query(item, query)
    }
    fn matches_kind(&self, item: &Item) -> bool {
        self.kind_filter
            .is_none_or(|kind| item.kind == ItemKind::PersonalFile(kind))
    }
    pub fn grouped_visible(&self, query: &str) -> Vec<ItemGroup> {
        let mut groups = group_items(self.visible(query));
        match self.sort {
            Sort::Size => groups.sort_by_key(|group| std::cmp::Reverse(group.bytes())),
            Sort::Smallest => groups.sort_by_key(ItemGroup::bytes),
            Sort::Name => groups.sort_by_key(|group| group.name.to_lowercase()),
            _ => {}
        }
        groups
    }
}
pub(super) fn matches_filter(item: &Item, filter: Filter) -> bool {
    match filter {
        Filter::All => true,
        Filter::Safe => item.safety == Safety::Safe,
        Filter::Review => item.safety == Safety::Review,
        Filter::Duplicates => item.duplicate_group.is_some(),
    }
}
fn matches_query(item: &Item, query: &str) -> bool {
    query.is_empty()
        || format!("{} {} {}", item.name, item.path.display(), item.kind)
            .to_lowercase()
            .contains(query)
}
fn sort_items(items: &mut [Item], sort: Sort) {
    match sort {
        Sort::Size => {
            items.sort_by(|a, b| b.bytes().cmp(&a.bytes()).then_with(|| a.name.cmp(&b.name)))
        }
        Sort::Smallest => items.sort_by_key(Item::bytes),
        Sort::Name => items.sort_by_key(|item| item.name.to_lowercase()),
        Sort::Oldest => items.sort_by_key(|item| item.modified),
        Sort::Newest => items.sort_by_key(|item| std::cmp::Reverse(item.modified)),
    }
}

impl AppModel {
    pub fn header_subtitle(&self, busy: bool) -> String {
        if busy && self.page.shows_scan_status() {
            return self.status.clone();
        }
        let subtitle = self.page_subtitle();
        if self.partial_scan_applies() {
            return format!("{subtitle} · 扫描已停止，结果不完整");
        }
        subtitle
    }
    fn partial_scan_applies(&self) -> bool {
        if !self.scan_cancelled {
            return false;
        }
        match self.page {
            Page::Overview => true,
            Page::Category(category) => self.scan_categories.contains(&category),
            _ => false,
        }
    }
    fn page_subtitle(&self) -> String {
        match self.page {
            Page::Overview => self
                .last_scan
                .as_ref()
                .map(|time| format!("上次扫描 {time}"))
                .unwrap_or_else(|| "尚未扫描".into()),
            Page::Category(category) => self.category_subtitle(category),
            Page::Settings => "按你的使用习惯调整扫描范围。".into(),
            Page::History => "每次清理都有记录，文件可以从回收站恢复。".into(),
            Page::About => "版本信息、数据与隐私。".into(),
        }
    }
    fn category_subtitle(&self, category: Category) -> String {
        let count = group_items(
            self.items
                .iter()
                .filter(|item| item.category() == category)
                .cloned()
                .collect(),
        )
        .len();
        let suffix = if category == Category::Applications {
            "（安装程序估算）"
        } else {
            ""
        };
        format!(
            "{count} 个项目 · {}{suffix}",
            crate::format::format_bytes(self.total(category))
        )
    }
}
