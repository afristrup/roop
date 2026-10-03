use crate::{
    Model, emit_backward, emit_batch_tests, emit_forward, emit_grad, emit_load, emit_step,
    emit_tests, emit_train,
};

/// The roop file for a model: its forward pass, backward
/// pass, gradient and training step. With a `batch` it adds the same four for a
/// batch of samples run through the network together (`_forward_batch`, and so
/// on, which use weave's matrix products), and `<name>_train`, which C can call to
/// train on that many samples. With `tests` it adds the tests that check the rest
/// against a reference in doubles, with `<name>_load` to give them the weights.
/// The loader is left out otherwise, since Lean cannot prove a long run of
/// writes into the rows of a matrix and a model without it is proved whole.
pub fn emit_model(model: &Model, tests: bool, batch: Option<usize>) -> String {
    let mut parts = vec![
        "// Written by roop-weave. Every layer is reversible, so the backward pass\n// rebuilds the input from the output and stores no activation.\n\nuse weave::*;\n"
            .to_string(),
        emit_forward(model, false),
        emit_backward(model, false),
        emit_grad(model, false),
        emit_step(model, false),
    ];
    if let Some(batch) = batch {
        parts.push(emit_forward(model, true));
        parts.push(emit_backward(model, true));
        parts.push(emit_grad(model, true));
        parts.push(emit_step(model, true));
        parts.push(emit_train(model, batch));
    }
    if tests {
        parts.push(emit_load(model));
        parts.push(emit_tests(model));
        if let Some(batch) = batch {
            parts.push(emit_batch_tests(model, batch));
        }
    }
    parts.join("\n")
}
