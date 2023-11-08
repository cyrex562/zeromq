use crate::ctx::ZmqContext;
use crate::defines::ZMQ_MSG_MORE;
use crate::err::ZmqError;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// pub struct ZmqGather<'a> {
//     pub socket_base: ZmqSocket<'a>,
//     pub _fq: ZmqFairQueue,
// }
//
// impl ZmqGather {
//     pub unsafe fn new(
//         options: &mut ZmqOptions,
//         parent_: &mut ZmqContext,
//         tid_: u32,
//         sid_: i32,
//     ) -> Self {
//         options.type_ = ZMQ_GATHER;
//         Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid_, true),
//             _fq: ZmqFairQueue::new(),
//         }
//     }
// }

pub fn gather_xattach_pipe(
    socket: &mut ZmqSocket,
    pipe_: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool,
) {
    socket.fq.attach(pipe_);
}

pub fn gather_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) -> Result<(),ZmqError> {
    socket.fq.activated(pipe_);
    Ok(())
}

pub fn gather_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.fq.pipe_terminated(pipe_);
}

pub fn gather_xrecv(ctx: &mut ZmqContext, socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    socket.fq.recvpipe(ctx, msg_, None)?;

    // Drop any messages with more flag
    while msg_.flag_set(ZMQ_MSG_MORE) {
        // drop all frames of the current multi-frame message
        socket.fq.recvpipe(ctx, msg_, None)?;

        while msg_.flag_set(ZMQ_MSG_MORE) {
            socket.fq.recvpipe(ctx, msg_, None)?;
        }

        // get the new message
        // if rc == 0 {
        //     socket.fq.recvpipe(ctx, msg_, None)?;
        // }
    }

    return Ok(())
}

pub fn gather_xhas_in(socket: &mut ZmqSocket) -> bool {
    socket.fq.has_in()
}

pub fn gather_xsetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> i32 {
    unimplemented!()
}

pub fn gather_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}
pub fn gather_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}

pub fn gather_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    unimplemented!()
}

pub fn gather_xhas_out(socket: &mut ZmqSocket) -> i32 {
    unimplemented!()
}

pub fn gather_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}
