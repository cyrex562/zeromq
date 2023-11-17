#[cfg(target_os="windows")]
use windows::Win32::Networking::WinSock::WSAPoll;
use crate::defines::err::ZmqError;
use crate::defines::ZmqPollFd;
use crate::net::platform_socket::platform_poll;

pub mod select;
pub mod socket_poller;

pub mod poller_base;
pub mod poller_event;
pub mod polling_util;
pub mod pollitem;

pub fn zmq_poll_int(poll_fds: &mut [ZmqPollFd], nitems: u32, timeout: u32) -> Result<(), ZmqError> {
    platform_poll(poll_fds, nitems, timeout)
}
