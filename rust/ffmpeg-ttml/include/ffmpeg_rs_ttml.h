#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsTtmlExtradataParseResult {
    int is_paragraph_mode; /* 1 if signature present, else 0 */
    int is_default;        /* 1 if no additional strings */

    size_t tt_params_offset;
    size_t pre_body_offset;
} FFmpegRsTtmlExtradataParseResult;

/* Parse TTML extradata produced by the TTML encoder.
 * Returns 0 on success, <0 on error.
 *
 * Offsets (if non-zero) point into `extradata` and reference NUL-terminated
 * strings already present in the buffer.
 */
int ffmpeg_rs_ttml_parse_extradata(const uint8_t *extradata, size_t extradata_len,
                                   FFmpegRsTtmlExtradataParseResult *out);

#ifdef __cplusplus
}
#endif

