use crate::ctx::ZmqContext;
use crate::defines::ZMQ_PUB;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::xpub::ZmqXPub;

pub struct ZmqPub<'a> {
    pub xpub: ZmqXPub<'a>,
}

impl ZmqPub {
    pub unsafe fn new(parent_: &mut ZmqContext, options_: &mut ZmqOptions, tid_: u32, sid_: i32) -> Self {
        options_.type_ = ZMQ_PUB;
        Self {
            xpub: ZmqXPub::new(options_, parent_, tid_, sid_),
        }
    }
    
    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        //  Don't delay pipe termination as there is no one
        //  to receive the delimiter.
        pipe_.set_nodelay ();
    
        self.xpub.xattach_pipe (pipe_, subscribe_to_all_, locally_initiated_);
    }
    
    pub fn xrecv(&mut self, msg: &mut ZmqMsg) -> i32 {
        -1
    }
    
    pub fn xhas_in(&mut self) -> bool {
        false
    }
}
