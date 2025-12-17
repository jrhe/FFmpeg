#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFmpegRsMpl2Event {
    pub start_ticks: i64,
    pub duration_ticks: i64,
    pub payload_offset: usize,
    pub payload_len: usize,
}

fn parse_i64_ascii(s: &[u8]) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let mut neg = false;
    let mut i = 0usize;
    if s[0] == b'-' {
        neg = true;
        i = 1;
    }
    let mut v: i64 = 0;
    let mut any = false;
    while i < s.len() {
        let b = s[i];
        if b < b'0' || b > b'9' {
            break;
        }
        any = true;
        v = v.saturating_mul(10).saturating_add((b - b'0') as i64);
        i += 1;
    }
    if !any {
        return None;
    }
    Some(if neg { -v } else { v })
}

fn find_byte(hay: &[u8], b: u8, start: usize) -> Option<usize> {
    let mut i = start;
    while i < hay.len() {
        if hay[i] == b {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_mpl2_parse_line(
    line: *const u8,
    line_len: usize,
    out: *mut FFmpegRsMpl2Event,
) -> c_int {
    if line.is_null() || out.is_null() {
        return -1;
    }
    let data = unsafe { core::slice::from_raw_parts(line, line_len) };
    if data.is_empty() || data[0] != b'[' {
        return -2;
    }
    let close0 = match find_byte(data, b']', 1) {
        Some(i) => i,
        None => return -3,
    };
    let start = match parse_i64_ascii(&data[1..close0]) {
        Some(v) => v,
        None => return -4,
    };
    if close0 + 1 >= data.len() || data[close0 + 1] != b'[' {
        return -5;
    }
    let close1 = match find_byte(data, b']', close0 + 2) {
        Some(i) => i,
        None => return -6,
    };

    let duration = if close1 == close0 + 2 {
        -1
    } else {
        let end = match parse_i64_ascii(&data[close0 + 2..close1]) {
            Some(v) => v,
            None => return -7,
        };
        if end < start {
            -1
        } else {
            end.saturating_sub(start)
        }
    };

    let payload_off = close1 + 1;
    let payload_len = data.len().saturating_sub(payload_off);

    unsafe {
        *out = FFmpegRsMpl2Event {
            start_ticks: start,
            duration_ticks: duration,
            payload_offset: payload_off,
            payload_len,
        };
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_end() {
        let txt = b"[12][34]Hello";
        let mut out = FFmpegRsMpl2Event { start_ticks: 0, duration_ticks: 0, payload_offset: 0, payload_len: 0 };
        let r = ffmpeg_rs_mpl2_parse_line(txt.as_ptr(), txt.len(), &mut out);
        assert_eq!(r, 0);
        assert_eq!(out.start_ticks, 12);
        assert_eq!(out.duration_ticks, 22);
        assert_eq!(&txt[out.payload_offset..out.payload_offset + out.payload_len], b"Hello");
    }

    #[test]
    fn parses_without_end() {
        let txt = b"[12][]Hello";
        let mut out = FFmpegRsMpl2Event { start_ticks: 0, duration_ticks: 0, payload_offset: 0, payload_len: 0 };
        let r = ffmpeg_rs_mpl2_parse_line(txt.as_ptr(), txt.len(), &mut out);
        assert_eq!(r, 0);
        assert_eq!(out.start_ticks, 12);
        assert_eq!(out.duration_ticks, -1);
    }
}

