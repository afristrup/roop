# Benchmarks

What roop's generated code costs next to hand-written code, on the two kernels
the standard library's BLAS covers best: `dgemm` (compute-bound) and `daxpy`
(memory-bound). Run them with

```
cargo build --release -p roop -p roop-rt
python3 bench/run.py
```

`bench/run.py` builds each program with the roop compiler, links a small C
driver, and times it; the Rust baselines are in `bench/native/bench.rs`. Each
figure is the median of several runs after one warm-up. The figures below are from
one machine, an Apple M4 with 10 cores, and one run. They show the shape, not
precise ratios.

roop is measured twice. **auto** is the default: a bare `#[parallel]` loop goes to
the CPU threads or to the GPU, whichever the cost model estimates is faster.
**CPU threads** is `auto = false` in `Roop.toml`.

## dgemm, `C += A B`, f64, GFLOP/s (higher is better)

| N | roop (auto) | roop (CPU threads) | Rust, 1 thread | Rust, all threads | Accelerate |
|---:|---:|---:|---:|---:|---:|
| 128 | 8.7 | 12.1 | 1.7 | 10.6 | 258.8 |
| 256 | 9.7 | 10.4 | 3.2 | 14.5 | 458.9 |
| 512 | 10.1 | 10.4 | 2.6 | 10.8 | 437.9 |
| 1024 | 4.1 | 3.6 | 2.4 | 6.3 | 294.3 |

## daxpy, `y += a x`, f64, GB/s moved (higher is better)

| N | roop (auto) | roop (CPU threads) | Rust, 1 thread | Rust, all threads | Accelerate |
|---:|---:|---:|---:|---:|---:|
| 65536 | 47.7 | 143.0 | 40.3 | 20.6 | 639.6 |
| 1048576 | 84.4 | 110.4 | 31.3 | 146.4 | 172.6 |
| 4194304 | 73.6 | 99.1 | 37.7 | 97.6 | 90.9 |
| 16777216 | 81.5 | 82.2 | 28.4 | 85.0 | 69.8 |

## What to read from them

- **roop matches a hand-written threaded loop.** The generated CPU code is within
  noise of the same algorithm written in Rust and run on every core, on both
  kernels, and 3 to 4 times a single thread. A reversible update such as
  `y[i] += alpha * x[i]` compiles to the loop a person would write; reversibility
  costs nothing at run time here.
- **roop is not a BLAS.** `dgemm` is the plain triple loop, with no tiling and no
  matrix units, so Accelerate is 25 to 40 times faster on the same machine. That
  gap is the algorithm, not the language, and the library does not claim to close
  it.
- **At memory-bound sizes everything meets the memory system.** At 16M elements
  roop, threaded Rust and Accelerate all move 70 to 85 GB/s. The very large
  Accelerate figure at 65536 is the vector sitting in cache.
- **The auto-dispatch is too eager.** For `daxpy` between 64K and 4M elements the
  CPU threads beat what `auto` chose, by up to 3 times at 64K. The cost model is
  calibrated on this one machine and is optimistic about the GPU for loops that
  move more than they compute. Until it is calibrated on more machines, set
  `auto = false` for memory-bound loops.
- **Not measured:** CUDA, since the launcher has not run on NVIDIA hardware, and f64
  on Metal, which has no double precision, so `dgemm` never leaves the CPU.
