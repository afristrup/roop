mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{project, roop, stderr};

const PROGRAM: &str = "
use weave::attn;
use weave::attn_back;
use weave::attn_back_batch;
use weave::attn_batch;
use weave::conv;
use weave::conv_back;
use weave::conv_back_batch;
use weave::conv_batch;
use weave::grad;
use weave::grad_batch;
use weave::mlp;
use weave::mlp_back;
use weave::mlp_back_batch;
use weave::mlp_batch;
use weave::mlp_layer;
use weave::mlp_layer_back;
use weave::mlp_layer_back_batch;
use weave::mlp_layer_batch;
use weave::mlp_norm;
use weave::mlp_norm_back;
use weave::mlp_norm_back_batch;
use weave::mlp_norm_batch;
use weave::residual;
use weave::residual_back;
use weave::residual_back_batch;
use weave::residual_batch;

fn one(total: &mut i64, q: &mut [i64; @N@], p: &mut [i64; @N@], aq: &mut [i64; @N@], ap: &mut [i64; @N@],
       gw: &mut [[[i64; @N@]; @M@]; @L@], gb: &mut [[i64; @M@]; @L@],
       ws: &[[[i64; @N@]; @M@]; @L@], bs: &[[i64; @M@]; @L@], h: &i64, kind: &i64, t: &[i64; @K@]) {
    call grad<@N@, @M@, @L@, @K@>(total, q, p, aq, ap, gw, gb, ws, bs, h, kind, t);
}

fn batch(total: &mut i64, q: &mut [[i64; @N@]; @B@], p: &mut [[i64; @N@]; @B@],
         aq: &mut [[i64; @N@]; @B@], ap: &mut [[i64; @N@]; @B@],
         gw: &mut [[[i64; @N@]; @M@]; @L@], gb: &mut [[i64; @M@]; @L@],
         ws: &[[[i64; @N@]; @M@]; @L@], bs: &[[i64; @M@]; @L@], h: &i64, kind: &i64, t: &[[i64; @K@]; @B@]) {
    call grad_batch<@N@, @M@, @L@, @K@, @B@>(total, q, p, aq, ap, gw, gb, ws, bs, h, kind, t);
}

fn mlp_one(y: &mut [i64; @N@], ay: &[i64; @N@], ax: &mut [i64; @N@],
           gw1: &mut [[i64; @N@]; @M@], gb1: &mut [i64; @M@], gw2: &mut [[i64; @M@]; @N@], gb2: &mut [i64; @N@],
           w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], w2: &[[i64; @M@]; @N@], b2: &[i64; @N@],
           x: &[i64; @N@], kind: &i64) {
    call mlp<@N@, @M@>(y, w1, b1, w2, b2, x, kind);
    call mlp_back<@N@, @M@>(y, ay, ax, gw1, gb1, gw2, gb2, w1, b1, w2, b2, x, kind);
}

fn mlp_all(y: &mut [[i64; @N@]; @B@], ay: &[[i64; @N@]; @B@], ax: &mut [[i64; @N@]; @B@],
           gw1: &mut [[i64; @N@]; @M@], gb1: &mut [i64; @M@], gw2: &mut [[i64; @M@]; @N@], gb2: &mut [i64; @N@],
           w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], w2: &[[i64; @M@]; @N@], b2: &[i64; @N@],
           x: &[[i64; @N@]; @B@], kind: &i64) {
    call mlp_batch<@N@, @M@, @B@>(y, w1, b1, w2, b2, x, kind);
    call mlp_back_batch<@N@, @M@, @B@>(y, ay, ax, gw1, gb1, gw2, gb2, w1, b1, w2, b2, x, kind);
}

fn attn_one(y: &mut [i64; @N@], ay: &[i64; @N@], ax: &mut [i64; @N@],
            gwq: &mut [[i64; @D@]; @D@], gwk: &mut [[i64; @D@]; @D@], gwv: &mut [[i64; @D@]; @D@],
            wq: &[[i64; @D@]; @D@], wk: &[[i64; @D@]; @D@], wv: &[[i64; @D@]; @D@], x: &[i64; @N@]) {
    call attn<@S@, @D@, @N@>(y, wq, wk, wv, x);
    call attn_back<@S@, @D@, @N@>(y, ay, ax, gwq, gwk, gwv, wq, wk, wv, x);
}

fn attn_all(y: &mut [[i64; @N@]; @B@], ay: &[[i64; @N@]; @B@], ax: &mut [[i64; @N@]; @B@],
            gwq: &mut [[i64; @D@]; @D@], gwk: &mut [[i64; @D@]; @D@], gwv: &mut [[i64; @D@]; @D@],
            wq: &[[i64; @D@]; @D@], wk: &[[i64; @D@]; @D@], wv: &[[i64; @D@]; @D@], x: &[[i64; @N@]; @B@]) {
    call attn_batch<@S@, @D@, @N@, @B@>(y, wq, wk, wv, x);
    call attn_back_batch<@S@, @D@, @N@, @B@>(y, ay, ax, gwq, gwk, gwv, wq, wk, wv, x);
}

fn norm_one(y: &mut [i64; @N@], ay: &[i64; @N@], ax: &mut [i64; @N@],
            gw1: &mut [[i64; @N@]; @M@], gb1: &mut [i64; @M@], gg: &mut [i64; @M@],
            gw2: &mut [[i64; @M@]; @N@], gb2: &mut [i64; @N@],
            w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], g: &[i64; @M@], w2: &[[i64; @M@]; @N@], b2: &[i64; @N@],
            x: &[i64; @N@], kind: &i64, eps: &i64) {
    call mlp_norm<@N@, @M@>(y, w1, b1, g, w2, b2, x, kind, eps);
    call mlp_norm_back<@N@, @M@>(y, ay, ax, gw1, gb1, gg, gw2, gb2, w1, b1, g, w2, b2, x, kind, eps);
}

fn norm_all(y: &mut [[i64; @N@]; @B@], ay: &[[i64; @N@]; @B@], ax: &mut [[i64; @N@]; @B@],
            gw1: &mut [[i64; @N@]; @M@], gb1: &mut [i64; @M@], gg: &mut [i64; @M@],
            gw2: &mut [[i64; @M@]; @N@], gb2: &mut [i64; @N@],
            w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], g: &[i64; @M@], w2: &[[i64; @M@]; @N@], b2: &[i64; @N@],
            x: &[[i64; @N@]; @B@], kind: &i64, eps: &i64) {
    call mlp_norm_batch<@N@, @M@, @B@>(y, w1, b1, g, w2, b2, x, kind, eps);
    call mlp_norm_back_batch<@N@, @M@, @B@>(y, ay, ax, gw1, gb1, gg, gw2, gb2, w1, b1, g, w2, b2, x, kind, eps);
}

fn layer_one(y: &mut [i64; @N@], ay: &[i64; @N@], ax: &mut [i64; @N@],
             gw1: &mut [[i64; @N@]; @M@], gb1: &mut [i64; @M@], gg: &mut [i64; @M@], gbeta: &mut [i64; @M@],
             gw2: &mut [[i64; @M@]; @N@], gb2: &mut [i64; @N@],
             w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], g: &[i64; @M@], beta: &[i64; @M@],
             w2: &[[i64; @M@]; @N@], b2: &[i64; @N@], x: &[i64; @N@], kind: &i64, eps: &i64) {
    call mlp_layer<@N@, @M@>(y, w1, b1, g, beta, w2, b2, x, kind, eps);
    call mlp_layer_back<@N@, @M@>(y, ay, ax, gw1, gb1, gg, gbeta, gw2, gb2, w1, b1, g, beta, w2, b2, x, kind, eps);
}

fn layer_all(y: &mut [[i64; @N@]; @B@], ay: &[[i64; @N@]; @B@], ax: &mut [[i64; @N@]; @B@],
             gw1: &mut [[i64; @N@]; @M@], gb1: &mut [i64; @M@], gg: &mut [i64; @M@], gbeta: &mut [i64; @M@],
             gw2: &mut [[i64; @M@]; @N@], gb2: &mut [i64; @N@],
             w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], g: &[i64; @M@], beta: &[i64; @M@],
             w2: &[[i64; @M@]; @N@], b2: &[i64; @N@], x: &[[i64; @N@]; @B@], kind: &i64, eps: &i64) {
    call mlp_layer_batch<@N@, @M@, @B@>(y, w1, b1, g, beta, w2, b2, x, kind, eps);
    call mlp_layer_back_batch<@N@, @M@, @B@>(y, ay, ax, gw1, gb1, gg, gbeta, gw2, gb2, w1, b1, g, beta, w2, b2, x, kind, eps);
}

fn conv_one(y: &mut [i64; @N@], ay: &[i64; @N@], ax: &mut [i64; @N@],
            gw: &mut [[i64; @CK@]; @C@], gb: &mut [i64; @C@],
            w: &[[i64; @CK@]; @C@], b: &[i64; @C@], x: &[i64; @N@]) {
    call conv<@C@, @KS@, @T@, @N@, @CK@>(y, w, b, x);
    call conv_back<@C@, @KS@, @T@, @N@, @CK@>(y, ay, ax, gw, gb, w, b, x);
}

fn conv_all(y: &mut [[i64; @N@]; @B@], ay: &[[i64; @N@]; @B@], ax: &mut [[i64; @N@]; @B@],
            gw: &mut [[i64; @CK@]; @C@], gb: &mut [i64; @C@],
            w: &[[i64; @CK@]; @C@], b: &[i64; @C@], x: &[[i64; @N@]; @B@]) {
    call conv_batch<@C@, @KS@, @T@, @N@, @CK@, @B@, @B@ * @T@>(y, w, b, x);
    call conv_back_batch<@C@, @KS@, @T@, @N@, @CK@, @B@, @B@ * @T@>(y, ay, ax, gw, gb, w, b, x);
}

fn res_one(q: &mut [i64; @N@], p: &mut [i64; @N@], aq: &mut [i64; @N@], ap: &mut [i64; @N@],
           gw1: &mut [[i64; @N@]; @M@], gb1: &mut [i64; @M@], gw2: &mut [[i64; @M@]; @N@], gb2: &mut [i64; @N@],
           w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], w2: &[[i64; @M@]; @N@], b2: &[i64; @N@], kind: &i64) {
    call residual<@N@, @M@, @R@>(q, p, w1, b1, w2, b2, kind);
    call residual_back<@N@, @M@, @R@>(q, p, aq, ap, gw1, gb1, gw2, gb2, w1, b1, w2, b2, kind);
}

fn res_all(q: &mut [[i64; @N@]; @B@], p: &mut [[i64; @N@]; @B@], aq: &mut [[i64; @N@]; @B@], ap: &mut [[i64; @N@]; @B@],
           gw1: &mut [[i64; @N@]; @M@], gb1: &mut [i64; @M@], gw2: &mut [[i64; @M@]; @N@], gb2: &mut [i64; @N@],
           w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], w2: &[[i64; @M@]; @N@], b2: &[i64; @N@], kind: &i64) {
    call residual_batch<@N@, @M@, @R@, @B@>(q, p, w1, b1, w2, b2, kind);
    call residual_back_batch<@N@, @M@, @R@, @B@>(q, p, aq, ap, gw1, gb1, gw2, gb2, w1, b1, w2, b2, kind);
}

fn res_fwd_one(q: &mut [i64; @N@], p: &mut [i64; @N@],
               w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], w2: &[[i64; @M@]; @N@], b2: &[i64; @N@], kind: &i64) {
    call residual<@N@, @M@, @R@>(q, p, w1, b1, w2, b2, kind);
}

fn res_fwd_all(q: &mut [[i64; @N@]; @B@], p: &mut [[i64; @N@]; @B@],
               w1: &[[i64; @N@]; @M@], b1: &[i64; @M@], w2: &[[i64; @M@]; @N@], b2: &[i64; @N@], kind: &i64) {
    call residual_batch<@N@, @M@, @R@, @B@>(q, p, w1, b1, w2, b2, kind);
}
";

const DRIVER: &str = r#"
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define N @N@
#define M @M@
#define L @L@
#define K @K@
#define B @B@
#define S @S@
#define D @D@
#define C @C@
#define KS @KS@
#define T @T@
#define CK @CK@
#define R @R@

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

void norm_one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
              int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void norm_all(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
              int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void layer_one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
               int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void layer_all(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
               int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void conv_one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void conv_all(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void res_one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
             int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void res_all(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*,
             int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void res_fwd_one(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void res_fwd_all(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);

static uint64_t state = 12345;
static int64_t rnd(int64_t range) {
    state = state * 6364136223846793005ULL + 1442695040888963407ULL;
    return (int64_t)((state >> 33) % (2 * range + 1)) - range;
}
static void fill(int64_t *v, int n, int64_t range) { for (int i = 0; i < n; i++) v[i] = rnd(range); }

static int64_t ws[L][M][N], bs[L][M], x[B][N], t[B][K];
static int64_t gain[M], beta[M], filters[C][CK], bias[C];
static int64_t rw1[M][N], rb1[M], rw2[N][M], rb2[N];
static int64_t w1[M][N], b1[M], w2[N][M], b2[N], ay[B][N], wq[D][D], wk[D][D], wv[D][D];

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
    static int64_t ga[3][D][D], gb[3][D][D], axa[B][N], axb[B][N], ya[B][N], yb[B][N];
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

static int check_norm(int64_t kind, int layered) {
    int64_t eps = 41;
    static int64_t gw1a[M][N], gb1a[M], gga[M], gba[M], gw2a[N][M], gb2a[N];
    static int64_t gw1b[M][N], gb1b[M], ggb[M], gbb[M], gw2b[N][M], gb2b[N];
    static int64_t axa[B][N], axb[B][N], ya[B][N], yb[B][N];
    memset(gw1a, 0, sizeof gw1a); memset(gb1a, 0, sizeof gb1a); memset(gga, 0, sizeof gga);
    memset(gba, 0, sizeof gba); memset(gw2a, 0, sizeof gw2a); memset(gb2a, 0, sizeof gb2a);
    memset(gw1b, 0, sizeof gw1b); memset(gb1b, 0, sizeof gb1b); memset(ggb, 0, sizeof ggb);
    memset(gbb, 0, sizeof gbb); memset(gw2b, 0, sizeof gw2b); memset(gb2b, 0, sizeof gb2b);
    memset(axa, 0, sizeof axa); memset(axb, 0, sizeof axb); memset(ya, 0, sizeof ya); memset(yb, 0, sizeof yb);
    for (int s = 0; s < B; s++) {
        if (layered)
            layer_one(ya[s], ay[s], axa[s], (int64_t*)gw1a, gb1a, gga, gba, (int64_t*)gw2a, gb2a,
                      (int64_t*)w1, b1, gain, beta, (int64_t*)w2, b2, x[s], &kind, &eps);
        else
            norm_one(ya[s], ay[s], axa[s], (int64_t*)gw1a, gb1a, gga, (int64_t*)gw2a, gb2a,
                     (int64_t*)w1, b1, gain, (int64_t*)w2, b2, x[s], &kind, &eps);
    }
    if (layered)
        layer_all((int64_t*)yb, (int64_t*)ay, (int64_t*)axb, (int64_t*)gw1b, gb1b, ggb, gbb, (int64_t*)gw2b, gb2b,
                  (int64_t*)w1, b1, gain, beta, (int64_t*)w2, b2, (int64_t*)x, &kind, &eps);
    else
        norm_all((int64_t*)yb, (int64_t*)ay, (int64_t*)axb, (int64_t*)gw1b, gb1b, ggb, (int64_t*)gw2b, gb2b,
                 (int64_t*)w1, b1, gain, (int64_t*)w2, b2, (int64_t*)x, &kind, &eps);
    if (memcmp(axa, axb, sizeof axa) != 0) return 31;
    if (memcmp(gw1a, gw1b, sizeof gw1a) != 0) return 32;
    if (memcmp(gb1a, gb1b, sizeof gb1a) != 0) return 33;
    if (memcmp(gga, ggb, sizeof gga) != 0) return 34;
    if (memcmp(gba, gbb, sizeof gba) != 0) return 35;
    if (memcmp(gw2a, gw2b, sizeof gw2a) != 0) return 36;
    if (memcmp(gb2a, gb2b, sizeof gb2a) != 0) return 37;
    static int64_t zero[B][N], none[M][N];
    if (memcmp(yb, zero, sizeof yb) != 0) return 38;
    if (memcmp(gw1b, none, sizeof gw1b) == 0) return 39;
    return 0;
}

static int check_conv(void) {
    static int64_t ga[C][CK], gb[C], gc[C][CK], gd[C], axa[B][N], axb[B][N], ya[B][N], yb[B][N];
    memset(ga, 0, sizeof ga); memset(gb, 0, sizeof gb); memset(gc, 0, sizeof gc); memset(gd, 0, sizeof gd);
    memset(axa, 0, sizeof axa); memset(axb, 0, sizeof axb); memset(ya, 0, sizeof ya); memset(yb, 0, sizeof yb);
    for (int s = 0; s < B; s++)
        conv_one(ya[s], ay[s], axa[s], (int64_t*)ga, gb, (int64_t*)filters, bias, x[s]);
    conv_all((int64_t*)yb, (int64_t*)ay, (int64_t*)axb, (int64_t*)gc, gd, (int64_t*)filters, bias, (int64_t*)x);
    if (memcmp(axa, axb, sizeof axa) != 0) return 41;
    if (memcmp(ga, gc, sizeof ga) != 0) return 42;
    if (memcmp(gb, gd, sizeof gb) != 0) return 43;
    static int64_t zero[B][N], none[C][CK];
    if (memcmp(yb, zero, sizeof yb) != 0) return 44;
    if (memcmp(gc, none, sizeof gc) == 0) return 45;
    return 0;
}

static int check_residual(int64_t kind) {
    static int64_t gw1a[M][N], gb1a[M], gw2a[N][M], gb2a[N], gw1b[M][N], gb1b[M], gw2b[N][M], gb2b[N];
    static int64_t qa[B][N], pa[B][N], aqa[B][N], apa[B][N], qb[B][N], pb[B][N], aqb[B][N], apb[B][N];
    static int64_t fa[B][N], fb[B][N], fpa[B][N], fpb[B][N];
    memset(gw1a, 0, sizeof gw1a); memset(gb1a, 0, sizeof gb1a); memset(gw2a, 0, sizeof gw2a); memset(gb2a, 0, sizeof gb2a);
    memset(gw1b, 0, sizeof gw1b); memset(gb1b, 0, sizeof gb1b); memset(gw2b, 0, sizeof gw2b); memset(gb2b, 0, sizeof gb2b);
    memset(pa, 0, sizeof pa); memset(pb, 0, sizeof pb); memset(fpa, 0, sizeof fpa); memset(fpb, 0, sizeof fpb);
    memcpy(qa, x, sizeof qa); memcpy(qb, x, sizeof qb); memcpy(fa, x, sizeof fa); memcpy(fb, x, sizeof fb);
    memcpy(aqa, ay, sizeof aqa); memcpy(aqb, ay, sizeof aqb);
    memset(apa, 0, sizeof apa); memset(apb, 0, sizeof apb);
    for (int s = 0; s < B; s++)
        res_fwd_one(fa[s], fpa[s], (int64_t*)rw1, rb1, (int64_t*)rw2, rb2, &kind);
    res_fwd_all((int64_t*)fb, (int64_t*)fpb, (int64_t*)rw1, rb1, (int64_t*)rw2, rb2, &kind);
    if (memcmp(fa, fb, sizeof fa) != 0) return 51;
    if (memcmp(fpa, fpb, sizeof fpa) != 0) return 52;
    for (int s = 0; s < B; s++)
        res_one(qa[s], pa[s], aqa[s], apa[s], (int64_t*)gw1a, gb1a, (int64_t*)gw2a, gb2a,
                (int64_t*)rw1, rb1, (int64_t*)rw2, rb2, &kind);
    res_all((int64_t*)qb, (int64_t*)pb, (int64_t*)aqb, (int64_t*)apb, (int64_t*)gw1b, gb1b, (int64_t*)gw2b, gb2b,
            (int64_t*)rw1, rb1, (int64_t*)rw2, rb2, &kind);
    if (memcmp(qa, qb, sizeof qa) != 0) return 53;
    if (memcmp(qb, x, sizeof qb) != 0) return 54;
    if (memcmp(pa, pb, sizeof pa) != 0) return 55;
    if (memcmp(aqa, aqb, sizeof aqa) != 0) return 56;
    if (memcmp(apa, apb, sizeof apa) != 0) return 57;
    if (memcmp(gw1a, gw1b, sizeof gw1a) != 0) return 58;
    if (memcmp(gb1a, gb1b, sizeof gb1a) != 0) return 59;
    if (memcmp(gw2a, gw2b, sizeof gw2a) != 0) return 60;
    if (memcmp(gb2a, gb2b, sizeof gb2a) != 0) return 61;
    static int64_t none[M][N];
    if (memcmp(gw1b, none, sizeof gw1b) == 0) return 62;
    return 0;
}

int main(void) {
    fill((int64_t*)ws, L * M * N, 2000); fill((int64_t*)bs, L * M, 1000);
    fill((int64_t*)x, B * N, 6000); fill((int64_t*)t, B * K, 3000);
    fill((int64_t*)w1, M * N, 2000); fill(b1, M, 1000); fill((int64_t*)w2, N * M, 2000); fill(b2, N, 1000);
    fill((int64_t*)ay, B * N, 3000);
    fill((int64_t*)rw1, M * N, 1200); fill(rb1, M, 300); fill((int64_t*)rw2, N * M, 1200); fill(rb2, N, 300);
    fill(gain, M, 4096); fill(beta, M, 2000);
    fill((int64_t*)filters, C * CK, 3000); fill(bias, C, 1000);
    fill((int64_t*)wq, D * D, 3000); fill((int64_t*)wk, D * D, 3000); fill((int64_t*)wv, D * D, 3000);
    for (int64_t kind = 0; kind < 8; kind++) {
        int r = check_grad(kind);
        if (r) { fprintf(stderr, "grad kind %lld failed with %d\n", (long long)kind, r); return r; }
        r = check_mlp(kind);
        if (r) { fprintf(stderr, "mlp kind %lld failed with %d\n", (long long)kind, r); return r; }
        r = check_norm(kind, 0);
        if (r) { fprintf(stderr, "mlp_norm kind %lld failed with %d\n", (long long)kind, r); return r; }
        r = check_norm(kind, 1);
        if (r) { fprintf(stderr, "mlp_layer kind %lld failed with %d\n", (long long)kind, r); return r; }
        r = check_residual(kind);
        if (r) { fprintf(stderr, "residual kind %lld failed with %d\n", (long long)kind, r); return r; }
    }
    int r = check_attn();
    if (r) { fprintf(stderr, "attention failed with %d\n", r); return r; }
    r = check_conv();
    if (r) { fprintf(stderr, "conv failed with %d\n", r); return r; }
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

struct Sizes {
    n: usize,
    m: usize,
    b: usize,
    s: usize,
    d: usize,
    c: usize,
    ks: usize,
}

const SMALL: Sizes = Sizes {
    n: 8,
    m: 6,
    b: 5,
    s: 4,
    d: 2,
    c: 2,
    ks: 3,
};

/// Large enough that the matrix products run on the runtime's kernel.
const LARGE: Sizes = Sizes {
    n: 24,
    m: 20,
    b: 9,
    s: 6,
    d: 4,
    c: 4,
    ks: 3,
};

fn compare(name: &str, sizes: Sizes) {
    let fill = |text: &str| {
        [
            ("@N@", sizes.n),
            ("@M@", sizes.m),
            ("@L@", 3),
            ("@K@", 2),
            ("@B@", sizes.b),
            ("@S@", sizes.s),
            ("@D@", sizes.d),
            ("@C@", sizes.c),
            ("@KS@", sizes.ks),
            ("@T@", sizes.n / sizes.c),
            ("@CK@", sizes.c * sizes.ks),
            ("@R@", 6),
        ]
        .iter()
        .fold(text.to_string(), |text, (at, n)| {
            text.replace(at, &n.to_string())
        })
    };
    let dir = project(name, &config(), &fill(PROGRAM));
    std::fs::write(dir.join("main.c"), fill(DRIVER)).unwrap();
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
fn batched_gradients_equal_the_sum_of_the_per_sample_gradients_bit_for_bit() {
    compare("weave-batch-small", SMALL);
}

#[test]
fn the_same_holds_when_the_products_run_on_the_kernel() {
    compare("weave-batch-large", LARGE);
}
