// C += sign * sum_j A[i][j] * B[j][k] for 64-bit integers: the loops of the `i64`
// einsums (imatmul, imatmul_nt, imatmul_tn), on the SME matrix unit of Apple M4
// and later. Built by build.rs with clang; the compiler calls it for those loops
// (see roop-llvm's gen_int_matmul).
//
// A double holds an integer of 53 bits, so when every product and every sum
// fits (the largest entries multiplied, times the inner size, below 2^52) the
// matrix unit computes these integers exactly. The whole product runs in
// streaming mode: the entries are converted to doubles on the way into the
// operand panels, the tiles accumulate in ZA, and each tile is converted back
// and added to C as it leaves. A product that might not fit goes to the plain
// loops, so the kernel always returns what the loops would, to the bit, and
// `sign` -1 takes off exactly what +1 added.
#include <arm_sme.h>
#include <stdint.h>
#include <stdlib.h>

#define KC 256
#define VL 8
#define CW (4 * VL)
#define CHUNK 64
#define SMALL_WORK 4096
#define THREADED_WORK (1 << 26)
#define EXACT_SUM 4503599627370496.0

enum { NN, NT, TN };

extern void roop_parallel_for(int64_t lo, int64_t count, int64_t step,
                              void (*body)(void *, int64_t), void *env);

static _Thread_local double *scratch;
static _Thread_local int64_t room;

static double *take(int64_t need) {
  if (room < need) {
    free(scratch);
    scratch = malloc(need * sizeof(double));
    room = need;
  }
  return scratch;
}

static void plain(int64_t *c, const int64_t *a, const int64_t *b, int64_t sign,
                  int64_t m, int64_t n, int64_t kk, int64_t layout) {
  for (int64_t i = 0; i < m; i++)
    for (int64_t k = 0; k < n; k++) {
      uint64_t sum = 0;
      for (int64_t j = 0; j < kk; j++) {
        int64_t x = layout == TN ? a[j * m + i] : a[i * kk + j];
        int64_t y = layout == NT ? b[k * kk + j] : b[j * n + k];
        sum += (uint64_t)x * (uint64_t)y;
      }
      c[i * n + k] = (int64_t)((uint64_t)c[i * n + k] + (uint64_t)sign * sum);
    }
}

static inline __attribute__((always_inline)) double absmax(const int64_t *v, int64_t len)
    __arm_streaming {
  svbool_t all = svptrue_b64();
  svuint64_t t0 = svdup_u64(0), t1 = t0, t2 = t0, t3 = t0;
  int64_t i = 0;
  for (; i + 4 * VL <= len; i += 4 * VL) {
    t0 = svmax_u64_x(all, t0, svreinterpret_u64_s64(svabs_s64_x(all, svld1_s64(all, v + i))));
    t1 = svmax_u64_x(all, t1, svreinterpret_u64_s64(svabs_s64_x(all, svld1_s64(all, v + i + VL))));
    t2 = svmax_u64_x(all, t2, svreinterpret_u64_s64(svabs_s64_x(all, svld1_s64(all, v + i + 2 * VL))));
    t3 = svmax_u64_x(all, t3, svreinterpret_u64_s64(svabs_s64_x(all, svld1_s64(all, v + i + 3 * VL))));
  }
  for (; i < len; i += VL) {
    svbool_t p = svwhilelt_b64(i, len);
    t0 = svmax_u64_m(p, t0, svreinterpret_u64_s64(svabs_s64_x(p, svld1_s64(p, v + i))));
  }
  t0 = svmax_u64_x(all, svmax_u64_x(all, t0, t1), svmax_u64_x(all, t2, t3));
  return (double)svmaxv_u64(all, t0);
}

static inline __attribute__((always_inline)) void convert(double *dst, const int64_t *src, int64_t len)
    __arm_streaming {
  svbool_t all = svptrue_b64();
  int64_t i = 0;
  for (; i + 4 * VL <= len; i += 4 * VL) {
    svst1_f64(all, dst + i, svcvt_f64_s64_x(all, svld1_s64(all, src + i)));
    svst1_f64(all, dst + i + VL, svcvt_f64_s64_x(all, svld1_s64(all, src + i + VL)));
    svst1_f64(all, dst + i + 2 * VL, svcvt_f64_s64_x(all, svld1_s64(all, src + i + 2 * VL)));
    svst1_f64(all, dst + i + 3 * VL, svcvt_f64_s64_x(all, svld1_s64(all, src + i + 3 * VL)));
  }
  for (; i < len; i += VL) {
    svbool_t p = svwhilelt_b64(i, len);
    svst1_f64(p, dst + i, svcvt_f64_s64_x(p, svld1_s64(p, src + i)));
  }
}

// dst[c * ldd + r] = (double)src[r * lds + c] for r < rows and c < cols, with
// `lanes` entries (at least `rows`) stored per column and the others zero.
static inline __attribute__((always_inline)) void transposed(
    double *dst, int64_t ldd, const int64_t *src, int64_t lds, int64_t rows, int64_t lanes,
    int64_t cols) __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  svbool_t out = svwhilelt_b64((int64_t)0, lanes);
  svint64_t zero = svdup_s64(0);
  for (int64_t c0 = 0; c0 < cols; c0 += VL) {
    svbool_t pc = svwhilelt_b64(c0, cols);
    svzero_mask_za(1);
    for (int64_t r = 0; r < rows; r++) svld1_hor_za64(0, (uint32_t)r, pc, src + r * lds + c0);
    int64_t width = cols - c0 < VL ? cols - c0 : VL;
    for (int64_t x = 0; x < width; x++) {
      svint64_t v = svread_ver_za64_s64_m(zero, all, 0, (uint32_t)x);
      svst1_f64(out, dst + (c0 + x) * ldd, svcvt_f64_s64_x(all, v));
    }
  }
}

static inline __attribute__((always_inline)) void pack_a(
    double *ap, const int64_t *a, int64_t layout, int64_t i, int64_t rows, int64_t k0, int64_t kc,
    int64_t m, int64_t kk) __arm_streaming __arm_inout("za") {
  if (layout == TN) {
    svbool_t all = svptrue_b64();
    svbool_t p0 = svwhilelt_b64((int64_t)0, rows), p1 = svwhilelt_b64((int64_t)VL, rows);
    for (int64_t l = 0; l < kc; l++) {
      const int64_t *at = a + (k0 + l) * m + i;
      svst1_f64(all, ap + l * 2 * VL, svcvt_f64_s64_x(all, svld1_s64(p0, at)));
      svst1_f64(all, ap + l * 2 * VL + VL, svcvt_f64_s64_x(all, svld1_s64(p1, at + VL)));
    }
    return;
  }
  const int64_t *from = a + i * kk + k0;
  transposed(ap, 2 * VL, from, kk, rows < VL ? rows : VL, VL, kc);
  transposed(ap + VL, 2 * VL, from + VL * kk, kk, rows > VL ? rows - VL : 0, VL, kc);
}

#define OUT(T, s, r, q, off)                                                     \
  {                                                                            \
    svint64_t d = svmul_n_s64_x(q, svcvt_s64_f64_x(q, svread_hor_za64_f64_m(zf, all, T, s)), sign); \
    int64_t *at = c + (r) * ldc + (off);                                       \
    svst1_s64(q, at, svadd_s64_x(q, svld1_s64(q, at), d));                     \
  }

static inline __attribute__((always_inline)) void tile(
    int64_t *c, int64_t ldc, int64_t sign, const double *b, int64_t ldb, int64_t kc,
    const double *ap, int64_t rows, svbool_t q0, svbool_t q1, svbool_t q2, svbool_t q3)
    __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  svzero_za();
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
  svfloat64_t zf = svdup_f64(0.0);
  for (int64_t r = 0; r < rows && r < VL; r++) {
    OUT(0, (uint32_t)r, r, q0, 0)
    OUT(1, (uint32_t)r, r, q1, VL)
    OUT(2, (uint32_t)r, r, q2, 2 * VL)
    OUT(3, (uint32_t)r, r, q3, 3 * VL)
  }
  for (int64_t r = VL; r < rows; r++) {
    OUT(4, (uint32_t)(r - VL), r, q0, 0)
    OUT(5, (uint32_t)(r - VL), r, q1, VL)
    OUT(6, (uint32_t)(r - VL), r, q2, 2 * VL)
    OUT(7, (uint32_t)(r - VL), r, q3, 3 * VL)
  }
}

typedef struct {
  int64_t *c;
  const int64_t *a, *b;
  int64_t sign, m, n, kk, layout;
} Job;

// Rows [lo, hi) of the product, with the right operand converted into `bd`.
// Returns 0, having changed nothing, when the product might not be exact.
__arm_new("za") __arm_locally_streaming
static int rows(const Job *j, double *bd, int64_t lo, int64_t hi, int check) {
  double ap[KC * 2 * VL];
  int64_t n = j->n, kk = j->kk;
  if (check && !(absmax(j->a, j->m * kk) * absmax(j->b, kk * n) * (double)kk < EXACT_SUM)) return 0;
  if (j->layout == NT) {
    for (int64_t r0 = 0; r0 < n; r0 += VL) {
      int64_t r = n - r0 < VL ? n - r0 : VL;
      transposed(bd + r0, n, j->b + r0 * kk, kk, r, r, kk);
    }
  } else {
    convert(bd, j->b, kk * n);
  }
  for (int64_t i = lo; i < hi; i += 2 * VL) {
    int64_t r = hi - i < 2 * VL ? hi - i : 2 * VL;
    for (int64_t k0 = 0; k0 < kk; k0 += KC) {
      int64_t kc = kk - k0 < KC ? kk - k0 : KC;
      pack_a(ap, j->a, j->layout, i, r, k0, kc, j->m, kk);
      for (int64_t x = 0; x < n; x += CW) {
        svbool_t q0 = svwhilelt_b64(x, n), q1 = svwhilelt_b64(x + VL, n);
        svbool_t q2 = svwhilelt_b64(x + 2 * VL, n), q3 = svwhilelt_b64(x + 3 * VL, n);
        tile(j->c + i * n + x, n, j->sign, bd + k0 * n + x, n, kc, ap, r, q0, q1, q2, q3);
      }
    }
  }
  return 1;
}

__arm_locally_streaming
static int fits(const Job *j) {
  return absmax(j->a, j->m * j->kk) * absmax(j->b, j->kk * j->n) * (double)j->kk < EXACT_SUM;
}

static void chunk(void *env, int64_t idx) {
  const Job *j = env;
  int64_t lo = idx * CHUNK;
  rows(j, take(j->kk * j->n), lo, j->m - lo < CHUNK ? j->m : lo + CHUNK, 0);
}

void roop_i64_matmul(int64_t *c, const int64_t *a, const int64_t *b, int64_t sign,
                     int64_t m, int64_t n, int64_t kk, int64_t layout) {
  Job job = {c, a, b, sign, m, n, kk, layout};
  int64_t work = m * n * kk;
  int done = 0;
  if (work >= SMALL_WORK && (work < THREADED_WORK || m <= CHUNK)) {
    done = rows(&job, take(kk * n), 0, m, 1);
  } else if (work >= SMALL_WORK && fits(&job)) {
    roop_parallel_for(0, (m + CHUNK - 1) / CHUNK, 1, chunk, &job);
    done = 1;
  }
  if (!done) plain(c, a, b, sign, m, n, kk, layout);
}
