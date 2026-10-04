use crate::{Kept, Model};

/// The calls that scale the weights of each residual block that has `keep_contraction`
/// back to its bound, after the optimizer has moved them.
pub fn emit_project(model: &Model) -> String {
    let project = |k: Kept| {
        format!(
            "    call project_contraction<{}, {}>({}, {}, {}, {});\n",
            k.n, k.m, k.w1, k.w2, k.slope, k.cap
        )
    };
    Kept::of(model).into_iter().map(project).collect()
}
