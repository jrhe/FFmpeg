#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse a RealText timestamp string into centiseconds.
 *
 * Supports the formats accepted by libavformat/realtextdec.c:read_ts().
 *
 * Returns 0 on success, <0 on parse failure.
 */
int ffmpeg_rs_realtext_read_ts(const uint8_t *s, size_t s_len, int64_t *out_cs);

#ifdef __cplusplus
}
#endif

