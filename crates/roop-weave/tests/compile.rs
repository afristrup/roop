mod support;

use roop_weave::parse_model;
use serde_json::json;
use support::{attention, compile, conv, leapfrog, mlp, mlp_norm, model, project, roop, text};

fn check(name: &str, layers: Vec<serde_json::Value>, width: usize, outputs: usize) {
    let dir = project(name);
    compile(
        &dir,
        &model(name, width, outputs, layers),
        &["--tests", "--batch", "3"],
    );
    let out = roop(&dir, &["test", "prog.roop"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("2 passed, 0 failed"), "{}", text(&out));
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

#[test]
fn lean_proves_the_weight_loader_exactly_reversible() {
    let dir = project("lean-load");
    let layers = vec![leapfrog("tanh", 2, 2, 1), mlp("relu", 2, 2, 2)];
    compile(&dir, &model("tiny", 2, 1, layers), &["--tests"]);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", text(&out));
    let report = text(&out);
    assert!(report.contains("tiny_load"), "{report}");
    assert!(report.contains("Lean accepted the file"), "{report}");
}

fn check_loss(name: &str, loss: &str, outputs: usize) {
    let dir = project(name);
    let layers = vec![
        mlp("tanh", 3, 4, 2),
        mlp("relu", 3, 4, 5),
        mlp("tanh", 3, 4, 7),
    ];
    let mut spec = model(name, 4, outputs, layers);
    spec["loss"] = loss.into();
    compile(&dir, &spec, &["--tests"]);
    let out = roop(&dir, &["test", "prog.roop"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("1 passed, 0 failed"), "{}", text(&out));
}

/// The generated tests, single and batched, of a model whose loss is scaled by 16.
fn check_scaled_loss(name: &str, loss: &str) {
    let dir = project(name);
    let layers = vec![mlp("tanh", 3, 4, 2), mlp("relu", 3, 4, 5)];
    let mut spec = model(name, 4, 3, layers);
    spec["loss"] = loss.into();
    spec["loss_scale"] = 16.into();
    compile(&dir, &spec, &["--tests", "--batch", "3"]);
    let out = roop(&dir, &["test", "prog.roop"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("2 passed, 0 failed"), "{}", text(&out));
}

#[test]
fn a_loss_scale_multiplies_the_gradients_and_nothing_else() {
    for loss in ["mse", "sigmoid", "softmax"] {
        check_scaled_loss(&format!("scaled_{loss}"), loss);
    }
}

#[test]
fn a_loss_scale_that_is_not_a_whole_number_in_range_is_refused() {
    for bad in [json!(0), json!(4097), json!(1.5), json!("big")] {
        let mut spec = model("bad_scale", 4, 1, vec![mlp("tanh", 3, 4, 2)]);
        spec["loss_scale"] = bad;
        let err = parse_model(&spec.to_string()).unwrap_err().to_string();
        assert!(err.contains("loss_scale"), "{err}");
    }
}

#[test]
fn the_cross_entropy_seeds_are_the_gradients_of_the_cross_entropy() {
    check_loss("ce_sigmoid", "sigmoid", 3);
    check_loss("ce_softmax", "softmax", 3);
}

#[test]
fn convolution_blocks_agree_with_the_reference() {
    check("conv_alone", vec![conv(2, 3, 1), conv(2, 3, 4)], 6, 2);
    check("conv_wide_kernel", vec![conv(2, 5, 2), conv(2, 1, 3)], 6, 2);
    check("conv_one_channel", vec![conv(1, 3, 5), conv(1, 3, 6)], 5, 2);
}

#[test]
fn convolution_mixed_with_the_other_blocks_agrees_with_the_reference() {
    let layers = vec![
        conv(2, 3, 1),
        mlp("silu", 3, 6, 2),
        attention(3, 2, 3),
        leapfrog("tanh", 4, 6, 4),
        conv(3, 3, 5),
    ];
    check("conv_mixed", layers, 6, 3);
}

#[test]
fn a_normalized_perceptron_of_each_activation_agrees_with_the_reference() {
    for act in ["identity", "relu", "tanh", "gelu"] {
        let layers = vec![mlp_norm(act, 4, 4, 2), mlp_norm(act, 3, 4, 5)];
        check(&format!("norm_{act}"), layers, 4, 2);
    }
}

#[test]
fn a_normalized_perceptron_mixed_with_the_other_blocks_agrees_with_the_reference() {
    let layers = vec![
        mlp_norm("silu", 5, 6, 1),
        conv(2, 3, 2),
        mlp("tanh", 3, 6, 3),
        mlp_norm("relu", 4, 6, 4),
        attention(3, 2, 5),
    ];
    check("norm_mixed", layers, 6, 3);
}

#[test]
fn lean_proves_a_compiled_batched_model_exactly_reversible() {
    let dir = project("lean-batch");
    let layers = vec![leapfrog("tanh", 2, 2, 1), mlp("relu", 2, 2, 2)];
    compile(&dir, &model("tiny", 2, 1, layers), &["--batch", "2"]);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", text(&out));
    let report = text(&out);
    for name in [
        "tiny_forward_batch",
        "tiny_backward_batch",
        "tiny_grad_batch",
    ] {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

#[test]
fn the_batched_functions_cover_convolution_normalization_and_cross_entropy() {
    let dir = project("batched-blocks");
    let layers = vec![
        conv(2, 3, 1),
        mlp_norm("silu", 4, 6, 2),
        attention(3, 2, 3),
        mlp("tanh", 3, 6, 4),
    ];
    let mut spec = model("batched", 6, 3, layers);
    spec["loss"] = "softmax".into();
    compile(&dir, &spec, &["--tests", "--batch", "2"]);
    let out = roop(&dir, &["test", "prog.roop"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("2 passed, 0 failed"), "{}", text(&out));
}
