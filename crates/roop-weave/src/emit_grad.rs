use crate::{Model, decl, names, param_list};

/// `<name>_grad`: the loss of one sample and its gradients.
pub fn emit_grad(model: &Model) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let mut params = vec!["total: &mut i64".to_string()];
    params.extend(
        ["q", "p", "aq", "ap"]
            .iter()
            .map(|s| format!("{s}: &mut [i64; {n}]")),
    );
    let grads = model.gradients();
    params.extend(grads.iter().map(|g| decl(g, true)));
    params.extend(model.tensors().into_iter().map(|t| decl(t, false)));
    params.push(format!("t: &[i64; {k}]"));
    let weights = names(&model.tensors()).join(", ");
    let gradients = names(&grads).join(", ");
    format!(
        "pub fn {name}_grad(\n{}) {{
    call {name}_forward(q, p, {weights});
    call loss<{n}, {k}>(total, q, t);
    call seed<{n}, {k}>(aq, q, t);
    call {name}_backward(q, p, aq, ap, {gradients}, {weights});
}}
",
        param_list(&params)
    )
}
