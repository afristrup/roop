use crate::{Model, decl, load_helper, param_list, quantize};

/// The most writes one helper makes. Lean proves a run of writes to one row in
/// a lemma of its own, and a long run takes it too long.
const WRITES_PER_HELPER: usize = 4;

/// `<name>_load`: adds the parameters, on the 1/4096 grid, into zeros. It calls
/// one small helper per few weights of a row, so that Lean proves each helper
/// alone and the loader as a chain of calls.
pub fn emit_load(model: &Model) -> String {
    let params: Vec<String> = model.tensors().into_iter().map(|t| decl(t, true)).collect();
    let mut helpers = String::new();
    let mut calls = String::new();
    let mut count = 0;
    for tensor in model.tensors() {
        let len = tensor.dims[tensor.dims.len() - 1];
        for (r, row) in tensor.data.chunks(len).enumerate() {
            let writes: Vec<(usize, i64)> = row
                .iter()
                .enumerate()
                .map(|(i, x)| (i, quantize(*x)))
                .filter(|(_, q)| *q != 0)
                .collect();
            for chunk in writes.chunks(WRITES_PER_HELPER) {
                let name = format!("{}_load_{count}", model.name);
                count += 1;
                helpers += &load_helper(&name, len, chunk);
                calls += &format!("    call {name}({});\n", tensor.row(r));
            }
        }
    }
    format!(
        "{helpers}pub fn {}_load(\n{}) {{\n{calls}}}\n",
        model.name,
        param_list(&params)
    )
}
