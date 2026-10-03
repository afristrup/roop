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
| 128 | 349.5 | 349.5 | 4.8 | 16.1 | 435.8 |
| 256 | 404.3 | 404.3 | 3.4 | 16.6 | 460.2 |
| 512 | 500.8 | 469.3 | 2.6 | 11.7 | 320.1 |
| 1024 | 443.6 | 418.8 | 2.4 | 6.1 | 273.1 |

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

- **dgemm: roop matches or beats Accelerate from 512 up, and is 10 to 20 per
  cent behind at 128 and 256.** The compiled loops alone reach 60 GFLOP/s, an
  eighth of the kernel. At 512 the kernel is close to what the matrix unit can
  do (about 560 GFLOP/s if it issues one 8 by 8 `fmopa` a cycle at 4.4 GHz,
  which is an estimate and not a measurement). The small sizes lose to
  Accelerate because a call has a fixed cost (entering streaming mode, and
  transposing each block of `A` through the ZA tiles before its first use) that
  is a larger share of a small product. Removing the pack's branches and
  double buffering it, tried in a C harness, did not show a gain over the
  noise.
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

`dgemm` transposes a 16-row block of `A`, scaled by alpha, through the ZA tiles
into a packed panel and sweeps it against 32 columns of `B` with eight `fmopa`
tiles, 256 deep at a time. Calls above about 64 million multiply-adds split into
64-row chunks on the thread pool, which adds 10 to 25 per cent. `daxpy` of 512K elements or more is chunked
over the pool the same way.

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

Not measured: CUDA, since the launcher has not run on NVIDIA hardware.
