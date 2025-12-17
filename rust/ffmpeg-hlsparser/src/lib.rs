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

#[cfg(test)]
mod tests {
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
}
