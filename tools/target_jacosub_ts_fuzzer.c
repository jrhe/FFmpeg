/*
 * Fuzzer target for the JACOsub timestamp parser helpers.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-jacosub ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_JACOSUB)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-jacosub/include/ffmpeg_rs_jacosub.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    int64_t start_cs = 0, dur_cs = 0;
    int shift = ffmpeg_rs_jacosub_parse_shift(30, data, size);
    (void)ffmpeg_rs_jacosub_read_ts(30, shift, data, size, &start_cs, &dur_cs);
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

