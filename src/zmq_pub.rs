


use crate::message::ZmqMessage;
use crate::socket::ZmqSocket;


// pub struct ZmqPub {
//     pub xpub: XPub,
//     // ZmqPub (ZmqContext *parent_, tid: u32, sid_: i32);
//
//     // ~ZmqPub ();
//
//     //  Implementations of virtual functions from ZmqSocketBase.
//     // void xattach_pipe (pipe: &mut ZmqPipe,
//     //                    bool subscribe_to_all_ = false,
//     //                    bool locally_initiated_ = false);
//
//     // int xrecv (msg: &mut ZmqMessage);
//
//     // bool xhas_in ();
//
//     // ZMQ_NON_COPYABLE_NOR_MOVABLE (pub_t)
// }

pub fn pub_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    //  Messages cannot be received from PUB socket.
    // errno = ENOTSUP; return - 1;
    unimplemented!()
}

// TODO send should call xpub_send()

pub fn pub_xhas_in() -> bool {
    return false;
}