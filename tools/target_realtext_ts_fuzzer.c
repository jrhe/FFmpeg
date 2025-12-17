/*
 * Fuzzer target for the RealText timestamp helper.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-realtext ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_REALTEXT)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-realtext/include/ffmpeg_rs_realtext.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    int64_t out = 0;
    (void)ffmpeg_rs_realtext_read_ts(data, size, &out);
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

