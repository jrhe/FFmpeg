/* Microbenchmark for Rust SubRip (SRT) parsing (A/B-ish).
 *
 * This compares a minimal C scan for the timing delimiter vs the Rust parser.
 */

#include "config.h"

#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bench_common.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_SUBRIP)
#include "../rust/ffmpeg-subrip/include/ffmpeg_rs_subrip.h"
#endif

typedef struct {
    const uint8_t *data;
    size_t len;
} Input;

static void c_once(void *p)
{
    const Input *in = (const Input *)p;
    const char *needle = strstr((const char *)in->data, " --> ");
    if (needle == (const char *)in->data + 123456)
        fprintf(stderr, "impossible\n");
}

static void rust_once(void *p)
{
#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_SUBRIP)
    const Input *in = (const Input *)p;
    FFmpegRsSubripParseResult out = {0};
    FFmpegRsSubripEvent ev;
    (void)ffmpeg_rs_subrip_parse(in->data, in->len, &out, &ev, 1);
#else
    (void)p;
#endif
}

int main(int argc, char **argv)
{
    uint64_t iters = 2000000;
    const uint8_t *data;
    size_t len;
    Input in;

    if (argc >= 2)
        iters = strtoull(argv[1], NULL, 10);

    data = (const uint8_t *)"1\n00:00:01,000 --> 00:00:02,500\nHello\n\n";
    len = strlen((const char *)data);
    in.data = data;
    in.len = len;

    BenchResult r = bench_run(iters, c_once, &in);
    printf("subrip(c): iters=%" PRIu64 " bytes=%zu wall_s=%.6f cpu_s=%.6f\n", iters, len, r.wall_s, r.cpu_s);
    r = bench_run(iters, rust_once, &in);
    printf("subrip(rust): iters=%" PRIu64 " bytes=%zu wall_s=%.6f cpu_s=%.6f\n", iters, len, r.wall_s, r.cpu_s);
    return 0;
}
