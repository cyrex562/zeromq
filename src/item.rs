use crate::defines::ZmqFileDesc;
use crate::socket::ZmqSocket;

#[derive(Default, Debug, Clone)]
pub struct ZmqItem {
    // ZmqSocketBase *socket;
    pub socket: Option<ZmqSocket>,
    // ZmqFileDesc fd;
    pub fd: ZmqFileDesc,
    // user_data: *mut c_void;
    pub user_data: Option<Vec<u8>>,
    // short events;
    pub events: i16,
    // #if defined ZMQ_POLL_BASED_ON_POLL
    pollfd_index: i32,
// #endif
}
