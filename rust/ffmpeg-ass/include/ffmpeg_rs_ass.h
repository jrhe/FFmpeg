#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsAssDialogueParseResult {
    int64_t start_cs;
    int duration_cs;
    int layer;
    size_t rest_off;
} FFmpegRsAssDialogueParseResult;

/* Parse an SSA/ASS "Dialogue:" line.
 *
 * On success, returns 0 and fills `out` with:
 * - `start_cs` / `duration_cs` in centiseconds
 * - `layer` parsed like `atoi(p + 10)` in libavformat/assdec.c (0 for "Marked=N")
 * - `rest_off` offset to the remainder after the second timestamp comma
 *
 * On failure, returns <0.
 */
int ffmpeg_rs_ass_parse_dialogue(const uint8_t *line, size_t line_len,
                                 FFmpegRsAssDialogueParseResult *out);

#ifdef __cplusplus
}
#endif

