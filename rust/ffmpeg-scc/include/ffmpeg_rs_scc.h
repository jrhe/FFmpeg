#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FFmpegRsSccParseWordsResult {
    size_t n_words_total;
    size_t n_words_written;
    int truncated;
} FFmpegRsSccParseWordsResult;

/* Parse SCC payload hex words ("9420 942c ...") into u16 words (0x9420, 0x942c, ...).
 *
 * Returns 0 on success, <0 on invalid args.
 * Parsing stops at the first invalid token (mirrors demuxer behavior).
 */
int ffmpeg_rs_scc_parse_words(const uint8_t *text, size_t text_len,
                              FFmpegRsSccParseWordsResult *out,
                              uint16_t *words, size_t words_cap);

#ifdef __cplusplus
}
#endif

