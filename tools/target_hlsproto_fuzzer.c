/*
 * Fuzzer target for the HLS protocol playlist parser.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-hlsparser ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_HLSPARSER)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-hlsparser/include/ffmpeg_rs_hlsparser.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsHlsPlaylist pl;
    FFmpegRsHlsSegment segs[256];
    FFmpegRsHlsVariant vars[64];
    (void)ffmpeg_rs_hls_parse(data, size, &pl, segs, 256, vars, 64);
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
