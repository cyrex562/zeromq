#![allow(non_camel_case_types)]

use windows::Win32::Networking::WinSock::FD_SETSIZE;
use crate::select::select_t;

#[cfg(feature="select")]
pub type poller_t = select_t;

#[cfg(feature="select")]
pub fn max_fds() -> i32 {
    FD_SETSIZE as i32
}