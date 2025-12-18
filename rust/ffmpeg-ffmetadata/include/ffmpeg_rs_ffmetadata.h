#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFMpegRsFFMetaSplit {
    size_t eq_offset;
    size_t key_escaped_len;
    size_t value_escaped_len;
    size_t key_unescaped_len;
    size_t value_unescaped_len;
} FFMpegRsFFMetaSplit;

/* Find the first '=' that is not escaped by a preceding backslash, matching
 * libavformat/ffmetadec.c:read_tag() semantics.
 *
 * `line_len` must include the trailing NUL byte (i.e. `strlen(line)+1`).
 * Returns 0 and fills `out` on success; returns 1 if no split exists; returns
 * negative errno on invalid args.
 */
int ffmpeg_rs_ffmetadata_split_kv(const uint8_t *line, size_t line_len, FFMpegRsFFMetaSplit *out);

/* Unescape a byte string, removing '\' escapes, matching
 * libavformat/ffmetadec.c:unescape().
 *
 * Returns 0 on success; negative errno on failure. Always writes a trailing
 * NUL when `dst_len > 0`.
 */
int ffmpeg_rs_ffmetadata_unescape(uint8_t *dst, size_t dst_len, const uint8_t *src, size_t src_len,
                                  size_t *out_written);

#ifdef __cplusplus
}
#endif

