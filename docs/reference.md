# roop reference

The whole language, one section at a time. For what roop is and why, start at the [README](../README.md).

## Reversible by default

Every `fn`, `struct` and `enum` is reversible: updates are invertible
(`+=`, `-=`, `^=`, `<=>`), temporaries are ancillas that must be restored, and
branches and loops carry exit assertions. Each function compiles to `f` and
its inverse `f_inv`.

An update may not read the place it writes (`x += x` is not injective), and a
call may not give a place to a `&mut` parameter while another argument overlaps
it: `call g(a, a)` or `call g(v[i], v[j])` is refused, since `g` would change what
it reads under its own update and `uncall g` could not put it back. Elements of an
array whose indices provably differ, such as `v[0]` and `v[1]`, do not overlap.
This is also what lets the compiler treat every reference parameter as the only
way to reach what it points at, as Rust does, which makes loops over them
vectorize.

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
survives to the later stages. Lengths in a call are numbers, the caller's own
parameters or products of them (`B * 8`), and `N - 1` and similar fold to a constant.

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

**What the history costs.** A kept value is not erased, it is moved: the world
holds it until the process ends or the program calls `std::process::forget`. So
a reversible `main` is reversible in the logical sense, and the physical cost of
erasing information (Landauer's) is paid at the end of the process or at
`forget`, not by `keep`. The history grows with every `keep`, so a program that
runs for long needs a bound, and there are two. `history_limit` in `Roop.toml`
is the most bytes the history may hold; a `keep` that would pass it stops the
program with status 70 and says why, so a leak is a crash you see and not a slow
death. It is unlimited when absent.

```toml
[world]
history_limit = 67108864   # 64 MiB
```

`forget` is the other: it is `irrev`, and it lets go of the history, the input
read so far, the clock readings and the journal of file changes for good. A
server calls it once per request, which makes the unbounded stack a bounded one
per request. Nothing before a `forget` can be run backward afterwards, and
asking to is a runtime error. Where the history is not needed, prefer a plain
`ancilla` that the checker proves is undone: it costs no history at all.

The history is a stack, so a function is run backward by exactly reversing it,
and the checker refuses an `uncall f` when the `call f` before it, in the same
block, has something between it and the `uncall` that kept values of its own: the
entries would come back last in, first out, and they are not `f`'s. A kept value is
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
integer) and a `d` (f64) version. The element type `q12` is fixed point for weave: the
numbers are `i64` holding x * 4096, and each product is divided by 4096, so a
contraction stays on that grid (`qmatvec`, `qmatvec_t`, `qouter`, `qhadamard`,
`qmatmul`, `qmatmul_nt`, `qmatmul_tn`). The element type `q12w` is the fixed
point that weave uses: the products of a sum are added exactly, in 64 bits, and
the sum is divided by 4096 once (`wmatvec`, `wmatvec_t`, `wmatmul`, `wmatmul_nt`,
`wmatmul_tn`, and for a batch `wproject`, `wunproject`, `wscores`, `wattend`,
`wattend_t`). It is more exact than `q12`, and a `q12w` einsum with a sum in it
is two functions: the `i64` einsum `<name>__sum` into a scratch array, and the
quotients added to `out`, with the scratch array taken off again by `uncall`
(which the compiler does by zeroing it, see Building). A contraction with no sum
(`qouter`, `qhadamard`) is the same in `q12` and `q12w`. `iweight_grad`
(`"bsi,bsk->ik"` in `i64`) is the gradient of a shared weight, and is not divided
by 4096. Their tests are checked against sums worked out by
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
stack of reversible layers on a state `(q, p)`, and backpropagation walks the
layers backward, rebuilding each layer's input from its output with `uncall`
while it carries the adjoints along. Nothing is stored for the backward pass, so
memory does not grow with depth.

Its contractions are the `q12` einsums of the `einsum` package, so a project
that uses weave lists both in `Roop.toml`:

```toml
[modules]
weave = "weave"
einsum = "einsum"
```

**The leapfrog layer** is one step of the Hamiltonian `|p|^2 / 2 + V(q)` with
`V'(z) = f(z)` and `z = W q + b`, where the state has width `N`, the hidden layer
has width `M` and `W` is `M` by `N`:

```
p += -h/2 * W^T f(W q + b)
q += h * p
p += -h/2 * W^T f(W q + b)
```

Every line adds something to a place it does not read, so it has an exact
inverse. Numbers are fixed point (an integer is the value times 4096), which
makes the reversal exact to the bit and lets Lean prove it; the rounding in a
product is the same going forward and back.

**The activation** `f` is chosen at run time by a kind number that is only read,
so the branch on it is undone by the same test going back:

| kind | `f(z)` |
| --- | --- |
| 0 | identity |
| 1 | `z / (1 + z^2)`, the force of `log(1 + z^2) / 2` |
| 2 | softsign, `z / (1 + abs(z))` |
| 3 | relu |
| 4 | tanh, as the Pade approximation `z (27 + z^2) / (27 + 9 z^2)` up to 3, then 1 |
| 5 | sigmoid, `1/2 + tanh(z / 2) / 2` |
| 6 | silu, `z sigmoid(z)` |
| 7 | gelu, `z sigmoid(1.702 z)`, the usual approximation |

The slope of each is in `weave::dact`. The tanh is smooth where it joins 1: its
slope `((9 - z^2) / (3 (3 + z^2)))^2` is zero there.

Contractions are the `q12w` einsums: the products of a sum are added exactly and
divided by 4096 once, which is more exact than dividing each and lets the compiler
run them on the matrix unit. The gradient of a matrix is the sum of the products
that `adjoint`, `mlp_vjp`, `mlp_norm_vjp` and `conv_vjp` add up, not divided by 4096, so it is in Q24 (4096
times a Q12 number); the gradient of a bias is in Q12. `sgd_mat` divides by
2^24 when it applies it.

**The perceptron block** is the other reversible layer, from RevNets and NICE:
the state is split in two, and each half takes in a function of the other,

```
q += F(p)        p += G(q)        F(x) = W2 f(W1 x + b1) + b2
```

`F` need not be invertible, since the half it reads is left alone, so any
`Linear, activation, Linear` fits. `weave::mlp` adds one half, and `weave::mlp_back`
runs it backward with the gradients of all four parameters.

**The attention block** is the second half-step, over a state read as `S` rows of `D`
numbers: `y += (Q K^T) V` with `Q`, `K` and `V` the rows times a weight. It has no
softmax, since that needs an exponential, so it is the attention weave can run
backward. Its adjoints are einsums as well (`weave::attn`, `attn_vjp`, `attn_back`).

**The convolution block** is a one-dimensional convolution with zero padding as the third
half-step: the state of width `C * T` is `C` channels of `T` numbers, the kernel size `K` is
odd, and `y += conv(x)` with a weight of `C` rows of `C * K`. It is a product with the
unfolded input, so it is a Q12 einsum like the rest (`weave::conv`, `conv_back`).

**Normalization** is a part of the perceptron block: with `"norm": {"gain": [...], "eps": 0.01}`
the hidden layer is `gain * z / sqrt(mean(z^2) + eps)` before the activation. weave has no
square root that adds into a zero place, so `1 / sqrt(s)` is Newton's method run as a chain of
21 values, 20 steps in four stages of five, each step a one-line function on scalar cells. It
starts from 1/4 and converges for `s` up to 48, and a larger one traps; `eps` cannot be below
1/4096. Lean proves the normalized block and its backward step (`mlp_norm`, `mlp_norm_vjp`,
`mlp_norm_back` at width 2) exactly reversible in about two minutes; each step and each stage
has its own call lemmas, so the proof of the chain is a few calls long instead of unfolding all
20 steps; the batched version (`mlp_norm_batch`) is proved too, in about a minute. This is RMSNorm.

**LayerNorm** is the same block with `"center": true` and a `"bias"` of one number per hidden
unit: `gain * (z - mean(z)) / sqrt(var(z) + eps) + bias`. The mean is taken off into a centered
copy (`weave::mean_of`, `center`), which `rmsnorm` then scales, since the variance of `z` is the
mean square of the centered copy, and the bias is added after. The adjoint of `z` is
`r (g u - mean(g u) - zhat * mean(g u zhat))` with `r = 1 / sqrt(var + eps)` and
`zhat = (z - mean) r`, which the code finds as the adjoint of `rmsnorm` on the centered copy
followed by the centering again (it is its own adjoint); the gain gets `u * zhat` and the bias
`u`. Like every function here each line only adds into a place it does not read, so
`weave::mlp_layer` (with `mlp_layer_back` and the `_batch` forms) is exactly reversible. Its
tensors are `w1, b1, w2, b2, gain, bias`, in that order. Lean proves the block and its
backward step (`mlp_layer`, `mlp_layer_vjp`, `mlp_layer_back` at width 2) exactly reversible.
What once made the kernel time out on `mlp_layer_vjp` was one case of the call-chain proof: a
call that returns two places (`layernorm_vjp` gives the adjoint of `z` and of the gain) left
its result as a pair, so the checks that its undone ancillas are zero could not be turned
into substitutions, the chain rewrite failed, and Lean fell back to unfolding every callee
body. The chain proof now opens a pair result first, so each call keeps its own lemma. The
batched forms (`mlp_layer_batch`) are proved too, by an ignored test, in about three minutes.

**The residual block** is `y = x + F(x)` with `F(x) = W2 f(W1 x + b1) + b2`, the invertible
residual connection of Behrmann et al. when `F` is a contraction, that is when its Lipschitz
constant `L` is below 1. `x + F(x)` writes the place it reads, so it is not a constructive
update and has no `uncall`. weave's block computes `y` into the other half of the state with the
input kept, then clears the input with the inverse that fixed-point iteration finds,
`x_0 = y`, `x_(k+1) = y - F(x_k)`, run as a chain of `K` cells where each step adds into a fresh
cell. On `(q, p)` it is four lines (`weave::residual`):

```
p += q + F(q)       p is y
q -= x_K(p)         q is x - x_K(y), the remnant r
q += p              q is y + r
p -= q              p is -r
```

Every line adds into a place it does not read, so the block is an exact bijection on `(q, p)`
whatever `F` and `K` are, and `uncall` rebuilds `(x, 0)` from the output bit for bit. That is
the guarantee: the forward pass is bit-exactly deterministic and the backward pass reverses it
exactly, with no tolerance, because the iteration is part of the definition of the block and not
an approximation to its inverse. What the contraction buys is the meaning of the output. The
iteration's error shrinks by `L` a step, so `x_K` is within `L^(K-1) |F(x)|` of `x`, the remnant
`r = x - x_K(y)` is then small, and with `p` zero on entry `q` leaves as `x + F(x)` up to `r`
and `p` as `-r`. With `K = ceil(12 ln 2 / ln(1 / L)) + 1` cells the bound is below one unit of Q12
(`L` = 0.5 takes 13 cells, 0.8 takes 39 and the limit 0.9 takes 80), and rounding in `F` leaves
a floor of a few units: in the tests the remnant is zero for `L` near 0.84 and `K` = 50, and a
unit or two when `K` is too short. A block whose `p` is not zero on entry, as after a leapfrog
layer, is still exactly reversible, but then `q` is not `x + F(x)`. Do not read the block as
exactly inverting `y = x + F(x)` on `q` alone: from `q` alone, `x` is the iteration's result,
approximate to `r`. The other direction was not chosen: running the iteration as the forward pass
would need `F` and the chain adjoint to be inverted by a loop of unknown length.

The backward step (`weave::residual_back`) is the exact adjoint of the four lines, taken in
reverse: the third and fourth are `aq -= ap` and `ap += aq`, the second is a chain of its own
(each cell hands `-J^T` of its cotangent to the one before and adds its own to `ap`, with the
gradients of the weights taken off on the way, in `K - 1` steps of `mlp_vjp`), and the first is
`aq += ap + J^T ap`. The gradients are those of the block as computed, which differ from those
of the ideal `x + F(x)` by terms that also shrink as `L^K`. Costs are `K` evaluations of `F`
forward (twice, as the chain is built and then undone), several times that backward, and
`2K` cells of `N` numbers for the chain (`4K` in the backward step). Each cell of the
chain has a twin that holds the step, so that no call reads the array it writes; that is what
lets Lean prove `residual` and `residual_back` at small sizes.

The batched block (`residual_batch`, `residual_back_batch`) runs B samples together. The chain is
the same for every row, so each of its `K` steps is one `mlp_batch` (or `mlp_vjp_batch` going
back) over all the rows, whose cells hold B rows each, and the weights are read once per step
instead of once per sample. Each row is what `residual` makes of it alone, and the gradients are
the sums over the rows, bit for bit; tests compare both against the per-sample forms. On 64
wide rows with a 64 wide hidden layer, 20 cells and 32 samples, forward and backward together
take 3.1 ms against 12.8 ms for 32 per-sample calls (about 4 times faster, release runtime).

A model's residual layer needs `L` below 0.9, which the compiler checks from the weights on
the 1/4096 grid: the spectral norms of `W1` and `W2` (power iteration, taken 1 percent high)
times the slope of the activation (1 for the identity, cauchy, softsign, ReLU and tanh, 0.2501
for sigmoid, 1.1205 for SiLU and GELU). A model above it is refused, with the bound, and its
chain is `"iters"` cells, which defaults to the length above for the bound. The bound is not
kept by weave's training step unless the layer says `"keep_contraction": 0.8` (a bound below
0.9 that the weights already meet; the chain then defaults to the length that bound needs).
Without it the update can take the weights past the bound, which leaves the block exactly
reversible but makes `r` large, and once `L` is above 1 the `K` cells of the chain multiply
what they hold by `L` each, so that a long chain (39 cells for 0.8) blows up the loss and, in
Q12, overflows a 64-bit number and wraps; the float network with the same chain diverges the
same way, so this is a property of the block and not of fixed point. With the key, every
`step` ends in `project_contraction` (an `irrev` function in `contract.roop`), which scales
`W1` and `W2` by the same factor whenever the bound of `F` is above it. The bound there is not
the power iteration but a stateless one from above: the spectral norm of each weight from
the squares of the trace-normalized Gram matrix `W^T W` (six squarings, whose trace is at
least the square of the largest eigenvalue), a percent high. The biases and the carry of the
optimizer are not touched. `weave_train.train(..., keep_contraction=0.8)` sets the key and the
chain length; a rate of 0.05 on the xor residual net of the tests diverges without it and
converges with it.

The projection runs after the optimizer in both `step` and `step_batch` (the same code emits
it), so it works with SGD, momentum and Adam. A rescale alone is a poor projection: the
bound of `F` is the product of the spectral norms of `W1` and `W2`, so its outward normal is
the rank one matrix `u v^T` of each weight, not the weights themselves, and a step followed by
a uniform rescale only stops where the gradient is along the weights, which is not where the
loss is smallest on the bound. And an optimizer that steps about the rate however small the
gradient (Adam) never stops answering a gradient that keeps pushing outward, while the rescale
keeps undoing it. So before the optimizer sees them, `project_gradient` (in
`contract_gradient.roop`) takes the part along that outward normal out of the gradients of `W1`
and `W2` whenever the bound is within about 5 percent of the cap and the gradient points
outward. The normal comes from the same squares of the Gram matrix as the bound: the squares
tend to the projector `H` onto the top right singular vector of `W`, and `W H / sigma` is
`u v^T`. What is left of the gradient vanishes at a minimum on the bound, the optimizer settles,
and the rescale after it only has the second order growth of a step along the bound to undo.
The biases and the carry of the optimizer are not touched, and neither are the moments, which
now see the gradient without its outward part. A model without `keep_contraction` emits
nothing new.

On the xor residual net of the tests (three blocks, bound 0.8), before and after, the loss at
epochs 1000, 2000 and 3000 and the worst in the last quarter of 3000 epochs:

| optimizer | before | after |
| --- | --- | --- |
| SGD, rate 0.05 | 0.032, 0.050, 0.052 (plateau) | 0.0000 from epoch 400 |
| momentum 0.9, rate 0.01 | 0.050, 0.051, 0.051 (plateau) | 0.0000 from epoch 200 |
| Adam, rate 0.01 | 0.016, 0.076, 0.085 (worst 0.17) | 0.0000 from epoch 200, to 12000 |

The 3-block torch round trip (`test_real_torch.py`, 3000 epochs) ends at 0.0000 with the bound
under 0.81 for SGD 0.05, momentum 0.01 and Adam at 0.003, 0.01 and 0.05; before, Adam ended
between 0.08 and 0.10 at every one of those rates and SGD and momentum at 0.053. A test trains
the xor net with Adam and requires the loss to stay under a hundredth of its first value for
the last quarter of the run. Removing the part of the step along the normal as well, after the
optimizer, was tried and changes nothing on these nets. A rate that is too high still
diverges (0.3 for SGD and for momentum with beta 0.9 on this net); the projection bounds the
weights of the blocks, not their biases.

```rust
use weave::step;

// One step of gradient descent on a batch of B samples through L layers of
// state width N and hidden width M, with K outputs, and kind 1 for the force.
call step<N, M, L, K, B>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr, kind);
```

`weave::grad` is the reversible core: the loss and the gradients of one sample,
with the input rebuilt at the end. `weave::forward` is the network alone, and
`forward_inv` is the network run backward. `weave::step` adds the one irreversible
part, clearing the buffers between samples, in an `irrev` function.

A test checks the gradients against central finite differences on a double
precision copy of the network (they agree to about 1%, which is the 4096 grid)
and that the input comes back bit for bit. Another trains XOR to a 50x lower
loss. Lean proves the forward pass, the backward pass and `grad` exactly reversible,
layer by layer and through the loop over layers, at widths 16 and 32 with 4 or 8
layers and at the benchmark's size, width 64 with hidden width 64, eight layers and
four outputs: the forward pass alone takes under a minute and all three functions
together about a minute and a half. Each call's own lemma settles a function of
many calls, so the proof does not grow with the number of layers.
`crates/roop/tests/weave_scale.rs` runs those three sizes.

The time used to grow sharply with the width, and not because of the loop lemmas
of the contractions. The proofs were small and quick to elaborate, but Lean's kernel
spent minutes checking them: a proof that unfolds a function and matches on the
result of a call makes the kernel reduce that call, and so the whole callee, to
see whether it is a constructor. With symbolic data it ran every iteration of the
callee's loops until it got stuck (the nested loops of a contraction, so a cost
that grows with the cube of the width). The prelude now keeps the loop runner
`Roop.janus` behind an opaque constant, with a theorem that it equals the real
definition, so the kernel stops at the first loop. Nothing else about the proofs
changed.

**Batches.** `weave::step` runs one sample at a time. `layer_batch`, `forward_batch`,
`layer_back_batch`, `backward_batch`, `grad_batch` and `step_batch` run `B`
samples as matrix products, on a state of `B` rows (`[[i64; N]; B]`), and
`mlp_batch`, `mlp_back_batch`, `attn_batch` and `attn_back_batch` do the same for
the perceptron and attention blocks. `mlp_norm_batch`, `mlp_layer_batch` and
`conv_batch` (with their `_back_batch` forms) do the same for the normalized
perceptrons, whose contractions are one matrix product over the rows and only the
normalization, with statistics of its own for each row, goes row by row, and for the
convolution, which stacks the unfolded signals of the rows side by side so that the
filters meet them in one product (`conv_batch<C, K, T, N, CK, B, BT>` takes
`BT = B * T`). They add into places they do not read and
rebuild every intermediate by `uncall` like the single-sample functions, so a
batch needs no more memory per layer than a sample does, and none that grows with
the depth. The gradients they add are the sums of what `B` calls of the
single-sample function add, to the bit (a test checks that for the leapfrog
network, the perceptron, attention, the normalized perceptrons and the
convolution, for every activation).
`step_parallel<N, M, L, K, B, C>` splits a step into `C` chunks of `B` samples,
each with its own buffers and its own gradients, runs them with `#[parallel(cpu)]`,
adds the gradients up and takes the step, which gives the weights of one batch of
`C * B` samples to the bit. A parallel loop may pass read-only places, the
weights, to a call, since the checker now counts a call as writing only the
arguments it gives to `&mut` parameters.

This is a research library, not a framework. There is no automatic
differentiation: each layer's adjoint is written out, and roop has no function
arguments to build a graph from. A test measures throughput: `cargo test -p roop
--test weave --release -- --ignored --nocapture`.

### Losses, optimizers and training from torch

The loss is chosen in the model: `"loss": "mse"` (half the squared error, the default),
`"sigmoid"` or `"softmax"`. For the two cross entropies the seed of the backward pass is
the usual `p - t`, with the sigmoid from the tanh above and an exponential computed as
`(1 + y + y^2 / 2 + y^3 / 6) ^ 64` for `y = x / 64` in Q24 (in Q24, within 1% down to -16, and 0
below -16); the cross entropy needs a
logarithm, which weave lacks, so `total` is the squared error of the probabilities there.
The softmax targets must sum to 1.

`"loss_scale": S` (a whole number from 1 to 4096, default 1) multiplies the seed of the backward
pass, so every adjoint and gradient is `S` times as large, and each optimizer divides it out in
its step (Adam's epsilon is scaled with it). Q12 adjoints lose what is below 1/4096 at every
layer on the way back, and the scale keeps it: with weights near 0.01 the gradients of a deep
network are zero without it. The softmax and sigmoid seeds are computed at the resolution of the
scale, and `total` is not scaled. In Python, `train(..., loss_scale=256)`.

`"optimizer"` is `{"kind": "sgd"}`, `{"kind": "momentum", "beta": 0.9}` or
`{"kind": "adam", "beta1": 0.9, "beta2": 0.999}`. The optimizers are `irrev`, since they
overwrite their moving averages, and live in the training step; the gradient stays
reversible. Adam corrects its moving averages for starting at zero, as torch does, with
two numbers of state per tensor, and keeps its second moment in Q24 so that small
squared gradients are not lost. Each optimizer computes its step in Q24 and keeps what is below
a unit of the Q12 weight in a carry (a tensor of state, `c`), added to the next step; without it
a step below a unit is dropped, and at a small learning rate most are. A matrix's gradient is in Q24, so the matrix
versions divide it by 4096 before the moving averages, and the vector ones take a unit of 1.

`crates/roop-weave/python/weave_train.py` closes the loop with torch:

```python
from weave_train import train
losses = train(net, xs, ts, epochs=1500, rate=0.05, loss="mse", optimizer={"kind": "adam"})
```

It exports the model, compiles it with `roop weave --batch B --driver main.c`, builds that
with the C program that trains on files (`prog weights data epochs rate samples trained`),
trains in fixed point with no activations stored, and copies the trained weights back into
the torch tensors they came from. `roop weave --driver` writes the program on its own, for
use outside Python. `roop weave --main` adds a `main` that holds the weights and runs the
model on the command line, the input as whole numbers on the 1/4096 grid, and prints the
outputs the same way: `roop run net.roop 1024 -2048 3072 0`.

### From torch

`roop weave` compiles a model of these layers to roop code. A model is JSON: the
state width, the number of outputs the loss reads, the step size and the layers,
each with its activation and its weights. A layer may have its own hidden width
and activation, which is why the compiler writes straight-line code for the
model, instead of calling the stack of `weave::forward`:

```json
{ "name": "net", "width": 4, "outputs": 1, "step": 0.25, "layers": [
  { "kind": "mlp", "activation": "tanh", "w1": [[...]], "b1": [...], "w2": [[...]], "b2": [...] },
  { "kind": "leapfrog", "activation": "relu", "weight": [[...]], "bias": [...] },
  { "kind": "attention", "seq": 2, "wq": [[...]], "wk": [[...]], "wv": [[...]] } ] }
```

```
roop weave model.json -o net.roop --tests --batch 8
roop test net.roop
```

The output has `net_forward`, `net_backward`, `net_grad` (the loss of a sample and its gradients, with the input
rebuilt at the end) and `net_step<B>`, which is the training step. With `--batch B`
it also has `net_forward_batch<B>`, `net_backward_batch<B>`, `net_grad_batch<B>` and
`net_step_batch<B>`, which run the samples through the network together as
matrix products, and `net_train`, a step of B samples with the length filled in,
which C can call; its buffers `q`, `p`, `aq` and `ap` hold B rows of the state.
The gradients of the matrices are in Q24. With `--tests` it adds `net_load` (the weights, rounded to 1/4096) and a test that runs
the forward pass and the gradient and compares them with a reference in doubles, whose gradients are
central differences, so it checks the compiler and weave together. With `--batch B`
it adds a second test that runs B different samples through the batched
functions and compares the outputs, the summed loss and the summed gradients
with the sum of the reference over the samples. Like every roop
test it also runs backward. Lean proves a compiled model's forward, backward and gradient exactly reversible
(`roop lean net.roop --check`), and the loader too when there is one (`--tests` or `--main`). The loader is
written as one small helper per few weights of a row, called on the row, so that Lean proves each helper
and the chain of calls.
A model that is not well formed is refused with the
name of the field, or the tensor, that is wrong.

`crates/roop-weave/python/torch_to_weave.py` writes that JSON from a torch module, an
`nn.Sequential` or a custom module that `torch.fx` can trace into a chain, with
`torch.relu`, `torch.tanh` and `F.softsign` read as activations:

```
python3 torch_to_weave.py pkg.module:factory --outputs 1 -o model.json
```

Most torch layers lose information, so it reads the model as blocks that do not.
`Linear(N, M), act, Linear(M, N)` is a perceptron block, `Linear(N, M), act`
alone is a leapfrog layer whose weight is tied to its transpose, and
`LinearAttention(seq, dim)`, from `weave_modules.py`, is an attention block, and an
`nn.Conv1d(C, C, K, padding=K // 2)` is a convolution block. `Linear, RMSNorm, act, Linear` is a
perceptron block with normalization (`nn.RMSNorm` needs `eps` of at least 1/4096), and
`Linear, LayerNorm, act, Linear` is one with a layer norm (its weight and bias are the gain and
the bias, and `eps` of at least 1/4096 is needed as well). A residual connection `x + F(x)`, with
`F` one `Linear, act, Linear`, is a residual block, found by tracing (an `nn.Sequential` that
holds a module of its own is traced too); it is refused unless its contraction bound is below 0.9,
and anything else that adds two paths is refused. The activations
are `Identity`, `ReLU`, `Tanh`, `Softsign`, `Sigmoid`, `SiLU` and `GELU`; any other layer is refused, naming
it, since turning it into something else would not be the model. The compiled
network is the network of those blocks run on `(q, p)` with the input in `q` and
`p` zero, so it is a reversible network trained like the torch one, and not the
same module run unchanged. A graph that branches, other than an `x + F(x)`, is refused. The exporter is
tested on real torch with uv (`uv run --extra torch python -m unittest`, in that
directory), where a mirror of the compiled network in torch finds the same
gradients as the reference that the generated test uses.

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

What keeps the proofs fast at size, for anyone changing the generator. The loop
function `Roop.janus` is `@[irreducible]`: otherwise `subst_vars` and `simp`
evaluate a loop with literal bounds step by step to see whether it is a
variable, and that cost grows with the bound, so width 32 did not finish. A
function that calls others is proved from their lemmas, never by unfolding them,
and so is one that only updates array elements (it gets lemmas of its own).
An array access at a literal index, `w[1]`, is written with the index as a
natural number (`Roop.agetN`, `Roop.asetN`), so that simplification settles its
bound at once instead of proving it again from a 64-bit literal, which made the
proof terms a hundred times larger. A chain of calls on elements of nested
arrays (`call f(w[0]); call g(w[1]);`) is proved call by call, and `roop_vec`
closes the equation between arrays that differ by `set`s that write back what
they replaced. Writes into the rows of a nested array in one function without
calls stay slow: a run of more than three or four in a function takes minutes,
so split it into functions of a few writes each, as the weight loader does.

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
sme = true
```

On an Apple M4 or later (the chips with the SME matrix unit), a `#[parallel]`
loop that is exactly a matrix product, `c[i][j] += alpha * a[i][l] * b[l][j]`
over loops of `i`, `l` and `j`, or a `y[i] += alpha * x[i]` loop of 2048 or more
`f64`s, runs on a kernel in the runtime instead of as compiled loops; this is
what `std::blas::level3::dgemm` and `std::blas::level1::daxpy` are. Backward
the kernel subtracts, so reversal works as for the loops. The results can
differ from the loops in the last bits, since the kernel fuses the
multiply and the add. `sme = false` under `[parallel]` turns it off.

The same recognition covers integer matrix products, the loops that `imatmul`,
`imatmul_nt` and `imatmul_tn` expand to (`c[i][k] += a[i][j] * b[j][k]`, or with
`a[j][i]` or `b[k][j]`), over `i64` arrays. With SME they call `roop_i64_matmul`,
which converts to doubles, uses the `dgemm` kernel and converts back; doubles hold
integers exactly, so it is used only when every product and every sum is below
2^52, and otherwise the loops run. The result is what the loops give, to the bit,
and backward the kernel subtracts it. The loops of the `q12` einsums, where each
product is divided by 4096 (`c[i][k] += a[i][j] * b[j][k] / 4096`), call a NEON
kernel instead, `roop_q12_matmul`, which does the same in doubles, truncating each
product, with the same fallback; `q12 = false` under `[parallel]` turns that one
off.

```toml
[optimize]
clear_ancillas = true
```

An ancilla that a `call f(w, ..)` made and an `uncall f(w, ..)` takes off again is
set to zero, not computed backward, when `w` is an array that starts at zero and
is written by nothing else, `f` writes nothing but `w` and does nothing to the
world, and nothing between the two writes what `f` reads. Running the function
backward does the same to the other of the two. Nothing observable changes, since
the `uncall` would have left zero, and a test runs weave's step with and without
it and compares the weights. `clear_ancillas = false` turns it off.

Metal kernels are emitted as AIR and built with Apple's `metal` tools; NVPTX
kernels go through LLVM's PTX backend and need a CUDA driver at run time. The
CUDA launcher has been verified on an NVIDIA GeForce RTX 4090 with driver 595.91.07
by `cargo test -p roop-rt --test cuda -- --nocapture`, which verifies PTX launch
and writable-buffer copyback. The compiler path is covered by
`cargo test -p roop-llvm --test parallel_nvptx`. CUDA performance benchmarks are
not included yet.

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
`editors/README.md` for installing it. `roop lsp` is a language server with
diagnostics, formatting and an outline; the same file says what it covers.
