#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Returns bytes written (excluding NUL), or <0 on error. */
ptrdiff_t ffmpeg_rs_hls_write_playlist_version(char *dst, size_t dst_len, int version);

#ifdef __cplusplus
}
#endif

