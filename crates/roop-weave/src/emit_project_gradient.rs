use crate::{Kept, Model};

/// The calls that remove, from the gradients of the weights of each residual block that
/// has `keep_contraction`, the part that pushes them out of the bound, before the
/// optimizer sees them.
pub fn emit_project_gradient(model: &Model) -> String {
    let project = |k: Kept| {
        format!(
            "    call project_gradient<{}, {}>(g{}, g{}, {}, {}, {}, {});\n",
            k.n, k.m, k.w1, k.w2, k.w1, k.w2, k.slope, k.cap
        )
    };
    Kept::of(model).into_iter().map(project).collect()
}
