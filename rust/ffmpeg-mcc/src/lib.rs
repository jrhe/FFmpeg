#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn hex_nibble(v: u8) -> u8 {
    if v < 10 {
        b'0' + v
    } else {
        b'A' + (v - 10)
    }
}

#[repr(C)]
pub struct FFmpegRsMccExpandPayloadResult {
    pub n_bytes_total: usize,
    pub n_bytes_written: usize,
    pub truncated: c_int,
}

fn convert_like_c(x: u8) -> Option<u8> {
    match x {
        b'0'..=b'9' => Some(x - b'0'),
        b'A'..=b'Z' => Some(x - 55),
        b'a'..=b'z' => Some(x - 87),
        _ => None,
    }
}

fn push_bytes(
    out: &mut FFmpegRsMccExpandPayloadResult,
    bytes: *mut u8,
    bytes_cap: usize,
    src: &[u8],
) {
    out.n_bytes_total = out.n_bytes_total.saturating_add(src.len());
    if bytes.is_null() || bytes_cap == 0 {
        out.truncated = 1;
        return;
    }
    let mut i = 0usize;
    while i < src.len() {
        if out.n_bytes_written < bytes_cap {
            unsafe {
                *bytes.add(out.n_bytes_written) = src[i];
            }
            out.n_bytes_written += 1;
        } else {
            out.truncated = 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_mcc_expand_payload(
    text: *const u8,
    text_len: usize,
    out: *mut FFmpegRsMccExpandPayloadResult,
    bytes: *mut u8,
    bytes_cap: usize,
) -> c_int {
    if text.is_null() || out.is_null() {
        return -1;
    }
    unsafe {
        (*out).n_bytes_total = 0;
        (*out).n_bytes_written = 0;
        (*out).truncated = 0;
    }
    let mut r = FFmpegRsMccExpandPayloadResult {
        n_bytes_total: 0,
        n_bytes_written: 0,
        truncated: 0,
    };
    let s = unsafe { core::slice::from_raw_parts(text, text_len) };
    let mut i = 0usize;
    while i < s.len() {
        let v = match convert_like_c(s[i]) {
            Some(v) => v,
            None => break,
        };
        i += 1;

        if (16..=35).contains(&v) {
            match v {
                16..=24 => {
                    let n = (v as usize - 15) * 3;
                    // Repeat "\xFA\x00\x00" (n/3) times.
                    let trip = [0xFAu8, 0x00u8, 0x00u8];
                    for _ in 0..(n / 3) {
                        push_bytes(&mut r, bytes, bytes_cap, &trip);
                    }
                }
                25 => push_bytes(&mut r, bytes, bytes_cap, &[0xFB, 0x80, 0x80]),
                26 => push_bytes(&mut r, bytes, bytes_cap, &[0xFC, 0x80, 0x80]),
                27 => push_bytes(&mut r, bytes, bytes_cap, &[0xFD, 0x80, 0x80]),
                28 => push_bytes(&mut r, bytes, bytes_cap, &[0x96, 0x69]),
                29 => push_bytes(&mut r, bytes, bytes_cap, &[0x61, 0x01]),
                30 | 31 => push_bytes(&mut r, bytes, bytes_cap, &[0xFC, 0x80, 0x80]),
                32 => push_bytes(&mut r, bytes, bytes_cap, &[0xE1, 0x00, 0x00, 0x00]),
                33 | 34 => {} // no-op
                35 => push_bytes(&mut r, bytes, bytes_cap, &[0x00]),
                _ => {}
            }
        } else {
            if i >= s.len() {
                break;
            }
            let vv = match convert_like_c(s[i]) {
                Some(vv) if vv < 16 => vv,
                _ => break,
            };
            i += 1;
            let b = (v << 4) | vv;
            push_bytes(&mut r, bytes, bytes_cap, &[b]);
        }
    }

    unsafe {
        (*out).n_bytes_total = r.n_bytes_total;
        (*out).n_bytes_written = r.n_bytes_written;
        (*out).truncated = r.truncated;
    }
    0
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_mcc_bytes_to_hex(
    dest: *mut u8,
    dest_cap: usize,
    bytes: *const u8,
    bytes_size: usize,
    use_u_alias: c_int,
) -> c_int {
    if dest.is_null() || bytes.is_null() {
        return -1;
    }
    // Worst-case output is 2 hex chars per input byte plus NUL.
    if dest_cap < 1 + 2 * bytes_size {
        return -2;
    }

    let inb = unsafe { core::slice::from_raw_parts(bytes, bytes_size) };
    let out = unsafe { core::slice::from_raw_parts_mut(dest, dest_cap) };
    let mut oi = 0usize;
    let mut i = 0usize;

    while i < inb.len() {
        match inb[i] {
            0xFA => {
                // Count 0xFA 00 00 triplets and map to 'G'..'O'
                let mut code = b'G';
                let mut j = i;
                while code <= b'O' {
                    if j + 2 >= inb.len() {
                        break;
                    }
                    if inb[j] != 0xFA || inb[j + 1] != 0 || inb[j + 2] != 0 {
                        break;
                    }
                    j += 3;
                    code += 1;
                }
                if code != b'G' {
                    // last successful code is (code-1)
                    out[oi] = code - 1;
                    oi += 1;
                    i = j;
                    continue;
                }
            }
            0xFB | 0xFC | 0xFD => {
                if i + 2 < inb.len() && inb[i + 1] == 0x80 && inb[i + 2] == 0x80 {
                    out[oi] = (inb[i] - 0xFB) + b'P';
                    oi += 1;
                    i += 3;
                    continue;
                }
            }
            0x96 => {
                if i + 1 < inb.len() && inb[i + 1] == 0x69 {
                    out[oi] = b'S';
                    oi += 1;
                    i += 2;
                    continue;
                }
            }
            0x61 => {
                if i + 1 < inb.len() && inb[i + 1] == 0x01 {
                    out[oi] = b'T';
                    oi += 1;
                    i += 2;
                    continue;
                }
            }
            0xE1 => {
                if use_u_alias != 0
                    && i + 3 < inb.len()
                    && inb[i + 1] == 0
                    && inb[i + 2] == 0
                    && inb[i + 3] == 0
                {
                    out[oi] = b'U';
                    oi += 1;
                    i += 4;
                    continue;
                }
            }
            0x00 => {
                out[oi] = b'Z';
                oi += 1;
                i += 1;
                continue;
            }
            _ => {}
        }

        let b = inb[i];
        out[oi] = hex_nibble((b >> 4) & 0xF);
        out[oi + 1] = hex_nibble(b & 0xF);
        oi += 2;
        i += 1;
    }

    out[oi] = 0;
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_hex() {
        let input = [0x12u8, 0xABu8, 0x00u8];
        let mut out = [0u8; 16];
        assert_eq!(
            ffmpeg_rs_mcc_bytes_to_hex(out.as_mut_ptr(), out.len(), input.as_ptr(), input.len(), 0),
            0
        );
        assert_eq!(&out[..5], b"12ABZ");
    }

    #[test]
    fn expands_payload_with_aliases_and_hex() {
        // 'G' => 3-byte cc_pad, 'Z' => 0x00, '0F' => 0x0F
        let input = b"GZ0F";
        let mut out = FFmpegRsMccExpandPayloadResult {
            n_bytes_total: 0,
            n_bytes_written: 0,
            truncated: 0,
        };
        let mut bytes = [0u8; 16];
        assert_eq!(
            ffmpeg_rs_mcc_expand_payload(
                input.as_ptr(),
                input.len(),
                &mut out,
                bytes.as_mut_ptr(),
                bytes.len()
            ),
            0
        );
        assert_eq!(out.n_bytes_written, 5);
        assert_eq!(&bytes[..5], &[0xFA, 0x00, 0x00, 0x00, 0x0F]);
    }
}
