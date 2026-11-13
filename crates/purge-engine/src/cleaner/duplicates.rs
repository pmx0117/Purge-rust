use crate::{platform, scanner};
use anyhow::{Result, bail};
use purge_core::{Item, path_id};
use std::{collections::HashSet, sync::atomic::AtomicBool};

pub(super) fn validate(item: &Item, selected: &HashSet<u64>) -> Result<()> {
    if item.duplicate_group.is_none() {
        return Ok(());
    }
    let cancel = AtomicBool::new(false);
    for copy in item.duplicate_copies.iter() {
        if platform::same_path(&copy.path, &item.path)
            || selected.contains(&path_id(&copy.path))
            || !copy.footprint.is_complete()
            || scanner::has_link_component(&copy.path)
            || !copy.path.is_file()
            || scanner::measure(&copy.path, &cancel) != copy.footprint
        {
            continue;
        }
        if scanner::files_equal(&item.path, &copy.path, &cancel)
            && scanner::measure(&copy.path, &cancel) == copy.footprint
        {
            return Ok(());
        }
    }
    bail!("没有可验证的保留副本，文件可能已被移动或修改。请重新查找重复文件，至少保留一份。")
}

#[cfg(test)]
mod tests {
    use super::*;
    use purge_core::{DuplicateCopy, FileKind, ItemKind, Safety};
    use std::{fs, path::Path};

    fn candidate(path: &Path) -> Item {
        Item {
            id: path_id(path),
            name: "duplicate fixture".into(),
            path: path.into(),
            safety: Safety::Review,
            description: String::new(),
            kind: ItemKind::PersonalFile(FileKind::Other),
            footprint: scanner::measure(path, &AtomicBool::new(false)),
            project_guard: None,
            modified: None,
            duplicate_group: Some(1),
            duplicate_copies: Default::default(),
        }
    }

    #[test]
    fn only_an_unchanged_unselected_copy_allows_duplicate_cleanup() {
        let directory = tempfile::TempDir::new().unwrap();
        let first = directory.path().join("first");
        let keeper = directory.path().join("keeper");
        fs::write(&first, b"identical").unwrap();
        fs::write(&keeper, b"identical").unwrap();
        let mut item = candidate(&first);
        item.duplicate_copies = vec![DuplicateCopy {
            path: keeper.clone(),
            footprint: scanner::measure(&keeper, &AtomicBool::new(false)),
        }]
        .into();
        assert!(validate(&item, &HashSet::from([item.id])).is_ok());
        assert!(validate(&item, &HashSet::from([item.id, path_id(&keeper)])).is_err());
        fs::write(&keeper, b"different").unwrap();
        assert!(validate(&item, &HashSet::from([item.id])).is_err());
        fs::remove_file(&keeper).unwrap();
        let report = crate::cleaner::clean_with(&[item], &Default::default(), |_| {
            panic!("last remaining copy must never reach the recycler")
        });
        assert!(report.moved_ids.is_empty());
        assert!(report.failures[0].contains("保留副本"));
        assert!(first.exists());
        assert!(report.items[0].error.is_some());
    }

    #[test]
    fn missing_proof_and_a_changed_copy_cannot_authorize_cleanup() {
        let directory = tempfile::TempDir::new().unwrap();
        let first = directory.path().join("first");
        let keeper = directory.path().join("keeper");
        fs::write(&first, b"original").unwrap();
        fs::write(&keeper, b"changed!").unwrap();
        let mut item = candidate(&first);
        assert!(validate(&item, &HashSet::new()).is_err());
        item.duplicate_copies = vec![DuplicateCopy {
            path: keeper.clone(),
            footprint: scanner::measure(&keeper, &AtomicBool::new(false)),
        }]
        .into();
        assert!(validate(&item, &HashSet::new()).is_err());
    }
}
