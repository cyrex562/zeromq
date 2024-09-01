use crate::defines::err::ZmqError;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// pub struct ZmqPush<'a> {
//     pub socket_base: ZmqSocket<'a>,
//     pub _lb: ZmqLoadBalancer,
// }
//
// impl ZmqPush {
//     pub unsafe fn new(
//         options: &mut ZmqOptions,
//         parent_: &mut ZmqContext,
//         tid_: u32,
//         sid: i32,
//     ) -> Self {
//         options.type_ = ZMQ_PUSH;
//         Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid, false),
//             _lb: ZmqLoadBalancer::new(),
//         }
//     }
// }

pub fn push_xattach_pipe(
    socket: &mut ZmqSocket,
    pipe_: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool,
) {
    pipe_.set_nodelay();
    socket.lb.attach(pipe_);
}

pub fn push_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.lb.activated(pipe_)
}

pub fn push_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.lb.pipe_terminated(pipe_);
}

pub fn push_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    socket.lb.send(msg_)
}

pub fn push_xhas_out(socket: &mut ZmqSocket) -> bool {
    socket.lb.has_out()
}

pub fn push_xsetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> Result<(),ZmqError> {
    unimplemented!()
}

pub fn push_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<Vec<u8>, ZmqError> {
    unimplemented!();
}
pub fn push_xjoin(socket: &mut ZmqSocket, group: &str) -> Result<(),ZmqError> {
    unimplemented!();
}

pub fn push_xrecv(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> Result<(),ZmqError> {
    unimplemented!()
}

pub fn push_xhas_in(socket: &mut ZmqSocket) -> bool {
    unimplemented!()
}

pub fn push_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
    unimplemented!()
}
