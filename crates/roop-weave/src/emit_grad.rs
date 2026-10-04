use crate::{LossKind, Model, batch_suffix, decl, names, param_list, state_type};

/// `<name>_grad`: the loss of one sample and its gradients. Batched,
/// `<name>_grad_batch<B>` is the loss of `B` samples and the sum of their gradients.
pub fn emit_grad(model: &Model, batched: bool) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let (s, generics) = (batch_suffix(batched), if batched { "<B>" } else { "" });
    let mut params = vec!["total: &mut i64".to_string()];
    params.extend(
        ["q", "p", "aq", "ap"]
            .iter()
            .map(|v| format!("{v}: &mut {}", state_type(n, batched))),
    );
    let grads = model.gradients();
    params.extend(grads.iter().map(|g| decl(g, true)));
    params.extend(model.tensors().into_iter().map(|t| decl(t, false)));
    params.push(match batched {
        true => format!("t: &[[i64; {k}]; B]"),
        false => format!("t: &[i64; {k}]"),
    });
    let weights = names(&model.tensors()).join(", ");
    let gradients = names(&grads).join(", ");
    let extra = if batched { ", B" } else { "" };
    let scale = model.loss_scale;
    let seed = match model.loss {
        LossKind::Mse => format!(
            "call loss{s}<{n}, {k}{extra}>(total, q, t);\n    call seed_scaled{s}<{n}, {k}{extra}>(aq, q, t, {scale});"
        ),
        LossKind::Sigmoid => {
            format!("call seed_sigmoid{s}<{n}, {k}{extra}>(aq, total, q, t, {scale});")
        }
        LossKind::Softmax => {
            format!("call seed_softmax{s}<{n}, {k}{extra}>(aq, total, q, t, {scale});")
        }
    };
    format!(
        "pub fn {name}_grad{s}{generics}(\n{}) {{
    call {name}_forward{s}(q, p, {weights});
    {seed}
    call {name}_backward{s}(q, p, aq, ap, {gradients}, {weights});
}}
",
        param_list(&params)
    )
}
