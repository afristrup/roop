// C += sum_j A[i][j] * B[j][k] for 64-bit integers: the loops of the `i64`
// einsums (imatmul, imatmul_nt, imatmul_tn). Built by build.rs with clang; the
// compiler calls it for those loops (see roop-llvm's gen_q12_matmul).
//
// A double holds an integer of 53 bits, so when every product and every sum
// fits, the matrix kernel for doubles computes these integers exactly, and it
// is the SME matrix unit. A product or a sum that might not fit goes to the
// plain loops, so the kernel always returns what the loops would, to the bit,
// and `sign` -1 takes off exactly what +1 added.
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

extern void roop_dgemm(double *c, const double *a, const double *b, double alpha,
                       int64_t m, int64_t n, int64_t k);

#define SMALL_WORK 4096
#define EXACT_SUM 4503599627370496.0

enum { NN, NT, TN };

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

static double largest(const int64_t *v, int64_t len) {
  uint64_t top = 0;
  for (int64_t i = 0; i < len; i++) {
    uint64_t x = v[i] < 0 ? -(uint64_t)v[i] : (uint64_t)v[i];
    top = x > top ? x : top;
  }
  return (double)top;
}

void roop_i64_matmul(int64_t *c, const int64_t *a, const int64_t *b, int64_t sign,
                     int64_t m, int64_t n, int64_t kk, int64_t layout) {
  double bound = largest(a, m * kk) * largest(b, kk * n) * (double)kk;
  if (m * n * kk < SMALL_WORK || bound >= EXACT_SUM) {
    plain(c, a, b, sign, m, n, kk, layout);
    return;
  }
  double *ad = take(m * kk + kk * n + m * n);
  double *bd = ad + m * kk, *cd = bd + kk * n;
  if (layout == TN) {
    for (int64_t j = 0; j < kk; j++)
      for (int64_t i = 0; i < m; i++) ad[i * kk + j] = (double)a[j * m + i];
  } else {
    for (int64_t i = 0; i < m * kk; i++) ad[i] = (double)a[i];
  }
  if (layout == NT) {
    for (int64_t k = 0; k < n; k++)
      for (int64_t j = 0; j < kk; j++) bd[j * n + k] = (double)b[k * kk + j];
  } else {
    for (int64_t i = 0; i < kk * n; i++) bd[i] = (double)b[i];
  }
  memset(cd, 0, m * n * sizeof(double));
  roop_dgemm(cd, ad, bd, 1.0, m, n, kk);
  for (int64_t i = 0; i < m * n; i++)
    c[i] = (int64_t)((uint64_t)c[i] + (uint64_t)sign * (uint64_t)(int64_t)cd[i]);
}
