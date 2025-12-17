#![no_std]

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn hex_nibble(v: u8) -> u8 {
    if v < 10 {
        b'0' + v
    } else {
        b'A' + (v - 10)
    }
}

#[no_mangle]
pub extern "C" fn ffmpeg_rs_mcc_bytes_to_hex(
    dest: *mut u8,
    dest_cap: usize,
    bytes: *const u8,
    bytes_size: usize,
    use_u_alias: c_int,
) -> c_int {
    if dest.is_null() || bytes.is_null() {
        return -1;
    }
    // Worst-case output is 2 hex chars per input byte plus NUL.
    if dest_cap < 1 + 2 * bytes_size {
        return -2;
    }

    let inb = unsafe { core::slice::from_raw_parts(bytes, bytes_size) };
    let out = unsafe { core::slice::from_raw_parts_mut(dest, dest_cap) };
    let mut oi = 0usize;
    let mut i = 0usize;

    while i < inb.len() {
        match inb[i] {
            0xFA => {
                // Count 0xFA 00 00 triplets and map to 'G'..'O'
                let mut code = b'G';
                let mut j = i;
                while code <= b'O' {
                    if j + 2 >= inb.len() {
                        break;
                    }
                    if inb[j] != 0xFA || inb[j + 1] != 0 || inb[j + 2] != 0 {
                        break;
                    }
                    j += 3;
                    code += 1;
                }
                if code != b'G' {
                    // last successful code is (code-1)
                    out[oi] = code - 1;
                    oi += 1;
                    i = j;
                    continue;
                }
            }
            0xFB | 0xFC | 0xFD => {
                if i + 2 < inb.len() && inb[i + 1] == 0x80 && inb[i + 2] == 0x80 {
                    out[oi] = (inb[i] - 0xFB) + b'P';
                    oi += 1;
                    i += 3;
                    continue;
                }
            }
            0x96 => {
                if i + 1 < inb.len() && inb[i + 1] == 0x69 {
                    out[oi] = b'S';
                    oi += 1;
                    i += 2;
                    continue;
                }
            }
            0x61 => {
                if i + 1 < inb.len() && inb[i + 1] == 0x01 {
                    out[oi] = b'T';
                    oi += 1;
                    i += 2;
                    continue;
                }
            }
            0xE1 => {
                if use_u_alias != 0
                    && i + 3 < inb.len()
                    && inb[i + 1] == 0
                    && inb[i + 2] == 0
                    && inb[i + 3] == 0
                {
                    out[oi] = b'U';
                    oi += 1;
                    i += 4;
                    continue;
                }
            }
            0x00 => {
                out[oi] = b'Z';
                oi += 1;
                i += 1;
                continue;
            }
            _ => {}
        }

        let b = inb[i];
        out[oi] = hex_nibble((b >> 4) & 0xF);
        out[oi + 1] = hex_nibble(b & 0xF);
        oi += 2;
        i += 1;
    }

    out[oi] = 0;
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_hex() {
        let input = [0x12u8, 0xABu8, 0x00u8];
        let mut out = [0u8; 16];
        assert_eq!(
            ffmpeg_rs_mcc_bytes_to_hex(out.as_mut_ptr(), out.len(), input.as_ptr(), input.len(), 0),
            0
        );
        assert_eq!(&out[..5], b"12ABZ");
    }
}

