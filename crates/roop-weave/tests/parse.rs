mod support;

use roop_weave::{WeaveError, parse_model};
use serde_json::json;
use support::{leapfrog, mlp, model};

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
    let error = error_of(model("net", 4, 1, vec![leapfrog("gelu", 4, 4, 1)]));
    assert!(
        error.to_string().contains("no activation `gelu`"),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("identity, cauchy, softsign, relu and tanh")
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
