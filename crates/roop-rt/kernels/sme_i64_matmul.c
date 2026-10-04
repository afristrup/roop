// C += sign * sum_j A[i][j] * B[j][k] for 64-bit integers: the loops of the `i64`
// einsums (imatmul, imatmul_nt, imatmul_tn), on the SME matrix unit of Apple M4
// and later. Built by build.rs with clang; the compiler calls it for those loops
// (see roop-llvm's gen_int_matmul).
//
// A double holds an integer of 53 bits, so when every product and every sum
// fits (the largest entries multiplied, times the inner size, below 2^52) the
// matrix unit computes these integers exactly. The whole product runs in
// streaming mode: the entries are converted to doubles into operand panels,
// and the largest ones are found on the way, so the check costs no pass of its
// own; then the tiles accumulate in ZA and each is converted back and added to
// C as it leaves. A product that might not fit goes to the plain loops, before
// C is touched, so the kernel always returns what the loops would, to the bit,
// and `sign` -1 takes off exactly what +1 added.
#include <arm_sme.h>
#include <stdint.h>
#include <stdlib.h>

#define KC 256
#define VL 8
#define CW (4 * VL)
#define CHUNK 64
#define BLOCK 512
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

typedef struct {
  int64_t *c;
  const int64_t *a, *b;
  int64_t sign, m, n, kk, layout;
} Job;

static void plain(const Job *j) {
  for (int64_t i = 0; i < j->m; i++)
    for (int64_t k = 0; k < j->n; k++) {
      uint64_t sum = 0;
      for (int64_t l = 0; l < j->kk; l++) {
        int64_t x = j->layout == TN ? j->a[l * j->m + i] : j->a[i * j->kk + l];
        int64_t y = j->layout == NT ? j->b[k * j->kk + l] : j->b[l * j->n + k];
        sum += (uint64_t)x * (uint64_t)y;
      }
      int64_t *at = j->c + i * j->n + k;
      *at = (int64_t)((uint64_t)*at + (uint64_t)j->sign * sum);
    }
}

// The extremes of the integers a pass has seen go into `top` and `bottom`.
#define TRACK(top, bottom, x)                     \
  {                                               \
    top = svmax_s64_x(svptrue_b64(), top, x);     \
    bottom = svmin_s64_x(svptrue_b64(), bottom, x); \
  }

// The largest magnitude among them, as a double.
static inline __attribute__((always_inline)) double magnitude(svint64_t top, svint64_t bottom)
    __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  uint64_t up = (uint64_t)svmaxv_s64(all, top);
  uint64_t down = -(uint64_t)svminv_s64(all, bottom);
  return (double)(up > down ? up : down);
}

static inline __attribute__((always_inline)) void convert(
    double *dst, const int64_t *src, int64_t len, svint64_t *top, svint64_t *bottom)
    __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  svint64_t t = *top, u = *bottom;
  for (int64_t i = 0; i < len; i += VL) {
    svbool_t p = svwhilelt_b64(i, len);
    svint64_t x = svld1_s64(p, src + i);
    TRACK(t, u, x)
    svst1_f64(p, dst + i, svcvt_f64_s64_x(p, x));
  }
  *top = t;
  *bottom = u;
}

// dst[c * ldd + r] = (double)src[r * lds + c] for r < rows and c < cols, with
// `lanes` entries (at least `rows`) stored per column and the others zero.
static inline __attribute__((always_inline)) void transposed(
    double *dst, int64_t ldd, const int64_t *src, int64_t lds, int64_t rows, int64_t lanes,
    int64_t cols, svint64_t *top, svint64_t *bottom) __arm_streaming __arm_inout("za") {
  svbool_t all = svptrue_b64();
  svbool_t out = svwhilelt_b64((int64_t)0, lanes);
  svint64_t zero = svdup_s64(0), t = *top, u = *bottom;
  for (int64_t c0 = 0; c0 < cols; c0 += VL) {
    svbool_t pc = svwhilelt_b64(c0, cols);
    svzero_mask_za(1);
    for (int64_t r = 0; r < rows; r++) svld1_hor_za64(0, (uint32_t)r, pc, src + r * lds + c0);
    int64_t width = cols - c0 < VL ? cols - c0 : VL;
    for (int64_t x = 0; x < width; x++) {
      svint64_t v = svread_ver_za64_s64_m(zero, all, 0, (uint32_t)x);
      TRACK(t, u, v)
      svst1_f64(out, dst + (c0 + x) * ldd, svcvt_f64_s64_x(all, v));
    }
  }
  *top = t;
  *bottom = u;
}

// The panel of rows [i, i + rows) of A for the whole inner size: for each j the
// 16 entries of its column, zero past `rows`.
static inline __attribute__((always_inline)) void pack_a(
    double *ap, const int64_t *a, int64_t layout, int64_t i, int64_t rows, int64_t m, int64_t kk,
    svint64_t *top, svint64_t *bottom) __arm_streaming __arm_inout("za") {
  if (layout == TN) {
    svbool_t all = svptrue_b64();
    svbool_t p0 = svwhilelt_b64((int64_t)0, rows), p1 = svwhilelt_b64((int64_t)VL, rows);
    svint64_t t = *top, u = *bottom;
    for (int64_t l = 0; l < kk; l++) {
      const int64_t *at = a + l * m + i;
      svint64_t x0 = svld1_s64(p0, at), x1 = svld1_s64(p1, at + VL);
      TRACK(t, u, x0)
      TRACK(t, u, x1)
      svst1_f64(all, ap + l * 2 * VL, svcvt_f64_s64_x(all, x0));
      svst1_f64(all, ap + l * 2 * VL + VL, svcvt_f64_s64_x(all, x1));
    }
    *top = t;
    *bottom = u;
    return;
  }
  const int64_t *from = a + i * kk;
  transposed(ap, 2 * VL, from, kk, rows < VL ? rows : VL, VL, kk, top, bottom);
  transposed(ap + VL, 2 * VL, from + VL * kk, kk, rows > VL ? rows - VL : 0, VL, kk, top, bottom);
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

static int64_t blocks(int64_t rows) { return (rows + 2 * VL - 1) / (2 * VL); }

static int64_t need(const Job *j, int64_t rows) {
  return j->kk * j->n + blocks(rows) * 2 * VL * j->kk;
}

__arm_new("za") __arm_locally_streaming
static int fits(const Job *j) {
  svint64_t zero = svdup_s64(0);
  svint64_t at = zero, ab = zero, bt = zero, bb = zero;
  svbool_t all = svptrue_b64();
  for (int64_t i = 0; i < j->m * j->kk; i += VL) {
    svint64_t x = svld1_s64(svwhilelt_b64(i, j->m * j->kk), j->a + i);
    TRACK(at, ab, x)
  }
  for (int64_t i = 0; i < j->kk * j->n; i += VL) {
    svint64_t x = svld1_s64(svwhilelt_b64(i, j->kk * j->n), j->b + i);
    TRACK(bt, bb, x)
  }
  (void)all;
  return magnitude(at, ab) * magnitude(bt, bb) * (double)j->kk < EXACT_SUM;
}

// Rows [lo, hi) of the product. Returns 0, having changed nothing, when
// `check` is set and the product might not be exact.
__arm_new("za") __arm_locally_streaming
static int rows(const Job *j, double *bd, int64_t lo, int64_t hi, int check) {
  int64_t n = j->n, kk = j->kk;
  double *panels = bd + kk * n;
  svint64_t zero = svdup_s64(0);
  svint64_t at = zero, ab = zero, bt = zero, bb = zero;
  if (j->layout == NT) {
    for (int64_t r0 = 0; r0 < n; r0 += VL) {
      int64_t r = n - r0 < VL ? n - r0 : VL;
      transposed(bd + r0, n, j->b + r0 * kk, kk, r, r, kk, &bt, &bb);
    }
  } else {
    convert(bd, j->b, kk * n, &bt, &bb);
  }
  for (int64_t i = lo; i < hi; i += 2 * VL) {
    int64_t r = hi - i < 2 * VL ? hi - i : 2 * VL;
    pack_a(panels + (i - lo) * kk, j->a, j->layout, i, r, j->m, kk, &at, &ab);
  }
  if (check && !(magnitude(at, ab) * magnitude(bt, bb) * (double)kk < EXACT_SUM)) return 0;
  for (int64_t i = lo; i < hi; i += 2 * VL) {
    int64_t r = hi - i < 2 * VL ? hi - i : 2 * VL;
    for (int64_t k0 = 0; k0 < kk; k0 += KC) {
      int64_t kc = kk - k0 < KC ? kk - k0 : KC;
      for (int64_t x = 0; x < n; x += CW) {
        svbool_t q0 = svwhilelt_b64(x, n), q1 = svwhilelt_b64(x + VL, n);
        svbool_t q2 = svwhilelt_b64(x + 2 * VL, n), q3 = svwhilelt_b64(x + 3 * VL, n);
        tile(j->c + i * n + x, n, j->sign, bd + k0 * n + x, n, kc,
             panels + (i - lo) * kk + k0 * 2 * VL, r, q0, q1, q2, q3);
      }
    }
  }
  return 1;
}

static void chunk(void *env, int64_t idx) {
  const Job *j = env;
  int64_t lo = idx * CHUNK;
  int64_t hi = j->m - lo < CHUNK ? j->m : lo + CHUNK;
  rows(j, take(need(j, hi - lo)), lo, hi, 0);
}

void roop_i64_matmul(int64_t *c, const int64_t *a, const int64_t *b, int64_t sign,
                     int64_t m, int64_t n, int64_t kk, int64_t layout) {
  Job job = {c, a, b, sign, m, n, kk, layout};
  int64_t work = m * n * kk;
  if (work < SMALL_WORK) {
    plain(&job);
  } else if (m <= BLOCK) {
    if (!rows(&job, take(need(&job, m)), 0, m, 1)) plain(&job);
  } else if (!fits(&job)) {
    plain(&job);
  } else if (work < THREADED_WORK) {
    double *space = take(need(&job, BLOCK));
    for (int64_t lo = 0; lo < m; lo += BLOCK) rows(&job, space, lo, m - lo < BLOCK ? m : lo + BLOCK, 0);
  } else {
    roop_parallel_for(0, (m + CHUNK - 1) / CHUNK, 1, chunk, &job);
  }
}
