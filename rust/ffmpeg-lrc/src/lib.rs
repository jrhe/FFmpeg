#![no_std]

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const AV_TIME_BASE: i64 = 1_000_000;

fn is_ws(b: u8) -> bool {
    b == b' ' || b == b'\t'
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
        v = v.checked_mul(10)?.checked_add((b - b'0') as u32)?;
        i += 1;
    }
    if !any {
        return None;
    }
    Some((v, i))
}

fn parse_seconds_f64(s: &[u8]) -> Option<(f64, usize)> {
    // Minimal "%lf" for forms like "12", "12.34"
    let mut i = 0usize;
    let (ip, n) = parse_u32_ascii(&s[i..])?;
    i += n;
    let mut v = ip as f64;
    if i < s.len() && s[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < s.len() && is_digit(s[i]) {
            i += 1;
        }
        if i == frac_start {
            return None;
        }
        let mut frac: f64 = 0.0;
        let mut scale: f64 = 1.0;
        for &b in &s[frac_start..i] {
            frac = frac * 10.0 + (b - b'0') as f64;
            scale *= 10.0;
        }
        v += frac / scale;
    }
    Some((v, i))
}

fn count_ts_prefix(line: &[u8]) -> usize {
    let mut offset = 0usize;
    let mut in_brackets: i32 = 0;
    loop {
        if offset >= line.len() {
            break;
        }
        let b = line[offset];
        if b == b' ' || b == b'\t' {
            offset += 1;
        } else if b == b'[' {
            offset += 1;
            in_brackets += 1;
        } else if b == b']' && in_brackets > 0 {
            offset += 1;
            in_brackets -= 1;
        } else if in_brackets > 0
            && (b == b':' || b == b'.' || b == b'-' || is_digit(b))
        {
            offset += 1;
        } else {
            break;
        }
    }
    offset
}

fn read_ts(line: &[u8]) -> Option<(usize, i64)> {
    let orig_len = line.len();
    let s = skip_ws(line);
    let consumed_ws = orig_len - s.len();
    if s.first().copied()? != b'[' {
        return None;
    }
    let mut i = 1usize;
    let mut neg = false;
    if i < s.len() && s[i] == b'-' {
        neg = true;
        i += 1;
    }
    let (mm, n) = parse_u32_ascii(&s[i..])?;
    i += n;
    if i >= s.len() || s[i] != b':' {
        return None;
    }
    i += 1;
    let (ss, n) = parse_seconds_f64(&s[i..])?;
    i += n;
    if ss < 0.0 || ss > 60.0 {
        return None;
    }
    if i >= s.len() || s[i] != b']' {
        return None;
    }
    i += 1; // include ']'

    let start = ((mm as f64) * 60.0 + ss) * (AV_TIME_BASE as f64);
    // Mirror llrint best-effort: round to nearest.
    let mut us = start.round() as i64;
    if neg {
        us = -us;
    }
    Some((consumed_ws + i, us))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_lrc_count_ts_prefix(line: *const u8, line_len: usize) -> usize {
    if line.is_null() {
        return 0;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    count_ts_prefix(buf)
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_lrc_read_ts(line: *const u8, line_len: usize, out_start_us: *mut i64) -> usize {
    if line.is_null() || out_start_us.is_null() {
        return 0;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    let (consumed, start) = match read_ts(buf) {
        Some(v) => v,
        None => return 0,
    };
    unsafe {
        *out_start_us = start;
    }
    consumed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_prefix() {
        assert_eq!(count_ts_prefix(b"[00:01.00][00:02.00] hi"), 21);
    }

    #[test]
    fn parses_ts() {
        let (c, us) = read_ts(b"[01:02.50] hi").unwrap();
        assert_eq!(c, 10);
        assert_eq!(us, 62_500_000);
    }

    #[test]
    fn parses_negative_ts() {
        let (c, us) = read_ts(b"  [-00:00.10]x").unwrap();
        assert_eq!(c, 13);
        assert_eq!(us, -100_000);
    }
}
