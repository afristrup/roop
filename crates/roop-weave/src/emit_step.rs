use crate::{Model, Tensor, decl, names, param_list};

fn clear(weight: &Tensor) -> String {
    match weight.dims.as_slice() {
        [rows, cols] => format!("    call clear_mat<{cols}, {rows}>(g{});\n", weight.name),
        [len] => format!("    call clear<{len}>(g{});\n", weight.name),
        _ => unreachable!("parameters are vectors or matrices"),
    }
}

/// `<name>_step<B>`: one step of the optimizer on a batch of B samples. The
/// gradient is reversible; the clearing of buffers between samples is not.
pub fn emit_step(model: &Model) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let mut params = vec!["total: &mut i64".to_string()];
    params.extend(model.tensors().into_iter().map(|t| decl(t, true)));
    let grads = model.gradients();
    params.extend(grads.iter().map(|g| decl(g, true)));
    let state = model.optimizer_state();
    params.extend(state.iter().map(|s| decl(s, true)));
    params.extend(
        ["q", "p", "aq", "ap"]
            .iter()
            .map(|s| format!("{s}: &mut [i64; {n}]")),
    );
    params.push(format!("xs: &[[i64; {n}]; B]"));
    params.push(format!("ts: &[[i64; {k}]; B]"));
    params.push("lr: &i64".into());
    let weights = names(&model.tensors()).join(", ");
    let gradients = names(&grads).join(", ");
    let updates: String = model
        .tensors()
        .into_iter()
        .map(|t| model.optimizer.update(t))
        .collect();
    let clears: String = model.tensors().into_iter().map(clear).collect();
    format!(
        "pub irrev fn {name}_step<B>(\n{}) {{
    ancilla n: i64 = 0 {{
        from n == 0 {{
            call add_vec<{n}>(q, xs[n]);
            call {name}_grad(total, q, p, aq, ap, {gradients}, {weights}, ts[n]);
            uncall add_vec<{n}>(q, xs[n]);
            call clear<{n}>(aq);
            call clear<{n}>(ap);
        }} loop {{ n += 1; }} until n == B - 1;
        n -= B - 1;
    }}
{updates}{clears}}}
",
        param_list(&params)
    )
}
