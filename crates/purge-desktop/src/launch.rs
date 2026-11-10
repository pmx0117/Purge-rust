use crate::model::Page;
use purge_core::Category;
#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    pub theme: Option<bool>,
    pub page: Option<Page>,
}
impl LaunchOptions {
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Self {
        let mut result = Self::default();
        for argument in arguments {
            result.theme = parse_theme(&argument).or(result.theme);
            result.page = parse_page(&argument).or(result.page);
        }
        result
    }
}
fn parse_theme(argument: &str) -> Option<bool> {
    match argument {
        "--theme=dark" => Some(true),
        "--theme=light" => Some(false),
        _ => None,
    }
}
const PAGE_ARGUMENTS: &[(&str, Page)] = &[
    ("--page=overview", Page::Overview),
    ("--page=caches", Page::Category(Category::Caches)),
    ("--page=developer", Page::Category(Category::Developer)),
    ("--page=large-files", Page::Category(Category::LargeFiles)),
    ("--page=apps", Page::Category(Category::Applications)),
    ("--page=settings", Page::Settings),
    ("--page=history", Page::History),
    ("--page=about", Page::About),
];
fn parse_page(argument: &str) -> Option<Page> {
    PAGE_ARGUMENTS
        .iter()
        .find(|entry| entry.0 == argument)
        .map(|entry| entry.1)
}
