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

fn parse_2d(s: &[u8]) -> Option<u32> {
    if s.len() < 2 || !is_digit(s[0]) || !is_digit(s[1]) {
        return None;
    }
    Some(((s[0] - b'0') as u32) * 10 + (s[1] - b'0') as u32)
}

fn parse_ts_cs(line: &[u8], i: &mut usize) -> Option<i64> {
    // "%2d:%2d:%2d:%2d"
    let hh = parse_2d(line.get(*i..*i + 2)?)?;
    *i += 2;
    if line.get(*i)? != &b':' {
        return None;
    }
    *i += 1;
    let mm = parse_2d(line.get(*i..*i + 2)?)?;
    *i += 2;
    if line.get(*i)? != &b':' {
        return None;
    }
    *i += 1;
    let ss = parse_2d(line.get(*i..*i + 2)?)?;
    *i += 2;
    if line.get(*i)? != &b':' {
        return None;
    }
    *i += 1;
    let cc = parse_2d(line.get(*i..*i + 2)?)?;
    *i += 2;
    Some((hh as i64 * 3600 + mm as i64 * 60 + ss as i64) * 100 + cc as i64)
}

fn parse_exact_bytes(line: &[u8], i: &mut usize, pat: &[u8]) -> bool {
    if line.len().saturating_sub(*i) < pat.len() {
        return false;
    }
    if &line[*i..*i + pat.len()] != pat {
        return false;
    }
    *i += pat.len();
    true
}

fn parse_line(line: &[u8]) -> Option<(usize, i64, i32)> {
    let mut i = 0usize;
    let start = parse_ts_cs(line, &mut i)?;
    if !parse_exact_bytes(line, &mut i, b" , ") {
        return None;
    }
    let end = parse_ts_cs(line, &mut i)?;
    if !parse_exact_bytes(line, &mut i, b" , ") {
        return None;
    }
    let dur = end.saturating_sub(start);
    let dur_i32 = i32::try_from(dur).ok()?;
    Some((i, start, dur_i32))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_stl_parse_line(
    line: *const u8,
    line_len: usize,
    out_payload_off: *mut usize,
    out_start_cs: *mut i64,
    out_duration_cs: *mut c_int,
) -> c_int {
    if line.is_null() || out_payload_off.is_null() || out_start_cs.is_null() || out_duration_cs.is_null() {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    let (off, start, dur) = match parse_line(buf) {
        Some(v) => v,
        None => return -2,
    };
    unsafe {
        *out_payload_off = off;
        *out_start_cs = start;
        *out_duration_cs = dur as c_int;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_line() {
        let (off, start, dur) = parse_line(b"00:00:01:02 , 00:00:03:04 , hi").unwrap();
        assert_eq!(start, 102);
        assert_eq!(dur, 202);
        assert_eq!(&b"00:00:01:02 , 00:00:03:04 , hi"[off..], b"hi");
    }
}

