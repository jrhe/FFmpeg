/*
 * Fuzzer target for the HLS demuxer event parser.
 *
 * This exercises the Rust-backed tokenizer/event parser when configured with:
 *   ./configure --enable-rust-hlsdemux-parser ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_HLSDEMUX_PARSER)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-hlsparser/include/ffmpeg_rs_hlsdemux.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsHlsDemuxParseEventsResult out;
    FFmpegRsHlsDemuxEvent events[512];
    (void)ffmpeg_rs_hls_demux_parse_events(data, size, &out, events, 512);
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

