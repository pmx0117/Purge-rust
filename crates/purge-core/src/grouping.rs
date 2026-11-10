use crate::{Category, Item};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
pub fn path_id(path: &std::path::Path) -> u64 {
    let mut hash = DefaultHasher::new();
    path.to_string_lossy().to_lowercase().hash(&mut hash);
    hash.finish()
}

#[derive(Debug, Clone)]
pub struct ItemGroup {
    pub id: u64,
    pub name: String,
    pub items: Vec<Item>,
}
impl ItemGroup {
    pub fn bytes(&self) -> u64 {
        self.items.iter().map(Item::bytes).sum()
    }
}

/// Like Purge's merged cache rows, a displayed app owns all of its cache
/// locations. The original candidates stay intact for per-path revalidation.
pub fn group_items(items: Vec<Item>) -> Vec<ItemGroup> {
    let mut groups: Vec<ItemGroup> = vec![];
    let mut index = std::collections::HashMap::new();
    for item in items {
        let name = if item.category() == Category::Caches {
            item.name
                .split('·')
                .next()
                .unwrap_or(&item.name)
                .trim()
                .to_string()
        } else {
            item.name.clone()
        };
        let key = if item.category() == Category::Caches {
            format!("cache-group:{name}")
        } else {
            format!("item:{}", item.id)
        };
        let group_index = *index.entry(key.clone()).or_insert_with(|| {
            let group_index = groups.len();
            groups.push(ItemGroup {
                id: path_id(std::path::Path::new(&key)),
                name,
                items: vec![],
            });
            group_index
        });
        groups[group_index].items.push(item);
    }
    groups
}
