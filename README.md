# roop

The reversible, parallel programming language. More at [uncompute.ai](https://www.uncompute.ai).

Every roop function has an inverse, and the compiler makes it: `f` and `f_inv`
from one definition. That is a strange constraint for a language, and it buys
things that are hard to get any other way.

## Why you would want it

- **Undo is real, not a snapshot.** A `try` that fails runs the work it did
  backward. Nothing is copied before the risky part and nothing is left half done,
  including what the work wrote to the screen or to files.
- **Debugging by running backward.** A function can be called with `uncall` to go
  from the state it ended in to the one it began in. The test runner does it for
  you: every `test` runs forward and then backward, and fails if anything did not
  come back to zero, so a leak is a test failure.
- **Training without storing activations.** [`weave`](docs/weave-memory.md) is a
  network of reversible layers whose backward pass recomputes by running the layers
  backward, so its memory does not grow with depth.
- **Parallel loops that are checked, not hoped for.** `#[parallel]` loops run on
  CPU threads, Metal or CUDA, and the checker proves the iterations touch disjoint
  places. Lean proves the iterations can run in any order.
- **A path toward hardware that does not erase.** Reversible logic is what
  adiabatic and quantum circuits need. roop does not target them yet; it is a
  language where the constraint is already met.

## Thirty seconds

```rust
use std::io::*;

fn transfer(src: &mut i64, dst: &mut i64, amount: &i64) {
    src -= amount;
    dst += amount;
    expect src >= 0;
}

fn main() {
    auto ancilla alice: i64 = 0;
    auto ancilla bob: i64 = 0;
    auto ancilla failed: bool = false;
    alice += 100;
    try {
        call transfer(alice, bob, 150);
        call println("sent");
    } catch_rollback {
        call println("not enough money: rolled back");
    } -> failed;
    call print_int(alice);
    call print(" ");
    call println_int(bob);
}
```

```
$ roop run roop/examples/bin/bank.roop
not enough money: rolled back
100 0
```

`transfer` fails its `expect` half way. The `try` takes back the subtraction by
running `transfer` backward, so Alice still has 100. No snapshot was taken.
`roop test` would also run `transfer` backward after any test of it, and check that
every fixture is zero again.

## Getting started

```
cargo build --release -p roop -p roop-rt
roop run roop/examples/bin/hello.roop
roop test roop/                # every test, forward then backward
roop fmt roop/                 # like cargo fmt, 88 columns
roop lean prog.roop --check    # prove the reversibility theorems in Lean
```

`roop/std` is the standard library (`io`, `fs`, `text`, `math`, `blas`, ...), and
`roop/examples/bin` has `hello`, `echo`, `cat`, `wc`, `rot13`, `fib`, `copy`, `ls`,
`undo` and `bank`, all reversible. Highlighting for Zed and Tree-sitter editors is
in `editors/`.

## What is proved, and what is not

- Lean checks, for every reversible function, that it never fails on an
  unrestored ancilla and that `f_inv` undoes `f`. It also proves loops terminate
  and `#[parallel]` loops commute. `weave`'s forward and backward passes are
  proved this way, for a width of 4 and three layers; 8 also passes.
- `try` gets only `f_inv_f`, not the reverse. Sessions are checked as
  specifications, not matched against the task code. Floating point carries no
  round-trip claim, and GPU floating point can differ between runtimes.
- **The CUDA launcher has not run on NVIDIA hardware.** Metal and CPU threads have.
- `keep` moves information into a history that lives until the process ends, so
  the energy cost of erasing is deferred, not removed. `history_limit` in
  `Roop.toml` bounds it and `std::process::forget` empties it; see
  [the reference](docs/reference.md#the-world).
- Speed: [docs/benchmarks.md](docs/benchmarks.md). Generated CPU code matches a
  hand-written threaded loop, the library's BLAS is a plain triple loop and far
  behind Accelerate, and the GPU auto-dispatch is calibrated on one machine.

## Documentation

[The reference](docs/reference.md) covers the whole language, one section each:
branches and loops with exit assertions, history stacks, `try` and rollback,
modules, generics over lengths, ancillas (`auto` and lifetimes), bytes and text,
programs and reversible I/O, einsum, the standard library, sessions, Lean, tests,
Bennett, tasks and channels, parallel loops, building, formatting and editors.

## License

See [LICENSE](LICENSE).
