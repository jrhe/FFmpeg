#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFmpegRsSubripEvent {
    pub start_ms: i64,
    pub duration_ms: i64,
    pub payload_offset: usize,
    pub payload_len: usize,
}

#[repr(C)]
pub struct FFmpegRsSubripParseResult {
    pub n_events: usize,
}

fn chomp_cr(mut s: &[u8]) -> &[u8] {
    if let Some((&b'\r', rest)) = s.split_last() {
        s = rest;
    }
    s
}

fn is_digit(b: u8) -> bool {
    b'0' <= b && b <= b'9'
}

fn parse_u64(s: &[u8]) -> Option<u64> {
    let mut v: u64 = 0;
    let mut any = false;
    for &b in s {
        if !is_digit(b) {
            break;
        }
        any = true;
        v = v.saturating_mul(10).saturating_add((b - b'0') as u64);
    }
    any.then_some(v)
}

fn parse_ts_ms(line: &[u8]) -> Option<i64> {
    // "HH:MM:SS,mmm" or "." as separator
    // Strict fixed-width parsing.
    if line.len() < 12 {
        return None;
    }
    let hh = parse_u64(&line[0..2])?;
    if line[2] != b':' {
        return None;
    }
    let mm = parse_u64(&line[3..5])?;
    if line[5] != b':' {
        return None;
    }
    let ss = parse_u64(&line[6..8])?;
    if line[8] != b',' && line[8] != b'.' {
        return None;
    }
    let ms = parse_u64(&line[9..12])?;
    let total = (hh * 3600 + mm * 60 + ss)
        .saturating_mul(1000)
        .saturating_add(ms);
    Some(total as i64)
}

fn parse_timing_line(line: &[u8]) -> Option<(i64, i64)> {
    // "<ts> --> <ts>" (ignore trailing positions)
    let arrow = line.windows(5).position(|w| w == b" --> ")?;
    let start = parse_ts_ms(&line[..arrow])?;
    let end = parse_ts_ms(&line[arrow + 5..])?;
    Some((start, end.saturating_sub(start)))
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_subrip_parse(
    text: *const u8,
    text_len: usize,
    out: *mut FFmpegRsSubripParseResult,
    events: *mut FFmpegRsSubripEvent,
    events_cap: usize,
) -> c_int {
    if text.is_null() || out.is_null() {
        return -1;
    }
    let data = unsafe { core::slice::from_raw_parts(text, text_len) };
    let mut n_events = 0usize;

    // Split into lines and parse in a simple state machine.
    let mut i = 0usize;
    while i < text_len {
        // skip blank lines
        while i < text_len && (data[i] == b'\n' || data[i] == b'\r') {
            i += 1;
        }
        if i >= text_len {
            break;
        }
        // line 1: optional index
        let l1_end = data[i..].iter().position(|&b| b == b'\n').map(|x| i + x).unwrap_or(text_len);
        let l1 = chomp_cr(&data[i..l1_end]);
        i = if l1_end < text_len { l1_end + 1 } else { text_len };
        if l1.is_empty() {
            continue;
        }

        // line 2: timing
        if i >= text_len {
            break;
        }
        let l2_end = data[i..].iter().position(|&b| b == b'\n').map(|x| i + x).unwrap_or(text_len);
        let l2 = chomp_cr(&data[i..l2_end]);
        let timing = parse_timing_line(l2);
        i = if l2_end < text_len { l2_end + 1 } else { text_len };

        let (start_ms, duration_ms) = match timing {
            Some(v) => v,
            None => {
                // If l1 was not an index, we might have misaligned.
                continue;
            }
        };

        // payload lines until blank line
        let payload_offset = i;
        while i < text_len {
            let nl = data[i..].iter().position(|&b| b == b'\n').map(|x| i + x).unwrap_or(text_len);
            let line = chomp_cr(&data[i..nl]);
            i = if nl < text_len { nl + 1 } else { text_len };
            if line.is_empty() {
                break;
            }
        }
        let payload_len = i.saturating_sub(payload_offset);
        // Trim trailing newlines/CRs from payload.
        let mut pl = payload_len;
        while pl > 0 {
            let b = data[payload_offset + pl - 1];
            if b == b'\n' || b == b'\r' {
                pl -= 1;
            } else {
                break;
            }
        }

        if n_events < events_cap {
            unsafe {
                *events.add(n_events) = FFmpegRsSubripEvent {
                    start_ms,
                    duration_ms,
                    payload_offset,
                    payload_len: pl,
                };
            }
        }
        n_events += 1;
    }

    unsafe {
        (*out).n_events = n_events;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_event() {
        let txt = b"1\n00:00:01,000 --> 00:00:02,500\nHello\n\n";
        let mut out = FFmpegRsSubripParseResult { n_events: 0 };
        let mut evs = [FFmpegRsSubripEvent {
            start_ms: 0,
            duration_ms: 0,
            payload_offset: 0,
            payload_len: 0,
        }; 4];
        let r = ffmpeg_rs_subrip_parse(txt.as_ptr(), txt.len(), &mut out, evs.as_mut_ptr(), evs.len());
        assert_eq!(r, 0);
        assert_eq!(out.n_events, 1);
        assert_eq!(evs[0].start_ms, 1000);
        assert_eq!(evs[0].duration_ms, 1500);
        assert_eq!(&txt[evs[0].payload_offset..evs[0].payload_offset + evs[0].payload_len], b"Hello");
    }
}

