/*
 * Fuzzer target for the MCC payload expander helper.
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
    FFmpegRsMccExpandPayloadResult r;
    uint8_t out[4096];
    (void)ffmpeg_rs_mcc_expand_payload(data, size, &r, out, sizeof(out));
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

