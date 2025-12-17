/*
 * Fuzzer target for ffconcat/concat script parsing primitives.
 *
 * This exercises the Rust-backed helpers when configured with:
 *   ./configure --enable-rust-concat ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_CONCAT)

#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "../rust/ffmpeg-concat/include/ffmpeg_rs_concat.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    size_t i, advance = 0, required = 0;
    FFMpegRsConcatKeyword kw = { 0 };
    char out[256];

    if (!data || !size)
        return 0;

    /* Ensure NUL-termination like the concatdemux line reader provides. */
    uint8_t buf[1024];
    size_t n = size < (sizeof(buf) - 1) ? size : (sizeof(buf) - 1);
    for (i = 0; i < n; i++)
        buf[i] = data[i];
    buf[n] = 0;

    (void)ffmpeg_rs_concat_parse_keyword(buf, n + 1, &kw);
    (void)ffmpeg_rs_concat_get_token(buf, n + 1, out, sizeof(out), &advance, &required);
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

