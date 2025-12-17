#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFMpegRsDataUriParsed {
    size_t content_type_offset;
    size_t content_type_len;
    size_t payload_offset;
    size_t payload_len;
    int base64;
} FFMpegRsDataUriParsed;

/* Parse a data: URI of the form:
 *   data:content/type[;base64][;opt...],payload
 *
 * `uri_len` must include the trailing NUL byte (i.e. `strlen(uri)+1`).
 *
 * Returns 0 on success, negative errno on failure.
 * On success, all offsets/lengths are relative to `uri`.
 */
int ffmpeg_rs_data_uri_parse(const char *uri, size_t uri_len, FFMpegRsDataUriParsed *out);

#ifdef __cplusplus
}
#endif

