#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define N @N@
#define INNER (N < 100000 ? 400000 / N : 1)
void axpy(double *y, double *x, double *alpha, long *i);
static double now(void) { return clock_gettime_nsec_np(CLOCK_UPTIME_RAW) * 1e-9; }
static int cmp(const void *a, const void *b) { double x = *(const double *)a, y = *(const double *)b; return (x > y) - (x < y); }
int main(void) {
    /* y sits 64 bytes past a whole number of pages from x: with the two a multiple of
       4096 bytes apart, loads of x and stores to y alias in the L1 and slow the loop */
    static double buf[2 * N + 8] __attribute__((aligned(128)));
    double *x = buf, *y = buf + N + 8;
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
