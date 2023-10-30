use std::ffi::c_void;
use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_DEALER, ZMQ_PROBE_ROUTER};
use crate::fair_queue::ZmqFairQueue;
use crate::load_balancer::ZmqLoadBalancer;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;

pub struct ZmqDealer<'a>
{
    pub socket_base: ZmqSocket<'a>,
    pub _fq: ZmqFairQueue,
    pub _lb: ZmqLoadBalancer,
    pub _probe_router: bool,
}

impl ZmqDealer {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self
    {
        options.type_ = ZMQ_DEALER;
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;
        
        Self {
            socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
            _fq: ZmqFairQueue::default(),
            _lb: ZmqLoadBalancer::default(),
            _probe_router: false,
        }
    }


}

pub fn dealer_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool)
{
    if socket.probe_router {
        // msg_t probe_msg;
        let probe_msg = ZmqMsg::new ();
        let rc = probe_msg.init ();
        // errno_assert (rc == 0);

        rc = pipe_.write (&probe_msg);
        // zmq_assert (rc) is not applicable here, since it is not a bug.
        // LIBZMQ_UNUSED (rc);

        pipe_.flush ();

        rc = probe_msg.close ();
        // errno_assert (rc == 0);
    }

    socket.fq.attach (pipe_);
    socket.lb.attach (pipe_);
}

pub fn dealer_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32
{
    let is_int = optvallen_ == 4;
    let mut value: u32 = u32::from_le_bytes(optval_[0..4].try_into().unwrap());

    if option_ == ZMQ_PROBE_ROUTER {
        socket.probe_router = value != 0;
        return 0;
    }
    else {
        return -1;
    }

    socket.xsetsockopt (option_, optval_, optvallen_)
}

pub fn dealer_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    socket.sendpipe(msg_, &mut None)
}

pub fn dealer_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    socket.recvpipe(msg_, &mut None)
}

pub fn dealer_xhas_in(socket: &mut ZmqSocket) -> bool {
    socket.fq.has_in()
}

pub fn dealer_xhas_out(socket: &mut ZmqSocket) -> bool {
    socket.lb.has_out()
}

pub fn dealer_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.fq.activated(pipe_)
}

pub fn dealer_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.lb.activated(pipe_)
}

pub unsafe fn dealer_sendpipe(socket: &mut ZmqSocket, msg_: &mut ZmqMsg, pipe_: &mut Option<&mut ZmqPipe>) -> i32 {
    socket.lb.sendpipe(msg_, pipe_)
}

pub unsafe fn dealer_recvpipe(socket: &mut ZmqSocket, msg_: &mut ZmqMsg, pipe_: &mut Option<&mut ZmqPipe>) -> i32 {
    socket.fq.recvpipe(msg_, pipe_)
}

pub fn dealer_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}
pub fn dealer_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}

pub fn dealer_xpipe_terminated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}
