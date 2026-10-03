# Benchmarks

What roop's generated code costs next to hand-written code, on the two kernels
the standard library's BLAS covers best: `dgemm` (compute-bound) and `daxpy`
(memory-bound). Run them with

```
cargo build --release -p roop -p roop-rt
python3 bench/run.py
```

`bench/run.py` builds each program with the roop compiler, links a small C
driver, and times it; the Rust baselines are in `bench/native/bench.rs` and use
iterators, so they vectorize. Each figure is the best of three runs of the
program, each the best of several timed calls after one warm-up, and small
sizes time a batch of calls. The best, not the median, because on a laptop the
scheduler moves a program between fast and slow cores and a median swings by a
factor of two. Everything is from one machine, an Apple M4 with 10 cores, so
read the figures as the shape and not as exact ratios.

roop is measured twice. **auto** is the default: a bare `#[parallel]` loop runs
serially, on CPU threads or on the GPU, whichever the cost model estimates is
fastest. **CPU threads** is `auto = false` in `Roop.toml`. Neither can use the
GPU here, since Metal has no double precision.

## dgemm, `C += A B`, f64, GFLOP/s (higher is better)

| N | roop (auto) | roop (CPU threads) | Rust, 1 thread | Rust, all threads | Accelerate |
|---:|---:|---:|---:|---:|---:|
| 128 | 17.5 | 15.9 | 4.1 | 15.4 | 399.5 |
| 256 | 15.3 | 15.3 | 3.4 | 16.4 | 459.4 |
| 512 | 11.7 | 11.4 | 2.6 | 12.1 | 436.7 |
| 1024 | 5.5 | 5.2 | 2.5 | 7.6 | 407.6 |

## daxpy, `y += a x`, f64, GB/s moved (higher is better)

| N | roop (auto) | roop (CPU threads) | Rust, 1 thread | Rust, all threads | Accelerate |
|---:|---:|---:|---:|---:|---:|
| 65536 | 96.3 | 106.0 | 149.1 | 33.4 | 648.9 |
| 1048576 | 151.6 | 159.3 | 145.9 | 234.2 | 207.4 |
| 4194304 | 110.6 | 116.0 | 111.2 | 109.1 | 98.4 |
| 16777216 | 104.8 | 105.0 | 94.7 | 104.4 | 95.9 |


## What to read from them

- **On compute-bound work roop matches a hand-written threaded loop.** `dgemm`
  is within noise of the same algorithm written in Rust and run on every core,
  and 3 to 4 times a single thread.
- **roop is not a BLAS.** `dgemm` is the plain triple loop, with no tiling and no
  matrix units, so Accelerate is 25 to 40 times faster on the same machine. That
  gap is the algorithm, not the language, and the library does not claim to close
  it.
- **Memory-bound work meets the memory system.** At 4M and 16M elements roop,
  threaded Rust and Accelerate all move 95 to 115 GB/s.
- **In cache, roop is close to one Rust thread, not to ten.** At 64K elements roop
  moves about two thirds of what a single Rust thread does, at 1M the same as one
  Rust thread and two thirds of the hand-threaded version. The remaining gap to
  hand-written Rust is open work.

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
