/*
 * Fuzzer target for the MPSub timing line parser helper.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-mpsub ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_MPSUB)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-mpsub/include/ffmpeg_rs_mpsub.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    int64_t a = 0, b = 0;
    (void)ffmpeg_rs_mpsub_parse_line(data, size, &a, &b);
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

