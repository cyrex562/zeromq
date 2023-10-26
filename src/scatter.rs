use crate::ctx::ZmqContext;
use crate::defines::ZMQ_SCATTER;
use crate::load_balancer::ZmqLoadBalancer;
use crate::msg::{MSG_MORE, ZmqMsg};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;

pub struct ZmqScatter {
    pub socket_base: ZmqSocketBase,
    pub _lb: ZmqLoadBalancer,
}

impl ZmqScatter {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_SCATTER;
        Self {
            socket_base: ZmqSocketBase::new(parent_, tid_, sid_, false),
            _lb: ZmqLoadBalancer::new(),
        }
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all: bool, locally_initiated_: bool) {
        pipe_.set_nodelay();
        self._lb.attach(pipe_);
    }

    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut ZmqPipe) {
        self._lb.activated(pipe_);
    }

    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        self._lb.terminated(pipe_);
    }

    pub unsafe fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32 {
        //  SCATTER sockets do not allow multipart data (ZMQ_SNDMORE)
        if (msg_.flags () & MSG_MORE) {
            // errno = EINVAL;
            return -1;
        }

        return self._lb.send (msg_);
    }

    // bool zmq::scatter_t::xhas_out ()
    pub unsafe fn xhas_out(&mut self) -> bool
    {
        return self._lb.has_out ();
    }
}
