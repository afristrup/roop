// C += alpha * A * B for row-major f64 matrices, on the SME matrix unit of
// Apple M4 and later. Built by build.rs with clang; the compiler calls it for
// a `dgemm` loop nest (see roop-llvm's gen_gemm).
//
// A block of 16 rows of A is transposed into `ap` through the ZA tiles, then
// swept against 32 columns of B with eight 8x8 FMOPA tiles. A scale of 1 or -1
// costs nothing: the pack copies A as it is and a scale of -1 uses FMOPS, which
// rounds exactly as the fused multiply-add of the negated product. Any other
// alpha is multiplied into the pack.
// Everything stays in streaming mode, since the matrix unit reads and writes
// through L2 and a store from normal mode would have to be flushed to it first.
#include <arm_sme.h>
#include <stdint.h>

#define KC 256
#define VL 8

#define CW (4 * VL)
#define CHUNK 64
#define THREADED_WORK (1 << 26)

enum { SCALED, UNIT, NEGATED };

#define MOPA(mode, tile, a, b)                                  \
  do {                                                          \
    if (mode == NEGATED) svmops_za64_f64_m(tile, all, all, a, b); \
    else svmopa_za64_f64_m(tile, all, all, a, b);               \
  } while (0)

static inline __attribute__((always_inline)) void tile(
    int mode, double *c, int64_t ldc, const double *b, int64_t ldb, int64_t kc,
    const double *ap, svbool_t q0, svbool_t q1, svbool_t q2, svbool_t q3) __arm_streaming __arm_inout("za") {
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
  for (int64_t l = 0; l < kc; l++) {
    svfloat64_t a0 = svld1(all, ap + l * 2 * VL);
    svfloat64_t a1 = svld1(all, ap + l * 2 * VL + VL);
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

static inline __attribute__((always_inline)) void block(
    int mode, int64_t m, int64_t n, int64_t kc, int64_t ldb, int64_t ldc, double *c,
    const double *ap, const double *b) __arm_streaming __arm_inout("za") {
  double edge[2 * VL * CW];
  for (int64_t j = 0; j < n; j += CW) {
    svbool_t q0 = svwhilelt_b64(j, n);
    svbool_t q1 = svwhilelt_b64(j + VL, n);
    svbool_t q2 = svwhilelt_b64(j + 2 * VL, n);
    svbool_t q3 = svwhilelt_b64(j + 3 * VL, n);
    int64_t w = n - j < CW ? n - j : CW;
    if (m == 2 * VL && w == CW) {
      tile(mode, c + j, ldc, b + j, ldb, kc, ap, q0, q1, q2, q3);
      continue;
    }
    for (int64_t r = 0; r < 2 * VL; r++)
      for (int64_t x = 0; x < CW; x++)
        edge[r * CW + x] = r < m && x < w ? c[r * ldc + j + x] : 0.0;
    tile(mode, edge, CW, b + j, ldb, kc, ap, q0, q1, q2, q3);
    for (int64_t r = 0; r < m; r++)
      for (int64_t x = 0; x < w; x++) c[r * ldc + j + x] = edge[r * CW + x];
  }
}

static inline __attribute__((always_inline)) svfloat64x4_t scaled(svfloat64x4_t v, double alpha)
    __arm_streaming {
  svbool_t all = svptrue_b64();
  return svcreate4(svmul_n_f64_x(all, svget4(v, 0), alpha), svmul_n_f64_x(all, svget4(v, 1), alpha),
                   svmul_n_f64_x(all, svget4(v, 2), alpha), svmul_n_f64_x(all, svget4(v, 3), alpha));
}

static inline __attribute__((always_inline)) void pack_group(
    int mode, const double *a, int64_t lda, double alpha, double *ap) __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  for (uint32_t r = 0; r < VL; r++) {
    svld1_hor_za64(0, r, all, a + r * lda);
    svld1_hor_za64(1, r, all, a + (VL + r) * lda);
  }
  for (uint32_t x = 0; x < VL; x += 4) {
    svfloat64x4_t v0 = svread_ver_za64_f64_vg4(0, x);
    svfloat64x4_t v1 = svread_ver_za64_f64_vg4(1, x);
    if (mode == SCALED) {
      v0 = scaled(v0, alpha);
      v1 = scaled(v1, alpha);
    }
    svst1(all, ap + (x + 0) * 2 * VL, svget4(v0, 0));
    svst1(all, ap + (x + 0) * 2 * VL + VL, svget4(v1, 0));
    svst1(all, ap + (x + 1) * 2 * VL, svget4(v0, 1));
    svst1(all, ap + (x + 1) * 2 * VL + VL, svget4(v1, 1));
    svst1(all, ap + (x + 2) * 2 * VL, svget4(v0, 2));
    svst1(all, ap + (x + 2) * 2 * VL + VL, svget4(v1, 2));
    svst1(all, ap + (x + 3) * 2 * VL, svget4(v0, 3));
    svst1(all, ap + (x + 3) * 2 * VL + VL, svget4(v1, 3));
  }
}

static inline __attribute__((always_inline)) void pack_edge(
    int64_t rows, int64_t cols, const double *a, int64_t lda, double alpha, double *ap)
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
    svst1(all, ap + x * 2 * VL, svmul_n_f64_x(all, svread_ver_za64_f64_m(z, all, 0, x), alpha));
    svst1(all, ap + x * 2 * VL + VL, svmul_n_f64_x(all, svread_ver_za64_f64_m(z, all, 1, x), alpha));
  }
}

static inline __attribute__((always_inline)) void pack(
    int mode, int64_t rows, int64_t kc, const double *a, int64_t lda, double alpha, double *ap)
    __arm_streaming __arm_inout("za") {
  double unit = mode == SCALED ? alpha : 1.0;
  for (int64_t l0 = 0; l0 < kc; l0 += VL) {
    if (rows == 2 * VL && l0 + VL <= kc)
      pack_group(mode, a + l0, lda, unit, ap + l0 * 2 * VL);
    else
      pack_edge(rows, kc - l0 < VL ? kc - l0 : VL, a + l0, lda, unit, ap + l0 * 2 * VL);
  }
  svzero_za();
}

static inline __attribute__((always_inline)) void sweep(
    int mode, double *c, const double *a, const double *b, double alpha, int64_t m, int64_t n,
    int64_t k, double *ap) __arm_streaming __arm_inout("za") {
  for (int64_t i = 0; i < m; i += 2 * VL) {
    int64_t r = m - i < 2 * VL ? m - i : 2 * VL;
    for (int64_t k0 = 0; k0 < k; k0 += KC) {
      int64_t kc = k - k0 < KC ? k - k0 : KC;
      pack(mode, r, kc, a + i * k + k0, k, alpha, ap);
      block(mode, r, n, kc, n, n, c + i * n, ap, b + k0 * n);
    }
  }
}

__arm_new("za") __arm_locally_streaming
static void rows(double *c, const double *a, const double *b, double alpha,
                 int64_t m, int64_t n, int64_t k) {
  double ap[KC * 2 * VL];
  if (alpha == 1.0)
    sweep(UNIT, c, a, b, alpha, m, n, k, ap);
  else if (alpha == -1.0)
    sweep(NEGATED, c, a, b, alpha, m, n, k, ap);
  else
    sweep(SCALED, c, a, b, alpha, m, n, k, ap);
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
