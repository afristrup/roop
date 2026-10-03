// y += alpha * x for f64 vectors, on the SME matrix unit of Apple M4 and
// later. Built by build.rs with clang; the compiler calls it for a daxpy loop
// (see roop-llvm's gen_axpy).
//
// Streaming loads and stores move four vectors at a time, and the update runs
// as a fused multiply-add into the ZA array, which keeps the sixteen vectors in
// flight that it takes to stream from L2 at full speed.
#include <arm_sme.h>
#include <stdint.h>

#define BLOCK 16
#define CHUNK (1 << 16)
#define THREADED_LEN (1 << 19)

extern void roop_parallel_for(int64_t lo, int64_t count, int64_t step,
                              void (*body)(void *, int64_t), void *env);

__arm_new("za") __arm_locally_streaming
static void run(double *y, const double *x, double alpha, int64_t n) {
  svcount_t all = svptrue_c64();
  svfloat64_t a = svdup_f64(alpha);
  int64_t vl = svcntd();
  int64_t i = 0;
  for (; i + BLOCK * vl <= n; i += BLOCK * vl) {
    svwrite_za64_f64_vg1x4(0, svld1_f64_x4(all, y + i));
    svwrite_za64_f64_vg1x4(4, svld1_f64_x4(all, y + i + 4 * vl));
    svwrite_za64_f64_vg1x4(8, svld1_f64_x4(all, y + i + 8 * vl));
    svwrite_za64_f64_vg1x4(12, svld1_f64_x4(all, y + i + 12 * vl));
    svmla_single_za64_f64_vg1x4(0, svld1_f64_x4(all, x + i), a);
    svmla_single_za64_f64_vg1x4(4, svld1_f64_x4(all, x + i + 4 * vl), a);
    svmla_single_za64_f64_vg1x4(8, svld1_f64_x4(all, x + i + 8 * vl), a);
    svmla_single_za64_f64_vg1x4(12, svld1_f64_x4(all, x + i + 12 * vl), a);
    svst1_f64_x4(all, y + i, svread_za64_f64_vg1x4(0));
    svst1_f64_x4(all, y + i + 4 * vl, svread_za64_f64_vg1x4(4));
    svst1_f64_x4(all, y + i + 8 * vl, svread_za64_f64_vg1x4(8));
    svst1_f64_x4(all, y + i + 12 * vl, svread_za64_f64_vg1x4(12));
  }
  for (; i < n; i += vl) {
    svbool_t p = svwhilelt_b64(i, n);
    svst1(p, y + i, svmla_n_f64_x(p, svld1(p, y + i), svld1(p, x + i), alpha));
  }
}

typedef struct {
  double *y;
  const double *x;
  double alpha;
  int64_t n;
} Job;

static void chunk(void *env, int64_t idx) {
  const Job *j = env;
  int64_t first = idx * CHUNK;
  int64_t len = j->n - first < CHUNK ? j->n - first : CHUNK;
  run(j->y + first, j->x + first, j->alpha, len);
}

void roop_daxpy(double *y, const double *x, double alpha, int64_t n) {
  if (n < THREADED_LEN) {
    run(y, x, alpha, n);
    return;
  }
  Job job = {y, x, alpha, n};
  roop_parallel_for(0, (n + CHUNK - 1) / CHUNK, 1, chunk, &job);
}
