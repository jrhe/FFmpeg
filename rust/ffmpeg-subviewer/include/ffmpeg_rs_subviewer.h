#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse a SubViewer timestamp line into start time and duration in ms.
 *
 * Supports: "HH:MM:SS.mmm,HH:MM:SS.mmm" with 1–3 fractional digits.
 *
 * Returns 0 on success, <0 on parse failure.
 */
int ffmpeg_rs_subviewer_read_ts(const uint8_t *line, size_t line_len,
                                int64_t *out_start_ms, int *out_duration_ms);

/* Parse a SubViewer v1 time tag "[HH:MM:SS]" (signed ints, like sscanf("%d")).
 *
 * Returns 1 on success, 0 on no match / parse failure.
 */
int ffmpeg_rs_subviewer1_parse_time(const uint8_t *line, size_t line_len,
                                    int *out_hh, int *out_mm, int *out_ss);

#ifdef __cplusplus
}
#endif

