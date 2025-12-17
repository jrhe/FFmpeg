/*
 * Benchmarks for the HLS playlist writer helpers.
 *
 * This is intentionally simple: it benchmarks the Rust-backed formatting used
 * by ff_hls_write_playlist_version().
 */

#include "config.h"

#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <string.h>

#include "bench_common.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_HLSWRITER)
#include "../rust/ffmpeg-hlswriter/include/ffmpeg_rs_hlswriter.h"
#endif

typedef struct {
    int version;
} Ctx;

static void c_write_once(void *p)
{
    const Ctx *ctx = (const Ctx *)p;
    char buf[64];
    // Match the output shape of ff_hls_write_playlist_version.
    (void)snprintf(buf, sizeof(buf), "#EXTM3U\n#EXT-X-VERSION:%d\n", ctx->version);
}

static void rust_write_once(void *p)
{
    const Ctx *ctx = (const Ctx *)p;
#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_HLSWRITER)
    char buf[64];
    (void)ffmpeg_rs_hls_write_playlist_version(buf, sizeof(buf), ctx->version);
#else
    (void)ctx;
#endif
}

int main(int argc, char **argv)
{
    uint64_t iters = 2000000;
    int version = 7;
    Ctx ctx;
    BenchResult r;

    if (argc >= 2)
        iters = strtoull(argv[1], NULL, 10);
    if (argc >= 3)
        version = atoi(argv[2]);

    ctx.version = version;

    r = bench_run(iters, c_write_once, &ctx);
    printf("hlswriter(c): iters=%" PRIu64 " version=%d wall_s=%.6f cpu_s=%.6f\n",
           iters, version, r.wall_s, r.cpu_s);

    r = bench_run(iters, rust_write_once, &ctx);
    printf("hlswriter(rust): iters=%" PRIu64 " version=%d wall_s=%.6f cpu_s=%.6f\n",
           iters, version, r.wall_s, r.cpu_s);
    return 0;
}
