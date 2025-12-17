#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsVplayerEvent {
    int64_t start_cs; /* centiseconds (1/100s) */
    size_t payload_offset;
    size_t payload_len;
} FFmpegRsVplayerEvent;

/* Parse a single VPlayer line:
 *   "H:MM:SS[.CC][: =]text"
 *
 * Returns 0 on success, <0 on error.
 */
int ffmpeg_rs_vplayer_parse_line(const uint8_t *line, size_t line_len,
                                 FFmpegRsVplayerEvent *out);

#ifdef __cplusplus
}
#endif

