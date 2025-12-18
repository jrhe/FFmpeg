/*
 * Fuzzer target for ffmetadata demuxer key/value parsing helpers.
 *
 * This exercises the Rust-backed helpers when configured with:
 *   ./configure --enable-rust-ffmetadata ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_FFMETADATA)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-ffmetadata/include/ffmpeg_rs_ffmetadata.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    uint8_t buf[1024];
    size_t n;
    FFMpegRsFFMetaSplit split;

    if (!data || !size)
        return 0;

    n = size < (sizeof(buf) - 1) ? size : (sizeof(buf) - 1);
    for (size_t i = 0; i < n; i++)
        buf[i] = data[i];
    buf[n] = 0;

    if (ffmpeg_rs_ffmetadata_split_kv(buf, n + 1, &split) == 0) {
        uint8_t out[1024];
        size_t written = 0;
        (void)ffmpeg_rs_ffmetadata_unescape(out, sizeof(out), buf, split.key_escaped_len + 1, &written);
        (void)ffmpeg_rs_ffmetadata_unescape(out, sizeof(out), buf + split.eq_offset + 1, split.value_escaped_len + 1, &written);
    }
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

