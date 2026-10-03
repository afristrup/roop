#!/usr/bin/env python3
"""Times roop's BLAS routines against hand-written Rust and Apple Accelerate.

    python3 bench/run.py            # prints a markdown table

roop is built twice: with the compiler free to pick a GPU for a bare
`#[parallel]` loop (the default), and with `auto = false`, so loops run on CPU
threads. Every figure is the median of several runs after one warm-up.
"""
import os, pathlib, platform, subprocess, sys, tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
ROOP = ROOT / "target" / "release" / "roop"
if not ROOP.exists():
    ROOP = ROOT / "target" / "debug" / "roop"
STD = ROOT / "roop" / "std"

GEMM = [(128, 9), (256, 7), (512, 5), (1024, 3)]
AXPY = [(1 << 16, 51), (1 << 20, 31), (1 << 22, 21), (1 << 24, 11)]


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, capture_output=True, text=True, **kw).stdout


def roop_time(kind, n, reps, auto, work):
    toml = f'[modules]\nstd = "{STD}"\n\n[parallel]\nauto = {"true" if auto else "false"}\n'
    (work / "Roop.toml").write_text(toml)
    src = (HERE / "roop" / f"{kind}.roop").read_text().replace("@N@", str(n))
    (work / "prog.roop").write_text(src)
    main = (HERE / "roop" / f"{kind}_main.c").read_text().replace("@N@", str(n)).replace("@REPS@", str(reps))
    (work / "main.c").write_text(main)
    env = dict(os.environ, ROOP_RT_LIB=str(ROOT / "target" / ("release" if "release" in str(ROOP) else "debug") / "libroop_rt.a"))
    run([str(ROOP), "build", "prog.roop", "--link", "main.c", "-o", "prog"], cwd=work, env=env)
    return float(run(["./prog"], cwd=work).strip())


def native_times(kind, n, reps):
    return [float(x) for x in run(["/tmp/roop_native_bench", kind, str(n), str(reps)]).split()]


def main():
    run(["rustc", "-O", "-C", "target-cpu=native", str(HERE / "native" / "bench.rs"), "-o", "/tmp/roop_native_bench"])
    work = pathlib.Path(tempfile.mkdtemp(prefix="roop-bench-"))
    chip = run(["sysctl", "-n", "machdep.cpu.brand_string"]).strip() if platform.system() == "Darwin" else platform.processor()
    print(f"Machine: {chip}, {os.cpu_count()} cores. roop: {ROOP.parent.name} build of the compiler.\n")

    print("### dgemm, C += A B, f64, GFLOP/s (higher is better)\n")
    print("| N | roop (auto) | roop (CPU threads) | Rust, 1 thread | Rust, all threads | Accelerate |")
    print("|---:|---:|---:|---:|---:|---:|")
    for n, reps in GEMM:
        flops = 2.0 * n**3
        auto = flops / roop_time("gemm", n, reps, True, work) / 1e9
        cpu = flops / roop_time("gemm", n, reps, False, work) / 1e9
        one, many, blas = (flops / t / 1e9 for t in native_times("gemm", n, reps))
        print(f"| {n} | {auto:.1f} | {cpu:.1f} | {one:.1f} | {many:.1f} | {blas:.1f} |", flush=True)

    print("\n### daxpy, y += a x, f64, GB/s moved (higher is better)\n")
    print("| N | roop (auto) | roop (CPU threads) | Rust, 1 thread | Rust, all threads | Accelerate |")
    print("|---:|---:|---:|---:|---:|---:|")
    for n, reps in AXPY:
        bytes_moved = 24.0 * n
        auto = bytes_moved / roop_time("axpy", n, reps, True, work) / 1e9
        cpu = bytes_moved / roop_time("axpy", n, reps, False, work) / 1e9
        one, many, blas = (bytes_moved / t / 1e9 for t in native_times("axpy", n, reps))
        print(f"| {n} | {auto:.1f} | {cpu:.1f} | {one:.1f} | {many:.1f} | {blas:.1f} |", flush=True)


if __name__ == "__main__":
    sys.exit(main())
