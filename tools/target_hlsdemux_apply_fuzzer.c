/*
 * Fuzzer target for the HLS demuxer apply layer (Rust strict parse).
 *
 * This exercises the Rust-backed apply helper when configured with:
 *   ./configure --enable-rust-hlsdemux-apply ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_HLSDEMUX_APPLY)

#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "../rust/ffmpeg-hlsparser/include/ffmpeg_rs_hlsparser.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    static const uint8_t prefix[] = "#EXTM3U\n";
    uint8_t buf[4096];
    size_t n;
    FFmpegRsHlsPlaylist pl;
    int r;

    if (!data || !size)
        return 0;

    memcpy(buf, prefix, sizeof(prefix) - 1);
    n = size < (sizeof(buf) - sizeof(prefix)) ? size : (sizeof(buf) - sizeof(prefix));
    memcpy(buf + (sizeof(prefix) - 1), data, n);
    buf[(sizeof(prefix) - 1) + n] = 0;

    memset(&pl, 0, sizeof(pl));
    r = ffmpeg_rs_hls_parse_strict(buf, (sizeof(prefix) - 1) + n, &pl, NULL, 0, NULL, 0);
    if (r == 0 && pl.n_segments <= 64 && pl.n_variants <= 64) {
        FFmpegRsHlsSegment segs[64];
        FFmpegRsHlsVariant vars[64];
        memset(segs, 0, sizeof(segs));
        memset(vars, 0, sizeof(vars));
        (void)ffmpeg_rs_hls_parse_strict(buf, (sizeof(prefix) - 1) + n, &pl, segs, 64, vars, 64);
    }
    return 0;
}

#else

#include <stddef.h>
#include <stdint.h>

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    (void)data;
    (void)size;
    return 0;
}

#endif

