mod catalog;
mod collector;
mod duplicates;
mod events;
mod exclusions;
mod fingerprint;
mod personal;
mod projects;
mod scope;
pub use catalog::{
    Rule, TEMPORARY_RETENTION_HOURS, cache_rules, discover_cache_rules, discover_cache_rules_for,
};
use collector::{Candidate, ScanCollector};
pub use duplicates::find_duplicates;
pub use events::ScanEvent;
pub use exclusions::{contains_exclusion, excluded};
pub use fingerprint::{has_link_component, measure};
pub use personal::file_kind;
pub use projects::project_artifact;
use purge_core::{Category, ItemKind, ScanOptions};
pub use scope::ScanScope;
use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
};

pub fn scan(
    settings: ScanOptions,
    categories: Vec<Category>,
    cancel: Arc<AtomicBool>,
    sender: Sender<ScanEvent>,
) {
    if cancel.load(Ordering::Relaxed) {
        let _ = sender.send(ScanEvent::Finished { cancelled: true });
        return;
    }
    let scope = ScanScope::current(&categories);
    scan_in(scope, settings, categories, cancel, sender);
}
pub fn scan_in(
    scope: ScanScope,
    settings: ScanOptions,
    categories: Vec<Category>,
    cancel: Arc<AtomicBool>,
    sender: Sender<ScanEvent>,
) {
    let mut collector = ScanCollector::new(&settings, &cancel, &sender);
    collector.emit(ScanEvent::Storage {
        disk: scope.disk.clone(),
        trash_bytes: scope.trash_bytes,
    });
    for category in categories {
        if collector.cancelled() {
            break;
        }
        collector.progress(format!("正在扫描{}…", category.label()));
        scan_category(category, &scope, &mut collector);
    }
    collector.emit(ScanEvent::Finished {
        cancelled: collector.cancelled(),
    });
}
fn scan_category(category: Category, scope: &ScanScope, collector: &mut ScanCollector<'_>) {
    match category {
        Category::Caches => scan_caches(category, &scope.rules, collector),
        Category::Developer => {
            scan_caches(category, &scope.rules, collector);
            projects::scan_projects(collector);
        }
        Category::LargeFiles => personal::scan_large_files(&scope.personal_roots, collector),
        Category::Applications => emit_applications(scope, collector),
    }
}
fn emit_applications(scope: &ScanScope, collector: &ScanCollector<'_>) {
    for item in &scope.applications {
        collector.emit(ScanEvent::Item(Box::new(item.clone())));
    }
    if let Some(message) = &scope.application_warning {
        collector.emit(ScanEvent::Warning(message.clone()));
    }
}
fn scan_caches(category: Category, rules: &[Rule], collector: &mut ScanCollector<'_>) {
    for rule in rules.iter().filter(|rule| rule.category == category) {
        if collector.cancelled() {
            return;
        }
        scan_rule(rule, collector);
    }
}
fn scan_rule(rule: &Rule, collector: &mut ScanCollector<'_>) {
    if excluded(&rule.path, collector.settings) || has_link_component(&rule.path) {
        return;
    }
    collector.progress(format!("正在检查 {}", rule.name));
    if rule.children {
        scan_old_children(rule, collector);
        return;
    }
    collector.add(Candidate {
        path: rule.path.clone(),
        name: rule.name.clone(),
        safety: rule.safety,
        description: rule.description.clone(),
        kind: cache_kind(rule.category),
        measured: None,
    });
}
fn cache_kind(category: Category) -> ItemKind {
    match category {
        Category::Caches => ItemKind::ApplicationCache,
        _ => ItemKind::DeveloperCache,
    }
}
fn scan_old_children(rule: &Rule, collector: &mut ScanCollector<'_>) {
    let Ok(children) = fs::read_dir(&rule.path) else {
        return;
    };
    for child in children.flatten() {
        if collector.cancelled() {
            return;
        }
        let footprint = measure(&child.path(), collector.cancel);
        if !old_enough(&footprint, rule.minimum_age) {
            continue;
        }
        collector.add(Candidate {
            path: child.path(),
            name: format!("{} · {}", rule.name, child.file_name().to_string_lossy()),
            safety: rule.safety,
            description: rule.description.clone(),
            kind: ItemKind::Temporary,
            measured: Some(footprint),
        });
    }
}
fn old_enough(state: &purge_core::Footprint, minimum: std::time::Duration) -> bool {
    state.older_than(std::time::SystemTime::now(), minimum)
}
