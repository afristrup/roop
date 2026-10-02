use crate::{Import, ModuleError, Tree, exports, item_name, module_at};
use roop_syntax::{UseDecl, UseShape};

/// The names a `use` brings in. Only `pub` items can be imported, and a glob
/// brings in all of them.
pub fn imports_of(
    tree: &Tree,
    from: usize,
    decl: &UseDecl,
    depth: usize,
) -> Result<Vec<Import>, ModuleError> {
    let here = tree.modules[from].path.join("::");
    let pick = |module: usize, name: &str, alias: &Option<String>| {
        let offered = exports(tree, module, depth)?;
        match offered.into_iter().find(|i| i.local == name) {
            Some(found) => Ok(Import {
                local: alias.clone().unwrap_or_else(|| name.to_string()),
                ..found
            }),
            None => {
                let target = &tree.modules[module];
                let label = target.path.join("::");
                let exists = target
                    .items
                    .iter()
                    .filter_map(item_name)
                    .any(|(n, _)| n == name);
                Err(if exists {
                    ModuleError::Private {
                        module: label,
                        name: name.to_string(),
                    }
                } else {
                    ModuleError::UnknownItem {
                        module: label,
                        name: name.to_string(),
                    }
                })
            }
        }
    };
    match &decl.shape {
        UseShape::Single => {
            let Some((item, modules)) = decl.path.split_last().filter(|(_, m)| !m.is_empty())
            else {
                return Err(ModuleError::EmptyUse(here));
            };
            let target = module_at(tree, from, modules)?;
            Ok(vec![pick(target, item, &decl.alias)?])
        }
        UseShape::Glob => {
            let target = module_at(tree, from, &decl.path)?;
            exports(tree, target, depth)
        }
        UseShape::Group(names) => {
            let target = module_at(tree, from, &decl.path)?;
            names
                .iter()
                .map(|(name, alias)| pick(target, name, alias))
                .collect()
        }
    }
}
