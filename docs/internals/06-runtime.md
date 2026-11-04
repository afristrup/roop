# The runtime and external world

`roop-rt` provides the functions that generated programs call for parallelism,
channels, kernels, I/O, and the reversible world. Its public exports are in
[`crates/roop-rt/src/lib.rs`](../../crates/roop-rt/src/lib.rs).

The world keeps a journal. Output enters a pending queue, so a reverse run can
remove it. `commit` makes pending output visible and prevents its removal.
Input is retained so the inverse of a read can put it back. File operations
record the bytes and metadata needed by their inverse functions.

`keep` moves a value into process history. `history_limit` bounds how much that
history can hold. `forget` drops it permanently, so a later reverse operation
that needs it fails. These operations move an information cost to the runtime.
They do not remove that cost.

The runtime also dispatches verified-disjoint loops to CPU workers, Metal, or
CUDA. The compiler only sends a loop to these paths after its own alias and
interference checks. Target code may therefore assume each iteration has an
independent set of places.

[Previous: Lean verification](05-lean.md)

[Next: Weave code generation](07-weave.md)
