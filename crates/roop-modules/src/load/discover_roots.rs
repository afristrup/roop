use crate::{Loader, ModuleError};
use roop_syntax::Item;

/// Loads every `Roop.toml` root that some `use` starts from, including the
/// roots those modules need in turn.
pub fn discover_roots(loader: &mut Loader) -> Result<(), ModuleError> {
    let mut next = 0;
    while next < loader.tree.modules.len() {
        let module = &loader.tree.modules[next];
        let wanted: Vec<String> = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Use(u) => u.path.first().cloned(),
                _ => None,
            })
            .filter(|first| !module.children.contains_key(first))
            .collect();
        for name in wanted {
            loader.load_root(&name)?;
        }
        next += 1;
    }
    Ok(())
}
