use crate::{ModuleError, ModuleScope, Tree, imports_of, item_global, item_name};
use roop_syntax::{Item, UseShape};
use std::collections::HashMap;

/// Local name to global name for everything a module can mention: its own
/// definitions, what it imports by name, and what globs bring in where nothing
/// else has the name.
pub fn build_scope(tree: &Tree, id: usize) -> Result<ModuleScope, ModuleError> {
    let module = &tree.modules[id];
    let label = module.path.join("::");
    let mut names: HashMap<String, String> = HashMap::new();
    let mut deps = Vec::new();
    let mut imports = Vec::new();
    let bind = |names: &mut HashMap<String, String>, local: String, global: String| match names
        .insert(local.clone(), global.clone())
    {
        Some(old) if old != global => Err(ModuleError::Duplicate {
            module: label.clone(),
            name: local,
        }),
        _ => Ok(()),
    };
    for item in &module.items {
        if let Some((name, _)) = item_name(item) {
            bind(
                &mut names,
                name.to_string(),
                item_global(&module.path, item, name),
            )?;
        }
    }
    let mut globs = Vec::new();
    for item in &module.items {
        let Item::Use(decl) = item else { continue };
        for import in imports_of(tree, id, decl, 0)? {
            deps.push(import.from);
            imports.push(import.global.clone());
            if decl.shape == UseShape::Glob {
                globs.push(import);
            } else {
                bind(&mut names, import.local, import.global)?;
            }
        }
    }
    for import in globs {
        names.entry(import.local).or_insert(import.global);
    }
    Ok(ModuleScope {
        names,
        deps,
        imports,
    })
}
