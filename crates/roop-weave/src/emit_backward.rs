use crate::{Model, batch_suffix, decl, layer_call, param_list, state_type};

/// `<name>_backward`: the adjoints, from the output state back to the input.
/// Batched, `<name>_backward_batch<B>` does it for `B` samples at once.
pub fn emit_backward(model: &Model, batched: bool) -> String {
    let n = model.width;
    let state = |name: &str| format!("{name}: &mut {}", state_type(n, batched));
    let mut params: Vec<String> = ["q", "p", "aq", "ap"].iter().map(|s| state(s)).collect();
    params.extend(model.gradients().iter().map(|g| decl(g, true)));
    params.extend(model.tensors().into_iter().map(|t| decl(t, false)));
    let calls: String = model
        .layers
        .iter()
        .zip(model.adds_into_q())
        .rev()
        .map(|(layer, into_q)| {
            let call = layer_call(layer, into_q, n, model.step, true, batched);
            format!("    {call}\n")
        })
        .collect();
    let generics = if batched { "<B>" } else { "" };
    format!(
        "pub fn {}_backward{}{generics}(\n{}) {{\n{calls}}}\n",
        model.name,
        batch_suffix(batched),
        param_list(&params)
    )
}
