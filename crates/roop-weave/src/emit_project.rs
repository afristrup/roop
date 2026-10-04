use crate::{Kept, Model};

/// The calls that scale the weights of each residual block that has `keep_contraction`
/// back to its bound, after the optimizer has moved them.
pub fn emit_project(model: &Model) -> String {
    let project = |k: Kept| {
        let (n, m, slope, cap) = (k.n, k.m, k.slope, k.cap);
        let (w1, w2) = (&k.w1, &k.w2);
        let step =
            format!("call project_step<{n}, {m}>({w1}, {w2}, old{w1}, old{w2}, {slope}, {cap});");
        let scale = format!("call project_contraction<{n}, {m}>({w1}, {w2}, {slope}, {cap});");
        format!("    {step}\n    {scale}\n")
    };
    Kept::of(model).into_iter().map(project).collect()
}
