mod support;

use roop_weave::{parse_model, quantize};
use serde_json::json;
use std::process::Command;
use support::{compile, mlp, mlp_layer, model, project, residual, roop, text};

const XS: [[i64; 4]; 4] = [[1, 1, 1, 0], [1, -1, 1, 0], [-1, 1, 1, 0], [-1, -1, 1, 0]];
const TS: [f64; 4] = [-0.5, 0.5, 0.5, -0.5];

/// A C program that trains the compiled model on xor and exits 0 when the loss
/// has fallen to a fifth.
fn driver(model: &roop_weave::Model, rate: f64, epochs: usize) -> String {
    let tensors = model.tensors();
    let numbers = |data: &[f64]| {
        let items: Vec<String> = data.iter().map(|x| quantize(*x).to_string()).collect();
        items.join(", ")
    };
    let mut c = String::from("#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n");
    let mut pointers = vec!["&total".to_string()];
    for t in &tensors {
        c += &format!(
            "static int64_t {}[{}] = {{{}}};\n",
            t.name,
            t.data.len(),
            numbers(&t.data)
        );
        pointers.push(t.name.clone());
    }
    for t in &tensors {
        c += &format!("static int64_t g{}[{}];\n", t.name, t.data.len());
        pointers.push(format!("g{}", t.name));
    }
    for t in model.optimizer_state() {
        c += &format!("static int64_t {}[{}];\n", t.name, t.data.len());
        pointers.push(t.name);
    }
    c += "static int64_t q[16], p[16], aq[16], ap[16];\n";
    pointers
        .extend(["q", "p", "aq", "ap", "(int64_t*)xs", "(int64_t*)ts", "&lr"].map(String::from));
    let xs: Vec<String> = XS.iter().map(|x| numbers(&x.map(|v| v as f64))).collect();
    let ts: Vec<String> = TS.iter().map(|t| quantize(*t).to_string()).collect();
    c += &format!("static int64_t xs[4][4] = {{{{{}}}}};\n", xs.join("}, {"));
    c += &format!("static int64_t ts[4][1] = {{{{{}}}}};\n", ts.join("}, {"));
    let types = vec!["int64_t*"; pointers.len()].join(", ");
    c += &format!("void {}_train({types});\n", model.name);
    let dump: String = tensors.iter().map(|t| format!("    if (getenv(\"WEAVE_DUMP\")) {{ fprintf(stderr, \"T {}\"); for (int i = 0; i < {}; i++) fprintf(stderr, \" %g\", ((int64_t*){})[i] / 4096.0); fprintf(stderr, \"\\n\"); }}\n", t.name, t.data.len(), t.name)).collect();
    c += &format!(
        "int main(void) {{
    int64_t lr = {};
    double first = 0, last = 0;
    for (int e = 0; e < {epochs}; e++) {{
        int64_t total = 0;
        {}_train({});
        last = total / 4096.0;
        if (e == 0) first = last;
        if (getenv(\"WEAVE_TRACE\") && e % 100 == 99) fprintf(stderr, \"%d %.4f\\n\", e + 1, last);
    }}
    fprintf(stderr, \"loss %.4f -> %.4f\\n\", first, last);
{}
    return last >= 0 && last < first / 5 ? 0 : 1;
}}
",
        quantize(rate),
        model.name,
        pointers.join(", "),
        dump
    );
    c
}

fn train_xor(name: &str, optimizer: serde_json::Value, default_rate: f64) {
    let layers = vec![
        mlp("tanh", 6, 4, 2),
        mlp("tanh", 6, 4, 5),
        mlp("tanh", 6, 4, 8),
    ];
    train_layers(name, layers, optimizer, default_rate);
}

fn train_layers(
    name: &str,
    layers: Vec<serde_json::Value>,
    optimizer: serde_json::Value,
    default_rate: f64,
) {
    let mut spec = model(name, 4, 1, layers);
    spec["optimizer"] = optimizer;
    let dir = project(name);
    compile(&dir, &spec, &["--batch", "4"]);
    let parsed = parse_model(&spec.to_string()).unwrap();
    let rate = std::env::var("WEAVE_LR").map_or(default_rate, |v| v.parse().unwrap());
    let epochs = std::env::var("WEAVE_EPOCHS").map_or(3000, |v| v.parse().unwrap());
    std::fs::write(dir.join("main.c"), driver(&parsed, rate, epochs)).unwrap();
    let built = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(built.status.success(), "{}", text(&built));
    let run = Command::new(dir.join("prog")).output().unwrap();
    eprint!("{}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(run.status.code(), Some(0));
}

#[test]
fn a_compiled_model_trains_on_xor_without_storing_activations() {
    train_xor("xor_sgd", json!({"kind": "sgd"}), 0.05);
}

#[test]
fn momentum_trains_on_xor() {
    train_xor(
        "xor_momentum",
        json!({"kind": "momentum", "beta": 0.9}),
        0.01,
    );
}

#[test]
fn adam_trains_on_xor() {
    train_xor("xor_adam", json!({"kind": "adam"}), 0.01);
}

#[test]
fn layer_normalized_perceptrons_train_on_xor() {
    let layers = vec![
        mlp_layer("tanh", 6, 4, 2),
        mlp_layer("tanh", 6, 4, 5),
        mlp_layer("tanh", 6, 4, 8),
    ];
    train_layers("xor_layer", layers, json!({"kind": "sgd"}), 0.05);
}

#[test]
fn residual_blocks_train_on_xor() {
    let layers = vec![
        residual("tanh", 6, 4, 2, 0.3),
        residual("tanh", 6, 4, 5, 0.3),
        residual("tanh", 6, 4, 8, 0.3),
    ];
    train_layers("xor_residual", layers, json!({"kind": "sgd"}), 0.05);
}

fn kept_residuals() -> Vec<serde_json::Value> {
    let mut layers = vec![
        residual("tanh", 6, 4, 2, 0.6),
        residual("tanh", 6, 4, 5, 0.6),
        residual("tanh", 6, 4, 8, 0.6),
    ];
    for layer in &mut layers {
        layer["keep_contraction"] = json!(0.8);
    }
    layers
}

#[test]
fn residual_blocks_with_a_kept_contraction_train_on_xor() {
    train_layers("xor_kept", kept_residuals(), json!({"kind": "sgd"}), 0.05);
}

#[test]
fn momentum_trains_residual_blocks_with_a_kept_contraction_on_xor() {
    let optimizer = json!({"kind": "momentum", "beta": 0.9});
    train_layers("xor_kept_momentum", kept_residuals(), optimizer, 0.01);
}

#[test]
fn adam_trains_residual_blocks_with_a_kept_contraction_on_xor() {
    train_layers(
        "xor_kept_adam",
        kept_residuals(),
        json!({"kind": "adam"}),
        0.01,
    );
}

#[test]
fn the_per_sample_and_the_batched_step_project_after_every_optimizer() {
    let optimizers = [
        ("sgd_carry_", json!({"kind": "sgd"})),
        ("momentum_", json!({"kind": "momentum", "beta": 0.9})),
        ("adam_", json!({"kind": "adam"})),
    ];
    for (update, optimizer) in optimizers {
        let mut spec = model("net", 4, 1, kept_residuals());
        spec["optimizer"] = optimizer;
        let parsed = parse_model(&spec.to_string()).unwrap();
        for batched in [false, true] {
            let step = roop_weave::emit_step(&parsed, batched);
            assert_eq!(step.matches("call project_contraction<4, 6>").count(), 3);
            let project = step.find("call project_contraction").unwrap();
            assert!(
                project > step.rfind(&format!("call {update}")).unwrap(),
                "{step}"
            );
            assert!(step[project..].contains("call clear<"), "{step}");
        }
    }
}
