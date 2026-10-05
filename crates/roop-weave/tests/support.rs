#![allow(dead_code)]

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn target() -> PathBuf {
    std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| format!("{}/../../target", env!("CARGO_MANIFEST_DIR")))
        .into()
}

fn root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../roop")
        .join(name)
}

fn build(package: &str) {
    let status = Command::new(env!("CARGO"))
        .args(["build", "-p", package])
        .status()
        .unwrap();
    assert!(status.success());
}

/// A directory with a Roop.toml that finds weave and einsum.
pub fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("roop-weave-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let config = format!(
        "[modules]\nweave = \"{}\"\neinsum = \"{}\"\nstd = \"{}\"\n",
        root("weave").display(),
        root("einsum").display(),
        root("std").display()
    );
    std::fs::write(dir.join("Roop.toml"), config).unwrap();
    dir
}

pub fn roop_binary() -> PathBuf {
    static BUILD: std::sync::Once = std::sync::Once::new();
    BUILD.call_once(|| {
        build("roop");
        build("roop-rt");
    });
    target().join("debug/roop")
}

pub fn runtime_library() -> PathBuf {
    roop_binary();
    target().join("debug/libroop_rt.a")
}

pub fn roop(dir: &Path, args: &[&str]) -> Output {
    Command::new(roop_binary())
        .current_dir(dir)
        .args(args)
        .env("ROOP_RT_LIB", runtime_library())
        .output()
        .unwrap()
}

pub fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn weights(rows: usize, cols: usize, seed: usize) -> Value {
    let row = |i: usize| -> Vec<f64> {
        (0..cols)
            .map(|j| (((seed + i * 7 + j * 3) % 9) as f64 - 4.0) / 10.0)
            .collect()
    };
    json!((0..rows).map(row).collect::<Vec<_>>())
}

fn biases(len: usize, seed: usize) -> Value {
    // Never zero, so that no unit starts on the kink of a relu.
    json!(
        (0..len)
            .map(|i| (((seed + i * 2) % 5) as f64 - 2.0) / 10.0 + 0.05)
            .collect::<Vec<_>>()
    )
}

pub fn leapfrog(act: &str, hidden: usize, width: usize, seed: usize) -> Value {
    json!({
        "kind": "leapfrog",
        "activation": act,
        "weight": weights(hidden, width, seed),
        "bias": biases(hidden, seed),
    })
}

pub fn mlp(act: &str, hidden: usize, width: usize, seed: usize) -> Value {
    json!({
        "kind": "mlp",
        "activation": act,
        "w1": weights(hidden, width, seed),
        "b1": biases(hidden, seed),
        "w2": weights(width, hidden, seed + 1),
        "b2": biases(width, seed + 1),
    })
}

/// A perceptron whose hidden layer is normalized by its root mean square.
pub fn mlp_norm(act: &str, hidden: usize, width: usize, seed: usize) -> Value {
    let mut layer = mlp(act, hidden, width, seed);
    let gain: Vec<f64> = (0..hidden).map(|j| 0.8 + 0.1 * (j % 3) as f64).collect();
    layer["norm"] = json!({ "eps": 0.01, "gain": gain });
    layer
}

/// A perceptron whose hidden layer is layer normalized: centered, scaled, shifted.
pub fn mlp_layer(act: &str, hidden: usize, width: usize, seed: usize) -> Value {
    let mut layer = mlp(act, hidden, width, seed);
    let gain: Vec<f64> = (0..hidden).map(|j| 0.8 + 0.1 * (j % 3) as f64).collect();
    let bias: Vec<f64> = (0..hidden).map(|j| 0.05 * (j % 4) as f64 - 0.07).collect();
    layer["norm"] = json!({ "eps": 0.01, "center": true, "gain": gain, "bias": bias });
    layer
}

/// A residual block whose weights are `scale` times those of a perceptron, so that
/// its function is a contraction for a small enough scale.
pub fn residual(act: &str, hidden: usize, width: usize, seed: usize, scale: f64) -> Value {
    let mut layer = mlp(act, hidden, width, seed);
    layer["kind"] = json!("residual");
    for key in ["w1", "w2"] {
        let rows = layer[key].as_array().unwrap().clone();
        let scaled = |row: &Value| -> Vec<f64> {
            row.as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap() * scale)
                .collect()
        };
        layer[key] = json!(rows.iter().map(scaled).collect::<Vec<_>>());
    }
    layer
}

pub fn attention(seq: usize, dim: usize, seed: usize) -> Value {
    json!({
        "kind": "attention",
        "seq": seq,
        "wq": weights(dim, dim, seed),
        "wk": weights(dim, dim, seed + 1),
        "wv": weights(dim, dim, seed + 2),
    })
}

pub fn conv(channels: usize, kernel: usize, seed: usize) -> Value {
    json!({
        "kind": "conv",
        "channels": channels,
        "kernel": kernel,
        "weight": weights(channels, channels * kernel, seed),
        "bias": biases(channels, seed),
    })
}

pub fn model(name: &str, width: usize, outputs: usize, layers: Vec<Value>) -> Value {
    json!({ "name": name, "width": width, "outputs": outputs, "step": 0.25, "layers": layers })
}

/// Compiles a model with `roop weave`, and writes it as prog.roop.
pub fn compile(dir: &Path, model: &Value, flags: &[&str]) {
    std::fs::write(dir.join("model.json"), model.to_string()).unwrap();
    let mut args = vec!["weave", "model.json", "-o", "prog.roop"];
    args.extend_from_slice(flags);
    let out = roop(dir, &args);
    assert!(out.status.success(), "{}", text(&out));
}

/// Runs a main whose exit status is 0 when `check` holds after `call` has run on the
/// arrays that `setup` makes.
pub fn holds(name: &str, setup: &str, call: &str, check: &str) -> bool {
    let dir = project(name);
    let program = format!(
        "use weave::*;

irrev fn run(status: &mut i64) {{
{setup}
    {call}
    if {check} {{ status = 0; }} else {{ status = 1; }} fi {check};
}}

fn main(status: &mut i64) {{
    irrev {{ call run(status); }}
}}
"
    );
    std::fs::write(dir.join("prog.roop"), program).unwrap();
    let out = roop(&dir, &["run", "prog.roop"]);
    assert!(out.status.code().is_some(), "{}", text(&out));
    out.status.success()
}
