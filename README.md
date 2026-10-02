# roop
The reversible parallel programming language. See more at [uncompute.ai](uncompute.ai)

## Parallel loops

A counted `from` loop marked `#[parallel]` runs its iterations at once. The
checker proves every write is private to its iteration (indexed by the loop
variable) and that nothing reads another iteration's write, so the loop is safe
to run, and to reverse, in any order.

```rust
rev fn axpy(a: &mut [i64; 1000], b: &[i64; 1000], i: &mut i64, k: &i64) {
    #[parallel]
    from i == 0 { a[i] += b[i] * k; } loop { i += 1; } until i == 999;
}
```

A bare `#[parallel]` lets the compiler pick CPU threads or a GPU from a cost
model measured on Apple M4. `#[parallel(cpu)]`, `#[parallel(metal)]` and
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
