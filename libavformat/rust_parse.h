/*
 * Optional Rust parsing helpers for libavformat.
 *
 * These helpers are internal glue; keep them small and fall back to C parsing
 * on any Rust-side error.
 */

#pragma once

#include <stddef.h>

#include "libavutil/avstring.h"
#include "libavutil/mem.h"

#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_UTIL_PARSE)
#include "../rust/ffmpeg-util-parse/include/ffmpeg_rs_util_parse.h"
#endif

static inline char *ff_rust_av_get_token(const char **cursor, const char *term)
{
#if defined(HAVE_FFMPEG_RUST) && defined(CONFIG_RUST_UTIL_PARSE)
    size_t cursor_len, term_len, advance = 0, required = 0;
    char *out;
    int rc;

    if (!cursor || !*cursor || !term)
        return NULL;

    cursor_len = strlen(*cursor) + 1;
    term_len = strlen(term) + 1;

    rc = ffmpeg_rs_util_get_token((const uint8_t *)*cursor, cursor_len,
                                  (const uint8_t *)term, term_len,
                                  NULL, 0, &advance, &required);
    if (rc != -28 /* ENOSPC */ || required == 0)
        return av_get_token(cursor, term);

    out = av_malloc(required);
    if (!out)
        return NULL;

    rc = ffmpeg_rs_util_get_token((const uint8_t *)*cursor, cursor_len,
                                  (const uint8_t *)term, term_len,
                                  out, required, &advance, &required);
    if (rc < 0) {
        av_free(out);
        return av_get_token(cursor, term);
    }

    *cursor += advance;
    return out;
#else
    return av_get_token(cursor, term);
#endif
}

