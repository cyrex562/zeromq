use std::ffi::c_void;
use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_DEALER, ZMQ_PROBE_ROUTER};
use crate::fair_queue::ZmqFairQueue;
use crate::load_balancer::ZmqLoadBalancer;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;

pub struct ZmqDealer<'a>
{
    pub socket_base: ZmqSocketBase<'a>,
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
            socket_base: ZmqSocketBase::new(parent_, tid_, sid_, false),
            _fq: ZmqFairQueue::default(),
            _lb: ZmqLoadBalancer::default(),
            _probe_router: false,
        }
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool)
    {
        if self._probe_router {
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

        self._fq.attach (pipe_);
        self._lb.attach (pipe_);
    }

    pub unsafe fn xsetsockopt(&mut self, option_: i32, optval_: &[u8], optvallen_: usize) -> i32
    {
        let is_int = optvallen_ == 4;
        let mut value: u32 = u32::from_le_bytes(optval_[0..4].try_into().unwrap());

        if (option_ == ZMQ_PROBE_ROUTER) {
            self._probe_router = value != 0;
            return 0;
        }
        else {
            return -1;
        }

        // self.socket_base.xsetsockopt (option_, optval_, optvallen_)
    }

    pub unsafe fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32 {
        self.sendpipe(msg_, &mut None)
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        self.recvpipe(msg_, &mut None)
    }

    pub fn xhas_in(&mut self) -> bool {
        self._fq.has_in()
    }

    pub unsafe fn xhas_out(&mut self) -> bool {
        self._lb.has_out()
    }

    pub fn xread_activated(&mut self, pipe_: &mut ZmqPipe) {
        self._fq.activated(pipe_)
    }

    pub fn xwrite_activated(&mut self, pipe_: &mut ZmqPipe) {
        self._lb.activated(pipe_)
    }

    pub unsafe fn sendpipe(&mut self, msg_: &mut ZmqMsg, pipe_: &mut Option<&mut ZmqPipe>) -> i32 {
        self._lb.sendpipe(msg_, pipe_)
    }

    pub unsafe fn recvpipe(&mut self, msg_: &mut ZmqMsg, pipe_: &mut Option<&mut ZmqPipe>) -> i32 {
        self._fq.recvpipe(msg_, pipe_)
    }
}
