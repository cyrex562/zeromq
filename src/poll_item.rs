use std::ffi::c_void;
use crate::fd::ZmqFileDesc;
use crate::socket::ZmqSocket;


#[derive(Default, Debug, Clone)]
pub struct ZmqPollItem {
    // socket: *mut c_void;
    pub socket: ZmqSocket,
    // zmq_fd_t fd;
    pub fd: ZmqFileDesc,
    // short events;
    pub events: i16,
    // short revents;
    pub revents: i16,
}
