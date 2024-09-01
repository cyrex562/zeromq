use std::ffi::c_uint;

pub fn seed_random() {
    unsafe {
        libc::srand(libc::time(0 as *mut libc::time_t) as c_uint)
    };
}

pub fn generate_random() -> u32 {
    let mut low = unsafe { libc::rand() } as u32;
    let mut high = unsafe { libc::rand() } as u32;
    high <<= 31; // 4 * 8 - 1
    high |= low;
    high
}

pub fn manage_random(init_: bool) {
    #[cfg(feature = "sodium")]
    {
        if init_ {
            sodium_init();
        }
        #[cfg(feature = "sodium_randombytes_close")]
        {
            if init_ == false {
                sodium_randombytes_close();
            }
        }
    }
}

pub fn random_open() {
    manage_random(true)
}

pub fn random_close() {
    manage_random(false)
}
