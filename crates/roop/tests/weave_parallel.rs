mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{project, roop, stderr};

const PROGRAM: &str = "
use weave::step;
use weave::step_batch;
use weave::step_parallel;

irrev fn by_sample(total: &mut i64, ws: &mut [[[i64; 24]; 20]; 3], bs: &mut [[i64; 20]; 3],
                   gw: &mut [[[i64; 24]; 20]; 3], gb: &mut [[i64; 20]; 3],
                   q: &mut [i64; 24], p: &mut [i64; 24], aq: &mut [i64; 24], ap: &mut [i64; 24],
                   xs: &[[i64; 24]; 12], ts: &[[i64; 2]; 12], h: &i64, lr: &i64, kind: &i64) {
    call step<24, 20, 3, 2, 12>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr, kind);
}

irrev fn by_batch(total: &mut i64, ws: &mut [[[i64; 24]; 20]; 3], bs: &mut [[i64; 20]; 3],
                  gw: &mut [[[i64; 24]; 20]; 3], gb: &mut [[i64; 20]; 3],
                  q: &mut [[i64; 24]; 12], p: &mut [[i64; 24]; 12],
                  aq: &mut [[i64; 24]; 12], ap: &mut [[i64; 24]; 12],
                  xs: &[[i64; 24]; 12], ts: &[[i64; 2]; 12], h: &i64, lr: &i64, kind: &i64) {
    call step_batch<24, 20, 3, 2, 12>(total, ws, bs, gw, gb, q, p, aq, ap, xs, ts, h, lr, kind);
}

irrev fn by_chunks(totals: &mut [i64; 3], ws: &mut [[[i64; 24]; 20]; 3], bs: &mut [[i64; 20]; 3],
                   gw: &mut [[[[i64; 24]; 20]; 3]; 3], gb: &mut [[[i64; 20]; 3]; 3],
                   tw: &mut [[[i64; 24]; 20]; 3], tb: &mut [[i64; 20]; 3],
                   q: &mut [[[i64; 24]; 4]; 3], p: &mut [[[i64; 24]; 4]; 3],
                   aq: &mut [[[i64; 24]; 4]; 3], ap: &mut [[[i64; 24]; 4]; 3],
                   xs: &[[[i64; 24]; 4]; 3], ts: &[[[i64; 2]; 4]; 3], h: &i64, lr: &i64, kind: &i64) {
    call step_parallel<24, 20, 3, 2, 4, 3>(totals, ws, bs, gw, gb, tw, tb, q, p, aq, ap, xs, ts, h, lr, kind);
}
";

const DRIVER: &str = r#"
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define N 24
#define M 20
#define L 3
#define K 2
#define S 12
#define C 3

void by_sample(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
               int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void by_batch(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
              int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void by_chunks(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
               int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);

static uint64_t state = 99;
static int64_t rnd(int64_t range) {
    state = state * 6364136223846793005ULL + 1442695040888963407ULL;
    return (int64_t)((state >> 33) % (2 * range + 1)) - range;
}

static int64_t ws0[L][M][N], bs0[L][M], xs[S][N], ts[S][K];
static int64_t ws_a[L][M][N], bs_a[L][M], gw_a[L][M][N], gb_a[L][M];
static int64_t ws_b[L][M][N], bs_b[L][M], gw_b[L][M][N], gb_b[L][M];
static int64_t ws_c[L][M][N], bs_c[L][M], gw_c[C][L][M][N], gb_c[C][L][M], tw[L][M][N], tb[L][M];
static int64_t qa[N], pa[N], aqa[N], apa[N];
static int64_t qb[S][N], pb[S][N], aqb[S][N], apb[S][N];
static int64_t qc[S][N], pc[S][N], aqc[S][N], apc[S][N];
static int64_t totals[C];

int main(void) {
    for (int i = 0; i < L * M * N; i++) ((int64_t*)ws0)[i] = rnd(1500);
    for (int i = 0; i < L * M; i++) ((int64_t*)bs0)[i] = rnd(800);
    for (int i = 0; i < S * N; i++) ((int64_t*)xs)[i] = rnd(4000);
    for (int i = 0; i < S * K; i++) ((int64_t*)ts)[i] = rnd(3000);
    memcpy(ws_a, ws0, sizeof ws0); memcpy(ws_b, ws0, sizeof ws0); memcpy(ws_c, ws0, sizeof ws0);
    memcpy(bs_a, bs0, sizeof bs0); memcpy(bs_b, bs0, sizeof bs0); memcpy(bs_c, bs0, sizeof bs0);
    int64_t h = 1024, lr = 300, kind = 4, total_a = 0, total_b = 0, total_c = 0;
    for (int step = 0; step < 3; step++) {
        by_sample(&total_a, (int64_t*)ws_a, (int64_t*)bs_a, (int64_t*)gw_a, (int64_t*)gb_a, qa, pa, aqa, apa,
                  (int64_t*)xs, (int64_t*)ts, &h, &lr, &kind);
        by_batch(&total_b, (int64_t*)ws_b, (int64_t*)bs_b, (int64_t*)gw_b, (int64_t*)gb_b, (int64_t*)qb,
                 (int64_t*)pb, (int64_t*)aqb, (int64_t*)apb, (int64_t*)xs, (int64_t*)ts, &h, &lr, &kind);
        by_chunks(totals, (int64_t*)ws_c, (int64_t*)bs_c, (int64_t*)gw_c, (int64_t*)gb_c, (int64_t*)tw,
                  (int64_t*)tb, (int64_t*)qc, (int64_t*)pc, (int64_t*)aqc, (int64_t*)apc, (int64_t*)xs,
                  (int64_t*)ts, &h, &lr, &kind);
        for (int c = 0; c < C; c++) total_c += totals[c], totals[c] = 0;
        if (memcmp(ws_a, ws_b, sizeof ws_a) != 0 || memcmp(bs_a, bs_b, sizeof bs_a) != 0) return 1;
        if (memcmp(ws_a, ws_c, sizeof ws_a) != 0 || memcmp(bs_a, bs_c, sizeof bs_a) != 0) return 2;
    }
    if (total_a != total_b || total_a != total_c) return 3;
    if (memcmp(ws0, ws_a, sizeof ws0) == 0) return 4;
    static int64_t zero[S][N];
    if (memcmp(qb, zero, sizeof qb) || memcmp(pb, zero, sizeof pb) || memcmp(aqb, zero, sizeof aqb) ||
        memcmp(apb, zero, sizeof apb) || memcmp(qc, zero, sizeof qc) || memcmp(aqc, zero, sizeof aqc))
        return 5;
    static int64_t none[L][M][N];
    if (memcmp(gw_c, none, sizeof none) || memcmp(tw, none, sizeof none)) return 6;
    uint64_t sum = 0;
    for (int i = 0; i < L * M * N; i++) sum = sum * 31 + (uint64_t)((int64_t*)ws_a)[i];
    for (int i = 0; i < L * M; i++) sum = sum * 31 + (uint64_t)((int64_t*)bs_a)[i];
    printf("%llu %lld\n", (unsigned long long)sum, (long long)total_a);
    return 0;
}
"#;

fn config() -> String {
    let weave = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/weave");
    format!(
        "[modules]\nweave = \"{}\"\neinsum = \"{}\"\n",
        weave.display(),
        weave.with_file_name("einsum").display()
    )
}

/// Builds and runs the program, with `extra` added to Roop.toml, and returns what it prints.
fn run(name: &str, extra: &str) -> String {
    let dir = project(name, &format!("{}{extra}", config()), PROGRAM);
    std::fs::write(dir.join("main.c"), DRIVER).unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let result = Command::new(dir.join("prog")).output().unwrap();
    assert_eq!(result.status.code(), Some(0));
    String::from_utf8_lossy(&result.stdout).into_owned()
}

#[test]
fn a_step_on_chunks_that_run_on_threads_equals_a_step_on_the_whole_batch_bit_for_bit() {
    assert!(!run("weave-parallel", "").is_empty());
}

#[test]
fn zeroing_an_ancilla_that_a_call_made_changes_no_result() {
    let cleared = run("weave-cleared", "\n[optimize]\nclear_ancillas = true\n");
    let computed = run("weave-computed", "\n[optimize]\nclear_ancillas = false\n");
    assert_eq!(cleared, computed);
}

#[test]
fn the_matrix_kernels_change_no_result() {
    let kernels = run("weave-kernels", "");
    let loops = run(
        "weave-loops",
        "\n[parallel]\nsme = false\nq12 = false\n[optimize]\nclear_ancillas = false\n",
    );
    assert_eq!(kernels, loops);
}
