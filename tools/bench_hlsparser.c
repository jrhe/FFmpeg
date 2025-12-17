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

static void c_parse_once(void *ctx)
{
    const Input *in = (const Input *)ctx;
    // Minimal C-equivalent parser for the same subset as the Rust parser,
    // operating on an in-memory buffer.
    int64_t target_duration_us = 0;
    int start_seq_no = 0;
    int finished = 0;
    int n_segments = 0;
    int n_variants = 0;
    int is_segment = 0;
    int is_variant = 0;
    int bandwidth = 0;
    int64_t duration_us = 0;

    const char *text = (const char *)in->data;
    const char *end = text + in->size;
    const char *line = text;
    const char *nl;

    if (in->size < 7 || memcmp(text, "#EXTM3U", 7))
        return;

    while (line < end) {
        nl = memchr(line, '\n', (size_t)(end - line));
        const char *line_end = nl ? nl : end;
        // trim trailing CR
        if (line_end > line && line_end[-1] == '\r')
            line_end--;

        size_t len = (size_t)(line_end - line);
        if (len == 0) {
            line = nl ? nl + 1 : end;
            continue;
        }

        if (len >= 22 && !memcmp(line, "#EXT-X-TARGETDURATION:", 22)) {
            target_duration_us = (int64_t)atoi(line + 22) * 1000000;
        } else if (len >= 22 && !memcmp(line, "#EXT-X-MEDIA-SEQUENCE:", 22)) {
            start_seq_no = atoi(line + 22);
        } else if (len >= 13 && !memcmp(line, "#EXT-X-ENDLIST", 13)) {
            finished = 1;
        } else if (len >= 8 && !memcmp(line, "#EXTINF:", 8)) {
            // parse until comma
            const char *p = line + 8;
            char tmp[64];
            size_t i = 0;
            while (p < line_end && *p != ',' && i + 1 < sizeof(tmp)) {
                tmp[i++] = *p++;
            }
            tmp[i] = 0;
            duration_us = (int64_t)(atof(tmp) * 1000000.0);
            is_segment = 1;
        } else if (len >= 18 && !memcmp(line, "#EXT-X-STREAM-INF:", 18)) {
            const char *p = line + 18;
            const char *bw = strstr(p, "BANDWIDTH=");
            bandwidth = bw ? atoi(bw + 10) : 0;
            is_variant = 1;
        } else if (line[0] == '#') {
            // comment
        } else {
            // uri line
            if (is_segment) {
                n_segments++;
                (void)duration_us;
                is_segment = 0;
            } else if (is_variant) {
                n_variants++;
                (void)bandwidth;
                is_variant = 0;
            }
        }

        line = nl ? nl + 1 : end;
    }

    // prevent optimizing out
    if (target_duration_us == 123456789)
        fprintf(stderr, "impossible: %d %d %d %d\n", start_seq_no, finished, n_segments, n_variants);
}

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

    r = bench_run(iters, c_parse_once, &in);
    printf("hlsparser(c): iters=%" PRIu64 " bytes=%zu wall_s=%.6f cpu_s=%.6f\n",
           iters, size, r.wall_s, r.cpu_s);

    r = bench_run(iters, rust_parse_once, &in);
    printf("hlsparser(rust): iters=%" PRIu64 " bytes=%zu wall_s=%.6f cpu_s=%.6f\n",
           iters, size, r.wall_s, r.cpu_s);

    free(data);
    return 0;
}
