/*
 * Fuzzer target for the ASS/SSA Dialogue line parser helper.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-ass ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_ASS)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-ass/include/ffmpeg_rs_ass.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsAssDialogueParseResult out;
    (void)ffmpeg_rs_ass_parse_dialogue(data, size, &out);
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

