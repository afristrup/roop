#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define N @N@
#define INNER (N < 100000 ? 400000 / N : 1)
void axpy(double *y, double *x, double *alpha, long *i);
static double now(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec + t.tv_nsec * 1e-9; }
static int cmp(const void *a, const void *b) { double x = *(const double *)a, y = *(const double *)b; return (x > y) - (x < y); }
int main(void) {
    static double x[N], y[N];
    double alpha = 1.0; long ii;
    for (long i = 0; i < N; i++) { x[i] = (i % 7) * 0.25; y[i] = 1.0; }
    int reps = @REPS@;
    static double times[2048];
    for (int k = 0; k < INNER; k++) { ii = 0; axpy(y, x, &alpha, &ii); }
    for (int r = 0; r < reps; r++) {
        double t0 = now();
        for (int k = 0; k < INNER; k++) { ii = 0; axpy(y, x, &alpha, &ii); }
        times[r] = (now() - t0) / INNER;
    }
    qsort(times, reps, sizeof(double), cmp);
    printf("%.9f\n", times[0]);
    return y[0] == -1.0;
}
