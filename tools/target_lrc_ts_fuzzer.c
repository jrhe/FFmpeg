/*
 * Fuzzer target for the LRC timestamp helpers.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-lrc ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_LRC)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-lrc/include/ffmpeg_rs_lrc.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    int64_t start = 0;
    (void)ffmpeg_rs_lrc_count_ts_prefix(data, size);
    (void)ffmpeg_rs_lrc_read_ts(data, size, &start);
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

