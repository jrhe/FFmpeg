#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
pub struct FFmpegRsAssDialogueParseResult {
    pub start_cs: i64,
    pub duration_cs: c_int,
    pub layer: c_int,
    pub rest_off: usize,
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
    if i >= s.len() || !is_digit(s[i]) {
        return None;
    }
    let mut v: i32 = 0;
    while i < s.len() && is_digit(s[i]) {
        v = v.saturating_mul(10).saturating_add((s[i] - b'0') as i32);
        i += 1;
    }
    Some((sign.saturating_mul(v), i))
}

fn parse_layer_like_atoi(after_dialogue_colon_space: &[u8]) -> i32 {
    // Mimic atoi: parse optional sign + decimal digits from start, else 0.
    let s = skip_ws(after_dialogue_colon_space);
    match parse_i32_ascii(s) {
        Some((v, _)) => v,
        None => 0,
    }
}

fn parse_time_cs(s: &[u8]) -> Option<(i64, usize)> {
    // "%d:%d:%d%*c%d" (separator between seconds and centis is any single char)
    let mut i = 0usize;
    let (hh, n) = parse_i32_ascii(&s[i..])?;
    i += n;
    if i >= s.len() || s[i] != b':' {
        return None;
    }
    i += 1;
    let (mm, n) = parse_i32_ascii(&s[i..])?;
    i += n;
    if i >= s.len() || s[i] != b':' {
        return None;
    }
    i += 1;
    let (ss, n) = parse_i32_ascii(&s[i..])?;
    i += n;
    if i >= s.len() {
        return None;
    }
    // consume any single separator character ('.' typically)
    i += 1;
    let (cs, n) = parse_i32_ascii(&s[i..])?;
    i += n;

    let hh64 = hh as i64;
    let mm64 = mm as i64;
    let ss64 = ss as i64;
    let cs64 = cs as i64;
    Some(((hh64 * 3600 + mm64 * 60 + ss64) * 100 + cs64, i))
}

fn parse_dialogue(line: &[u8]) -> Option<FFmpegRsAssDialogueParseResult> {
    let prefix = b"Dialogue:";
    if line.len() < prefix.len() || &line[..prefix.len()] != prefix {
        return None;
    }
    // C uses atoi(p + 10); p points at the full line.
    // Index 10 is "Dialogue: " (9 + space).
    let layer = if line.len() > 10 {
        parse_layer_like_atoi(&line[10..])
    } else {
        0
    };

    // Find first comma after "Dialogue:"; sscanf skips first field up to comma.
    let mut i = prefix.len();
    // optional space
    if i < line.len() && line[i] == b' ' {
        i += 1;
    }
    // skip to first comma
    while i < line.len() && line[i] != b',' {
        i += 1;
    }
    if i >= line.len() || line[i] != b',' {
        return None;
    }
    i += 1;

    let (start_cs, used) = parse_time_cs(&line[i..])?;
    i += used;
    if i >= line.len() || line[i] != b',' {
        return None;
    }
    i += 1;

    let (end_cs, used) = parse_time_cs(&line[i..])?;
    i += used;
    if i >= line.len() || line[i] != b',' {
        return None;
    }
    i += 1;

    let dur = end_cs.saturating_sub(start_cs);
    if dur < i32::MIN as i64 || dur > i32::MAX as i64 {
        return None;
    }
    Some(FFmpegRsAssDialogueParseResult {
        start_cs,
        duration_cs: dur as c_int,
        layer: layer as c_int,
        rest_off: i,
    })
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_ass_parse_dialogue(
    line: *const u8,
    line_len: usize,
    out: *mut FFmpegRsAssDialogueParseResult,
) -> c_int {
    if line.is_null() || out.is_null() {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(line, line_len) };
    let r = match parse_dialogue(buf) {
        Some(v) => v,
        None => return -2,
    };
    unsafe {
        *out = r;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_dialogue() {
        let line = b"Dialogue: 0,0:00:01.23,0:00:02.34,Default,,0,0,0,,hi";
        let r = parse_dialogue(line).unwrap();
        assert_eq!(r.layer, 0);
        assert_eq!(r.start_cs, 123);
        assert_eq!(r.duration_cs, 111);
        assert_eq!(&line[r.rest_off..], b"Default,,0,0,0,,hi");
    }

    #[test]
    fn parses_marked_as_layer_zero() {
        let line = b"Dialogue: Marked=1,0:00:00.00,0:00:00.10,foo";
        let r = parse_dialogue(line).unwrap();
        assert_eq!(r.layer, 0);
        assert_eq!(r.start_cs, 0);
        assert_eq!(r.duration_cs, 10);
    }
}

