use purge_core::*;
use std::{collections::HashSet, path::PathBuf};

fn item(kind: ItemKind) -> Item {
    Item {
        id: 1,
        name: "sample".into(),
        path: PathBuf::from("sample"),
        safety: Safety::Safe,
        description: String::new(),
        kind,
        footprint: Footprint {
            bytes: 10,
            files: 1,
            ..Default::default()
        },
        project_guard: None,
        modified: None,
        duplicate_group: None,
    }
}

#[test]
fn safe_bulk_selection_never_accepts_personal_files_even_with_a_wrong_safety_label() {
    let personal = item(ItemKind::PersonalFile(FileKind::Video));
    assert!(!personal.safe_cleanable());
    let (add, _) = selection_toggle(&[personal], &HashSet::new(), true);
    assert!(add.is_empty());
}

#[test]
fn kind_owns_category_and_cleanup_capability() {
    for (kind, category, can_recycle, can_bulk) in [
        (ItemKind::ApplicationCache, Category::Caches, true, true),
        (ItemKind::DeveloperCache, Category::Developer, true, true),
        (ItemKind::Temporary, Category::Caches, true, true),
        (ItemKind::ProjectArtifact, Category::Developer, true, false),
        (
            ItemKind::PersonalFile(FileKind::Document),
            Category::LargeFiles,
            true,
            false,
        ),
        (
            ItemKind::InstalledApplication,
            Category::Applications,
            false,
            false,
        ),
    ] {
        let candidate = item(kind);
        assert_eq!(candidate.category(), category);
        assert_eq!(candidate.cleanable(), can_recycle);
        assert_eq!(candidate.safe_cleanable(), can_bulk);
    }
}

#[test]
fn unreadable_or_linked_candidates_are_never_selectable_for_cleanup() {
    let mut candidate = item(ItemKind::ApplicationCache);
    candidate.footprint.incomplete = true;
    assert!(!candidate.cleanable());
    assert!(!candidate.safe_cleanable());
    candidate.footprint.incomplete = false;
    candidate.footprint.has_links = true;
    assert!(!candidate.cleanable());
    assert!(!candidate.safe_cleanable());
}

#[test]
fn file_classification_handles_case_and_unknown_extensions() {
    for (extension, expected) in [
        ("MP4", FileKind::Video),
        ("wav", FileKind::Audio),
        ("png", FileKind::Image),
        ("7z", FileKind::Archive),
        ("pdf", FileKind::Document),
        ("gguf", FileKind::AiModel),
        ("vhdx", FileKind::VirtualDisk),
        ("msi", FileKind::Installer),
        ("unrecognized", FileKind::Other),
    ] {
        assert_eq!(FileKind::from_extension(extension), expected);
        assert!(!expected.label().is_empty());
    }
}

#[test]
fn equally_named_personal_files_stay_separate() {
    let first = item(ItemKind::PersonalFile(FileKind::Other));
    let mut second = first.clone();
    second.id = 2;
    assert_eq!(group_items(vec![first, second]).len(), 2);
}

#[test]
fn safe_bulk_toggle_selects_only_safe_cache_entries_then_clears_them() {
    let safe = item(ItemKind::ApplicationCache);
    let mut review = item(ItemKind::DeveloperCache);
    review.id = 2;
    review.safety = Safety::Review;
    let mut personal = item(ItemKind::PersonalFile(FileKind::Other));
    personal.id = 3;
    let entries = [safe, review, personal];
    let mut selected = HashSet::new();
    let (add, remove) = selection_toggle(&entries, &selected, true);
    assert_eq!(add, vec![1]);
    assert!(remove.is_empty());
    selected.extend(add);
    selected.insert(2);
    let (add, remove) = selection_toggle(&entries, &selected, true);
    assert!(add.is_empty());
    assert_eq!(remove, vec![1, 2]);
}

#[test]
fn manual_bulk_toggle_can_select_personal_files_and_only_clears_visible_entries() {
    let first = item(ItemKind::PersonalFile(FileKind::Video));
    let mut second = first.clone();
    second.id = 2;
    let entries = [first, second];
    let mut selected = HashSet::from([99]);
    let (add, remove) = selection_toggle(&entries, &selected, false);
    assert_eq!(add, vec![1, 2]);
    assert!(remove.is_empty());
    selected.extend(add);
    let (add, remove) = selection_toggle(&entries, &selected, false);
    assert!(add.is_empty());
    assert_eq!(remove, vec![1, 2]);
    assert!(!remove.contains(&99));
}

#[test]
fn nested_mounts_do_not_count_as_the_home_volume_just_because_paths_start_with_root() {
    let mut candidate = item(ItemKind::ApplicationCache);
    candidate.path = "/mnt/other/cache".into();
    candidate.footprint.volume_id = Some(22);
    let disk = DiskInfo {
        name: "/".into(),
        total: 100,
        free: 20,
        volume_id: Some(11),
    };
    assert!(!disk.contains(&candidate));
    candidate.footprint.volume_id = Some(11);
    assert!(disk.contains(&candidate));
}
