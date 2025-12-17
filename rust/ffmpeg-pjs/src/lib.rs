#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn is_digit(b: u8) -> bool {
    b'0' <= b && b <= b'9'
}

fn parse_i64_ascii(s: &[u8]) -> Option<(i64, usize)> {
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
    Some((sign.saturating_mul(v), i))
}

fn parse_line(line: &[u8]) -> Option<(usize, i64, i32)> {
    let (start, n1) = parse_i64_ascii(line)?;
    if n1 >= line.len() || line[n1] != b',' {
        return None;
    }
    let rest = &line[n1 + 1..];
    let (end, n2) = parse_i64_ascii(rest)?;
    let idx_after_end = n1 + 1 + n2;
    if idx_after_end >= line.len() || line[idx_after_end] != b',' {
        return None;
    }
    let payload_off = idx_after_end + 1;
    let duration = end.saturating_sub(start);
    if duration < 0 || duration > i32::MAX as i64 {
        return None;
    }
    Some((payload_off, start, duration as i32))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_pjs_parse_line(
    line: *const u8,
    line_len: usize,
    out_payload_off: *mut usize,
    out_start: *mut i64,
    out_duration: *mut c_int,
) -> c_int {
    if line.is_null() || out_payload_off.is_null() || out_start.is_null() || out_duration.is_null() {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    let (off, start, dur) = match parse_line(buf) {
        Some(v) => v,
        None => return -2,
    };
    unsafe {
        *out_payload_off = off;
        *out_start = start;
        *out_duration = dur as c_int;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        let (off, start, dur) = parse_line(b"12,34,\"hi\"").unwrap();
        assert_eq!(start, 12);
        assert_eq!(dur, 22);
        assert_eq!(&b"12,34,\"hi\""[off..], b"\"hi\"");
    }
}

