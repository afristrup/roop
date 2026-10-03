use crate::{item_name, lookup, rename_names};
use roop_syntax::Item;
use std::collections::{HashMap, HashSet};

/// Keeps the first `keep` items and whatever they reach, so a program only
/// carries the library code it uses.
pub fn prune(items: Vec<Item>, keep: usize) -> Vec<Item> {
    let index: HashMap<String, String> = items
        .iter()
        .filter_map(item_name)
        .map(|(name, _)| (name.to_string(), name.to_string()))
        .collect();
    let by_name: HashMap<&str, usize> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| item_name(item).map(|(name, _)| (name, i)))
        .collect();
    let mut reached: HashSet<usize> = (0..keep).collect();
    let mut work: Vec<usize> = (0..keep).collect();
    while let Some(i) = work.pop() {
        let mut item = items[i].clone();
        rename_names(&mut item, &mut |name| {
            let Some(target) = lookup(&index, name) else {
                return;
            };
            if let Some(&j) = by_name.get(target.as_str())
                && reached.insert(j)
            {
                work.push(j);
            }
        });
    }
    items
        .into_iter()
        .enumerate()
        .filter(|(i, _)| reached.contains(i))
        .map(|(_, item)| item)
        .collect()
}
