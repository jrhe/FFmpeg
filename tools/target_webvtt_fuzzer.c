/*
 * Fuzzer target for the WebVTT cue parser.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-webvtt ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_WEBVTT)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-webvtt/include/ffmpeg_rs_webvtt.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsWebvttParseResult out;
    FFmpegRsWebvttCue cues[64];
    (void)ffmpeg_rs_webvtt_parse(data, size, &out, cues, 64);
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

