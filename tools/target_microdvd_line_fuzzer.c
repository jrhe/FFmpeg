/*
 * Fuzzer target for the MicroDVD line parser.
 *
 * This exercises the Rust-backed parser when configured with:
 *   ./configure --enable-rust-microdvd ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_MICRODVD)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-microdvd/include/ffmpeg_rs_microdvd.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    FFmpegRsMicrodvdEvent ev;
    (void)ffmpeg_rs_microdvd_parse_line(data, size, &ev);
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

