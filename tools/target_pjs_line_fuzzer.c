/*
 * Fuzzer target for the PJS line parser helper.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-pjs ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_PJS)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-pjs/include/ffmpeg_rs_pjs.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    size_t off = 0;
    int64_t start = 0;
    int dur = 0;
    (void)ffmpeg_rs_pjs_parse_line(data, size, &off, &start, &dur);
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

