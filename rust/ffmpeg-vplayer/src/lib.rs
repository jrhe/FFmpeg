#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFmpegRsVplayerEvent {
    pub start_cs: i64,
    pub payload_offset: usize,
    pub payload_len: usize,
}

fn is_delim(b: u8) -> bool {
    b == b':' || b == b' ' || b == b'='
}

fn parse_u32_ascii(s: &[u8]) -> Option<(u32, usize)> {
    let mut i = 0usize;
    let mut v: u32 = 0;
    let mut any = false;
    while i < s.len() {
        let b = s[i];
        if b < b'0' || b > b'9' {
            break;
        }
        any = true;
        v = v.saturating_mul(10).saturating_add((b - b'0') as u32);
        i += 1;
    }
    if !any {
        return None;
    }
    Some((v, i))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_vplayer_parse_line(
    line: *const u8,
    line_len: usize,
    out: *mut FFmpegRsVplayerEvent,
) -> c_int {
    if line.is_null() || out.is_null() {
        return -1;
    }
    let data = unsafe { core::slice::from_raw_parts(line, line_len) };
    // Parse H:MM:SS(.CC)? then a delimiter char in ": =".
    let mut i = 0usize;

    let (hh, n) = match parse_u32_ascii(&data[i..]) {
        Some(v) => v,
        None => return -2,
    };
    i += n;
    if i >= data.len() || data[i] != b':' {
        return -3;
    }
    i += 1;

    let (mm, n) = match parse_u32_ascii(&data[i..]) {
        Some(v) => v,
        None => return -4,
    };
    i += n;
    if i >= data.len() || data[i] != b':' {
        return -5;
    }
    i += 1;

    let (ss, n) = match parse_u32_ascii(&data[i..]) {
        Some(v) => v,
        None => return -6,
    };
    i += n;

    let mut cs: u32 = 0;
    if i < data.len() && data[i] == b'.' {
        i += 1;
        // Parse 1-2 digits as centiseconds; ignore any extra digits.
        let mut digits = 0u32;
        while i < data.len() && digits < 2 {
            let b = data[i];
            if b < b'0' || b > b'9' {
                break;
            }
            cs = cs.saturating_mul(10).saturating_add((b - b'0') as u32);
            digits += 1;
            i += 1;
        }
        if digits == 1 {
            cs = cs.saturating_mul(10);
        }
        // Skip remaining digit run, if any.
        while i < data.len() {
            let b = data[i];
            if b < b'0' || b > b'9' {
                break;
            }
            i += 1;
        }
    }

    if i >= data.len() || !is_delim(data[i]) {
        return -7;
    }
    i += 1;

    let start_cs = (hh as i64)
        .saturating_mul(3600)
        .saturating_add(mm as i64 * 60)
        .saturating_add(ss as i64)
        .saturating_mul(100)
        .saturating_add(cs as i64);

    let payload_off = i;
    let payload_len = data.len().saturating_sub(payload_off);

    unsafe {
        *out = FFmpegRsVplayerEvent {
            start_cs,
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
    fn parses_time_with_cs() {
        let txt = b"1:02:03.45:Hello";
        let mut out = FFmpegRsVplayerEvent { start_cs: 0, payload_offset: 0, payload_len: 0 };
        let r = ffmpeg_rs_vplayer_parse_line(txt.as_ptr(), txt.len(), &mut out);
        assert_eq!(r, 0);
        assert_eq!(out.start_cs, (1 * 3600 + 2 * 60 + 3) * 100 + 45);
        assert_eq!(&txt[out.payload_offset..out.payload_offset + out.payload_len], b"Hello");
    }

    #[test]
    fn parses_time_without_cs() {
        let txt = b"0:00:01 Hello";
        let mut out = FFmpegRsVplayerEvent { start_cs: 0, payload_offset: 0, payload_len: 0 };
        let r = ffmpeg_rs_vplayer_parse_line(txt.as_ptr(), txt.len(), &mut out);
        assert_eq!(r, 0);
        assert_eq!(out.start_cs, 100);
        assert_eq!(&txt[out.payload_offset..out.payload_offset + out.payload_len], b"Hello");
    }
}

