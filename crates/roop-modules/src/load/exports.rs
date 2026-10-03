use crate::{Import, ModuleError, Tree, imports_of, item_global, item_name};
use roop_syntax::Item;

/// How deep re-exports may chain before they are taken to be a cycle.
pub const MAX_REEXPORT_DEPTH: usize = 16;

/// Everything a module offers to others: its `pub` definitions, and what it
/// re-exports with `pub use`.
pub fn exports(tree: &Tree, id: usize, depth: usize) -> Result<Vec<Import>, ModuleError> {
    let module = &tree.modules[id];
    if depth > MAX_REEXPORT_DEPTH {
        return Err(ModuleError::ImportCycle(module.path.join("::")));
    }
    let mut offered: Vec<Import> = module
        .items
        .iter()
        .filter_map(|item| item_name(item).map(|n| (item, n)))
        .filter(|(_, (_, public))| *public)
        .map(|(item, (name, _))| Import {
            local: name.to_string(),
            global: item_global(&module.path, item, name),
            from: id,
        })
        .collect();
    for item in &module.items {
        if let Item::Use(decl) = item
            && decl.public
        {
            offered.extend(imports_of(tree, id, decl, depth + 1)?);
        }
    }
    Ok(offered)
}
