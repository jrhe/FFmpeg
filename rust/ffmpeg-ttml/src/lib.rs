#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
pub struct FFmpegRsTtmlExtradataParseResult {
    pub is_paragraph_mode: c_int,
    pub is_default: c_int,
    pub tt_params_offset: usize,
    pub pre_body_offset: usize,
}

const SIG: &[u8] = b"lavc-ttmlenc";

fn find_nul(buf: &[u8]) -> Option<usize> {
    for (i, &b) in buf.iter().enumerate() {
        if b == 0 {
            return Some(i);
        }
    }
    None
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_ttml_parse_extradata(
    extradata: *const u8,
    extradata_len: usize,
    out: *mut FFmpegRsTtmlExtradataParseResult,
) -> c_int {
    if extradata.is_null() || out.is_null() {
        return -1;
    }
    let data = unsafe { core::slice::from_raw_parts(extradata, extradata_len) };
    if data.len() < SIG.len() || &data[..SIG.len()] != SIG {
        return -2;
    }

    let mut r = FFmpegRsTtmlExtradataParseResult {
        is_paragraph_mode: 1,
        is_default: 0,
        tt_params_offset: 0,
        pre_body_offset: 0,
    };

    let mut rem = &data[SIG.len()..];
    if rem.is_empty() {
        r.is_default = 1;
        unsafe { *out = r };
        return 0;
    }

    // Expect: tt_element_params\0pre_body_elements\0 (both must be NUL-terminated).
    let off_tt = SIG.len();
    let tt_nul = match find_nul(rem) {
        Some(i) => i,
        None => return -3,
    };
    if tt_nul == 0 {
        return -4;
    }
    rem = &rem[tt_nul + 1..];
    if rem.is_empty() {
        return -5;
    }
    let off_pre = off_tt + tt_nul + 1;
    let pre_nul = match find_nul(rem) {
        Some(i) => i,
        None => return -6,
    };
    if pre_nul == 0 {
        return -7;
    }

    r.tt_params_offset = off_tt;
    r.pre_body_offset = off_pre;

    unsafe { *out = r };
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default() {
        let ed = b"lavc-ttmlenc";
        let mut out = FFmpegRsTtmlExtradataParseResult {
            is_paragraph_mode: 0,
            is_default: 0,
            tt_params_offset: 0,
            pre_body_offset: 0,
        };
        let r = ffmpeg_rs_ttml_parse_extradata(ed.as_ptr(), ed.len(), &mut out);
        assert_eq!(r, 0);
        assert_eq!(out.is_default, 1);
    }

    #[test]
    fn parses_custom() {
        let ed = b"lavc-ttmlencAAA\0BBB\0";
        let mut out = FFmpegRsTtmlExtradataParseResult {
            is_paragraph_mode: 0,
            is_default: 0,
            tt_params_offset: 0,
            pre_body_offset: 0,
        };
        let r = ffmpeg_rs_ttml_parse_extradata(ed.as_ptr(), ed.len(), &mut out);
        assert_eq!(r, 0);
        assert_eq!(&ed[out.tt_params_offset..out.tt_params_offset + 3], b"AAA");
        assert_eq!(&ed[out.pre_body_offset..out.pre_body_offset + 3], b"BBB");
    }
}

