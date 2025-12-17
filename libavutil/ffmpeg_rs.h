#pragma once

/* Optional Rust helpers. This header is safe to include even when Rust is not
 * built/enabled; users must guard calls behind HAVE_FFMPEG_RUST. */

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifdef HAVE_FFMPEG_RUST
uint32_t ffmpeg_rs_version(void);
const void *ffmpeg_rs_memchr(const void *haystack, size_t len, uint8_t needle);
int ffmpeg_rs_copy_str(char *dst, size_t dst_len, const char *src);
#endif

uint32_t avpriv_ffmpeg_rs_link_anchor(void);

#ifdef __cplusplus
}
#endif
