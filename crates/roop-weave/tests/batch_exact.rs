mod support;

use roop_weave::{Model, Tensor, decl, names, parse_model};
use serde_json::Value;
use std::fmt::Write;
use support::{
    attention, compile, conv, leapfrog, mlp, mlp_layer, mlp_norm, model, project, residual, roop,
    text,
};

const ROWS: usize = 3;

fn plain(t: &Tensor) -> String {
    decl(t, true).replace(": &mut ", ": ")
}

fn gradients_of(model: &Model, suffix: &str) -> Vec<Tensor> {
    model
        .gradients()
        .into_iter()
        .map(|mut g| {
            g.name += suffix;
            g
        })
        .collect()
}

/// A test that runs the gradient of each of `ROWS` samples one at a time and
/// then the batched gradient, and requires every number to be equal.
fn exact_test(model: &Model) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let weights = names(&model.tensors()).join(", ");
    let (single, together) = (gradients_of(model, "_one"), gradients_of(model, "_all"));
    let mut fixtures = vec!["total_one: i64".to_string(), "total_all: i64".to_string()];
    for r in 0..ROWS {
        for v in ["q", "p", "aq", "ap"] {
            fixtures.push(format!("{v}{r}: [i64; {n}]"));
        }
        fixtures.push(format!("t{r}: [i64; {k}]"));
    }
    for v in ["q", "p", "aq", "ap"] {
        fixtures.push(format!("{v}_all: [[i64; {n}]; {ROWS}]"));
    }
    fixtures.push(format!("t_all: [[i64; {k}]; {ROWS}]"));
    fixtures.extend(model.tensors().into_iter().map(plain));
    fixtures.extend(single.iter().chain(&together).map(plain));

    let mut body = format!("    call {name}_load({weights});\n");
    for r in 0..ROWS {
        for i in 0..n {
            let x = ((r * 5 + i * 3) % 7) as i64 * 900 - 2700;
            writeln!(body, "    q{r}[{i}] += {x};\n    q_all[{r}][{i}] += {x};").unwrap();
        }
        for j in 0..k {
            let x = ((r * 3 + j * 2) % 5) as i64 * 700 - 1400;
            writeln!(body, "    t{r}[{j}] += {x};\n    t_all[{r}][{j}] += {x};").unwrap();
        }
    }
    let (one, all) = (names(&single).join(", "), names(&together).join(", "));
    for r in 0..ROWS {
        writeln!(
            body,
            "    call {name}_grad(total_one, q{r}, p{r}, aq{r}, ap{r}, {one}, {weights}, t{r});"
        )
        .unwrap();
    }
    writeln!(
        body,
        "    call {name}_grad_batch(total_all, q_all, p_all, aq_all, ap_all, {all}, {weights}, t_all);"
    )
    .unwrap();
    body += "    expect total_one == total_all;\n";
    let mut any = Vec::new();
    for (a, b) in single.iter().zip(&together) {
        for flat in 0..a.data.len() {
            let (x, y) = (a.element(flat), b.element(flat));
            writeln!(body, "    expect {x} == {y};").unwrap();
            any.push(format!("{x} != 0"));
        }
    }
    writeln!(body, "    expect {};", any.join(" || ")).unwrap();
    for r in 0..ROWS {
        for i in 0..n {
            writeln!(body, "    expect aq{r}[{i}] == aq_all[{r}][{i}];").unwrap();
        }
    }
    format!(
        "test {name}_batch_is_the_sum_of_the_rows_bit_for_bit {{\n    {};\n{body}}}\n",
        fixtures.join(",\n    ")
    )
}

fn check(name: &str, spec: Value) {
    let dir = project(name);
    compile(&dir, &spec, &["--tests", "--batch", &ROWS.to_string()]);
    let parsed = parse_model(&spec.to_string()).unwrap();
    let mut program = std::fs::read_to_string(dir.join("prog.roop")).unwrap();
    program += &exact_test(&parsed);
    std::fs::write(dir.join("prog.roop"), program).unwrap();
    let out = roop(&dir, &["test", "prog.roop"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("3 passed, 0 failed"), "{}", text(&out));
}

#[test]
fn a_batched_perceptron_with_rms_normalization_equals_its_rows_one_at_a_time() {
    let layers = vec![mlp_norm("silu", 4, 6, 2), mlp_norm("tanh", 5, 6, 3)];
    check("exact_norm", model("exactnorm", 6, 3, layers));
}

#[test]
fn a_batched_perceptron_with_layer_normalization_equals_its_rows_one_at_a_time() {
    let layers = vec![mlp_layer("gelu", 4, 6, 2), mlp_layer("relu", 5, 6, 3)];
    check("exact_layer", model("exactlayer", 6, 3, layers));
}

#[test]
fn a_batched_convolution_equals_its_rows_one_at_a_time() {
    let layers = vec![conv(2, 3, 1), conv(3, 3, 2)];
    check("exact_conv", model("exactconv", 6, 3, layers));
}

#[test]
fn a_batched_mixed_model_equals_its_rows_one_at_a_time() {
    let layers = vec![
        leapfrog("tanh", 3, 6, 1),
        conv(2, 3, 1),
        mlp_norm("silu", 4, 6, 2),
        attention(3, 2, 3),
        mlp_layer("relu", 4, 6, 4),
        mlp("tanh", 3, 6, 4),
    ];
    check("exact_mixed", model("exactmixed", 6, 3, layers));
}

#[test]
fn a_batched_residual_block_equals_its_rows_one_at_a_time() {
    let layers = vec![
        residual("tanh", 4, 6, 2, 0.3),
        residual("silu", 5, 6, 3, 0.3),
    ];
    check("exact_residual", model("exactresidual", 6, 3, layers));
}

#[test]
fn a_batched_residual_block_among_other_layers_equals_its_rows_one_at_a_time() {
    let layers = vec![
        mlp("tanh", 3, 6, 4),
        residual("relu", 4, 6, 5, 0.3),
        leapfrog("tanh", 3, 6, 1),
    ];
    check("exact_residual_mixed", model("exactresmixed", 6, 3, layers));
}
