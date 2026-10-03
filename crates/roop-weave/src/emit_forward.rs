use crate::{Model, batch_suffix, decl, layer_call, param_list, state_type};

/// `<name>_forward`: the network, run on (q, p). Batched, `<name>_forward_batch<B>`
/// runs it on `B` samples at once.
pub fn emit_forward(model: &Model, batched: bool) -> String {
    let n = model.width;
    let state = state_type(n, batched);
    let mut params = vec![format!("q: &mut {state}"), format!("p: &mut {state}")];
    params.extend(model.tensors().into_iter().map(|t| decl(t, false)));
    let calls: String = model
        .layers
        .iter()
        .zip(model.adds_into_q())
        .map(|(layer, into_q)| {
            let call = layer_call(layer, into_q, n, model.step, false, batched);
            format!("    {call}\n")
        })
        .collect();
    let generics = if batched { "<B>" } else { "" };
    format!(
        "pub fn {}_forward{}{generics}(\n{}) {{\n{calls}}}\n",
        model.name,
        batch_suffix(batched),
        param_list(&params)
    )
}
