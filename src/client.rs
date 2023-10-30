use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_CLIENT};
use crate::err::ZmqError;
use crate::fair_queue::ZmqFairQueue;
use crate::load_balancer::ZmqLoadBalancer;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;

pub struct ZmqClient<'a> {
    pub socket_base: ZmqSocket<'a>,
    pub _fq: ZmqFairQueue<'a>,
    pub _lb: ZmqLoadBalancer,
}

impl ZmqClient {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        let mut out = Self {
            socket_base: ZmqSocket::new(parent_, tid_, sid_, true),
            _fq: ZmqFairQueue::default(),
            _lb: ZmqLoadBalancer::default(),
        };
        options.type_ = ZMQ_CLIENT;
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;
        out
    }
}

pub fn client_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    socket.fq.attach(pipe_);
    socket.lb.attach(pipe_);
}

pub unsafe fn client_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    if msg_.flags() & MSG_MORE != 0 {
        return -1;
    }
    socket.lb.sendpipe(msg_, &mut None)
}

pub unsafe fn client_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    let mut rc = socket.fq.recvpipe(msg_, &mut None);

    while rc == 0 && msg_.flags() & MSG_MORE > 0 {
        rc = socket.fq.recvpipe(msg_, &mut None);
        while rc == 0 && msg_.flags() & MSG_MORE > 0 {
            rc = socket.fq.recvpipe(msg_, &mut None);
        }

        if rc == 0 {
            rc = socket.fq.recvpipe(msg_, &mut None)
        }
    }

    rc
}

pub fn client_xhas_in(socket: &mut ZmqSocket) -> bool {
    socket.fq.has_in()
}

pub  fn client_xhas_out(socket: &mut ZmqSocket) -> bool {
    socket.lb.has_out()
}

pub fn client_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.fq.activated(pipe_)
}

pub fn client_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.lb.activated(pipe_)
}

pub fn client_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.fq.pipe_terminated(pipe_);
    socket.lb.pipe_terminated(pipe_);
}

pub fn client_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    unimplemented!()
}

pub fn client_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn client_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}
