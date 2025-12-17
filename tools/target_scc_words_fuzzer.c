/*
 * Fuzzer target for the SCC word parser helper.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-scc ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_SCC)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-scc/include/ffmpeg_rs_scc.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsSccParseWordsResult out;
    uint16_t words[2048];
    (void)ffmpeg_rs_scc_parse_words(data, size, &out, words, 2048);
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

