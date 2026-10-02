# roop
The reversible parallel programming language. See more at [uncompute.ai](uncompute.ai)

## Reversible by default

Every `fn`, `struct` and `enum` is reversible: updates are invertible
(`+=`, `-=`, `^=`, `<=>`), temporaries are ancillas that must be restored, and
branches and loops carry exit assertions. Each function compiles to `f` and
its inverse `f_inv`.

`irrev` is the escape hatch, like `unsafe` in Rust. An `irrev fn`, or an
`irrev { ... }` block, lifts those rules and allows destroying values (`x = e`,
`*=`, `/=`, `%=`) and a `try` that forgets its outcome. Prefer the next
section, which keeps both reversible. A function with `irrev` code has no inverse,
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

## History, rollback and monads

Effects are monads, and the paper "Reversible monadic computing" says which
ones keep a computation reversible: the Frobenius monads, such as a log kept in
a group. roop has two, and each shows up in a function's signature.

**History.** `Stack<T, N>` holds at most `N` values. `push s <- x;` moves `x`
onto the stack and leaves zero, and `pop s -> x;` moves the top into a zero `x`,
so each is the other's inverse. Inside `logged h { ... }` destroying updates are
allowed: each one pushes the value it destroys onto `h`, and backward it checks
the result, pops the old value and puts it back. A function that destroys
information therefore stays reversible, and takes its history as a parameter,
because that is where the information went. A temporary stack is an ancilla that
starts `empty` and must be emptied again, which is Bennett's trick: compute,
copy the result, uncompute.

```rust
fn settle_to(balance: &mut i64, amount: &i64, h: &mut Stack<i64, 8>) {
    logged h { balance = amount * 2; }
}
fn quote(balance: &mut i64, amount: &i64, quoted: &mut i64) {
    ancilla h: Stack<i64, 8> = empty {
        call settle_to(balance, amount, h);
        quoted += balance;
        uncall settle_to(balance, amount, h);      // the history is empty again
    }
}
```

**Failure.** `try { body } catch_rollback { handler } -> failed;` runs the body.
If anything in it fails (an exit assertion, a full or empty stack, an index out
of bounds, a failing callee), the body is undone by running what it already did
backward, which is the dagger of the executed prefix, and then the handler runs
on the restored state. `failed` must be zero going in and is set when the handler
ran. That bit is the exception monad's outcome kept as data, and it is what makes
the statement reversible: backward, it says whether to undo the handler or the
body. The body and handler cannot read or write `failed`, and cannot contain
`irrev` code, tasks, parallel loops or channels, since those could not be undone.
Without `-> failed`, a `try` is the irreversible kind and needs `irrev`.

Rollback is real reverse execution, not a snapshot. Every function a `try` calls
gets a failure-atomic variant that undoes itself and reports failure, so a
failing callee leaves its arguments as it found them. A loop that fails in its
fifth iteration undoes the four finished ones with its own inverse.

```rust
fn transfer(payer: &mut i64, payee: &mut i64, amount: &i64,
            refusals: &mut i64, refused: &mut bool) {
    try {
        payer -= amount;
        payee += amount;
        if payer >= 0 { } fi payer >= 0;
    } catch_rollback {
        refusals += 1;
    } -> refused;
}
```

`roop/examples/history` has these. `crates/roop-lean/lean/Frobenius.lean` states
the theory they rest on and checks it in Lean: the Frobenius law holds for every
group (so for the `try` outcome, `Bool` under xor, and for counters under `+`);
it fails for lists under concatenation, which is why a stack is not reversed by a
structure of its own but by each pop being the dagger of its push; and the
dagger of a computation that logs to a group is the computation that runs
backward and logs the inverse.

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

Stacks, `logged` blocks and `try` are covered. A `try` is undone by its outcome
on the states the function produced, so such a function gets the first roundtrip
theorem (`f_inv_f`) and not the second: the inverse of a handler also applies to
states the body would not have failed on. `roop lean` lists these functions.

Loops are covered too. Each loop's entry, exit, body and step are lifted into
top-level definitions, Lean proves that the inverse body and step undo the body
and step, and a general lemma about the reversible loop (`Roop.janus_inv`, in the
prelude) turns that into the loop's own roundtrip. Loops inside loops, calls
between functions and loops in a branch all compose, and a loop whose body is
not reversible is rejected like any other function.

For a `#[parallel]` loop Lean also proves what the checker's disjointness rules
promise: any two iterations commute. Running iteration `v` then `w` ends in the
same state as `w` then `v`, or both fail, whatever the starting state. That is
the condition for threads or a GPU to run them in any order. A loop that reads
what another iteration writes, or accumulates into a shared variable, fails the
proof.

Not covered: floating point is translated with no roundtrip claim, since
`x + k - k` need not equal `x`; irreversible functions get a forward model only;
a `try` without an outcome, channels and concurrent tasks are skipped and listed; borrow
exclusivity and the disjointness of `#[concurrent]` tasks are checked by the
compiler but not stated as theorems; the commutation theorem is skipped for a
parallel loop with another loop in its body.

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
