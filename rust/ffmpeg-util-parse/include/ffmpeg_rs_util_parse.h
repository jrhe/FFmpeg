#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse a token like libavutil:av_get_token(buf, term) but with caller-managed
 * output buffer.
 *
 * - Skips leading " \\n\\t\\r"
 * - Stops when the current char is in `term`
 * - Supports backslash escapes and single-quoted strings
 * - Trims trailing whitespace to the last non-whitespace produced
 *
 * `buf_len` and `term_len` must include trailing NUL bytes (i.e. strlen+1).
 *
 * On return:
 * - `out_advance` is the number of bytes to advance the input cursor (points at
 *   the delimiter or NUL, matching `av_get_token`).
 * - `out_required` is the required `dst_len` to succeed (including NUL).
 *
 * Returns 0 on success, `-ENOSPC` if `dst_len` is too small, negative errno on
 * invalid args.
 */
int ffmpeg_rs_util_get_token(const uint8_t *buf, size_t buf_len,
                             const uint8_t *term, size_t term_len,
                             char *dst, size_t dst_len,
                             size_t *out_advance, size_t *out_required);

#ifdef __cplusplus
}
#endif

