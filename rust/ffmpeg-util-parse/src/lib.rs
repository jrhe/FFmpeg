#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const EINVAL: c_int = -22;
const ENOSPC: c_int = -28;

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\t' | b'\r')
}

fn is_term(b: u8, term: &[u8]) -> bool {
    if b == 0 {
        return false;
    }
    term.iter().take_while(|&&x| x != 0).any(|&x| x == b)
}

fn write_byte(dst: *mut u8, dst_len: usize, pos: &mut usize, b: u8) {
    if dst_len > 0 && *pos + 1 < dst_len {
        unsafe {
            *dst.add(*pos) = b;
        }
    }
    *pos += 1;
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_util_get_token(
    buf: *const u8,
    buf_len: usize,
    term: *const u8,
    term_len: usize,
    dst: *mut u8,
    dst_len: usize,
    out_advance: *mut usize,
    out_required: *mut usize,
) -> c_int {
    if buf.is_null()
        || term.is_null()
        || out_advance.is_null()
        || out_required.is_null()
        || buf_len == 0
        || term_len == 0
    {
        return EINVAL;
    }
    if dst_len > 0 && dst.is_null() {
        return EINVAL;
    }

    let input = unsafe { core::slice::from_raw_parts(buf, buf_len) };
    let term_bytes = unsafe { core::slice::from_raw_parts(term, term_len) };

    let mut i = 0usize;
    while i < input.len() && is_ws(input[i]) {
        i += 1;
    }

    let mut out_pos = 0usize;
    let mut end_pos = 0usize;

    while i < input.len() {
        let c = input[i];
        if c == 0 || is_term(c, term_bytes) {
            break;
        }
        i += 1;

        if c == b'\\' && i < input.len() && input[i] != 0 {
            let escaped = input[i];
            i += 1;
            write_byte(dst, dst_len, &mut out_pos, escaped);
            end_pos = out_pos;
            continue;
        }

        if c == b'\'' {
            while i < input.len() && input[i] != 0 && input[i] != b'\'' {
                let inner = input[i];
                i += 1;
                write_byte(dst, dst_len, &mut out_pos, inner);
            }
            if i < input.len() && input[i] == b'\'' {
                i += 1;
                end_pos = out_pos;
            }
            continue;
        }

        write_byte(dst, dst_len, &mut out_pos, c);
    }

    let mut trim_pos = out_pos;
    while trim_pos > end_pos {
        let idx = trim_pos - 1;
        let b = if dst_len > 0 && idx < dst_len {
            unsafe { *dst.add(idx) }
        } else {
            break;
        };
        if !is_ws(b) {
            break;
        }
        trim_pos -= 1;
    }

    let required = trim_pos.saturating_add(1);
    unsafe {
        *out_required = required;
        *out_advance = i;
    }

    if dst_len == 0 || required > dst_len {
        return ENOSPC;
    }

    unsafe {
        *dst.add(trim_pos) = 0;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_basic_av_get_token_behavior() {
        let s = b"  a\\ b 'c d' ;rest\0";
        let term = b";\0";
        let mut adv = 0usize;
        let mut req = 0usize;
        let mut buf = [0u8; 64];
        let rc = ffmpeg_rs_util_get_token(
            s.as_ptr(),
            s.len(),
            term.as_ptr(),
            term.len(),
            buf.as_mut_ptr(),
            buf.len(),
            &mut adv,
            &mut req,
        );
        assert_eq!(rc, 0);
        assert_eq!(adv, 15);
        assert_eq!(&buf[..8], b"a b c d\0");
    }
}

