#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const ID3V2_HEADER_SIZE: i32 = 10;

#[no_mangle]
pub extern "C" fn ffmpeg_rs_id3v2_tag_len(buf: *const u8, buf_len: usize) -> c_int {
    if buf.is_null() || buf_len < 10 {
        return 0;
    }
    let b = unsafe { core::slice::from_raw_parts(buf, 10) };
    let len = ((b[6] & 0x7f) as i32) << 21
        | ((b[7] & 0x7f) as i32) << 14
        | ((b[8] & 0x7f) as i32) << 7
        | ((b[9] & 0x7f) as i32);
    let mut total = len.saturating_add(ID3V2_HEADER_SIZE);
    if (b[5] & 0x10) != 0 {
        total = total.saturating_add(ID3V2_HEADER_SIZE);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_length_with_ext_header_flag() {
        let mut hdr = [0u8; 10];
        hdr[5] = 0x10;
        hdr[9] = 1;
        assert_eq!(ffmpeg_rs_id3v2_tag_len(hdr.as_ptr(), hdr.len()), 21);
    }
}

