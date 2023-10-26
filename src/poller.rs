

use windows::Win32::Networking::WinSock::FD_SETSIZE;
use crate::select::ZmqSelect;

#[cfg(feature="select")]
pub type ZmqPoller = ZmqSelect;

#[cfg(feature="select")]
pub fn max_fds() -> i32 {
    FD_SETSIZE as i32
}
