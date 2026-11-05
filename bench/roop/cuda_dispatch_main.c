#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define N @N@
#define REPS @REPS@

void add_cpu(int64_t*, int64_t*, int64_t*);
void add_cuda(int64_t*, int64_t*, int64_t*);

typedef void (*Add)(int64_t*, int64_t*, int64_t*);

static double measure(Add add) {
    int64_t *q = malloc(N * sizeof(*q));
    int64_t *p = malloc(N * sizeof(*p));
    if (!q || !p) exit(2);
    int64_t i = 0;
    for (int64_t n = 0; n < N; n++) { q[n] = n * 3 - 7; p[n] = n - 5; }
    add(q, p, &i);
    i = 0;
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    for (int r = 0; r < REPS; r++) { i = 0; add(q, p, &i); }
    clock_gettime(CLOCK_MONOTONIC, &end);
    volatile int64_t observed = p[N / 2];
    (void)observed;
    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    free(q);
    free(p);
    return elapsed;
}

int main(void) {
    printf("%.9f %.9f\n", measure(add_cpu), measure(add_cuda));
    return 0;
}
