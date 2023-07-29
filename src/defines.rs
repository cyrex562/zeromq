#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
pub type zmq_fd_t = u64;
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86")]
pub type zmq_fd_t = u32;
#[cfg(target_os = "linux")]
pub type zmq_fd_t = i32;