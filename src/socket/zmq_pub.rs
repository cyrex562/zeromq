use crate::err::ZmqError;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;


// pub struct ZmqPub<'a> {
//     pub xpub: XPub<'a>,
// }
//
// impl ZmqPub {
//     pub unsafe fn new(parent_: &mut ZmqContext, options_: &mut ZmqOptions, tid_: u32, sid_: i32) -> Self {
//         options_.type_ = ZMQ_PUB;
//         Self {
//             xpub: ZmqXPub::new(options_, parent_, tid_, sid_),
//         }
//     }
//
//
// }


 pub fn pub_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    //  Don't delay pipe termination as there is no one
    //  to receive the delimiter.
    pipe_.set_nodelay ();

    socket.xpub_xattach_pipe (pipe_, subscribe_to_all_, locally_initiated_);
}

pub fn pub_xrecv(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> Result<(),ZmqError> {
    unimplemented!()
}

pub fn pub_xhas_in(socket: &mut ZmqSocket) -> bool {
    false
}

pub fn pub_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    unimplemented!()
}

pub fn pub_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn pub_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}

pub fn pub_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    unimplemented!()
}

pub fn pub_xhas_out(socket: &mut ZmqSocket) -> bool {
    unimplemented!()
}

pub fn pub_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
    unimplemented!()
}

pub fn pub_xwrite_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}

pub fn pub_xpipe_terminated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}
