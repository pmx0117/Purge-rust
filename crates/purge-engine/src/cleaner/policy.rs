use crate::{
    platform,
    scanner::{self, Rule},
};
use anyhow::{Context, Result, bail};
use purge_core::{Category, Item, ItemKind, ScanOptions};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

/// A cleanup batch captures one policy, while every candidate is revalidated
/// against the live filesystem immediately before the recycler receives it.
pub(super) struct CleanupPolicy<'a> {
    settings: &'a ScanOptions,
    rules: Vec<Rule>,
    protected: Vec<PathBuf>,
    current_directory: Option<PathBuf>,
    personal_roots: Vec<PathBuf>,
}
impl<'a> CleanupPolicy<'a> {
    pub(super) fn current(settings: &'a ScanOptions) -> Self {
        Self {
            settings,
            rules: scanner::cache_rules(),
            protected: vec![
                platform::home_dir(),
                platform::local_dir(),
                platform::roaming_dir(),
            ],
            current_directory: std::env::current_dir().ok(),
            personal_roots: platform::personal_roots(),
        }
    }
    pub(super) fn validate(&self, item: &Item) -> Result<()> {
        require_cleanable(item)?;
        self.validate_location(&item.path)?;
        self.validate_allowlist(item)?;
        validate_snapshot(item)?;
        validate_project_proof(item, self.settings)
    }
    fn validate_location(&self, path: &Path) -> Result<()> {
        validate_path_shape(path)?;
        if scanner::contains_exclusion(path, self.settings) {
            bail!("该路径位于排除列表中");
        }
        self.validate_roots(path)?;
        self.validate_current_directory(path)
    }
    fn validate_roots(&self, path: &Path) -> Result<()> {
        if path.parent().is_none()
            || self
                .protected
                .iter()
                .any(|root| platform::same_path(path, root))
        {
            bail!("受保护的根目录");
        }
        Ok(())
    }
    fn validate_current_directory(&self, path: &Path) -> Result<()> {
        if self
            .current_directory
            .as_ref()
            .is_some_and(|directory| paths_overlap(directory, path))
        {
            bail!("正在运行 Purge 的工作目录受到保护");
        }
        Ok(())
    }
    fn validate_allowlist(&self, item: &Item) -> Result<()> {
        let allowed = match item.category() {
            Category::Caches => self.cache_allowed(item),
            Category::Developer => self.cache_allowed(item) || self.project_allowed(&item.path),
            Category::LargeFiles => self.personal_file_allowed(&item.path),
            Category::Applications => false,
        };
        if !allowed {
            bail!("路径不在清理白名单中");
        }
        Ok(())
    }
    fn cache_allowed(&self, item: &Item) -> bool {
        self.rules.iter().any(|rule| rule_matches(rule, item))
    }
    fn project_allowed(&self, path: &Path) -> bool {
        scanner::project_artifact(path)
            && inside_one_root(path, &self.settings.project_roots)
            && project_is_unlocked(path)
    }
    fn personal_file_allowed(&self, path: &Path) -> bool {
        is_local_file(path) && inside_one_root(path, &self.personal_roots)
    }
}
fn require_cleanable(item: &Item) -> Result<()> {
    if !item.cleanable() {
        bail!("该项目不能移入回收站");
    }
    Ok(())
}
fn validate_path_shape(path: &Path) -> Result<()> {
    if !path.is_absolute() || has_parent_component(path) || scanner::has_link_component(path) {
        bail!("路径包含符号链接、联接点或无法访问的位置");
    }
    Ok(())
}
fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}
fn paths_overlap(first: &Path, second: &Path) -> bool {
    platform::within(first, second) || platform::within(second, first)
}
fn inside_one_root(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| is_child_of(path, root))
}
fn is_child_of(path: &Path, root: &Path) -> bool {
    platform::within(path, root) && !platform::same_path(path, root)
}
fn project_is_unlocked(path: &Path) -> bool {
    path.parent().is_some_and(project_directory_unlocked)
}
fn project_directory_unlocked(project: &Path) -> bool {
    !project.join(".purge-keep").exists() && !project.join(".git/index.lock").exists()
}
fn is_local_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && !platform::is_cloud_placeholder(&metadata))
}
fn rule_matches(rule: &Rule, item: &Item) -> bool {
    if rule.category != item.category() {
        return false;
    }
    if rule.children {
        return old_child_matches(rule, &item.path);
    }
    platform::same_path(&item.path, &rule.path)
}
fn old_child_matches(rule: &Rule, path: &Path) -> bool {
    if !path
        .parent()
        .is_some_and(|parent| platform::same_path(parent, &rule.path))
    {
        return false;
    }
    let state = scanner::measure(path, &AtomicBool::new(false));
    readable_old_enough(&state, rule.minimum_age)
}
fn readable_old_enough(state: &purge_core::Footprint, minimum: std::time::Duration) -> bool {
    state.is_complete() && state.older_than(std::time::SystemTime::now(), minimum)
}

fn validate_snapshot(item: &Item) -> Result<()> {
    if scanner::measure(&item.path, &AtomicBool::new(false)) != item.footprint {
        bail!("文件在扫描后发生变化，请重新扫描");
    }
    Ok(())
}
fn validate_project_proof(item: &Item, settings: &ScanOptions) -> Result<()> {
    if needs_project_proof(item) {
        return validate_project_activity(item, settings);
    }
    Ok(())
}
fn needs_project_proof(item: &Item) -> bool {
    item.kind == ItemKind::ProjectArtifact || scanner::project_artifact(&item.path)
}
fn validate_project_activity(item: &Item, settings: &ScanOptions) -> Result<()> {
    let previous = item
        .project_guard
        .as_ref()
        .context("项目扫描信息不完整，请重新扫描")?;
    let project = item.path.parent().context("项目目录不存在")?;
    let current = scanner::measure(project, &AtomicBool::new(false));
    if &current != previous {
        bail!("项目在扫描后有新活动，请重新扫描");
    }
    validate_project_age(&current, settings)
}
fn validate_project_age(current: &purge_core::Footprint, settings: &ScanOptions) -> Result<()> {
    if !settings.project_is_stale(current, std::time::SystemTime::now()) {
        bail!("项目尚未达到设置的闲置时间");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use purge_core::{Footprint, Safety, path_id};
    use std::time::{Duration, SystemTime};

    fn temporary_item(path: &Path) -> Item {
        Item {
            id: path_id(path),
            name: "temporary fixture".into(),
            path: path.into(),
            kind: ItemKind::Temporary,
            safety: Safety::Safe,
            description: String::new(),
            footprint: scanner::measure(path, &AtomicBool::new(false)),
            project_guard: None,
            modified: None,
            duplicate_group: None,
            duplicate_copies: Default::default(),
        }
    }

    #[test]
    fn temporary_cleanup_rechecks_fixed_retention_independently_of_project_settings() {
        let fixture = tempfile::TempDir::new().unwrap();
        let temporary = fixture.path().join("Temp");
        fs::create_dir(&temporary).unwrap();
        let old = temporary.join("old.tmp");
        let recent = temporary.join("recent.tmp");
        fs::write(&old, "old").unwrap();
        fs::write(&recent, "recent").unwrap();
        fs::OpenOptions::new()
            .write(true)
            .open(&old)
            .unwrap()
            .set_times(
                fs::FileTimes::new()
                    .set_modified(SystemTime::now() - Duration::from_secs(48 * 3600)),
            )
            .unwrap();
        let old_item = temporary_item(&old);
        let recent_item = temporary_item(&recent);
        assert_ne!(old_item.footprint, Footprint::default());
        for stale_project_days in [0, 30, 90, 180] {
            let settings = ScanOptions {
                stale_project_days,
                ..Default::default()
            };
            let policy = CleanupPolicy {
                settings: &settings,
                rules: scanner::discover_cache_rules(
                    fixture.path(),
                    fixture.path(),
                    fixture.path(),
                ),
                protected: vec![],
                current_directory: None,
                personal_roots: vec![],
            };
            policy.validate(&old_item).unwrap();
            assert!(
                policy.validate(&recent_item).is_err(),
                "project setting {stale_project_days} bypassed temporary retention"
            );
        }
        fs::write(&old, "recent activity").unwrap();
        let settings = ScanOptions::default();
        let policy = CleanupPolicy {
            settings: &settings,
            rules: scanner::discover_cache_rules(fixture.path(), fixture.path(), fixture.path()),
            protected: vec![],
            current_directory: None,
            personal_roots: vec![],
        };
        assert!(
            policy.validate(&old_item).is_err(),
            "a newly active temporary file cannot be recycled from its old snapshot"
        );
    }
}
