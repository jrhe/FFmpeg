#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Parse an AQTitle frame marker line ("-->> <frame>").
 *
 * Returns 0 on success, <0 on parse failure.
 */
int ffmpeg_rs_aqtitle_parse_marker(const uint8_t *line, size_t line_len, int64_t *out_frame);

#ifdef __cplusplus
}
#endif

