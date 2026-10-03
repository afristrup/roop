#!/usr/bin/env python3
"""weave's reversible backward pass against the usual one, on memory and on the answer.

    python3 bench/weave_compare.py

The same network, in the same integer arithmetic, is run three ways for a batch
of samples in flight: roop's `grad`, which stores no activation, and a Rust
version (bench/native/weave.rs) that stores every layer's input state, and one
that stores every sqrt(L)-th and recomputes the rest. The gradients' checksums
must agree exactly. The table has each process's peak resident memory and what
the activations alone took.
"""
import math, os, pathlib, re, subprocess, tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
ROOP = ROOT / "target" / "release" / "roop"
RT = ROOT / "target" / "release" / "libroop_rt.a"
WEAVE = ROOT / "roop" / "weave"
N, K = 64, 4
BATCH = 256
DEPTHS = [8, 32, 128, 512, 1024]

PROGRAM = """use weave::grad;

fn one(total: &mut i64, q: &mut [i64; {N}], p: &mut [i64; {N}], aq: &mut [i64; {N}], ap: &mut [i64; {N}],
       gw: &mut [[[i64; {N}]; {N}]; {L}], gb: &mut [[i64; {N}]; {L}],
       ws: &[[[i64; {N}]; {N}]; {L}], bs: &[[i64; {N}]; {L}], h: &i64, t: &[i64; {K}]) {{
    call grad<{N}, {L}, {K}>(total, q, p, aq, ap, gw, gb, ws, bs, h, t);
}}
"""

DRIVER = """#include <stdint.h>
#include <stdio.h>
#include <string.h>
#define N {N}
#define L {L}
#define B {B}
#define K {K}
static int64_t ws[L][N][N], bs[L][N], gw[L][N][N], gb[L][N];
static int64_t xs[B][N], ts[B][K], q[N], p[N], aq[N], ap[N];
void one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
         int64_t*, int64_t*, int64_t*, int64_t*);
static int64_t fold(const int64_t *v, long n) {{
    int64_t a = 0;
    for (long i = 0; i < n; i++) a = (int64_t)((uint64_t)a * 31u + (uint64_t)v[i]);
    return a;
}}
int main(void) {{
    for (int l = 0; l < L; l++)
        for (int j = 0; j < N; j++) {{
            bs[l][j] = (j % 5 - 2) * 8;
            for (int i = 0; i < N; i++) ws[l][j][i] = ((l * 5 + j * 7 + i * 3) % 9 - 4) * 8;
        }}
    for (int s = 0; s < B; s++) {{
        for (int i = 0; i < N; i++) xs[s][i] = ((s * 3 + i) % 7 - 3) * 512;
        for (int k = 0; k < K; k++) ts[s][k] = ((s + k) % 3 - 1) * 512;
    }}
    int64_t h = 1024, total = 0;
    for (int s = 0; s < B; s++) {{
        memcpy(q, xs[s], sizeof q);
        one(&total, q, p, aq, ap, (int64_t*)gw, (int64_t*)gb, (int64_t*)ws, (int64_t*)bs, &h, ts[s]);
        memset(q, 0, sizeof q); memset(p, 0, sizeof p); memset(aq, 0, sizeof aq); memset(ap, 0, sizeof ap);
    }}
    printf("total %lld gw %lld gb %lld\\n", (long long)total, (long long)fold((int64_t*)gw, (long)L * N * N), (long long)fold((int64_t*)gb, (long)L * N));
    return 0;
}}
"""


def timed(cmd, cwd=None):
    out = subprocess.run(["/usr/bin/time", "-l", *cmd], cwd=cwd, capture_output=True, text=True, check=True)
    rss = int(re.search(r"(\d+)\s+maximum resident set size", out.stderr).group(1))
    return out.stdout.strip(), rss, out.stderr


def roop(depth):
    work = pathlib.Path(tempfile.mkdtemp(prefix="roop-weave-cmp-"))
    (work / "Roop.toml").write_text(f'[modules]\nweave = "{WEAVE}"\n')
    (work / "prog.roop").write_text(PROGRAM.format(N=N, L=depth, K=K))
    (work / "main.c").write_text(DRIVER.format(N=N, L=depth, B=BATCH, K=K))
    subprocess.run([str(ROOP), "build", "prog.roop", "--link", "main.c", "-o", "prog"], cwd=work, check=True,
                   capture_output=True, env=dict(os.environ, ROOP_RT_LIB=str(RT)))
    return timed(["./prog"], cwd=work)


def svg(path, rows):
    """Peak resident memory against depth, one line for each way of running the backward pass."""
    w, h, left, bottom = 600, 360, 70, 50
    names = [("weights + gradients", "params", "#777777"), ("stored activations", "stored", "#b83232"),
             ("checkpointed, every sqrt(L) layers", "checkpoint", "#b7791f"), ("roop (reversible)", "roop", "#0a7d5a")]
    depths = [r["depth"] for r in rows]
    lo = math.log10(min(r["params"] for r in rows) / 1e6)
    hi = math.log10(max(r["stored"] for r in rows) / 1e6)
    x = lambda d: left + (math.log2(d) - math.log2(depths[0])) / (math.log2(depths[-1]) - math.log2(depths[0])) * (w - left - 20)
    y = lambda mb: h - bottom - (math.log10(mb) - lo) / (hi - lo) * (h - bottom - 50)
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" font-family="sans-serif" font-size="12">',
           '<rect width="100%" height="100%" fill="white"/>',
           f'<text x="{w/2}" y="18" text-anchor="middle" font-size="14">Peak memory of a backward pass, width {N}, {BATCH} samples in flight</text>']
    for e in range(math.ceil(lo), math.floor(hi) + 1):
        out.append(f'<line x1="{left}" x2="{w-20}" y1="{y(10**e):.1f}" y2="{y(10**e):.1f}" stroke="#ddd"/>')
        out.append(f'<text x="{left-6}" y="{y(10**e)+4:.1f}" text-anchor="end">{10**e:g} MB</text>')
    for d in depths:
        out.append(f'<text x="{x(d):.1f}" y="{h-bottom+16}" text-anchor="middle">{d}</text>')
    out.append(f'<text x="{(left+w-20)/2}" y="{h-bottom+34}" text-anchor="middle">layers L</text>')
    for k, (label, key, color) in enumerate(names):
        pts = " ".join(f"{x(r['depth']):.1f},{y(r[key] / 1e6):.1f}" for r in rows)
        dash = ' stroke-dasharray="5 4"' if key == "params" else ""
        out.append(f'<polyline points="{pts}" fill="none" stroke="{color}" stroke-width="2.5"{dash}/>')
        out.append(f'<text x="{left+10}" y="{36+16*k}" fill="{color}">{label}</text>')
    out.append("</svg>")
    path.write_text("\n".join(out))


def main():
    native = pathlib.Path("/tmp/roop_native_weave")
    subprocess.run(["rustc", "-O", "-C", "target-cpu=native", str(HERE / "native" / "weave.rs"), "-o", str(native)], check=True)
    print(f"Width {N}, {BATCH} samples in flight, 8-byte words.\n")
    print("| layers L | weights + gradients | roop: peak | stored: peak | stored: activations | checkpointed: peak | checkpointed: activations | same gradients |")
    print("|---:|---:|---:|---:|---:|---:|---:|:---|")
    rows = []
    for depth in DEPTHS:
        params = 2 * depth * (N * N + N) * 8
        answers, peaks, activations = {}, {}, {}
        answers["roop"], peaks["roop"], _ = roop(depth)
        for mode in ("stored", "checkpoint"):
            out, rss, err = timed([str(native), mode, str(N), str(depth), str(BATCH)])
            answers[mode], peaks[mode] = out, rss
            kept, recomputed = (int(x) for x in re.findall(r"(\d+) bytes", err)[:2])
            activations[mode] = kept + recomputed
        same = "yes" if len(set(answers.values())) == 1 else "NO: " + " / ".join(answers.values())
        rows.append({"depth": depth, "params": params, "roop": peaks["roop"], "stored": peaks["stored"], "checkpoint": peaks["checkpoint"]})
        mb = lambda b: f"{b / 1e6:.1f} MB"
        print(f"| {depth} | {mb(params)} | {mb(peaks['roop'])} | {mb(peaks['stored'])} | {mb(activations['stored'])} | {mb(peaks['checkpoint'])} | {mb(activations['checkpoint'])} | {same} |", flush=True)
    svg(ROOT / "docs" / "weave-memory.svg", rows)


if __name__ == "__main__":
    main()
