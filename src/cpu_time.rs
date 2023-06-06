use anyhow::anyhow;
#[cfg(target_os="unix")]
use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};

// now() calls used by SystemTime:
// SGX: insecure_time()
// UNIX clock_gettime()
// Darwin: gettimeofday()
// VXWorks: clock_gettime()
// SOLID: SOLID_RTC_ReadTime
// WASI: __wasi_clock_time_get()
// Windows: GetSystemTimePreciseAsFileTime()

pub fn get_cpu_tick_counter() -> anyhow::Result<u64> {
    #[cfg(target_os="unix")]
    return get_cpu_tick_counter_nix();
    #[cfg(target_os="windows")]
    return get_cpu_tick_counter_win();

}

#[cfg(target_os="unix")]
pub fn get_cpu_tick_counter_nix() -> anyhow::Result<u4> {
    let mut ts: timespec = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    if unsafe { libc::clock_gettime(CLOCK_MONOTONIC, &mut ts) } != 0 {
        return Err(anyhow!("call to clock_gettime failed"));
    }
    let mut out: u64 = (0 + ts.tv_nsec + (ts.tv_sec * 1000000000)) as u64;
    return Ok(out);
}

#[cfg(target_os="windows")]
pub fn get_cpu_tick_counter_win() -> anyhow::Result<u64>
{
    unimplemented!()
}