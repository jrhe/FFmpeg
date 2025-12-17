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

fn parse_u64_ascii(s: &[u8]) -> Option<(u64, usize)> {
    let mut i = 0usize;
    let mut v: u64 = 0;
    let mut any = false;
    while i < s.len() {
        let b = s[i];
        if !is_digit(b) {
            break;
        }
        any = true;
        v = v.saturating_mul(10).saturating_add((b - b'0') as u64);
        i += 1;
    }
    if !any {
        return None;
    }
    Some((v, i))
}

fn skip_ws(mut s: &[u8]) -> &[u8] {
    while let Some((&b, rest)) = s.split_first() {
        if b != b' ' && b != b'\t' && b != b'\r' && b != b'\n' {
            break;
        }
        s = rest;
    }
    s
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
    let (v, n) = parse_u64_ascii(&s[i..])?;
    i += n;
    let v_i64 = if v > i64::MAX as u64 {
        return None;
    } else {
        v as i64
    };
    Some((sign.saturating_mul(v_i64), i))
}

fn abs_i64_to_u128(v: i64) -> u128 {
    let v128 = v as i128;
    if v128 < 0 {
        (-v128) as u128
    } else {
        v128 as u128
    }
}

fn parse_shift_like_c(text: &[u8], timeres: u32) -> i32 {
    let t = skip_ws(text);
    if t.is_empty() {
        return 0;
    }

    // Parse up to 4 signed integers separated by exactly one '.' or ':'.
    let mut vals: [i64; 4] = [0, 0, 0, 0];
    let mut n = 0usize;
    let mut i = 0usize;
    while i < t.len() && n < 4 {
        let (v, used) = match parse_i64_ascii(&t[i..]) {
            Some(v) => v,
            None => break,
        };
        if v < i32::MIN as i64 || v > i32::MAX as i64 {
            return 0;
        }
        vals[n] = v;
        n += 1;
        i += used;
        if i < t.len() && (t[i] == b'.' || t[i] == b':') {
            i += 1;
        } else {
            break;
        }
    }

    if n == 0 {
        return 0;
    }

    // Match C behavior:
    // - sign is driven by a leading '-' or a negative first field
    // - the first field is absolute-valued before shifting
    // - a single field is treated as invalid (cleared to 0)
    let mut sign: i64 = 1;
    let h0_i32 = vals[0] as i32;
    if h0_i32 == i32::MIN {
        return 0;
    }
    let mut h0 = vals[0];
    if t[0] == b'-' || h0 < 0 {
        sign = -1;
        h0 = h0.abs();
    }

    let (h, m, s, d): (i64, i64, i64, i64) = match n {
        1 => (0, 0, 0, 0),
        2 => (0, 0, h0, vals[1]),
        3 => (0, h0, vals[1], vals[2]),
        _ => (h0, vals[1], vals[2], vals[3]),
    };

    let base = h.saturating_mul(3600).saturating_add(m.saturating_mul(60)).saturating_add(s);

    let abs_base = abs_i64_to_u128(base);
    let abs_d = abs_i64_to_u128(d);
    let timeres_u128 = timeres as u128;
    if timeres_u128 == 0 {
        return 0;
    }
    if abs_base > (i64::MAX as u128 - abs_d) / timeres_u128 {
        return 0;
    }

    let ret = sign
        .saturating_mul(
            base.saturating_mul(timeres as i64).saturating_add(d),
        );

    if ret < i32::MIN as i64 || ret > i32::MAX as i64 {
        return 0;
    }
    ret as i32
}

fn parse_timed(buf: &[u8], timeres: u32) -> Option<(i64, i64, usize)> {
    // "hs:ms:ss.fs he:me:se.fe ..."
    let mut i = 0usize;
    let (hs, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    if i >= buf.len() || buf[i] != b':' {
        return None;
    }
    i += 1;
    let (ms, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    if i >= buf.len() || buf[i] != b':' {
        return None;
    }
    i += 1;
    let (ss, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    if i >= buf.len() || buf[i] != b'.' {
        return None;
    }
    i += 1;
    let (fs, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    i = buf.len() - skip_ws(&buf[i..]).len();

    let (he, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    if i >= buf.len() || buf[i] != b':' {
        return None;
    }
    i += 1;
    let (me, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    if i >= buf.len() || buf[i] != b':' {
        return None;
    }
    i += 1;
    let (se, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    if i >= buf.len() || buf[i] != b'.' {
        return None;
    }
    i += 1;
    let (fe, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    i = buf.len() - skip_ws(&buf[i..]).len();

    let ts_start = ((hs * 3600 + ms * 60 + ss) as i64)
        .saturating_mul(timeres as i64)
        .saturating_add(fs as i64);
    let ts_end = ((he * 3600 + me * 60 + se) as i64)
        .saturating_mul(timeres as i64)
        .saturating_add(fe as i64);
    Some((ts_start, ts_end, i))
}

fn parse_at(buf: &[u8]) -> Option<(i64, i64, usize)> {
    // "@start @end ..."
    let mut i = 0usize;
    if i >= buf.len() || buf[i] != b'@' {
        return None;
    }
    i += 1;
    let (ts_start, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    i = buf.len() - skip_ws(&buf[i..]).len();
    if i >= buf.len() || buf[i] != b'@' {
        return None;
    }
    i += 1;
    let (ts_end, n) = parse_u64_ascii(&buf[i..])?;
    i += n;
    i = buf.len() - skip_ws(&buf[i..]).len();
    Some((ts_start as i64, ts_end as i64, i))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_jacosub_parse_shift(
    timeres: u32,
    text: *const u8,
    text_len: usize,
) -> c_int {
    if text.is_null() || timeres == 0 {
        return 0;
    }
    let buf = unsafe { core::slice::from_raw_parts(text, text_len) };
    parse_shift_like_c(buf, timeres) as c_int
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_jacosub_read_ts(
    timeres: u32,
    shift_frames: c_int,
    line: *const u8,
    line_len: usize,
    out_start_cs: *mut i64,
    out_duration_cs: *mut i64,
) -> c_int {
    if line.is_null() || out_start_cs.is_null() || out_duration_cs.is_null() || timeres == 0 {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    let parsed = if let Some(v) = parse_timed(buf, timeres) { Some(v) } else { parse_at(buf) };
    let (ts_start, ts_end, _off) = match parsed {
        Some(v) => v,
        None => return -2,
    };
    let ts_start64 = (ts_start.saturating_add(shift_frames as i64))
        .saturating_mul(100)
        .saturating_div(timeres as i64);
    let ts_end64 = (ts_end.saturating_add(shift_frames as i64))
        .saturating_mul(100)
        .saturating_div(timeres as i64);
    unsafe {
        *out_start_cs = ts_start64;
        *out_duration_cs = ts_end64.saturating_sub(ts_start64);
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_returns_zero_for_empty() {
        assert_eq!(parse_shift_like_c(b"", 30), 0);
    }
}
