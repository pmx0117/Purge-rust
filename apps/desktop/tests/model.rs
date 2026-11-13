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
        duplicate_copies: Default::default(),
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
                volume_id: None,
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

#[test]
fn rescan_clears_stale_duplicate_filter_and_keeps_comparison_available() {
    let mut model = AppModel::default();
    model.navigate(Page::Category(Category::LargeFiles));
    model.items = vec![
        item(1, "a", ItemKind::PersonalFile(FileKind::Other), 10),
        item(2, "b", ItemKind::PersonalFile(FileKind::Other), 10),
    ];
    model.apply_duplicates(HashMap::from([(1, 1), (2, 1)]), false);
    assert_eq!(model.filter, Filter::Duplicates);
    model.begin_scan(vec![Category::LargeFiles]);
    for id in [1, 2] {
        model.apply_scan_event(
            purge_engine::scanner::ScanEvent::Item(Box::new(item(
                id,
                &format!("file-{id}"),
                ItemKind::PersonalFile(FileKind::Other),
                10,
            ))),
            "fixture",
        );
    }
    assert_eq!(model.visible("").len(), 2);
    model.kind_filter = Some(FileKind::Video);
    assert!(model.visible("").is_empty());
    assert!(model.can_find_duplicates());
}

#[test]
fn cleanup_updates_duplicate_groups_and_preserves_valid_remaining_groups() {
    let mut model = AppModel {
        items: (1..=3)
            .map(|id| {
                item(
                    id,
                    &format!("file-{id}"),
                    ItemKind::PersonalFile(FileKind::Other),
                    10,
                )
            })
            .collect(),
        ..Default::default()
    };
    model.apply_duplicates(HashMap::from([(1, 1), (2, 1), (3, 1)]), false);
    model.record_cleanup(
        CleanupReport {
            moved_ids: vec![3],
            ..Default::default()
        },
        "fixture".into(),
    );
    assert!(
        model
            .items
            .iter()
            .all(|item| item.duplicate_copies.len() == 2)
    );
    model.record_cleanup(
        CleanupReport {
            moved_ids: vec![2],
            ..Default::default()
        },
        "fixture".into(),
    );
    assert_eq!(model.items.len(), 1);
    assert!(model.items[0].duplicate_group.is_none());
    assert!(model.items[0].duplicate_copies.is_empty());
}

#[test]
fn safe_empty_results_explain_and_reveal_review_items() {
    let mut model = AppModel::default();
    let mut review = item(1, "uv", ItemKind::DeveloperCache, 8 * 1024 * 1024);
    review.safety = Safety::Review;
    model.items.push(review);
    model.navigate(Page::Category(Category::Developer));
    assert!(model.visible("").is_empty());
    let empty = model.empty_state("", false);
    assert_eq!(empty.title, "没有可安全清理的项目");
    assert!(empty.hint.contains("1 项"));
    assert_eq!(empty.action, Some(purge::model::EmptyAction::Review));
    model.set_filter(Filter::Review);
    assert_eq!(model.visible("").len(), 1);
}

#[test]
fn hidden_selection_and_explicit_keeper_match_the_cleanup_plan() {
    let mut model = AppModel::default();
    model.navigate(Page::Category(Category::LargeFiles));
    model.items = (1..=3)
        .map(|id| {
            item(
                id,
                &format!("file-{id}"),
                ItemKind::PersonalFile(FileKind::Other),
                10,
            )
        })
        .collect();
    model.apply_duplicates(HashMap::from([(1, 1), (2, 1), (3, 1)]), false);
    model.select_duplicate_keeper(2);
    assert_eq!(model.selected, [1, 3].into_iter().collect());
    assert_eq!(model.hidden_selection_count(&model.visible("file-1")), 1);
    model.only_selected = true;
    assert_eq!(
        model.empty_state("missing-file", false).title,
        "没有匹配的文件"
    );
    model.show_review(model.selected_items());
    model.toggle_acknowledgement();
    model.remove_review_item(3);
    assert!(!model.acknowledged());
    assert_eq!(model.review.as_ref().unwrap().items.len(), 1);
    model.toggle_acknowledgement();
    assert_eq!(model.cleanup_plan().unwrap()[0].id, 1);
}

#[test]
fn changed_settings_keep_the_scanned_threshold_and_mark_only_affected_results_stale() {
    let mut model = AppModel::default();
    model.begin_scan(vec![Category::LargeFiles, Category::Caches]);
    let threshold = model.scanned_file_threshold();
    model.settings.scan.minimum_file_mb = threshold + 100;
    assert!(model.results_stale(Category::LargeFiles));
    assert!(!model.results_stale(Category::Caches));
    assert_eq!(model.scanned_file_threshold(), threshold);
    model.begin_scan(vec![Category::LargeFiles]);
    assert!(!model.results_stale(Category::LargeFiles));
}

#[cfg(target_os = "linux")]
#[test]
fn scanning_case_distinct_files_preserves_both_in_the_ui_model() {
    use std::sync::{Arc, atomic::AtomicBool, mpsc};
    let directory = tempfile::TempDir::new().unwrap();
    for name in ["Case.bin", "case.bin"] {
        std::fs::write(directory.path().join(name), b"fixture").unwrap();
    }
    let (sender, receiver) = mpsc::channel();
    purge_engine::scanner::scan_in(
        purge_engine::scanner::ScanScope {
            personal_roots: vec![directory.path().into()],
            ..Default::default()
        },
        ScanOptions {
            minimum_file_mb: 0,
            ..Default::default()
        },
        vec![Category::LargeFiles],
        Arc::new(AtomicBool::new(false)),
        sender,
    );
    let mut model = AppModel::default();
    model.navigate(Page::Category(Category::LargeFiles));
    model.begin_scan(vec![Category::LargeFiles]);
    for event in receiver {
        model.apply_scan_event(event, "fixture");
    }
    assert_eq!(model.visible("").len(), 2);
    assert_ne!(model.items[0].id, model.items[1].id);
}

#[test]
fn duplicate_bulk_selection_preserves_a_copy_in_every_group() {
    let mut model = AppModel {
        items: (1..=4)
            .map(|id| {
                item(
                    id,
                    &format!("file-{id}"),
                    ItemKind::PersonalFile(FileKind::Other),
                    10,
                )
            })
            .collect(),
        ..Default::default()
    };
    model.navigate(Page::Category(Category::LargeFiles));
    model.apply_duplicates(HashMap::from([(1, 1), (2, 1), (3, 3), (4, 3)]), false);
    model.toggle_duplicate_extras(&model.visible(""));
    assert_eq!(model.selected.len(), 2);
    for group in [1, 3] {
        let keeper = model
            .items
            .iter()
            .find(|item| item.duplicate_group == Some(group) && !model.selected.contains(&item.id))
            .unwrap();
        assert!(!model.can_select(keeper.id));
    }
    model.toggle_duplicate_extras(&model.visible(""));
    assert!(model.selected.is_empty());
}
