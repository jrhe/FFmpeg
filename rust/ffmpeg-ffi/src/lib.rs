#![no_std]

// `staticlib` + `no_std` requires a panic handler, but tests link `std` which
// already provides one.
#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::ffi::{c_char, c_int, c_void};

#[no_mangle]
pub extern "C" fn ffmpeg_rs_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonzero() {
        assert!(ffmpeg_rs_version() > 0);
    }
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_memchr(haystack: *const c_void, len: usize, needle: u8) -> *const c_void {
    if haystack.is_null() {
        return core::ptr::null();
    }
    let bytes = unsafe { core::slice::from_raw_parts(haystack as *const u8, len) };
    match bytes.iter().position(|&b| b == needle) {
        Some(i) => unsafe { (haystack as *const u8).add(i) as *const c_void },
        None => core::ptr::null(),
    }
}

// Example ABI template: C passes in output buffer + length, Rust writes into it.
// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn ffmpeg_rs_copy_str(dst: *mut c_char, dst_len: usize, src: *const c_char) -> c_int {
    if dst.is_null() || src.is_null() || dst_len == 0 {
        return -1;
    }
    let mut i = 0usize;
    unsafe {
        while i + 1 < dst_len {
            let b = *src.add(i) as u8;
            *dst.add(i) = *src.add(i);
            i += 1;
            if b == 0 {
                return 0;
            }
        }
        *dst.add(dst_len - 1) = 0;
    }
    0
}
