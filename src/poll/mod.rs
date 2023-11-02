use crate::defines::ZmqPollFd;
use crate::err::ZmqError;

pub mod i_poll_events;
pub mod select;
pub mod socket_poller;

pub fn zmq_poll_int(poll_fd: ZmqPollFd, nitems_: u32, timeout: u32) -> Result<(), ZmqError> {
    let mut result = 0i32;
    #[cfg(target_os = "windows")]
    {
        // pub unsafe fn WSAPoll(fdarray: *mut WSAPOLLFD, fds: u32, timeout: i32) -> i32
        let fdarray = [poll_fd];
        unsafe {
            result = WSAPoll(fdarray.as_mut_ptr(), nitems_, timeout as i32);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe {
            result = libc::poll(poll_fd.as_ptr(), nitems_, timeout as i32);
        }
    }

    Ok(())
}
