use crate::{
    ModuleError, Tree, build_scope, item_global, item_name, lookup, order, prune, rename_names,
};
use roop_syntax::{Item, Program};
use std::collections::HashSet;

/// One program from a module tree: names made global, `mod` and `use` gone,
/// library modules pruned to what is reached.
pub fn flatten(tree: &Tree) -> Result<Program, ModuleError> {
    let built: Vec<_> = (0..tree.modules.len())
        .map(|id| build_scope(tree, id))
        .collect::<Result<_, _>>()?;
    let deps: Vec<Vec<usize>> = built.iter().map(|s| s.deps.clone()).collect();
    let imported: HashSet<&String> = built.iter().flat_map(|s| &s.imports).collect();
    let mut library = Vec::new();
    let mut entry = Vec::new();
    for id in order(tree, &deps) {
        let module = &tree.modules[id];
        let scope = &built[id].names;
        for item in &module.items {
            let Some((name, _)) = item_name(item) else {
                continue;
            };
            let mut item = item.clone();
            rename_names(&mut item, &mut |n| {
                if let Some(global) = lookup(scope, n) {
                    *n = global;
                }
            });
            let global = item_global(&module.path, &item, name);
            set_name(&mut item, global.clone());
            // A session names no code that would keep it, so an import does.
            let kept = id == tree.entry
                || (matches!(item, Item::Session(_)) && imported.contains(&global));
            if kept {
                entry.push(item);
            } else {
                library.push(item);
            }
        }
    }
    let keep = entry.len();
    let kept = prune(entry.into_iter().chain(library).collect(), keep);
    let (entry, library) = kept.split_at(keep);
    Ok(Program {
        items: library.iter().chain(entry).cloned().collect(),
    })
}

fn set_name(item: &mut Item, name: String) {
    match item {
        Item::Fn(f) => f.name = name,
        Item::Struct(s) => s.name = name,
        Item::Enum(e) => e.name = name,
        Item::Session(s) => s.name = name,
        Item::Mod(_) | Item::Use(_) => {}
    }
}
