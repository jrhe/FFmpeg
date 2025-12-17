/*
 * Benchmarks for the HLS playlist parser.
 *
 * Build:
 *   ./configure --enable-rust-hlsparser --disable-programs
 *   make tools/bench_hlsparser
 */

#include "config.h"

#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bench_common.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_HLSPARSER)
#include "../rust/ffmpeg-hlsparser/include/ffmpeg_rs_hlsparser.h"
#endif

typedef struct {
    const uint8_t *data;
    size_t size;
} Input;

static void rust_parse_once(void *ctx)
{
    const Input *in = (const Input *)ctx;
#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_HLSPARSER)
    FFmpegRsHlsPlaylist pl;
    FFmpegRsHlsSegment segs[256];
    FFmpegRsHlsVariant vars[64];
    (void)ffmpeg_rs_hls_parse(in->data, in->size, &pl, segs, 256, vars, 64);
#else
    (void)in;
#endif
}

static uint8_t *read_file(const char *path, size_t *out_size)
{
    FILE *f = fopen(path, "rb");
    uint8_t *buf;
    size_t n;
    long sz;

    if (!f) {
        fprintf(stderr, "failed to open %s: %s\n", path, strerror(errno));
        return NULL;
    }
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return NULL;
    }
    sz = ftell(f);
    if (sz < 0) {
        fclose(f);
        return NULL;
    }
    rewind(f);

    buf = malloc((size_t)sz + 1);
    if (!buf) {
        fclose(f);
        return NULL;
    }
    n = fread(buf, 1, (size_t)sz, f);
    fclose(f);
    buf[n] = 0;
    *out_size = n;
    return buf;
}

int main(int argc, char **argv)
{
    const char *path;
    uint64_t iters = 20000;
    size_t size = 0;
    uint8_t *data;
    Input in;
    BenchResult r;

    if (argc < 2) {
        fprintf(stderr, "usage: %s <playlist.m3u8> [iters]\n", argv[0]);
        return 2;
    }
    path = argv[1];
    if (argc >= 3)
        iters = strtoull(argv[2], NULL, 10);

    data = read_file(path, &size);
    if (!data)
        return 1;

    in.data = data;
    in.size = size;

    r = bench_run(iters, rust_parse_once, &in);
    printf("hlsparser: iters=%" PRIu64 " bytes=%zu wall_s=%.6f cpu_s=%.6f\n",
           iters, size, r.wall_s, r.cpu_s);

    free(data);
    return 0;
}

