#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Compute ID3v2 tag length from the 10-byte header.
 *
 * Returns 0 on invalid args, otherwise the total tag length (including header).
 */
int ffmpeg_rs_id3v2_tag_len(const uint8_t *buf, size_t buf_len);

#ifdef __cplusplus
}
#endif

