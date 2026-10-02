# roop
The reversible parallel programming language. See more at [uncompute.ai](uncompute.ai)

## Reversible by default

Every `fn`, `struct` and `enum` is reversible: updates are invertible
(`+=`, `-=`, `^=`, `<=>`), temporaries are ancillas that must be restored, and
branches and loops carry exit assertions. Each function compiles to `f` and
its inverse `f_inv`.

`irrev` is the escape hatch, like `unsafe` in Rust. An `irrev fn`, or an
`irrev { ... }` block, lifts those rules and allows destroying values (`x = e`,
`x %= e`) and a `try` that forgets its outcome. Prefer the next
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

## Branches and scaling

`if c { A } else { B } fi e;` asserts `e` after the branch that ran: it must
hold after `A` and fail after `B`. That is what lets the inverse tell the
branches apart, since backward `e` selects the branch and `c` becomes the
assertion. A run that breaks the assertion traps (or rolls back, inside a
`try`), so a program that gets through is always reversible.

`x *= e` and `x /= e` are inverse updates, guarded so that they lose nothing.
Multiplying traps unless `e` is nonzero and the product does not overflow, and
dividing traps unless `e` is nonzero and the division is exact, so `x *= e`
followed by `x /= e` always gives `x` back. For floats only the nonzero check
applies. Lean proves the integer pair cancels, for all 64-bit values.

An update may read another element of the array it writes when the indices
provably differ: constants (`a[0] += a[1]`), offsets (`a[i] += a[i + 1]`), or
what an enclosing `if` says (`if j < i { a[i] += a[j]; } ...`).

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

## Modules

A project is a tree of directories. `Roop.toml` names the roots, and each root
is a directory with a `mod.roop`:

```toml
[modules]
std = "std"
```

A file declares its submodules with `mod name;`, found as `name.roop` or
`name/mod.roop` next to it. Items are private unless marked `pub`. `use` brings
a public item into scope, and every item has one program-wide name made from its
module path, so two modules can use the same name.

```rust
mod util;                              // util.roop or util/mod.roop
use std::blas::igemm;                  // one item
use std::blas::{iaxpy, idot as dot};   // several, with a new name for one
use std::complex::*;                   // every public item; never shadows your own
pub use inner::*;                      // re-export, so others can import it from here
use super::base;                       // the parent module
```

A build only carries the library code it reaches. A session you import is kept
and checked even though no code mentions it. `roop build` and `roop lean` load
the program this way, starting from the file you give them.

## Generic lengths

Array and stack lengths can be parameters. `fn f<N>(x: &mut [i64; N])` is
written once, and `call f<8>(x);` instantiates it at 8. Every instance is an
ordinary function, checked, compiled and proved on its own, so nothing generic
survives to the later stages. Lengths in a call are numbers or the caller's own
parameters, and `N - 1` and similar fold to a constant.

```rust
fn axpy<N>(y: &mut [i64; N], x: &[i64; N], k: &i64) {
    ancilla i: i64 = 0 {
        #[parallel]
        from i == 0 { y[i] += k * x[i]; } loop { i += 1; } until i == N - 1;
        i -= N - 1;        // a counted loop ends at its bound, so i is restored
    }
}
```

## Standard library

`std` is built on the modules above. Each routine is a function, and its reverse
is the compiler's `f_inv`, or `uncall f`.

**std::blas** is the reversible BLAS of Perumalla and Yoginath ("Towards
Reversible Basic Linear Algebra Subprograms"). That paper reverses BLAS routines
either by checkpointing memory or by running the inverse computation. Here the
constructive routines reverse by computation, and the ones that destroy
information save it on the history stack. The prefix is `i` for 64-bit
integers, which reverse exactly, and `d` for f64.

| Level | Routines |
| --- | --- |
| 1 | `axpy`, `swap`, `dot`, `scal`, `rot`, `copy` (history), `fill` (history) |
| 2 | `gemv`, `gemvt`, `ger`, `trmv`, `trsv` |
| 3 | `gemm`, `syrk`, `syr2k`, `mscal`, `trmm`, `trsm` |

Several follow the paper's tables directly. `trsv` and `trsm` are `uncall trmv`
and `uncall trmm`, for unit triangular matrices, the ones whose product loses
nothing. `scal` uses the guarded `*=`, which is the paper's bound on the factor.
`rot` is a rotation written as three shears, each constructive, so it reverses
exactly in integers. `copy` is the paper's destructive routine, and it keeps the
overwritten vector on a history stack. The paper measures accuracy as the RMS
difference after a forward and a reverse call; a test does the same on `dgemm`.

**std::complex** has `Complex` (two f64, the `z` routines) and `Gaussian` (two
i64, with a `g` prefix): `axpy`, `swap`, `dotu`, `gemv`, `geru` and `gemm`.
Gaussian integers are exact, so Lean proves those reverse. Complex scaling is
left out because it loses information at zero.

**std::session** has the sessions of the next section, and `roop/examples/server`
is a server built on them: when a better offer turns up it rolls its last step
back with `uncall` and offers something else.

## Sessions

A session is a protocol for a conversation that either party can roll back, from
"Compliance for reversible client/server interactions" (Barbanera,
Dezani-Ciancaglini and de'Liguoro).

```rust
pub session Negotiation {
    client: checkpoint select {
        sea: checkpoint select { house: end, bung: end },
        mount: select { house: end },
    };
}
```

`offer { a: b }` waits for the other party to pick an action, `select { a: b }`
picks one, `end` is success, `rec x { ... }` repeats, and `checkpoint` marks a
point both parties can return to. With one role the server is its dual. With two
roles the first is the client and the second the server.

The compiler checks that the client is checkpoint compliant with the server:
whatever either does, including rolling back to its last checkpoint, the
conversation never gets stuck before the client has succeeded. It decides this
with the paper's axiomatic system, which terminates because every configuration
is visited once. A failure says what went wrong and how to get there: a label is
sent that the other side does not offer, both sides wait or both choose, or one
side can roll back and the other cannot.

Lean proves the same independently, for the declared pair and for each role with
its own dual (the paper's proposition that every behaviour is compliant with its
dual). A test checks that proposition on thousands of random behaviours as well.
Sessions describe a protocol and are verified as a specification. They are not
yet matched against the code of the tasks that talk over a channel.

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
than the roop checker: it rejects a `match` whose exit assertions overlap, since
then the inverse cannot tell which arm ran, which the checker cannot decide. (An
`if` needs no such check, because its exit assertion is enforced at run time.)

Stacks, `logged` blocks and `try` are covered. A `try` is undone by its outcome
on the states the function produced, so such a function gets the first roundtrip
theorem (`f_inv_f`) and not the second: the inverse of a handler also applies to
states the body would not have failed on. `roop lean` lists these functions.

Loops are covered too. Each loop's entry, exit, body and step are lifted into
top-level definitions, Lean proves that the inverse body and step undo the body
and step, and a general lemma about the reversible loop (`Roop.janus_inv`, in the
prelude) turns that into the loop's own roundtrip. A loop also leaves its exit
test true, which is how Lean knows a counter loop ends at its bound and the
counter ancilla around it is restored. Loops inside loops, calls between
functions and loops in a branch all compose, and a loop whose body is not
reversible is rejected like any other function.

For a `#[parallel]` loop Lean also proves what the checker's disjointness rules
promise: any two iterations commute. Running iteration `v` then `w` ends in the
same state as `w` then `v`, or both fail, whatever the starting state. That is
the condition for threads or a GPU to run them in any order. A loop that reads
what another iteration writes, or accumulates into a shared variable, fails the
proof.

Guarded scaling is covered by two prelude lemmas: when `x *= e` runs, the guard
of `x /= e` holds on the result and gives `x` back, and the other way round. A
session gets theorems that its client is checkpoint compliant with its server,
by a Lean version of the same decision procedure that `decide` evaluates.

Not covered: floating point is translated with no roundtrip claim, since
`x + k - k` need not equal `x`, and that includes any struct with an f64 inside;
irreversible functions get a forward model only;
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
