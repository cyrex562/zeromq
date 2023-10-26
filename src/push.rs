use libc::option;
use crate::ctx::ZmqContext;
use crate::defines::ZMQ_PUSH;
use crate::load_balancer::ZmqLoadBalancer;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;

pub struct ZmqPush<'a> {
    pub socket_base: ZmqSocketBase<'a>,
    pub _lb: ZmqLoadBalancer,
}

impl ZmqPush {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid: i32) -> Self {
        options.type_ = ZMQ_PUSH;
        Self {
            socket_base: ZmqSocketBase::new(parent_, tid_, sid, false),
            _lb: ZmqLoadBalancer::new(),
        }
    }
    
    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        pipe_.set_nodelay();
        self._lb.attach(pipe_);
    }
    
    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut ZmqPipe) {
        self._lb.activated(pipe_)
    }
    
    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        self._lb.pipe_terminated(pipe_);
    }
    
    pub unsafe fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32 {
        self._lb.send(msg_)
    }
    
    pub unsafe fn xhas_out(&mut self) -> bool {
        self._lb.has_out()
    }
}
