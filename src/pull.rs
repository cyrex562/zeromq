use crate::ctx::ZmqContext;
use crate::defines::ZMQ_PULL;
use crate::fair_queue::ZmqFairQueue;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;

pub struct ZmqPull<'a> {
    pub socket_base: ZmqSocketBase<'a>,
    pub _fq: ZmqFairQueue,
}

impl ZmqPull {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_PULL;
        Self {
            socket_base: ZmqSocketBase::new(parent_, tid_, sid_, false),
            _fq: ZmqFairQueue::new(),
        }
    }
    
    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        self._fq.attach(pipe_)
    }
    
    pub fn xread_activated(&mut self, pipe: &mut ZmqPipe) {
        self._fq.activated(pipe)
    }
    
    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        self._fq.pipe_terminated(pipe)
    }
    
    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        self._fq.recv(msg_)
    }
    
    pub unsafe fn xhas_in(&mut self) -> bool {
        self._fq.has_in()
    }
}
