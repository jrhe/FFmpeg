#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFmpegRsWebvttCue {
    pub start_ms: i64,
    pub end_ms: i64,
    pub payload_offset: usize,
    pub payload_len: usize,
    pub identifier_offset: usize,
    pub identifier_len: usize,
    pub settings_offset: usize,
    pub settings_len: usize,
}

#[repr(C)]
pub struct FFmpegRsWebvttParseResult {
    pub n_cues: usize,
}

fn chomp_cr(mut s: &[u8]) -> &[u8] {
    if let Some((&b'\r', rest)) = s.split_last() {
        s = rest;
    }
    s
}

fn is_ws(b: u8) -> bool {
    b == b' ' || b == b'\t'
}

fn parse_u64(mut s: &[u8]) -> Option<(u64, usize)> {
    let mut v: u64 = 0;
    let mut n = 0usize;
    while n < s.len() {
        let b = s[n];
        if b < b'0' || b > b'9' {
            break;
        }
        v = v.saturating_mul(10).saturating_add((b - b'0') as u64);
        n += 1;
    }
    if n == 0 {
        None
    } else {
        Some((v, n))
    }
}

fn parse_ts_ms(s: &[u8]) -> Option<i64> {
    // Supports HH:MM:SS.mmm or MM:SS.mmm
    // Minimal strict parsing.
    let s = s;
    // Split at '.'
    let dot = s.iter().position(|&b| b == b'.')?;
    let (lhs, rhs) = (&s[..dot], &s[dot + 1..]);
    let mut ms = 0u64;
    if rhs.len() < 1 {
        return None;
    }
    let mut frac = 0u64;
    let mut digits = 0usize;
    for &b in rhs.iter().take_while(|&&b| b >= b'0' && b <= b'9') {
        if digits < 3 {
            frac = frac * 10 + (b - b'0') as u64;
            digits += 1;
        }
    }
    while digits < 3 {
        frac *= 10;
        digits += 1;
    }
    ms += frac;

    let mut parts = lhs.split(|&b| b == b':');
    let a = parts.next()?;
    let b = parts.next()?;
    let c = parts.next();
    let (hh, mm, ss) = if let Some(c) = c {
        let (hh, n1) = parse_u64(a)?;
        if n1 != a.len() {
            return None;
        }
        let (mm, n2) = parse_u64(b)?;
        if n2 != b.len() {
            return None;
        }
        let (ss, n3) = parse_u64(c)?;
        if n3 != c.len() {
            return None;
        }
        (hh, mm, ss)
    } else {
        let (mm, n1) = parse_u64(a)?;
        if n1 != a.len() {
            return None;
        }
        let (ss, n2) = parse_u64(b)?;
        if n2 != b.len() {
            return None;
        }
        (0, mm, ss)
    };

    let total_ms = (hh * 3600 + mm * 60 + ss)
        .saturating_mul(1000)
        .saturating_add(ms);
    Some(total_ms as i64)
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_webvtt_parse(
    text: *const u8,
    text_len: usize,
    out: *mut FFmpegRsWebvttParseResult,
    cues: *mut FFmpegRsWebvttCue,
    cues_cap: usize,
) -> c_int {
    if text.is_null() || out.is_null() {
        return -1;
    }
    let data = unsafe { core::slice::from_raw_parts(text, text_len) };
    let mut n_cues = 0usize;

    // Skip UTF-8 BOM if present.
    let mut i = 0usize;
    if text_len >= 3 && data[0..3] == [0xEF, 0xBB, 0xBF] {
        i = 3;
    }

    // Require "WEBVTT" at start.
    if i + 6 > text_len || &data[i..i + 6] != b"WEBVTT" {
        return -2;
    }

    // Split by blank lines into blocks.
    // We'll do a simple scan for blocks separated by two newlines.
    let mut pos = 0usize;
    while pos < text_len {
        // Find next block start: skip leading newlines.
        while pos < text_len && (data[pos] == b'\n' || data[pos] == b'\r') {
            pos += 1;
        }
        if pos >= text_len {
            break;
        }
        // Find block end.
        let mut end = pos;
        while end < text_len {
            if data[end] == b'\n' {
                // check for blank line (\n\n or \n\r\n)
                if end + 1 < text_len && data[end + 1] == b'\n' {
                    break;
                }
                if end + 2 < text_len && data[end + 1] == b'\r' && data[end + 2] == b'\n' {
                    break;
                }
            }
            end += 1;
        }
        let block = &data[pos..end];

        // Header-style blocks to ignore: WEBVTT/STYLE/REGION/NOTE.
        let first_line_end = block.iter().position(|&b| b == b'\n').unwrap_or(block.len());
        let first_line = chomp_cr(&block[..first_line_end]);
        if first_line.starts_with(b"WEBVTT") || first_line == b"STYLE" || first_line == b"REGION" || first_line == b"NOTE" {
            pos = end + 1;
            continue;
        }

        // Parse optional identifier line if the first line does not contain "-->".
        let mut line1 = first_line;
        let mut rest_start = first_line_end;
        if rest_start < block.len() && block[rest_start] == b'\n' {
            rest_start += 1;
        }

        let mut identifier_offset = 0usize;
        let mut identifier_len = 0usize;
        let mut ts_line = line1;
        let mut ts_line_offset = pos;

        if !line1.windows(3).any(|w| w == b"-->") {
            identifier_offset = pos;
            identifier_len = line1.len();
            // Next line is timestamp line.
            let next_end = block[rest_start..].iter().position(|&b| b == b'\n').map(|x| rest_start + x).unwrap_or(block.len());
            ts_line = chomp_cr(&block[rest_start..next_end]);
            ts_line_offset = pos + rest_start;
            rest_start = next_end;
            if rest_start < block.len() && block[rest_start] == b'\n' {
                rest_start += 1;
            }
        }

        // Parse timestamp line: "<ts> --> <ts> [settings]"
        let arrow = ts_line.windows(3).position(|w| w == b"-->");
        let arrow = match arrow {
            Some(a) => a,
            None => {
                pos = end + 1;
                continue;
            }
        };
        let left = chomp_cr(&ts_line[..arrow]);
        let mut right = &ts_line[arrow + 3..];
        while !right.is_empty() && is_ws(right[0]) {
            right = &right[1..];
        }

        // Right side may contain settings after whitespace.
        let right_end_ts = right.iter().position(|&b| is_ws(b)).unwrap_or(right.len());
        let right_ts = &right[..right_end_ts];
        let mut settings = &right[right_end_ts..];
        while !settings.is_empty() && is_ws(settings[0]) {
            settings = &settings[1..];
        }

        let start_ms = match parse_ts_ms(left) {
            Some(v) => v,
            None => {
                pos = end + 1;
                continue;
            }
        };
        let end_ms = match parse_ts_ms(right_ts) {
            Some(v) => v,
            None => {
                pos = end + 1;
                continue;
            }
        };

        // Payload is rest of block after the timestamp line.
        let payload_offset = pos + rest_start;
        let payload_len = if rest_start <= block.len() { block.len() - rest_start } else { 0 };

        let settings_offset = (ts_line_offset + (ts_line.len() - settings.len())) as usize;
        let settings_len = settings.len();

        if n_cues < cues_cap {
            unsafe {
                *cues.add(n_cues) = FFmpegRsWebvttCue {
                    start_ms,
                    end_ms,
                    payload_offset,
                    payload_len,
                    identifier_offset,
                    identifier_len,
                    settings_offset,
                    settings_len,
                };
            }
        }
        n_cues += 1;
        pos = end + 1;
    }

    unsafe {
        (*out).n_cues = n_cues;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_cue() {
        let txt = b"WEBVTT\n\n00:00.000 --> 00:01.000\nHello\n\n";
        let mut out = FFmpegRsWebvttParseResult { n_cues: 0 };
        let mut cues = [FFmpegRsWebvttCue {
            start_ms: 0,
            end_ms: 0,
            payload_offset: 0,
            payload_len: 0,
            identifier_offset: 0,
            identifier_len: 0,
            settings_offset: 0,
            settings_len: 0,
        }; 4];
        let r = ffmpeg_rs_webvtt_parse(txt.as_ptr(), txt.len(), &mut out, cues.as_mut_ptr(), cues.len());
        assert_eq!(r, 0);
        assert_eq!(out.n_cues, 1);
        assert_eq!(cues[0].start_ms, 0);
        assert_eq!(cues[0].end_ms, 1000);
        assert_eq!(&txt[cues[0].payload_offset..cues[0].payload_offset + cues[0].payload_len], b"Hello");
    }
}
