use crate::{Model, decl, forward, gradients, loss, names, quantize};

/// `a - e` for a number a and an integer e, as roop writes it.
fn difference(a: &str, e: i64) -> String {
    match e < 0 {
        true => format!("({a} + {})", -e),
        false => format!("({a} - {e})"),
    }
}

fn near(a: &str, e: i64, tolerance: i64) -> String {
    let d = difference(a, e);
    format!("    expect {d} * {d} <= {};\n", tolerance * tolerance)
}

/// A fixed input and target, on the grid.
fn sample(model: &Model) -> (Vec<f64>, Vec<f64>) {
    let x = (0..model.width)
        .map(|i| ((i * 3 + 1) % 7) as f64 / 4.0 - 0.75)
        .collect();
    let t = (0..model.outputs)
        .map(|k| ((k * 2 + 1) % 3) as f64 / 4.0 - 0.25)
        .collect();
    (x, t)
}

/// A test that loads the model, and checks its forward pass, its loss and every
/// gradient against the reference in doubles. Like every roop test it also runs
/// backward, which checks that all of it undoes itself.
pub fn emit_tests(model: &Model) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let reference = model.snapped();
    let (x, t) = sample(&reference);
    let out = forward(&reference, &x);
    let total = loss(&reference, &x, &t);
    let grads = gradients(&reference, &x, &t);
    let scale = grads.iter().flatten().fold(0.0_f64, |a, g| a.max(g.abs()));
    let layers = model.layers.len() as f64;

    let weights = names(&model.tensors()).join(", ");
    let gradient_names = names(&model.gradients()).join(", ");
    let mut fixtures = vec!["total: i64".to_string(), format!("t: [i64; {k}]")];
    fixtures.extend(
        ["q", "p", "aq", "ap"]
            .iter()
            .map(|s| format!("{s}: [i64; {n}]")),
    );
    fixtures.extend(
        model
            .tensors()
            .into_iter()
            .map(|t| decl(t, true).replace(": &mut ", ": ")),
    );
    fixtures.extend(
        model
            .gradients()
            .iter()
            .map(|g| decl(g, true).replace(": &mut ", ": ")),
    );

    let mut body = format!("    call {name}_load({weights});\n");
    for (i, x) in x.iter().enumerate().filter(|(_, x)| quantize(**x) != 0) {
        body += &format!("    q[{i}] += {};\n", quantize(*x));
    }
    body += &format!("    call {name}_forward(q, p, {weights});\n");
    let forward_tolerance = quantize(0.01 * (layers + 1.0));
    for (i, out) in out.iter().enumerate() {
        body += &near(&format!("q[{i}]"), quantize(*out), forward_tolerance);
    }
    body += &format!("    uncall {name}_forward(q, p, {weights});\n");
    for (i, t) in t.iter().enumerate().filter(|(_, t)| quantize(**t) != 0) {
        body += &format!("    t[{i}] += {};\n", quantize(*t));
    }
    body +=
        &format!("    call {name}_grad(total, q, p, aq, ap, {gradient_names}, {weights}, t);\n");
    body += &near("total", quantize(total), quantize(0.02 + 0.02 * total));
    let gradient_tolerance = quantize(0.05 * scale + 0.005);
    for (tensor, grad) in model.gradients().iter().zip(&grads) {
        for (flat, g) in grad.iter().enumerate() {
            body += &near(&tensor.element(flat), quantize(*g), gradient_tolerance);
        }
    }
    format!(
        "test {name}_matches_the_reference {{\n    {};\n{body}}}\n",
        fixtures.join(",\n    ")
    )
}
