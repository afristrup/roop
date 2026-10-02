use crate::{ModuleError, Tree};

/// The module a path names, starting from `from`: a child of it, a
/// `Roop.toml` root, or `super` for its parent.
pub fn module_at(tree: &Tree, from: usize, path: &[String]) -> Result<usize, ModuleError> {
    let here = &tree.modules[from];
    let Some((first, rest)) = path.split_first() else {
        return Ok(from);
    };
    let mut target = if first == "super" {
        here.parent
            .ok_or_else(|| ModuleError::NoParent(here.path.join("::")))?
    } else {
        *here
            .children
            .get(first)
            .or_else(|| tree.roots.get(first))
            .ok_or_else(|| ModuleError::UnknownModule(first.clone()))?
    };
    for name in rest {
        target = if name == "super" {
            tree.modules[target]
                .parent
                .ok_or_else(|| ModuleError::NoParent(tree.modules[target].path.join("::")))?
        } else {
            *tree.modules[target]
                .children
                .get(name)
                .ok_or_else(|| ModuleError::UnknownModule(name.clone()))?
        };
    }
    Ok(target)
}
