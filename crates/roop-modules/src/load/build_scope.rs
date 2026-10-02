use crate::{ModuleError, Tree, global_name, item_name, resolve_use};
use roop_syntax::Item;
use std::collections::HashMap;

/// Local name to global name for everything a module can mention: its own
/// definitions and what it imports. Also returns the modules it imports from.
pub fn build_scope(
    tree: &Tree,
    id: usize,
) -> Result<(HashMap<String, String>, Vec<usize>), ModuleError> {
    let module = &tree.modules[id];
    let label = module.path.join("::");
    let mut scope = HashMap::new();
    let mut deps = Vec::new();
    let mut bind = |local: String, global: String| match scope.insert(local.clone(), global) {
        Some(_) => Err(ModuleError::Duplicate {
            module: label.clone(),
            name: local,
        }),
        None => Ok(()),
    };
    for item in &module.items {
        if let Some((name, _)) = item_name(item) {
            bind(name.to_string(), global_name(&module.path, name))?;
        }
        if let Item::Use(decl) = item {
            let (target, global) = resolve_use(tree, id, decl)?;
            let local = decl
                .alias
                .clone()
                .unwrap_or_else(|| decl.path.last().cloned().unwrap_or_default());
            bind(local, global)?;
            deps.push(target);
        }
    }
    Ok((scope, deps))
}
