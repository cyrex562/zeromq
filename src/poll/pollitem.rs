use std::os::raw::{c_short, c_void};
use crate::defines::ZmqFd;

pub struct ZmqPollitem {
    pub socket: *mut c_void,
    pub fd: ZmqFd,
    pub events: c_short,
    pub revents: c_short,
}
