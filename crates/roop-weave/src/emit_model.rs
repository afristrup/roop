use crate::{
    Model, emit_backward, emit_forward, emit_grad, emit_load, emit_step, emit_tests, emit_train,
};

/// The roop file for a model: its forward pass, backward
/// pass, gradient and training step. With a `batch` it adds `<name>_train`, which
/// C can call to train on that many samples, and with `tests` the test that
/// checks the rest against a reference in doubles, with `<name>_load` to give it
/// the weights. The loader is only for the tests, so it is left out otherwise.
pub fn emit_model(model: &Model, tests: bool, batch: Option<usize>) -> String {
    let mut parts = vec![
        "// Written by roop-weave. Every layer is reversible, so the backward pass\n// rebuilds the input from the output and stores no activation.\n\nuse weave::*;\n"
            .to_string(),
        emit_forward(model),
        emit_backward(model),
        emit_grad(model),
        emit_step(model),
    ];
    if let Some(batch) = batch {
        parts.push(emit_train(model, batch));
    }
    if tests {
        parts.push(emit_load(model));
        parts.push(emit_tests(model));
    }
    parts.join("\n")
}
