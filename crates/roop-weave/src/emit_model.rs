use crate::{
    Model, emit_backward, emit_forward, emit_grad, emit_load, emit_main, emit_step, emit_tests,
    emit_train,
};

/// The roop file for a model: its forward pass, backward
/// pass, gradient and training step. With a `batch` it adds `<name>_train`, which
/// C can call to train on that many samples, and with `tests` the test that
/// checks the rest against a reference in doubles, with `<name>_load` to give it
/// the weights. The loader is left out otherwise, since Lean cannot prove a long
/// run of writes into the rows of a matrix and a model without it is proved whole.
pub fn emit_model(model: &Model, tests: bool, batch: Option<usize>, main: bool) -> String {
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
    if tests || main {
        parts.push(emit_load(model));
    }
    if main {
        parts.push(emit_main(model));
    }
    if tests {
        parts.push(emit_tests(model));
    }
    parts.join("\n")
}
