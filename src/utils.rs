use std::mem;

use anyhow::bail;
use chrono::{DateTime, Local, NaiveTime};
use libc::EINVAL;

pub fn copy_bytes(dest: &mut [u8], dest_offset: i32, src: &[u8], src_offset: usize, count: i32) {
    for i in 0..count {
        dest[dest_offset + i] = src[src_offset + i]
    }
}

pub fn set_bytes(dest: &mut [u8], offset: usize, val: u8, count: usize) {
    let real_count = usize::min(dest.len() - offset, count);
    for i in 0..real_count {
        dest[i] = val;
    }
}

pub fn cmp_bytes(a: &[u8], offset_a: usize, b: &[u8], offset_b: usize, count: usize) -> u8 {
    let mut real_count = usize::min(a.len() - offset_a, b.len() - offset_b);
    real_count = usize::min(real_count, count);
    for i in 0..real_count {
        if a[offset_a + i] != b[offset_b + i] {
            return 1;
        }
    }
    return 0;
}

pub fn put_u8(dest: &mut [u8], dest_offset: usize, src: u8) {
    dest[dest_offset] = src;
}

pub fn get_u8(src: &[u8], src_offset: usize) -> u8 {
    src[src_offset]
}

pub fn put_u16(dest: &mut [u8], dest_offset: usize, src: u16) {
    let src_bytes: [u8; 2] = src.to_le_bytes();
    for i in 0..mem::size_of::<src> {
        dest[dest_offset + i] = src_bytes[i]
    }
}

pub fn get_u16(src: &[u8], src_offset: usize) -> u16 {
    let out_bytes: [u8; 2] = [0; 2];
    out_bytes.clone_from_slice(src + src_offset);
    u16::from_le_bytes(out_bytes)
}

pub fn put_u32(dest: &mut [u8], dest_offset: usize, src: u32) {
    let src_bytes: [u8; 4] = src.to_le_bytes();
    for i in 0..mem::size_of::<src>() {
        dest[dest_offset + i] = src_bytes[i]
    }
}

pub fn get_u32(src: &[u8], src_offset: usize) -> u32 {
    let out_bytes: [u8; 4] = [0; 4];
    out_bytes.clone_from_slice(src + src_offset);
    u32::from_le_bytes(out_bytes)
}

pub fn put_u64(dest: &mut [u8], dest_offset: usize, src: u64) {
    let src_bytes: [u8; 8] = src.to_le_bytes();
    for i in 0..mem::size_of::<src>() {
        dest[dest_offset + i] = src_bytes[i]
    }
}

pub fn get_u64(src: &[u8], src_offset: usize) -> u64 {
    let out_bytes: [u8; 8] = [0; 8];
    out_bytes.clone_from_slice(src + src_offset);
    u64::from_le_bytes(out_bytes)
}

pub fn zmq_stopwatch_start() -> NaiveTime {
    // u64 *watch = static_cast<u64 *> (malloc (mem::size_of::<u64>()));
    // alloc_assert (watch);
    // *watch = clock_t::now_us ();
    // return  (watch);
    let dt: DateTime<Local> = Local::now();
    dt.time()
}

pub fn zmq_stopwatch_intermediate(watch: &NaiveTime) -> u64 {
    let now: DateTime<Local> = Local::now();
    let diff = now.time() - watch;
    // const u64 end = clock_t::now_us ();
    // const u64 start = *static_cast<u64 *> (watch_);
    // return static_cast<unsigned long> (end - start);
    diff.into()
}

pub fn zmq_stopwatch_stop(watch: &NaiveTime) -> u64 {
    let duration = zmq_stopwatch_intermediate(watch);
    duration
}

//  Maps base 256 to base 85
pub const encoder: &'static str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

//  Maps base 85 to base 256
//  We chop off lower 32 and higher 128 ranges
//  0xFF denotes invalid characters within this range
pub const decoder: [u8; 96] = [
    0xFF, 0x44, 0xFF, 0x54, 0x53, 0x52, 0x48, 0xFF, 0x4B, 0x4C, 0x46, 0x41, 0xFF, 0x3F, 0x3E, 0x45,
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x40, 0xFF, 0x49, 0x42, 0x4A, 0x47,
    0x51, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32,
    0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x4D, 0xFF, 0x4E, 0x43, 0xFF,
    0xFF, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
    0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x4F, 0xFF, 0x50, 0xFF, 0xFF,
];

pub fn zmq_z85_encode(dest_: &mut [u8], data: &[u8], size: usize) -> Option<String> {
    if size % 4 != 0 {
        errno = EINVAL;
        return None;
    }
    let mut char_nbr = 0;
    let mut byte_nbr = 0;
    let mut value = 0;
    while byte_nbr < size {
        //  Accumulate value in base 256 (binary)
        value = value * 256 + data[byte_nbr += 1];
        if byte_nbr % 4 == 0 {
            //  Output value in base 85
            let mut divisor = 85 * 85 * 85 * 85;
            while (divisor) {
                dest_[char_nbr += 1] = encoder[value / divisor % 85];
                divisor /= 85;
            }
            value = 0;
        }
    }
    assert(char_nbr == size * 5 / 4);
    dest_[char_nbr] = 0;
    return Some(String::from_utf8_lossy(dest_).into());
}

pub fn zmq_z85_decode(dest_: &mut [u8], string_: &str) -> anyhow::Result<[u8]> {
    let mut byte_nbr = 0;
    let mutchar_nbr = 0;
    let mut value = 0;
    // size_t src_len = strlen (string_);
    let src_len = string_.len();

    if (src_len < 5 || src_len % 5 != 0) {
        bail!("invalid source length: {}", src_len);
    }

    while (string_[char_nbr]) {
        //  Accumulate value in base 85
        if (UINT32_MAX / 85 < value) {
            //  Invalid z85 encoding, represented value exceeds 0xffffffff
            // goto error_inval;
            bail!("Invalid z85 encoding, represented value exceeds 0xffffffff");
        }
        value *= 85;
        let index = string_[char_nbr += 1] - 32;
        if (index >= mem::size_of::<decoder>()) {
            //  Invalid z85 encoding, character outside range
            // goto error_inval;
            bail!("Invalid z85 encoding, character outside range")
        }
        let summand = decoder[index];
        if (summand == 0xFF || summand > (UINT32_MAX - value)) {
            //  Invalid z85 encoding, invalid character or represented value exceeds 0xffffffff
            // goto error_inval;
            bail!("Invalid z85 encoding, invalid character or represented value exceeds 0xffffffff")
        }
        value += summand;
        if (char_nbr % 5 == 0) {
            //  Output value in base 256
            let mut divisor = 256 * 256 * 256;
            while (divisor) {
                dest_[byte_nbr += 1] = value / divisor % 256;
                divisor /= 256;
            }
            value = 0;
        }
    }
    if (char_nbr % 5 != 0) {
        // goto error_inval;
        bai!("invalid")
    }
    // assert (byte_nbr == strlen (string_) * 4 / 5);
    return dest_;

    // error_inval:
    //     errno = EINVAL;
    //     return null_mut();
}

pub fn zmq_curve_keypair(z85_public_key_: &mut [u8], z85_secret_key_: &mut [u8]) -> i32 {
    // #if defined(ZMQ_HAVE_CURVE)
    // #if CRYPTO_BOX_PUBLICKEYBYTES != 32 || CRYPTO_BOX_SECRETKEYBYTES != 32
    // #error "CURVE encryption library not built correctly"
    // #endif

    let mut public_key: [u8; 32] = [0; 32];
    let mut secret_key: [u8; 32] = [0; 32];

    random_open();

    let res: i32 = crypto_box_keypair(public_key, secret_key);
    zmq_z85_encode(z85_public_key_, &public_key, 32);
    zmq_z85_encode(z85_secret_key_, &secret_key, 32);

    random_close();

    return res;
    // #else
    // (void) z85_public_key_, (void) z85_secret_key_;
    // errno = ENOTSUP;
    // return -1;
    // #endif
}

pub fn zmq_curve_public(z85_public_key_: &mut [u8], z85_secret_key_: &str) -> anyhow::Result<()> {
    // #if defined(ZMQ_HAVE_CURVE)
    // #if CRYPTO_BOX_PUBLICKEYBYTES != 32 || CRYPTO_BOX_SECRETKEYBYTES != 32
    // #error "CURVE encryption library not built correctly"
    // #endif

    let mut public_key: [u8; 32] = [0; 32];
    let mut secret_key: [u8; 32] = [0; 32];

    random_open();

    // if (zmq_z85_decode (&mut secret_key, z85_secret_key_) == null_mut())
    //     return -1;
    zmq_z85_decode(&mut secret_key, z85_secret_key_)?;

    // Return codes are suppressed as none of these can actually fail.
    crypto_scalarmult_base(public_key, secret_key);
    zmq_z85_encode(z85_public_key_, &public_key, 32);

    random_close();

    //     return 0;
    // // #else
    //     (void) z85_public_key_, (void) z85_secret_key_;
    //     errno = ENOTSUP;
    //     return -1;
    Ok(())
    // #endif
}
