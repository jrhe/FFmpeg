#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const TSBASE: i64 = 10_000_000;

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

fn parse_u64_ascii(s: &[u8]) -> Option<(u64, usize)> {
    if s.is_empty() || !is_digit(s[0]) {
        return None;
    }
    let mut i = 0usize;
    let mut v: u64 = 0;
    while i < s.len() && is_digit(s[i]) {
        v = v.saturating_mul(10).saturating_add((s[i] - b'0') as u64);
        i += 1;
    }
    Some((v, i))
}

fn pow10_u64(mut n: usize) -> u64 {
    let mut v: u64 = 1;
    while n > 0 {
        v = v.saturating_mul(10);
        n -= 1;
    }
    v
}

fn sat_add_i64(a: i64, b: i64) -> i64 {
    a.saturating_add(b)
}

fn sat_sub_i64(a: i64, b: i64) -> i64 {
    a.saturating_sub(b)
}

fn parse_one(mut s: &[u8]) -> Option<(i64, &[u8])> {
    s = skip_ws(s);
    let (intval, used_int) = parse_i64_ascii(s)?;

    if intval < i64::MIN / TSBASE || intval > i64::MAX / TSBASE {
        return None;
    }
    let mut v = intval.saturating_mul(TSBASE);
    s = &s[used_int..];

    if s.first().copied() == Some(b'.') {
        s = &s[1..];
        let (mut frac, used_frac) = parse_u64_ascii(s)?;
        // frac must be non-negative by construction
        let frac_digits = used_frac;
        if frac_digits < 7 {
            frac = frac.saturating_mul(pow10_u64(7 - frac_digits));
        } else if frac_digits > 7 {
            frac /= pow10_u64(frac_digits - 7);
        }
        let frac_i64 = if frac > i64::MAX as u64 { i64::MAX } else { frac as i64 };
        if v > 0 {
            v = sat_add_i64(v, frac_i64);
        } else {
            v = sat_sub_i64(v, frac_i64);
        }
        s = &s[used_frac..];
    }
    Some((v, s))
}

fn parse_line(line: &[u8]) -> Option<(i64, i64)> {
    let (a, rest) = parse_one(line)?;
    let (b, _rest2) = parse_one(rest)?;
    Some((a, b))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_mpsub_parse_line(
    line: *const u8,
    line_len: usize,
    out_start: *mut i64,
    out_duration: *mut i64,
) -> c_int {
    if line.is_null() || out_start.is_null() || out_duration.is_null() {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    let (a, b) = match parse_line(buf) {
        Some(v) => v,
        None => return -2,
    };
    unsafe {
        *out_start = a;
        *out_duration = b;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_two_values() {
        let (a, b) = parse_line(b"  1.23 4.5").unwrap();
        assert_eq!(a, 1 * TSBASE + 2_300_000);
        assert_eq!(b, 4 * TSBASE + 5_000_000);
    }
}

