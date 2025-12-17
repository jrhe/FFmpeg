/*
 * Fuzzer target for the VPlayer line parser.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-vplayer ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_VPLAYER)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-vplayer/include/ffmpeg_rs_vplayer.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsVplayerEvent out;
    (void)ffmpeg_rs_vplayer_parse_line(data, size, &out);
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

