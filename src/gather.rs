use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_GATHER};
use crate::fair_queue::ZmqFairQueue;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;

pub struct ZmqGather<'a> {
    pub socket_base: ZmqSocket<'a>,
    pub _fq: ZmqFairQueue,
}

impl ZmqGather {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_GATHER;
        Self {
            socket_base: ZmqSocket::new(parent_, tid_, sid_, true),
            _fq: ZmqFairQueue::new(),
        }
    }


}


pub fn gather_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        socket._fq.attach(pipe_);
}

pub fn gather_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket._fq.activated(pipe_);
}

pub fn gather_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket._fq.pipe_terminated(pipe_);
}

pub fn gather_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    let mut rc = socket._fq.recvpipe (msg_, &mut None);

    // Drop any messages with more flag
    while rc == 0 && msg_.flag_set(MSG_MORE) {
        // drop all frames of the current multi-frame message
        rc = socket._fq.recvpipe (msg_, &mut None);

        while rc == 0 && msg_.flag_set(MSG_MORE) {
            rc = socket._fq.recvpipe(msg_, &mut None);
        }

        // get the new message
        if rc == 0 {
            rc = socket._fq.recvpipe(msg_, &mut None);
        }
    }

    return rc;
}

pub unsafe fn gather_xhas_in(socket: &mut ZmqSocket) -> bool {
    socket._fq.has_in()
}
