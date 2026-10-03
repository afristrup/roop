mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{project, roop, stderr};

const PROGRAM: &str = "
use weave::attn;
use weave::attn_back;
use weave::attn_back_batch;
use weave::attn_batch;
use weave::grad;
use weave::grad_batch;
use weave::mlp;
use weave::mlp_back;
use weave::mlp_back_batch;
use weave::mlp_batch;

fn one(total: &mut i64, q: &mut [i64; 8], p: &mut [i64; 8], aq: &mut [i64; 8], ap: &mut [i64; 8],
       gw: &mut [[[i64; 8]; 6]; 3], gb: &mut [[i64; 6]; 3],
       ws: &[[[i64; 8]; 6]; 3], bs: &[[i64; 6]; 3], h: &i64, kind: &i64, t: &[i64; 2]) {
    call grad<8, 6, 3, 2>(total, q, p, aq, ap, gw, gb, ws, bs, h, kind, t);
}

fn batch(total: &mut i64, q: &mut [[i64; 8]; 5], p: &mut [[i64; 8]; 5],
         aq: &mut [[i64; 8]; 5], ap: &mut [[i64; 8]; 5],
         gw: &mut [[[i64; 8]; 6]; 3], gb: &mut [[i64; 6]; 3],
         ws: &[[[i64; 8]; 6]; 3], bs: &[[i64; 6]; 3], h: &i64, kind: &i64, t: &[[i64; 2]; 5]) {
    call grad_batch<8, 6, 3, 2, 5>(total, q, p, aq, ap, gw, gb, ws, bs, h, kind, t);
}

fn mlp_one(y: &mut [i64; 8], ay: &[i64; 8], ax: &mut [i64; 8],
           gw1: &mut [[i64; 8]; 6], gb1: &mut [i64; 6], gw2: &mut [[i64; 6]; 8], gb2: &mut [i64; 8],
           w1: &[[i64; 8]; 6], b1: &[i64; 6], w2: &[[i64; 6]; 8], b2: &[i64; 8],
           x: &[i64; 8], kind: &i64) {
    call mlp<8, 6>(y, w1, b1, w2, b2, x, kind);
    call mlp_back<8, 6>(y, ay, ax, gw1, gb1, gw2, gb2, w1, b1, w2, b2, x, kind);
}

fn mlp_all(y: &mut [[i64; 8]; 5], ay: &[[i64; 8]; 5], ax: &mut [[i64; 8]; 5],
           gw1: &mut [[i64; 8]; 6], gb1: &mut [i64; 6], gw2: &mut [[i64; 6]; 8], gb2: &mut [i64; 8],
           w1: &[[i64; 8]; 6], b1: &[i64; 6], w2: &[[i64; 6]; 8], b2: &[i64; 8],
           x: &[[i64; 8]; 5], kind: &i64) {
    call mlp_batch<8, 6, 5>(y, w1, b1, w2, b2, x, kind);
    call mlp_back_batch<8, 6, 5>(y, ay, ax, gw1, gb1, gw2, gb2, w1, b1, w2, b2, x, kind);
}

fn attn_one(y: &mut [i64; 8], ay: &[i64; 8], ax: &mut [i64; 8],
            gwq: &mut [[i64; 2]; 2], gwk: &mut [[i64; 2]; 2], gwv: &mut [[i64; 2]; 2],
            wq: &[[i64; 2]; 2], wk: &[[i64; 2]; 2], wv: &[[i64; 2]; 2], x: &[i64; 8]) {
    call attn<4, 2, 8>(y, wq, wk, wv, x);
    call attn_back<4, 2, 8>(y, ay, ax, gwq, gwk, gwv, wq, wk, wv, x);
}

fn attn_all(y: &mut [[i64; 8]; 5], ay: &[[i64; 8]; 5], ax: &mut [[i64; 8]; 5],
            gwq: &mut [[i64; 2]; 2], gwk: &mut [[i64; 2]; 2], gwv: &mut [[i64; 2]; 2],
            wq: &[[i64; 2]; 2], wk: &[[i64; 2]; 2], wv: &[[i64; 2]; 2], x: &[[i64; 8]; 5]) {
    call attn_batch<4, 2, 8, 5>(y, wq, wk, wv, x);
    call attn_back_batch<4, 2, 8, 5>(y, ay, ax, gwq, gwk, gwv, wq, wk, wv, x);
}
";

const DRIVER: &str = r#"
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define N 8
#define M 6
#define L 3
#define K 2
#define B 5

void one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
         int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void batch(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
           int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void mlp_one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
             int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void mlp_all(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
             int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void attn_one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
              int64_t*, int64_t*, int64_t*, int64_t*);
void attn_all(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
              int64_t*, int64_t*, int64_t*, int64_t*);

static uint64_t state = 12345;
static int64_t rnd(int64_t range) {
    state = state * 6364136223846793005ULL + 1442695040888963407ULL;
    return (int64_t)((state >> 33) % (2 * range + 1)) - range;
}
static void fill(int64_t *v, int n, int64_t range) { for (int i = 0; i < n; i++) v[i] = rnd(range); }

static int64_t ws[L][M][N], bs[L][M], x[B][N], t[B][K];
static int64_t w1[M][N], b1[M], w2[N][M], b2[N], ay[B][N], wq[2][2], wk[2][2], wv[2][2];

static int check_grad(int64_t kind) {
    int64_t H = 1024, total_one = 0, total_all = 0;
    static int64_t gw1[L][M][N], gb1[L][M], gw2[L][M][N], gb2[L][M];
    static int64_t q[B][N], p[B][N], aq[B][N], ap[B][N];
    memset(gw1, 0, sizeof gw1); memset(gb1, 0, sizeof gb1);
    memset(gw2, 0, sizeof gw2); memset(gb2, 0, sizeof gb2);
    memset(q, 0, sizeof q); memset(p, 0, sizeof p); memset(aq, 0, sizeof aq); memset(ap, 0, sizeof ap);
    static int64_t aq_one[B][N];
    for (int s = 0; s < B; s++) {
        int64_t qs[N], ps[N] = {0}, aqs[N] = {0}, aps[N] = {0};
        memcpy(qs, x[s], sizeof qs);
        one(&total_one, qs, ps, aqs, aps, (int64_t*)gw1, (int64_t*)gb1, (int64_t*)ws, (int64_t*)bs,
            &H, &kind, t[s]);
        if (memcmp(qs, x[s], sizeof qs) != 0) return 1;
        memcpy(aq_one[s], aqs, sizeof aqs);
    }
    memcpy(q, x, sizeof q);
    batch(&total_all, (int64_t*)q, (int64_t*)p, (int64_t*)aq, (int64_t*)ap, (int64_t*)gw2, (int64_t*)gb2,
          (int64_t*)ws, (int64_t*)bs, &H, &kind, (int64_t*)t);
    if (memcmp(q, x, sizeof q) != 0) return 2;
    if (total_one != total_all) return 3;
    if (memcmp(gw1, gw2, sizeof gw1) != 0) return 4;
    if (memcmp(gb1, gb2, sizeof gb1) != 0) return 5;
    if (memcmp(aq_one, aq, sizeof aq) != 0) return 6;
    static int64_t zero[B][N];
    if (memcmp(p, zero, sizeof p) != 0) return 7;
    static int64_t none[L][M][N];
    if (memcmp(gw1, none, sizeof gw1) == 0) return 8;
    return 0;
}

static int check_mlp(int64_t kind) {
    static int64_t gw1a[M][N], gb1a[M], gw2a[N][M], gb2a[N], gw1b[M][N], gb1b[M], gw2b[N][M], gb2b[N];
    static int64_t axa[B][N], axb[B][N], ya[B][N], yb[B][N];
    memset(gw1a, 0, sizeof gw1a); memset(gb1a, 0, sizeof gb1a); memset(gw2a, 0, sizeof gw2a); memset(gb2a, 0, sizeof gb2a);
    memset(gw1b, 0, sizeof gw1b); memset(gb1b, 0, sizeof gb1b); memset(gw2b, 0, sizeof gw2b); memset(gb2b, 0, sizeof gb2b);
    memset(axa, 0, sizeof axa); memset(axb, 0, sizeof axb); memset(ya, 0, sizeof ya); memset(yb, 0, sizeof yb);
    for (int s = 0; s < B; s++)
        mlp_one(ya[s], ay[s], axa[s], (int64_t*)gw1a, gb1a, (int64_t*)gw2a, gb2a, (int64_t*)w1, b1,
                (int64_t*)w2, b2, x[s], &kind);
    mlp_all((int64_t*)yb, (int64_t*)ay, (int64_t*)axb, (int64_t*)gw1b, gb1b, (int64_t*)gw2b, gb2b,
            (int64_t*)w1, b1, (int64_t*)w2, b2, (int64_t*)x, &kind);
    if (memcmp(axa, axb, sizeof axa) != 0) return 11;
    if (memcmp(gw1a, gw1b, sizeof gw1a) != 0) return 12;
    if (memcmp(gb1a, gb1b, sizeof gb1a) != 0) return 13;
    if (memcmp(gw2a, gw2b, sizeof gw2a) != 0) return 14;
    if (memcmp(gb2a, gb2b, sizeof gb2a) != 0) return 15;
    static int64_t zero[B][N];
    if (memcmp(yb, zero, sizeof yb) != 0) return 16;
    return 0;
}

static int check_attn(void) {
    static int64_t ga[3][2][2], gb[3][2][2], axa[B][N], axb[B][N], ya[B][N], yb[B][N];
    memset(ga, 0, sizeof ga); memset(gb, 0, sizeof gb);
    memset(axa, 0, sizeof axa); memset(axb, 0, sizeof axb); memset(ya, 0, sizeof ya); memset(yb, 0, sizeof yb);
    for (int s = 0; s < B; s++)
        attn_one(ya[s], ay[s], axa[s], (int64_t*)ga[0], (int64_t*)ga[1], (int64_t*)ga[2],
                 (int64_t*)wq, (int64_t*)wk, (int64_t*)wv, x[s]);
    attn_all((int64_t*)yb, (int64_t*)ay, (int64_t*)axb, (int64_t*)gb[0], (int64_t*)gb[1], (int64_t*)gb[2],
             (int64_t*)wq, (int64_t*)wk, (int64_t*)wv, (int64_t*)x);
    if (memcmp(axa, axb, sizeof axa) != 0) return 21;
    if (memcmp(ga, gb, sizeof ga) != 0) return 22;
    static int64_t zero[B][N];
    if (memcmp(yb, zero, sizeof yb) != 0) return 23;
    return 0;
}

int main(void) {
    fill((int64_t*)ws, L * M * N, 2000); fill((int64_t*)bs, L * M, 1000);
    fill((int64_t*)x, B * N, 6000); fill((int64_t*)t, B * K, 3000);
    fill((int64_t*)w1, M * N, 2000); fill(b1, M, 1000); fill((int64_t*)w2, N * M, 2000); fill(b2, N, 1000);
    fill((int64_t*)ay, B * N, 3000);
    fill((int64_t*)wq, 4, 3000); fill((int64_t*)wk, 4, 3000); fill((int64_t*)wv, 4, 3000);
    for (int64_t kind = 0; kind < 8; kind++) {
        int r = check_grad(kind);
        if (r) { fprintf(stderr, "grad kind %lld failed with %d\n", (long long)kind, r); return r; }
        r = check_mlp(kind);
        if (r) { fprintf(stderr, "mlp kind %lld failed with %d\n", (long long)kind, r); return r; }
    }
    int r = check_attn();
    if (r) { fprintf(stderr, "attention failed with %d\n", r); return r; }
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

#[test]
fn batched_gradients_equal_the_sum_of_the_per_sample_gradients_bit_for_bit() {
    let dir = project("weave-batch", &config(), PROGRAM);
    std::fs::write(dir.join("main.c"), DRIVER).unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let result = Command::new(dir.join("prog")).output().unwrap();
    eprint!("{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(result.status.code(), Some(0));
}
