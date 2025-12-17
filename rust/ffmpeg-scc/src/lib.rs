#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
pub struct FFmpegRsSccParseWordsResult {
    pub n_words_total: usize,
    pub n_words_written: usize,
    pub truncated: c_int,
}

fn is_hex(b: u8) -> bool {
    (b'0' <= b && b <= b'9') || (b'a' <= b && b <= b'f') || (b'A' <= b && b <= b'F')
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
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

fn parse_word(token: &[u8]) -> Option<u16> {
    if token.len() < 4 {
        return None;
    }
    let c1 = token[0];
    let c2 = token[1];
    let c3 = token[2];
    let c4 = token[3];
    if !is_hex(c1) || !is_hex(c2) || !is_hex(c3) || !is_hex(c4) {
        return None;
    }
    let hi = (hex_val(c1)? as u16) << 12
        | (hex_val(c2)? as u16) << 8
        | (hex_val(c3)? as u16) << 4
        | (hex_val(c4)? as u16);
    Some(hi)
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_scc_parse_words(
    text: *const u8,
    text_len: usize,
    out: *mut FFmpegRsSccParseWordsResult,
    words: *mut u16,
    words_cap: usize,
) -> c_int {
    if text.is_null() || out.is_null() {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts(text, text_len) };
    let mut s = skip_ws(buf);

    let mut total = 0usize;
    let mut written = 0usize;
    let mut truncated = 0;

    while !s.is_empty() {
        s = skip_ws(s);
        if s.is_empty() {
            break;
        }
        let mut tok_len = 0usize;
        while tok_len < s.len() && !is_ws(s[tok_len]) {
            tok_len += 1;
        }
        if tok_len == 0 {
            break;
        }

        let tok = &s[..tok_len];
        let word = match parse_word(tok) {
            Some(w) => w,
            None => break,
        };

        total += 1;
        if !words.is_null() && written < words_cap {
            unsafe { *words.add(written) = word };
            written += 1;
        } else if words_cap > 0 {
            truncated = 1;
        }

        s = &s[tok_len..];
    }

    unsafe {
        (*out).n_words_total = total;
        (*out).n_words_written = written;
        (*out).truncated = truncated;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_words() {
        let input = b"9420 942c 80ff";
        let mut out = FFmpegRsSccParseWordsResult {
            n_words_total: 0,
            n_words_written: 0,
            truncated: 0,
        };
        let mut words = [0u16; 8];
        assert_eq!(
            ffmpeg_rs_scc_parse_words(input.as_ptr(), input.len(), &mut out, words.as_mut_ptr(), words.len()),
            0
        );
        assert_eq!(out.n_words_total, 3);
        assert_eq!(out.n_words_written, 3);
        assert_eq!(words[0], 0x9420);
        assert_eq!(words[1], 0x942c);
        assert_eq!(words[2], 0x80ff);
    }
}

