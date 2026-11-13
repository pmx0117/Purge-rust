use purge::{
    controller::AppController,
    launch::LaunchOptions,
    model::Page,
    preferences::{Filter, Sort},
};
use purge_core::*;
use purge_engine::storage::LocalRepository;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tempfile::TempDir;

fn controller(directory: &TempDir, launch: LaunchOptions) -> AppController {
    AppController::new(LocalRepository::new(directory.path()), launch)
}
fn candidate() -> Item {
    Item {
        id: 7,
        name: "invalid relative candidate".into(),
        path: PathBuf::from("sample-relative"),
        safety: Safety::Safe,
        description: String::new(),
        kind: ItemKind::ApplicationCache,
        footprint: Footprint {
            bytes: 10,
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
fn a_preview_theme_override_does_not_change_saved_preferences() {
    let directory = TempDir::new().unwrap();
    let mut app = controller(
        &directory,
        LaunchOptions {
            theme: Some(true),
            page: Some(Page::Category(Category::Caches)),
        },
    );
    assert!(app.appearance());
    app.set_filter(Filter::All);
    app.set_sort(Sort::Smallest);
    let stored = LocalRepository::new(directory.path())
        .load_settings()
        .unwrap();
    assert!(!stored.dark);
    assert_eq!(stored.cache_filter, "all");
    assert_eq!(stored.cache_sort, "smallest");
    app.set_appearance(true);
    assert!(
        LocalRepository::new(directory.path())
            .load_settings()
            .unwrap()
            .dark
    );
}
#[test]
fn cleanup_is_never_started_before_the_review_is_acknowledged() {
    let directory = TempDir::new().unwrap();
    let mut app = controller(&directory, LaunchOptions::default());
    assert!(app.review_items(vec![candidate()]));
    assert!(!app.confirm_cleanup());
    assert!(!app.busy());
    assert!(app.can_close());
}
#[test]
fn a_failed_cleanup_is_reported_and_persisted_without_claiming_bytes_moved() {
    let directory = TempDir::new().unwrap();
    let mut app = controller(&directory, LaunchOptions::default());
    app.model.items.push(candidate());
    app.review_items(vec![candidate()]);
    app.model.toggle_acknowledgement();
    assert!(app.confirm_cleanup());
    assert!(!app.can_close());
    let deadline = Instant::now() + Duration::from_secs(10);
    while app.busy() && Instant::now() < deadline {
        app.poll();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!app.busy());
    assert!(app.can_close());
    assert_eq!(app.model.warnings.len(), 1);
    let history = LocalRepository::new(directory.path())
        .load_history()
        .unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].bytes, 0);
    assert_eq!(history[0].moved, 0);
    assert_eq!(history[0].failed, 1);
}
#[test]
fn temporary_group_exclusion_is_one_parent_rule_not_thousands_of_duplicates() {
    let directory = TempDir::new().unwrap();
    let mut app = controller(&directory, LaunchOptions::default());
    let mut first = candidate();
    first.kind = ItemKind::Temporary;
    first.path = PathBuf::from("temporary/a");
    let mut second = first.clone();
    second.id = 8;
    second.path = PathBuf::from("temporary/b");
    app.model.items = vec![first.clone(), second.clone()];
    app.exclude_group(&ItemGroup {
        id: 1,
        name: "localized display name".into(),
        items: vec![first, second],
    });
    assert!(app.model.items.is_empty());
    assert_eq!(
        app.model.settings.scan.exclusions,
        vec![PathBuf::from("temporary")]
    );
}

#[test]
fn selection_and_directory_actions_are_coordinated_without_gui_state() {
    let directory = TempDir::new().unwrap();
    let mut app = controller(&directory, LaunchOptions::default());
    app.model.items = vec![candidate()];
    app.toggle_group(&[7]);
    assert!(app.model.selected.contains(&7));
    app.toggle_group(&[7]);
    assert!(app.model.selected.is_empty());
    app.navigate(Page::Category(Category::Caches));
    app.set_filter(Filter::All);
    app.toggle_visible("");
    assert!(app.model.selected.contains(&7));
    app.toggle_visible("");
    assert!(app.model.selected.is_empty());
    let mut installed = candidate();
    installed.id = 8;
    installed.kind = ItemKind::InstalledApplication;
    app.model.items.push(installed);
    app.select_items(&[7, 8, 999]);
    assert_eq!(app.model.selected, [7].into_iter().collect());
    app.add_directories(
        vec![PathBuf::from("projects"), PathBuf::from("projects")],
        false,
    );
    app.add_directories(vec![PathBuf::from("excluded")], true);
    let stored = LocalRepository::new(directory.path())
        .load_settings()
        .unwrap();
    assert_eq!(stored.scan.project_roots, vec![PathBuf::from("projects")]);
    assert_eq!(stored.scan.exclusions, vec![PathBuf::from("excluded")]);
    assert_eq!(app.data_directory().unwrap(), directory.path());
}

#[test]
fn changing_application_layout_keeps_the_current_results_and_persists_the_choice() {
    use purge::preferences::AppLayout;
    let directory = TempDir::new().unwrap();
    let mut app = controller(&directory, LaunchOptions::default());
    assert_eq!(app.model.settings.app_layout, AppLayout::Cards);
    let mut installed = candidate();
    installed.name = "Fixture Editor".into();
    installed.kind = ItemKind::InstalledApplication;
    let mut cache = candidate();
    cache.id = 99;
    app.model.items = vec![installed, cache];
    app.navigate(Page::Category(Category::Applications));
    app.set_sort(Sort::Name);
    app.model.selected.insert(99);
    let before: Vec<_> = app
        .model
        .visible("editor")
        .iter()
        .map(|item| item.id)
        .collect();
    app.set_app_layout(AppLayout::List);
    assert_eq!(
        app.model
            .visible("editor")
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        before
    );
    assert_eq!(app.model.sort, Sort::Name);
    assert_eq!(app.model.selected, [99].into_iter().collect());
    let reopened = controller(&directory, LaunchOptions::default());
    assert_eq!(reopened.model.settings.app_layout, AppLayout::List);
    assert_eq!(AppLayout::from_key("unknown-layout"), AppLayout::Cards);
}
