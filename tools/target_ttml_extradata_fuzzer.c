/*
 * Fuzzer target for the TTML extradata parser.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-ttml ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_TTML)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-ttml/include/ffmpeg_rs_ttml.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsTtmlExtradataParseResult out;
    (void)ffmpeg_rs_ttml_parse_extradata(data, size, &out);
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

