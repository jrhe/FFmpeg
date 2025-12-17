/* Microbenchmark for Rust WebVTT cue parsing (A/B-ish).
 *
 * This compares a minimal C timestamp scan vs the Rust parser for a cue-sized
 * chunk.
 */

#include "config.h"

#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bench_common.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_WEBVTT)
#include "../rust/ffmpeg-webvtt/include/ffmpeg_rs_webvtt.h"
#endif

typedef struct {
    const uint8_t *data;
    size_t len;
} Input;

static void c_once(void *p)
{
    const Input *in = (const Input *)p;
    // Minimal scan for "-->" to simulate baseline work.
    const char *needle = strstr((const char *)in->data, "-->");
    if (needle == (const char *)in->data + 123456)
        fprintf(stderr, "impossible\n");
}

static void rust_once(void *p)
{
#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_WEBVTT)
    const Input *in = (const Input *)p;
    FFmpegRsWebvttParseResult out = {0};
    FFmpegRsWebvttCue cue;
    (void)ffmpeg_rs_webvtt_parse(in->data, in->len, &out, &cue, 1);
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

    data = (const uint8_t *)"WEBVTT\n\n00:00.000 --> 00:01.000\nHello\n\n";
    len = strlen((const char *)data);
    in.data = data;
    in.len = len;

    BenchResult r = bench_run(iters, c_once, &in);
    printf("webvtt(c): iters=%" PRIu64 " bytes=%zu wall_s=%.6f cpu_s=%.6f\n", iters, len, r.wall_s, r.cpu_s);
    r = bench_run(iters, rust_once, &in);
    printf("webvtt(rust): iters=%" PRIu64 " bytes=%zu wall_s=%.6f cpu_s=%.6f\n", iters, len, r.wall_s, r.cpu_s);
    return 0;
}
