#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFMpegRsConcatKeyword {
    size_t skip;
    size_t len;
    size_t advance;
} FFMpegRsConcatKeyword;

/* Parse a whitespace-delimited keyword, like concatdec.c:get_keyword().
 *
 * `buf_len` must include the trailing NUL byte (i.e. `strlen(buf)+1`).
 * On success, fills `out`:
 * - `skip`: leading whitespace bytes skipped
 * - `len`: keyword length
 * - `advance`: bytes to advance buf pointer (skips delimiter + whitespace)
 */
int ffmpeg_rs_concat_parse_keyword(const uint8_t *buf, size_t buf_len, FFMpegRsConcatKeyword *out);

/* Parse a token, like libavutil:av_get_token(buf, SPACE_CHARS).
 *
 * `buf_len` must include the trailing NUL byte (i.e. `strlen(buf)+1`).
 * Returns 0 on success; `dst` is always NUL-terminated when `dst_len > 0`.
 *
 * `out_advance` is the number of bytes to advance the input pointer to the
 * delimiter (matching `av_get_token` behavior).
 * `out_required` is the required `dst_len` for success (including NUL).
 */
int ffmpeg_rs_concat_get_token(const uint8_t *buf, size_t buf_len, char *dst, size_t dst_len,
                               size_t *out_advance, size_t *out_required);

#ifdef __cplusplus
}
#endif

