# Benchmarks

What roop's generated code costs next to hand-written code and Apple's
Accelerate, on the two kernels the standard library's BLAS covers best:
`dgemm` (compute-bound) and `daxpy` (memory-bound). Run them with

```
cargo build --release -p roop -p roop-rt
python3 bench/run.py            # or: python3 bench/run.py gemm | axpy
```

`bench/run.py` builds each program with the roop compiler, links a small C
driver, and times it; the Rust baselines are in `bench/native/bench.rs` and use
iterators, so they vectorize. Each figure is the best of three runs of the
program, each the best of several hundred timed calls after a tenth of a second
of warm-up calls, with small sizes timing a batch of calls. The best, not the
median, because on a laptop the scheduler moves a program between fast and slow
cores and a median swings by a factor of two. The warm-up is there because the
SME matrix unit starts slow: the same `dgemm` call has a median of 76 GFLOP/s
over its first 400 calls and 320 over 4000. Everything is from one machine, an Apple M4 with 10 cores that other
work was also using, so read the figures as the shape and not as exact ratios:
Accelerate's `dgemm` figures at 512 and 1024 moved between runs by up to 40 per
cent, roop's by 10.

roop is measured twice. **auto** is the default: a bare `#[parallel]` loop runs
serially, on CPU threads or on the GPU, whichever the cost model estimates is
fastest, and a `dgemm` or `daxpy` loop runs on the SME kernels (below).
**CPU threads** is `auto = false` in `Roop.toml`; the SME kernels are used there
too. Neither can use the GPU here, since Metal has no double precision.

## dgemm, `C += A B`, f64, GFLOP/s (higher is better)

| N | roop (auto) | roop (CPU threads) | Rust, 1 thread | Rust, all threads | Accelerate |
|---:|---:|---:|---:|---:|---:|
| 128 | 387.2 | 388.7 | 4.8 | 16.2 | 435.8 |
| 256 | 468.7 | 484.5 | 3.4 | 16.7 | 459.9 |
| 512 | 544.6 | 547.8 | 2.7 | 12.3 | 439.0 |
| 1024 | 528.0 | 525.1 | 2.5 | 8.2 | 442.9 |

The second pass over the kernel (packing, tile exchange, threading, below) took
roop from 349.5, 404.3, 488.1 and 476.2 GFLOP/s (auto, the same machine and
script, rerun before the change) to the figures above. The matrices of
`bench/roop/gemm_main.c` are now aligned to 128 bytes: they were not, and a
matrix that is not 64-byte aligned costs the kernel 17 per cent (321.6 against
387.2 at 128) and Accelerate 23 per cent in a C harness, while the Rust and
Accelerate columns were measured on page-aligned `Vec`s. The timer is now
`clock_gettime_nsec_np`; `CLOCK_MONOTONIC` has a resolution of one microsecond
on macOS, which put every 128 figure on a step of about 40 GFLOP/s.

Where roop stood before the work of this section, on the same machine
(GFLOP/s, auto): 15.7, 14.5, 9.7 and 4.7 at the four sizes. Reordering the
loops to `i, l, j` so the inner loop is contiguous gave 49.3, 64.5, 62.1 and
57.1 (Accelerate was 8 to 40 times faster). Calling the SME kernel gave the
table above.

## daxpy, `y += a x`, f64, GB/s moved (higher is better)

| N | roop (auto) | roop (CPU threads) | Rust, 1 thread | Rust, all threads | Accelerate |
|---:|---:|---:|---:|---:|---:|
| 65536 | 496.6 | 496.6 | 149.1 | 31.8 | 648.9 |
| 1048576 | 340.1 | 331.1 | 146.9 | 259.4 | 193.5 |
| 4194304 | 102.7 | 104.4 | 111.6 | 106.6 | 96.8 |
| 16777216 | 103.2 | 100.2 | 93.3 | 104.7 | 95.9 |

Before: 147.5, 149.8, 113.7 and 101.6 (auto), measured with a shorter warm-up.

## What to read from them

- **dgemm: roop is ahead of Accelerate from 256 up (about 1.02 to 1.05 at 256,
  1.2 at 512 and 1024) and still about 11 per cent behind at 128. Parity at 128
  was not reached.** The compiled loops alone reach 60 GFLOP/s, an eighth of the
  kernel. At 512 the kernel is close to what the matrix unit can do (about 560
  GFLOP/s if it issues one 8 by 8 `fmopa` a cycle at 4.4 GHz, which is an
  estimate and not a measurement). Measured by removing parts of the kernel in a
  C harness at 128 (one thread, 11.2 microseconds a call before this work, 9.6
  for Accelerate): the transposing pack of `A` was about 20 per cent of the time
  and loading and storing the C tiles through ZA about 10; a call's own cost
  (entering streaming mode) is 0.13 microsecond. What helped: no multiply when
  alpha is 1 (and `fmops` when it is -1), which the loops and the reversed call
  use; reading the transposed columns out of ZA four at a time and storing them
  four at a time; packing the next block between two tiles of the current one;
  storing C tiles four ahead of reloading them; and splitting calls above 8
  million multiply-adds into 16-row chunks on the thread pool in place of
  64-row chunks above 64 million, which lets a 256 product use more than one
  core's matrix unit (the gain is measured, the reason is a guess). What did not
  help: pipelining the pack's loads two half-blocks ahead (slower), threading 128 (a pool wake costs more
  than it saves, 262 to 276 GFLOP/s), larger chunks (worse at 512), and the
  multi-vector ZA group moves, which address four rows of one tile stride 2
  apart and not the four tiles of a row, so they cannot load a row of C. Single
  threaded the kernel runs 389, 444, 453 and 413 at the four sizes in the C
  harness, so at 128 the gap to Accelerate is in the pack and the C round trip
  that a 128-deep product cannot amortise.
- **daxpy: ahead of Accelerate from 1M elements up, about 25 per cent behind at
  64K.** At 4M and 16M everything is limited by memory at around 100 GB/s and
  roop is level with Accelerate. In cache the kernel moves 256 bytes a load
  and keeps `y` in the ZA array while it adds; in a C harness that ran at 500
  to 650 GB/s at 64K to 256K elements, level with Accelerate, but the same
  kernel reached through roop's benchmark gives 500 at 64K and the gap to
  649 is not explained.
- **Rust, with every thread, does not catch the matrix unit.** The kernel is 30
  to 70 times faster on `dgemm` than ten Rust threads, and a single Rust thread
  is 200 times behind.
- **Neither number is a claim about other routines.** Only these two loop
  shapes have kernels. The other level 1, 2 and 3 routines (`ddot`, `dgemv`,
  `dsyrk`, and so on) are still compiled loops, not benchmarked here. `ddot`
  and `dgemv` accumulate in program order, which LLVM may not vectorize without
  reordering the sums, so they are the likeliest to be behind Accelerate.

## The SME kernels

The M4 has an SME matrix unit that Accelerate uses and compiled loops do not:
`fmopa` multiplies two vectors of 8 doubles into an 8 by 8 tile of
accumulators (128 flops an instruction), and its multi-vector `fmla` into the
ZA array moves 256 bytes a load. LLVM does not target it from ordinary loops, so
the kernels in `crates/roop-rt/kernels` (C with the SME intrinsics, compiled by
`crates/roop-rt/build.rs` with `clang -mcpu=apple-m4`) are called by the
compiler when it sees the loop shape.

The compiler (`crates/roop-llvm/src/stmts/parallel/gemm` and `axpy`) recognizes
a `#[parallel]` loop that is exactly `c[i][j] += alpha * a[i][l] * b[l][j]`
over `i`, `l` and `j` in that order, or `y[i] += alpha * x[i]` over a vector of
at least 2048 doubles, with the places named in the types it expects, and
emits a call in place of the loops. Anything else, including a loop with any
other body, stays a loop. Run backward the call passes `-alpha`, so `uncall`
still undoes `call`. The Lean model sees the source loops, not the kernel, so
the proofs of `std::blas` are unchanged; the results differ from the loops in
the last bits because the kernel fuses the multiply and the add, and a
reversed `dgemm` restores its input to within rounding, as it did before.

`dgemm` transposes a 16-row block of `A` through the ZA tiles into a packed panel
(scaled by alpha unless alpha is 1 or -1, where `fmopa` or `fmops` does the sign)
and sweeps it against 32 columns of `B` with eight `fmopa` tiles, 256 deep at a
time. Calls above about 8 million multiply-adds split into 16-row chunks on the
thread pool, which is most of the gain at 256 and 512. `daxpy` of 512K
elements or more is chunked over the pool the same way. The kernel assumes
nothing about alignment but is a fifth slower on matrices that are not 64-byte
aligned, so the compiler aligns array locals to 64 bytes; arrays that come from
a caller keep the caller's alignment.

Without SME (an M1 to M3, or `sme = false` under `[parallel]` in `Roop.toml`)
the loops are compiled as before. A host whose `clang` cannot build the kernels
gets plain Rust loops with the same names, and a warning from `cargo build`.

Not measured: dgemm above 1024, where the single-call kernel is limited by
streaming `B` from memory (a C harness with 2048 by 2048 matrices ran at a third
of the 1024 rate on one thread and at nine tenths with ten threads, which the
runtime does not yet arrange for), and other
element types, since the matrix unit's `f32` and integer forms are not used.

## What `noalias` changed

The first version of this page had roop's serial loop running at a third of one
Rust thread at 64K elements. The cause is visible in the generated code: every
reference parameter was a plain `ptr`, so after the store to `y[i]` LLVM had to
assume the loop counter, which is also a reference, might have changed, kept it in
memory, and did not vectorize.

roop's rules say otherwise, and now say it completely. The checker refuses a call
whose arguments overlap where one is written (`call g(a, a)`, `call g(v[i], v[j])`),
which it did not before: such a call was accepted although `uncall` could not undo
it, because `g` changed what it read under its own update. With that check in
place every reference parameter is the only way in to what it points at, so the
compiler marks them `noalias`, and the same loop runs 2.7 times faster (10.5
instead of 29 microseconds at 64K elements). Tests in
`crates/roop-check/tests/call_aliasing.rs` cover the check.

## The serial/thread/GPU decision

A bare `#[parallel]` loop is placed by `CostModel`
(`crates/roop-llvm/src/module/cost_model.rs`). `bench/calibrate_dispatch.py`
times the same `daxpy` loop serially and with `#[parallel(cpu)]`:

| N | serial (us) | threads (us) | faster |
|---:|---:|---:|:---|
| 256 | 0.02 | 12.18 | serial |
| 1024 | 0.10 | 17.10 | serial |
| 4096 | 0.38 | 15.99 | serial |
| 16384 | 2.62 | 15.79 | serial |
| 32768 | 5.25 | 19.50 | serial |
| 65536 | 10.50 | 23.83 | serial |
| 131072 | 29.00 | 40.00 | serial |
| 262144 | 60.00 | 54.00 | threads |
| 1048576 | 173.00 | 151.00 | threads |

Starting threads costs 12 to 17 microseconds, one thread streams about 150 bytes
a nanosecond out of cache, and threads win from about 256K elements. The model
follows these figures, so a streaming loop of 32K elements runs serially and one
of 512K on threads (a test in `crates/roop-llvm/tests/parallel_auto.rs` holds that).
Above a few million elements the data comes from memory, everything moves 100 GB/s
or so, and threads and serial code end up within noise of each other.

An earlier version of this page said the GPU was chosen too eagerly. That was
wrong. The GPU is not a candidate for double precision on Metal, the generated
code at 1M elements and up is identical with `auto` on or off, and the difference
in the first table was run-to-run noise. The cost model's GPU side, calibrated for
64-bit integer loops, has not been re-checked.

CUDA runtime verification: `cargo test -p roop-rt --test cuda -- --nocapture` passed on an NVIDIA GeForce RTX 4090 with driver 595.91.07, verifying PTX launch and writable-buffer copyback. CUDA performance benchmarks are not included yet.

## weave's training step

One step of gradient descent on 32 samples through 8 leapfrog layers of width 64
(the cauchy force, 4 outputs), the network of `throughput_of_a_training_step` in
`crates/roop/tests/weave.rs`. Run `python3 bench/weave_speed.py --baseline
CHECKOUT` (a checkout of the commit to compare with, built in release); it builds
each variant of the roop program, times the Rust baselines of
`bench/native/weave_speed.rs`, and prints this table. Each figure is the best of
several rounds of ten steps with the first round dropped. The machine was an
Apple M4 that other work was using (load average 7 to 8 at the start, and the
figures moved by up to 10 per cent between two runs; the earliest measurements,
taken at a load of 15 to 80, moved by a factor of two, and none of them are used
here), so read the ratios and not the digits.

| | samples/s | ms a step |
|:---|---:|---:|
| before (`feat/torch-to-weave` with the dgemm and daxpy kernels): one sample at a time | 2200 to 2350 | 13.6 to 14.8 |
| one sample at a time, ancillas computed backward | 1700 to 1930 | 16.6 to 18.9 |
| one sample at a time | 6470 to 6590 | 4.9 |
| the batch as matrix products, as loops | 1450 to 1520 | 21 to 22 |
| the batch, ancillas zeroed, as loops | 6050 to 6080 | 5.3 |
| the batch, ancillas zeroed, matrix kernels | 25600 to 26100 | 1.23 to 1.25 |
| the batch in 2 chunks on threads | 32700 to 34500 | 0.93 to 0.98 |
| the batch in 4 chunks on threads | 18300 to 19700 | 1.6 to 1.75 |
| Rust, integers, one sample at a time | 6970 to 7150 | 4.5 |
| Rust, integers, the batch as matrices | 7190 to 7480 | 4.3 to 4.45 |
| Rust, integers, the batch in 10 chunks | 24200 to 24700 | 1.3 |
| Rust, doubles, one sample at a time | 7540 to 7580 | 4.2 |
| Rust, doubles, the batch in 10 chunks | 26100 to 27700 | 1.2 |
| Accelerate, doubles, the batch as `dgemm` | 47100 to 48000 | 0.67 |
| Accelerate, doubles, the batch in 10 chunks | 34000 to 34900 | 0.92 |

The first row is the old code, with the old arithmetic. Every roop row below it
runs the new code, and all of them end three steps with the same weights, to the
bit (checksum 6843866773079962822), which is also what the Rust integer version
ends with after three steps: roop's reversible step and hand-written Rust that
stores its activations compute the same numbers. Memory does not grow with the
depth in any roop row.

What each change is worth, in the order they were made:

- **Zeroing ancillas.** The old code, run one sample at a time, spent most of its
  time computing intermediates backward to restore them. When a `call f(w, ..)`
  made an ancilla and an `uncall f(w, ..)` takes it off, with nothing between
  that changes what `f` read and `f` writing nothing else, the compiler now sets
  the ancilla to zero (`[optimize] clear_ancillas`, on by default). Running
  backward it does the same to the other statement. The two rows differ by a
  factor of 3.4 to 3.8 with this alone, and it needs no particular kind of
  program.
- **Wide contractions.** The old contractions divided every product by 4096
  before adding. A sum cannot be given to the matrix unit that way, so weave's
  contractions (`q12w` in `einsum`) add the products exactly in 64 bits and divide
  the sum once, which is also more accurate. It is not the same arithmetic as
  before, and the gradient of a matrix is now in Q24, the sums of the products
  that `adjoint` adds up. This is what takes the old 2200 samples a second to the
  new 1700 to 1900 when everything else is off (the same loops, with a pass to
  scale and a scratch array added), and it is the price of the next row.
- **Batching.** Run as loops, the batch is as fast as one sample at a time, since
  the same multiply-adds are done. It pays when the products are matrices: the
  compiler recognizes the loops of `imatmul`, `imatmul_nt` and `imatmul_tn` and
  calls a kernel that converts the integers to doubles, which hold them exactly
  when every product and sum is below 2^52 (otherwise it runs the loops), runs
  PR 13's SME `dgemm`, and converts back. A 32 by 64 by 64 product takes 6 to 9
  microseconds that way and about 80 as loops, in a C harness; the step goes from
  6000 to 26000 samples a second. For `q12`, which divides every product,
  doubles cannot be summed, and a NEON kernel (`roop_q12_matmul`, about 3 to 4
  times faster than the loops, exact and falling back to loops outside the range
  the doubles hold) is used; weave no longer calls it, but `qmatmul` and the
  other `q12` einsums do.
- **Threads.** `weave::step_parallel` splits the batch into chunks, each with its
  own gradients, runs them with `#[parallel(cpu)]` and adds the gradients up,
  which gives, to the bit, the weights of one batch. Two chunks of 16 help (about
  1.3 times), four of 8 hurt, because the SME unit is shared by the cores of a
  cluster, the matrix products of a small chunk spend more of their time on
  conversions than on the unit, and each chunk repeats the weights' conversions.
  On a quiet machine four chunks gave 26000 to 38000 in earlier runs; with the
  other work on this machine the figure was not stable, and the table is the
  run that was.

Caveats. The Rust rows are what an idiomatic Rust program does, with iterators
and no intrinsics, in the same arithmetic as roop or in doubles; a tuned integer
Rust program could use the same trick of summing in doubles. Accelerate's
`dgemm` is not reversible and keeps its activations; it is 1.8 times faster than
roop's best single-thread step, which is the answer to whether a reversible step
can match it: not here (the section after this one measures the step again
with a faster integer kernel). The old row is `0baf7b8`, built from a `git archive` into
its own target directory. Other work on the machine, including Lean runs, was
active throughout, and the numbers that were taken when the load average was
above 15 are not reported.

## weave against Accelerate

The step of the section above, measured again after the integer kernel moved
entirely onto the matrix unit. The machine was the same Apple M4 (10 cores) with
other agents running Lean and benchmarks on it: the load average was between 4
and 8 for every figure below, and went up to 25 during some other runs, which
are not reported. Each figure is a range over four to six rounds in which the old
build, the new build and Accelerate ran one after the other, so within a round
the ratios are fair; across rounds a figure moves by 20 to 40 per cent, and the
same Accelerate program ran at 47 000 samples a second in the section above and
at 57 000 to 71 000 today. Read the ratios, not the digits.
`bench/weave_speed.py` prints one build (new options: `--width`, `--batch`,
`--only`, `--checkout` to measure another checkout, `--accelerate-only`); the
comparison below ran `origin/main` and this branch alternately.

### What changed

- **One streaming pass for an `i64` product** (`crates/roop-rt/kernels/
  sme_i64_matmul.c`, replacing `i64_matmul.c`). The kernel used to convert the
  three matrices to doubles in normal mode, call `roop_dgemm`, which entered
  streaming mode, and convert back. In a C harness a 32 by 64 by 64 call spent
  about three times as long converting as multiplying. Now it enters streaming
  mode once, converts the operands into panels (A is transposed through the ZA
  tiles on the way, B is converted, or transposed for `imatmul_nt`), accumulates
  in ZA, and converts each finished tile back and adds it to C as it leaves. A
  helper that is not inlined into the streaming function is a call with a lazy
  save of ZA around it, and made the first version of this several times slower;
  they are all `always_inline` with the same ZA state as the caller now.
- **The exactness check costs no pass of its own.** The kernel must know that
  every product and sum is below 2^52 before it touches C. It first scanned both
  operands for the largest entry (a sixth of the step in a profile of the first
  version); it now tracks the largest and smallest entry while it converts, before
  any tile runs, and goes to the plain loops with C untouched when the bound
  fails. Products with more than 512 rows are checked in a pass of their own first
  and then run in blocks, so the panels stay small.
- **The matrix kernels are declared `nocapture nounwind`** in the generated IR.
  This did not change the speed measurably, and is kept because it is true.
- A negative sign (`uncall`) subtracts the converted tile, so it restores C to the
  bit. `crates/roop-rt/tests/i64_matmul.rs` checks the kernel against the loops
  for awkward sizes, more than a block of rows, the threaded path, products that
  overflow the doubles, the edge of the exact range and `i64::MIN`, forward and
  backward; the weave tests that compare kernels on and off pass unchanged.

One call, in a C harness (microseconds, best of 37 batches of 100, load 3.7):

| m x n x k | old kernel NN / NT / TN | new kernel NN / NT / TN | roop dgemm (f64) | Accelerate dgemm |
|:---|---:|---:|---:|---:|
| 32 x 64 x 64 | 4.4 / 5.4 / 5.4 | 1.6 / 1.8 / 1.5 | 1.16 | 0.81 |
| 128 x 64 x 64 | 13.1 / 14.2 / 11.8 | 4.9 / 5.0 / 4.4 | 4.6 | 3.0 |
| 256 x 64 x 64 | 24.4 / 26.5 / 32.2 | 9.7 / 9.7 / 8.4 | 9.3 | 6.0 |
| 32 x 128 x 128 | 11.5 / 35.8 / 18.8 | 5.0 / 5.9 / 5.0 | 3.0 | 2.6 |
| 128 x 128 x 128 | 30.1 / 46.5 / 49.2 | 16.1 / 15.4 / 16.4 | 12.6 | 10.8 |
| 256 x 128 x 128 | 51.7 / 68.0 / 130.7 | 27.2 / 29.9 / 28.3 | 23.7 | 19.9 |

The kernel is 2 to 4 times faster than before, 1.05 to 1.7 times the cost of the
f64 `dgemm` it is built on, and 1.4 to 2 times Accelerate's. The harness runs
with the matrices hot in L1, which a step does not.

### The training step, samples a second

Eight layers, four outputs, the weights and data of `throughput_of_a_training_step`.
"Before" is `origin/main`, "after" is this branch, both the batch as matrix
products with the kernels on; "2 chunks" is `step_parallel` with two chunks on
threads; Accelerate is the hand-written Rust step of `bench/native/weave_speed.rs`
with every product a `cblas_dgemm`, on one thread, and in 10 chunks on threads.
Every roop row ends three steps with the same weights to the bit (checksum
6843866773079962822 at width 64, batch 32).

| width, batch | before | after | before, 2 chunks | after, 2 chunks | Accelerate, 1 thread | Accelerate, 10 chunks |
|:---|---:|---:|---:|---:|---:|---:|
| 64, 32 | 23 000 to 36 000 | 36 000 to 52 000 | 32 000 to 48 000 | 47 000 to 68 000 | 57 000 to 71 000 | 38 000 to 43 000 |
| 64, 128 | 28 000 to 40 000 | 43 000 to 60 000 | 45 000 to 65 000 | 74 000 to 88 000 | 62 000 to 77 000 | 78 000 to 117 000 |
| 64, 256 | 34 000 to 41 000 | 61 000 to 63 000 | 64 000 to 75 000 | 88 000 to 102 000 | 69 000 to 77 000 | 145 000 to 151 000 |
| 128, 32 | 9 200 to 9 700 | 13 800 to 17 500 | 8 300 to 10 700 | 14 600 to 20 000 | 18 000 to 26 000 | 13 600 to 18 500 |
| 128, 128 | 12 700 to 15 000 | 22 100 to 26 800 | 18 800 to 24 600 | 27 500 to 29 300 | 23 400 to 26 200 | 34 000 to 35 500 |
| 128, 256 | 14 800 to 18 300 | 23 900 to 30 300 | 28 200 to 31 400 | 31 000 to 42 200 | 28 900 to 33 800 | 44 600 to 54 200 |

The first row is two runs (six and four rounds, load 3.7 and 4.4), the others one
run of four rounds at load 8.1 (64, 128), 5.0, 4.7, 4.4 and 4.1. Batch 32 by
width 64 is the case of the section above, where the old step was 1.8 times
slower than Accelerate on one thread; its gap is smaller now and is not closed.

What the table says. The reversible step on one thread is 1.3 to 1.8 times faster
than before and runs at about 0.6 to 0.75 of Accelerate's speed at width 64 and
batch 32, 0.7 to 0.8 at batch 128, 0.8 to 0.9 at batch 256, and at width 128 about
0.7 at batch 32, 1.0 at batch 128 (within the noise) and 0.85 at batch 256. So
Accelerate's `dgemm` on one thread is still faster at the shape the question was
asked about, and the larger the batch the closer roop gets. With two chunks on
threads roop is above Accelerate on one thread at batches of 128 and 256 (and
about equal at batch 32, width 64), but Accelerate on ten threads is ahead at
the large batches, where we did not try more than four chunks. Four chunks, which
were slower than one before, are now about as fast as two (47 000 to 50 000 at
width 64, batch 32, in a separate run of four rounds, against 47 000 to 68 000
for two); that is probably because the conversions, which ran in normal mode and
in parallel, now run on the matrix unit with the products, and the unit is shared
by the cores of a cluster, but that was not isolated.

### Where the rest of the gap is

A profile of the batched step at width 64 and batch 32, with the first version of
the new kernel: 54 per cent in the kernel (a third of it the scan for the largest
entry, since removed), 16 per cent in the activations and their slopes inside
`force_back_batch`, 9 in `force_batch`, 5 in `adjoint_batch`, 11 in `memset` (the
ancillas are zeroed when they are made and again when they are unmade, and the
compiler cannot fold the first zero into the first `+=`), the rest small. Replacing
the matrix kernel with a call that does nothing leaves 0.40 ms a step, which is 80
000 samples a second, measured with the real kernel at 1.1 ms in the same minutes:
the elementwise code of roop alone costs about as much as the whole Accelerate
step (0.43 to 0.47 ms). So even a free matrix product would only make roop equal
to Accelerate here, and with the kernel as it is (a 32 by 64 by 64 call in 1.6
microseconds in the harness and about 2.7 in the step, where the operands were just
written in normal mode and have to reach the matrix unit through L2) the step
is at 0.75 to 0.8 ms.

A streaming loop here is limited by the bytes it moves through L2 and not by its
instruction count: four-vector loads and `min`/`max`, four ZA tiles loading in
parallel with four reading, a branch on the sign in place of a multiply, 128-byte
aligned scratch, `-O3` and non-trivial loop unswitching for the generated code,
and a larger or smaller `KC` in the f64 kernel were each tried and were within the
noise, so none is in the code.

What was not done:

- The f64 `dgemm` trails Accelerate at 128 (0.89 of its speed after the second
  pass, from 0.8) and is ahead from 256. The gap at 128 is the pack and the C
  tile round trip, which a product 128 deep does not amortise; a 16 by 24 tile
  that leaves two ZA tiles free to pack the next block while this one multiplies
  was not tried, and 128 is not a multiple of 24.
- Batched attention, convolution and the norm MLP are still loops over rows with
  no matrix kernel. The convolution and the attention products are `q12` einsums
  per sample, so they run on the NEON kernel; one product over all rows would be a
  rewrite of those library functions and of their Lean models, and was not
  attempted.
- The activations, the elementwise adds and `kick` are scalar code, since NEON has
  no 64-bit multiply or divide. The division of the cauchy force can be done in
  doubles, exactly while the numerator is below 2^53, but a vector double divide
  issues about as slowly as a scalar integer divide, so it would not be faster.
- Contractions that share an operand are not fused: the two products of
  `force_back_batch` that read W with the same layout each convert W again, which
  is about 0.2 microsecond of the 1.6.
