use crate::zmq_draft::zmq_fd_t;
use std::os::raw::c_void;

pub struct ZmqPollerEvent {
    pub socket: *mut c_void,
    pub fd: zmq_fd_t,
    pub user_data: *mut c_void,
    pub events: i16,
}
