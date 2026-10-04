use crate::{Kept, Model};

/// The calls that remove, from the gradients of the weights of each residual block that
/// has `keep_contraction`, the part that pushes them out of the bound, and that keep
/// the weights as they were for the projection of the step that follows.
pub fn emit_project_gradient(model: &Model) -> String {
    let project = |k: Kept| {
        let (n, m, slope, cap) = (k.n, k.m, k.slope, k.cap);
        let (w1, w2) = (&k.w1, &k.w2);
        let filter =
            format!("call project_gradient<{n}, {m}>(g{w1}, g{w2}, {w1}, {w2}, {slope}, {cap});");
        format!(
            "    {filter}
    ancilla old{w1}: [[i64; {n}]; {m}] = 0;
    ancilla old{w2}: [[i64; {m}]; {n}] = 0;
    call save<{n}, {m}>(old{w1}, {w1});
    call save<{m}, {n}>(old{w2}, {w2});
"
        )
    };
    Kept::of(model).into_iter().map(project).collect()
}
