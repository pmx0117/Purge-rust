use super::measure;
use purge_core::{Category, Item};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
/// Same-size candidates are hashed, then compared byte for byte. This never
/// changes selection and never deletes a candidate.
pub fn find_duplicates(items: &[Item], cancel: &AtomicBool) -> HashMap<u64, u64> {
    let by_size = size_groups(items);
    let mut result = HashMap::new();
    for candidates in by_size.values().filter(|items| items.len() > 1) {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let groups = hash_groups(candidates, cancel);
        record_matches(&groups, cancel, &mut result);
    }
    result
}
fn size_groups(items: &[Item]) -> HashMap<u64, Vec<&Item>> {
    let mut groups = HashMap::new();
    for item in items.iter().filter(|item| duplicate_eligible(item)) {
        groups
            .entry(item.bytes())
            .or_insert_with(Vec::new)
            .push(item);
    }
    groups
}
fn duplicate_eligible(item: &Item) -> bool {
    item.category() == Category::LargeFiles && item.cleanable()
}
fn hash_groups<'a>(
    candidates: &[&'a Item],
    cancel: &AtomicBool,
) -> HashMap<Vec<u8>, Vec<&'a Item>> {
    let mut groups = HashMap::new();
    for item in candidates {
        if let Some(digest) = item_digest(item, cancel) {
            groups.entry(digest).or_insert_with(Vec::new).push(*item);
        }
    }
    groups
}
fn item_digest(item: &Item, cancel: &AtomicBool) -> Option<Vec<u8>> {
    if measure(&item.path, cancel) != item.footprint {
        return None;
    }
    let mut file = fs::File::open(&item.path).ok()?;
    let digest = read_digest(&mut file, cancel)?;
    (measure(&item.path, cancel) == item.footprint).then_some(digest)
}
fn read_digest(file: &mut impl Read, cancel: &AtomicBool) -> Option<Vec<u8>> {
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return None;
        }
        match file.read(&mut buffer) {
            Ok(0) => return Some(digest.finalize().to_vec()),
            Ok(count) => digest.update(&buffer[..count]),
            Err(_) => return None,
        }
    }
}
fn record_matches(
    groups: &HashMap<Vec<u8>, Vec<&Item>>,
    cancel: &AtomicBool,
    result: &mut HashMap<u64, u64>,
) {
    for group in groups.values().filter(|items| items.len() > 1) {
        let keeper = group[0];
        for item in group.iter().skip(1) {
            if files_equal(&keeper.path, &item.path, cancel) {
                result.insert(keeper.id, keeper.id);
                result.insert(item.id, keeper.id);
            }
        }
    }
}
fn files_equal(first: &Path, second: &Path, cancel: &AtomicBool) -> bool {
    let (Ok(mut a), Ok(mut b)) = (fs::File::open(first), fs::File::open(second)) else {
        return false;
    };
    let mut left = [0; 64 * 1024];
    let mut right = [0; 64 * 1024];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return false;
        }
        let (Ok(count), Ok(other)) = (a.read(&mut left), b.read(&mut right)) else {
            return false;
        };
        if count != other || left[..count] != right[..other] {
            return false;
        }
        if count == 0 {
            return true;
        }
    }
}
