/*
 * Fuzzer target for the AQTitle marker helper.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-aqtitle ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_AQTITLE)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-aqtitle/include/ffmpeg_rs_aqtitle.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    int64_t frame = 0;
    (void)ffmpeg_rs_aqtitle_parse_marker(data, size, &frame);
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

