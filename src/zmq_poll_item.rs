use std::ffi::c_void;
use crate::zmq_hdr::zmq_fd_t;

#[derive(Default, Debug, Clone)]
pub struct zmq_pollitem_t {
    // socket: *mut c_void;
    pub socket: &mut [u8],
    // zmq_fd_t fd;
    pub fd: zmq_fd_t,
    // short events;
    pub events: i16,
    // short revents;
    pub revents: i16,
}
