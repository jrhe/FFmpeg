/*
 * Fuzzer target for Rust util token parsing helper (av_get_token equivalent).
 *
 * This exercises the Rust-backed helper when configured with:
 *   ./configure --enable-rust-util-parse ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_UTIL_PARSE)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-util-parse/include/ffmpeg_rs_util_parse.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    uint8_t buf[1024];
    uint8_t term[32];
    size_t n, tlen, advance = 0, required = 0;
    char out[512];

    if (!data || !size)
        return 0;

    tlen = data[0] & 31;
    if (tlen == 0)
        tlen = 1;
    if (tlen > sizeof(term) - 1)
        tlen = sizeof(term) - 1;

    for (size_t i = 0; i < tlen && i + 1 < size; i++)
        term[i] = data[i + 1];
    term[tlen] = 0;

    n = size > (tlen + 1) ? (size - (tlen + 1)) : 0;
    if (n > sizeof(buf) - 1)
        n = sizeof(buf) - 1;
    for (size_t i = 0; i < n; i++)
        buf[i] = data[tlen + 1 + i];
    buf[n] = 0;

    (void)ffmpeg_rs_util_get_token(buf, n + 1, term, tlen + 1, out, sizeof(out), &advance, &required);
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

