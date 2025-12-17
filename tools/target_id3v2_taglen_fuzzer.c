/*
 * Fuzzer target for ID3v2 tag length helper.
 *
 * This exercises the Rust-backed function when configured with:
 *   ./configure --enable-rust-id3v2 ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_ID3V2)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-id3v2/include/ffmpeg_rs_id3v2.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    (void)ffmpeg_rs_id3v2_tag_len(data, size);
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

