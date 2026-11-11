use crate::platform;
use purge_core::{Category, Safety};
use std::{fs, path::PathBuf, time::Duration};
#[path = "catalog_unix.rs"]
mod unix;

/// Safety margin for temporary files and crash dumps, independent of projects.
pub const TEMPORARY_RETENTION_HOURS: u64 = 24;
#[derive(Clone, Debug)]
pub struct Rule {
    pub path: PathBuf,
    pub name: String,
    pub description: String,
    pub category: Category,
    pub safety: Safety,
    pub children: bool,
    pub minimum_age: Duration,
}

pub fn cache_rules() -> Vec<Rule> {
    discover_cache_rules_for(
        platform::OperatingSystem::current(),
        &platform::home_dir(),
        &platform::local_dir(),
        &platform::roaming_dir(),
    )
}
pub fn discover_cache_rules_for(
    system: platform::OperatingSystem,
    home: &std::path::Path,
    local: &std::path::Path,
    roaming: &std::path::Path,
) -> Vec<Rule> {
    if system != platform::OperatingSystem::Windows {
        return unix::rules(system, home, local, roaming);
    }
    discover_cache_rules(home, local, roaming)
}
pub fn discover_cache_rules(
    home: &std::path::Path,
    local: &std::path::Path,
    roaming: &std::path::Path,
) -> Vec<Rule> {
    let mut rules = browser_rules(local);
    rules.extend(application_rules(roaming));
    rules.extend(developer_rules(home, local));
    rules.extend(temporary_rules(local));
    rules
}
fn cache_rule(
    path: PathBuf,
    name: String,
    category: Category,
    safety: Safety,
    description: &str,
) -> Rule {
    Rule {
        path,
        name,
        category,
        safety,
        description: description.into(),
        children: false,
        minimum_age: Duration::ZERO,
    }
}
fn browser_rules(local: &std::path::Path) -> Vec<Rule> {
    let mut rules = vec![];
    for (base, name) in [
        ("Google/Chrome/User Data", "Google Chrome"),
        ("Microsoft/Edge/User Data", "Microsoft Edge"),
        ("BraveSoftware/Brave-Browser/User Data", "Brave"),
        ("Vivaldi/User Data", "Vivaldi"),
    ] {
        rules.extend(profile_rules(&local.join(base), name));
    }
    rules
}
fn profile_rules(root: &std::path::Path, name: &str) -> Vec<Rule> {
    fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(is_user_profile)
        .flat_map(|profile| {
            profile_caches(
                profile.path(),
                name,
                profile.file_name().to_string_lossy().as_ref(),
            )
        })
        .collect()
}
fn is_user_profile(entry: &fs::DirEntry) -> bool {
    let name = entry.file_name();
    let name = name.to_string_lossy();
    name == "Default" || name.starts_with("Profile ")
}
fn profile_caches(path: PathBuf, name: &str, profile: &str) -> Vec<Rule> {
    ["Cache", "Code Cache", "GPUCache"]
        .into_iter()
        .map(|leaf| {
            cache_rule(
                path.join(leaf),
                format!("{name} · {profile} · {leaf}"),
                Category::Caches,
                Safety::Safe,
                "请先关闭浏览器。清理后缓存会重建；密码、书签和浏览记录会保留。",
            )
        })
        .collect()
}
fn application_rules(roaming: &std::path::Path) -> Vec<Rule> {
    let mut rules = vec![];
    for app in [
        "Code",
        "Cursor",
        "discord",
        "Slack",
        "Notion",
        "Microsoft/Teams",
    ] {
        for leaf in ["Cache", "Code Cache", "GPUCache"] {
            rules.push(cache_rule(
                roaming.join(app).join(leaf),
                format!("{} · {leaf}", app.rsplit('/').next().unwrap_or(app)),
                Category::Caches,
                Safety::Safe,
                "应用生成的缓存，下次启动会重建。请先关闭对应应用。",
            ));
        }
    }
    rules
}
fn developer_rules(home: &std::path::Path, local: &std::path::Path) -> Vec<Rule> {
    [
        (
            local.join("npm-cache/_cacache"),
            "npm",
            "npm 下载缓存。下次安装依赖时会重新下载。",
        ),
        (
            local.join("pnpm/store"),
            "pnpm store",
            "pnpm 的共享依赖仓库，可能影响离线安装。请先关闭包管理器。",
        ),
        (
            local.join("Yarn/Cache"),
            "Yarn",
            "Yarn 包下载缓存，之后按需重新下载。",
        ),
        (
            local.join("pip/Cache"),
            "pip",
            "Python 安装包下载和 wheel 缓存，不会删除虚拟环境。",
        ),
        (
            local.join("uv/cache"),
            "uv",
            "uv 共享依赖缓存，可能影响离线安装。",
        ),
        (
            home.join(".cargo/registry/cache"),
            "Cargo 下载缓存",
            "已下载的 crate 压缩包。不会删除工具链、已安装命令或凭据。",
        ),
        (
            home.join(".gradle/caches"),
            "Gradle",
            "Gradle 构建缓存，下一次构建需要重新下载或生成。",
        ),
        (
            home.join(".nuget/packages"),
            "NuGet",
            "NuGet 共享包目录。可能影响离线构建，请确认后清理。",
        ),
    ]
    .into_iter()
    .map(|(path, name, description)| {
        cache_rule(
            path,
            name.into(),
            Category::Developer,
            tool_safety(name),
            description,
        )
    })
    .collect()
}
fn tool_safety(name: &str) -> Safety {
    match name {
        "NuGet" | "pnpm store" | "uv" => Safety::Review,
        _ => Safety::Safe,
    }
}
fn temporary_rules(local: &std::path::Path) -> Vec<Rule> {
    [("Temp", "临时文件"), ("CrashDumps", "应用崩溃报告")]
        .into_iter()
        .map(|(directory, name)| Rule {
            path: local.join(directory),
            name: name.into(),
            category: Category::Caches,
            safety: Safety::Safe,
            description: format!("仅列出至少 {TEMPORARY_RETENTION_HOURS} 小时未修改的临时条目；不受开发项目闲置时间影响。"),
            children: true,
            minimum_age: Duration::from_secs(TEMPORARY_RETENTION_HOURS * 60 * 60),
        })
        .collect()
}
