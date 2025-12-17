#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse a SAMI "Start" attribute value into milliseconds.
 *
 * Returns 0 on success, <0 on parse failure / overflow.
 */
int ffmpeg_rs_sami_parse_start_ms(const uint8_t *s, size_t s_len, int64_t *out_ms);

#ifdef __cplusplus
}
#endif

