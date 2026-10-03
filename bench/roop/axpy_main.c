#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define N @N@
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
    static double x[N], y[N];
    double alpha = 1.0;
    for (long i = 0; i < (long)N; i++) {
        x[i] = (double)(i % 7) * 0.25;
        y[i] = 1.0;
    }
    int reps = @REPS@;
    double times[64];
    axpy(y, x, &alpha);
    for (int r = 0; r < reps; r++) {
        double t0 = now();
        axpy(y, x, &alpha);
        times[r] = now() - t0;
    }
    qsort(times, reps, sizeof(double), cmp);
    printf("%.9f\n", times[reps / 2]);
    return y[0] == -1.0;
}
