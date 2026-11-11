use super::{Rule, TEMPORARY_RETENTION_HOURS, cache_rule, profile_rules};
use crate::platform::OperatingSystem;
use purge_core::{Category, Safety};
use std::{path::Path, time::Duration};

pub(super) fn rules(
    system: OperatingSystem,
    home: &Path,
    cache: &Path,
    config: &Path,
) -> Vec<Rule> {
    let mut result = vec![];
    let browsers: &[(&str, &str)] = if system == OperatingSystem::MacOs {
        &[
            ("Google/Chrome", "Google Chrome"),
            ("Microsoft Edge", "Microsoft Edge"),
            ("BraveSoftware/Brave-Browser", "Brave"),
            ("Vivaldi", "Vivaldi"),
        ]
    } else {
        &[
            ("google-chrome", "Google Chrome"),
            ("chromium", "Chromium"),
            ("microsoft-edge", "Microsoft Edge"),
            ("BraveSoftware/Brave-Browser", "Brave"),
            ("vivaldi", "Vivaldi"),
        ]
    };
    for &(root, label) in browsers {
        result.extend(profile_rules(&cache.join(root), label));
    }
    result.extend(application_rules(config));
    if system == OperatingSystem::MacOs {
        result.push(cache_rule(
            cache.join("com.apple.Safari"),
            "Safari".into(),
            Category::Caches,
            Safety::Safe,
            "Safari 生成的缓存，清理前请先关闭浏览器。",
        ));
    }
    let data = if system == OperatingSystem::MacOs {
        home.join("Library")
    } else if home == crate::platform::home_dir() {
        crate::platform::user_data_dir()
    } else {
        home.join(".local/share")
    };
    for (path, name, safety) in [
        (home.join(".npm/_cacache"), "npm", Safety::Safe),
        (data.join("pnpm/store"), "pnpm store", Safety::Review),
        (
            cache.join(if system == OperatingSystem::MacOs {
                "Yarn"
            } else {
                "yarn"
            }),
            "Yarn",
            Safety::Safe,
        ),
        (cache.join("pip"), "pip", Safety::Safe),
        (cache.join("uv"), "uv", Safety::Review),
        (
            home.join(".cargo/registry/cache"),
            "Cargo 下载缓存",
            Safety::Safe,
        ),
        (home.join(".gradle/caches"), "Gradle", Safety::Safe),
        (home.join(".nuget/packages"), "NuGet", Safety::Review),
    ] {
        result.push(cache_rule(
            path,
            name.into(),
            Category::Developer,
            safety,
            "工具下载或构建缓存，清理后需重新下载；请先关闭包管理器和构建任务。",
        ));
    }
    // Never enumerate the shared /tmp tree or a user's whole cache directory.
    for (path, name) in [
        (cache.join("tmp"), "临时文件"),
        (cache.join("Crashpad"), "应用崩溃报告"),
    ] {
        result.push(Rule { path, name: name.into(), category: Category::Caches, safety: Safety::Safe,
            description: format!("仅列出至少 {TEMPORARY_RETENTION_HOURS} 小时未修改的用户临时条目；不受开发项目闲置时间影响。"),
            children: true, minimum_age: Duration::from_secs(TEMPORARY_RETENTION_HOURS * 3600) });
    }
    result
}

fn application_rules(config: &Path) -> Vec<Rule> {
    let mut result = vec![];
    for app in ["Code", "Cursor", "discord", "Slack", "Notion"] {
        for leaf in ["Cache", "Code Cache", "GPUCache"] {
            result.push(cache_rule(
                config.join(app).join(leaf),
                format!("{app} · {leaf}"),
                Category::Caches,
                Safety::Safe,
                "应用生成的缓存，关闭应用后可清理，之后按需重建。",
            ));
        }
    }
    result
}
