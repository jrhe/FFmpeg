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
    let v_i64 = if v > i64::MAX as u64 { i64::MAX } else { v as i64 };
    Some((sign.saturating_mul(v_i64), i))
}

fn parse_time_to_cs(s: &[u8]) -> Option<i64> {
    let s = skip_ws(s);
    if s.is_empty() {
        return None;
    }

    // Try unsigned time formats first (matches %u patterns).
    let (a, mut i) = parse_u64_ascii(s)?;
    if i < s.len() && s[i] == b':' {
        i += 1;
        let (b, n) = parse_u64_ascii(&s[i..])?;
        i += n;
        if i < s.len() && s[i] == b':' {
            // hh:mm:ss(.ms)?
            i += 1;
            let (c, n) = parse_u64_ascii(&s[i..])?;
            i += n;
            if i < s.len() && s[i] == b'.' {
                i += 1;
                let (ms, _n) = parse_u64_ascii(&s[i..])?;
                return Some(
                    (a as i64)
                        .saturating_mul(3600)
                        .saturating_add((b as i64).saturating_mul(60))
                        .saturating_add(c as i64)
                        .saturating_mul(100)
                        .saturating_add(ms as i64),
                );
            }
            return Some(
                (a as i64)
                    .saturating_mul(3600)
                    .saturating_add((b as i64).saturating_mul(60))
                    .saturating_add(c as i64)
                    .saturating_mul(100),
            );
        }

        // mm:ss(.ms)?
        if i < s.len() && s[i] == b'.' {
            i += 1;
            let (ms, _n) = parse_u64_ascii(&s[i..])?;
            return Some(
                (a as i64)
                    .saturating_mul(60)
                    .saturating_add(b as i64)
                    .saturating_mul(100)
                    .saturating_add(ms as i64),
            );
        }
        return Some(
            (a as i64)
                .saturating_mul(60)
                .saturating_add(b as i64)
                .saturating_mul(100),
        );
    }

    // ss.ms
    if i < s.len() && s[i] == b'.' {
        i += 1;
        let (ms, _n) = parse_u64_ascii(&s[i..])?;
        return Some((a as i64).saturating_mul(100).saturating_add(ms as i64));
    }

    // Fallback: integer seconds * 100 (matches strtoll()*100 path, but signed).
    let (secs, _n) = parse_i64_ascii(s)?;
    Some(secs.saturating_mul(100))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_realtext_read_ts(
    s: *const u8,
    s_len: usize,
    out_cs: *mut i64,
) -> c_int {
    if s.is_null() || out_cs.is_null() {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(s, s_len) };
    let v = match parse_time_to_cs(buf) {
        Some(v) => v,
        None => return -2,
    };
    unsafe {
        *out_cs = v;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hh_mm_ss_ms() {
        assert_eq!(parse_time_to_cs(b"1:2:3.4").unwrap(), 372_304);
    }

    #[test]
    fn parses_mm_ss() {
        assert_eq!(parse_time_to_cs(b"2:03").unwrap(), 12_300);
    }

    #[test]
    fn parses_ss_ms() {
        assert_eq!(parse_time_to_cs(b"9.1").unwrap(), 901);
    }
}
