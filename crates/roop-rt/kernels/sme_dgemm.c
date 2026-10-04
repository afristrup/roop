// C += alpha * A * B for row-major f64 matrices, on the SME matrix unit of
// Apple M4 and later. Built by build.rs with clang; the compiler calls it for
// a `dgemm` loop nest (see roop-llvm's gen_gemm).
//
// A block of 16 rows of A is transposed into `ap` through the ZA tiles, scaled
// by alpha, then swept against 32 columns of B with eight 8x8 FMOPA tiles.
// Everything stays in streaming mode, since the matrix unit reads and writes
// through L2 and a store from normal mode would have to be flushed to it first.
#include <arm_sme.h>
#include <stdint.h>

#define KC 256
#define VL 8

#define CW (4 * VL)
#define CHUNK 64
#define THREADED_WORK (1 << 26)

static inline __attribute__((always_inline)) void tile(
    double *c, int64_t ldc, const double *b, int64_t ldb, int64_t kc,
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
    svmopa_za64_f64_m(0, all, all, a0, b0);
    svmopa_za64_f64_m(1, all, all, a0, b1);
    svmopa_za64_f64_m(2, all, all, a0, b2);
    svmopa_za64_f64_m(3, all, all, a0, b3);
    svmopa_za64_f64_m(4, all, all, a1, b0);
    svmopa_za64_f64_m(5, all, all, a1, b1);
    svmopa_za64_f64_m(6, all, all, a1, b2);
    svmopa_za64_f64_m(7, all, all, a1, b3);
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

static void block(int64_t m, int64_t n, int64_t kc, int64_t ldb, int64_t ldc,
                  double *c, const double *ap, const double *b) __arm_streaming __arm_inout("za") {
  double edge[2 * VL * CW];
  for (int64_t j = 0; j < n; j += CW) {
    svbool_t q0 = svwhilelt_b64(j, n);
    svbool_t q1 = svwhilelt_b64(j + VL, n);
    svbool_t q2 = svwhilelt_b64(j + 2 * VL, n);
    svbool_t q3 = svwhilelt_b64(j + 3 * VL, n);
    int64_t w = n - j < CW ? n - j : CW;
    if (m == 2 * VL && w == CW) {
      tile(c + j, ldc, b + j, ldb, kc, ap, q0, q1, q2, q3);
      continue;
    }
    for (int64_t r = 0; r < 2 * VL; r++)
      for (int64_t x = 0; x < CW; x++)
        edge[r * CW + x] = r < m && x < w ? c[r * ldc + j + x] : 0.0;
    tile(edge, CW, b + j, ldb, kc, ap, q0, q1, q2, q3);
    for (int64_t r = 0; r < m; r++)
      for (int64_t x = 0; x < w; x++) c[r * ldc + j + x] = edge[r * CW + x];
  }
}

static void pack(int64_t rows, int64_t kc, const double *a, int64_t lda, double alpha, double *ap)
    __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  svzero_za();
  svfloat64_t z = svdup_f64(0.0);
  for (int64_t l0 = 0; l0 < kc; l0 += VL) {
    svbool_t pl = svwhilelt_b64(l0, kc);
    for (uint32_t r = 0; r < VL; r++) {
      if (r < rows) svld1_hor_za64(0, r, pl, a + r * lda + l0);
      if (VL + r < rows) svld1_hor_za64(1, r, pl, a + (VL + r) * lda + l0);
    }
    int64_t cols = kc - l0 < VL ? kc - l0 : VL;
    for (uint32_t x = 0; x < VL; x++) {
      if (x < cols) {
        svfloat64_t v0 = svread_ver_za64_f64_m(z, all, 0, x);
        svfloat64_t v1 = svread_ver_za64_f64_m(z, all, 1, x);
        svst1(all, ap + (l0 + x) * 2 * VL, svmul_n_f64_x(all, v0, alpha));
        svst1(all, ap + (l0 + x) * 2 * VL + VL, svmul_n_f64_x(all, v1, alpha));
      }
    }
  }
  svzero_za();
}

__arm_new("za") __arm_locally_streaming
static void rows(double *c, const double *a, const double *b, double alpha,
                 int64_t m, int64_t n, int64_t k) {
  double ap[KC * 2 * VL];
  for (int64_t i = 0; i < m; i += 2 * VL) {
    int64_t r = m - i < 2 * VL ? m - i : 2 * VL;
    for (int64_t k0 = 0; k0 < k; k0 += KC) {
      int64_t kc = k - k0 < KC ? k - k0 : KC;
      pack(r, kc, a + i * k + k0, k, alpha, ap);
      block(r, n, kc, n, n, c + i * n, ap, b + k0 * n);
    }
  }
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
