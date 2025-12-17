/*
 * Fuzzer target for the SubViewer / SubViewer1 timestamp helpers.
 *
 * This exercises the Rust-backed parsers when configured with:
 *   ./configure --enable-rust-subviewer --enable-rust-subviewer1 ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && (defined(CONFIG_RUST_SUBVIEWER) || defined(CONFIG_RUST_SUBVIEWER1))

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-subviewer/include/ffmpeg_rs_subviewer.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    int64_t start_ms = 0;
    int duration_ms = 0;
    int hh = 0, mm = 0, ss = 0;

    (void)ffmpeg_rs_subviewer_read_ts(data, size, &start_ms, &duration_ms);
    (void)ffmpeg_rs_subviewer1_parse_time(data, size, &hh, &mm, &ss);
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

