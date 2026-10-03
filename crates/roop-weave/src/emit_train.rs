use crate::{Model, decl, names, param_list};

/// `<name>_train`: `<name>_step` for a batch of `batch` samples, with the length
/// filled in so that it can be called from C.
pub fn emit_train(model: &Model, batch: usize) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let mut params = vec!["total: &mut i64".to_string()];
    params.extend(model.tensors().into_iter().map(|t| decl(t, true)));
    let grads = model.gradients();
    params.extend(grads.iter().map(|g| decl(g, true)));
    params.extend(
        ["q", "p", "aq", "ap"]
            .iter()
            .map(|s| format!("{s}: &mut [i64; {n}]")),
    );
    params.push(format!("xs: &[[i64; {n}]; {batch}]"));
    params.push(format!("ts: &[[i64; {k}]; {batch}]"));
    params.push("lr: &i64".into());
    let all: Vec<String> = [names(&model.tensors()), names(&grads)].concat();
    format!(
        "pub irrev fn {name}_train(\n{}) {{\n    call {name}_step<{batch}>(total, {}, q, p, aq, ap, xs, ts, lr);\n}}\n",
        param_list(&params),
        all.join(", ")
    )
}
