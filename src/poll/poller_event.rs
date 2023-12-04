use crate::defines::ZmqFd;
use crate::socket::ZmqSocket;

pub struct ZmqPollerEvent<'a> {
    pub socket: Option<&'a mut ZmqSocket<'a>>,
    pub fd: ZmqFd,
    pub user_data: Vec<u8>,
    pub events: u32,
}

impl Default for ZmqPollerEvent<'_> {
    fn default() -> Self {
        Self {
            socket: None,
            fd: ZmqFd::default(),
            user_data: Vec::new(),
            events: 0,
        }
    }
}
