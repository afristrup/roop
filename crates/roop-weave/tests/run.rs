mod support;

use roop_weave::{forward, parse_model};
use support::{compile, leapfrog, mlp, model, project, roop, text};

#[test]
fn the_main_of_a_model_prints_what_the_reference_computes() {
    let dir = project("main");
    let layers = vec![leapfrog("tanh", 3, 4, 1), mlp("relu", 3, 4, 2)];
    let spec = model("net", 4, 2, layers);
    compile(&dir, &spec, &["--main"]);
    let inputs = [1024_i64, -2048, 3072, 0];
    let args: Vec<String> = inputs.iter().map(i64::to_string).collect();
    let mut command = vec!["run", "prog.roop"];
    command.extend(args.iter().map(String::as_str));
    let out = roop(&dir, &command);
    assert!(out.status.success(), "{}", text(&out));
    let printed: Vec<f64> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().parse::<i64>().unwrap() as f64 / 4096.0)
        .collect();
    let model = parse_model(&spec.to_string()).unwrap().snapped();
    let x: Vec<f64> = inputs.iter().map(|v| *v as f64 / 4096.0).collect();
    let expected = forward(&model, &x);
    assert_eq!(printed.len(), expected.len(), "{printed:?}");
    for (got, want) in printed.iter().zip(&expected) {
        assert!(
            (got - want).abs() < 0.02,
            "{printed:?} against {expected:?}"
        );
    }
}

#[test]
fn the_main_refuses_the_wrong_number_of_arguments() {
    let dir = project("main-args");
    compile(
        &dir,
        &model("net", 4, 1, vec![leapfrog("tanh", 3, 4, 1)]),
        &["--main"],
    );
    let out = roop(&dir, &["run", "prog.roop", "1", "2"]);
    assert!(!out.status.success(), "{}", text(&out));
}
