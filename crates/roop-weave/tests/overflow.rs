mod support;

use roop_weave::{parse_model, quantize};
use std::path::Path;
use std::process::{Command, Output};
use support::{mlp, model, project, roop, text};

fn le(values: impl Iterator<Item = i64>) -> Vec<u8> {
    values.flat_map(i64::to_le_bytes).collect()
}

/// Trains xor for two epochs with the driver `roop weave --driver` writes.
fn train(name: &str, rate: f64) -> Output {
    let spec = model(name, 4, 1, vec![mlp("tanh", 6, 4, 2)]);
    let dir = project(name);
    std::fs::write(dir.join("model.json"), spec.to_string()).unwrap();
    let woven = roop(
        &dir,
        &[
            "weave",
            "model.json",
            "--batch",
            "4",
            "--driver",
            "main.c",
            "-o",
            "prog.roop",
        ],
    );
    assert!(woven.status.success(), "{}", text(&woven));
    let built = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(built.status.success(), "{}", text(&built));
    let parsed = parse_model(&spec.to_string()).unwrap();
    let weights = parsed.tensors().into_iter().flat_map(|t| t.data.iter());
    std::fs::write(dir.join("weights.bin"), le(weights.map(|x| quantize(*x)))).unwrap();
    let xs = [[1, 1, 1, 0], [1, -1, 1, 0], [-1, 1, 1, 0], [-1, -1, 1, 0]];
    let ts = [-0.5, 0.5, 0.5, -0.5];
    let data = xs.iter().flatten().map(|x| quantize(*x as f64));
    let data = data.chain(ts.iter().map(|t| quantize(*t)));
    std::fs::write(dir.join("data.bin"), le(data)).unwrap();
    run(&dir, rate)
}

fn run(dir: &Path, rate: f64) -> Output {
    Command::new(dir.join("prog"))
        .current_dir(dir)
        .args(["weights.bin", "data.bin", "2"])
        .arg(quantize(rate).to_string())
        .args(["4", "trained.bin"])
        .output()
        .unwrap()
}

#[test]
fn a_stable_rate_reports_a_loss_for_each_epoch() {
    let out = train("overflow_stable", 0.05);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert_eq!(String::from_utf8_lossy(&out.stdout).lines().count(), 2);
}

#[test]
fn a_diverging_rate_stops_instead_of_reporting_a_wrapped_loss() {
    let out = train("overflow_diverge", 1.0e9);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    assert!(String::from_utf8_lossy(&out.stderr).contains("overflow"));
}
