use crate::{Model, decl, param_list, quantize};

/// `<name>_load`: adds the parameters, on the 1/4096 grid, into zeros.
pub fn emit_load(model: &Model) -> String {
    let params: Vec<String> = model.tensors().into_iter().map(|t| decl(t, true)).collect();
    let mut body = String::new();
    for tensor in model.tensors() {
        for (flat, x) in tensor.data.iter().enumerate() {
            let q = quantize(*x);
            let sign = if q < 0 { '-' } else { '+' };
            if q != 0 {
                body += &format!("    {} {sign}= {};\n", tensor.element(flat), q.abs());
            }
        }
    }
    format!(
        "pub fn {}_load(\n{}) {{\n{body}}}\n",
        model.name,
        param_list(&params)
    )
}
