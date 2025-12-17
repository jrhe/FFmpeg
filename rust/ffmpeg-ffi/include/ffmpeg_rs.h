#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

uint32_t ffmpeg_rs_version(void);

const void *ffmpeg_rs_memchr(const void *haystack, size_t len, uint8_t needle);

int ffmpeg_rs_copy_str(char *dst, size_t dst_len, const char *src);

#ifdef __cplusplus
}
#endif

