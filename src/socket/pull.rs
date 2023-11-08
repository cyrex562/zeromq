use crate::ctx::ZmqContext;
use crate::err::ZmqError;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// pub struct ZmqPull<'a> {
//     pub socket_base: ZmqSocket<'a>,
//     pub _fq: ZmqFairQueue,
// }
//
// impl ZmqPull {
//     pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
//         options.type_ = ZMQ_PULL;
//         Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
//             _fq: ZmqFairQueue::new(),
//         }
//     }
//
//
// }

pub fn pull_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    unimplemented!()
}

pub fn pull_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    socket.fq.attach(pipe_)
}

pub fn oull_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
    socket.fq.activated(pipe)
}

pub fn pull_xpipe_terminated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    socket.fq.pipe_terminated(pipe)
}

pub fn pull_xrecv(ctx: &mut ZmqContext, socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    socket.fq.recv(ctx, msg_)
}

pub  fn pull_xhas_in(socket: &mut ZmqSocket) -> bool {
    socket.fq.has_in()
}


pub fn pull_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn pull_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}

pub fn pull_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    unimplemented!()
}

pub fn pull_xhas_out(socket: &mut ZmqSocket) -> bool {
    unimplemented!()
}

pub fn pull_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
    unimplemented!()
}

pub fn pull_xwrite_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}
