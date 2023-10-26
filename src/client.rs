use crate::ctx::ZmqContext;
use crate::defines::ZMQ_CLIENT;
use crate::fair_queue::ZmqFairQueue;
use crate::load_balancer::ZmqLoadBalancer;
use crate::msg::{MSG_MORE, ZmqMsg};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;

pub struct ZmqClient<'a>
{
    pub socket_base: ZmqSocketBase<'a>,
    pub _fq: ZmqFairQueue,
    pub _lb: ZmqLoadBalancer,
}

impl ZmqClient {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        let mut out = Self {
            socket_base: ZmqSocketBase::new(parent_, tid_, sid_, true),
            _fq: ZmqFairQueue::default(),
            _lb: ZmqLoadBalancer::default(),
        };
        options.type_ = ZMQ_CLIENT;
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;
        out
    }

    pub fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool)
    {
        self._fq.attach(pipe_);
        self._lb.attach(pipe_);
    }

    pub unsafe fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        if msg_.flags() & MSG_MORE != 0 {
            return -1;
        }
        self._lb.sendpipe(msg_, &mut None)
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        let mut rc = self._fq.recvpipe(msg_, &mut None);

        while rc == 0 && msg_.flags() & MSG_MORE > 0 {
            rc = self._fq.recvpipe(msg_, &mut None) ;
            while rc == 0 && msg_.flags() & MSG_MORE > 0 {
                rc = self._fq.recvpipe(msg_, &mut None);
            }

            if rc == 0 {
                rc = self._fq.recvpipe(msg_, &mut None)
            }
        }

        rc
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

    pub fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        self._fq.pipe_terminated(pipe_);
        self._lb.pipe_terminated(pipe_);
    }
}
