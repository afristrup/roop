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
        "[modules]\nweave = \"{}\"\neinsum = \"{}\"\n",
        root("weave").display(),
        root("einsum").display()
    );
    std::fs::write(dir.join("Roop.toml"), config).unwrap();
    dir
}

pub fn roop(dir: &Path, args: &[&str]) -> Output {
    static BUILD: std::sync::Once = std::sync::Once::new();
    BUILD.call_once(|| {
        build("roop");
        build("roop-rt");
    });
    Command::new(target().join("debug/roop"))
        .current_dir(dir)
        .args(args)
        .env("ROOP_RT_LIB", target().join("debug/libroop_rt.a"))
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

pub fn model(name: &str, width: usize, outputs: usize, layers: Vec<Value>) -> Value {
    json!({ "name": name, "width": width, "outputs": outputs, "step": 0.25, "layers": layers })
}

/// Compiles a model with roop-weave, with its tests, and writes it as prog.roop.
pub fn compile(dir: &Path, model: &Value, flags: &[&str]) {
    let input = dir.join("model.json");
    std::fs::write(&input, model.to_string()).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_roop-weave"))
        .arg(&input)
        .args(["-o", dir.join("prog.roop").to_str().unwrap()])
        .args(flags)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
}
