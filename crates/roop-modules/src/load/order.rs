use crate::Tree;

/// Modules in dependency order: a module comes after every module it imports
/// from or declares, and the entry module is last.
pub fn order(tree: &Tree, deps: &[Vec<usize>]) -> Vec<usize> {
    fn visit(
        id: usize,
        tree: &Tree,
        deps: &[Vec<usize>],
        seen: &mut Vec<bool>,
        out: &mut Vec<usize>,
    ) {
        if std::mem::replace(&mut seen[id], true) {
            return;
        }
        let children = tree.modules[id].children.values();
        for &next in children.chain(&deps[id]) {
            visit(next, tree, deps, seen, out);
        }
        out.push(id);
    }
    let mut seen = vec![false; tree.modules.len()];
    let mut out = Vec::new();
    visit(tree.entry, tree, deps, &mut seen, &mut out);
    out
}
