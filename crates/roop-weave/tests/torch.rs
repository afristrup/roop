mod support;

use roop_weave::{gradients, loss, parse_model, sample};
use std::path::PathBuf;
use std::process::Command;
use support::{compile, project, roop, roop_binary, runtime_library, text};

const MODELS: [&str; 6] = [
    "attention_and_mlp",
    "mlp_then_leapfrog",
    "sigmoid_head",
    "smooth_activations",
    "softmax_head",
    "two_blocks_no_bias",
];

/// Runs the tests on real torch with uv, and leaves the exported models and
/// torch's gradients in `out`. False when uv is not installed.
fn export_with_torch(out: &std::path::Path) -> bool {
    let python = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("python");
    let run = Command::new("uv")
        .args(["run", "--extra", "torch", "python", "-m", "unittest", "-q"])
        .current_dir(python)
        .env("WEAVE_TORCH_OUT", out)
        .env("WEAVE_ROOP", roop_binary())
        .env("ROOP_RT_LIB", runtime_library())
        .output();
    let Ok(run) = run else {
        eprintln!("uv is not installed, so torch is not tested");
        return false;
    };
    assert!(run.status.success(), "{}", text(&run));
    true
}

#[test]
fn the_reference_gradients_are_the_gradients_torch_finds() {
    let dir = project("torch-grads");
    if !export_with_torch(&dir) {
        return;
    }
    for name in MODELS {
        let spec = std::fs::read_to_string(dir.join(format!("{name}.json"))).unwrap();
        let torch: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join(format!("{name}.grads.json"))).unwrap(),
        )
        .unwrap();
        let model = parse_model(&spec).unwrap().snapped();
        let (x, t) = sample(&model);
        let expected = loss(&model, &x, &t);
        let found = torch["loss"].as_f64().unwrap();
        assert!(
            (expected - found).abs() < 1e-9,
            "{name}: loss {expected} against torch {found}"
        );
        for (k, (ours, theirs)) in gradients(&model, &x, &t)
            .iter()
            .zip(torch["grads"].as_array().unwrap())
            .enumerate()
        {
            for (e, (a, b)) in ours.iter().zip(theirs.as_array().unwrap()).enumerate() {
                let b = b.as_f64().unwrap();
                // Central differences are off by about eps where a unit starts on the
                // kink of softsign, as the one with no bias does.
                assert!(
                    (a - b).abs() < 1e-5,
                    "{name}: tensor {k} element {e}: {a} against torch {b}"
                );
            }
        }
    }
}

#[test]
fn a_model_exported_from_torch_compiles_and_agrees_with_the_reference() {
    let dir = project("torch-compile");
    if !export_with_torch(&dir) {
        return;
    }
    for name in MODELS {
        let model = serde_json::from_str(
            &std::fs::read_to_string(dir.join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        compile(&dir, &model, &["--tests"]);
        let out = roop(&dir, &["test", "prog.roop"]);
        assert!(out.status.success(), "{name}: {}", text(&out));
        assert!(
            text(&out).contains("1 passed, 0 failed"),
            "{name}: {}",
            text(&out)
        );
    }
}
