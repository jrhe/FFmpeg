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

fn is_ws(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\r' || b == b'\n'
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

fn parse_u32_ascii(s: &[u8]) -> Option<(u32, usize)> {
    let mut i = 0usize;
    let mut v: u32 = 0;
    let mut any = false;
    while i < s.len() {
        let b = s[i];
        if !is_digit(b) {
            break;
        }
        any = true;
        v = v
            .checked_mul(10)?
            .checked_add((b - b'0') as u32)?;
        i += 1;
    }
    if !any {
        return None;
    }
    Some((v, i))
}

fn parse_i32_ascii(s: &[u8]) -> Option<(i32, usize)> {
    if s.is_empty() {
        return None;
    }
    let mut i = 0usize;
    let mut sign: i32 = 1;
    if s[i] == b'+' {
        i += 1;
    } else if s[i] == b'-' {
        sign = -1;
        i += 1;
    }
    let (v, n) = parse_u32_ascii(&s[i..])?;
    let v_i32 = i32::try_from(v).ok()?;
    i += n;
    Some((sign.saturating_mul(v_i32), i))
}

fn get_multiplier(n_digits: usize) -> Option<i64> {
    match n_digits {
        1 => Some(100),
        2 => Some(10),
        3 => Some(1),
        _ => None,
    }
}

fn parse_time_hhmmss_ms(line: &[u8]) -> Option<(i64, i64, i64, i64, usize)> {
    // Matches: "%u:%u:%u.%u" where the fractional part has 1..=3 digits.
    // Returns (hh, mm, ss, ms_scaled, bytes_used)
    let mut i = 0usize;
    let (hh, n) = parse_u32_ascii(&line[i..])?;
    i += n;
    if i >= line.len() || line[i] != b':' {
        return None;
    }
    i += 1;
    let (mm, n) = parse_u32_ascii(&line[i..])?;
    i += n;
    if i >= line.len() || line[i] != b':' {
        return None;
    }
    i += 1;
    let (ss, n) = parse_u32_ascii(&line[i..])?;
    i += n;
    if i >= line.len() || line[i] != b'.' {
        return None;
    }
    i += 1;
    let ms_start = i;
    let (ms, n) = parse_u32_ascii(&line[i..])?;
    i += n;
    let n_digits = i - ms_start;
    let mult = get_multiplier(n_digits)?;

    let hh64 = i64::from(hh);
    let mm64 = i64::from(mm);
    let ss64 = i64::from(ss);
    let ms64 = i64::from(ms);
    Some((hh64, mm64, ss64, ms64.saturating_mul(mult), i))
}

fn parse_subviewer_ts(line: &[u8]) -> Option<(i64, i32)> {
    // "%u:%u:%u.%u,%u:%u:%u.%u"
    let line = skip_ws(line);
    let (hh1, mm1, ss1, ms1_scaled, mut i) = parse_time_hhmmss_ms(line)?;
    if i >= line.len() || line[i] != b',' {
        return None;
    }
    i += 1;
    let (hh2, mm2, ss2, ms2_scaled, _j) = parse_time_hhmmss_ms(&line[i..])?;

    let start_ms = (hh1.saturating_mul(3600)
        .saturating_add(mm1.saturating_mul(60))
        .saturating_add(ss1))
        .saturating_mul(1000)
        .saturating_add(ms1_scaled);
    let end_ms = (hh2.saturating_mul(3600)
        .saturating_add(mm2.saturating_mul(60))
        .saturating_add(ss2))
        .saturating_mul(1000)
        .saturating_add(ms2_scaled);

    let duration_ms = end_ms.saturating_sub(start_ms);
    let dur_i32 = i32::try_from(duration_ms).ok()?;
    Some((start_ms, dur_i32))
}

fn parse_subviewer1_tag(line: &[u8]) -> Option<(i32, i32, i32)> {
    // sscanf("[%d:%d:%d]")
    let line = skip_ws(line);
    let mut i = 0usize;
    if i >= line.len() || line[i] != b'[' {
        return None;
    }
    i += 1;
    let (hh, n) = parse_i32_ascii(&line[i..])?;
    i += n;
    if i >= line.len() || line[i] != b':' {
        return None;
    }
    i += 1;
    let (mm, n) = parse_i32_ascii(&line[i..])?;
    i += n;
    if i >= line.len() || line[i] != b':' {
        return None;
    }
    i += 1;
    let (ss, n) = parse_i32_ascii(&line[i..])?;
    i += n;
    if i >= line.len() || line[i] != b']' {
        return None;
    }
    Some((hh, mm, ss))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_subviewer_read_ts(
    line: *const u8,
    line_len: usize,
    out_start_ms: *mut i64,
    out_duration_ms: *mut c_int,
) -> c_int {
    if line.is_null() || out_start_ms.is_null() || out_duration_ms.is_null() {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    let (start, dur) = match parse_subviewer_ts(buf) {
        Some(v) => v,
        None => return -2,
    };
    unsafe {
        *out_start_ms = start;
        *out_duration_ms = dur as c_int;
    }
    0
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_subviewer1_parse_time(
    line: *const u8,
    line_len: usize,
    out_hh: *mut c_int,
    out_mm: *mut c_int,
    out_ss: *mut c_int,
) -> c_int {
    if line.is_null() || out_hh.is_null() || out_mm.is_null() || out_ss.is_null() {
        return 0;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    let (hh, mm, ss) = match parse_subviewer1_tag(buf) {
        Some(v) => v,
        None => return 0,
    };
    unsafe {
        *out_hh = hh as c_int;
        *out_mm = mm as c_int;
        *out_ss = ss as c_int;
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subviewer_ts_parses_1_digit_fraction() {
        let (s, d) = parse_subviewer_ts(b"00:00:01.2,00:00:01.3").unwrap();
        assert_eq!(s, 1200);
        assert_eq!(d, 100);
    }

    #[test]
    fn subviewer_ts_parses_3_digit_fraction() {
        let (s, d) = parse_subviewer_ts(b"00:00:01.234,00:00:02.000").unwrap();
        assert_eq!(s, 1234);
        assert_eq!(d, 766);
    }

    #[test]
    fn subviewer1_tag_parses() {
        assert_eq!(parse_subviewer1_tag(b"[1:2:3]"), Some((1, 2, 3)));
    }
}
