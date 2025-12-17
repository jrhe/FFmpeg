#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn is_ws(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\r' || b == b'\n'
}

fn is_digit(b: u8) -> bool {
    b'0' <= b && b <= b'9'
}

fn skip_ws(mut s: &[u8]) -> &[u8] {
    while let Some((&b, rest)) = s.split_first() {
        if !is_ws(b) {
            break;
        }
        s = rest;
    }
    s
}

fn parse_i64_ascii(s: &[u8]) -> Option<i64> {
    let s = skip_ws(s);
    if s.is_empty() {
        return None;
    }
    let mut i = 0usize;
    let mut sign: i64 = 1;
    if s[i] == b'+' {
        i += 1;
    } else if s[i] == b'-' {
        sign = -1;
        i += 1;
    }
    if i >= s.len() || !is_digit(s[i]) {
        return None;
    }
    let mut v: i64 = 0;
    while i < s.len() && is_digit(s[i]) {
        v = v.saturating_mul(10).saturating_add((s[i] - b'0') as i64);
        i += 1;
    }
    Some(sign.saturating_mul(v))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_sami_parse_start_ms(
    s: *const u8,
    s_len: usize,
    out_ms: *mut i64,
) -> c_int {
    if s.is_null() || out_ms.is_null() {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(s, s_len) };
    let v = match parse_i64_ascii(buf) {
        Some(v) => v,
        None => return -2,
    };
    unsafe { *out_ms = v };
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_start() {
        assert_eq!(parse_i64_ascii(b" 123 "), Some(123));
    }
}

