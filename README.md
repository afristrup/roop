# roop
The reversible parallel programming language. See more at [uncompute.ai](uncompute.ai)

## Reversible by default

Every `fn`, `struct` and `enum` is reversible: updates are invertible
(`+=`, `-=`, `^=`, `<=>`), temporaries are ancillas that must be restored, and
branches and loops carry exit assertions. Each function compiles to `f` and
its inverse `f_inv`.

`irrev` is the escape hatch, like `unsafe` in Rust. An `irrev fn`, or an
`irrev { ... }` block, lifts those rules and allows destroying values (`x = e`,
`*=`, `/=`, `%=`) and `try ... catch_rollback`. Such a function has no inverse,
reversible code cannot call it outside an `irrev` block, and nothing can
`uncall` it. The safety rules (borrows, parallel loops, concurrent tasks) still
apply inside `irrev`.

```rust
fn add(x: &mut i64, k: &i64) { x += k; }       // reversible: add and add_inv
irrev fn wipe(x: &mut i64) { x = 0; }          // no inverse
fn both(a: &mut i64, b: &mut i64, k: &i64) {
    call add(a, k);
    irrev { call wipe(b); }                      // this function is irreversible too
}
```

## Verification in Lean

```
roop lean prog.roop --check
```

translates the program to a pure Lean 4 model in the style of Aeneas: no
mutation, each `&mut` parameter goes in and comes back as a result, a borrow
reads a place into a local and writes it back, an ancilla is a temporary with a
check that it is restored, and every reversible function `f` gets a second
function `f_inv`. Integers are 64-bit bit-vectors, so arithmetic wraps exactly
like the compiled code.

For each reversible function Lean checks two theorems, `f_inv_f` and `f_f_inv`:
running `f` then `f_inv` returns the inputs, and `f_inv` then `f` returns the
outputs, so no information is lost. Functions with ancillas also get a theorem
that neither direction ever fails with an unrestored ancilla. Lean is stricter
than the roop checker: it rejects a branch whose exit assertion does not
actually identify which side ran, which the checker cannot decide.

Not yet covered: loops are translated, but their theorems need an induction and
are reported as open; floating point is translated with no roundtrip claim,
since `x + k - k` need not equal `x`; irreversible functions get a forward model
only; `try`, channels and concurrent tasks are skipped and listed.

## Concurrent tasks and channels

Adjacent `#[concurrent]` blocks run as tasks on their own threads. They share no
mutable state and talk over channels. A send moves a value out (leaving zero)
and a receive moves one in, so each is the other's inverse, and running a task
group backward sends the messages back the other way.

```rust
fn serve(x: &mut i64, y: &mut i64, z: &mut i64) {
    chan req: i64 { chan rep: i64 {
        #[concurrent] { send req <- x; recv rep -> y; }
        #[concurrent] { recv req -> z; z += 10; send rep <- z; }
    } }
}
```

The checker proves each group deadlock-free and every channel drained, with one
sender and one receiver per channel (in the spirit of session-type compliance).
A `try ... catch_rollback` around a whole group rolls all of its tasks back
together to a checkpoint of what they wrote, then runs the handler. A task may
not roll back half of a conversation, so channel operations may not sit under a
`try` inside a task.

## Parallel loops

A counted `from` loop marked `#[parallel]` runs its iterations at once. The
checker proves every write is private to its iteration (indexed by the loop
variable) and that nothing reads another iteration's write, so the loop is safe
to run, and to reverse, in any order.

```rust
fn axpy(a: &mut [i64; 1000], b: &[i64; 1000], i: &mut i64, k: &i64) {
    #[parallel]
    from i == 0 { a[i] += b[i] * k; } loop { i += 1; } until i == 999;
}
```

A bare `#[parallel]` lets the compiler pick CPU threads or a GPU. It estimates
each device as the slower of arithmetic and memory traffic plus a fixed launch
cost, with constants measured on an Apple M4. On that machine the CPU wins for
streaming loops and for light arithmetic; the GPU wins once a loop does enough
64-bit integer work per element (about 50 dependent multiply-adds and up). `#[parallel(cpu)]`, `#[parallel(metal)]` and
`#[parallel(nvptx)]` pin a target. Adjacent loops with the same iteration space
fuse into one loop (one GPU kernel) when the merged body is still safe.

## Building

```
roop build prog.roop --link main.c -o prog
```

`Roop.toml` configures it. Remove a target to opt out of it, or set
`auto = false` so a bare `#[parallel]` always means CPU threads:

```toml
[parallel]
auto = true
targets = ["cpu", "metal", "nvptx"]
```

Metal kernels are emitted as AIR and built with Apple's `metal` tools; NVPTX
kernels go through LLVM's PTX backend and need a CUDA driver at run time. The
CUDA launcher has not yet run on NVIDIA hardware.
