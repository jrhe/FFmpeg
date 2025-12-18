#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum FFmpegRsHlsDemuxEventKind {
    FFMPEG_RS_HLS_EVENT_URI = 0,
    FFMPEG_RS_HLS_EVENT_EXTINF = 1,
    FFMPEG_RS_HLS_EVENT_STREAM_INF = 2,
    FFMPEG_RS_HLS_EVENT_TARGETDURATION = 3,
    FFMPEG_RS_HLS_EVENT_MEDIA_SEQUENCE = 4,
    FFMPEG_RS_HLS_EVENT_ENDLIST = 5,
    FFMPEG_RS_HLS_EVENT_UNKNOWN = 255,
} FFmpegRsHlsDemuxEventKind;

typedef struct FFmpegRsHlsDemuxEvent {
    uint32_t kind;
    uint32_t line_no;

    /* Primary slice into the input buffer (e.g. URI, attribute list, etc.). */
    size_t a_offset;
    size_t a_len;

    /* Secondary slice into the input buffer (e.g. EXTINF title). */
    size_t b_offset;
    size_t b_len;

    /* Parsed numeric fields (meaning depends on kind). */
    int64_t i64_a;
    int64_t i64_b;
} FFmpegRsHlsDemuxEvent;

typedef struct FFmpegRsHlsDemuxParseEventsResult {
    size_t n_events_total;
    size_t n_events_written;
    int truncated;
} FFmpegRsHlsDemuxParseEventsResult;

/*
 * Parse an HLS playlist into a flat stream of events.
 *
 * - Returns 0 on success, <0 on error.
 * - All returned slices are offsets/lengths into `text`; caller must keep
 *   `text` alive while consuming events.
 * - If `events` is NULL or `events_cap` is 0, this performs a size-only pass
 *   and reports `n_events_total`.
 */
int ffmpeg_rs_hls_demux_parse_events(const uint8_t *text, size_t text_len,
                                     FFmpegRsHlsDemuxParseEventsResult *out,
                                     FFmpegRsHlsDemuxEvent *events, size_t events_cap);

#ifdef __cplusplus
}
#endif
