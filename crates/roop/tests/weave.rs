mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{project, roop, stderr};

const PROGRAM: &str = "
use weave::forward;
use weave::grad;
use weave::step;

fn fwd(q: &mut [i64; 4], p: &mut [i64; 4], ws: &[[[i64; 4]; 4]; 2], bs: &[[i64; 4]; 2], h: &i64, kind: &i64) {
    call forward<4, 4, 2>(q, p, ws, bs, h, kind);
}

fn g(total: &mut i64, q: &mut [i64; 4], p: &mut [i64; 4], aq: &mut [i64; 4], ap: &mut [i64; 4],
     gw: &mut [[[i64; 4]; 4]; 2], gb: &mut [[i64; 4]; 2],
     ws: &[[[i64; 4]; 4]; 2], bs: &[[i64; 4]; 2], h: &i64, kind: &i64, t: &[i64; 1]) {
    call grad<4, 4, 2, 1>(total, q, p, aq, ap, gw, gb, ws, bs, h, kind, t);
}

irrev fn train(total: &mut i64, ws: &mut [[[i64; 4]; 4]; 2], bs: &mut [[i64; 4]; 2],
               gw: &mut [[[i64; 4]; 4]; 2], gb: &mut [[i64; 4]; 2],
               q: &mut [i64; 4], p: &mut [i64; 4], aq: &mut [i64; 4], ap: &mut [i64; 4],
               xs: &[[i64; 4]; 4], ts: &[[i64; 1]; 4], h: &i64, lr: &i64, kind: &i64) {
    call step<4, 4, 2, 1, 4>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr, kind);
}
";

const DRIVER: &str = r#"
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define N 4
#define L 2
#define S 4096.0
typedef int64_t W3[L][N][N];
typedef int64_t B2[L][N];

void fwd(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void fwd_inv(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void g(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
       int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void train(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
           int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);

static double sigma(double z) { return z / (1 + z * z); }

/* The same network in doubles: the reference the fixed-point code is held to. */
static double ref_loss(double w[L][N][N], double b[L][N], const double *x, double t, double h) {
    double q[N], p[N] = {0, 0, 0, 0};
    memcpy(q, x, sizeof q);
    double c = -h / 2;
    for (int l = 0; l < L; l++) {
        for (int pass = 0; pass < 3; pass++) {
            if (pass == 1) { for (int i = 0; i < N; i++) q[i] += h * p[i]; continue; }
            double s[N];
            for (int j = 0; j < N; j++) {
                double z = b[l][j];
                for (int i = 0; i < N; i++) z += w[l][j][i] * q[i];
                s[j] = sigma(z);
            }
            for (int i = 0; i < N; i++)
                for (int j = 0; j < N; j++) p[i] += c * w[l][j][i] * s[j];
        }
    }
    return 0.5 * (q[0] - t) * (q[0] - t);
}

static double rw(int l, int j, int i) { return (((l * 5 + j * 7 + i * 3) % 9) - 4) / 16.0; }
static double rb(int l, int j) { return (((l * 3 + j * 2) % 5) - 2) / 16.0; }

int main(void) {
    const double h = 0.25;
    int64_t H = (int64_t)llround(h * S);
    double xs[4][N] = {{1, 1, 1, 0}, {1, -1, 1, 0}, {-1, 1, 1, 0}, {-1, -1, 1, 0}};
    double ts[4] = {-0.5, 0.5, 0.5, -0.5};

    W3 ws; B2 bs;
    double dw[L][N][N], db[L][N];
    for (int l = 0; l < L; l++)
        for (int j = 0; j < N; j++) {
            bs[l][j] = llround(rb(l, j) * S); db[l][j] = bs[l][j] / S;
            for (int i = 0; i < N; i++) { ws[l][j][i] = llround(rw(l, j, i) * S); dw[l][j][i] = ws[l][j][i] / S; }
        }

    /* Gradients from the reverse sweep, summed over the four samples: the matrices in Q24. */
    W3 gw; B2 gb;
    memset(gw, 0, sizeof gw); memset(gb, 0, sizeof gb);
    int64_t total = 0;
    for (int n = 0; n < 4; n++) {
        int64_t q[N], p[N] = {0}, aq[N] = {0}, ap[N] = {0}, t = llround(ts[n] * S);
        for (int i = 0; i < N; i++) q[i] = llround(xs[n][i] * S);
        int64_t q0[N]; memcpy(q0, q, sizeof q);
        int64_t kind = 1;
        g(&total, q, p, aq, ap, (int64_t*)gw, (int64_t*)gb, (int64_t*)ws, (int64_t*)bs, &H, &kind, &t);
        /* The activations were rebuilt, not stored: the input comes back bit for bit. */
        if (memcmp(q, q0, sizeof q) != 0) return 1;
        for (int i = 0; i < N; i++) if (p[i] != 0) return 2;
    }

    /* Central finite differences on the double network. */
    double worst = 0, scale = 0, eps = 1e-6;
    for (int l = 0; l < L; l++)
        for (int j = 0; j < N; j++)
            for (int i = 0; i < N; i++) {
                double up = 0, dn = 0;
                dw[l][j][i] += eps;
                for (int n = 0; n < 4; n++) up += ref_loss(dw, db, xs[n], ts[n], H / S);
                dw[l][j][i] -= 2 * eps;
                for (int n = 0; n < 4; n++) dn += ref_loss(dw, db, xs[n], ts[n], H / S);
                dw[l][j][i] += eps;
                double numeric = (up - dn) / (2 * eps), got = gw[l][j][i] / (S * S);
                if (fabs(numeric - got) > worst) worst = fabs(numeric - got);
                if (fabs(numeric) > scale) scale = fabs(numeric);
            }
    for (int l = 0; l < L; l++)
        for (int j = 0; j < N; j++) {
            double up = 0, dn = 0;
            db[l][j] += eps;
            for (int n = 0; n < 4; n++) up += ref_loss(dw, db, xs[n], ts[n], H / S);
            db[l][j] -= 2 * eps;
            for (int n = 0; n < 4; n++) dn += ref_loss(dw, db, xs[n], ts[n], H / S);
            db[l][j] += eps;
            double numeric = (up - dn) / (2 * eps), got = gb[l][j] / S;
            if (fabs(numeric - got) > worst) worst = fabs(numeric - got);
            if (fabs(numeric) > scale) scale = fabs(numeric);
        }
    fprintf(stderr, "largest gradient %.4f, worst error %.5f\n", scale, worst);
    if (scale < 0.01) return 3;
    if (worst > 0.05 * scale + 0.002) return 4;

    /* The forward pass alone reverses exactly. */
    int64_t q[N], p[N] = {0}, q0[N];
    for (int i = 0; i < N; i++) q[i] = q0[i] = llround(xs[1][i] * S);
    int64_t kind = 1;
    fwd(q, p, (int64_t*)ws, (int64_t*)bs, &H, &kind);
    if (memcmp(q, q0, sizeof q) == 0) return 5;
    fwd_inv(q, p, (int64_t*)ws, (int64_t*)bs, &H, &kind);
    if (memcmp(q, q0, sizeof q) != 0) return 6;
    for (int i = 0; i < N; i++) if (p[i] != 0) return 7;
    return 0;
}
"#;

const TRAIN: &str = r#"
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define N 4
#define L 2
#define S 4096.0

void train(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
           int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);

static double rw(int l, int j, int i) { return (((l * 5 + j * 7 + i * 3) % 9) - 4) / 16.0; }
static double rb(int l, int j) { return (((l * 3 + j * 2) % 5) - 2) / 16.0; }

int main(void) {
    int64_t H = llround(0.25 * S), LR = llround(LEARNING_RATE * S), kind = 1;
    double xd[4][N] = {{1, 1, 1, 0}, {1, -1, 1, 0}, {-1, 1, 1, 0}, {-1, -1, 1, 0}};
    double td[4] = {-0.5, 0.5, 0.5, -0.5};
    int64_t xs[4][N], ts[4][1];
    for (int n = 0; n < 4; n++) {
        for (int i = 0; i < N; i++) xs[n][i] = llround(xd[n][i] * S);
        ts[n][0] = llround(td[n] * S);
    }
    int64_t ws[L][N][N], bs[L][N], gw[L][N][N] = {{{0}}}, gb[L][N] = {{0}};
    for (int l = 0; l < L; l++)
        for (int j = 0; j < N; j++) {
            bs[l][j] = llround(rb(l, j) * S);
            for (int i = 0; i < N; i++) ws[l][j][i] = llround(rw(l, j, i) * S);
        }
    int64_t q[N] = {0}, p[N] = {0}, aq[N] = {0}, ap[N] = {0};
    double first = 0, last = 0;
    for (int epoch = 0; epoch < EPOCHS; epoch++) {
        int64_t total = 0;
        train(&total, (int64_t*)ws, (int64_t*)bs, (int64_t*)gw, (int64_t*)gb, q, p, aq, ap,
              (int64_t*)xs, (int64_t*)ts, &H, &LR, &kind);
        last = total / S;
        if (epoch == 0) first = last;
    }
    fprintf(stderr, "loss %.4f -> %.4f\n", first, last);
    return last < first / 20 ? 0 : 1;
}
"#;

fn weave_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/weave")
}

fn config() -> String {
    format!(
        "[modules]\nweave = \"{}\"\neinsum = \"{}\"\n",
        weave_dir().display(),
        weave_dir().with_file_name("einsum").display()
    )
}

fn run(name: &str, driver: &str) -> Option<i32> {
    let dir = project(name, &config(), PROGRAM);
    std::fs::write(dir.join("main.c"), driver).unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let result = Command::new(dir.join("prog")).output().unwrap();
    eprint!("{}", String::from_utf8_lossy(&result.stderr));
    result.status.code()
}

#[test]
fn the_reverse_sweep_matches_finite_differences_and_rebuilds_the_input_exactly() {
    assert_eq!(run("weave-gradient", DRIVER), Some(0));
}

#[test]
fn training_on_xor_drives_the_loss_down_without_storing_activations() {
    let rate = std::env::var("WEAVE_LR").unwrap_or("1.0".into());
    let epochs = std::env::var("WEAVE_EPOCHS").unwrap_or("2000".into());
    let driver = TRAIN
        .replace("LEARNING_RATE", &rate)
        .replace("EPOCHS", &epochs);
    assert_eq!(run("weave-train", &driver), Some(0));
}

const BENCH_PROGRAM: &str = "
use weave::step;
use weave::step_batch;

irrev fn train_batch(total: &mut i64, ws: &mut [[[i64; 64]; 64]; 8], bs: &mut [[i64; 64]; 8],
                     gw: &mut [[[i64; 64]; 64]; 8], gb: &mut [[i64; 64]; 8],
                     q: &mut [[i64; 64]; 32], p: &mut [[i64; 64]; 32],
                     aq: &mut [[i64; 64]; 32], ap: &mut [[i64; 64]; 32],
                     xs: &[[i64; 64]; 32], ts: &[[i64; 4]; 32], h: &i64, lr: &i64, kind: &i64) {
    call step_batch<64, 64, 8, 4, 32>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr, kind);
}

irrev fn train(total: &mut i64, ws: &mut [[[i64; 64]; 64]; 8], bs: &mut [[i64; 64]; 8],
               gw: &mut [[[i64; 64]; 64]; 8], gb: &mut [[i64; 64]; 8],
               q: &mut [i64; 64], p: &mut [i64; 64], aq: &mut [i64; 64], ap: &mut [i64; 64],
               xs: &[[i64; 64]; 32], ts: &[[i64; 4]; 32], h: &i64, lr: &i64, kind: &i64) {
    call step<64, 64, 8, 4, 32>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr, kind);
}
";

const BENCH_DRIVER: &str = r#"
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

#define N 64
#define L 8
#define B 32
static int64_t ws[L][N][N], bs[L][N], gw[L][N][N], gb[L][N];
static int64_t xs[B][N], ts[B][4], q[N], p[N], aq[N], ap[N];
static int64_t qb[B][N], pb[B][N], aqb[B][N], apb[B][N];

void train(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
           int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);

void train_batch(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
                 int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);

static void reset(void) {
    memset(gw, 0, sizeof gw);
    memset(gb, 0, sizeof gb);
    for (int l = 0; l < L; l++)
        for (int j = 0; j < N; j++) {
            bs[l][j] = (j % 5 - 2) * 64;
            for (int i = 0; i < N; i++) ws[l][j][i] = ((l * 5 + j * 7 + i * 3) % 9 - 4) * 64;
        }
}

static double now(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return t.tv_sec + t.tv_nsec / 1e9;
}

#define ROUNDS 9
#define MANY 20
#define STEPS 10

int main(int argc, char **argv) {
    int only_batched = argc > 1;
    for (int n = 0; n < B; n++) {
        for (int i = 0; i < N; i++) xs[n][i] = ((n * 3 + i) % 7 - 3) * 512;
        for (int k = 0; k < 4; k++) ts[n][k] = ((n + k) % 3 - 1) * 512;
    }
    int64_t H = 1024, LR = 64, kind = 1, total = 0;
    double best_one = 1e9, best_batch = 1e9;
    for (int round = 0; round < (only_batched ? MANY : ROUNDS); round++) {
        reset();
        double t = now();
        for (int s = 0; s < (only_batched ? 0 : STEPS); s++)
            train(&total, (int64_t*)ws, (int64_t*)bs, (int64_t*)gw, (int64_t*)gb, q, p, aq, ap,
                  (int64_t*)xs, (int64_t*)ts, &H, &LR, &kind);
        t = (now() - t) / STEPS;
        if (round > 0 && t < best_one && !only_batched) best_one = t;
        reset();
        t = now();
        for (int s = 0; s < STEPS; s++)
            train_batch(&total, (int64_t*)ws, (int64_t*)bs, (int64_t*)gw, (int64_t*)gb, (int64_t*)qb,
                        (int64_t*)pb, (int64_t*)aqb, (int64_t*)apb, (int64_t*)xs, (int64_t*)ts, &H, &LR, &kind);
        t = (now() - t) / STEPS;
        if (round > 0 && t < best_batch) best_batch = t;
    }
    fprintf(stderr, "weave: %d layers of width %d, %d samples a step: %.1f samples/s, %.2f ms a step\n",
            L, N, B, B / best_one, 1000 * best_one);
    reset();
    for (int s = 0; s < 3; s++)
        train_batch(&total, (int64_t*)ws, (int64_t*)bs, (int64_t*)gw, (int64_t*)gb, (int64_t*)qb,
                    (int64_t*)pb, (int64_t*)aqb, (int64_t*)apb, (int64_t*)xs, (int64_t*)ts, &H, &LR, &kind);
    int64_t sum = 0;
    for (int i = 0; i < L * N * N; i++) sum = (int64_t)((uint64_t)sum * 31u + (uint64_t)((int64_t*)ws)[i]);
    for (int i = 0; i < L * N; i++) sum = (int64_t)((uint64_t)sum * 31u + (uint64_t)((int64_t*)bs)[i]);
    fprintf(stderr, "weights checksum after 3 steps: %lld\n", (long long)sum);
    fprintf(stderr, "weave batched: %d layers of width %d, %d samples a step: %.1f samples/s, %.2f ms a step\n",
            L, N, B, B / best_batch, 1000 * best_batch);
    return 0;
}
"#;

#[test]
#[ignore = "a benchmark: run it with --ignored --nocapture"]
fn throughput_of_a_training_step() {
    let dir = project("weave-bench", &config(), BENCH_PROGRAM);
    std::fs::write(dir.join("main.c"), BENCH_DRIVER).unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let result = Command::new(dir.join("prog")).output().unwrap();
    eprint!("{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(result.status.code(), Some(0));
}

#[test]
fn lean_proves_the_forward_pass_exactly_reversible() {
    let src = "
use weave::layer;
use weave::forward;

fn one(q: &mut [i64; 2], p: &mut [i64; 2], w: &[[i64; 2]; 2], b: &[i64; 2], h: &i64, kind: &i64) {
    call layer<2, 2>(q, p, w, b, h, kind);
}
fn net(q: &mut [i64; 2], p: &mut [i64; 2], ws: &[[[i64; 2]; 2]; 2], bs: &[[i64; 2]; 2], h: &i64, kind: &i64) {
    call forward<2, 2, 2>(q, p, ws, bs, h, kind);
}
";
    let dir = project("weave-lean", &config(), src);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in [
        "weave__net__layer__2_2",
        "weave__net__forward__2_2_2",
        "one",
        "net",
    ] {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

#[test]
fn lean_proves_the_backward_pass_exactly_reversible() {
    let src = "
use weave::layer_back;
use weave::backward;

fn one(q: &mut [i64; 4], p: &mut [i64; 4], aq: &mut [i64; 4], ap: &mut [i64; 4],
       gw: &mut [[i64; 4]; 4], gb: &mut [i64; 4], w: &[[i64; 4]; 4], b: &[i64; 4], h: &i64, kind: &i64) {
    call layer_back<4, 4>(q, p, aq, ap, gw, gb, w, b, h, kind);
}
fn net(q: &mut [i64; 4], p: &mut [i64; 4], aq: &mut [i64; 4], ap: &mut [i64; 4],
       gw: &mut [[[i64; 4]; 4]; 3], gb: &mut [[i64; 4]; 3],
       ws: &[[[i64; 4]; 4]; 3], bs: &[[i64; 4]; 3], h: &i64, kind: &i64) {
    call backward<4, 4, 3>(q, p, aq, ap, gw, gb, ws, bs, h, kind);
}
";
    let dir = project("weave-lean-back", &config(), src);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in [
        "weave__net__vjp__4_4",
        "weave__net__layer_back__4_4",
        "weave__net__backward__4_4_3",
        "one",
        "net",
    ] {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

#[test]
fn lean_proves_the_reversible_mlp_block_and_its_backward_step() {
    let src = "
use weave::mlp;
use weave::mlp_back;

fn fwd(y: &mut [i64; 4], w1: &[[i64; 4]; 3], b1: &[i64; 3], w2: &[[i64; 3]; 4], b2: &[i64; 4],
       x: &[i64; 4], kind: &i64) {
    call mlp<4, 3>(y, w1, b1, w2, b2, x, kind);
}
fn back(y: &mut [i64; 4], ay: &[i64; 4], ax: &mut [i64; 4],
        gw1: &mut [[i64; 4]; 3], gb1: &mut [i64; 3], gw2: &mut [[i64; 3]; 4], gb2: &mut [i64; 4],
        w1: &[[i64; 4]; 3], b1: &[i64; 3], w2: &[[i64; 3]; 4], b2: &[i64; 4],
        x: &[i64; 4], kind: &i64) {
    call mlp_back<4, 3>(y, ay, ax, gw1, gb1, gw2, gb2, w1, b1, w2, b2, x, kind);
}
";
    let dir = project("weave-lean-mlp", &config(), src);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in [
        "weave__mlp__mlp__4_3",
        "weave__mlp__mlp_back__4_3",
        "fwd",
        "back",
    ] {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

#[test]
fn lean_proves_the_attention_block_and_its_backward_step() {
    let src = "
use weave::attn;
use weave::attn_back;

fn fwd(y: &mut [i64; 4], wq: &[[i64; 2]; 2], wk: &[[i64; 2]; 2], wv: &[[i64; 2]; 2], x: &[i64; 4]) {
    call attn<2, 2, 4>(y, wq, wk, wv, x);
}
fn back(y: &mut [i64; 4], ay: &[i64; 4], ax: &mut [i64; 4],
        gwq: &mut [[i64; 2]; 2], gwk: &mut [[i64; 2]; 2], gwv: &mut [[i64; 2]; 2],
        wq: &[[i64; 2]; 2], wk: &[[i64; 2]; 2], wv: &[[i64; 2]; 2], x: &[i64; 4]) {
    call attn_back<2, 2, 4>(y, ay, ax, gwq, gwk, gwv, wq, wk, wv, x);
}
";
    let dir = project("weave-lean-attn", &config(), src);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in [
        "weave__attention__attn__2_2_4",
        "weave__attention__attn_back__2_2_4",
    ] {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}
