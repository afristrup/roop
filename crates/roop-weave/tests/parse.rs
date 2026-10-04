mod support;

use roop_weave::{WeaveError, parse_model};
use serde_json::json;
use support::{attention, conv, leapfrog, mlp, mlp_layer, mlp_norm, model, residual};

fn error_of(value: serde_json::Value) -> WeaveError {
    parse_model(&value.to_string()).unwrap_err()
}

#[test]
fn a_good_model_reads_back_with_its_shapes() {
    let layers = vec![leapfrog("tanh", 3, 4, 1), mlp("relu", 2, 4, 2)];
    let parsed = parse_model(&model("net", 4, 2, layers).to_string()).unwrap();
    assert_eq!(parsed.layers.len(), 2);
    assert_eq!(parsed.layers[0].hidden(), 3);
    assert_eq!(parsed.layers[1].hidden(), 2);
    assert_eq!(parsed.adds_into_q(), vec![false, true]);
}

#[test]
fn perceptrons_alternate_which_half_they_add_into() {
    let layers = vec![
        mlp("relu", 2, 4, 1),
        leapfrog("tanh", 4, 4, 2),
        mlp("tanh", 2, 4, 3),
    ];
    let parsed = parse_model(&model("net", 4, 1, layers).to_string()).unwrap();
    assert_eq!(parsed.adds_into_q(), vec![true, false, false]);
}

#[test]
fn a_layer_that_is_not_reversible_is_refused() {
    let layer = json!({ "kind": "linear", "activation": "relu" });
    let error = error_of(model("net", 4, 1, vec![layer]));
    assert_eq!(
        error,
        WeaveError::Layer {
            index: 0,
            kind: "linear".into()
        }
    );
}

#[test]
fn an_unknown_activation_is_refused_and_the_known_ones_named() {
    let error = error_of(model("net", 4, 1, vec![leapfrog("softplus", 4, 4, 1)]));
    assert!(
        error.to_string().contains("no activation `softplus`"),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("identity, cauchy, softsign, relu, tanh, sigmoid, silu and gelu")
    );
}

#[test]
fn a_weight_of_the_wrong_shape_names_the_tensor() {
    let mut layer = leapfrog("tanh", 3, 4, 1);
    layer["weight"] = json!([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]);
    let error = error_of(model("net", 4, 1, vec![layer]));
    assert!(
        matches!(&error, WeaveError::Shape { name, .. } if name == "w0"),
        "{error}"
    );
}

#[test]
fn the_second_matrix_of_a_perceptron_must_come_back_to_the_width() {
    let mut layer = mlp("relu", 2, 4, 1);
    layer["w2"] = json!([[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]);
    let error = error_of(model("net", 4, 1, vec![layer]));
    assert!(
        matches!(&error, WeaveError::Shape { name, .. } if name == "w2_0"),
        "{error}"
    );
}

#[test]
fn a_model_with_no_layers_or_too_many_outputs_is_refused() {
    assert_eq!(error_of(model("net", 4, 1, vec![])), WeaveError::NoLayers);
    let layers = vec![leapfrog("tanh", 4, 4, 1)];
    assert_eq!(
        error_of(model("net", 4, 5, layers)),
        WeaveError::Outputs {
            outputs: 5,
            width: 4
        }
    );
}

#[test]
fn a_name_that_is_not_an_identifier_is_refused() {
    let layers = vec![leapfrog("tanh", 4, 4, 1)];
    assert_eq!(
        error_of(model("my net", 4, 1, layers)),
        WeaveError::Name("my net".into())
    );
}

#[test]
fn text_that_is_not_json_is_refused() {
    assert!(matches!(parse_model("{"), Err(WeaveError::Json(_))));
}

#[test]
fn attention_alternates_with_perceptrons_and_needs_a_width_its_rows_divide() {
    let layers = vec![mlp("relu", 2, 4, 1), attention(2, 2, 2)];
    let parsed = parse_model(&model("net", 4, 1, layers).to_string()).unwrap();
    assert_eq!(parsed.adds_into_q(), vec![true, false]);
    let bad = error_of(model("net", 5, 1, vec![attention(2, 2, 1)]));
    assert!(matches!(bad, WeaveError::Shape { .. }), "{bad}");
}

#[test]
fn a_convolution_needs_an_odd_kernel_and_a_width_its_channels_divide() {
    let even = error_of(model("net", 6, 1, vec![conv(2, 2, 1)]));
    assert!(even.to_string().contains("an odd kernel"), "{even}");
    let uneven = error_of(model("net", 5, 1, vec![conv(2, 3, 1)]));
    assert!(uneven.to_string().contains("channels divide"), "{uneven}");
}

#[test]
fn a_norm_needs_an_epsilon_weave_can_hold_and_a_gain_for_each_hidden_unit() {
    let mut tiny = mlp_norm("relu", 3, 4, 1);
    tiny["norm"]["eps"] = serde_json::json!(1e-9);
    let error = error_of(model("net", 4, 1, vec![tiny]));
    assert!(error.to_string().contains("at least 1/4096"), "{error}");
    let mut short = mlp_norm("relu", 3, 4, 1);
    short["norm"]["gain"] = serde_json::json!([1.0, 1.0]);
    let error = error_of(model("net", 4, 1, vec![short]));
    assert!(
        matches!(&error, WeaveError::Shape { name, .. } if name == "gain_0"),
        "{error}"
    );
    let parsed =
        parse_model(&model("net", 4, 1, vec![mlp_norm("relu", 3, 4, 1)]).to_string()).unwrap();
    assert_eq!(parsed.tensors().len(), 5);
}

#[test]
fn a_layer_norm_has_a_bias_for_each_hidden_unit_after_its_gain() {
    let parsed =
        parse_model(&model("net", 4, 1, vec![mlp_layer("relu", 3, 4, 1)]).to_string()).unwrap();
    let names: Vec<&str> = parsed.tensors().iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["w1_0", "b1_0", "w2_0", "b2_0", "gain_0", "beta_0"]);
    let mut missing = mlp_layer("relu", 3, 4, 1);
    missing["norm"].as_object_mut().unwrap().remove("bias");
    let error = error_of(model("net", 4, 1, vec![missing]));
    assert!(error.to_string().contains("norm.bias"), "{error}");
    let mut short = mlp_layer("relu", 3, 4, 1);
    short["norm"]["bias"] = json!([0.0]);
    let error = error_of(model("net", 4, 1, vec![short]));
    assert!(
        matches!(&error, WeaveError::Shape { name, .. } if name == "beta_0"),
        "{error}"
    );
}

#[test]
fn a_residual_block_takes_its_chain_length_from_its_contraction_bound() {
    let layer = residual("tanh", 4, 4, 2, 0.3);
    let parsed = parse_model(&model("net", 4, 1, vec![layer]).to_string()).unwrap();
    let roop_weave::Layer::Residual { iters, .. } = &parsed.layers[0] else {
        panic!("a residual layer");
    };
    assert!((5..60).contains(iters), "{iters}");
    assert!(!parsed.layers[0].is_half_step());
    let mut fixed = residual("tanh", 4, 4, 2, 0.3);
    fixed["iters"] = json!(7);
    let parsed = parse_model(&model("net", 4, 1, vec![fixed]).to_string()).unwrap();
    assert!(matches!(
        parsed.layers[0],
        roop_weave::Layer::Residual { iters: 7, .. }
    ));
}

#[test]
fn a_residual_function_that_is_not_a_contraction_is_refused() {
    let error = error_of(model("net", 4, 1, vec![residual("tanh", 4, 4, 2, 3.0)]));
    assert!(
        matches!(error, WeaveError::Contraction { index: 0, bound } if bound >= 0.9),
        "{error}"
    );
    assert!(error.to_string().contains("not a contraction"), "{error}");
}

#[test]
fn a_residual_block_has_no_norm_and_a_chain_of_at_least_two_cells() {
    let mut normed = residual("tanh", 4, 4, 2, 0.3);
    normed["norm"] = json!({ "eps": 0.01, "gain": [1.0, 1.0, 1.0, 1.0] });
    let error = error_of(model("net", 4, 1, vec![normed]));
    assert!(error.to_string().contains("no norm"), "{error}");
    let mut short = residual("tanh", 4, 4, 2, 0.3);
    short["iters"] = json!(1);
    let error = error_of(model("net", 4, 1, vec![short]));
    assert!(error.to_string().contains("at least 2"), "{error}");
}

#[test]
fn a_kept_contraction_sets_the_chain_and_must_be_met_by_the_weights() {
    let mut layer = residual("tanh", 4, 4, 2, 0.3);
    layer["keep_contraction"] = json!(0.8);
    let parsed = parse_model(&model("net", 4, 1, vec![layer.clone()]).to_string()).unwrap();
    let roop_weave::Layer::Residual { iters, keep, .. } = &parsed.layers[0] else {
        panic!("a residual layer");
    };
    assert_eq!((*iters, *keep), (39, Some(0.8)));
    layer["keep_contraction"] = json!(0.01);
    let error = error_of(model("net", 4, 1, vec![layer.clone()]));
    assert!(
        matches!(error, WeaveError::Kept { index: 0, .. }),
        "{error}"
    );
    layer["keep_contraction"] = json!(0.95);
    let error = error_of(model("net", 4, 1, vec![layer]));
    assert!(error.to_string().contains("keep_contraction"), "{error}");
}
