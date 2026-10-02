use crate::{ModuleError, Tree, global_name, item_name};
use roop_syntax::UseDecl;

/// The module and global name a `use` refers to. Only `pub` items can be
/// imported.
pub fn resolve_use(
    tree: &Tree,
    from: usize,
    decl: &UseDecl,
) -> Result<(usize, String), ModuleError> {
    let here = &tree.modules[from];
    let shown = || {
        global_name(&here.path, "")
            .trim_end_matches('_')
            .to_string()
    };
    let Some((item, modules)) = decl.path.split_last().filter(|(_, m)| !m.is_empty()) else {
        return Err(ModuleError::EmptyUse(shown()));
    };
    let start = here
        .children
        .get(&modules[0])
        .or_else(|| tree.roots.get(&modules[0]))
        .ok_or_else(|| ModuleError::UnknownModule(modules[0].clone()))?;
    let mut target = *start;
    for name in &modules[1..] {
        target = *tree.modules[target]
            .children
            .get(name)
            .ok_or_else(|| ModuleError::UnknownModule(name.clone()))?;
    }
    let module = &tree.modules[target];
    let label = module.path.join("::");
    let (_, public) = module
        .items
        .iter()
        .filter_map(item_name)
        .find(|(name, _)| name == item)
        .ok_or_else(|| ModuleError::UnknownItem {
            module: label.clone(),
            name: item.clone(),
        })?;
    if !public {
        return Err(ModuleError::Private {
            module: label,
            name: item.clone(),
        });
    }
    Ok((target, global_name(&module.path, item)))
}
