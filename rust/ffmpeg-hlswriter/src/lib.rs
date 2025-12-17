#![no_std]

use core::ffi::{c_char, c_int};

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Writes a master playlist header:
//   #EXTM3U\n#EXT-X-VERSION:<version>\n
// Returns number of bytes written (excluding NUL), or <0 on error.
#[no_mangle]
pub extern "C" fn ffmpeg_rs_hls_write_playlist_version(
    dst: *mut c_char,
    dst_len: usize,
    version: c_int,
) -> isize {
    if dst.is_null() || dst_len == 0 {
        return -1;
    }
    // Conservative upper bound.
    // "#EXTM3U\n" (8) + "#EXT-X-VERSION:" (15) + up to 11 digits + "\n" (1)
    // = 35.
    if dst_len < 40 {
        return -2;
    }

    // Manual integer formatting to avoid allocation.
    let mut tmp = [0u8; 12];
    let mut n = version as i64;
    let neg = n < 0;
    if neg {
        n = -n;
    }
    let mut i = 0usize;
    loop {
        tmp[i] = (n % 10) as u8 + b'0';
        i += 1;
        n /= 10;
        if n == 0 {
            break;
        }
        if i >= tmp.len() {
            return -3;
        }
    }

    unsafe {
        let out = core::slice::from_raw_parts_mut(dst as *mut u8, dst_len);
        let mut pos = 0usize;

        for &b in b"#EXTM3U\n#EXT-X-VERSION:" {
            out[pos] = b;
            pos += 1;
        }
        if neg {
            out[pos] = b'-';
            pos += 1;
        }
        while i > 0 {
            i -= 1;
            out[pos] = tmp[i];
            pos += 1;
        }
        out[pos] = b'\n';
        pos += 1;
        // NUL terminate for convenience.
        if pos < dst_len {
            out[pos] = 0;
        }
        pos as isize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_expected() {
        let mut buf = [0i8; 128];
        let n = ffmpeg_rs_hls_write_playlist_version(buf.as_mut_ptr(), buf.len(), 7);
        assert!(n > 0);
        let bytes = unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const u8, n as usize) };
        assert_eq!(bytes, b"#EXTM3U\n#EXT-X-VERSION:7\n");
    }
}
