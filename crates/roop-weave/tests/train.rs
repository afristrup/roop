mod support;

use roop_weave::{parse_model, quantize};
use std::process::Command;
use support::{compile, mlp, model, project, roop, text};

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
    let mut c = String::from("#include <stdint.h>\n#include <stdio.h>\n");
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
    c += "static int64_t q[4], p[4], aq[4], ap[4];\n";
    pointers
        .extend(["q", "p", "aq", "ap", "(int64_t*)xs", "(int64_t*)ts", "&lr"].map(String::from));
    let xs: Vec<String> = XS.iter().map(|x| numbers(&x.map(|v| v as f64))).collect();
    let ts: Vec<String> = TS.iter().map(|t| quantize(*t).to_string()).collect();
    c += &format!("static int64_t xs[4][4] = {{{{{}}}}};\n", xs.join("}, {"));
    c += &format!("static int64_t ts[4][1] = {{{{{}}}}};\n", ts.join("}, {"));
    let types = vec!["int64_t*"; pointers.len()].join(", ");
    c += &format!("void {}_train({types});\n", model.name);
    c += &format!(
        "int main(void) {{
    int64_t lr = {};
    double first = 0, last = 0;
    for (int e = 0; e < {epochs}; e++) {{
        int64_t total = 0;
        {}_train({});
        last = total / 4096.0;
        if (e == 0) first = last;
    }}
    fprintf(stderr, \"loss %.4f -> %.4f\\n\", first, last);
    return last < first / 5 ? 0 : 1;
}}
",
        quantize(rate),
        model.name,
        pointers.join(", ")
    );
    c
}

#[test]
fn a_compiled_model_trains_on_xor_without_storing_activations() {
    let layers = vec![
        mlp("tanh", 6, 4, 2),
        mlp("tanh", 6, 4, 5),
        mlp("tanh", 6, 4, 8),
    ];
    let spec = model("xor", 4, 1, layers);
    let dir = project("train");
    compile(&dir, &spec, &["--batch", "4"]);
    let parsed = parse_model(&spec.to_string()).unwrap();
    let rate = std::env::var("WEAVE_LR").map_or(0.05, |v| v.parse().unwrap());
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
