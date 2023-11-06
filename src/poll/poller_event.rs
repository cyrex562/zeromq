use std::ffi::c_void;
use crate::defines::ZmqFd;

pub struct ZmqPollerEvent {
    pub socket: *mut c_void,
    pub fd: ZmqFd,
    pub user_data: *mut c_void,
    pub events: i16,
}
