use crate::{
    ModuleError, Tree, build_scope, global_name, item_name, lookup, order, prune, rename_names,
};
use roop_syntax::{Item, Program};

/// One program from a module tree: names made global, `mod` and `use` gone,
/// library modules pruned to what is reached.
pub fn flatten(tree: &Tree) -> Result<Program, ModuleError> {
    let built: Vec<_> = (0..tree.modules.len())
        .map(|id| build_scope(tree, id))
        .collect::<Result<_, _>>()?;
    let deps: Vec<Vec<usize>> = built.iter().map(|(_, d)| d.clone()).collect();
    let mut library = Vec::new();
    let mut entry = Vec::new();
    for id in order(tree, &deps) {
        let module = &tree.modules[id];
        let scope = &built[id].0;
        let target = if id == tree.entry {
            &mut entry
        } else {
            &mut library
        };
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
            set_name(&mut item, global_name(&module.path, name));
            target.push(item);
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
        Item::Mod(_) | Item::Use(_) => {}
    }
}
