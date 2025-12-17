#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const EINVAL: c_int = -22;
const ENOSPC: c_int = -28;

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\t' | b'\r')
}

#[repr(C)]
pub struct FFMpegRsConcatKeyword {
    skip: usize,
    len: usize,
    advance: usize,
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_concat_parse_keyword(
    buf: *const u8,
    buf_len: usize,
    out: *mut FFMpegRsConcatKeyword,
) -> c_int {
    if buf.is_null() || out.is_null() || buf_len == 0 {
        return EINVAL;
    }
    let bytes = unsafe { core::slice::from_raw_parts(buf, buf_len) };

    let mut i = 0usize;
    while i < bytes.len() && is_ws(bytes[i]) {
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i] != 0 && !is_ws(bytes[i]) {
        i += 1;
    }
    let end = i;

    let mut adv = end;
    if adv < bytes.len() && bytes[adv] != 0 {
        adv += 1;
        while adv < bytes.len() && is_ws(bytes[adv]) {
            adv += 1;
        }
    }

    unsafe {
        *out = FFMpegRsConcatKeyword {
            skip: start,
            len: end.saturating_sub(start),
            advance: adv,
        };
    }
    0
}

fn write_byte(dst: *mut u8, dst_len: usize, pos: &mut usize, b: u8) {
    if dst_len > 0 && *pos + 1 < dst_len {
        unsafe {
            *dst.add(*pos) = b;
        }
    }
    *pos += 1;
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_concat_get_token(
    buf: *const u8,
    buf_len: usize,
    dst: *mut u8,
    dst_len: usize,
    out_advance: *mut usize,
    out_required: *mut usize,
) -> c_int {
    if buf.is_null() || out_advance.is_null() || out_required.is_null() || buf_len == 0 {
        return EINVAL;
    }
    if dst_len > 0 && dst.is_null() {
        return EINVAL;
    }

    let bytes = unsafe { core::slice::from_raw_parts(buf, buf_len) };
    let mut i = 0usize;
    while i < bytes.len() && is_ws(bytes[i]) {
        i += 1;
    }

    let mut out_pos = 0usize;
    let mut end_pos = 0usize;

    while i < bytes.len() && bytes[i] != 0 && !is_ws(bytes[i]) {
        let c = bytes[i];
        i += 1;
        if c == b'\\' && i < bytes.len() && bytes[i] != 0 {
            let escaped = bytes[i];
            i += 1;
            write_byte(dst, dst_len, &mut out_pos, escaped);
            end_pos = out_pos;
        } else if c == b'\'' {
            while i < bytes.len() && bytes[i] != 0 && bytes[i] != b'\'' {
                let inner = bytes[i];
                i += 1;
                write_byte(dst, dst_len, &mut out_pos, inner);
            }
            if i < bytes.len() && bytes[i] == b'\'' {
                i += 1;
                end_pos = out_pos;
            }
        } else {
            write_byte(dst, dst_len, &mut out_pos, c);
        }
    }

    let mut trim_pos = out_pos;
    while trim_pos > end_pos {
        let idx = trim_pos - 1;
        let b = if dst_len > 0 && idx < dst_len {
            unsafe { *dst.add(idx) }
        } else {
            break;
        };
        if !is_ws(b) {
            break;
        }
        trim_pos -= 1;
    }

    let required = trim_pos.saturating_add(1);
    unsafe {
        *out_required = required;
        *out_advance = i;
    }

    if dst_len == 0 || required > dst_len {
        return ENOSPC;
    }

    unsafe {
        *dst.add(trim_pos) = 0;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_matches_get_keyword_behavior() {
        let mut line = b"   file   abc\n\0".to_vec();
        let mut out = FFMpegRsConcatKeyword {
            skip: 0,
            len: 0,
            advance: 0,
        };
        let rc = ffmpeg_rs_concat_parse_keyword(line.as_ptr(), line.len(), &mut out);
        assert_eq!(rc, 0);
        assert_eq!(out.skip, 3);
        assert_eq!(out.len, 4);
        assert_eq!(out.advance, 10);
        assert_eq!(&line[out.skip..out.skip + out.len], b"file");
    }

    #[test]
    fn token_parses_backslash_and_quotes() {
        let s = b"  a\\ b 'c d'  \0";
        let mut advance = 0usize;
        let mut req = 0usize;

        let mut buf = [0u8; 64];
        let rc = ffmpeg_rs_concat_get_token(s.as_ptr(), s.len(), buf.as_mut_ptr(), buf.len(), &mut advance, &mut req);
        assert_eq!(rc, 0);
        assert_eq!(advance, 6); // "  a\\ b" ends at delimiter
        assert_eq!(&buf[..4], b"a b\0");

        let s2 = &s[advance..];
        let rc2 = ffmpeg_rs_concat_get_token(
            s2.as_ptr(),
            s2.len(),
            buf.as_mut_ptr(),
            buf.len(),
            &mut advance,
            &mut req,
        );
        assert_eq!(rc2, 0);
        assert_eq!(&buf[..4], b"c d\0");
    }
}

