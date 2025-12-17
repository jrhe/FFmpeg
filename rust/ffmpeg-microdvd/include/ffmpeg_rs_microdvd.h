#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsMicrodvdEvent {
    int64_t start_frame;
    int64_t duration_frames; /* -1 if unknown */
    size_t payload_offset;
    size_t payload_len;
} FFmpegRsMicrodvdEvent;

/* Parse a single MicroDVD subtitle line of the form:
 *   "{start}{end}text" or "{start}{}text"
 * Returns 0 on success, <0 on error.
 *
 * `payload_*` are slices into the original `line` buffer.
 */
int ffmpeg_rs_microdvd_parse_line(const uint8_t *line, size_t line_len,
                                  FFmpegRsMicrodvdEvent *out);

#ifdef __cplusplus
}
#endif

