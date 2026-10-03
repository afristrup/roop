mod support;

use support::{attention, compile, leapfrog, mlp, model, project, roop, text};

fn check(name: &str, layers: Vec<serde_json::Value>, width: usize, outputs: usize) {
    let dir = project(name);
    compile(&dir, &model(name, width, outputs, layers), &["--tests"]);
    let out = roop(&dir, &["test", "prog.roop"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("1 passed, 0 failed"), "{}", text(&out));
}

#[test]
fn a_leapfrog_layer_of_each_activation_agrees_with_the_reference() {
    for act in [
        "identity", "cauchy", "softsign", "relu", "tanh", "sigmoid", "silu", "gelu",
    ] {
        check(&format!("leap_{act}"), vec![leapfrog(act, 3, 4, 1)], 4, 2);
    }
}

#[test]
fn a_perceptron_of_each_activation_agrees_with_the_reference() {
    for act in [
        "identity", "cauchy", "softsign", "relu", "tanh", "sigmoid", "silu", "gelu",
    ] {
        check(
            &format!("mlp_{act}"),
            vec![mlp(act, 3, 4, 2), mlp(act, 2, 4, 5)],
            4,
            2,
        );
    }
}

#[test]
fn layers_of_different_widths_and_kinds_agree_with_the_reference() {
    let layers = vec![
        leapfrog("tanh", 3, 4, 1),
        mlp("relu", 3, 4, 2),
        mlp("softsign", 2, 4, 4),
        leapfrog("cauchy", 5, 4, 3),
    ];
    check("mixed", layers, 4, 2);
}

#[test]
fn attention_blocks_agree_with_the_reference() {
    check(
        "attn_alone",
        vec![attention(2, 2, 1), attention(2, 2, 4)],
        4,
        2,
    );
    check(
        "attn_longer",
        vec![attention(3, 2, 2), attention(3, 2, 5)],
        6,
        3,
    );
}

#[test]
fn attention_mixed_with_perceptrons_and_leapfrog_agrees_with_the_reference() {
    let layers = vec![
        attention(2, 2, 1),
        mlp("gelu", 3, 4, 2),
        leapfrog("tanh", 3, 4, 3),
        attention(2, 2, 4),
        mlp("softsign", 2, 4, 5),
    ];
    check("attn_mixed", layers, 4, 2);
}

#[test]
fn lean_proves_a_compiled_model_exactly_reversible() {
    let dir = project("lean");
    let layers = vec![leapfrog("tanh", 2, 2, 1), mlp("relu", 2, 2, 2)];
    compile(&dir, &model("tiny", 2, 1, layers), &[]);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", text(&out));
    let report = text(&out);
    for name in ["tiny_forward", "tiny_backward", "tiny_grad"] {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}
