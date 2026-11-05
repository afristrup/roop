#!/usr/bin/env python3
"""Times compiler-emitted i64 additions on CPU threads and CUDA.

The CUDA time includes module load, device allocation, transfers, launch, and
copyback. Build first with `cargo build --release -p roop -p roop-rt`, then run
`python3 bench/calibrate_cuda.py` on a CUDA host.
"""
import os
import pathlib
import subprocess
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
ROOP = ROOT / "target" / "release" / "roop"
RT = ROOT / "target" / "release" / "libroop_rt.a"
SIZES = [(4, 101), (4096, 101), (65536, 31), (1048576, 5)]


def run(command, **kwargs):
    return subprocess.run(command, check=True, capture_output=True, text=True, **kwargs).stdout


def measure(n, reps, work):
    source = f"""fn add_cpu(q: &[i64; {n}], p: &mut [i64; {n}], i: &mut i64) {{
    #[parallel(cpu)] from i == 0 {{ p[i] += q[i]; }} loop {{ i += 1; }} until i == {n - 1};
}}
fn add_cuda(q: &[i64; {n}], p: &mut [i64; {n}], i: &mut i64) {{
    #[parallel(cuda)] from i == 0 {{ p[i] += q[i]; }} loop {{ i += 1; }} until i == {n - 1};
}}
"""
    (work / "Roop.toml").write_text("[parallel]\ntargets = [\"cpu\", \"cuda\"]\n")
    (work / "prog.roop").write_text(source)
    driver = (HERE / "roop" / "cuda_dispatch_main.c").read_text()
    (work / "main.c").write_text(driver.replace("@N@", str(n)).replace("@REPS@", str(reps)))
    environment = dict(os.environ, ROOP_RT_LIB=str(RT))
    run([str(ROOP), "build", "prog.roop", "--link", "main.c", "-o", "prog"], cwd=work, env=environment)
    trials = [tuple(map(float, run(["./prog"], cwd=work).split())) for _ in range(3)]
    return tuple(min(values) for values in zip(*trials))


def main():
    if not ROOP.exists() or not RT.exists():
        raise SystemExit("build release roop and roop-rt first")
    work = pathlib.Path(tempfile.mkdtemp(prefix="roop-cuda-dispatch-"))
    print("| N | CPU threads (us) | CUDA end to end (us) | faster |")
    print("|---:|---:|---:|:---|")
    for n, reps in SIZES:
        cpu, cuda = measure(n, reps, work)
        cpu_us, cuda_us = cpu * 1e6 / reps, cuda * 1e6 / reps
        faster = "CPU" if cpu_us < cuda_us else "CUDA"
        print(f"| {n} | {cpu_us:.2f} | {cuda_us:.2f} | {faster} |", flush=True)


if __name__ == "__main__":
    main()
