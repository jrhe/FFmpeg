#pragma once

#include <stdint.h>

typedef struct BenchResult {
    double wall_s;
    double cpu_s;
} BenchResult;

BenchResult bench_run(uint64_t iters, void (*fn)(void *ctx), void *ctx);

