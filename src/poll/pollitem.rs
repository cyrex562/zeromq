use crate::zmq_draft::zmq_fd_t;
use std::os::raw::{c_short, c_void};

pub struct ZmqPollitem {
    pub socket: *mut c_void,
    pub fd: zmq_fd_t,
    pub events: c_short,
    pub revents: c_short,
}
