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
    ancilla i: i64 = 0;
    #[parallel]
    from i == 0 { y[i] += k * x[i]; } loop { i += 1; } until i == N - 1;
    i -= N - 1;        // a counted loop ends at its bound, so i is restored
}
```

A call that leaves the lengths out gets them from its arguments: `call axpy(y, x, k)`
is `call axpy<4>(y, x, k)` when `y` is a `[i64; 4]`, and `<N>` when it is the
caller's own `[i64; N]`. This works from parameters, ancillas and string
literals, through fields and indices, and for several lengths at once. A call the
arguments do not settle still needs its lengths written out.

## Ancilla declarations

An ancilla that lasts to the end of its block does not need a block of its own:

```rust
fn scale_rows<M, N>(a: &mut [[i64; N]; M], k: &i64) {
    ancilla i: i64 = 0;          // lasts to the closing brace
    from i == 0 {
        ancilla j: i64 = 0;      // and this one to the end of the loop body
        from j == 0 { a[i][j] += k; } loop { j += 1; } until j == N - 1;
        j -= N - 1;
    } loop { i += 1; } until i == M - 1;
    i -= M - 1;
}
```

`ancilla x: T = e;` followed by statements means `ancilla x: T = e { statements }`,
so it is checked the same way, and `roop fmt` writes an ancilla that ends its
block this way. The braces stay for an ancilla with something after it.

## Bytes and text

`u8` is an unsigned byte. It adds, subtracts and xors with wrap-around, compares
and divides without sign, and an integer literal in a place that holds a `u8`
is one. Casts convert between the number types: `n as u8` keeps the low byte,
`c as i64` widens, `x as f64` converts, and `x as i64` from a float saturates, so
a number out of range or a NaN is never undefined. `b'a'` is a byte, and
`"hello\n"` is a read-only `[u8; 6]`, which can be an argument to a `&[u8; N]`
parameter and nothing else; the escapes are `\n \t \r \0 \\ \" \' \xHH`. Text is
an array of bytes and a length, and the bytes after the length are zero, so
appending adds into zeros and running the append backward takes it off again.

## Programs

A program starts at `main`, and it is a reversible function like any other, with
no `irrev`. `fn main(status: &mut i64)` leaves the exit status in `status`, and
`fn main()` exits with 0. `roop run` builds and runs it, and `roop build` makes
an executable when there is a `main`:

```rust
use std::io::println;

fn main() {
    call println("Hello, world!");
}
```

```
roop run examples/bin/hello.roop
roop run examples/bin/echo.roop a b c      # what follows the file is the program's
```

`extern fn name<N>(params);` declares a function the runtime provides. It takes
its arguments by reference like any roop function, and is irreversible.
`extern world fn` is the reversible kind, which the next section explains. The
library declares them in `std::sys`, and the modules built on it, `io`, `fs`,
`process`, `text` and `math`, are what programs use. They are in `roop/std` and
`roop/examples/bin` has `hello`, `echo`, `cat`, `wc`, `rot13`, `fib`, `copy`, `ls`
and `undo`.

## The world

Output cannot be unprinted and a file that was overwritten is gone. Reversible
languages mostly leave this outside, and reverse only the program. roop makes
the world something a program can run backward, for as long as nobody has seen
or touched it for good. Three ideas do it, each from the literature.

**Output is pending until it is committed.** `print` adds to a queue the runtime
holds, and its inverse takes the last of it back. Nothing is shown until
`std::io::commit`, the end of the program, or a read from the real standard
input, since whoever is to answer must see the question. So `uncall greet()`
un-prints, and a `try` that fails shows nothing of what it wrote: the rollback
that already undoes a failed `try` runs the inverse of each print. After a
commit the output cannot be taken back, and asking to is a runtime error. This
is Bennett's trick with the world as the copy: compute, copy out, uncompute
([Bennett 1973](https://mathweb.ucsd.edu/~sbuss/CourseWeb/Math268_2013W/Bennett_Reversibiity.pdf)),
and `print_int` is written that way, building the digits in a buffer, writing them,
and running the building backward. It is also the shape of the log monads that
[Heunen and Karvonen](https://arxiv.org/abs/1505.04330) show reversible: a
computation with an effect is reversible when the monad is Frobenius, which
holds for a log kept in a group and fails for a list under concatenation. A
pending queue is not a group, so it is reversed by each write being the dagger
of its take-back, like a stack, the same reason roop's history stack is.

**Input is a tape that remembers.** `read_line` takes the next line from the
real input, or from lines that were put back, and the inverse of a read puts its
line back, so the next read gets it again. This is what record and replay
debuggers do: they record what the outside world told the program, so a run can
be repeated and reversed ([rr](https://en.wikipedia.org/wiki/Rr_(debugging))). The
clock works the same way: a time that was read is kept, so reading again after an
undo gives the same time.

**A file operation keeps what it replaced.** Writing, appending, removing,
renaming, copying and making directories each leave an entry in a journal, and
their inverses restore the file, after checking that it is as the operation left
it. If someone changed the file in between, the runtime stops rather than lose
their change. Reads are reversible by being repeatable: running one backward
empties its results, and running it again gives the same, since the files were
put back.

A result goes into a place that is zero, like a `pop` or a `recv`, because the
old value would be lost and the inverse could not bring it back; the runtime
refuses a place that is not zero. So a loop that reads lines has to empty its
buffer before the next read, and `keep` is how.

**Letting go with `keep`.** `keep x;` hands the value of `x` to the world, which
remembers it, and leaves `x` zero. Run backward it takes the value back, so
nothing is destroyed: the information moved from the program to the world's
history, a stack that lives until the process ends, which is when the operating
system finally erases it. This is the log of Heunen and Karvonen's reversible
monads, made general: any value can be a log entry, because each entry has its
take-back. It is also what makes `main` reversible. An ancilla that starts at
zero is released by a `keep` of the whole variable at the end of its block, or
at the end of each round of a loop that touches it, wherever it was changed
before, and the checker accepts that in place of an inverse.

**`auto ancilla` writes the `keep` for you.** It works like a lifetime in Rust:
you say what a variable is, and the compiler places the end of it. `auto ancilla
x: T = 0;` is an ancilla that is let go of with `keep x;` where its block ends,
so a buffer declared inside a loop body is let go of at the end of every round,
and the next round starts from zero. It must start at zero. A loop that reads
lines is then:

```rust
auto ancilla eof: i64 = 0;
auto ancilla lines: i64 = 0;
from lines == 0 {
    auto ancilla line: [u8; 4096] = 0;
    auto ancilla len: i64 = 0;
    call read_line(line, len, eof);
    if eof == 0 { call print_buf(line, len); call print("\n"); } fi eof == 0;
} loop { lines += 1; } until eof == 1;
```

What stays out of the loop body is what the loop itself looks at, here `eof` and
`lines`, so those are let go of after it.

**Explicit lifetimes.** Where a variable is let go of is its lifetime, and by
default it is the end of its block. Name a shorter one with a label, the way Rust
names a region: `'round: from ...` labels a loop, or `'part: { ... }` a block, and
`auto<'round> ancilla x: T = 0;` declares `x` outside it but lets go of it at the
end of every run of that block, as well as where its own block ends. So the
buffer can be declared once, among the others, and still start each round empty:

```rust
auto<'round> ancilla line: [u8; 4096] = 0;
auto<'round> ancilla len: i64 = 0;
'round: from lines == 0 {
    call read_line(line, len, eof);
    ...
} loop { lines += 1; } until eof == 1;
```

A label that nothing follows, or that is on anything but a loop or a block, is an
error. Labels are only names for `auto`; they change nothing else. Write `keep` by hand when you want it
somewhere else, or `ancilla` when you want the checker to prove the ancilla is
undone, which is the stronger guarantee and costs no history.

The history is a stack, so a function is run backward by exactly reversing it:
do not `uncall` something that kept values after other code has kept values of
its own since, since the entries come back last in, first out. A kept value is
one you are done with. Every `roop test` runs a test backward as well, which
checks that the kept values come back. Parallel loops and
concurrent tasks may not call anything that changes the world or `keep`, since
the world has an order and they have none; the checker rejects it.

```rust
fn announce() {
    call println("step one");
    call println("step two");
}

fn main() {
    auto ancilla failed: bool = false;
    try {
        call announce();
        expect false;                      // the try fails
    } catch_rollback {
        call println("rolled back: nothing from announce was shown");
    } -> failed;
}
```

That prints only the handler's line. Tests may use all of this: every `roop test`
runs forward and then backward, and checks that no output is left pending. Files
the tests wrote are gone again, which `roop/tests/files.roop` checks. `std::io`,
`std::fs` and `std::process` are written this way, and Lean does not model the
world: functions that reach it are skipped and listed.

## Einsum

`einsum fn name: T = "subscripts";` writes a tensor contraction as loops:

```rust
einsum fn matmul: f64 = "ij,jk->ik";

test matmul_of_two_by_two {
    out: [[f64; 2]; 2], x0: [[f64; 2]; 2], x1: [[f64; 2]; 2];
    x0[0][0] += 1.0; x0[0][1] += 2.0; x0[1][0] += 3.0; x0[1][1] += 4.0;
    x1[0][0] += 5.0; x1[0][1] += 6.0; x1[1][0] += 7.0; x1[1][1] += 8.0;
    call matmul(out, x0, x1);              // the lengths come from the arguments
    expect out[0][0] == 19.0 && out[1][1] == 50.0;
}
```

The function is `matmul(out, x0, x1)`: `out` gets the sum, over the labels not in
the output, of the product of the elements of the operands. It is added into
`out`, so a zero `out` ends up holding the result and running it backward takes
it off. Each label becomes a length, `n_i`, `n_j`, in order of first appearance.
The loops over the output labels are parallel, since each iteration writes its
own part of `out`, and the sums are sequential, so the result is the same on
every run. Without `->` the output is the labels that appear once, in
alphabetical order, as in NumPy. A label used twice in one operand takes the
diagonal, so `"ii->"` is the trace.

The `einsum` package, `roop/einsum`, has `dot`, `outer`, `matvec`, `matmul`,
`transpose`, `trace`, sums, a bilinear form, batched matrix products, the
contraction of three indices with a matrix, and the two products of attention,
`scores` (`"bqd,bkd->bqk"`) and `attend` (`"bqk,bkd->bqd"`), each with an `i` (64-bit
integer) and a `d` (f64) version. Their tests are checked against sums worked out by
hand, in `roop/tests/einsum.roop`. Because a contraction is reversible, so is its
gradient: the vector-jacobian product of an einsum is another einsum with the
labels moved, such as `"ik,jk->ij"` for the gradient of `matmul` with respect to
its first operand.

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

**std::io**, **std::fs**, **std::process**, **std::text** and **std::math** are what
programs are made of. `io` has `print`, `println`, `print_int`, `read_line` and
`commit`; `fs` has `exists`, `size`, `read_file`, `write_file`, `append_file`,
`remove`, `rename`, `copy_file`, `make_dir`, `remove_dir` and `list_dir`; `process`
has the arguments, the environment, the clock and `exit`; `text` has `put`,
`put_int` (decimal, reversible), `eq`, `copy` and `parse_int`; `math` has `ipow`,
`imax`, `iabs`, `isign` and the Fibonacci pair `fibpair`, which is injective and
runs backward from a pair to its index.

**std::session** has the sessions of the next section, and `roop/examples/server`
is a server built on them: when a better offer turns up it rolls its last step
back with `uncall` and offers something else.

## weave

weave is a small neural-network library written in roop, built on one fact: a
leapfrog step is exactly invertible, whatever force it uses. A network is a
stack of leapfrog layers on a state `(q, p)`, and backpropagation walks the
layers backward, rebuilding each layer's input from its output with `uncall`
while it carries the adjoints along. Nothing is stored for the backward pass, so
memory does not grow with depth.

A layer is one step of the Hamiltonian `|p|^2 / 2 + V(q)` with
`V(q) = sum log(1 + z^2) / 2` and `z = W q + b`:

```
p += -h/2 * W^T sigma(W q + b)      sigma(z) = z / (1 + z^2)
q += h * p
p += -h/2 * W^T sigma(W q + b)
```

Every line adds something to a place it does not read, so it has an exact
inverse. Numbers are fixed point (an integer is the value times 4096), which
makes the reversal exact to the bit and lets Lean prove it; the rounding in a
product is the same going forward and back.

```rust
use weave::step;

// One step of gradient descent on a batch of B samples through L layers of
// width N, with K outputs.
call step<N, L, K, B>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr);
```

`weave::grad` is the reversible core: the loss and the gradients of one sample,
with the input rebuilt at the end. `weave::forward` is the network alone, and
`forward_inv` is the network run backward. `weave::step` adds the one irreversible
part, clearing the buffers between samples, in an `irrev` function.

A test checks the gradients against central finite differences on a double
precision copy of the network (they agree to about 1%, which is the 4096 grid)
and that the input comes back bit for bit. Another trains XOR to a 50x lower
loss. Lean proves the forward pass exactly reversible, layer by layer.

This is a research library, not a framework. It has one layer type, one loss and
a fixed activation, because roop has no function arguments to build a graph
from, and no automatic differentiation: each layer's adjoint is written out. A
test measures throughput: `cargo test -p roop --test weave --release --
--ignored --nocapture`.

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

A reversible loop over a finite state also ends, which Lean proves in the
prelude as `Roop.janus_terminates` (the result of Yokoyama, Axelsen and Glück
that the Glück and Yokoyama paper cites): if the run went on for ever, its
states would all be different, because a repeat leads back, step by step,
through the inverse body and step, to the entry state, and the entry assertion
tells that state apart from every later one; there are only finitely many
states, so that cannot be. A loop whose state is numbers, bools and arrays of
them gets `loop_terminates`: given enough fuel it does not run out, unless one
of its own pieces, an inner loop, does. A loop with a stack in its state gets
no such theorem, since the model's stack does not bound its length.

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

## Tests

A test is a reversible computation whose verdict is the exception monad's
outcome. It names its fixtures, which start at zero, and `expect` is a statement
that does nothing but check:

```rust
test fibpair_of_4 {
    n: i64, a: i64, b: i64;
    n += 4;
    call fibpair(n, a, b);
    expect n == 0 && a == 5 && b == 8;
}

// Inverse computation as the specification: from the pair, the index.
test fibpair_decodes_its_index {
    n: i64, a: i64, b: i64;
    a += 5;
    b += 8;
    uncall fibpair(n, a, b);
    expect n == 4;
}
```

`expect e;` is `if e { } fi true;`, which fails when `e` is false. `roop test`
runs every test twice: forward, and then backward, which is the test's own
inverse, and requires every fixture to be zero again. So a test needs no
teardown, no test leaks into another, and every test also checks that what it
calls undoes itself. Each test runs in a process of its own, which is why tests
can call parallel kernels and irreversible code that a `try` could not roll
back.

```
roop test                  # every file of the project that has tests
roop test tests/blas.roop --filter gemm
roop test --lean           # run each test on the Lean model too, and compare
```

When a test fails, it is run again a statement at a time and the state before
each statement is shown, so you see how the state got to where the check failed.
That is reversible debugging, as in the paper's Fig. 9, and it works because
nothing was overwritten: the states before are all there is to know.

```
test a_wrong_expectation ... FAILED: an expectation or another check failed
    state before each statement:
      line 28: n += 4;                               n=0 a=0 b=0
      line 29: call fibpair(n, a, b);                n=4 a=0 b=0
      line 30: expect a == 6;                        n=0 a=5 b=8   <- stopped here
```

`--lean` evaluates each test on the Lean model as well and compares verdicts, so
a disagreement is a bug in the compiler or in the model. Fixtures are numbers,
bools, arrays of them, or a stack of `i64`. Tests live in files you give to
`roop test`, such as `roop/tests/blas.roop`, and `roop build` leaves them out.

## Bennett

`bennett fn quote = settle_to;` writes the compute, copy, uncompute version of
a function (Bennett 1973, section 6.3 of Glück and Yokoyama). Say `settle_to`
destroys information, so it logs to a history stack. `quote` has the same
parameters without the history, and a zero output `<name>_out` for each mutable
one. It runs `settle_to` with a fresh history, copies what each mutable
parameter became into its output, and runs `settle_to` backward. The inputs are
back, the history is empty, and what remains depends only on what `settle_to`
computes, not on how: the paper's extensional garbage, as against the trace of
a Landauer embedding, which is intensional. A generic target keeps its lengths
(`call quote<8>(..)`), and the result is an ordinary function that Lean proves
reversible like any other. What it copies must be numbers, bools, structs and
arrays of them.

```rust
fn settle_to(balance: &mut i64, amount: &i64, h: &mut Stack<i64, 8>) {
    logged h { balance = amount * 2; }
}

bennett fn quote = settle_to;       // quote(balance, amount, balance_out)
```

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
`#[parallel(cuda)]` pin a target. Adjacent loops with the same iteration space
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
targets = ["cpu", "metal", "cuda"]
```

Metal kernels are emitted as AIR and built with Apple's `metal` tools; NVPTX
kernels go through LLVM's PTX backend and need a CUDA driver at run time. The
CUDA launcher has not yet run on NVIDIA hardware.

## Formatting

`roop fmt` rewrites `.roop` files in place, like `cargo fmt`. With no path it
formats every file of the project, found from the nearest `Roop.toml`.

```
roop fmt                    # the whole project
roop fmt std/blas           # a directory or a file
roop fmt --check            # list what would change, change nothing, fail if any
roop fmt --width 80         # override the line width
roop fmt --stdin < f.roop   # filter text
```

```toml
[format]
max_width = 88
indent = 4
```

The formatter prints the program from its syntax tree, so the layout never
depends on how the file was written. Calls, parameter lists and long
expressions wrap at the width, one item to a line with a trailing comma. A block
of one simple statement stays on the line of its header when it fits, and an
empty `else` or `loop` is left out. Comments stay where they are: above a
statement, beside it, or last in its block, and blank lines between statements
are kept. A comment inside a statement header or a declaration, such as between
the arguments of a call, would have to move, so the formatter refuses the file
and names the line. After formatting it parses the result again and fails if
the program differs.

## Editors

`editors/` has a Tree-sitter grammar for roop and an extension for Zed, with
highlighting, an outline, bracket matching and indentation. See
`editors/README.md` for installing it, and for what a language server would
build on.
