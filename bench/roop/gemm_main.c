#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define N @N@
void gemm(double *c, double *a, double *b, double *alpha);

static double now(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return t.tv_sec + t.tv_nsec * 1e-9;
}

static int cmp(const void *x, const void *y) {
    double a = *(const double *)x, b = *(const double *)y;
    return (a > b) - (a < b);
}

int main(void) {
    static double a[N * N], b[N * N], c[N * N];
    double alpha = 1.0;
    for (long i = 0; i < (long)N * N; i++) {
        a[i] = (double)(i % 7) * 0.25;
        b[i] = (double)(i % 5) * 0.5;
    }
    int reps = @REPS@;
    static double times[4096];
    gemm(c, a, b, &alpha);
    for (int r = 0; r < reps; r++) {
        double t0 = now();
        gemm(c, a, b, &alpha);
        times[r] = now() - t0;
    }
    qsort(times, reps, sizeof(double), cmp);
    printf("%.9f\n", times[0]);
    return c[0] == -1.0;
}
