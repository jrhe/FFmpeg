/*
 * Fuzzer target for the MCC bytes-to-hex helper.
 *
 * This exercises the Rust-backed function when configured with:
 *   ./configure --enable-rust-mcc ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_MCC)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-mcc/include/ffmpeg_rs_mcc.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    /* worst-case output: 2 chars per byte + NUL */
    if (size > 4096)
        size = 4096;
    char out[1 + 2 * 4096];
    (void)ffmpeg_rs_mcc_bytes_to_hex(out, sizeof(out), data, size, 1);
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

