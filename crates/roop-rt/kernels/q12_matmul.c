// C += sum_j trunc(A[i][j] * B[j][k] / 4096) for 64-bit integers holding Q12
// numbers: the loops of the `q12` einsums (qmatmul, qmatmul_nt, qmatmul_tn),
// each product divided by 4096 and truncated toward zero, then summed. Built by
// build.rs with clang; the compiler calls it for those loops (see roop-llvm's
// gen_q12_matmul).
//
// NEON has no 64-bit integer multiply, but a double holds an integer of 53
// bits and a product by a power of two is exact, so when every product fits
// the doubles compute the same integers: (a / 4096) * b is exact, frintz
// truncates it as the division would, and the sums are exact. A product that
// might not fit goes to the plain loops, so the kernel always returns what the
// loops would, to the bit, and `sign` -1 takes off exactly what +1 added.
#include <arm_neon.h>
#include <stdint.h>
#include <stdlib.h>

#define MR 4
#define NR 8
#define SMALL_WORK 2048
#define EXACT_PRODUCT 4503599627370496.0
#define EXACT_SUM 18446744073709551616.0

enum { NN, NT, TN };

static _Thread_local double *packed_a, *packed_b;
static _Thread_local int64_t room_a, room_b;

static double *room(double **buffer, int64_t *have, int64_t need) {
  if (*have < need) {
    free(*buffer);
    *buffer = malloc(need * sizeof(double));
    *have = need;
  }
  return *buffer;
}

static void plain(int64_t *c, const int64_t *a, const int64_t *b, int64_t sign,
                  int64_t m, int64_t n, int64_t kk, int64_t layout) {
  for (int64_t i = 0; i < m; i++)
    for (int64_t k = 0; k < n; k++) {
      int64_t sum = 0;
      for (int64_t j = 0; j < kk; j++) {
        int64_t x = layout == TN ? a[j * m + i] : a[i * kk + j];
        int64_t y = layout == NT ? b[k * kk + j] : b[j * n + k];
        sum += (int64_t)((uint64_t)x * (uint64_t)y) / 4096;
      }
      c[i * n + k] += sign * sum;
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

static void pack_a(double *ap, const int64_t *a, int64_t m, int64_t kk, int64_t layout) {
  const double scale = 1.0 / 4096;
  for (int64_t i = 0; i < (m + MR - 1) / MR * MR; i++) {
    double *out = ap + (i / MR) * kk * MR + i % MR;
    for (int64_t j = 0; j < kk; j++) {
      int64_t x = i >= m ? 0 : layout == TN ? a[j * m + i] : a[i * kk + j];
      out[j * MR] = (double)x * scale;
    }
  }
}

static void pack_b(double *bp, const int64_t *b, int64_t n, int64_t np, int64_t kk, int64_t layout) {
  for (int64_t j = 0; j < kk; j++)
    for (int64_t k = 0; k < np; k++)
      bp[j * np + k] = k >= n ? 0.0 : (double)(layout == NT ? b[k * kk + j] : b[j * n + k]);
}

static void tile(float64x2_t acc[MR][NR / 2], const double *ap, const double *bp, int64_t kk, int64_t np) {
  for (int r = 0; r < MR; r++)
    for (int x = 0; x < NR / 2; x++) acc[r][x] = vdupq_n_f64(0);
  for (int64_t j = 0; j < kk; j++) {
    float64x2_t a01 = vld1q_f64(ap + j * MR), a23 = vld1q_f64(ap + j * MR + 2);
    float64x2_t b0 = vld1q_f64(bp + j * np), b1 = vld1q_f64(bp + j * np + 2);
    float64x2_t b2 = vld1q_f64(bp + j * np + 4), b3 = vld1q_f64(bp + j * np + 6);
#define ROW(r, av, lane)                                                      \
  acc[r][0] = vaddq_f64(acc[r][0], vrndq_f64(vmulq_laneq_f64(b0, av, lane))); \
  acc[r][1] = vaddq_f64(acc[r][1], vrndq_f64(vmulq_laneq_f64(b1, av, lane))); \
  acc[r][2] = vaddq_f64(acc[r][2], vrndq_f64(vmulq_laneq_f64(b2, av, lane))); \
  acc[r][3] = vaddq_f64(acc[r][3], vrndq_f64(vmulq_laneq_f64(b3, av, lane)));
    ROW(0, a01, 0)
    ROW(1, a01, 1)
    ROW(2, a23, 0)
    ROW(3, a23, 1)
  }
}

static void store(int64_t *c, float64x2_t acc[MR][NR / 2], int64_t sign, int64_t i0, int64_t k0,
                  int64_t m, int64_t n) {
  for (int r = 0; r < MR && i0 + r < m; r++)
    for (int x = 0; x < NR / 2; x++) {
      int64x2_t v = vcvtq_s64_f64(acc[r][x]);
      if (sign < 0) v = vnegq_s64(v);
      int64_t *at = c + (i0 + r) * n + k0 + 2 * x;
      if (k0 + 2 * x + 1 < n) {
        vst1q_s64(at, vaddq_s64(vld1q_s64(at), v));
      } else if (k0 + 2 * x < n) {
        *at += vgetq_lane_s64(v, 0);
      }
    }
}

void roop_q12_matmul(int64_t *c, const int64_t *a, const int64_t *b, int64_t sign,
                     int64_t m, int64_t n, int64_t kk, int64_t layout) {
  double product = largest(a, m * kk) * largest(b, kk * n);
  if (m * n * kk < SMALL_WORK || product >= EXACT_PRODUCT || product * (double)kk >= EXACT_SUM) {
    plain(c, a, b, sign, m, n, kk, layout);
    return;
  }
  int64_t mp = (m + MR - 1) / MR * MR, np = (n + NR - 1) / NR * NR;
  double *ap = room(&packed_a, &room_a, mp * kk);
  double *bp = room(&packed_b, &room_b, kk * np);
  pack_a(ap, a, m, kk, layout);
  pack_b(bp, b, n, np, kk, layout);
  for (int64_t i = 0; i < mp; i += MR)
    for (int64_t k = 0; k < np; k += NR) {
      float64x2_t acc[MR][NR / 2];
      tile(acc, ap + i * kk, bp + k, kk, np);
      store(c, acc, sign, i, k, m, n);
    }
}
