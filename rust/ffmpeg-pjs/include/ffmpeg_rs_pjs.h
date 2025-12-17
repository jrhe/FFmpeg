#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse a PJS line of the form:
 *   "<start>,<end>,\"payload\""
 *
 * Returns 0 on success, <0 on parse failure.
 */
int ffmpeg_rs_pjs_parse_line(const uint8_t *line, size_t line_len,
                             size_t *out_payload_off,
                             int64_t *out_start,
                             int *out_duration);

#ifdef __cplusplus
}
#endif

