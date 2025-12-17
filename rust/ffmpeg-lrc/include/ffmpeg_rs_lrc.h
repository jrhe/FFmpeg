#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Count the timestamp prefix length (in bytes) for an LRC line.
 * Mirrors libavformat/lrcdec.c:count_ts().
 */
size_t ffmpeg_rs_lrc_count_ts_prefix(const uint8_t *line, size_t line_len);

/* Parse a single leading timestamp from an LRC line.
 *
 * Accepts:
 *   "[mm:ss.xxx]" or "[-mm:ss.xxx]" with spaces/tabs before '['.
 *
 * On success: returns number of bytes consumed (including closing ']') and
 * writes `out_start_us` in AV_TIME_BASE (microseconds).
 * On failure or no timestamp at start: returns 0.
 */
size_t ffmpeg_rs_lrc_read_ts(const uint8_t *line, size_t line_len, int64_t *out_start_us);

#ifdef __cplusplus
}
#endif

