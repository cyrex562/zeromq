use std::ffi::c_void;
use crate::defines::ZmqFileDesc;
use crate::socket::ZmqSocket;


#[derive(Default, Debug, Clone)]
pub struct ZmqPollItem<'a> {
    // socket: *mut c_void;
    pub socket: ZmqSocket<'a>,
    // zmq_fd_t fd;
    pub fd: ZmqFileDesc,
    // short events;
    pub events: i16,
    // short revents;
    pub revents: i16,
}
