#![no_std]

use core::ffi::{c_char, c_int};

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const EINVAL: c_int = -22;

#[repr(C)]
pub struct FFMpegRsDataUriParsed {
    content_type_offset: usize,
    content_type_len: usize,
    payload_offset: usize,
    payload_len: usize,
    base64: c_int,
}

fn eq_ascii_case_insensitive(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(&x, &y)| x.to_ascii_lowercase() == y.to_ascii_lowercase())
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_data_uri_parse(
    uri: *const c_char,
    uri_len: usize,
    out: *mut FFMpegRsDataUriParsed,
) -> c_int {
    if uri.is_null() || out.is_null() || uri_len < 6 {
        return EINVAL;
    }
    let bytes = unsafe { core::slice::from_raw_parts(uri as *const u8, uri_len) };
    if !bytes.starts_with(b"data:") {
        return EINVAL;
    }
    let mut comma = None;
    for (i, &b) in bytes.iter().enumerate() {
        if b == 0 {
            break;
        }
        if b == b',' {
            comma = Some(i);
            break;
        }
    }
    let comma_idx = match comma {
        Some(i) => i,
        None => return EINVAL,
    };

    let meta_start = 5usize;
    if comma_idx <= meta_start {
        return EINVAL;
    }

    let mut first_end = comma_idx;
    for i in meta_start..comma_idx {
        if bytes[i] == b';' {
            first_end = i;
            break;
        }
    }
    let ctype = &bytes[meta_start..first_end];
    if ctype.is_empty() || !ctype.iter().any(|&b| b == b'/') {
        return EINVAL;
    }

    let mut base64 = 0;
    let mut opt = first_end;
    while opt < comma_idx {
        if bytes[opt] != b';' {
            break;
        }
        opt += 1;
        if opt >= comma_idx {
            break;
        }
        let mut next = comma_idx;
        for i in opt..comma_idx {
            if bytes[i] == b';' {
                next = i;
                break;
            }
        }
        let seg = &bytes[opt..next];
        if !seg.is_empty() && eq_ascii_case_insensitive(seg, b"base64") {
            base64 = 1;
        }
        opt = next;
    }

    let payload_offset = comma_idx + 1;
    let payload_len = bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(uri_len)
        .saturating_sub(payload_offset);

    unsafe {
        *out = FFMpegRsDataUriParsed {
            content_type_offset: meta_start,
            content_type_len: first_end.saturating_sub(meta_start),
            payload_offset,
            payload_len,
            base64,
        };
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_and_base64() {
        let s = b"data:audio/wav;base64,AAAA\0";
        let mut out = FFMpegRsDataUriParsed {
            content_type_offset: 0,
            content_type_len: 0,
            payload_offset: 0,
            payload_len: 0,
            base64: 0,
        };
        let rc = ffmpeg_rs_data_uri_parse(s.as_ptr() as *const c_char, s.len(), &mut out);
        assert_eq!(rc, 0);
        assert_eq!(out.base64, 1);
        assert_eq!(&s[out.content_type_offset..out.content_type_offset + out.content_type_len], b"audio/wav");
        assert_eq!(out.payload_len, 4);
    }
}

