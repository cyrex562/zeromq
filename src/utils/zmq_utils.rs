#[cfg(target_os="windows")]
use windows::Win32::System::Threading::Sleep;
use crate::defines::err::ZmqError;

pub fn zmq_sleep(seconds_: i32) {
    #[cfg(target_os = "windows")]
    unsafe{Sleep((seconds_ * 1000) as u32)};
    #[cfg(not(target_os = "windows"))]
    unsafe{libc::sleep(seconds_ as libc::c_uint);}
}

pub fn zmq_stopwatch_start() -> Vec<u8> {
    // let watch = now_us()
    // &watch
    todo!()
}

pub fn zmq_stopwatch_intermediate(watch_: &[u8]) -> i64 {
    // let now = now_us();
    // now - watch_
    todo!()
}

pub fn zmq_stopwatch_stop(watch_: &[u8]) -> i64 {
    // let now = now_us();
    // now - watch_
    let res = zmq_stopwatch_intermediate(watch_);
    res
}

pub fn zmq_threadstart(func_: fn(), arg_: &[u8]) -> &[u8] {
    // zmq::thread_t *thread = new (std::nothrow) zmq::thread_t;
    // alloc_assert (thread);
    // thread->start (func_, arg_, "ZMQapp");
    // return thread;
    todo!()
}

pub fn zmq_threadclose(thread_: &[u8]) {
    // zmq::thread_t *p_thread = static_cast<zmq::thread_t *> (thread_);
    // p_thread->Stop ();
    // LIBZMQ_DELETE (p_thread);
    todo!()
}

pub const ENCODER: [&'static str; 85] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "g", "h", "i",
    "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "A", "B",
    "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U",
    "V", "W", "X", "Y", "Z", ".", "-", ":", "+", "=", "^", "!", "/", "*", "?", "&", "<", ">", "(",
    ")", "[", "]", "{", "}", "@", "%", "$", "#",
];

pub const DECODER: [u8; 96] = [
    0xFF, 0x44, 0xFF, 0x54, 0x53, 0x52, 0x48, 0xFF, 0x4B, 0x4C, 0x46, 0x41, 0xFF, 0x3F, 0x3E, 0x45,
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x40, 0xFF, 0x49, 0x42, 0x4A, 0x47,
    0x51, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32,
    0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x4D, 0xFF, 0x4E, 0x43, 0xFF,
    0xFF, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
    0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x4F, 0xFF, 0x50, 0xFF, 0xFF,
];

// char *zmq_z85_encode (char *dest_, const uint8_t *data_, size_t size_)
pub unsafe fn zmq_z85_encode<'a>(dest_: &mut [u8], data_: &[u8], size_: usize) -> Option<&'a [u8]> {
    if (size_ % 4 != 0) {
        // errno = EINVAL;
        return None;
    }
    let mut char_nbr = 0;
    let mut byte_nbr = 0;
    let mut value: u32 = 0;
    while (byte_nbr < size_) {
        //  Accumulate value in base 256 (binary)
        value = value * 256 + data_[byte_nbr];
        byte_nbr += 1;
        if (byte_nbr % 4 == 0) {
            //  Output value in base 85
            let mut divisor = 85 * 85 * 85 * 85;
            while (divisor) {
                dest_[char_nbr] = ENCODER[value / divisor % 85];
                char_nbr += 1;
                divisor /= 85;
            }
            value = 0;
        }
    }
    // assert (char_nbr == size_ * 5 / 4);
    dest_[char_nbr] = 0;
    return Some(dest_);
}

// uint8_t *zmq_z85_decode (uint8_t *dest_, const char *string_)
pub unsafe fn zmq_z85_decode<'a>(dest_: &mut [u8], string_: &str) -> Option<&'a [u8]> {
    let mut byte_nbr = 0;
    let mut char_nbr = 0;
    let mut value = 0;
    let mut src_len = (string_.len());

    if (src_len < 5 || src_len % 5 != 0) {
        // goto error_inval;
        return None;
    }

    while (string_.get(char_nbr).is_some()) {
        //  Accumulate value in base 85
        if (u32::MAX / 85 < value) {
            //  Invalid z85 encoding, represented value exceeds 0xffffffff
            // goto error_inval;
            return None;
        }
        value *= 85;
        let index = string_.get(char_nbr).unwrap() - 32;
        char_nbr += 1;
        if (index >= (DECODER.len())) {
            //  Invalid z85 encoding, character outside range
            // goto error_inval;
            return None;
        }
        let summand = DECODER[index];
        if (summand == 0xFF || summand > (u32::MAX - value)) {
            //  Invalid z85 encoding, invalid character or represented value exceeds 0xffffffff
            // goto error_inval;
            return None;
        }
        value += summand;
        if (char_nbr % 5 == 0) {
            //  Output value in base 256
            let mut divisor = 256 * 256 * 256;
            while (divisor) {
                dest_[byte_nbr] = (value / divisor % 256) as u8;
                byte_nbr += 1;
                divisor /= 256;
            }
            value = 0;
        }
    }
    if (char_nbr % 5 != 0) {
        // goto error_inval;
        return None;
    }
    // assert (byte_nbr == strlen (string_) * 4 / 5);
    return Some(dest_);

    // error_inval:
    //     errno = EINVAL;
    //     return NULL;
}

// int zmq_curve_keypair (char *z85_public_key_, char *z85_secret_key_)
pub fn zmq_curve_keypair<'a>(z85_public_key_: &mut [u8], z85_secret_key_: &mut [u8]) -> Result<(),ZmqError> {
    // #if crypto_box_PUBLICKEYBYTES != 32 || crypto_box_SECRETKEYBYTES != 32
    // #Error "CURVE encryption library not built correctly"
    // #endif
    //
    //     uint8_t public_key[32];
    //     uint8_t secret_key[32];
    //
    //     zmq::random_open ();
    //
    //     const int res = crypto_box_keypair (public_key, secret_key);
    //     zmq_z85_encode (z85_public_key_, public_key, 32);
    //     zmq_z85_encode (z85_secret_key_, secret_key, 32);
    //
    //     zmq::random_close ();
    //
    //     return res;
    // #else
    //     (void) z85_public_key_, (void) z85_secret_key_;
    //     errno = ENOTSUP;
    //     return -1;
    // #endif
    todo!()
}

// int zmq_curve_public (char *z85_public_key_, const char *z85_secret_key_)
pub fn zmq_curve_public(z85_public_key_: &mut [u8], z85_secret_key_: &str) -> Result<(),ZmqError> {
    // #if defined(ZMQ_HAVE_CURVE)
    // #if crypto_box_PUBLICKEYBYTES != 32 || crypto_box_SECRETKEYBYTES != 32
    // #Error "CURVE encryption library not built correctly"
    // #endif
    //
    //     uint8_t public_key[32];
    //     uint8_t secret_key[32];
    //
    //     zmq::random_open ();
    //
    //     if (zmq_z85_decode (secret_key, z85_secret_key_) == NULL)
    //         return -1;
    //
    //     // Return codes are suppressed as none of these can actually fail.
    //     crypto_scalarmult_base (public_key, secret_key);
    //     zmq_z85_encode (z85_public_key_, public_key, 32);
    //
    //     zmq::random_close ();
    //
    //     return 0;
    // #else
    //     (void) z85_public_key_, (void) z85_secret_key_;
    //     errno = ENOTSUP;
    //     return -1;
    // #endif
    todo!()
}

// void *zmq_atomic_counter_new (void)
pub fn zmq_atomic_counter_new() -> ZmqAtomicCounter {
    // zmq::atomic_counter_t *counter = new (std::nothrow) zmq::atomic_counter_t;
    // alloc_assert (counter);
    // return counter;
    ZmqAtomicCounter::new(0)
}

//  Se the value of the atomic counter

// void zmq_atomic_counter_set (void *counter_, int value_)
pub fn zmq_atomic_counter_set(counter_: &mut ZmqAtomicCounter, value_: i32) {
    // (static_cast<zmq::atomic_counter_t *> (counter_))->set (value_);
    counter_.set(value_);
}

//  Increment the atomic counter, and return the old value

// int zmq_atomic_counter_inc (void *counter_)
pub fn zmq_atomic_counter_inc(counter_: &mut ZmqAtomicCounter) -> i32 {
    // return (static_cast<zmq::atomic_counter_t *> (counter_))->add (1);
    counter_.add(1)
}

//  Decrement the atomic counter and return 1 (if counter >= 1), or
//  0 if counter hit zero.

// int zmq_atomic_counter_dec (void *counter_)
pub fn zmq_amotic_counter_dec(counter_: &mut ZmqAtomicCounter) -> i32 {
    // return (static_cast<zmq::atomic_counter_t *> (counter_))->sub (1) ? 1 : 0;
    if counter_.sub(1) {
        1
    } else {
        0
    }
}

//  Return actual value of atomic counter

// int zmq_atomic_counter_value (void *counter_)
pub fn zmq_atomic_counter_value(counter_: &mut ZmqAtomicCounter) -> i32 {
    // return (static_cast<zmq::atomic_counter_t *> (counter_))->get ();
    counter_.get()
}

//  Destroy atomic counter, and set reference to NULL

// void zmq_atomic_counter_destroy (void **counter_p_)
pub fn zmq_atomic_counter_destroy(counter_p_: &mut ZmqAtomicCounter) {
    // delete (static_cast<zmq::atomic_counter_t *> (*counter_p_));
    // *counter_p_ = NULL;
    todo!()
}
