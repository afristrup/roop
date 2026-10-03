use crate::{
    LossKind, Model, batch_sample, decl, forward, gradient_unit, gradients, loss, names, near,
    quantize,
};

/// A test that loads the model, runs a batch of different samples through its
/// batched forward pass and gradient, and checks the outputs, the summed loss
/// and every summed gradient against the reference in doubles, one sample at a time.
pub fn emit_batch_tests(model: &Model, batch: usize) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let reference = model.snapped();
    let rows = batch_sample(&reference, batch);
    let outs: Vec<Vec<f64>> = rows.iter().map(|(x, _)| forward(&reference, x)).collect();
    let total: f64 = rows.iter().map(|(x, t)| loss(&reference, x, t)).sum();
    let per_row: Vec<Vec<Vec<f64>>> = rows
        .iter()
        .map(|(x, t)| gradients(&reference, x, t))
        .collect();
    let summed: Vec<Vec<f64>> = (0..model.tensors().len())
        .map(|t| {
            (0..per_row[0][t].len())
                .map(|e| per_row.iter().map(|g| g[t][e]).sum())
                .collect()
        })
        .collect();
    let scale: f64 = per_row
        .iter()
        .map(|g| g.iter().flatten().fold(0.0_f64, |a, x| a.max(x.abs())))
        .sum();
    let layers = model.layers.len() as f64;

    let weights = names(&model.tensors()).join(", ");
    let gradient_names = names(&model.gradients()).join(", ");
    let mut fixtures = vec![
        "total: i64".to_string(),
        format!("t: [[i64; {k}]; {batch}]"),
    ];
    fixtures.extend(
        ["q", "p", "aq", "ap"]
            .iter()
            .map(|s| format!("{s}: [[i64; {n}]; {batch}]")),
    );
    let plain = |d: String| d.replace(": &mut ", ": ");
    fixtures.extend(model.tensors().into_iter().map(|t| plain(decl(t, true))));
    fixtures.extend(model.gradients().iter().map(|g| plain(decl(g, true))));

    let mut body = format!("    call {name}_load({weights});\n");
    for (row, (x, _)) in rows.iter().enumerate() {
        for (i, x) in x.iter().enumerate().filter(|(_, x)| quantize(**x) != 0) {
            body += &format!("    q[{row}][{i}] += {};\n", quantize(*x));
        }
    }
    body += &format!("    call {name}_forward_batch(q, p, {weights});\n");
    let forward_tolerance = quantize(0.01 * (layers + 1.0));
    for (row, out) in outs.iter().enumerate() {
        for (i, out) in out.iter().enumerate() {
            body += &near(&format!("q[{row}][{i}]"), quantize(*out), forward_tolerance);
        }
    }
    body += &format!("    uncall {name}_forward_batch(q, p, {weights});\n");
    for (row, (_, t)) in rows.iter().enumerate() {
        for (i, t) in t.iter().enumerate().filter(|(_, t)| quantize(**t) != 0) {
            body += &format!("    t[{row}][{i}] += {};\n", quantize(*t));
        }
    }
    body += &format!(
        "    call {name}_grad_batch(total, q, p, aq, ap, {gradient_names}, {weights}, t);\n"
    );
    let rows_len = batch as f64;
    if model.loss == LossKind::Mse {
        body += &near(
            "total",
            quantize(total),
            quantize(0.02 * rows_len + 0.02 * total),
        );
    }
    let gradient_tolerance = quantize(0.05 * scale + 0.005 * rows_len);
    for (tensor, grad) in model.gradients().iter().zip(&summed) {
        for (flat, g) in grad.iter().enumerate() {
            let unit = gradient_unit(tensor);
            let expected = (g * 4096.0 * unit as f64).round() as i64;
            body += &near(&tensor.element(flat), expected, gradient_tolerance * unit);
        }
    }
    format!(
        "test {name}_batch_matches_the_reference {{\n    {};\n{body}}}\n",
        fixtures.join(",\n    ")
    )
}
