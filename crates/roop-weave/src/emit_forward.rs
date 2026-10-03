use crate::{Model, decl, layer_call, param_list};

/// `<name>_forward`: the network, run on (q, p).
pub fn emit_forward(model: &Model) -> String {
    let n = model.width;
    let mut params = vec![format!("q: &mut [i64; {n}]"), format!("p: &mut [i64; {n}]")];
    params.extend(model.tensors().into_iter().map(|t| decl(t, false)));
    let calls: String = model
        .layers
        .iter()
        .zip(model.adds_into_q())
        .map(|(layer, into_q)| format!("    {}\n", layer_call(layer, into_q, n, model.step, false)))
        .collect();
    format!(
        "pub fn {}_forward(\n{}) {{\n{calls}}}\n",
        model.name,
        param_list(&params)
    )
}
