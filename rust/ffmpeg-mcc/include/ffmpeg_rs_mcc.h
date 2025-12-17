#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Convert bytes to MCC hex string, including MCC alias rules.
 *
 * `dest_cap` must be >= 1 + 2*bytes_size.
 * Returns 0 on success, <0 on invalid args / insufficient output buffer.
 */
int ffmpeg_rs_mcc_bytes_to_hex(char *dest, size_t dest_cap,
                               const uint8_t *bytes, size_t bytes_size,
                               int use_u_alias);

typedef struct FFmpegRsMccExpandPayloadResult {
    size_t n_bytes_total;
    size_t n_bytes_written;
    int truncated;
} FFmpegRsMccExpandPayloadResult;

/* Expand an MCC payload string (hex + alias chars) into bytes.
 *
 * Returns 0 on success, <0 on invalid args.
 */
int ffmpeg_rs_mcc_expand_payload(const uint8_t *text, size_t text_len,
                                 FFmpegRsMccExpandPayloadResult *out,
                                 uint8_t *bytes, size_t bytes_cap);

#ifdef __cplusplus
}
#endif
