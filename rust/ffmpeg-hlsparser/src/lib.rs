#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFmpegRsHlsSegment {
    pub duration_us: i64,
    pub url_offset: usize,
    pub url_len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFmpegRsHlsVariant {
    pub bandwidth: c_int,
    pub url_offset: usize,
    pub url_len: usize,
}

#[repr(C)]
pub struct FFmpegRsHlsPlaylist {
    pub target_duration_us: i64,
    pub start_seq_no: c_int,
    pub finished: c_int,
    pub n_segments: usize,
    pub n_variants: usize,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum FFmpegRsHlsDemuxEventKind {
    Uri = 0,
    ExtInf = 1,
    StreamInf = 2,
    TargetDuration = 3,
    MediaSequence = 4,
    EndList = 5,
    Unknown = 255,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFmpegRsHlsDemuxEvent {
    pub kind: u32,
    pub line_no: u32,
    pub a_offset: usize,
    pub a_len: usize,
    pub b_offset: usize,
    pub b_len: usize,
    pub i64_a: i64,
    pub i64_b: i64,
}

#[repr(C)]
pub struct FFmpegRsHlsDemuxParseEventsResult {
    pub n_events_total: usize,
    pub n_events_written: usize,
    pub truncated: c_int,
}

fn chomp_cr(mut s: &[u8]) -> &[u8] {
    if let Some((&b'\r', rest)) = s.split_last() {
        s = rest;
    }
    s
}

fn starts_with(bytes: &[u8], prefix: &[u8]) -> bool {
    bytes.len() >= prefix.len() && &bytes[..prefix.len()] == prefix
}

fn parse_i64_ascii(mut s: &[u8]) -> Option<i64> {
    s = s.split(|&b| b == b' ' || b == b'\t').next().unwrap_or(s);
    if s.is_empty() {
        return None;
    }
    let mut neg = false;
    if s[0] == b'-' {
        neg = true;
        s = &s[1..];
    }
    let mut v: i64 = 0;
    let mut any = false;
    for &b in s {
        if b < b'0' || b > b'9' {
            break;
        }
        any = true;
        v = v.saturating_mul(10).saturating_add((b - b'0') as i64);
    }
    if !any {
        return None;
    }
    Some(if neg { -v } else { v })
}

fn parse_f64_seconds_to_us(s: &[u8]) -> Option<i64> {
    // Minimal float parsing: <int>[.<frac>]
    let mut i = 0usize;
    while i < s.len() && (s[i] == b' ' || s[i] == b'\t') {
        i += 1;
    }
    let s = &s[i..];
    if s.is_empty() {
        return None;
    }
    let mut i = 0usize;
    let mut neg = false;
    if s[i] == b'-' {
        neg = true;
        i += 1;
    }
    let mut int: i64 = 0;
    let mut any = false;
    while i < s.len() {
        let b = s[i];
        if b < b'0' || b > b'9' {
            break;
        }
        any = true;
        int = int.saturating_mul(10).saturating_add((b - b'0') as i64);
        i += 1;
    }
    let mut frac: i64 = 0;
    let mut frac_scale: i64 = 1;
    if i < s.len() && s[i] == b'.' {
        i += 1;
        let mut digits = 0;
        while i < s.len() {
            let b = s[i];
            if b < b'0' || b > b'9' {
                break;
            }
            any = true;
            if digits < 6 {
                frac = frac.saturating_mul(10).saturating_add((b - b'0') as i64);
                frac_scale = frac_scale.saturating_mul(10);
                digits += 1;
            }
            i += 1;
        }
        while digits < 6 {
            frac = frac.saturating_mul(10);
            frac_scale = frac_scale.saturating_mul(10);
            digits += 1;
        }
    } else {
        frac_scale = 1_000_000;
    }
    if !any {
        return None;
    }
    let us = int
        .saturating_mul(1_000_000)
        .saturating_add(frac.saturating_mul(1_000_000).saturating_div(frac_scale));
    Some(if neg { -us } else { us })
}

fn parse_bandwidth(attrs: &[u8]) -> Option<i64> {
    // Very small subset: find BANDWIDTH=digits.
    let mut i = 0usize;
    while i + 10 <= attrs.len() {
        if &attrs[i..i + 10] == b"BANDWIDTH=" {
            let rest = &attrs[i + 10..];
            return parse_i64_ascii(rest);
        }
        i += 1;
    }
    None
}

fn push_event(
    out: *mut FFmpegRsHlsDemuxParseEventsResult,
    events: *mut FFmpegRsHlsDemuxEvent,
    cap: usize,
    idx: &mut usize,
    kind: FFmpegRsHlsDemuxEventKind,
    line_no: u32,
    a_offset: usize,
    a_len: usize,
    b_offset: usize,
    b_len: usize,
    i64_a: i64,
    i64_b: i64,
) {
    if out.is_null() {
        return;
    }
    unsafe {
        (*out).n_events_total = (*out).n_events_total.saturating_add(1);
    }
    if events.is_null() || cap == 0 {
        return;
    }
    if *idx >= cap {
        unsafe {
            (*out).truncated = 1;
        }
        return;
    }
    unsafe {
        *events.add(*idx) = FFmpegRsHlsDemuxEvent {
            kind: kind as u32,
            line_no,
            a_offset,
            a_len,
            b_offset,
            b_len,
            i64_a,
            i64_b,
        };
        (*out).n_events_written = (*out).n_events_written.saturating_add(1);
    }
    *idx += 1;
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_hls_demux_parse_events(
    text: *const u8,
    text_len: usize,
    out: *mut FFmpegRsHlsDemuxParseEventsResult,
    events: *mut FFmpegRsHlsDemuxEvent,
    events_cap: usize,
) -> c_int {
    if text.is_null() || out.is_null() {
        return -1;
    }

    unsafe {
        (*out).n_events_total = 0;
        (*out).n_events_written = 0;
        (*out).truncated = 0;
    }

    let data = unsafe { core::slice::from_raw_parts(text, text_len) };

    // Require #EXTM3U first line.
    let mut iter = data.split(|&b| b == b'\n');
    let first = iter.next().unwrap_or(&[]);
    if chomp_cr(first) != b"#EXTM3U" {
        return -2;
    }

    let mut idx = 0usize;
    let mut line_no: u32 = 1;

    for line in iter {
        line_no = line_no.saturating_add(1);
        let line = chomp_cr(line);
        if line.is_empty() {
            continue;
        }

        let offset = (line.as_ptr() as usize).wrapping_sub(data.as_ptr() as usize);
        let len = line.len();

        if starts_with(line, b"#") {
            if starts_with(line, b"#EXTINF:") {
                let v = &line[b"#EXTINF:".len()..];
                let mut dur_us = 0i64;
                let mut title_off = 0usize;
                let mut title_len = 0usize;
                let mut dur_off = 0usize;
                let mut dur_len = 0usize;
                let mut parts = v.splitn(2, |&b| b == b',');
                if let Some(sec_part) = parts.next() {
                    if let Some(us) = parse_f64_seconds_to_us(sec_part) {
                        dur_us = us;
                    }
                    dur_off = (sec_part.as_ptr() as usize).wrapping_sub(data.as_ptr() as usize);
                    dur_len = sec_part.len();
                }
                if let Some(title) = parts.next() {
                    let title = chomp_cr(title);
                    title_off = (title.as_ptr() as usize).wrapping_sub(data.as_ptr() as usize);
                    title_len = title.len();
                }
                push_event(out, events, events_cap, &mut idx, FFmpegRsHlsDemuxEventKind::ExtInf, line_no,
                           dur_off, dur_len, title_off, title_len, dur_us, 0);
            } else if starts_with(line, b"#EXT-X-STREAM-INF:") {
                let attrs = &line[b"#EXT-X-STREAM-INF:".len()..];
                let attrs = chomp_cr(attrs);
                let a_off = (attrs.as_ptr() as usize).wrapping_sub(data.as_ptr() as usize);
                let a_len = attrs.len();
                let bw = parse_bandwidth(attrs).unwrap_or(0);
                push_event(out, events, events_cap, &mut idx, FFmpegRsHlsDemuxEventKind::StreamInf, line_no,
                           a_off, a_len, 0, 0, bw, 0);
            } else if starts_with(line, b"#EXT-X-TARGETDURATION:") {
                let v = &line[b"#EXT-X-TARGETDURATION:".len()..];
                let a_off = (v.as_ptr() as usize).wrapping_sub(data.as_ptr() as usize);
                let a_len = v.len();
                let us = parse_i64_ascii(v).unwrap_or(0).saturating_mul(1_000_000);
                push_event(out, events, events_cap, &mut idx, FFmpegRsHlsDemuxEventKind::TargetDuration, line_no,
                           a_off, a_len, 0, 0, us, 0);
            } else if starts_with(line, b"#EXT-X-MEDIA-SEQUENCE:") {
                let v = &line[b"#EXT-X-MEDIA-SEQUENCE:".len()..];
                let a_off = (v.as_ptr() as usize).wrapping_sub(data.as_ptr() as usize);
                let a_len = v.len();
                let seq = parse_i64_ascii(v).unwrap_or(0);
                push_event(out, events, events_cap, &mut idx, FFmpegRsHlsDemuxEventKind::MediaSequence, line_no,
                           a_off, a_len, 0, 0, seq, 0);
            } else if starts_with(line, b"#EXT-X-ENDLIST") {
                push_event(out, events, events_cap, &mut idx, FFmpegRsHlsDemuxEventKind::EndList, line_no,
                           0, 0, 0, 0, 0, 0);
            } else {
                // Preserve unknown tag line for debugging/fallback decisions.
                push_event(out, events, events_cap, &mut idx, FFmpegRsHlsDemuxEventKind::Unknown, line_no,
                           offset, len, 0, 0, 0, 0);
            }
        } else {
            push_event(out, events, events_cap, &mut idx, FFmpegRsHlsDemuxEventKind::Uri, line_no,
                       offset, len, 0, 0, 0, 0);
        }
    }

    0
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_hls_parse(
    text: *const u8,
    text_len: usize,
    out_playlist: *mut FFmpegRsHlsPlaylist,
    out_segments: *mut FFmpegRsHlsSegment,
    out_segments_cap: usize,
    out_variants: *mut FFmpegRsHlsVariant,
    out_variants_cap: usize,
) -> c_int {
    if text.is_null() || out_playlist.is_null() {
        return -1;
    }
    let data = unsafe { core::slice::from_raw_parts(text, text_len) };

    let mut playlist = FFmpegRsHlsPlaylist {
        target_duration_us: 0,
        start_seq_no: 0,
        finished: 0,
        n_segments: 0,
        n_variants: 0,
    };

    // Require #EXTM3U first line.
    let mut iter = data.split(|&b| b == b'\n');
    let first = iter.next().unwrap_or(&[]);
    if chomp_cr(first) != b"#EXTM3U" {
        return -2;
    }

    let mut pending_seg_dur: Option<i64> = None;
    let mut pending_variant_bw: Option<i32> = None;

    for line in iter {
        let line = chomp_cr(line);
        if line.is_empty() {
            continue;
        }
        if starts_with(line, b"#") {
            if starts_with(line, b"#EXT-X-TARGETDURATION:") {
                let v = &line[b"#EXT-X-TARGETDURATION:".len()..];
                if let Some(sec) = parse_i64_ascii(v) {
                    playlist.target_duration_us = sec.saturating_mul(1_000_000);
                }
            } else if starts_with(line, b"#EXT-X-MEDIA-SEQUENCE:") {
                let v = &line[b"#EXT-X-MEDIA-SEQUENCE:".len()..];
                if let Some(n) = parse_i64_ascii(v) {
                    playlist.start_seq_no = n as c_int;
                }
            } else if starts_with(line, b"#EXT-X-ENDLIST") {
                playlist.finished = 1;
            } else if starts_with(line, b"#EXTINF:") {
                let v = &line[b"#EXTINF:".len()..];
                // Stop at comma if present.
                let v = v.split(|&b| b == b',').next().unwrap_or(v);
                pending_seg_dur = parse_f64_seconds_to_us(v);
            } else if starts_with(line, b"#EXT-X-STREAM-INF:") {
                // Very small subset: BANDWIDTH=...
                let attrs = &line[b"#EXT-X-STREAM-INF:".len()..];
                // Scan for BANDWIDTH=digits
                let mut bw: Option<i32> = None;
                let mut i = 0usize;
                while i + 10 <= attrs.len() {
                    if &attrs[i..i + 10] == b"BANDWIDTH=" {
                        let rest = &attrs[i + 10..];
                        if let Some(v) = parse_i64_ascii(rest) {
                            bw = Some(v as i32);
                        }
                        break;
                    }
                    i += 1;
                }
                pending_variant_bw = bw;
            }
            continue;
        }

        // URI line.
        let offset = (line.as_ptr() as usize).wrapping_sub(data.as_ptr() as usize);
        let len = line.len();
        if let Some(dur) = pending_seg_dur.take() {
            if playlist.n_segments < out_segments_cap {
                unsafe {
                    *out_segments.add(playlist.n_segments) = FFmpegRsHlsSegment {
                        duration_us: dur,
                        url_offset: offset,
                        url_len: len,
                    };
                }
            }
            playlist.n_segments += 1;
        } else if pending_variant_bw.is_some() {
            let bw = pending_variant_bw.take().unwrap_or(0);
            if playlist.n_variants < out_variants_cap {
                unsafe {
                    *out_variants.add(playlist.n_variants) = FFmpegRsHlsVariant {
                        bandwidth: bw as c_int,
                        url_offset: offset,
                        url_len: len,
                    };
                }
            }
            playlist.n_variants += 1;
        }
    }

    unsafe {
        *out_playlist = playlist;
    }
    0
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_hls_parse_strict(
    text: *const u8,
    text_len: usize,
    out_playlist: *mut FFmpegRsHlsPlaylist,
    out_segments: *mut FFmpegRsHlsSegment,
    out_segments_cap: usize,
    out_variants: *mut FFmpegRsHlsVariant,
    out_variants_cap: usize,
) -> c_int {
    if text.is_null() || out_playlist.is_null() {
        return -1;
    }
    let data = unsafe { core::slice::from_raw_parts(text, text_len) };

    let mut playlist = FFmpegRsHlsPlaylist {
        target_duration_us: 0,
        start_seq_no: 0,
        finished: 0,
        n_segments: 0,
        n_variants: 0,
    };

    // Require #EXTM3U first line.
    let mut iter = data.split(|&b| b == b'\n');
    let first = iter.next().unwrap_or(&[]);
    if chomp_cr(first) != b"#EXTM3U" {
        return -2;
    }

    let mut pending_seg_dur: Option<i64> = None;
    let mut pending_variant_bw: Option<i32> = None;

    for line in iter {
        let line = chomp_cr(line);
        if line.is_empty() {
            continue;
        }
        if starts_with(line, b"#") {
            if starts_with(line, b"#EXT-X-TARGETDURATION:") {
                let v = &line[b"#EXT-X-TARGETDURATION:".len()..];
                if let Some(sec) = parse_i64_ascii(v) {
                    playlist.target_duration_us = sec.saturating_mul(1_000_000);
                }
            } else if starts_with(line, b"#EXT-X-MEDIA-SEQUENCE:") {
                let v = &line[b"#EXT-X-MEDIA-SEQUENCE:".len()..];
                if let Some(n) = parse_i64_ascii(v) {
                    playlist.start_seq_no = n as c_int;
                }
            } else if starts_with(line, b"#EXT-X-ENDLIST") {
                playlist.finished = 1;
            } else if starts_with(line, b"#EXTINF:") {
                let v = &line[b"#EXTINF:".len()..];
                // Stop at comma if present.
                let v = v.split(|&b| b == b',').next().unwrap_or(v);
                pending_seg_dur = parse_f64_seconds_to_us(v);
            } else if starts_with(line, b"#EXT-X-STREAM-INF:") {
                // Very small subset: BANDWIDTH=...
                let attrs = &line[b"#EXT-X-STREAM-INF:".len()..];
                // Scan for BANDWIDTH=digits
                let mut bw: Option<i32> = None;
                let mut i = 0usize;
                while i + 10 <= attrs.len() {
                    if &attrs[i..i + 10] == b"BANDWIDTH=" {
                        let rest = &attrs[i + 10..];
                        if let Some(v) = parse_i64_ascii(rest) {
                            bw = Some(v as i32);
                        }
                        break;
                    }
                    i += 1;
                }
                pending_variant_bw = bw;
            } else if starts_with(line, b"#EXT") {
                // Strict mode: any other EXT tag is treated as unsupported.
                return -3;
            }
            continue;
        }

        // URI line.
        let offset = (line.as_ptr() as usize).wrapping_sub(data.as_ptr() as usize);
        let len = line.len();
        if let Some(dur) = pending_seg_dur.take() {
            if playlist.n_segments < out_segments_cap {
                unsafe {
                    *out_segments.add(playlist.n_segments) = FFmpegRsHlsSegment {
                        duration_us: dur,
                        url_offset: offset,
                        url_len: len,
                    };
                }
            }
            playlist.n_segments += 1;
        } else if pending_variant_bw.is_some() {
            let bw = pending_variant_bw.take().unwrap_or(0);
            if playlist.n_variants < out_variants_cap {
                unsafe {
                    *out_variants.add(playlist.n_variants) = FFmpegRsHlsVariant {
                        bandwidth: bw as c_int,
                        url_offset: offset,
                        url_len: len,
                    };
                }
            }
            playlist.n_variants += 1;
        }
    }

    unsafe {
        *out_playlist = playlist;
    }
    0
}

#[cfg(test)]
mod tests_demux_events {
    use super::*;

    #[test]
    fn demux_events_basic() {
        let txt = b"#EXTM3U\n#EXT-X-TARGETDURATION:6\n#EXTINF:2.5,hello\nseg.ts\n#EXT-X-ENDLIST\n";
        let mut out = FFmpegRsHlsDemuxParseEventsResult { n_events_total: 0, n_events_written: 0, truncated: 0 };
        let mut evs = [FFmpegRsHlsDemuxEvent {
            kind: 0, line_no: 0, a_offset: 0, a_len: 0, b_offset: 0, b_len: 0, i64_a: 0, i64_b: 0
        }; 16];

        let r = ffmpeg_rs_hls_demux_parse_events(txt.as_ptr(), txt.len(), &mut out, evs.as_mut_ptr(), evs.len());
        assert_eq!(r, 0);
        assert_eq!(out.truncated, 0);
        assert!(out.n_events_written >= 4);

        assert_eq!(evs[0].kind, FFmpegRsHlsDemuxEventKind::TargetDuration as u32);
        assert_eq!(evs[0].i64_a, 6_000_000);

        assert_eq!(evs[1].kind, FFmpegRsHlsDemuxEventKind::ExtInf as u32);
        assert_eq!(evs[1].i64_a, 2_500_000);
        let title = &txt[evs[1].b_offset..evs[1].b_offset + evs[1].b_len];
        assert_eq!(title, b"hello");

        assert_eq!(evs[2].kind, FFmpegRsHlsDemuxEventKind::Uri as u32);
        let uri = &txt[evs[2].a_offset..evs[2].a_offset + evs[2].a_len];
        assert_eq!(uri, b"seg.ts");

        assert_eq!(evs[3].kind, FFmpegRsHlsDemuxEventKind::EndList as u32);
    }

    #[test]
    fn demux_events_size_only() {
        let txt = b"#EXTM3U\n#EXT-X-ENDLIST\n";
        let mut out = FFmpegRsHlsDemuxParseEventsResult { n_events_total: 0, n_events_written: 0, truncated: 0 };
        let r = ffmpeg_rs_hls_demux_parse_events(txt.as_ptr(), txt.len(), &mut out, core::ptr::null_mut(), 0);
        assert_eq!(r, 0);
        assert_eq!(out.n_events_total, 1);
        assert_eq!(out.n_events_written, 0);
        assert_eq!(out.truncated, 0);
    }
}

#[cfg(test)]
mod tests_hls_parse {
    use super::*;

    #[test]
    fn parses_segments_and_variants() {
        let text = b"#EXTM3U\n#EXT-X-TARGETDURATION:10\n#EXTINF:9.1,\nseg0.ts\n#EXT-X-STREAM-INF:BANDWIDTH=12345\nlow.m3u8\n#EXT-X-ENDLIST\n";
        let mut pl = FFmpegRsHlsPlaylist {
            target_duration_us: 0,
            start_seq_no: 0,
            finished: 0,
            n_segments: 0,
            n_variants: 0,
        };
        let mut segs = [FFmpegRsHlsSegment {
            duration_us: 0,
            url_offset: 0,
            url_len: 0,
        }; 4];
        let mut vars = [FFmpegRsHlsVariant {
            bandwidth: 0,
            url_offset: 0,
            url_len: 0,
        }; 4];
        let r = ffmpeg_rs_hls_parse(
            text.as_ptr(),
            text.len(),
            &mut pl,
            segs.as_mut_ptr(),
            segs.len(),
            vars.as_mut_ptr(),
            vars.len(),
        );
        assert_eq!(r, 0);
        assert_eq!(pl.target_duration_us, 10_000_000);
        assert_eq!(pl.finished, 1);
        assert_eq!(pl.n_segments, 1);
        assert_eq!(pl.n_variants, 1);
        assert_eq!(segs[0].duration_us, 9_100_000);
        assert_eq!(&text[segs[0].url_offset..segs[0].url_offset + segs[0].url_len], b"seg0.ts");
        assert_eq!(vars[0].bandwidth, 12345);
        assert_eq!(&text[vars[0].url_offset..vars[0].url_offset + vars[0].url_len], b"low.m3u8");
    }

    #[test]
    fn strict_rejects_unknown_tags() {
        let text = b"#EXTM3U\n#EXT-X-UNKNOWN:1\nseg0.ts\n";
        let mut pl = FFmpegRsHlsPlaylist {
            target_duration_us: 0,
            start_seq_no: 0,
            finished: 0,
            n_segments: 0,
            n_variants: 0,
        };
        let r = ffmpeg_rs_hls_parse_strict(
            text.as_ptr(),
            text.len(),
            &mut pl,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        assert_eq!(r, -3);
    }
}
