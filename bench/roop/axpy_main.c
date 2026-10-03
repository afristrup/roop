#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define N @N@
#define INNER (N < 100000 ? 400000 / N : 1)
void axpy(double *y, double *x, double *alpha);

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
    /* y sits 64 bytes past a whole number of pages from x: with the two a multiple of
       4096 bytes apart, loads of x and stores to y alias in the L1 and slow the loop */
    static double buf[2 * N + 8];
    double *x = buf, *y = buf + N + 8;
    double alpha = 1.0;
    for (long i = 0; i < (long)N; i++) {
        x[i] = (double)(i % 7) * 0.25;
        y[i] = 1.0;
    }
    int reps = @REPS@;
    static double times[4096];
    for (double t = now(); now() - t < 0.1;) axpy(y, x, &alpha);
    for (int r = 0; r < reps; r++) {
        double t0 = now();
        for (int k = 0; k < INNER; k++) axpy(y, x, &alpha);
        times[r] = (now() - t0) / INNER;
    }
    qsort(times, reps, sizeof(double), cmp);
    printf("%.9f\n", times[0]);
    return y[0] == -1.0;
}
