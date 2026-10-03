use crate::{Dir, FnGen};

/// The scale a kernel is called with: `alpha` forward, `-alpha` backward, so
/// the same kernel that adds a product also takes it off.
pub fn signed_factor(g: &mut FnGen, alpha: &str, dir: Dir) -> String {
    match dir {
        Dir::Forward => alpha.to_string(),
        Dir::Backward => {
            let negated = format!("%{}", g.fresh("t"));
            g.emit(&format!("{negated} = fneg double {alpha}"));
            negated
        }
    }
}
