use super::*;
#[derive(Debug)]
pub struct SelectionSummary {
    pub all: bool,
    pub mixed: bool,
    pub safe_first: bool,
    pub review_add: Vec<u64>,
    pub review_bytes: u64,
    pub show_review_link: bool,
}
impl SelectionSummary {
    pub fn label(&self) -> &'static str {
        if self.safe_first && !self.all {
            "全选安全项目"
        } else {
            "全选"
        }
    }
}
#[derive(Debug)]
pub struct FilterTab {
    pub filter: Filter,
    pub label: &'static str,
    pub glyph: &'static str,
    pub count: usize,
    pub selected: bool,
}
impl AppModel {
    pub fn selection_summary(&self, visible: &[Item]) -> SelectionSummary {
        let selectable: Vec<_> = visible.iter().filter(|item| item.cleanable()).collect();
        let all = !selectable.is_empty()
            && selectable
                .iter()
                .all(|item| self.selected.contains(&item.id));
        let mixed = !all
            && selectable
                .iter()
                .any(|item| self.selected.contains(&item.id));
        let safe_first = matches!(
            self.page,
            Page::Category(Category::Caches | Category::Developer)
        ) && self.filter == Filter::All
            && visible.iter().any(|item| item.safety == Safety::Review);
        let review_add: Vec<_> = visible
            .iter()
            .filter(|item| item.safety == Safety::Review && !self.selected.contains(&item.id))
            .map(|item| item.id)
            .collect();
        let review_bytes = visible
            .iter()
            .filter(|item| review_add.contains(&item.id))
            .map(Item::bytes)
            .sum();
        let show_review_link = safe_first
            && visible
                .iter()
                .filter(|item| item.safety == Safety::Safe)
                .all(|item| self.selected.contains(&item.id))
            && !review_add.is_empty();
        SelectionSummary {
            all,
            mixed,
            safe_first,
            review_add,
            review_bytes,
            show_review_link,
        }
    }
    pub fn filter_tabs(&self, category: Category) -> Vec<FilterTab> {
        filters(category)
            .iter()
            .map(|filter| self.filter_tab(category, *filter))
            .collect()
    }
    fn filter_tab(&self, category: Category, filter: Filter) -> FilterTab {
        let (label, glyph) = filter_metadata(filter);
        let count = group_items(
            self.items
                .iter()
                .filter(|item| {
                    item.category() == category && super::queries::matches_filter(item, filter)
                })
                .cloned()
                .collect(),
        )
        .len();
        FilterTab {
            filter,
            label,
            glyph,
            count,
            selected: self.filter == filter,
        }
    }
}
fn filters(category: Category) -> &'static [Filter] {
    match category {
        Category::Applications => &[],
        Category::LargeFiles => &[Filter::All, Filter::Review, Filter::Duplicates],
        _ => &[Filter::All, Filter::Safe, Filter::Review],
    }
}
fn filter_metadata(filter: Filter) -> (&'static str, &'static str) {
    match filter {
        Filter::All => ("全部", "overview"),
        Filter::Safe => ("可安全清理", "shield"),
        Filter::Review => ("需要确认", "info"),
        Filter::Duplicates => ("重复文件", "copy"),
    }
}
