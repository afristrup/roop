// C += alpha * A * B for row-major f64 matrices, on the SME matrix unit of
// Apple M4 and later. Built by build.rs with clang; the compiler calls it for
// a `dgemm` loop nest (see roop-llvm's gen_gemm).
//
// A block of 16 rows of A is transposed into a panel through the ZA tiles,
// then swept against 32 columns of B with eight 8x8 FMOPA tiles. The next
// block's panel is packed between the first two tiles of this one, so the
// stores of the pack have reached L2 by the time the panel is read. A scale of
// 1 or -1 costs nothing: the pack copies A as it is and a scale of -1 uses
// FMOPS, which rounds exactly as the fused multiply-add of the negated
// product. Any other alpha is multiplied into the pack.
// Matrices not aligned to 64 bytes run about a fifth slower, since every vector
// then spans two cache lines.
// Everything stays in streaming mode, since the matrix unit reads and writes
// through L2 and a store from normal mode would have to be flushed to it first.
#include <arm_sme.h>
#include <stdint.h>

#define KC 256
#define VL 8

#define CW (4 * VL)
#define CHUNK 16
#define THREADED_WORK (1 << 23)

enum { SCALED, UNIT, NEGATED };

typedef struct {
  double lo[KC * VL], hi[KC * VL];
} Panel;

typedef struct {
  int64_t rows, kc;
  const double *a;
  Panel *panel;
} Pending;

#define MOPA(mode, tile, a, b)                                  \
  do {                                                          \
    if (mode == NEGATED) svmops_za64_f64_m(tile, all, all, a, b); \
    else svmopa_za64_f64_m(tile, all, all, a, b);               \
  } while (0)

static inline __attribute__((always_inline)) void load_c(double *c, int64_t ldc)
    __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  for (uint32_t r = 0; r < VL; r++) {
    svld1_hor_za64(0, r, all, c + r * ldc);
    svld1_hor_za64(1, r, all, c + r * ldc + VL);
    svld1_hor_za64(2, r, all, c + r * ldc + 2 * VL);
    svld1_hor_za64(3, r, all, c + r * ldc + 3 * VL);
    svld1_hor_za64(4, r, all, c + (VL + r) * ldc);
    svld1_hor_za64(5, r, all, c + (VL + r) * ldc + VL);
    svld1_hor_za64(6, r, all, c + (VL + r) * ldc + 2 * VL);
    svld1_hor_za64(7, r, all, c + (VL + r) * ldc + 3 * VL);
  }
}

static inline __attribute__((always_inline)) void store_c(double *c, int64_t ldc)
    __arm_streaming __arm_in("za") {
  svbool_t all = svptrue_b64();
  for (uint32_t r = 0; r < VL; r++) {
    svst1_hor_za64(0, r, all, c + r * ldc);
    svst1_hor_za64(1, r, all, c + r * ldc + VL);
    svst1_hor_za64(2, r, all, c + r * ldc + 2 * VL);
    svst1_hor_za64(3, r, all, c + r * ldc + 3 * VL);
    svst1_hor_za64(4, r, all, c + (VL + r) * ldc);
    svst1_hor_za64(5, r, all, c + (VL + r) * ldc + VL);
    svst1_hor_za64(6, r, all, c + (VL + r) * ldc + 2 * VL);
    svst1_hor_za64(7, r, all, c + (VL + r) * ldc + 3 * VL);
  }
}

// Stores tile t of the old block and loads it from the next, before the next
// tile is touched, so the first products of the next block can start while the
// last tiles are still being exchanged.
#define ST(t, row, col)                                    \
  do {                                                     \
    for (uint32_t r = 0; r < VL; r++)                      \
      svst1_hor_za64(t, r, all, old + (row + r) * ldc + col); \
  } while (0)

#define LD(t, row, col)                                    \
  do {                                                     \
    for (uint32_t r = 0; r < VL; r++)                      \
      svld1_hor_za64(t, r, all, next + (row + r) * ldc + col); \
  } while (0)

static inline __attribute__((always_inline)) void swap_c(double *old, double *next, int64_t ldc)
    __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  ST(0, 0, 0);
  ST(1, 0, VL);
  ST(2, 0, 2 * VL);
  ST(3, 0, 3 * VL);
  LD(0, 0, 0);
  ST(4, VL, 0);
  LD(1, 0, VL);
  ST(5, VL, VL);
  LD(2, 0, 2 * VL);
  ST(6, VL, 2 * VL);
  LD(3, 0, 3 * VL);
  ST(7, VL, 3 * VL);
  LD(4, VL, 0);
  LD(5, VL, VL);
  LD(6, VL, 2 * VL);
  LD(7, VL, 3 * VL);
}

static inline __attribute__((always_inline)) void tile(
    int mode, int64_t kc, const double *b, int64_t ldb,
    const Panel *p, svbool_t q0, svbool_t q1, svbool_t q2, svbool_t q3)
    __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  for (int64_t l = 0; l < kc; l++) {
    svfloat64_t a0 = svld1(all, p->lo + l * VL);
    svfloat64_t a1 = svld1(all, p->hi + l * VL);
    svfloat64_t b0 = svld1(q0, b + l * ldb);
    svfloat64_t b1 = svld1(q1, b + l * ldb + VL);
    svfloat64_t b2 = svld1(q2, b + l * ldb + 2 * VL);
    svfloat64_t b3 = svld1(q3, b + l * ldb + 3 * VL);
    MOPA(mode, 0, a0, b0);
    MOPA(mode, 1, a0, b1);
    MOPA(mode, 2, a0, b2);
    MOPA(mode, 3, a0, b3);
    MOPA(mode, 4, a1, b0);
    MOPA(mode, 5, a1, b1);
    MOPA(mode, 6, a1, b2);
    MOPA(mode, 7, a1, b3);
  }
}

static inline __attribute__((always_inline)) svfloat64x4_t scaled(svfloat64x4_t v, double alpha)
    __arm_streaming {
  svbool_t all = svptrue_b64();
  return svcreate4(svmul_n_f64_x(all, svget4(v, 0), alpha), svmul_n_f64_x(all, svget4(v, 1), alpha),
                   svmul_n_f64_x(all, svget4(v, 2), alpha), svmul_n_f64_x(all, svget4(v, 3), alpha));
}

#define PANEL(mode, t, x, alpha, lo, hi)                                      \
  do {                                                                        \
    svfloat64x4_t u = svread_ver_za64_f64_vg4(t, x);                          \
    svfloat64x4_t v = svread_ver_za64_f64_vg4(t + 4, x);                      \
    if (mode == SCALED) {                                                     \
      u = scaled(u, alpha);                                                   \
      v = scaled(v, alpha);                                                   \
    }                                                                         \
    svst1_f64_x4(all, lo + (8 * t + x) * VL, u);                              \
    svst1_f64_x4(all, hi + (8 * t + x) * VL, v);                              \
  } while (0)

// 16 rows by 32 columns of A, transposed into 32 vectors of each half panel.
static inline __attribute__((always_inline)) void pack_slab(
    int mode, const double *a, int64_t lda, double alpha, double *lo, double *hi)
    __arm_streaming __arm_inout("za") {
  svcount_t all = svptrue_c64();
  svbool_t pb = svptrue_b64();
  for (uint32_t r = 0; r < VL; r++) {
    svld1_hor_za64(0, r, pb, a + r * lda);
    svld1_hor_za64(4, r, pb, a + (VL + r) * lda);
    svld1_hor_za64(1, r, pb, a + r * lda + VL);
    svld1_hor_za64(5, r, pb, a + (VL + r) * lda + VL);
    svld1_hor_za64(2, r, pb, a + r * lda + 2 * VL);
    svld1_hor_za64(6, r, pb, a + (VL + r) * lda + 2 * VL);
    svld1_hor_za64(3, r, pb, a + r * lda + 3 * VL);
    svld1_hor_za64(7, r, pb, a + (VL + r) * lda + 3 * VL);
  }
  PANEL(mode, 0, 0, alpha, lo, hi);
  PANEL(mode, 0, 4, alpha, lo, hi);
  PANEL(mode, 1, 0, alpha, lo, hi);
  PANEL(mode, 1, 4, alpha, lo, hi);
  PANEL(mode, 2, 0, alpha, lo, hi);
  PANEL(mode, 2, 4, alpha, lo, hi);
  PANEL(mode, 3, 0, alpha, lo, hi);
  PANEL(mode, 3, 4, alpha, lo, hi);
}

// Up to 8 columns of up to 16 rows, with the rest of the tile zero.
static inline __attribute__((always_inline)) void pack_edge(
    int64_t rows, int64_t cols, const double *a, int64_t lda, double alpha, double *lo, double *hi)
    __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  svbool_t pl = svwhilelt_b64((int64_t)0, cols);
  svfloat64_t z = svdup_f64(0.0);
  svzero_za();
  for (uint32_t r = 0; r < VL; r++) {
    if (r < rows) svld1_hor_za64(0, r, pl, a + r * lda);
    if (VL + r < rows) svld1_hor_za64(1, r, pl, a + (VL + r) * lda);
  }
  for (int64_t x = 0; x < cols; x++) {
    svst1(all, lo + x * VL, svmul_n_f64_x(all, svread_ver_za64_f64_m(z, all, 0, x), alpha));
    svst1(all, hi + x * VL, svmul_n_f64_x(all, svread_ver_za64_f64_m(z, all, 1, x), alpha));
  }
}

static inline __attribute__((always_inline)) void pack(
    int mode, const Pending *nx, int64_t lda, double alpha) __arm_streaming __arm_inout("za") {
  double *lo = nx->panel->lo, *hi = nx->panel->hi;
  int64_t l0 = 0;
  if (nx->rows == 2 * VL)
    for (; l0 + 4 * VL <= nx->kc; l0 += 4 * VL)
      pack_slab(mode, nx->a + l0, lda, alpha, lo + l0 * VL, hi + l0 * VL);
  for (; l0 < nx->kc; l0 += VL)
    pack_edge(nx->rows, nx->kc - l0 < VL ? nx->kc - l0 : VL, nx->a + l0, lda,
              mode == SCALED ? alpha : 1.0, lo + l0 * VL, hi + l0 * VL);
}

// One block of rows against all of n, packing the next block after the first tile.
static inline __attribute__((always_inline)) void block(
    int mode, int64_t m, int64_t n, int64_t kc, int64_t ldb, int64_t ldc, double *c,
    const Panel *p, const double *b, const Pending *nx, int64_t lda, double alpha)
    __arm_streaming __arm_inout("za") {
  double edge[2 * VL * CW];
  int loaded = 0;
  for (int64_t j = 0; j < n; j += CW) {
    svbool_t q0 = svwhilelt_b64(j, n);
    svbool_t q1 = svwhilelt_b64(j + VL, n);
    svbool_t q2 = svwhilelt_b64(j + 2 * VL, n);
    svbool_t q3 = svwhilelt_b64(j + 3 * VL, n);
    int64_t w = n - j < CW ? n - j : CW;
    int full = m == 2 * VL && w == CW;
    if (!full)
      for (int64_t r = 0; r < 2 * VL; r++)
        for (int64_t x = 0; x < CW; x++)
          edge[r * CW + x] = r < m && x < w ? c[r * ldc + j + x] : 0.0;
    if (!loaded) {
      if (full) load_c(c + j, ldc);
      else load_c(edge, CW);
    }
    tile(mode, kc, b + j, ldb, p, q0, q1, q2, q3);
    int chain = j > 0 && full && m == 2 * VL && n - j - CW >= CW;
    if (chain) {
      swap_c(c + j, c + j + CW, ldc);
    } else if (full) {
      store_c(c + j, ldc);
    } else {
      store_c(edge, CW);
      for (int64_t r = 0; r < m; r++)
        for (int64_t x = 0; x < w; x++) c[r * ldc + j + x] = edge[r * CW + x];
    }
    loaded = chain;
    if (j == 0 && nx->panel) pack(mode, nx, lda, alpha);
  }
}

// The s-th block of 16 rows by KC columns of A, in row-major order of blocks.
static inline __attribute__((always_inline)) Pending step(
    int64_t s, const double *a, int64_t m, int64_t k, Panel *panel) __arm_streaming_compatible {
  int64_t blocks = (k + KC - 1) / KC;
  int64_t i = s / blocks * 2 * VL, k0 = s % blocks * KC;
  Pending p = {m - i < 2 * VL ? m - i : 2 * VL, k - k0 < KC ? k - k0 : KC, a + i * k + k0, panel};
  return p;
}

static inline __attribute__((always_inline)) void sweep(
    int mode, double *c, const double *a, const double *b, double alpha, int64_t m, int64_t n,
    int64_t k, Panel *panels) __arm_streaming __arm_inout("za") {
  int64_t kblocks = (k + KC - 1) / KC;
  int64_t steps = (m + 2 * VL - 1) / (2 * VL) * kblocks;
  Pending now = step(0, a, m, k, panels);
  pack(mode, &now, k, alpha);
  for (int64_t s = 0; s < steps; s++) {
    Pending nx = step(s + 1 < steps ? s + 1 : s, a, m, k, panels + (s + 1) % 2);
    if (s + 1 == steps) nx.panel = 0;
    int64_t i = s / kblocks * 2 * VL, k0 = s % kblocks * KC;
    block(mode, now.rows, n, now.kc, n, n, c + i * n, panels + s % 2, b + k0 * n, &nx, k, alpha);
    now = nx;
  }
}

__arm_new("za") __arm_locally_streaming
static void rows(double *c, const double *a, const double *b, double alpha,
                 int64_t m, int64_t n, int64_t k) {
  Panel panels[2];
  if (alpha == 1.0)
    sweep(UNIT, c, a, b, alpha, m, n, k, panels);
  else if (alpha == -1.0)
    sweep(NEGATED, c, a, b, alpha, m, n, k, panels);
  else
    sweep(SCALED, c, a, b, alpha, m, n, k, panels);
}

typedef struct {
  double *c;
  const double *a, *b;
  double alpha;
  int64_t m, n, k;
} Job;

extern void roop_parallel_for(int64_t lo, int64_t count, int64_t step,
                              void (*body)(void *, int64_t), void *env);

static void chunk(void *env, int64_t idx) {
  const Job *j = env;
  int64_t first = idx * CHUNK;
  int64_t m = j->m - first < CHUNK ? j->m - first : CHUNK;
  rows(j->c + first * j->n, j->a + first * j->k, j->b, j->alpha, m, j->n, j->k);
}

void roop_dgemm(double *c, const double *a, const double *b, double alpha,
                    int64_t m, int64_t n, int64_t k) {
  if (m * n * k < THREADED_WORK || m <= CHUNK) {
    rows(c, a, b, alpha, m, n, k);
    return;
  }
  Job job = {c, a, b, alpha, m, n, k};
  roop_parallel_for(0, (m + CHUNK - 1) / CHUNK, 1, chunk, &job);
}
