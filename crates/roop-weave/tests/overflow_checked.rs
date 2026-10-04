mod support;

use roop_weave::{parse_model, quantize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output};
use support::{mlp, model, project, roop, text};

fn le(values: impl Iterator<Item = i64>) -> Vec<u8> {
    values.flat_map(i64::to_le_bytes).collect()
}

/// Weaves xor with the checking driver and builds it, with the overflow flag
/// compiled in when `checked`.
fn build(name: &str, checked: bool) -> std::path::PathBuf {
    let spec = model(name, 4, 1, vec![mlp("tanh", 6, 4, 2), mlp("tanh", 6, 4, 3)]);
    let dir = project(name);
    if checked {
        let mut config = OpenOptions::new()
            .append(true)
            .open(dir.join("Roop.toml"))
            .unwrap();
        config.write_all(b"\n[checks]\noverflow = true\n").unwrap();
    }
    std::fs::write(dir.join("model.json"), spec.to_string()).unwrap();
    let mut args = vec!["weave", "model.json", "--batch", "4", "--driver", "main.c"];
    args.extend(["-o", "prog.roop"]);
    if checked {
        args.push("--checked");
    }
    let woven = roop(&dir, &args);
    assert!(woven.status.success(), "{}", text(&woven));
    let built = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(built.status.success(), "{}", text(&built));
    let parsed = parse_model(&spec.to_string()).unwrap();
    let weights = parsed.tensors().into_iter().flat_map(|t| t.data.iter());
    std::fs::write(dir.join("weights.bin"), le(weights.map(|x| quantize(*x)))).unwrap();
    dir
}

/// Trains two epochs on xor with the second input scaled by `magnitude`. The
/// loss reads the first output only, and the saturated tanh gives the second
/// layer a bounded input and the first a zero gradient, so a huge second input
/// wraps inside the step while the loss and the weights stay small.
fn run(dir: &Path, magnitude: f64) -> Output {
    let xs = [[1, 1, 1, 0], [1, -1, 1, 0], [-1, 1, 1, 0], [-1, -1, 1, 0]];
    let ts = [-0.5, 0.5, 0.5, -0.5];
    let data = xs.iter().flat_map(|row| {
        let scaled = row.iter().enumerate();
        scaled.map(|(j, x)| quantize(*x as f64 * if j == 1 { magnitude } else { 1.0 }))
    });
    let data = data.chain(ts.iter().map(|t| quantize(*t)));
    std::fs::write(dir.join("data.bin"), le(data)).unwrap();
    Command::new(dir.join("prog"))
        .current_dir(dir)
        .args(["weights.bin", "data.bin", "2"])
        .arg(quantize(0.05).to_string())
        .args(["4", "trained.bin"])
        .output()
        .unwrap()
}

#[test]
fn a_wrap_inside_the_step_is_missed_unchecked_and_caught_checked() {
    let plain = build("checked_off", false);
    let missed = run(&plain, 3.0e13);
    assert_eq!(missed.status.code(), Some(0), "{}", text(&missed));

    let flagged = build("checked_on", true);
    let caught = run(&flagged, 3.0e13);
    assert_eq!(caught.status.code(), Some(3), "{}", text(&caught));
    assert!(String::from_utf8_lossy(&caught.stderr).contains("wrapped inside a step"));
}

#[test]
fn a_run_that_does_not_wrap_is_the_same_in_both_modes() {
    let (plain, flagged) = (build("same_off", false), build("same_on", true));
    let (a, b) = (run(&plain, 1.0), run(&flagged, 1.0));
    assert_eq!(a.status.code(), Some(0), "{}", text(&a));
    assert_eq!(b.status.code(), Some(0), "{}", text(&b));
    assert_eq!(a.stdout, b.stdout);
    let (trained_a, trained_b) = (plain.join("trained.bin"), flagged.join("trained.bin"));
    assert_eq!(
        std::fs::read(trained_a).unwrap(),
        std::fs::read(trained_b).unwrap()
    );
}
