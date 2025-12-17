/*
 * Fuzzer target for the SAMI Start attribute helper.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-sami ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_SAMI)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-sami/include/ffmpeg_rs_sami.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    int64_t out = 0;
    (void)ffmpeg_rs_sami_parse_start_ms(data, size, &out);
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

