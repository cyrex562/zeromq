use crate::ctx::ZmqContext;
use crate::defines::ZMQ_PULL;
use crate::fair_queue::ZmqFairQueue;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;

pub struct ZmqPull<'a> {
    pub socket_base: ZmqSocket<'a>,
    pub _fq: ZmqFairQueue,
}

impl ZmqPull {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_PULL;
        Self {
            socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
            _fq: ZmqFairQueue::new(),
        }
    }
    

}


pub unsafe fn pull_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    socket._fq.attach(pipe_)
}

pub fn oull_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    socket._fq.activated(pipe)
}

pub fn pull_xpipe_terminated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    socket._fq.pipe_terminated(pipe)
}

pub unsafe fn pull_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    socket._fq.recv(msg_)
}

pub unsafe fn pull_xhas_in(socket: &mut ZmqSocket) -> bool {
    socket._fq.has_in()
}
