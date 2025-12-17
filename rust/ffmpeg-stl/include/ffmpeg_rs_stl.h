#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse an STL subtitle line:
 *   "HH:MM:SS:CC , HH:MM:SS:CC , <payload>"
 *
 * Returns 0 on success, <0 on parse failure.
 */
int ffmpeg_rs_stl_parse_line(const uint8_t *line, size_t line_len,
                             size_t *out_payload_off,
                             int64_t *out_start_cs,
                             int *out_duration_cs);

#ifdef __cplusplus
}
#endif

