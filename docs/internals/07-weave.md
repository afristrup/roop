# Weave code generation

`roop-weave` turns a restricted model description into a Roop program. Its
public entry points are in [`crates/roop-weave/src/lib.rs`](../../crates/roop-weave/src/lib.rs).
The parser validates the model name, tensor shapes, layer sequence, and output
shape before emission begins.

`emit_model` writes the common module import and then emits forward, backward,
gradient, and step functions. Optional flags add batch code, a loader, a command
line `main`, and tests. The emitted source goes back through the normal Roop
checker, compiler, and Lean translator.

The generated backward pass calls the inverse layer code to recover the input
state. It carries adjoints while it walks backwards. It therefore stores no
activation array for every depth. It does still store model weights, gradients,
and the current state.

The generator also writes reference tests. They compare the emitted fixed-point
code against a double-precision reference implementation. The Lean tests prove
the emitted reversible code for supported model sizes. The normalisation block
has a stated limit. Its Newton chain has not been accepted by Lean within the
project's proof limit.

The model format comes from the Python exporter in
[`crates/roop-weave/python`](../../crates/roop-weave/python). Keep the exporter,
model parser, reference implementation, and emitted declarations in agreement
when adding a layer field.

[Previous: the runtime and external world](06-runtime.md)

[Back to the contents](README.md)
