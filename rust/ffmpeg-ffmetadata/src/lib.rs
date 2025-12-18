#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const EINVAL: c_int = -22;
const ENOSPC: c_int = -28;

#[repr(C)]
pub struct FFMpegRsFFMetaSplit {
    eq_offset: usize,
    key_escaped_len: usize,
    value_escaped_len: usize,
    key_unescaped_len: usize,
    value_unescaped_len: usize,
}

fn unescaped_len(src: &[u8]) -> usize {
    let mut i = 0usize;
    let mut out = 0usize;
    while i < src.len() {
        if src[i] == 0 {
            break;
        }
        if src[i] == b'\\' {
            i += 1;
            if i >= src.len() || src[i] == 0 {
                break;
            }
        }
        out += 1;
        i += 1;
    }
    out
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_ffmetadata_split_kv(
    line: *const u8,
    line_len: usize,
    out: *mut FFMpegRsFFMetaSplit,
) -> c_int {
    if line.is_null() || out.is_null() || line_len == 0 {
        return EINVAL;
    }
    let bytes = unsafe { core::slice::from_raw_parts(line, line_len) };
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == 0 {
            break;
        }
        if b == b'=' {
            unsafe {
                let key = &bytes[..i];
                let value = &bytes[i + 1..];
                *out = FFMpegRsFFMetaSplit {
                    eq_offset: i,
                    key_escaped_len: key.iter().position(|&c| c == 0).unwrap_or(key.len()),
                    value_escaped_len: value.iter().position(|&c| c == 0).unwrap_or(value.len()),
                    key_unescaped_len: unescaped_len(key),
                    value_unescaped_len: unescaped_len(value),
                };
            }
            return 0;
        }
        if b == b'\\' {
            i += 1;
            if i >= bytes.len() || bytes[i] == 0 {
                break;
            }
        }
        i += 1;
    }
    1
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_ffmetadata_unescape(
    dst: *mut u8,
    dst_len: usize,
    src: *const u8,
    src_len: usize,
    out_written: *mut usize,
) -> c_int {
    if src.is_null() || out_written.is_null() {
        return EINVAL;
    }
    if dst_len > 0 && dst.is_null() {
        return EINVAL;
    }

    let input = unsafe { core::slice::from_raw_parts(src, src_len) };
    let mut i = 0usize;
    let mut o = 0usize;

    while i < input.len() {
        let b = input[i];
        if b == 0 {
            break;
        }
        if b == b'\\' {
            i += 1;
            if i >= input.len() || input[i] == 0 {
                break;
            }
        }
        if dst_len > 0 && o + 1 < dst_len {
            unsafe { *dst.add(o) = input[i] };
        }
        o += 1;
        i += 1;
    }

    unsafe {
        *out_written = o;
    }

    if dst_len == 0 || o + 1 > dst_len {
        if dst_len > 0 {
            unsafe { *dst.add(dst_len - 1) = 0 };
        }
        return ENOSPC;
    }
    unsafe { *dst.add(o) = 0 };
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_skips_escaped_equals() {
        let s = b"key\\=part=val\\=x\0";
        let mut out = FFMpegRsFFMetaSplit {
            eq_offset: 0,
            key_escaped_len: 0,
            value_escaped_len: 0,
            key_unescaped_len: 0,
            value_unescaped_len: 0,
        };
        let rc = ffmpeg_rs_ffmetadata_split_kv(s.as_ptr(), s.len(), &mut out);
        assert_eq!(rc, 0);
        assert_eq!(out.eq_offset, 9);
    }

    #[test]
    fn unescape_removes_backslashes() {
        let s = b"a\\=b\\c\0";
        let mut out = [0u8; 8];
        let mut written = 0usize;
        let rc = ffmpeg_rs_ffmetadata_unescape(out.as_mut_ptr(), out.len(), s.as_ptr(), s.len(), &mut written);
        assert_eq!(rc, 0);
        assert_eq!(written, 4);
        assert_eq!(&out[..5], b"a=bc\0");
    }
}

