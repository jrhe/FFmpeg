#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsMpl2Event {
    int64_t start_ticks;     /* 1 tick = 1/10 s (matches demuxer time_base) */
    int64_t duration_ticks;  /* -1 if unknown */
    size_t payload_offset;
    size_t payload_len;
} FFmpegRsMpl2Event;

/* Parse a single MPL2 line:
 *   "[start][end]text" or "[start][]text"
 * where start/end are integer ticks in 1/10s.
 *
 * Returns 0 on success, <0 on error.
 */
int ffmpeg_rs_mpl2_parse_line(const uint8_t *line, size_t line_len,
                              FFmpegRsMpl2Event *out);

#ifdef __cplusplus
}
#endif

