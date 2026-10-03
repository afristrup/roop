#!/usr/bin/env python3
"""Times weave's training step, in roop and in hand-written Rust, on the same network.

    python3 bench/weave_speed.py [--baseline CHECKOUT]

The network is 8 leapfrog layers of width 64 with the cauchy force, trained on 32
samples a step with 4 outputs; the weights and the data are the ones of the
`throughput_of_a_training_step` test. roop is built several ways to say what
each change is worth: one sample at a time or the batch as matrix products, with
and without zeroing the ancillas that a call made (`clear_ancillas`), with and
without the matrix kernels, and with the batch in chunks that run on threads.
`bench/native/weave_speed.rs` is the same network in Rust, in integers and in
doubles, and with Accelerate. `--baseline` is a checkout of an older commit,
built with `cargo build --release -p roop -p roop-rt`, whose one-sample-at-a-time
step is timed too. Every figure is the best of several rounds of ten steps, the
first round dropped, so it is the machine at its quietest; run it when nothing
else is.
"""
import argparse, os, pathlib, subprocess, sys, tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent

PER_SAMPLE = """use weave::step;

irrev fn train(total: &mut i64, ws: &mut [[[i64; 64]; 64]; 8], bs: &mut [[i64; 64]; 8],
               gw: &mut [[[i64; 64]; 64]; 8], gb: &mut [[i64; 64]; 8],
               q: &mut [i64; 64], p: &mut [i64; 64], aq: &mut [i64; 64], ap: &mut [i64; 64],
               xs: &[[i64; 64]; 32], ts: &[[i64; 4]; 32], h: &i64, lr: &i64, kind: &i64) {
    call step<64, 64, 8, 4, 32>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr, kind);
}
"""

BATCHED = """use weave::step_batch;

irrev fn train(total: &mut i64, ws: &mut [[[i64; 64]; 64]; 8], bs: &mut [[i64; 64]; 8],
               gw: &mut [[[i64; 64]; 64]; 8], gb: &mut [[i64; 64]; 8],
               q: &mut [[i64; 64]; 32], p: &mut [[i64; 64]; 32], aq: &mut [[i64; 64]; 32], ap: &mut [[i64; 64]; 32],
               xs: &[[i64; 64]; 32], ts: &[[i64; 4]; 32], h: &i64, lr: &i64, kind: &i64) {
    call step_batch<64, 64, 8, 4, 32>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr, kind);
}
"""

CHUNKS = """use weave::step_parallel;

irrev fn train(total: &mut [i64; @C@], ws: &mut [[[i64; 64]; 64]; 8], bs: &mut [[i64; 64]; 8],
               gw: &mut [[[[i64; 64]; 64]; 8]; @C@], gb: &mut [[[i64; 64]; 8]; @C@],
               tw: &mut [[[i64; 64]; 64]; 8], tb: &mut [[i64; 64]; 8],
               q: &mut [[[i64; 64]; @B@]; @C@], p: &mut [[[i64; 64]; @B@]; @C@],
               aq: &mut [[[i64; 64]; @B@]; @C@], ap: &mut [[[i64; 64]; @B@]; @C@],
               xs: &[[[i64; 64]; @B@]; @C@], ts: &[[[i64; 4]; @B@]; @C@], h: &i64, lr: &i64, kind: &i64) {
    call step_parallel<64, 64, 8, 4, @B@, @C@>(total, ws, bs, gw, gb, tw, tb, q, p, aq, ap, xs, ts, h, lr, kind);
}
"""

# The arguments of `train` for each way of calling it.
CALLS = {
    "sample": "&total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, &H, &LR, &kind",
    "batch": "&total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, &H, &LR, &kind",
    "chunks": "totals, ws, bs, gwc, gbc, tw, tb, q, p, aq, ap, xs, ts, &H, &LR, &kind",
}

DRIVER = """#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#define N 64
#define L 8
#define S 32
#define C @C@
static int64_t ws_[L][N][N], bs_[L][N], gw_[L][N][N], gb_[L][N], tw_[L][N][N], tb_[L][N];
static int64_t gwc_[C][L][N][N], gbc_[C][L][N];
static int64_t xs_[S][N], ts_[S][4], q_[S][N], p_[S][N], aq_[S][N], ap_[S][N], totals_[C];
#define ws (int64_t*)ws_
#define bs (int64_t*)bs_
#define gw (int64_t*)gw_
#define gb (int64_t*)gb_
#define tw (int64_t*)tw_
#define tb (int64_t*)tb_
#define gwc (int64_t*)gwc_
#define gbc (int64_t*)gbc_
#define xs (int64_t*)xs_
#define ts (int64_t*)ts_
#define q (int64_t*)q_
#define p (int64_t*)p_
#define aq (int64_t*)aq_
#define ap (int64_t*)ap_
#define totals (int64_t*)totals_
void train(@PROTO@);
static void reset(void) {
    memset(gw_, 0, sizeof gw_); memset(gb_, 0, sizeof gb_);
    for (int l = 0; l < L; l++)
        for (int j = 0; j < N; j++) {
            bs_[l][j] = (j % 5 - 2) * 64;
            for (int i = 0; i < N; i++) ws_[l][j][i] = ((l * 5 + j * 7 + i * 3) % 9 - 4) * 64;
        }
}
static double now(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec + t.tv_nsec / 1e9; }
int main(void) {
    for (int n = 0; n < S; n++) {
        for (int i = 0; i < N; i++) xs_[n][i] = ((n * 3 + i) % 7 - 3) * 512;
        for (int k = 0; k < 4; k++) ts_[n][k] = ((n + k) % 3 - 1) * 512;
    }
    int64_t H = 1024, LR = 64, kind = 1, total = 0;
    double best = 1e9;
    for (int round = 0; round < @ROUNDS@; round++) {
        reset();
        double t = now();
        for (int s = 0; s < 10; s++) train(@CALL@);
        t = (now() - t) / 10;
        if (round > 0 && t < best) best = t;
    }
    reset();
    for (int s = 0; s < 3; s++) train(@CALL@);
    int64_t sum = 0;
    for (int i = 0; i < L * N * N; i++) sum = (int64_t)((uint64_t)sum * 31u + (uint64_t)((int64_t*)ws_)[i]);
    for (int i = 0; i < L * N; i++) sum = (int64_t)((uint64_t)sum * 31u + (uint64_t)((int64_t*)bs_)[i]);
    printf("%.1f samples/s, %.2f ms a step, weights after 3 steps %lld\\n", S / best, 1000 * best, (long long)sum);
    return 0;
}
"""


def fill(text, **kw):
    for key, value in kw.items():
        text = text.replace("@%s@" % key, str(value))
    return text


def run(roop, rt, weave, einsum, source, call, proto, toml, chunks=1, rounds=30):
    work = pathlib.Path(tempfile.mkdtemp(prefix="roop-weave-speed-"))
    (work / "Roop.toml").write_text(f'[modules]\nweave = "{weave}"\neinsum = "{einsum}"\n{toml}')
    kw = dict(C=chunks, B=32 // chunks, ROUNDS=rounds, CALL=CALLS[call], PROTO=proto)
    (work / "prog.roop").write_text(fill(source, **kw))
    (work / "main.c").write_text(fill(DRIVER, **kw))
    env = dict(os.environ, ROOP_RT_LIB=str(rt))
    subprocess.run([str(roop), "build", "prog.roop", "--link", "main.c", "-o", "prog"], cwd=work, env=env, check=True,
                   capture_output=True)
    return subprocess.run(["./prog"], cwd=work, capture_output=True, text=True, check=True).stdout.strip()


def proto(arity):
    return ", ".join(["int64_t*"] * arity)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", help="a checkout of an older commit, built in release")
    args = parser.parse_args()
    roop, rt = ROOT / "target/release/roop", ROOT / "target/release/libroop_rt.a"
    weave, einsum = ROOT / "roop/weave", ROOT / "roop/einsum"
    chip = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"], capture_output=True, text=True).stdout.strip()
    load = os.getloadavg()[0]
    print(f"{chip}, {os.cpu_count()} cores, load average {load:.1f} when it started\n")

    off = "\n[parallel]\nsme = false\nq12 = false\n\n[optimize]\nclear_ancillas = false\n"
    cleared = "\n[parallel]\nsme = false\nq12 = false\n\n[optimize]\nclear_ancillas = true\n"
    rows = []
    if args.baseline:
        base = pathlib.Path(args.baseline).resolve()
        rows.append(("before: one sample at a time (the commit given)", run(
            base / "target/release/roop", base / "target/release/libroop_rt.a", base / "roop/weave",
            base / "roop/einsum", PER_SAMPLE, "sample", proto(14), "")))
    rows += [
        ("roop, one sample at a time, ancillas computed backward", run(roop, rt, weave, einsum, PER_SAMPLE, "sample", proto(14), off)),
        ("roop, one sample at a time", run(roop, rt, weave, einsum, PER_SAMPLE, "sample", proto(14), "")),
        ("roop, the batch as matrices, as loops", run(roop, rt, weave, einsum, BATCHED, "batch", proto(14), off)),
        ("roop, the batch, ancillas zeroed", run(roop, rt, weave, einsum, BATCHED, "batch", proto(14), cleared)),
        ("roop, the batch, matrix kernels", run(roop, rt, weave, einsum, BATCHED, "batch", proto(14), "")),
    ]
    for c in (2, 4):
        rows.append((f"roop, the batch in {c} chunks on threads", run(roop, rt, weave, einsum, CHUNKS, "chunks", proto(16), "", chunks=c)))
    for label, line in rows:
        print(f"| {label} | {line} |")

    native = pathlib.Path(tempfile.mkdtemp(prefix="roop-native-")) / "weave_speed"
    subprocess.run(["rustc", "-O", "-C", "target-cpu=native", str(HERE / "native/weave_speed.rs"), "-o", str(native)], check=True)
    print()
    print(subprocess.run([str(native), "5"], capture_output=True, text=True, check=True).stdout)


if __name__ == "__main__":
    sys.exit(main())
