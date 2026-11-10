use purge::{
    launch::LaunchOptions,
    model::{AppModel, Page},
    preferences::{Filter, Sort},
};
use purge_core::*;
use std::{collections::HashMap, path::PathBuf};
fn item(id: u64, name: &str, kind: ItemKind, bytes: u64) -> Item {
    Item {
        id,
        name: name.into(),
        path: PathBuf::from(format!("sample/{name}")),
        safety: Safety::Safe,
        description: String::new(),
        kind,
        footprint: Footprint {
            bytes,
            files: 1,
            ..Default::default()
        },
        project_guard: None,
        modified: None,
        duplicate_group: None,
    }
}
#[test]
fn background_scanning_does_not_replace_about_information_with_internal_progress() {
    let mut model = AppModel::default();
    model.navigate(Page::About);
    model.status = "正在检查 internal/cache/location".into();
    assert_eq!(model.header_subtitle(true), "版本信息、数据与隐私。");
}
#[test]
fn rescan_only_replaces_requested_categories_and_their_selection() {
    let mut model = AppModel {
        items: vec![
            item(1, "browser", ItemKind::ApplicationCache, 20),
            item(2, "cargo", ItemKind::DeveloperCache, 10),
        ],
        ..Default::default()
    };
    model.selected.extend([1, 2]);
    model
        .scanned
        .extend([Category::Caches, Category::Developer]);
    model.begin_scan(vec![Category::Caches]);
    assert_eq!(model.items.len(), 1);
    assert_eq!(model.items[0].id, 2);
    assert_eq!(model.selected.len(), 1);
    assert!(model.selected.contains(&2));
    assert!(model.scanned.contains(&Category::Developer));
    assert!(!model.scanned.contains(&Category::Caches));
}
#[test]
fn grouped_results_and_filter_sort_search_stay_consistent() {
    let mut model = AppModel {
        items: vec![
            item(1, "Chrome · Default", ItemKind::ApplicationCache, 20),
            item(2, "Chrome · Profile 1", ItemKind::ApplicationCache, 30),
            item(3, "Code", ItemKind::ApplicationCache, 10),
        ],
        ..Default::default()
    };
    model.navigate(Page::Category(Category::Caches));
    model.set_filter(Filter::All);
    let groups = model.grouped_visible("");
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].name, "Chrome");
    assert_eq!(groups[0].bytes(), 50);
    assert_eq!(model.visible("cHroMe").len(), 2);
    model.set_sort(Sort::Smallest);
    assert_eq!(model.grouped_visible("")[0].name, "Code");
    model.items[2].safety = Safety::Review;
    model.set_filter(Filter::Review);
    assert_eq!(model.visible("")[0].id, 3);
}
#[test]
fn duplicate_results_do_not_install_a_hidden_filter_on_another_page() {
    let mut model = AppModel::default();
    model.navigate(Page::Category(Category::Caches));
    model.set_filter(Filter::All);
    model.apply_duplicates(HashMap::new(), false);
    assert_eq!(model.filter, Filter::All);
    model.navigate(Page::Category(Category::LargeFiles));
    model.apply_duplicates(HashMap::new(), true);
    assert_eq!(model.filter, Filter::Duplicates);
}
#[test]
fn cleanup_requires_acknowledgement_and_keeps_a_duplicate_copy() {
    let mut model = AppModel::default();
    let mut first = item(1, "a.mp4", ItemKind::PersonalFile(FileKind::Video), 100);
    first.duplicate_group = Some(1);
    let mut second = first.clone();
    second.id = 2;
    model.items = vec![first.clone(), second.clone()];
    assert!(model.show_review(vec![first.clone()]));
    assert!(model.cleanup_plan().is_none());
    assert!(model.review.is_some());
    model.toggle_acknowledgement();
    assert_eq!(model.cleanup_plan().unwrap().len(), 1);
    assert!(model.review.is_none());
    model.show_review(vec![first, second]);
    model.toggle_acknowledgement();
    assert!(model.cleanup_plan().is_none());
    assert!(model.notice.as_ref().unwrap().contains("至少保留一份"));
}
#[test]
fn installed_applications_cannot_open_a_filesystem_cleanup_plan() {
    let mut model = AppModel::default();
    assert!(!model.show_review(vec![item(1, "app", ItemKind::InstalledApplication, 10)]));
    assert!(model.review.is_none());
}
#[test]
fn launch_options_are_parsed_once_with_the_existing_switches() {
    let options = LaunchOptions::parse(
        [
            "--theme=dark",
            "--page=caches",
            "--theme=light",
            "--page=about",
            "--unknown",
        ]
        .into_iter()
        .map(str::to_string),
    );
    assert_eq!(options.theme, Some(false));
    assert_eq!(options.page, Some(Page::About));
}

#[test]
fn scan_events_apply_in_order_and_only_finish_the_requested_categories() {
    use purge_engine::scanner::ScanEvent;
    let mut model = AppModel::default();
    model.begin_scan(vec![Category::Caches]);
    model.apply_scan_event(ScanEvent::Progress("checking".into()), "14:00");
    assert_eq!(model.header_subtitle(true), "checking");
    let cached = item(1, "browser", ItemKind::ApplicationCache, 20);
    model.apply_scan_event(ScanEvent::Item(Box::new(cached.clone())), "14:00");
    model.apply_scan_event(ScanEvent::Item(Box::new(cached)), "14:00");
    assert_eq!(model.items.len(), 1);
    model.apply_scan_event(
        ScanEvent::Storage {
            disk: DiskInfo {
                name: "disk".into(),
                total: 100,
                free: 30,
            },
            trash_bytes: Some(10),
        },
        "14:00",
    );
    assert_eq!(model.disk.used(), 70);
    assert_eq!(model.trash_bytes, Some(10));
    model.apply_scan_event(ScanEvent::Warning("skipped".into()), "14:00");
    assert_eq!(model.warnings, vec!["skipped"]);
    model.apply_scan_event(ScanEvent::Finished { cancelled: false }, "14:00");
    assert!(model.scanned.contains(&Category::Caches));
    assert!(!model.scanned.contains(&Category::Developer));
    assert_eq!(model.last_scan.as_deref(), Some("14:00"));
    model.begin_scan(vec![Category::Developer]);
    model.apply_scan_event(ScanEvent::Finished { cancelled: true }, "14:01");
    assert!(!model.scanned.contains(&Category::Developer));
    assert!(model.header_subtitle(false).contains("结果不完整"));
    model.navigate(Page::Settings);
    assert!(!model.header_subtitle(false).contains("结果不完整"));
}

#[test]
fn selection_bar_and_filter_counts_describe_the_same_visible_groups() {
    let mut review = item(3, "Code", ItemKind::ApplicationCache, 10);
    review.safety = Safety::Review;
    let mut model = AppModel {
        items: vec![
            item(1, "Chrome · Default", ItemKind::ApplicationCache, 20),
            item(2, "Chrome · Profile", ItemKind::ApplicationCache, 30),
            review,
        ],
        ..Default::default()
    };
    model.navigate(Page::Category(Category::Caches));
    model.set_filter(Filter::All);
    let tabs = model.filter_tabs(Category::Caches);
    assert_eq!(
        tabs.iter().map(|tab| tab.count).collect::<Vec<_>>(),
        vec![2, 1, 1]
    );
    assert!(tabs[0].selected);
    let visible = model.visible("");
    let summary = model.selection_summary(&visible);
    assert_eq!(summary.label(), "全选安全项目");
    assert!(!summary.show_review_link);
    model.selected.extend([1, 2]);
    let summary = model.selection_summary(&visible);
    assert!(summary.mixed);
    assert!(summary.show_review_link);
    assert_eq!(summary.review_add, vec![3]);
    assert_eq!(summary.review_bytes, 10);
    model.selected.insert(3);
    let summary = model.selection_summary(&visible);
    assert!(summary.all);
    assert_eq!(summary.label(), "全选");
    assert!(model.filter_tabs(Category::Applications).is_empty());
    assert_eq!(
        model
            .filter_tabs(Category::LargeFiles)
            .last()
            .unwrap()
            .filter,
        Filter::Duplicates
    );
}
