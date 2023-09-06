

pub const usecs_per_msec: u64 = 1000;
pub const nsecs_per_usec: u64 = 1000;
pub const usecs_per_sec: u64 = 1000000;
pub const clock_precision: u64 = 1000000;

pub struct clock_t
{
    pub last_tsc: u64,
    pub last_time: u64,
}

impl clock_t {
    pub fn new() -> Self {
        Self {
            last_tsc: rdtsc(),
            last_time: now_us() / usecs_per_msec,
        }
    }


    pub fn rdtsc(&mut self) -> u64 {
        // TODO on windows __rdtsc()
        // TODO on win for arm __rdpmccntr64()
        // TODO on win for arm 64 some custom assembly + _ReadStatusReg()
        // TODO on GNU C i386/amd64 asm read rdtsc
        // 
        unimplemented!();
    }

    pub fn now_us(&mut self) -> u64 {
        // TODO get time since system was started in microseconds
        // on windows this function calls QueryPerformanceCounter
        // on linux/osx it queries clock_gettime
        // for posix it uses gettimeofday
        unimplemented!();
    }

    pub fn now_ms(&mut self) -> u64 {
        let tsc = self.rdtsc();

        if !tsc {
            self.now_us() / usecs_per_msec;
        }

        if tsc - self.last_tsc < (clock_precision / 2) && tsc >= self.last_tsc {
            return self.last_time;
        }
    }

}

pub type f_compatible_get_tick_count64 = fn() -> u64;

pub static compatible_get_tick_count64_mutex: Mutex<()> = Mutex::new(());

pub fn compatible_get_tick_count64() -> u64 {
    // let result = GetTickCount64();
    // return result;
    // TODO get number of millis since system started in a platform-independent way
    unimplemented!();
}

pub fn init_compatible_get_tick_count64() -> f_compatible_get_tick_count64 {
    // do nothing?
    unimplemented!();
}