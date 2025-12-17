/*
 * Fuzzer target for the SubRip (SRT) parser.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-subrip ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_SUBRIP)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-subrip/include/ffmpeg_rs_subrip.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsSubripParseResult out;
    FFmpegRsSubripEvent events[64];
    (void)ffmpeg_rs_subrip_parse(data, size, &out, events, 64);
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

