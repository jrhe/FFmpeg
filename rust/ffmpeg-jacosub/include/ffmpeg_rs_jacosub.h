#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse a SHIFT directive parameter into a frame offset.
 * Matches libavformat/jacosubdec.c behavior.
 * Returns 0 if parsing fails or overflows.
 */
int ffmpeg_rs_jacosub_parse_shift(unsigned timeres, const uint8_t *text, size_t text_len);

/* Parse a timed line and compute packet timing in centiseconds.
 *
 * Supports:
 *   "HH:MM:SS.FF HH:MM:SS.FF ..."
 *   "@start @end ..."
 *
 * Returns 0 on success, <0 on parse failure.
 */
int ffmpeg_rs_jacosub_read_ts(unsigned timeres, int shift_frames,
                              const uint8_t *line, size_t line_len,
                              int64_t *out_start_cs, int64_t *out_duration_cs);

#ifdef __cplusplus
}
#endif

