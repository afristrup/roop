#!/usr/bin/env python3
"""How much memory weave's reversible training step needs as the network gets deeper.

    python3 bench/weave_memory.py

Two things come out. A table measured on this machine: the peak resident
memory of a roop training step at several depths, next to the bytes its weights
and gradients take, so what is left over is what the step itself needs. And a
model of the memory a stored-activation backward pass and a checkpointed one need
for the same network, written to docs/weave-memory.svg.
"""
import math, os, pathlib, re, subprocess, tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
ROOP = ROOT / "target" / "release" / "roop"
RT = ROOT / "target" / "release" / "libroop_rt.a"
WEAVE = ROOT / "roop" / "weave"
N, B, K = 64, 8, 4
DEPTHS = [8, 32, 128, 512, 1024]
WORD = 8

PROGRAM = """use weave::step;

irrev fn train(total: &mut i64, ws: &mut [[[i64; {N}]; {N}]; {L}], bs: &mut [[i64; {N}]; {L}],
               gw: &mut [[[i64; {N}]; {N}]; {L}], gb: &mut [[i64; {N}]; {L}],
               q: &mut [i64; {N}], p: &mut [i64; {N}], aq: &mut [i64; {N}], ap: &mut [i64; {N}],
               xs: &[[i64; {N}]; {B}], ts: &[[i64; {K}]; {B}], h: &i64, lr: &i64) {{
    call step<{N}, {L}, {K}, {B}>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr);
}}
"""

DRIVER = """#include <stdint.h>
#include <stdio.h>
#define N {N}
#define L {L}
#define B {B}
static int64_t ws[L][N][N], bs[L][N], gw[L][N][N], gb[L][N];
static int64_t xs[B][N], ts[B][{K}], q[N], p[N], aq[N], ap[N];
void train(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
           int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
int main(void) {{
    for (int l = 0; l < L; l++)
        for (int j = 0; j < N; j++) {{
            bs[l][j] = (j % 5 - 2) * 8;
            for (int i = 0; i < N; i++) ws[l][j][i] = ((l * 5 + j * 7 + i * 3) % 9 - 4) * 8;
        }}
    for (int n = 0; n < B; n++) {{
        for (int i = 0; i < N; i++) xs[n][i] = ((n * 3 + i) % 7 - 3) * 512;
        for (int k = 0; k < {K}; k++) ts[n][k] = ((n + k) % 3 - 1) * 512;
    }}
    int64_t H = 1024, LR = 64, total = 0;
    train(&total, (int64_t*)ws, (int64_t*)bs, (int64_t*)gw, (int64_t*)gb, q, p, aq, ap,
          (int64_t*)xs, (int64_t*)ts, &H, &LR);
    return 0;
}}
"""


def peak_rss(work):
    out = subprocess.run(["/usr/bin/time", "-l", "./prog"], cwd=work, capture_output=True, text=True, check=True)
    return int(re.search(r"(\d+)\s+maximum resident set size", out.stderr).group(1))


def measure(depth):
    work = pathlib.Path(tempfile.mkdtemp(prefix="roop-weave-mem-"))
    (work / "Roop.toml").write_text(f'[modules]\nweave = "{WEAVE}"\n')
    (work / "prog.roop").write_text(PROGRAM.format(N=N, L=depth, B=B, K=K))
    (work / "main.c").write_text(DRIVER.format(N=N, L=depth, B=B, K=K))
    subprocess.run([str(ROOP), "build", "prog.roop", "--link", "main.c", "-o", "prog"], cwd=work, check=True,
                   capture_output=True, env=dict(os.environ, ROOP_RT_LIB=str(RT)))
    return peak_rss(work)


def model(depth):
    """Words of working memory for one sample in flight, besides weights and gradients."""
    roop = 4 * N                      # q, p, aq, ap: the buffers the step is given
    stored = 4 * N + depth * 5 * N    # every layer keeps q, p and the z, sigma, dsigma the adjoint reads
    checkpointed = 4 * N + 2 * math.ceil(math.sqrt(depth)) * 5 * N
    return roop, stored, checkpointed


def svg(path):
    w, h, left, bottom = 560, 340, 70, 50
    depths = [2 ** k for k in range(3, 11)]
    series = {"roop (reversible)": 0, "checkpointed, every sqrt(L) layers": 2, "stored activations": 1}
    colors = {"roop (reversible)": "#0a7d5a", "checkpointed, every sqrt(L) layers": "#b7791f", "stored activations": "#b83232"}
    values = {name: [model(d)[i] * WORD for d in depths] for name, i in series.items()}
    lo, hi = math.log10(min(min(v) for v in values.values())), math.log10(max(max(v) for v in values.values()))
    x = lambda d: left + (math.log2(d) - 3) / 7 * (w - left - 20)
    y = lambda v: h - bottom - (math.log10(v) - lo) / (hi - lo) * (h - bottom - 30)
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" font-family="sans-serif" font-size="12">',
           '<rect width="100%" height="100%" fill="white"/>',
           f'<text x="{w/2}" y="18" text-anchor="middle" font-size="14">Working memory per sample in flight, width {N}</text>']
    for e in range(math.ceil(lo), math.floor(hi) + 1):
        out.append(f'<line x1="{left}" x2="{w-20}" y1="{y(10**e):.1f}" y2="{y(10**e):.1f}" stroke="#ddd"/>')
        label = f"{10**e/1024:.0f} KiB" if 10**e < 1024**2 else f"{10**e/1024**2:.0f} MiB"
        out.append(f'<text x="{left-6}" y="{y(10**e)+4:.1f}" text-anchor="end">{label}</text>')
    for d in depths:
        out.append(f'<text x="{x(d):.1f}" y="{h-bottom+16}" text-anchor="middle">{d}</text>')
    out.append(f'<text x="{(left+w-20)/2}" y="{h-bottom+34}" text-anchor="middle">layers L</text>')
    for k, (name, vals) in enumerate(values.items()):
        pts = " ".join(f"{x(d):.1f},{y(v):.1f}" for d, v in zip(depths, vals))
        out.append(f'<polyline points="{pts}" fill="none" stroke="{colors[name]}" stroke-width="2.5"/>')
        out.append(f'<text x="{left+10}" y="{36+16*k}" fill="{colors[name]}">{name}</text>')
    out.append("</svg>")
    path.write_text("\n".join(out))


def main():
    print(f"Width {N}, batch {B}, f64-sized words ({WORD} bytes). Peak resident memory of one roop training step.\n")
    print("| layers L | weights + gradients | peak resident | the rest | the rest per layer |")
    print("|---:|---:|---:|---:|---:|")
    base = None
    for depth in DEPTHS:
        params = 2 * depth * (N * N + N) * WORD
        rss = measure(depth)
        rest = rss - params
        base = base if base is not None else rest
        print(f"| {depth} | {params/1e6:.1f} MB | {rss/1e6:.1f} MB | {rest/1e6:.2f} MB | {((rest-base)/(depth-DEPTHS[0])/1e3 if depth != DEPTHS[0] else 0):.2f} KB |", flush=True)
    print("\nModel: words of working memory for one sample in flight, besides weights and gradients.\n")
    print("| layers L | roop | stored activations | checkpointed |")
    print("|---:|---:|---:|---:|")
    for depth in DEPTHS:
        r, s, c = (m * WORD / 1024 for m in model(depth))
        print(f"| {depth} | {r:.1f} KiB | {s:.1f} KiB | {c:.1f} KiB |")
    svg(ROOT / "docs" / "weave-memory.svg")


if __name__ == "__main__":
    main()
