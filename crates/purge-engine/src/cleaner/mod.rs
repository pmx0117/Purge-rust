mod duplicates;
mod policy;
use crate::platform;
use anyhow::{Context, Result};
use policy::CleanupPolicy;
use purge_core::{CleanupEntry, CleanupReport, Item, ScanOptions as Settings};
use std::{collections::HashSet, path::Path};
/// Public boundary for validating one candidate using current trusted rules.
pub fn validate(item: &Item, settings: &Settings) -> Result<()> {
    duplicates::validate(item, &HashSet::new())?;
    CleanupPolicy::current(settings).validate(item)
}
pub fn clean(items: &[Item], settings: &Settings) -> CleanupReport {
    let mut report = clean_with(items, settings, |path| {
        trash::delete(path).context("系统回收站操作失败")
    });
    report.trash_bytes = platform::trash_bytes().unwrap_or(None);
    report
}

/// Injectable recycler allows safety tests without deleting any host files.
pub fn clean_with(
    items: &[Item],
    settings: &Settings,
    mut recycle: impl FnMut(&Path) -> Result<()>,
) -> CleanupReport {
    let policy = CleanupPolicy::current(settings);
    let mut report = CleanupReport::default();
    let mut seen = HashSet::new();
    let selected = items
        .iter()
        .map(|item| purge_core::path_id(&item.path))
        .collect();
    for item in items {
        if !seen.insert(item.id) {
            continue;
        }
        let result = duplicates::validate(item, &selected)
            .and_then(|()| policy.validate(item))
            .and_then(|()| recycle(&item.path));
        let error = result.as_ref().err().map(|error| format!("{error:#}"));
        report.items.push(CleanupEntry {
            name: item.name.clone(),
            path: item.path.clone(),
            bytes: item.bytes(),
            error,
        });
        match result {
            Ok(()) => {
                report.moved_ids.push(item.id);
                report.bytes = report.bytes.saturating_add(item.bytes());
            }
            Err(error) => report
                .failures
                .push(format!("{}：{error:#}", item.path.display())),
        }
    }
    report
}
