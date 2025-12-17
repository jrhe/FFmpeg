#include "bench_common.h"

#include <time.h>

static double ts_to_s(const struct timespec *ts)
{
    return (double)ts->tv_sec + (double)ts->tv_nsec / 1000000000.0;
}

BenchResult bench_run(uint64_t iters, void (*fn)(void *ctx), void *ctx)
{
    struct timespec wall0, wall1;
    struct timespec cpu0, cpu1;
    BenchResult r;

    clock_gettime(CLOCK_MONOTONIC, &wall0);
    clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &cpu0);
    for (uint64_t i = 0; i < iters; i++)
        fn(ctx);
    clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &cpu1);
    clock_gettime(CLOCK_MONOTONIC, &wall1);

    r.wall_s = ts_to_s(&wall1) - ts_to_s(&wall0);
    r.cpu_s  = ts_to_s(&cpu1) - ts_to_s(&cpu0);
    return r;
}

