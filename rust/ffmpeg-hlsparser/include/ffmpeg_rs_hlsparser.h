#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsHlsSegment {
    int64_t duration_us;
    size_t url_offset;
    size_t url_len;
} FFmpegRsHlsSegment;

typedef struct FFmpegRsHlsVariant {
    int bandwidth;
    size_t url_offset;
    size_t url_len;
} FFmpegRsHlsVariant;

typedef struct FFmpegRsHlsPlaylist {
    int64_t target_duration_us;
    int start_seq_no;
    int finished;

    size_t n_segments;
    size_t n_variants;
} FFmpegRsHlsPlaylist;

/*
 * Parses an HLS playlist. Returns 0 on success, <0 on error.
 *
 * `text` is the full playlist bytes (UTF-8 / ASCII).
 * Segment/variant URLs returned as slices into the same `text` buffer via
 * offsets/lengths. Caller must keep `text` alive while consuming results.
 */
int ffmpeg_rs_hls_parse(const uint8_t *text, size_t text_len,
                        FFmpegRsHlsPlaylist *out_playlist,
                        FFmpegRsHlsSegment *out_segments, size_t out_segments_cap,
                        FFmpegRsHlsVariant *out_variants, size_t out_variants_cap);

/*
 * Like `ffmpeg_rs_hls_parse`, but fails on any unknown `#EXT*` tags.
 * Intended for use by the experimental HLS demuxer apply layer.
 */
int ffmpeg_rs_hls_parse_strict(const uint8_t *text, size_t text_len,
                               FFmpegRsHlsPlaylist *out_playlist,
                               FFmpegRsHlsSegment *out_segments, size_t out_segments_cap,
                               FFmpegRsHlsVariant *out_variants, size_t out_variants_cap);

#ifdef __cplusplus
}
#endif
