/*
 * Fuzzer target for data: URI protocol parsing helper.
 *
 * This exercises the Rust-backed helper when configured with:
 *   ./configure --enable-rust-data-uri ...
 */

#include "config.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_DATA_URI)

#include <stddef.h>
#include <stdint.h>

#include "../rust/ffmpeg-data-uri/include/ffmpeg_rs_data_uri.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    char uri[1024];
    size_t n;
    FFMpegRsDataUriParsed out;

    if (!data || !size)
        return 0;

    n = size < (sizeof(uri) - 1) ? size : (sizeof(uri) - 1);
    for (size_t i = 0; i < n; i++)
        uri[i] = (char)data[i];
    uri[n] = 0;

    (void)ffmpeg_rs_data_uri_parse(uri, n + 1, &out);
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

