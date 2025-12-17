#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse one MPSub timing line into (start, duration) in TSBASE ticks (10,000,000).
 * Mirrors libavformat/mpsubdec.c:parse_line().
 *
 * Returns 0 on success, <0 on parse failure.
 */
int ffmpeg_rs_mpsub_parse_line(const uint8_t *line, size_t line_len,
                               int64_t *out_start, int64_t *out_duration);

#ifdef __cplusplus
}
#endif

