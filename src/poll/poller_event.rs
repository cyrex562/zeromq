use crate::defines::ZmqFd;
use crate::socket::ZmqSocket;

pub struct ZmqPollerEvent<'a> {
    pub socket: &'a mut ZmqSocket<'a>,
    pub fd: ZmqFd,
    pub user_data: Vec<u8>,
    pub events: i16,
}

impl Default for ZmqPollerEvent<'_> {
    fn default() -> Self {
        Self {
            socket: &mut ZmqSocket::default(),
            fd: ZmqFd::default(),
            user_data: Vec::new(),
            events: 0,
        }
    }
}
