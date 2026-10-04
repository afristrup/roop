#!/usr/bin/env python3
"""Where a loop stops being faster serial and starts being faster on threads.

    python3 bench/calibrate_dispatch.py

Times the same f64 `y += a x` loop with and without `#[parallel(cpu)]` over a
range of sizes. The compiler decides between serial code and threads for a bare
`#[parallel]` loop from `CostModel` (crates/roop-llvm/src/module/cost_model.rs);
the crossover here is what its `serial_cutoff_ns` and `cpu_launch_ns` should
agree with. SME is off in Roop.toml: with it on, a `#[parallel(cpu)]` loop of
2048 or more elements is the matrix kernel, not threads.
"""
import os, pathlib, subprocess, tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
ROOP = ROOT / "target" / "release" / "roop"
RT = ROOT / "target" / "release" / "libroop_rt.a"
STD = ROOT / "roop" / "std"
SIZES = [256, 1024, 4096, 16384, 32768, 65536, 131072, 262144, 1048576]


def best(n, attribute, work):
    program = f"""fn axpy(y: &mut [f64; {n}], x: &[f64; {n}], alpha: &f64, i: &mut i64) {{
    {attribute} from i == 0 {{ y[i] += alpha * x[i]; }} loop {{ i += 1; }} until i == {n - 1};
}}
"""
    reps = 401 if n <= 65536 else 101
    (work / "Roop.toml").write_text(f'[modules]\nstd = "{STD}"\n\n[parallel]\nsme = false\n')
    (work / "prog.roop").write_text(program)
    (work / "main.c").write_text((HERE / "roop" / "dispatch_main.c").read_text().replace("@N@", str(n)).replace("@REPS@", str(reps)))
    subprocess.run([str(ROOP), "build", "prog.roop", "--link", "main.c", "-o", "prog"], cwd=work, check=True,
                   capture_output=True, env=dict(os.environ, ROOP_RT_LIB=str(RT)))
    runs = [float(subprocess.run(["./prog"], cwd=work, capture_output=True, text=True).stdout) for _ in range(3)]
    return min(runs)


def main():
    work = pathlib.Path(tempfile.mkdtemp(prefix="roop-dispatch-"))
    print("| N | serial (us) | threads (us) | faster |")
    print("|---:|---:|---:|:---|")
    for n in SIZES:
        serial, threads = best(n, "", work), best(n, "#[parallel(cpu)]", work)
        print(f"| {n} | {serial * 1e6:.2f} | {threads * 1e6:.2f} | {'serial' if serial < threads else 'threads'} |", flush=True)


if __name__ == "__main__":
    main()
