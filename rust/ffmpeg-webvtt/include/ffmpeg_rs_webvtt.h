#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsWebvttCue {
    int64_t start_ms;
    int64_t end_ms;
    size_t payload_offset;
    size_t payload_len;

    size_t identifier_offset;
    size_t identifier_len;

    size_t settings_offset;
    size_t settings_len;
} FFmpegRsWebvttCue;

typedef struct FFmpegRsWebvttParseResult {
    size_t n_cues;
} FFmpegRsWebvttParseResult;

/* Parses WebVTT text into cues. Returns 0 on success, <0 on error. */
int ffmpeg_rs_webvtt_parse(const uint8_t *text, size_t text_len,
                           FFmpegRsWebvttParseResult *out,
                           FFmpegRsWebvttCue *cues, size_t cues_cap);

#ifdef __cplusplus
}
#endif

