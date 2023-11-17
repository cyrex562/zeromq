use std::ffi::c_void;
use crate::defines::ZmqFd;
use crate::socket::ZmqSocket;

pub struct ZmqPollerEvent<'a> {
    pub socket: &'a mut ZmqSocket<'a>,
    pub fd: ZmqFd,
    pub user_data: Vec<u8>,
    pub events: i16,
}
