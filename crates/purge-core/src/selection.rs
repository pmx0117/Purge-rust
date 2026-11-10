use crate::Item;
/// Port of SafeFirstSelectAll.toggled(): selecting mixed results never sweeps
/// review items into the initial bulk selection.
pub fn selection_toggle(
    items: &[Item],
    selected: &std::collections::HashSet<u64>,
    safe_first: bool,
) -> (Vec<u64>, Vec<u64>) {
    let entries: Vec<_> = items.iter().filter(|i| i.cleanable()).collect();
    let should_clear = if safe_first {
        entries
            .iter()
            .filter(|i| i.safe_cleanable())
            .all(|i| selected.contains(&i.id))
    } else {
        !entries.is_empty() && entries.iter().all(|i| selected.contains(&i.id))
    };
    if should_clear {
        (
            vec![],
            entries
                .iter()
                .filter(|i| selected.contains(&i.id))
                .map(|i| i.id)
                .collect(),
        )
    } else {
        (
            entries
                .iter()
                .filter(|i| (!safe_first || i.safe_cleanable()) && !selected.contains(&i.id))
                .map(|i| i.id)
                .collect(),
            vec![],
        )
    }
}
