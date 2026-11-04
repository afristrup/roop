# Lean verification

`roop lean prog.roop --check` translates a checked program to Lean 4. The
translator models mutable parameters as values returned from pure functions.
It emits a forward function and an inverse function for each reversible Roop
function.

Lean proves `f_inv_f` and `f_f_inv` when both round trips are valid. It also
proves restoration obligations for ancillas. A reversible `try` gets only the
round trip that is valid for its guarded inverse. A handler's inverse may be
defined for states where the body did not fail, so the other theorem would be
too strong.

The loop translation lifts a loop's entry assertion, exit assertion, body, and
step to Lean definitions. The prelude proves that the inverse loop recovers the
previous state. It can also prove termination for finite loop state.

For a checked `#[parallel]` loop, Lean proves that two distinct iterations
commute. This is the property a CPU or GPU needs to execute iterations in an
arbitrary order.

The proof model does not claim every property of every backend. Floating point
has no round-trip theorem. Channels, concurrent tasks, and a `try` without an
outcome are unsupported by the translation. The separate CSLib Lean project
models execution reversal, but it does not prove that the Rust compiler or
runtime refines that model.

[Previous: Frobenius monads](04-frobenius.md)

[Next: the runtime and external world](06-runtime.md)
