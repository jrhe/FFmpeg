/*
 * Fuzzer target for the MPL2 line parser.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-mpl2 ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_MPL2)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-mpl2/include/ffmpeg_rs_mpl2.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsMpl2Event out;
    (void)ffmpeg_rs_mpl2_parse_line(data, size, &out);
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

