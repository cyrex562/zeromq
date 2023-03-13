use libc::{clock_gettime, CLOCK_MONOTONIC, timespec};

// now() calls used by SystemTime:
// SGX: insecure_time()
// UNIX clock_gettime()
// Darwin: gettimeofday()
// VXWorks: clock_gettime()
// SOLID: SOLID_RTC_ReadTime
// WASI: __wasi_clock_time_get()
// Windows: GetSystemTimePreciseAsFileTime()

pub fn get_cpu_tick_counter() -> anyhow::Result<u64> {
    if cfg!(unix) {
        let mut ts: timespec = timespec{
            tv_sec: 0,
            tv_nsec: 0
        };
        if unsafe { libc::clock_gettime(CLOCK_MONOTONIC, &mut ts) } != 0 {
            return Err(anyhow!("call to clock_gettime failed"));
        }
        let mut out:u64 = (0 + ts.tv_nsec + (ts.tv_sec * 1000000000)) as u64;
        return Ok(out)
    } else if (windows) {
        unimplemented!()
    }
    return Err(anyhow!("unsupported platform"));
}