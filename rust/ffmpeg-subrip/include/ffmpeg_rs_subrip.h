#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsSubripEvent {
    int64_t start_ms;
    int64_t duration_ms;
    size_t payload_offset;
    size_t payload_len;
} FFmpegRsSubripEvent;

typedef struct FFmpegRsSubripParseResult {
    size_t n_events;
} FFmpegRsSubripParseResult;

/*
 * Parses SubRip (SRT) into events. Returns 0 on success.
 * Payload is returned as a slice into the input buffer.
 */
int ffmpeg_rs_subrip_parse(const uint8_t *text, size_t text_len,
                           FFmpegRsSubripParseResult *out,
                           FFmpegRsSubripEvent *events, size_t events_cap);

#ifdef __cplusplus
}
#endif

