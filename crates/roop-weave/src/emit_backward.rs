use crate::{Model, decl, layer_call, param_list};

/// `<name>_backward`: the adjoints, from the output state back to the input.
pub fn emit_backward(model: &Model) -> String {
    let n = model.width;
    let state = |name: &str| format!("{name}: &mut [i64; {n}]");
    let mut params: Vec<String> = ["q", "p", "aq", "ap"].iter().map(|s| state(s)).collect();
    params.extend(model.gradients().iter().map(|g| decl(g, true)));
    params.extend(model.tensors().into_iter().map(|t| decl(t, false)));
    let calls: String = model
        .layers
        .iter()
        .zip(model.adds_into_q())
        .rev()
        .map(|(layer, into_q)| format!("    {}\n", layer_call(layer, into_q, n, model.step, true)))
        .collect();
    format!(
        "pub fn {}_backward(\n{}) {{\n{calls}}}\n",
        model.name,
        param_list(&params)
    )
}
